#[cfg(feature = "lind_perf")]
pub mod enabled {
    use lind_perf::Counter;

    pub static MAKE_SYSCALL: Counter = Counter::new("threei::make_syscall");
    pub static MAKE_SYSCALL_CHECK_HANDLER_TABLE: Counter =
        Counter::new("threei::make_syscall::check_handler_table");
    pub static CALL_GRATE_FUNC: Counter = Counter::new("threei::_call_grate_func");
    pub static CALL_GRATE_FUNC_GET_RUNTIME_TRAMPOLINE: Counter =
        Counter::new("threei::_call_grate_func::get_runtime_trampoline");

    pub static ALL_COUNTERS: &[&Counter] = &[
        &MAKE_SYSCALL,
        &MAKE_SYSCALL_CHECK_HANDLER_TABLE,
        &CALL_GRATE_FUNC,
        &CALL_GRATE_FUNC_GET_RUNTIME_TRAMPOLINE,
    ];
}
