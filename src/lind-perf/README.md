## lind-perf

### Motivation & Internals
`lind-perf` is a microbenchmark helper for hot paths in Lind (syscalls, trampolines, fd tables).
Examples include `make_syscall()` and `grate_callback_trampoline()`.

Each benchmark site is defined as a `Counter`, which tracks:
- total time spent across invocations
- total number of calls

Timing uses Rust scopes:
- At the top of a function, create a scope guard (`Counter::scope()`).
- The guard records the start time immediately.
- When the function returns, the guard is dropped and records the end time.
- The elapsed time is added to the counter total and the call count increments.

Because the guard records on drop, it must remain alive until the end of the function body.
If a function returns early through `return foo(...)` expressions, the guard can be dropped
*before* the call is evaluated. The fix is to keep the guard alive until after the work:

```
let _scope = perf::enabled::YOUR_COUNTER.scope();
let ret = (|| { ...work... })();
std::hint::black_box(&_scope);
ret
```

To avoid the overhead of multiple counters running at the same time, the benchmark module
is run multiple times, once with each counter exclusively enabled.

With `--perf` flags, lind-boot does the following to run the benchmarks:
```
for counter C in All_Available_Counters:
    disable all counters.
    enable C.
    run wasm module

Emit Report
```

Timer backends:
- `TimerKind::Clock` uses `clock_gettime(CLOCK_MONOTONIC_RAW)` and reports in nanoseconds.
- `TimerKind::Rdtsc` uses `rdtsc/rdtscp` and reports in cycles (x86_64 only).

### Building
- Build with the `lind_perf` feature enabled in the binary crate that uses it.

### Using
1. Create counters in a module-local `perf.rs` and expose `ALL_COUNTERS`.
2. In `lind-boot`, include that module’s counters in:
- enumeration (for “one counter at a time” runs)
- enable/reset/report

### Adding a new timing point
1. Define a new `Counter` and add it to `ALL_COUNTERS`.
2. Add a scope guard at the start of the function.
3. If the function has multiple return paths, keep the guard alive until the end:

```
let _scope = perf::enabled::YOUR_COUNTER.scope();
let ret = (|| {
    // measured work
    ...
})();
std::hint::black_box(&_scope);
ret
```

### Adding a new module
1. Add a `perf.rs` with counters and `ALL_COUNTERS`.
2. Update `lind-boot` to include those counters for enumeration, enable/reset, and reporting.
