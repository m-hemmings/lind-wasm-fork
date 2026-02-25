use wasmtime::{Caller, Linker};

/// Minimal replacement for wasi-common that provides only the 4 WASI preview1
/// functions our glibc `_start()` needs for argv/environ initialization.
///
/// The guest imports these as `wasi_snapshot_preview1::{args_sizes_get, args_get,
/// environ_sizes_get, environ_get}`.
#[derive(Clone)]
pub struct LindEnviron {
    args: Vec<String>,
    env: Vec<(String, String)>,
}

impl LindEnviron {
    /// Build from CLI options.  `vars` entries with `None` values inherit
    /// the variable from the host process via `std::env::var`.
    pub fn new(args: &[String], vars: &[(String, Option<String>)]) -> Self {
        let env = vars
            .iter()
            .filter_map(|(key, val)| {
                let resolved = match val {
                    Some(v) => v.clone(),
                    None => std::env::var(key).ok()?,
                };
                Some((key.clone(), resolved))
            })
            .collect();
        Self {
            args: args.to_vec(),
            env,
        }
    }

    /// Clone args + env for a forked cage.
    pub fn fork(&self) -> Self {
        self.clone()
    }
}

/// Register the 4 WASI preview1 functions under `wasi_snapshot_preview1`.
pub fn add_to_linker<T: Send + 'static>(
    linker: &mut Linker<T>,
    get_cx: impl Fn(&T) -> &LindEnviron + Send + Sync + Copy + 'static,
) -> anyhow::Result<()> {
    linker.func_wrap(
        "wasi_snapshot_preview1",
        "args_sizes_get",
        move |mut caller: Caller<'_, T>, ptr_argc: i32, ptr_buf_size: i32| -> i32 {
            let cx = get_cx(caller.data());
            let argc = cx.args.len() as u32;
            let buf_size: u32 = cx.args.iter().map(|a| a.len() as u32 + 1).sum();

            let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
            let data = mem.data_mut(&mut caller);
            data[ptr_argc as usize..][..4].copy_from_slice(&argc.to_le_bytes());
            data[ptr_buf_size as usize..][..4].copy_from_slice(&buf_size.to_le_bytes());
            0 // success
        },
    )?;

    linker.func_wrap(
        "wasi_snapshot_preview1",
        "args_get",
        move |mut caller: Caller<'_, T>, argv_ptrs: i32, argv_buf: i32| -> i32 {
            let cx = get_cx(caller.data());
            let args: Vec<String> = cx.args.clone();

            let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
            let data = mem.data_mut(&mut caller);

            let mut buf_offset = argv_buf as u32;
            for (i, arg) in args.iter().enumerate() {
                // Write pointer to this arg's string data
                let ptr_slot = argv_ptrs as usize + i * 4;
                data[ptr_slot..][..4].copy_from_slice(&buf_offset.to_le_bytes());

                // Write the arg string + NUL terminator
                let bytes = arg.as_bytes();
                let dst = buf_offset as usize;
                data[dst..dst + bytes.len()].copy_from_slice(bytes);
                data[dst + bytes.len()] = 0;
                buf_offset += bytes.len() as u32 + 1;
            }
            0
        },
    )?;

    linker.func_wrap(
        "wasi_snapshot_preview1",
        "environ_sizes_get",
        move |mut caller: Caller<'_, T>, ptr_count: i32, ptr_buf_size: i32| -> i32 {
            let cx = get_cx(caller.data());
            let count = cx.env.len() as u32;
            // Each entry is "KEY=VALUE\0"
            let buf_size: u32 = cx
                .env
                .iter()
                .map(|(k, v)| k.len() as u32 + 1 + v.len() as u32 + 1) // +1 for '=', +1 for '\0'
                .sum();

            let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
            let data = mem.data_mut(&mut caller);
            data[ptr_count as usize..][..4].copy_from_slice(&count.to_le_bytes());
            data[ptr_buf_size as usize..][..4].copy_from_slice(&buf_size.to_le_bytes());
            0
        },
    )?;

    linker.func_wrap(
        "wasi_snapshot_preview1",
        "environ_get",
        move |mut caller: Caller<'_, T>, env_ptrs: i32, env_buf: i32| -> i32 {
            let cx = get_cx(caller.data());
            let env: Vec<(String, String)> = cx.env.clone();

            let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
            let data = mem.data_mut(&mut caller);

            let mut buf_offset = env_buf as u32;
            for (i, (key, val)) in env.iter().enumerate() {
                // Write pointer to this entry's string data
                let ptr_slot = env_ptrs as usize + i * 4;
                data[ptr_slot..][..4].copy_from_slice(&buf_offset.to_le_bytes());

                // Write "KEY=VALUE\0"
                let entry = format!("{}={}", key, val);
                let bytes = entry.as_bytes();
                let dst = buf_offset as usize;
                data[dst..dst + bytes.len()].copy_from_slice(bytes);
                data[dst + bytes.len()] = 0;
                buf_offset += bytes.len() as u32 + 1;
            }
            0
        },
    )?;

    Ok(())
}
