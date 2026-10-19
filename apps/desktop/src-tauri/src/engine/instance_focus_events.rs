//! Opt-in observer of Windows foreground-change EVENT COUNTS, not Minecraft.
//! There is exactly one hook thread per desktop manager, no polling/daemon,
//! and no process, window title, PID, credentials or FPS are retained.
//! The hook cannot authorize a game launch, window attribution or frame cap.
use serde::Serialize;
use std::io;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ForegroundEventObserverStatus {
    pub schema: u32,
    pub mode: &'static str,
    pub enabled: bool,
    pub event_count: u64,
}

fn status(enabled: bool, events: u64) -> ForegroundEventObserverStatus {
    ForegroundEventObserverStatus {
        schema: 1,
        mode: "OBSERVE_ONLY",
        enabled,
        // Keep JSON numeric transport exactly representable in JS.
        event_count: events.min(9_007_199_254_740_991),
    }
}

#[cfg(windows)]
mod windows {
    use super::{status, ForegroundEventObserverStatus};
    use std::{
        io,
        ptr,
        sync::{
            atomic::{AtomicBool, AtomicU64, Ordering},
            mpsc, Arc, Mutex,
        },
        thread::{self, JoinHandle},
        time::Duration,
    };
    use windows_sys::Win32::{
        Foundation::HWND,
        System::Threading::GetCurrentThreadId,
        UI::{
            Accessibility::{
                SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK,
            },
            WindowsAndMessaging::{
                DispatchMessageW, GetMessageW, PeekMessageW, PostThreadMessageW,
                TranslateMessage, EVENT_SYSTEM_FOREGROUND, WINEVENT_OUTOFCONTEXT,
                WINEVENT_SKIPOWNPROCESS, MSG, PM_NOREMOVE, WM_QUIT,
            },
        },
    };

    struct Worker {
        thread_id: u32,
        alive: Arc<AtomicBool>,
        join: JoinHandle<()>,
    }

    static WORKER: Mutex<Option<Worker>> = Mutex::new(None);
    static EVENTS: AtomicU64 = AtomicU64::new(0);

