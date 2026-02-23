#[cfg(feature = "lind_perf")]
pub mod enabled {
    use lind_perf::Counter;

    pub static ADD_TO_LINKER_MAKE_SYSCALL: Counter =
        Counter::new("lind_common::add_to_linker::make-syscall");

    pub static ALL_COUNTERS: &[&Counter] = &[&ADD_TO_LINKER_MAKE_SYSCALL];
}
