/// lind-perf related feature modules.
#[cfg(feature = "lind_perf")]
pub mod enabled {
    use lind_perf::Counter;

    /// Define a counter for close_virtualfd
    pub static CLOSE_VIRTUALFD: Counter = Counter::new("fdtables::close_virtualfd");

    /// Define a list of all counters
    pub static ALL_COUNTERS: &[&Counter] = &[&CLOSE_VIRTUALFD];

    /// Reset all counters in this module to 0
    pub fn reset_all() {
        lind_perf::reset_all(ALL_COUNTERS);
    }

    /// Print a report for all counters
    pub fn report() {
        lind_perf::report_header(Some(format!("FDTABLES")));
        lind_perf::report(ALL_COUNTERS);
    }
}
