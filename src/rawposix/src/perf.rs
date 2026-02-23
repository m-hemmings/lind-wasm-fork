#[cfg(feature = "lind_perf")]
pub mod enabled {
    use lind_perf::Counter;

    pub static FDT_SYSCALL: Counter = Counter::new("rawposix::fdt_syscall");
    pub static LIBC_SYSCALL: Counter = Counter::new("rawposix::libc_syscall");

    pub static ALL_COUNTERS: &[&Counter] = &[&FDT_SYSCALL, &LIBC_SYSCALL];

    pub fn reset_all() {
        lind_perf::reset_all(ALL_COUNTERS);
    }

    pub fn report() {
        lind_perf::report_header(Some(format!("RAWPOSIX")));
        lind_perf::report(ALL_COUNTERS);
    }
}
