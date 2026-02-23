mod cli;
mod lind_wasmtime;
mod perf;

use crate::{
    cli::CliOptions,
    lind_wasmtime::{execute_wasmtime, precompile_module},
};
use clap::Parser;
use lind_perf::TimerKind;
use rawposix::init::{rawposix_shutdown, rawposix_start};

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

    #[cfg(feature = "lind_perf")]
    {
        let kind = if lindboot_cli.perftsc {
            Some(TimerKind::Rdtsc)
        } else if lindboot_cli.perf {
            Some(TimerKind::Clock)
        } else {
            None
        };

        match kind {
            Some(k) => {
                perf::enabled::init(k);

                for name in perf::enabled::all_counter_names() {
                    perf::enabled::enable_one(name);

                    rawposix_start(0);

                    let _ = execute_wasmtime(lindboot_cli.clone());

                    rawposix_shutdown();
                }
                perf::enabled::report();

                return Ok(());
            }
            None => {}
        };
    }

    // AOT-compile only �~@~T no runtime needed
    if lindboot_cli.precompile {
        precompile_module(&lindboot_cli)?;
        return Ok(());
    }

    // Initialize RawPOSIX and register RawPOSIX syscalls with 3i
    rawposix_start(0);

    // Execute with user-selected runtime. Can be switched to other runtime implementation
    // in the future (e.g.: MPK).
    let run_result = execute_wasmtime(lindboot_cli.clone());

    // after all cage exits, finalize the lind
    rawposix_shutdown();

    run_result?;

    Ok(())
}
