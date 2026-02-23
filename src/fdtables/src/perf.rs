/// lind-perf related feature modules.
#[cfg(feature = "lind_perf")]
pub mod enabled {
    use lind_perf::Counter;

    /// Define a counter for close_virtualfd
    pub static CLOSE_VIRTUALFD: Counter = Counter::new("fdtables::close_virtualfd");

    /// Define a list of all counters
    pub static ALL_COUNTERS: &[&Counter] = &[&CLOSE_VIRTUALFD];
}
