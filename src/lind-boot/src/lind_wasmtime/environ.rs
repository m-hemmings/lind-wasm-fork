use wasmtime::{Caller, Linker};
use wasmtime_lind_multi_process::get_memory_base;

/// Stores argv and environment variables for the guest program. During glibc's
/// `_start()`, the guest calls 4 imported functions to retrieve argc/argv and
/// environ. This struct holds the data those functions serve.
#[derive(Clone)]
pub struct LindEnviron {
    args: Vec<String>,
    env: Vec<(String, String)>,
}

impl LindEnviron {
    /// Build from CLI args and `--env` flags. For `--env FOO=BAR`, the value
    /// is used directly. For `--env FOO` (no `=`), the value is inherited
    /// from the host process via `std::env::var`.
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

/// Register the 4 argv/environ host functions under the `lind` linker module.
///
/// These are called by glibc's `_start()` to initialize `argc`, `argv`, and
/// `environ` before entering `main()`. Each function writes directly into the
/// guest's linear memory at offsets provided by the caller.
pub fn add_to_linker<T: Clone + Send + Sync + 'static>(
    linker: &mut Linker<T>,
    get_cx: impl Fn(&T) -> &LindEnviron + Send + Sync + Copy + 'static,
) -> anyhow::Result<()> {
    // args_sizes_get: writes argc and total argv buffer size (sum of NUL-terminated arg lengths)
    // to the two guest memory offsets provided.
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

    // args_get: writes an array of i32 pointers at argv_ptrs and the NUL-terminated
    // argument strings at argv_buf. The guest pre-allocates both regions using
    // sizes from args_sizes_get.
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

    // environ_sizes_get: writes the number of environment variables and the total
    // buffer size (sum of "KEY=VALUE\0" lengths) to the two guest memory offsets.
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

    // environ_get: writes an array of i32 pointers at env_ptrs and the
    // NUL-terminated "KEY=VALUE" strings at env_buf. Same allocation contract
    // as args_get — guest uses sizes from environ_sizes_get.
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
