//! One foreground-event worker. Public observation remains count-only.
//! Private policy leases pin explicitly supplied processes and publish kernel
//! events; they never launch, authenticate or attribute Minecraft themselves.
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
        os::windows::io::{AsHandle, AsRawHandle, BorrowedHandle, FromRawHandle, OwnedHandle},
        ptr,
        sync::{
            atomic::{AtomicBool, AtomicU64, Ordering},
            mpsc, Arc, Mutex,
        },
        thread::{self, JoinHandle},
        time::Duration,
    };
    use windows_sys::Win32::System::Threading::SYNCHRONIZATION_SYNCHRONIZE as SYNCHRONIZE;
    use windows_sys::Win32::{
        Foundation::{DuplicateHandle, FILETIME, HANDLE, HWND, WAIT_TIMEOUT},
        System::{
            RemoteDesktop::ProcessIdToSessionId,
            Threading::{
                CreateEventW, GetCurrentProcess, GetCurrentProcessId, GetCurrentThreadId,
                GetProcessId, GetProcessTimes, ResetEvent, SetEvent, WaitForSingleObject,
                PROCESS_QUERY_LIMITED_INFORMATION,
            },
        },
        UI::{
            Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK},
            WindowsAndMessaging::{
                DispatchMessageW, GetForegroundWindow, GetMessageW, GetWindowThreadProcessId,
                PeekMessageW, PostThreadMessageW, TranslateMessage, EVENT_SYSTEM_FOREGROUND, MSG,
                PM_NOREMOVE, WINEVENT_OUTOFCONTEXT, WM_APP, WM_QUIT,
            },
        },
    };

    struct Worker {
        thread_id: u32,
        alive: Arc<AtomicBool>,
        // Set only after WM_QUIT has actually entered the worker queue.
        // Never post duplicate shutdown messages on repeated Stop actions.
        stop_requested: bool,
        join: JoinHandle<io::Result<()>>,
    }

    static WORKER: Mutex<Option<Worker>> = Mutex::new(None);
    static EVENTS: AtomicU64 = AtomicU64::new(0);
    const REFRESH_POLICY: u32 = WM_APP + 0x324;
    static POLICIES: Mutex<Vec<Arc<Binding>>> = Mutex::new(Vec::new());
    static PRIVATE_START: AtomicBool = AtomicBool::new(false);

    fn owned(raw: HANDLE) -> io::Result<OwnedHandle> {
        if raw.is_null() {
            Err(io::Error::last_os_error())
        } else {
            Ok(unsafe { OwnedHandle::from_raw_handle(raw as _) })
        }
    }
    fn duplicate(source: BorrowedHandle<'_>, access: u32) -> io::Result<OwnedHandle> {
        let mut raw = ptr::null_mut();
        if unsafe {
            DuplicateHandle(
                GetCurrentProcess(),
                source.as_raw_handle() as _,
                GetCurrentProcess(),
                &mut raw,
                access,
                0,
                0,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        owned(raw)
    }
    fn event(manual: bool) -> io::Result<OwnedHandle> {
        owned(unsafe { CreateEventW(ptr::null(), manual as i32, 0, ptr::null()) })
    }
    fn birth(process: &OwnedHandle) -> io::Result<u64> {
        let (mut created, mut exited, mut kernel, mut user): (
            FILETIME,
            FILETIME,
            FILETIME,
            FILETIME,
        ) = unsafe { std::mem::zeroed() };
        if unsafe {
            GetProcessTimes(
                process.as_raw_handle() as _,
                &mut created,
                &mut exited,
                &mut kernel,
                &mut user,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok((u64::from(created.dwHighDateTime) << 32) | u64::from(created.dwLowDateTime))
    }
    struct Binding {
        process: OwnedHandle,
        pid: u32,
        created: u64,
        level: OwnedHandle,
        changed: OwnedHandle,
        stop: OwnedHandle,
        active: AtomicBool,
        stopped: AtomicBool,
    }
    impl Binding {
        fn new(process: BorrowedHandle<'_>, pid: u32, created: u64) -> io::Result<Self> {
            let process = duplicate(process, SYNCHRONIZE | PROCESS_QUERY_LIMITED_INFORMATION)?;
            let mut session = 0;
            let mut current_session = 0;
            if pid == 0
                || created == 0
                || pid == unsafe { GetCurrentProcessId() }
                || unsafe { GetProcessId(process.as_raw_handle() as _) } != pid
                || birth(&process)? != created
                || unsafe { WaitForSingleObject(process.as_raw_handle() as _, 0) } != WAIT_TIMEOUT
                || unsafe { ProcessIdToSessionId(pid, &mut session) } == 0
                || unsafe { ProcessIdToSessionId(GetCurrentProcessId(), &mut current_session) } == 0
                || session == 0
                || session != current_session
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Policy process identity changed",
                ));
            }
            Ok(Self {
                process,
                pid,
                created,
                level: event(true)?,
                changed: event(false)?,
                stop: event(true)?,
                active: AtomicBool::new(false),
                stopped: AtomicBool::new(false),
            })
        }
        fn live(&self) -> bool {
            (unsafe { WaitForSingleObject(self.process.as_raw_handle() as _, 0) }) == WAIT_TIMEOUT
                && birth(&self.process).ok() == Some(self.created)
        }
        fn finish(&self) {
            self.stopped.store(true, Ordering::Release);
            unsafe {
                ResetEvent(self.level.as_raw_handle() as _);
                SetEvent(self.stop.as_raw_handle() as _);
                SetEvent(self.changed.as_raw_handle() as _);
            }
            self.active.store(false, Ordering::Release);
        }
        fn publish(&self, active: bool) -> io::Result<()> {
            if self.stopped.load(Ordering::Acquire) {
                return Ok(());
            }
            if !self.live() {
                self.finish();
                return Ok(());
            }
            if self.active.load(Ordering::Acquire) == active {
                return Ok(());
            }
            let changed = unsafe {
                if active {
                    SetEvent(self.level.as_raw_handle() as _)
                } else {
                    ResetEvent(self.level.as_raw_handle() as _)
                }
            };
            if changed == 0 || unsafe { SetEvent(self.changed.as_raw_handle() as _) } == 0 {
                let error = io::Error::last_os_error();
                self.finish();
                return Err(error);
            }
            self.active.store(active, Ordering::Release);
            Ok(())
        }
        fn readers(&self) -> io::Result<[OwnedHandle; 4]> {
            let process = unsafe { BorrowedHandle::borrow_raw(GetCurrentProcess() as _) };
            Ok([
                duplicate(self.level.as_handle(), SYNCHRONIZE)?,
                duplicate(self.changed.as_handle(), SYNCHRONIZE)?,
                duplicate(self.stop.as_handle(), SYNCHRONIZE)?,
                duplicate(process, SYNCHRONIZE | PROCESS_QUERY_LIMITED_INFORMATION)?,
            ])
        }
    }
    impl Drop for Binding {
        fn drop(&mut self) {
            self.finish();
        }
    }
    fn publish(bindings: &[Arc<Binding>], selected: Option<u32>) {
        // Demote before promoting; never transiently advertise two active clients.
        for binding in bindings.iter().filter(|p| Some(p.pid) != selected) {
            if binding.publish(false).is_err() {
                for p in bindings {
                    p.finish();
                }
                return;
            }
        }
        for binding in bindings.iter().filter(|p| Some(p.pid) == selected) {
            if binding.publish(true).is_err() {
                for p in bindings {
                    p.finish();
                }
                return;
            }
        }
    }
    fn refresh() {
        let bindings = match POLICIES.lock() {
            Ok(list) => list,
            Err(poisoned) => {
                for p in poisoned.into_inner().iter() {
                    p.finish();
                }
                return;
            }
        };
        if bindings.is_empty() {
            return;
        }
        let window = unsafe { GetForegroundWindow() };
        let mut pid = 0;
        let mut confirmed_pid = 0;
        let stable = !window.is_null()
            && unsafe { GetWindowThreadProcessId(window, &mut pid) } != 0
            && unsafe { GetForegroundWindow() } == window
            && unsafe { GetWindowThreadProcessId(window, &mut confirmed_pid) } != 0
            && pid == confirmed_pid
            && unsafe { GetForegroundWindow() } == window;
        publish(&bindings, if stable { Some(pid) } else { None });
    }
    fn finish_policies() {
        let list = POLICIES
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        for p in list.iter() {
            p.finish();
        }
    }
    /// Internal lease only. No public command or game activation is added.
    pub(crate) struct PolicyChannel {
        binding: Arc<Binding>,
        thread_id: u32,
        alive: Arc<AtomicBool>,
    }
    impl PolicyChannel {
        // Order: foreground level, change notification, sticky stop, manager process.
        pub(crate) fn reader_handles(&self) -> io::Result<[OwnedHandle; 4]> {
            if self.binding.stopped.load(Ordering::Acquire) || !self.binding.live() {
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "Policy lease ended",
                ));
            }
            self.binding.readers()
        }
    }
    impl Drop for PolicyChannel {
        fn drop(&mut self) {
            self.binding.finish();
            {
                let mut list = POLICIES.lock().unwrap_or_else(|p| p.into_inner());
                list.retain(|p| !Arc::ptr_eq(p, &self.binding));
            }
            // The worker decides whether the last private lease also ends its loop.
            if self.alive.load(Ordering::Acquire)
                && unsafe { PostThreadMessageW(self.thread_id, REFRESH_POLICY, 0, 0) } == 0
            {
                eprintln!(
                    "Private focus worker cleanup notification unverified: {}",
                    io::Error::last_os_error()
                );
            }
        }
    }
    pub(crate) fn register_policy(
        process: BorrowedHandle<'_>,
        pid: u32,
        created: u64,
    ) -> io::Result<PolicyChannel> {
        let binding = Arc::new(Binding::new(process, pid, created)?);
        let mut guard = lock()?;
        let active = guard.as_ref().is_some_and(|w| {
            w.alive.load(Ordering::Acquire) && !w.stop_requested && !w.join.is_finished()
        });
        let started = if active {
            false
        } else {
            start_locked(&mut guard)?;
            PRIVATE_START.store(true, Ordering::Release);
            true
        };
        let worker = guard
            .as_ref()
            .ok_or_else(|| io::Error::other("Focus worker unavailable"))?;
        let result = (|| {
            let mut list = POLICIES
                .lock()
                .map_err(|_| io::Error::other("Focus policy lock poisoned"))?;
            if worker.stop_requested
                || !worker.alive.load(Ordering::Acquire)
                || worker.join.is_finished()
            {
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "Focus worker is stopping",
                ));
            }
            if list.len() >= 5
                || list
                    .iter()
                    .any(|p| p.pid == pid || p.stopped.load(Ordering::Acquire))
            {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "Policy slots occupied or previous leases require cleanup",
                ));
            }
            list.push(Arc::clone(&binding));
            if unsafe { PostThreadMessageW(worker.thread_id, REFRESH_POLICY, 0, 0) } == 0 {
                list.pop();
                return Err(io::Error::last_os_error());
            }
            Ok(PolicyChannel {
                binding,
                thread_id: worker.thread_id,
                alive: Arc::clone(&worker.alive),
            })
        })();
        if result.is_err() && started {
            // WORKER remains locked: another registration cannot be stopped here.
            if let Some(worker) = guard.as_mut() {
                if unsafe { PostThreadMessageW(worker.thread_id, WM_QUIT, 0, 0) } != 0 {
                    worker.stop_requested = true;
                }
            }
        }
        result
    }

    // Out-of-context callback runs on the installing worker's message thread.
    // Public mode only counts. Private mode samples transient HWND/PID, never titles.
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
            let _ = EVENTS.try_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
                Some(n.saturating_add(1))
            });
            refresh();
        }
    }

    fn worker_main(
        ready: mpsc::SyncSender<io::Result<u32>>,
        alive: Arc<AtomicBool>,
    ) -> io::Result<()> {
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
                WINEVENT_OUTOFCONTEXT,
            )
        };
        if hook.is_null() {
            let _ = ready.send(Err(io::Error::last_os_error()));
            // Nothing was installed; the error has been sent to the caller.
            return Ok(());
        }
        // Publish liveness BEFORE the handshake so a fast second caller
        // cannot mistake a newly installed hook for a stopped worker.
        alive.store(true, Ordering::Release);
        // If caller timed out/dropped the handshake, never orphan a hook.
        let mut dispatch_error = None;
        if ready.send(Ok(unsafe { GetCurrentThreadId() })).is_ok() {
            loop {
                let result = unsafe { GetMessageW(&mut msg, ptr::null_mut(), 0, 0) };
                if result == 0 {
                    break;
                } // WM_QUIT is an orderly Stop.
                if result == -1 {
                    dispatch_error = Some(io::Error::last_os_error());
                    break;
                }
                if msg.message == REFRESH_POLICY {
                    let list = POLICIES.lock().unwrap_or_else(|p| p.into_inner());
                    if list.is_empty() && PRIVATE_START.swap(false, Ordering::AcqRel) {
                        alive.store(false, Ordering::Release);
                        break;
                    }
                    drop(list);
                    refresh();
                    continue;
                }
                unsafe {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        }
        finish_policies();
        PRIVATE_START.store(false, Ordering::Release);
        // The installing message-loop thread is the sole unhook owner.
        // Do not report a successful Stop if Windows refused UnhookWinEvent.
        let unhook_result = if unsafe { UnhookWinEvent(hook) } == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        };
        alive.store(false, Ordering::Release);
        unhook_result?;
        if let Some(error) = dispatch_error {
            return Err(error);
        }
        Ok(())
    }

    fn lock() -> io::Result<std::sync::MutexGuard<'static, Option<Worker>>> {
        WORKER.try_lock().map_err(|_| {
            io::Error::new(
                io::ErrorKind::WouldBlock,
                "Foreground observer transition is in progress",
            )
        })
    }

    pub fn read() -> io::Result<ForegroundEventObserverStatus> {
        // A poisoned/busy lock or a worker awaiting reap is UNKNOWN, not
        // evidence that the hook has stopped. Caller can retry Stop.
        let guard = lock()?;
        if !POLICIES
            .lock()
            .map_err(|_| io::Error::other("Focus policy lock poisoned"))?
            .is_empty()
        {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "Private policy owns the foreground worker",
            ));
        }
        let active = match guard.as_ref() {
            None => false,
            Some(worker)
                if worker.thread_id != 0
                    && worker.alive.load(Ordering::Acquire)
                    && !worker.join.is_finished() =>
            {
                true
            }
            Some(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "Foreground hook state unverified; Stop/cleanup required",
                ))
            }
        };
        Ok(status(active, EVENTS.load(Ordering::Relaxed)))
    }

    pub fn start() -> io::Result<ForegroundEventObserverStatus> {
        let mut guard = lock()?;
        start_locked(&mut guard)
    }
    fn start_locked(guard: &mut Option<Worker>) -> io::Result<ForegroundEventObserverStatus> {
        if let Some(worker) = guard.as_ref() {
            if worker.thread_id != 0 && worker.alive.load(Ordering::Acquire) {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "Foreground observer is already active",
                ));
            }
            // This covers both a timed-out startup and an exited hook that
            // has not yet unregistered. Never create a replacement meanwhile.
            if !worker.join.is_finished() {
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "Previous foreground hook is still closing; retry after it exits",
                ));
            }
        }
        if let Some(old) = guard.take() {
            old.join.join().map_err(|_| {
                io::Error::other("Previous foreground hook thread terminated unexpectedly")
            })??;
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
                *guard = Some(Worker {
                    thread_id: 0,
                    alive,
                    stop_requested: false,
                    join,
                });
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Windows focus event hook did not confirm startup",
                ));
            }
        };
        *guard = Some(Worker {
            thread_id,
            alive,
            stop_requested: false,
            join,
        });
        Ok(status(true, EVENTS.load(Ordering::Relaxed)))
    }

    pub fn stop() -> io::Result<ForegroundEventObserverStatus> {
        let mut guard = lock()?;
        finish_policies();
        if let Some(worker) = guard.as_mut() {
            // A successful PostThreadMessageW only queues WM_QUIT; it does
            // NOT prove GetMessageW has processed it. Never join a worker
            // that may still be running: doing so could block every manager
            // operation behind the shared resource lock indefinitely.
            if !worker.join.is_finished() {
                if worker.thread_id == 0 {
                    // The timed-out zero-capacity handshake forces its
                    // worker to unhook itself before exiting.
                    return Err(io::Error::new(
                        io::ErrorKind::WouldBlock,
                        "Timed-out hook is still cleaning up",
                    ));
                }
                if worker.alive.load(Ordering::Acquire) && !worker.stop_requested {
                    if unsafe { PostThreadMessageW(worker.thread_id, WM_QUIT, 0, 0) } == 0 {
                        // Preserve ownership; a later explicit Stop can retry.
                        return Err(io::Error::last_os_error());
                    }
                    worker.stop_requested = true;
                }
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "Foreground observer shutdown requested; retry Stop to confirm cleanup",
                ));
            }
        }
        // A finished worker cannot block on join. Propagate message-loop
        // and UnhookWinEvent failures rather than inventing a clean Stop.
        if let Some(worker) = guard.take() {
            worker
                .join
                .join()
                .map_err(|_| io::Error::other("Foreground hook worker exited unexpectedly"))??;
        }
        Ok(status(false, EVENTS.load(Ordering::Relaxed)))
    }

    #[cfg(test)]
    mod policy_tests {
        use super::*;
        use windows_sys::Win32::{
            Foundation::{GetLastError, ERROR_ACCESS_DENIED, WAIT_OBJECT_0},
            System::{
                LibraryLoader::GetModuleFileNameW,
                Threading::{
                    CreateProcessW, TerminateProcess, CREATE_NO_WINDOW, CREATE_SUSPENDED,
                    PROCESS_INFORMATION, STARTUPINFOW,
                },
            },
        };
        struct Child {
            process: OwnedHandle,
            _thread: OwnedHandle,
            pid: u32,
            created: u64,
        }
        impl Child {
            fn new() -> Self {
                let mut path = [0u16; 32768];
                let size = unsafe {
                    GetModuleFileNameW(ptr::null_mut(), path.as_mut_ptr(), path.len() as u32)
                };
                assert!(size > 0 && size < path.len() as u32);
                let mut command: Vec<u16> = format!(
                    "\"{}\" --list\0",
                    String::from_utf16_lossy(&path[..size as usize])
                )
                .encode_utf16()
                .collect();
                let mut startup: STARTUPINFOW = unsafe { std::mem::zeroed() };
                startup.cb = std::mem::size_of_val(&startup) as u32;
                let mut process: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
                assert_ne!(
                    unsafe {
                        CreateProcessW(
                            path.as_ptr(),
                            command.as_mut_ptr(),
                            ptr::null(),
                            ptr::null(),
                            0,
                            CREATE_SUSPENDED | CREATE_NO_WINDOW,
                            ptr::null(),
                            ptr::null(),
                            &startup,
                            &mut process,
                        )
                    },
                    0
                );
                let handle = owned(process.hProcess).unwrap();
                Self {
                    created: birth(&handle).unwrap(),
                    process: handle,
                    _thread: owned(process.hThread).unwrap(),
                    pid: process.dwProcessId,
                }
            }
        }
        impl Drop for Child {
            fn drop(&mut self) {
                // Only this original unresumed copy of our test executable.
                unsafe {
                    TerminateProcess(self.process.as_raw_handle() as _, 93);
                    WaitForSingleObject(self.process.as_raw_handle() as _, 10000);
                }
            }
        }
        fn state(handle: &OwnedHandle) -> bool {
            let result = unsafe { WaitForSingleObject(handle.as_raw_handle() as _, 0) };
            assert!(result == WAIT_OBJECT_0 || result == WAIT_TIMEOUT);
            result == WAIT_OBJECT_0
        }
        #[test]
        fn five_policy_events_preserve_identity_read_only_receivers_and_stop() {
            let children: Vec<_> = (0..5).map(|_| Child::new()).collect();
            let bindings: Vec<_> = children
                .iter()
                .map(|c| Arc::new(Binding::new(c.process.as_handle(), c.pid, c.created).unwrap()))
                .collect();
            assert!(Binding::new(
                children[0].process.as_handle(),
                children[0].pid.wrapping_add(1),
                children[0].created
            )
            .is_err());
            assert!(Binding::new(
                children[0].process.as_handle(),
                children[0].pid,
                children[0].created.wrapping_add(1)
            )
            .is_err());
            let readers: Vec<_> = bindings.iter().map(|p| p.readers().unwrap()).collect();
            for endpoint in &readers {
                for handle in &endpoint[..3] {
                    assert_eq!(unsafe { SetEvent(handle.as_raw_handle() as _) }, 0);
                    assert_eq!(unsafe { GetLastError() }, ERROR_ACCESS_DENIED);
                }
            }
            for selected in 0..5 {
                publish(&bindings, Some(children[selected].pid));
                for (slot, endpoint) in readers.iter().enumerate() {
                    assert_eq!(state(&endpoint[0]), slot == selected);
                    assert!(!state(&endpoint[2]));
                }
            }
            publish(&bindings, None); // Includes focus on the manager or an unrelated application.
            assert!(readers.iter().all(|r| !state(&r[0])));
            bindings[0].finish();
            publish(&bindings, Some(children[0].pid));
            assert!(!state(&readers[0][0]) && state(&readers[0][2]));
            drop(bindings);
            assert!(readers.iter().all(|r| state(&r[2])));
        }
        #[test]
        fn registered_policies_share_one_worker_and_leave_no_active_lease() {
            let children: Vec<_> = (0..6).map(|_| Child::new()).collect();
            let mut channels = Vec::new();
            for c in &children[..5] {
                channels.push(register_policy(c.process.as_handle(), c.pid, c.created).unwrap());
            }
            assert!(channels
                .iter()
                .all(|c| c.thread_id == channels[0].thread_id));
            let c = &children[0];
            assert!(register_policy(c.process.as_handle(), c.pid, c.created).is_err());
            let sixth = &children[5];
            assert!(register_policy(sixth.process.as_handle(), sixth.pid, sixth.created).is_err());
            assert!(read().is_err()); // Public observation cannot mislabel policy mode.
            let readers: Vec<_> = channels
                .iter()
                .map(|c| c.reader_handles().unwrap())
                .collect();
            let stop_result = stop();
            assert!(
                stop_result.is_ok()
                    || stop_result
                        .as_ref()
                        .is_err_and(|e| e.kind() == io::ErrorKind::WouldBlock)
            );
            assert!(channels.iter().all(|c| c.reader_handles().is_err()));
            drop(channels);
            assert!(readers.iter().all(|r| state(&r[2])));
            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            loop {
                match stop() {
                    Ok(report) => {
                        assert!(!report.enabled);
                        break;
                    }
                    Err(e)
                        if e.kind() == io::ErrorKind::WouldBlock
                            && std::time::Instant::now() < deadline =>
                    {
                        thread::sleep(Duration::from_millis(5));
                    }
                    other => panic!("Policy worker cleanup not confirmed: {other:?}"),
                }
            }
            assert!(!read().unwrap().enabled);
            let c = &children[0];
            let fresh = register_policy(c.process.as_handle(), c.pid, c.created).unwrap();
            let endpoint = fresh.reader_handles().unwrap();
            drop(fresh);
            assert!(state(&endpoint[2]));
            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            loop {
                let finished = {
                    let guard = lock().unwrap();
                    guard.as_ref().is_none_or(|w| w.join.is_finished())
                };
                if finished {
                    break;
                }
                assert!(
                    std::time::Instant::now() < deadline,
                    "Last private lease did not end its worker"
                );
                thread::sleep(Duration::from_millis(5));
            }
            assert!(!stop().unwrap().enabled); // Reap only after confirmed thread completion.
            assert!(start().unwrap().enabled); // An explicit public observer keeps its ownership.
            let lease = register_policy(c.process.as_handle(), c.pid, c.created).unwrap();
            drop(lease);
            thread::sleep(Duration::from_millis(5));
            assert!(read().unwrap().enabled);
            let deadline = std::time::Instant::now() + Duration::from_secs(5);
            loop {
                match stop() {
                    Ok(report) => {
                        assert!(!report.enabled);
                        break;
                    }
                    Err(e)
                        if e.kind() == io::ErrorKind::WouldBlock
                            && std::time::Instant::now() < deadline =>
                    {
                        thread::sleep(Duration::from_millis(5))
                    }
                    other => panic!("Public observer cleanup not confirmed: {other:?}"),
                }
            }
        }
    }
}

