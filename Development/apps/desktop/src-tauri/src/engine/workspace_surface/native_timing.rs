//! Debug opt-in wall-clock counts for existing Windows workspace operations.
//! No game/PID/HWND, personal data, new worker, timer, IPC or behavior changes.
use std::io;
#[derive(Clone, Copy)]
pub(super) enum Operation { Aperture, Foreground }

#[cfg(debug_assertions)]
mod debug {
    use super::Operation;
    use std::{sync::{OnceLock, atomic::{AtomicU64, Ordering}}, time::{Duration, Instant}};
    struct Counters {
        calls: AtomicU64, errors: AtomicU64, total_us: AtomicU64,
        max_us: AtomicU64, slow_5ms: AtomicU64, slow_50ms: AtomicU64,
    }
    #[derive(Debug, PartialEq, Eq)]
    struct Summary {
        calls: u64, errors: u64, total_us: u64,
        max_us: u64, slow_5ms: u64, slow_50ms: u64,
    }
    impl Counters {
        const fn new() -> Self {
            Self {
                calls: AtomicU64::new(0), errors: AtomicU64::new(0),
                total_us: AtomicU64::new(0), max_us: AtomicU64::new(0),
                slow_5ms: AtomicU64::new(0), slow_50ms: AtomicU64::new(0),
            }
        }
        fn record(&self, micros: u64, error: bool) {
            self.total_us.fetch_add(micros, Ordering::Relaxed);
            self.max_us.fetch_max(micros, Ordering::Relaxed);
            if error { self.errors.fetch_add(1, Ordering::Relaxed); }
            if micros >= 5_000 { self.slow_5ms.fetch_add(1, Ordering::Relaxed); }
            if micros >= 50_000 { self.slow_50ms.fetch_add(1, Ordering::Relaxed); }
            self.calls.fetch_add(1, Ordering::Relaxed);
        }
        fn take(&self) -> Summary {
            Summary {
                calls: self.calls.swap(0, Ordering::Relaxed),
                errors: self.errors.swap(0, Ordering::Relaxed),
                total_us: self.total_us.swap(0, Ordering::Relaxed),
                max_us: self.max_us.swap(0, Ordering::Relaxed),
                slow_5ms: self.slow_5ms.swap(0, Ordering::Relaxed),
                slow_50ms: self.slow_50ms.swap(0, Ordering::Relaxed),
            }
        }
    }
    static APERTURE: Counters = Counters::new();
    static FOREGROUND: Counters = Counters::new();
    static ENABLED: OnceLock<bool> = OnceLock::new();

    pub(super) fn enabled() -> bool {
        *ENABLED.get_or_init(|| std::env::var_os("MIVUBI_DEBUG_WORKSPACE_TIMING")
            .as_deref() == Some(std::ffi::OsStr::new("1")))
    }
    fn counters(op: Operation) -> &'static Counters {
        match op {
            Operation::Aperture => &APERTURE,
            Operation::Foreground => &FOREGROUND,
        }
    }
    pub(super) fn observe<T>(op: Operation, call: impl FnOnce() -> io::Result<T>) -> io::Result<T> {
        let start = Instant::now();
        let result = call();
        let micros = start.elapsed().as_micros().min(u64::MAX as u128) as u64;
        counters(op).record(micros, result.is_err());
        result
    }
    pub(super) fn report() {
        if !enabled() { return; }
        // Summary only at an existing workspace release, outside SURFACE
        // lock. No per-event console I/O or any identifier is emitted.
        for (operation, data) in [
            ("aperture_repaint", &APERTURE),
            ("foreground_confirmation", &FOREGROUND),
        ] {
            let samples = data.take();
            if samples.calls == 0 { continue; }
            eprintln!(
                "LAZY_TESTER_NATIVE_TIMING operation={operation} calls={} errors={} avg_us={} max_us={} over_5ms={} over_50ms={}",
                samples.calls, samples.errors, samples.total_us / samples.calls,
                samples.max_us, samples.slow_5ms, samples.slow_50ms,
            );
        }
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn counters_report_aggregate_latency_without_native_game_or_window() {
            let stats = Counters::new();
            stats.record(800, false);
            stats.record(7_500, true);
            stats.record(50_100, false);
            assert_eq!(stats.take(), Summary {
                calls: 3, errors: 1, total_us: 58_400,
                max_us: 50_100, slow_5ms: 2, slow_50ms: 1,
            });
            assert_eq!(stats.take().calls, 0);
        }
    }
}

pub(super) fn measure<T>(op: Operation, call: impl FnOnce() -> io::Result<T>) -> io::Result<T> {
    #[cfg(debug_assertions)]
    if debug::enabled() { return debug::observe(op, call); }
    let _ = op;
    call()
}
pub(super) fn report() {
    #[cfg(debug_assertions)]
    debug::report();
}
