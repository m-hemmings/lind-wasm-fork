use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
use std::time::Duration;

struct PrettyDuration(Duration);

impl std::fmt::Display for PrettyDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ns_f = self.0.as_nanos() as f64;

        let format = if ns_f < 1_000.0 {
            format!("{:.3}ns", ns_f)
        } else if ns_f < 1_000_000.0 {
            format!("{:.3}µs", ns_f / 1_000.0)
        } else if ns_f < 1_000_000_000.0 {
            format!("{:.3}ms", ns_f / 1_000_000.0)
        } else {
            format!("{:.3}s", ns_f / 1_000_000_000.0)
        };

        write!(f, "{}", format)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TimerKind {
    Rdtsc = 0,
    Clock = 1,
}

impl TimerKind {
    pub const fn name(self) -> &'static str {
        match self {
            TimerKind::Rdtsc => "RdtscTimer",
            TimerKind::Clock => "ClockTimer",
        }
    }

    pub const fn unit(self) -> &'static str {
        match self {
            TimerKind::Rdtsc => "cycles",
            TimerKind::Clock => "ns",
        }
    }
}

pub struct Counter {
    pub cycles: AtomicU64,
    pub calls: AtomicU64,
    pub name: &'static str,
    pub enabled: AtomicBool,
    timer: AtomicU8,
}

impl Counter {
    pub const fn new(name: &'static str) -> Self {
        Self::new_with_timer(name, default_timer_kind())
    }

    pub const fn new_with_timer(name: &'static str, timer: TimerKind) -> Self {
        Self {
            cycles: AtomicU64::new(0),
            calls: AtomicU64::new(0),
            name,
            enabled: AtomicBool::new(false),
            timer: AtomicU8::new(timer as u8),
        }
    }

    #[inline(always)]
    pub fn start(&self) -> u64 {
        if self.enabled.load(Ordering::Relaxed) {
            read_start(self.timer_kind())
        } else {
            0
        }
    }

    #[inline(always)]
    pub fn record(&self, start: u64) {
        if self.enabled.load(Ordering::Relaxed) {
            let elapsed = read_end(self.timer_kind()).saturating_sub(start);
            self.cycles.fetch_add(elapsed, Ordering::Relaxed);
            self.calls.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[inline(always)]
    pub fn scope(&self) -> Scope<'_> {
        Scope {
            counter: self,
            start: self.start(),
        }
    }

    pub fn enable(&self) {
        self.enabled.store(true, Ordering::Relaxed);
    }

    pub fn disable(&self) {
        self.enabled.store(false, Ordering::Relaxed);
    }

    pub fn reset(&self) {
        self.cycles.store(0, Ordering::Relaxed);
        self.calls.store(0, Ordering::Relaxed);
    }

    pub fn set_timer_kind(&self, kind: TimerKind) {
        self.timer.store(kind as u8, Ordering::Relaxed);
    }

    pub fn timer_kind(&self) -> TimerKind {
        match self.timer.load(Ordering::Relaxed) {
            0 => TimerKind::Rdtsc,
            _ => TimerKind::Clock,
        }
    }
}

pub struct Scope<'a> {
    counter: &'a Counter,
    start: u64,
}

impl Drop for Scope<'_> {
    fn drop(&mut self) {
        self.counter.record(self.start);
    }
}

#[inline(always)]
fn read_start(kind: TimerKind) -> u64 {
    match kind {
        TimerKind::Rdtsc => rdtsc_start_inner(),
        TimerKind::Clock => clock_now(),
    }
}

#[inline(always)]
fn read_end(kind: TimerKind) -> u64 {
    match kind {
        TimerKind::Rdtsc => rdtsc_end_inner(),
        TimerKind::Clock => clock_now(),
    }
}

#[inline(always)]
fn rdtsc_start_inner() -> u64 {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::x86_64::_mm_lfence();
        return core::arch::x86_64::_rdtsc();
    }
    return clock_now();
}

#[inline(always)]
fn rdtsc_end_inner() -> u64 {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        let mut aux = 0u32;
        let tsc = core::arch::x86_64::__rdtscp(&mut aux);
        core::arch::x86_64::_mm_lfence();
        return tsc;
    }
    return clock_now();
}

#[inline(always)]
fn clock_now() -> u64 {
    let mut ts = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    let rc = unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC_RAW, &mut ts) };
    if rc != 0 {
        panic!("Unable to get a CLOCK_MONOTONIC_RAW time. Aborting benchmarks.");
    }
    return (ts.tv_sec as u64)
        .saturating_mul(1_000_000_000)
        .saturating_add(ts.tv_nsec as u64);
}

const fn default_timer_kind() -> TimerKind {
    TimerKind::Clock
}

pub fn reset_all(counters: &[&Counter]) {
    for c in counters {
        c.reset();
    }
}

pub fn set_timer(counters: &[&Counter], kind: TimerKind) {
    for c in counters {
        c.set_timer_kind(kind);
    }
}

pub fn enable_name(counters: &[&Counter], name: &str) {
    for c in counters {
        if c.name == name {
            c.enable();
        } else {
            c.disable();
        }
    }
}

pub fn report_header(header: String) {
    let pad = "-";
    let total = 97 - header.len();
    let left = total / 2;
    let right = total - left;

    println!("\n{}{}{}", pad.repeat(left), header, pad.repeat(right),);
}

pub fn report(counters: &[&Counter]) {
    // Tunable constants
    const NAME_W: usize = 60;
    const CALLS_W: usize = 10;
    const NUM_W: usize = 12;

    let mut rows: Vec<String> = Vec::new();

    for c in counters {
        let calls = c.calls.load(Ordering::Relaxed);
        if calls == 0 {
            continue;
        }

        let cycles = match c.timer_kind() {
            TimerKind::Rdtsc => format!("{:#?}", c.cycles.load(Ordering::Relaxed)),
            TimerKind::Clock => format!(
                "{}",
                PrettyDuration(Duration::from_nanos(c.cycles.load(Ordering::Relaxed)))
            ),
        };

        let avg = match c.timer_kind() {
            TimerKind::Rdtsc => format!("{:#?}", c.cycles.load(Ordering::Relaxed) / calls),
            TimerKind::Clock => format!(
                "{}",
                PrettyDuration(Duration::from_nanos(
                    c.cycles.load(Ordering::Relaxed) / calls
                ))
            ),
        };

        // {:<UNIT_W$}
        rows.push(format!(
            "{:<NAME_W$} {:>CALLS_W$} {:>NUM_W$} {:>NUM_W$}",
            c.name, calls, cycles, avg,
        ));
    }

    if rows.len() == 0 {
        return;
    }

    eprintln!(
        "{:<NAME_W$} {:>CALLS_W$} {:>NUM_W$} {:>NUM_W$}",
        "name", "calls", "total", "avg",
    );

    eprintln!("{}", "-".repeat(NAME_W + CALLS_W + NUM_W * 2 + 3));

    for i in rows {
        eprintln!("{}", i);
    }

    println!("");
}

#[macro_export]
macro_rules! scope {
    ($counter:expr) => {
        let _lind_perf_scope = $counter.scope();
    };
}
