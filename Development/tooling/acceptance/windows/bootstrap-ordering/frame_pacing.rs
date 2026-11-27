//! Native frame deadline arithmetic and Windows pacing lifetime.
//! The existing controller remains the sole external policy producer.
const HUNDRED_NS_PER_SECOND: i64 = 10_000_000;
const MAX_FREQUENCY: i64 = i64::MAX / 20_000_000;
const INVALID_DEADLINE: i64 = 1;

fn valid_ceilings(active: u32, background: u32) -> bool {
    (1..=1000).contains(&active) && (1..=active).contains(&background)
}

fn valid_frequency(frequency: i64) -> bool {
    (1..=MAX_FREQUENCY).contains(&frequency)
}

// Zero = release now; negative = relative Windows timer due in 100 ns units;
// positive = invalid input. No catch-up: the next deadline starts at release.
fn frame_deadline(frequency: i64, previous: i64, now: i64, ceiling: u32) -> i64 {
    if !valid_frequency(frequency) || !valid_ceilings(ceiling, 1)
        || previous < 0 || now < previous {
        return INVALID_DEADLINE;
    }
    let interval = (frequency + i64::from(ceiling) - 1) / i64::from(ceiling);
    let elapsed = now - previous;
    if previous == 0 || elapsed >= interval {
        return 0;
    }
    let remaining = interval - elapsed;
    -(remaining * HUNDRED_NS_PER_SECOND + frequency - 1) / frequency
}

#[no_mangle]
pub extern "C" fn lazytester_frame_pacing_valid_ceilings(active: u32, background: u32) -> u32 {
    u32::from(valid_ceilings(active, background))
}

#[no_mangle]
pub extern "C" fn lazytester_frame_pacing_valid_frequency(frequency: i64) -> u32 {
    u32::from(valid_frequency(frequency))
}

#[no_mangle]
pub extern "C" fn lazytester_frame_pacing_deadline(frequency: i64, previous: i64, now: i64, ceiling: u32) -> i64 {
    frame_deadline(frequency, previous, now, ceiling)
}

#[cfg(windows)]
mod windows {
    use super::{frame_deadline, valid_ceilings, valid_frequency};
    use std::{ffi::{c_char, c_void}, ptr, sync::atomic::{AtomicI64, AtomicU32, Ordering}};

    type Handle = *mut c_void;
    const SYNCHRONIZE: u32 = 0x0010_0000;
    const WAIT_TIMEOUT: u32 = 258;
    const INFINITE: u32 = u32::MAX;
    const BACKGROUND: u32 = 0;
    const FOREGROUND: u32 = 1;
    const STOPPED: u32 = 2;

    // Exact Win32 x64 ABI; only the existing documented APIs are used.
    #[link(name = "kernel32")]
    extern "system" {
        fn GetCurrentProcess() -> Handle;
        fn GetProcessId(process: Handle) -> u32;
        fn GetModuleHandleW(name: *const u16) -> Handle;
        fn GetProcAddress(module: Handle, name: *const c_char) -> *mut c_void;
        fn DuplicateHandle(source_process: Handle, source: Handle, target_process: Handle,
            target: *mut Handle, access: u32, inherit: i32, options: u32) -> i32;
        fn CloseHandle(handle: Handle) -> i32;
        fn CreateEventW(attributes: *const c_void, manual: i32, initial: i32, name: *const u16) -> Handle;
        fn CreateWaitableTimerExW(attributes: *const c_void, name: *const u16, flags: u32, access: u32) -> Handle;
        fn SetWaitableTimerEx(timer: Handle, due: *const i64, period: i32, completion: *const c_void,
            argument: *const c_void, wake: *const c_void, tolerance: u32) -> i32;
        fn SetEvent(event: Handle) -> i32;
        fn WaitForSingleObject(handle: Handle, milliseconds: u32) -> u32;
        fn WaitForMultipleObjects(count: u32, handles: *const Handle, all: i32, milliseconds: u32) -> u32;
        fn QueryPerformanceFrequency(value: *mut i64) -> i32;
        fn QueryPerformanceCounter(value: *mut i64) -> i32;
    }

