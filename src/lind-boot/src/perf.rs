/// lind-boot's perf file binds together every other module's perf file.
///
/// This involves:
/// - Reading their COUNTERS
/// - Initializing them
/// - Combining all the COUNTERS into one list to iterate over and sequentially enable
/// - Printing a combined lind-perf report.
#[cfg(feature = "lind_perf")]
pub mod enabled {
    use lind_perf::{Counter, TimerKind, enable_name, reset_all, set_timer};

    // These are counters defined within lind-boot.
    pub static READ_WASM_OR_CWASM: Counter = Counter::new("lind_boot::read_wasm_or_cwasm");
    pub static LOAD_MAIN_MODULE: Counter = Counter::new("lind_boot::load_main_module");
    pub static INVOKE_FUNC: Counter = Counter::new("lind_boot::invoke_func");
    pub static GRATE_CALLBACK_TRAMPOLINE: Counter =
        Counter::new("lind_boot::grate_callback_trampoline");
    pub static TRAMPOLINE_GET_VMCTX: Counter = Counter::new("lind_boot::trampoline::get_vmctx");
    pub static TRAMPOLINE_CALLER_WITH: Counter =
        Counter::new("lind_boot::trampoline::Caller::with");
    pub static TRAMPOLINE_GET_PASS_FPTR_TO_WT: Counter =
        Counter::new("lind_boot::trampoline::get_pass_fptr_to_wt");
    pub static TRAMPOLINE_TYPED_DISPATCH_CALL: Counter =
        Counter::new("lind_boot::trampoline::typed_dispatch_call");

    pub static LIND_BOOT_COUNTERS: &[&Counter] = &[
        &READ_WASM_OR_CWASM,
        &LOAD_MAIN_MODULE,
        &INVOKE_FUNC,
        &GRATE_CALLBACK_TRAMPOLINE,
        &TRAMPOLINE_GET_VMCTX,
        &TRAMPOLINE_CALLER_WITH,
        &TRAMPOLINE_GET_PASS_FPTR_TO_WT,
        &TRAMPOLINE_TYPED_DISPATCH_CALL,
    ];

    /// Initialize counters for all modules, involves setting the TimerKind and resetting the
    /// counts.
    pub fn init(kind: TimerKind) {
        set_timer(LIND_BOOT_COUNTERS, kind);
        set_timer(wasmtime_lind_common::perf::enabled::ALL_COUNTERS, kind);
        set_timer(rawposix::perf::enabled::ALL_COUNTERS, kind);
        set_timer(threei::perf::enabled::ALL_COUNTERS, kind);
        set_timer(fdtables::perf::enabled::ALL_COUNTERS, kind);

        reset_all(LIND_BOOT_COUNTERS);
        reset_all(wasmtime_lind_common::perf::enabled::ALL_COUNTERS);
        reset_all(rawposix::perf::enabled::ALL_COUNTERS);
        reset_all(threei::perf::enabled::ALL_COUNTERS);
        reset_all(fdtables::perf::enabled::ALL_COUNTERS);
    }

    /// Finds a counter by it's name and searches for it across modules to enable it. Disables all
    /// other counters.
    pub fn enable_one(name: &str) {
        enable_name(LIND_BOOT_COUNTERS, name);
        enable_name(wasmtime_lind_common::perf::enabled::ALL_COUNTERS, name);
        enable_name(rawposix::perf::enabled::ALL_COUNTERS, name);
        enable_name(threei::perf::enabled::ALL_COUNTERS, name);
        enable_name(fdtables::perf::enabled::ALL_COUNTERS, name);
    }

    /// Get a list of all counter names.
    pub fn all_counter_names() -> Vec<&'static str> {
        let mut names = Vec::new();
        names.extend(LIND_BOOT_COUNTERS.iter().map(|c| c.name));
        names.extend(
            wasmtime_lind_common::perf::enabled::ALL_COUNTERS
                .iter()
                .map(|c| c.name),
        );
        names.extend(threei::perf::enabled::ALL_COUNTERS.iter().map(|c| c.name));
        names.extend(rawposix::perf::enabled::ALL_COUNTERS.iter().map(|c| c.name));
        names.extend(fdtables::perf::enabled::ALL_COUNTERS.iter().map(|c| c.name));
        names
    }

    /// Print a report for every module
    pub fn report() {
        lind_perf::report_header(format!("LIND-BOOT"));
        lind_perf::report(LIND_BOOT_COUNTERS);

        lind_perf::report_header(format!("LIND-COMMON"));
        lind_perf::report(wasmtime_lind_common::perf::enabled::ALL_COUNTERS);

        lind_perf::report_header(format!("THREE-I"));
        lind_perf::report(threei::perf::enabled::ALL_COUNTERS);

        lind_perf::report_header(format!("RAWPOSIX"));
        lind_perf::report(rawposix::perf::enabled::ALL_COUNTERS);

        lind_perf::report_header(format!("FDTABLES"));
        lind_perf::report(fdtables::perf::enabled::ALL_COUNTERS);
    }
}
