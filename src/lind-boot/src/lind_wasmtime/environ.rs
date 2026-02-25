use wasmtime::{Caller, Linker};
use wasmtime_lind_multi_process::get_memory_base;

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

/// Write a little-endian u32 at `base + offset`.
unsafe fn write_u32(base: *mut u8, offset: usize, val: u32) {
    unsafe {
        std::ptr::copy_nonoverlapping(val.to_le_bytes().as_ptr(), base.add(offset), 4);
    }
}

/// Write `src` bytes at `base + offset`.
unsafe fn write_bytes(base: *mut u8, offset: usize, src: &[u8]) {
    unsafe {
        std::ptr::copy_nonoverlapping(src.as_ptr(), base.add(offset), src.len());
    }
}

/// Register the 4 WASI preview1 functions under `wasi_snapshot_preview1`.
pub fn add_to_linker<T: Clone + Send + Sync + 'static>(
    linker: &mut Linker<T>,
    get_cx: impl Fn(&T) -> &LindEnviron + Send + Sync + Copy + 'static,
) -> anyhow::Result<()> {
    linker.func_wrap(
        "lind",
        "args_sizes_get",
        move |caller: Caller<'_, T>, ptr_argc: i32, ptr_buf_size: i32| -> i32 {
            let cx = get_cx(caller.data());
            let argc = cx.args.len() as u32;
            let buf_size: u32 = cx.args.iter().map(|a| a.len() as u32 + 1).sum();
            let base = get_memory_base(&caller) as *mut u8;
            unsafe {
                write_u32(base, ptr_argc as usize, argc);
                write_u32(base, ptr_buf_size as usize, buf_size);
            }
            0
        },
    )?;

    linker.func_wrap(
        "lind",
        "args_get",
        move |caller: Caller<'_, T>, argv_ptrs: i32, argv_buf: i32| -> i32 {
            let cx = get_cx(caller.data());
            let args: Vec<String> = cx.args.clone();
            let base = get_memory_base(&caller) as *mut u8;
            let mut buf_offset = argv_buf as u32;
            for (i, arg) in args.iter().enumerate() {
                let ptr_slot = argv_ptrs as usize + i * 4;
                let bytes = arg.as_bytes();
                unsafe {
                    write_u32(base, ptr_slot, buf_offset);
                    write_bytes(base, buf_offset as usize, bytes);
                    *base.add(buf_offset as usize + bytes.len()) = 0;
                }
                buf_offset += bytes.len() as u32 + 1;
            }
            0
        },
    )?;

    linker.func_wrap(
        "lind",
        "environ_sizes_get",
        move |caller: Caller<'_, T>, ptr_count: i32, ptr_buf_size: i32| -> i32 {
            let cx = get_cx(caller.data());
            let count = cx.env.len() as u32;
            let buf_size: u32 = cx
                .env
                .iter()
                .map(|(k, v)| k.len() as u32 + 1 + v.len() as u32 + 1)
                .sum();
            let base = get_memory_base(&caller) as *mut u8;
            unsafe {
                write_u32(base, ptr_count as usize, count);
                write_u32(base, ptr_buf_size as usize, buf_size);
            }
            0
        },
    )?;

    linker.func_wrap(
        "lind",
        "environ_get",
        move |caller: Caller<'_, T>, env_ptrs: i32, env_buf: i32| -> i32 {
            let cx = get_cx(caller.data());
            let env: Vec<(String, String)> = cx.env.clone();
            let base = get_memory_base(&caller) as *mut u8;
            let mut buf_offset = env_buf as u32;
            for (i, (key, val)) in env.iter().enumerate() {
                let ptr_slot = env_ptrs as usize + i * 4;
                let entry = format!("{}={}", key, val);
                let bytes = entry.as_bytes();
                unsafe {
                    write_u32(base, ptr_slot, buf_offset);
                    write_bytes(base, buf_offset as usize, bytes);
                    *base.add(buf_offset as usize + bytes.len()) = 0;
                }
                buf_offset += bytes.len() as u32 + 1;
            }
            0
        },
    )?;

    Ok(())
}