    struct OwnedHandle(Handle);
    impl Drop for OwnedHandle {
        fn drop(&mut self) {
            if !self.0.is_null() { unsafe { CloseHandle(self.0); } }
        }
    }
    pub struct NativePacer {
        mode: AtomicU32,
        changed: OwnedHandle,
        timer: OwnedHandle,
        external: [OwnedHandle; 4],
        frequency: i64,
        previous: AtomicI64,
        active_fps: u32,
        background_fps: u32,
    }

    #[repr(C)]
    pub struct FramePermit {
        outcome: u32, // 0 Frame, 1 Stopped, 2 Error
        ceiling: u32,
        released: i64,
    }
    impl FramePermit {
        fn stopped() -> Self { Self { outcome: 1, ceiling: 0, released: 0 } }
        fn error() -> Self { Self { outcome: 2, ceiling: 0, released: 0 } }
    }
    const _: () = assert!(std::mem::size_of::<FramePermit>() == 16 && std::mem::offset_of!(FramePermit, released) == 8);

    impl NativePacer {
        unsafe fn new(source: [Handle; 4], active_fps: u32, background_fps: u32) -> Option<Self> {
            if !valid_ceilings(active_fps, background_fps) || source.iter().any(|h| (*h as isize) < 0) {
                return None;
            }
            let count = source.iter().filter(|h| !h.is_null()).count();
            if count != 0 && count != 4 { return None; }
            let mut external = std::array::from_fn(|_| OwnedHandle(ptr::null_mut()));
            if count == 4 {
                if GetProcessId(source[3]) == 0 { return None; }
                let module = GetModuleHandleW([107u16,101,114,110,101,108,98,97,115,101,46,100,108,108,0].as_ptr());
                if module.is_null() { return None; }
                let address = GetProcAddress(module, b"CompareObjectHandles\0".as_ptr().cast());
                if address.is_null() { return None; }
                let compare: unsafe extern "system" fn(Handle, Handle) -> i32 = std::mem::transmute(address);
                for i in 0..4 {
                    for j in 0..i {
                        if source[i] == source[j] || compare(source[i], source[j]) != 0 { return None; }
                    }
                    let process = GetCurrentProcess();
                    if DuplicateHandle(process, source[i], process, &mut external[i].0, SYNCHRONIZE, 0, 0) == 0 {
                        return None;
                    }
                }
            }
            let mut frequency = 0;
            if QueryPerformanceFrequency(&mut frequency) == 0 || !valid_frequency(frequency) { return None; }
            let changed = OwnedHandle(CreateEventW(ptr::null(), 0, 0, ptr::null()));
            let timer = OwnedHandle(CreateWaitableTimerExW(ptr::null(), ptr::null(), 2, SYNCHRONIZE | 2));
            if changed.0.is_null() || timer.0.is_null() { return None; }
            Some(Self { mode: AtomicU32::new(BACKGROUND), changed, timer, external, frequency,
                previous: AtomicI64::new(0), active_fps, background_fps })
        }

        fn foreground(&self, active: bool) -> bool {
            if !self.external[3].0.is_null() { return false; }
            let desired = if active { FOREGROUND } else { BACKGROUND };
            let mut expected = self.mode.load(Ordering::Acquire);
            loop {
                if expected == STOPPED { return false; }
                if expected == desired { return true; }
                match self.mode.compare_exchange_weak(expected, desired, Ordering::AcqRel, Ordering::Acquire) {
                    Ok(_) => return unsafe { SetEvent(self.changed.0) != 0 },
                    Err(actual) => expected = actual,
                }
            }
        }
        fn stop(&self) -> bool {
            self.mode.store(STOPPED, Ordering::Release);
            unsafe { SetEvent(self.changed.0) != 0 }
        }
        // Only the original serialized render consumer calls wait. Other
        // callers may publish foreground or stop, never own another timer.
        fn wait(&self) -> FramePermit {
            let waits = [self.external[2].0, self.external[3].0, self.external[1].0, self.changed.0, self.timer.0];
            let external = !self.external[3].0.is_null();
            loop {
                let mut mode = self.mode.load(Ordering::Acquire);
                if mode == STOPPED { return FramePermit::stopped(); }
                if external {
                    for handle in &waits[..2] {
                        match unsafe { WaitForSingleObject(*handle, 0) } {
                            0 => { self.stop(); return FramePermit::stopped(); }
                            WAIT_TIMEOUT => {}
                            _ => return FramePermit::error(),
                        }
                    }
                    mode = match unsafe { WaitForSingleObject(self.external[0].0, 0) } {
                        0 => FOREGROUND,
                        WAIT_TIMEOUT => BACKGROUND,
                        _ => return FramePermit::error(),
                    };
                }
                let ceiling = if mode == FOREGROUND { self.active_fps } else { self.background_fps };
                let mut now = 0;
                if unsafe { QueryPerformanceCounter(&mut now) } == 0 { return FramePermit::error(); }
                let deadline = frame_deadline(self.frequency, self.previous.load(Ordering::Relaxed), now, ceiling);
                if deadline > 0 { return FramePermit::error(); }
                if deadline == 0 {
                    if self.mode.load(Ordering::Acquire) == STOPPED { return FramePermit::stopped(); }
                    self.previous.store(now, Ordering::Relaxed);
                    return FramePermit { outcome: 0, ceiling, released: now };
                }
                if unsafe { SetWaitableTimerEx(self.timer.0, &deadline, 0, ptr::null(), ptr::null(), ptr::null(), 0) } == 0 {
                    return FramePermit::error();
                }
                let slice = if external { &waits[..] } else { &waits[3..] };
                let count = slice.len() as u32;
                if unsafe { WaitForMultipleObjects(count, slice.as_ptr(), 0, INFINITE) } >= count {
                    return FramePermit::error();
                }
            }
        }
    }

