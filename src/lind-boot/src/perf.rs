#[cfg(feature = "lind_perf")]
pub mod enabled {
    use lind_perf::Counter;

    pub static READ_WASM_OR_CWASM: Counter = Counter::new("lind_boot::read_wasm_or_cwasm");
    pub static LOAD_MAIN_MODULE: Counter = Counter::new("lind_boot::load_main_module");
    pub static INVOKE_FUNC: Counter = Counter::new("lind_boot::invoke_func");
    pub static GRATE_CALLBACK_TRAMPOLINE: Counter =
        Counter::new("lind_boot::grate_callback_trampoline");
    pub static TRAMPOLINE_GET_VMCTX: Counter =
        Counter::new("lind_boot::trampoline::get_vmctx");
    pub static TRAMPOLINE_CALLER_WITH: Counter =
        Counter::new("lind_boot::trampoline::Caller::with");
    pub static TRAMPOLINE_GET_PASS_FPTR_TO_WT: Counter =
        Counter::new("lind_boot::trampoline::get_pass_fptr_to_wt");
    pub static TRAMPOLINE_TYPED_DISPATCH_CALL: Counter =
        Counter::new("lind_boot::trampoline::typed_dispatch_call");

    pub static ALL_COUNTERS: &[&Counter] = &[
        &READ_WASM_OR_CWASM,
        &LOAD_MAIN_MODULE,
        &INVOKE_FUNC,
        &GRATE_CALLBACK_TRAMPOLINE,
        &TRAMPOLINE_GET_VMCTX,
        &TRAMPOLINE_CALLER_WITH,
        &TRAMPOLINE_GET_PASS_FPTR_TO_WT,
        &TRAMPOLINE_TYPED_DISPATCH_CALL,
    ];

    fn set_timer_for_slice(counters: &[&Counter], kind: lind_perf::TimerKind) {
        for c in counters {
            c.set_timer_kind(kind);
        }
    }

    fn set_timer_for_all(kind: lind_perf::TimerKind) {
        set_timer_for_slice(ALL_COUNTERS, kind);
        set_timer_for_slice(wasmtime_lind_common::perf::enabled::ALL_COUNTERS, kind);
        set_timer_for_slice(threei::perf::enabled::ALL_COUNTERS, kind);
        set_timer_for_slice(rawposix::perf::enabled::ALL_COUNTERS, kind);
        set_timer_for_slice(fdtables::perf::enabled::ALL_COUNTERS, kind);
    }

    fn set_only_in_slice(counters: &[&Counter], name: &str, found: &mut bool) {
        for c in counters {
            if c.name == name {
                c.enable();
                *found = true;
            } else {
                c.disable();
            }
        }
    }

    pub fn enable_only(name: &str) -> bool {
        let mut found = false;
        set_only_in_slice(ALL_COUNTERS, name, &mut found);
        set_only_in_slice(threei::perf::enabled::ALL_COUNTERS, name, &mut found);
        set_only_in_slice(rawposix::perf::enabled::ALL_COUNTERS, name, &mut found);
        set_only_in_slice(fdtables::perf::enabled::ALL_COUNTERS, name, &mut found);
        set_only_in_slice(wasmtime_lind_common::perf::enabled::ALL_COUNTERS, name, &mut found);
        found
    }

    pub fn reset_all() {
        lind_perf::reset_all(ALL_COUNTERS);
        threei::perf::enabled::reset_all();
        rawposix::perf::enabled::reset_all();
        fdtables::perf::enabled::reset_all();
        wasmtime_lind_common::perf::enabled::reset_all();
    }

    pub fn report() {
        lind_perf::report_header(None);

        lind_perf::report_header(Some(format!("LIND-BOOT")));
        lind_perf::report(ALL_COUNTERS);

        wasmtime_lind_common::perf::enabled::report();
        threei::perf::enabled::report();
        rawposix::perf::enabled::report();
        fdtables::perf::enabled::report();
    }

    pub fn all_counter_names() -> Vec<&'static str> {
        let mut names = Vec::new();
        for c in ALL_COUNTERS {
            names.push(c.name);
        }
        for c in wasmtime_lind_common::perf::enabled::ALL_COUNTERS {
            names.push(c.name);
        }
        for c in threei::perf::enabled::ALL_COUNTERS {
            names.push(c.name);
        }
        for c in rawposix::perf::enabled::ALL_COUNTERS {
            names.push(c.name);
        }
        for c in fdtables::perf::enabled::ALL_COUNTERS {
            names.push(c.name);
        }
        names
    }

    pub fn set_timer_source(source: i32) {
        match source {
            1 => set_timer_for_all(lind_perf::TimerKind::Rdtsc),
            _ => set_timer_for_all(lind_perf::TimerKind::Clock),
        }
    }
}

/*
#[cfg(not(feature = "lind_perf"))]
pub mod enabled {
    pub fn enable_all() {}
    pub fn enable_only(_name: &str) -> bool {
        false
    }
    pub fn disable_all() {}
    pub fn reset_all() {}
    pub fn report() {}
    pub fn all_counter_names() -> Vec<&'static str> {
        Vec::new()
    }
    pub fn set_timer_source(_source: i32) {}
}
*/