    // Out-of-context callback runs on the installing worker's message thread.
    // Never inspect or retain HWND/PID/title here; no per-event helper calls.
    unsafe extern "system" fn foreground_event(
        _hook: HWINEVENTHOOK,
        event: u32,
        _hwnd: HWND,
        object_id: i32,
        child_id: i32,
        _event_thread: u32,
        _event_time: u32,
    ) {
        // OBJID_WINDOW and CHILDID_SELF are both zero. Ignore sub-objects.
        if event == EVENT_SYSTEM_FOREGROUND && object_id == 0 && child_id == 0 {
            let _ = EVENTS.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
                Some(n.saturating_add(1))
            });
        }
    }

    fn worker_main(
        ready: mpsc::SyncSender<io::Result<u32>>,
        alive: Arc<AtomicBool>,
    ) {
        // Install, dispatch and UNINSTALL the hook on this same OS thread.
        // PeekMessage creates the thread message queue before start handshake.
        let mut msg: MSG = unsafe { std::mem::zeroed() };
        unsafe { PeekMessageW(&mut msg, ptr::null_mut(), 0, 0, PM_NOREMOVE) };
        let hook = unsafe {
            SetWinEventHook(
                EVENT_SYSTEM_FOREGROUND,
                EVENT_SYSTEM_FOREGROUND,
                ptr::null_mut(),
                Some(foreground_event),
                0,
                0,
                WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
            )
        };
        if hook.is_null() {
            let _ = ready.send(Err(io::Error::last_os_error()));
            return;
        }
        // Publish liveness BEFORE the handshake so a fast second caller
        // cannot mistake a newly installed hook for a stopped worker.
        alive.store(true, Ordering::Release);
        // If caller timed out/dropped the handshake, never orphan a hook.
        if ready.send(Ok(unsafe { GetCurrentThreadId() })).is_ok() {
            loop {
                let result = unsafe { GetMessageW(&mut msg, ptr::null_mut(), 0, 0) };
                if result <= 0 { break; }
                unsafe {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        }
        unsafe { UnhookWinEvent(hook) };
        alive.store(false, Ordering::Release);
    }

    fn lock() -> io::Result<std::sync::MutexGuard<'static, Option<Worker>>> {
        WORKER.try_lock().map_err(|_| io::Error::new(
            io::ErrorKind::WouldBlock, "Foreground observer transition is in progress",
        ))
    }

    pub fn read() -> ForegroundEventObserverStatus {
        match WORKER.lock() {
            Ok(guard) => status(
                guard.as_ref().is_some_and(|worker|
                    worker.thread_id != 0 && worker.alive.load(Ordering::Acquire)),
                EVENTS.load(Ordering::Relaxed),
            ),
            Err(_) => status(false, EVENTS.load(Ordering::Relaxed)),
        }
    }

    pub fn start() -> io::Result<ForegroundEventObserverStatus> {
        let mut guard = lock()?;
        if let Some(worker) = guard.as_ref() {
            if worker.thread_id != 0 && worker.alive.load(Ordering::Acquire) {
                return Err(io::Error::new(io::ErrorKind::AlreadyExists,
                    "Foreground observer is already active"));
            }
            // This covers both a timed-out startup and an exited hook that
            // has not yet unregistered. Never create a replacement meanwhile.
            if !worker.join.is_finished() {
                return Err(io::Error::new(io::ErrorKind::WouldBlock,
                    "Previous foreground hook is still closing; retry after it exits"));
            }
        }
        if let Some(old) = guard.take() {
            old.join.join().map_err(|_| io::Error::other(
                "Previous foreground hook thread terminated unexpectedly"))?;
        }
        EVENTS.store(0, Ordering::Relaxed);
        // Zero-capacity handshake: the hook thread's send succeeds only
        // when this caller actually receives the thread ID. After a timeout
        // a late send must fail and the hook is uninstalled on its own thread.
        let (tx, rx) = mpsc::sync_channel(0);
        let alive = Arc::new(AtomicBool::new(false));
        let worker_alive = Arc::clone(&alive);
        let join = thread::Builder::new()
            .name("mivubi-focus-events".into())
            .spawn(move || worker_main(tx, worker_alive))?;
        let thread_id = match rx.recv_timeout(Duration::from_secs(5)) {
            Ok(Ok(id)) if id != 0 => id,
            Ok(Ok(_)) | Ok(Err(_)) | Err(_) => {
                // Do not detach: a late worker may still be installing its
                // hook. Preserve the only worker ownership until joined.
                // sync_channel(0) guarantees a late send cannot succeed.
                *guard = Some(Worker { thread_id: 0, alive, join });
                return Err(io::Error::new(io::ErrorKind::Other,
                    "Windows focus event hook did not confirm startup"));
            }
        };
        *guard = Some(Worker { thread_id, alive, join });
        Ok(status(true, EVENTS.load(Ordering::Relaxed)))
    }

    pub fn stop() -> io::Result<ForegroundEventObserverStatus> {
        let mut guard = lock()?;
        if let Some(worker) = guard.as_ref() {
            if worker.thread_id == 0 {
                // Startup timed out. A zero-capacity handshake ensures the
                // late worker unhooks itself. Do not join a thread still
                // potentially blocked in the Windows install call.
                if !worker.join.is_finished() {
                    return Err(io::Error::new(io::ErrorKind::WouldBlock,
                        "Timed-out hook is still cleaning up"));
                }
            } else if worker.alive.load(Ordering::Acquire)
                && unsafe { PostThreadMessageW(worker.thread_id, WM_QUIT, 0, 0) } == 0
            {
                // Never discard the worker handle on a refused shutdown.
                return Err(io::Error::last_os_error());
            }
        }
        if let Some(worker) = guard.take() {
            worker.join.join().map_err(|_| io::Error::other(
                "Foreground hook worker exited unexpectedly"))?;
        }
        Ok(status(false, EVENTS.load(Ordering::Relaxed)))
    }
}

pub fn read_observer() -> ForegroundEventObserverStatus {
    #[cfg(windows)]
    { windows::read() }
    #[cfg(not(windows))]
    { status(false, 0) }
}

pub fn start_observer() -> io::Result<ForegroundEventObserverStatus> {
    #[cfg(windows)]
    { windows::start() }
    #[cfg(not(windows))]
    { Err(io::Error::new(io::ErrorKind::Unsupported, "Windows only")) }
}

pub fn stop_observer() -> io::Result<ForegroundEventObserverStatus> {
    #[cfg(windows)]
    { windows::stop() }
    #[cfg(not(windows))]
    { Err(io::Error::new(io::ErrorKind::Unsupported, "Windows only")) }
}

/// Called after the Tauri run loop exits. Windows also removes hooks on
/// process exit, but our normal path explicitly unhooks on its owner thread.
pub fn shutdown() {
    #[cfg(windows)]
    { let _ = windows::stop(); }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn observer_contract_is_count_only_without_window_identity_or_fps() {
        let report = serde_json::to_value(status(false, 7)).unwrap();
        assert_eq!(report["schema"], 1);
        assert_eq!(report["mode"], "OBSERVE_ONLY");
        assert_eq!(report["enabled"], false);
        assert_eq!(report["eventCount"], 7);
        assert_eq!(report.as_object().unwrap().len(), 4);
        for field in ["pid", "windowHandle", "windowTitle", "account", "fps", "gameplay"] {
            assert!(report.get(field).is_none());
        }
    }
    #[test]
    fn observer_is_off_by_default_outside_explicit_user_start() {
        // Pure status projection, no OS hook or timer is installed by tests.
        assert_eq!(status(false, 0).event_count, 0);
        assert!(!status(false, 0).enabled);
    }
}