    // C++ owns one opaque allocation; destroy only after all callers join.
    #[no_mangle]
    pub unsafe extern "C" fn lazytester_pacer_create(level: Handle, changed: Handle, stop: Handle,
        controller: Handle, active: u32, background: u32) -> *mut NativePacer {
        NativePacer::new([level, changed, stop, controller], active, background)
            .map_or(ptr::null_mut(), |pacer| Box::into_raw(Box::new(pacer)))
    }
    #[no_mangle]
    pub unsafe extern "C" fn lazytester_pacer_destroy(pacer: *mut NativePacer) {
        if !pacer.is_null() { drop(Box::from_raw(pacer)); }
    }
    #[no_mangle]
    pub unsafe extern "C" fn lazytester_pacer_foreground(pacer: *const NativePacer, active: u32) -> u32 {
        u32::from(pacer.as_ref().is_some_and(|p| p.foreground(active != 0)))
    }
    #[no_mangle]
    pub unsafe extern "C" fn lazytester_pacer_stop(pacer: *const NativePacer) -> u32 {
        u32::from(pacer.as_ref().is_some_and(NativePacer::stop))
    }
    #[no_mangle]
    pub unsafe extern "C" fn lazytester_pacer_wait(pacer: *const NativePacer, output: *mut FramePermit) {
        if !output.is_null() { output.write(pacer.as_ref().map_or_else(FramePermit::error, NativePacer::wait)); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_frame_and_no_catch_up() {
        assert_eq!(frame_deadline(10_000_000, 0, 1, 144), 0);
        assert_eq!(frame_deadline(10_000_000, 1, 10_000_001, 144), 0);
        assert_eq!(frame_deadline(10_000_000, 10_000_001, 10_000_002, 144), -69_444);
    }

    #[test]
    fn promotion_and_demotion_recompute_from_last_release() {
        assert_eq!(frame_deadline(10_000_000, 1, 100_001, 144), 0);
        assert_eq!(frame_deadline(10_000_000, 1, 100_001, 10), -900_000);
        assert_eq!(frame_deadline(3, 1, 1, 2), -6_666_667);
    }

    #[test]
    fn invalid_limits_and_clock_are_refused() {
        for (active, background) in [(0, 1), (1001, 1), (60, 0), (60, 61)] {
            assert!(!valid_ceilings(active, background));
        }
        assert!(valid_ceilings(60, 5));
        assert_eq!(frame_deadline(0, 1, 2, 60), INVALID_DEADLINE);
        assert_eq!(frame_deadline(MAX_FREQUENCY + 1, 1, 2, 60), INVALID_DEADLINE);
        assert_eq!(frame_deadline(1000, 2, 1, 60), INVALID_DEADLINE);
        assert_eq!(frame_deadline(1000, 1, 2, 0), INVALID_DEADLINE);
        assert!(frame_deadline(MAX_FREQUENCY, 1, 1, 1) < 0);
    }
}
