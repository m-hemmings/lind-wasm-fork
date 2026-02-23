use crate::perf;
use fdtables;
use sysdefs::constants::err_const::{syscall_error, Errno};
use typemap::datatype_conversion::*;

pub extern "C" fn libc_syscall(
    cageid: u64,
    arg1: u64,
    arg1_cageid: u64,
    arg2: u64,
    arg2_cageid: u64,
    arg3: u64,
    arg3_cageid: u64,
    arg4: u64,
    arg4_cageid: u64,
    arg5: u64,
    arg5_cageid: u64,
    arg6: u64,
    arg6_cageid: u64,
) -> i32 {
    #[cfg(feature = "lind_perf")]
    let _libc_scope = perf::enabled::LIBC_SYSCALL.scope();

    // Validate that each extra argument is unused.
    if !(sc_unusedarg(arg1, arg1_cageid)
        && sc_unusedarg(arg2, arg2_cageid)
        && sc_unusedarg(arg3, arg3_cageid)
        && sc_unusedarg(arg4, arg4_cageid)
        && sc_unusedarg(arg5, arg5_cageid)
        && sc_unusedarg(arg6, arg6_cageid))
    {
        panic!(
            "{}: unused arguments contain unexpected values -- security violation",
            "libc_syscall"
        );
    }

    let ret = (unsafe { libc::geteuid() }) as i32;

    #[cfg(feature = "lind_perf")]
    std::hint::black_box(&_libc_scope);

    ret
}

pub extern "C" fn fdt_syscall(
    cageid: u64,
    vfd_arg: u64,
    vfd_cageid: u64,
    arg2: u64,
    arg2_cageid: u64,
    arg3: u64,
    arg3_cageid: u64,
    arg4: u64,
    arg4_cageid: u64,
    arg5: u64,
    arg5_cageid: u64,
    arg6: u64,
    arg6_cageid: u64,
) -> i32 {
    #[cfg(feature = "lind_perf")]
    let _fdt_scope = perf::enabled::FDT_SYSCALL.scope();

    if !(sc_unusedarg(arg2, arg2_cageid)
        && sc_unusedarg(arg3, arg3_cageid)
        && sc_unusedarg(arg4, arg4_cageid)
        && sc_unusedarg(arg5, arg5_cageid)
        && sc_unusedarg(arg6, arg6_cageid))
    {
        panic!(
            "{}: unused arguments contain unexpected values -- security violation",
            "fdt_syscall"
        );
    }

    match fdtables::close_virtualfd(cageid, vfd_arg) {
        Ok(()) => {
            return 0;
        }
        Err(e) => {
            if e == Errno::EBADFD as u64 {
                return syscall_error(Errno::EBADF, "close", "Bad File Descriptor");
            } else if e == Errno::EINTR as u64 {
                return syscall_error(Errno::EINTR, "close", "Interrupted system call");
            } else {
                return syscall_error(Errno::EIO, "close", "I/O error");
            }
        }
    };

    #[cfg(feature = "lind_perf")]
    std::hint::black_box(&_fdt_scope);
}
