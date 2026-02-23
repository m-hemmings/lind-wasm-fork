mod counter;
mod report;
mod timers;

pub use counter::*;
pub use report::*;
pub use timers::*;

/// Create a scope guard for a counter.
///
/// This macro is a convenience wrapper around `Counter::scope()`.
#[macro_export]
macro_rules! scope {
    ($counter:expr) => {
        let _lind_perf_scope = $counter.scope();
    };
}
