//! Process-local accepted Present rate. No OS handles, allocation or renderer ownership.
#[repr(C)]
pub struct FpsMeterState {
    frequency: i64,
    first: i64,
    last: i64,
    intervals: u64,
    accepted: u64,
    milli_fps: u32,
    valid: u32,
}

impl FpsMeterState {
    fn new(frequency: i64) -> Self {
        Self { frequency: if (1..=1_000_000_000_000).contains(&frequency) { frequency } else { 0 },
            first: 0, last: 0, intervals: 0, accepted: 0, milli_fps: 0, valid: 0 }
    }

    fn record(&mut self, now: i64) {
        if self.frequency == 0 || now <= 0 { return; }
        self.accepted = self.accepted.saturating_add(1);
        let now = now.max(self.last);
        if self.first == 0 || now - self.last > self.frequency * 2 {
            self.first = now;
            self.intervals = 0;
            self.valid = 0;
        } else {
            self.intervals = self.intervals.saturating_add(1);
            let elapsed = now - self.first;
            if elapsed >= (self.frequency + 1) / 2 {
                let rate = 1000.0 * self.intervals as f64 * self.frequency as f64 / elapsed as f64;
                self.milli_fps = rate.min(9_999_999.0) as u32;
                self.valid = 1;
                self.first = now;
                self.intervals = 0;
            }
        }
        self.last = now;
    }

    fn valid_at(&self, now: i64) -> bool {
        self.valid != 0 && now >= self.last && now - self.last <= self.frequency * 2
    }
}

// ABI: C++ owns inline storage and the original SRWLOCK; no allocator,
// Rust runtime thread, callback, or ownership transfer crosses this boundary.
// SAFETY: caller supplies aligned live state, exclusive for writes, shared for reads.
#[no_mangle]
pub unsafe extern "C" fn lazytester_fps_meter_init(state: *mut FpsMeterState, frequency: i64) {
    if !state.is_null() { state.write(FpsMeterState::new(frequency)); }
}

#[no_mangle]
pub unsafe extern "C" fn lazytester_fps_meter_reset(state: *mut FpsMeterState) {
    if let Some(state) = state.as_mut() { *state = FpsMeterState::new(state.frequency); }
}

#[no_mangle]
pub unsafe extern "C" fn lazytester_fps_meter_record(state: *mut FpsMeterState, now: i64) {
    if let Some(state) = state.as_mut() { state.record(now); }
}

#[repr(C)]
pub struct FpsMeterReading {
    accepted: u64,
    milli_fps: u32,
    valid: u32,
}

#[no_mangle]
pub unsafe extern "C" fn lazytester_fps_meter_read(state: *const FpsMeterState, now: i64, output: *mut FpsMeterReading) {
    if output.is_null() { return; }
    let reading = match state.as_ref() {
        Some(s) => FpsMeterReading { accepted: s.accepted, milli_fps: s.milli_fps, valid: u32::from(s.valid_at(now)) },
        None => FpsMeterReading { accepted: 0, milli_fps: 0, valid: 0 },
    };
    output.write(reading);
}

const _: () = assert!(std::mem::size_of::<FpsMeterState>() == 48);
const _: () = assert!(std::mem::size_of::<FpsMeterReading>() == 16);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rate_staleness_and_reset() {
        let mut state = FpsMeterState::new(144000);
        for i in 0..=144 { state.record(1 + i * 1000); }
        assert_eq!((state.accepted, state.milli_fps), (145, 144000));
        assert!(state.valid_at(144001));
        assert!(!state.valid_at(432002));
        state.record(432002);
        assert!(!state.valid_at(432002));
        state = FpsMeterState::new(state.frequency);
        assert_eq!(state.accepted, 0);
        assert!(!state.valid_at(1));
    }
    #[test]
    fn invalid_clock_and_out_of_order_samples() {
        let mut invalid = FpsMeterState::new(0);
        invalid.record(1);
        assert_eq!(invalid.accepted, 0);
        let mut state = FpsMeterState::new(1000);
        state.record(100); state.record(200); state.record(150); state.record(600);
        assert_eq!((state.accepted, state.milli_fps), (4, 6000));
        assert!(!state.valid_at(599));
    }
}
