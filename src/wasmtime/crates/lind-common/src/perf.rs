#[cfg(feature = "lind_perf")]
pub mod enabled {
    use lind_perf::Counter;

    pub static ADD_TO_LINKER_MAKE_SYSCALL: Counter =
        Counter::new("lind_common::add_to_linker::make-syscall");

    pub static ALL_COUNTERS: &[&Counter] = &[&ADD_TO_LINKER_MAKE_SYSCALL];

    pub fn reset_all() {
        lind_perf::reset_all(ALL_COUNTERS);
    }

    pub fn report() {
        lind_perf::report_header(Some(format!("LIND-COMMON")));
        lind_perf::report(ALL_COUNTERS);
    }
}
