mod cli;
mod lind_wasmtime;

use crate::{
    cli::CliOptions,
    lind_wasmtime::{execute_wasmtime, precompile_module},
};
use clap::Parser;
use libc;
use rawposix::init::{rawposix_shutdown, rawposix_start};
use std::ffi::CString;

const LINDFS_ROOT: &'static str = "/home/lind/lind-wasm/lindfs";

/// Helper function which `chroot`s to `lindfs`.
///
/// - mkdir LINDFS_ROOT.
/// - chroots to LINDFS_ROOT
/// - chdirs to new '/'
fn chroot_to_lindfs() {
    unsafe {
        let lindfs_path = CString::new(LINDFS_ROOT).unwrap();
        libc::mkdir(lindfs_path.as_ptr(), 0o775);
        let ret = libc::chroot(lindfs_path.as_ptr());
        if ret != 0 {
            panic!(
                "Failed to chroot to {}: {}",
                LINDFS_ROOT,
                std::io::Error::last_os_error()
            );
        }
        let root = CString::new("/").unwrap();
        let ret = libc::chdir(root.as_ptr());
        if ret != 0 {
            panic!(
                "Failed to chdir to / after chroot: {}",
                std::io::Error::last_os_error()
            )
        }
    }
}

/// Entry point of the lind-boot executable.
///
/// The expected invocation follows: the first non-flag argument specifies the
/// Wasm binary to execute and all remaining arguments are forwarded verbatim to
/// the guest program:
///
///     lind-boot [flags...] wasm_file.wasm arg1 arg2 ...
///
/// All process lifecycle management, runtime initialization, and error
/// handling semantics are delegated to `execute.rs`.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lindboot_cli = CliOptions::parse();

    // AOT-compile only — no runtime needed
    if lindboot_cli.precompile {
        precompile_module(&lindboot_cli)?;
        return Ok(());
    }

    // Not a precompile command, chroot to lindfs
    chroot_to_lindfs();

    // Initialize RawPOSIX and register RawPOSIX syscalls with 3i
    rawposix_start(0);

    // Execute with user-selected runtime. Can be switched to other runtime implementation
    // in the future (e.g.: MPK).
    execute_wasmtime(lindboot_cli)?;

    // after all cage exits, finalize the lind
    rawposix_shutdown();

    Ok(())
}