#[cfg(windows)]
#[allow(unused_imports)]
pub(crate) use windows::{register_policy, PolicyChannel};

pub fn read_observer() -> io::Result<ForegroundEventObserverStatus> {
    #[cfg(windows)]
    {
        windows::read()
    }
    #[cfg(not(windows))]
    {
        Err(io::Error::new(io::ErrorKind::Unsupported, "Windows only"))
    }
}

pub fn start_observer() -> io::Result<ForegroundEventObserverStatus> {
    #[cfg(windows)]
    {
        windows::start()
    }
    #[cfg(not(windows))]
    {
        Err(io::Error::new(io::ErrorKind::Unsupported, "Windows only"))
    }
}

pub fn stop_observer() -> io::Result<ForegroundEventObserverStatus> {
    #[cfg(windows)]
    {
        windows::stop()
    }
    #[cfg(not(windows))]
    {
        Err(io::Error::new(io::ErrorKind::Unsupported, "Windows only"))
    }
}

/// Called after the Tauri run loop exits. Windows also removes hooks on
/// process exit, but our normal path explicitly unhooks on its owner thread.
pub fn shutdown() {
    #[cfg(windows)]
    if let Err(error) = windows::stop() {
        // Tauri has already exited, so preserve a diagnostic instead of
        // silently claiming an orderly cleanup was verified.
        eprintln!("MIVUBI foreground observer cleanup unverified: {error}");
    }
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
        for field in [
            "pid",
            "windowHandle",
            "windowTitle",
            "account",
            "fps",
            "gameplay",
        ] {
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
