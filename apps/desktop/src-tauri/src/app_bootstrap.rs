use crate::commands;
use crate::engine::app_instance;
use rfd::{MessageButtons, MessageDialog, MessageLevel};

pub fn run() {
    if let Err(error) = crate::engine::dpi::configure() {
        show_startup_error(&format!("Failed to configure display scaling: {error}"));
        return;
    }

    let _instance_lease = match app_instance::acquire() {
        Ok(lease) => lease,
        Err(error) => {
            show_startup_error(&error);
            return;
        }
    };

    let result = tauri::Builder::default()
        .setup(|app|{
            #[cfg(windows)] {
                use tauri::Manager;
                let window=app.get_webview_window("main").ok_or_else(||std::io::Error::other("Main workspace window missing"))?;
                workspace_foreground::start(window.as_ref().window())?;
            }
            Ok(())
        })
        // Legacy VM/Tauri routes are deliberately not registered in the
        // native-first pilot. Source and account-bearing guest data remain.
        // Re-enabling a route requires an explicit product decision.
        .invoke_handler(tauri::generate_handler![
            commands::native_clients::desktop_compare_process_contexts,
            commands::native_clients::desktop_present_workspace,
            commands::native_clients::desktop_update_workspace,
            commands::native_clients::desktop_release_workspace,
            commands::native_clients::desktop_suspend_workspace,
            commands::native_clients::desktop_manager_settings,
            commands::native_clients::desktop_owned_test_operation,
            commands::native_clients::desktop_select_owned_test_session,
            commands::native_clients::desktop_list_instance_slots,
            commands::native_clients::desktop_inspect_instance_overview,
            commands::native_clients::desktop_open_native_minecraft,
            commands::native_clients::desktop_inspect_native_workspace,
            commands::native_clients::desktop_inspect_native_candidates,
            commands::native_clients::desktop_select_education_exe,
            commands::native_clients::desktop_request_sandbox_client_02,
            commands::native_clients::desktop_inspect_sandbox_client_02,
            commands::native_clients::desktop_inspect_owned_engine_candidate,
            commands::native_clients::desktop_inspect_owned_engine_host,
            commands::native_clients::desktop_discover_existing_virtuals,
            commands::native_clients::desktop_select_owned_engine_candidate_directory,
            commands::native_clients::desktop_inspect_instance_focus,
            commands::native_clients::desktop_read_focus_observer,
            commands::native_clients::desktop_start_focus_observer,
            commands::native_clients::desktop_stop_focus_observer,
            commands::native_clients::desktop_focus_observed_client_02,
            commands::native_clients::desktop_place_observed_client_02,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested{api,..}=event {
                if crate::engine::workspace_surface::is_active() {
                    use std::sync::atomic::{AtomicBool,Ordering};
                    static CONFIRMING:AtomicBool=AtomicBool::new(false);
                    api.prevent_close();
                    if !CONFIRMING.swap(true,Ordering::SeqCst) {
                        let window=window.clone();
                        tauri::async_runtime::spawn_blocking(move || {
                            let choice=MessageDialog::new().set_title("Close manager?").set_description("Minecraft will stay open in its normal window. Your game will not be closed.").set_buttons(MessageButtons::YesNo).show();
                            if choice==rfd::MessageDialogResult::Yes {
                                match crate::engine::workspace_surface::release(){Ok(())=>{let _=window.destroy();},Err(error)=>show_startup_error(&format!("Window recovery needs attention: {error}"))}
                            }
                            CONFIRMING.store(false,Ordering::SeqCst);
                        });
                    }
                }
            }
            #[cfg(windows)]
            if matches!(
                event,
                tauri::WindowEvent::Moved(_)
                    | tauri::WindowEvent::Resized(_)
                    | tauri::WindowEvent::Focused(true)
            ) {
                queue_workspace_update(window.clone());
            }
        })
        .run(tauri::generate_context!());
    #[cfg(windows)]
    workspace_foreground::shutdown();
    let _ = crate::engine::workspace_surface::release();
    // Unhook from the worker's installation thread before exit.
    crate::engine::instance_focus_events::shutdown();
    result.expect("failed to run Minecraft Instance Manager desktop runtime");
}

#[cfg(windows)]
mod workspace_foreground {
    use std::{
        io, ptr,
        sync::{
            atomic::{AtomicU64, Ordering},
            mpsc, Mutex, OnceLock,
        },
        thread::{self, JoinHandle},
        time::Duration,
    };
    use windows_sys::Win32::{
        Foundation::HWND,
        System::Threading::GetCurrentThreadId,
        UI::{
            Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK},
            WindowsAndMessaging::*,
        },
    };
    static WINDOW: OnceLock<tauri::Window> = OnceLock::new();
    static HANDLE: AtomicU64 = AtomicU64::new(0);
    static WORKER: Mutex<Option<(u32, JoinHandle<()>)>> = Mutex::new(None);
    fn accepts(event: u32, window: u64, object: i32, child: i32, manager: u64) -> bool {
        manager != 0
            && window == manager
            && event == EVENT_SYSTEM_FOREGROUND
            && object == 0
            && child == 0
    }
    unsafe extern "system" fn changed(
        _: HWINEVENTHOOK,
        event: u32,
        window: HWND,
        object: i32,
        child: i32,
        _: u32,
        _: u32,
    ) {
        // Observe only the manager's own foreground window; never inspect external titles/PIDs.
        if accepts(
            event,
            window as usize as u64,
            object,
            child,
            HANDLE.load(Ordering::Acquire),
        ) {
            if let Some(window) = WINDOW.get() {
                super::queue_workspace_update(window.clone());
            }
        }
    }
    pub fn start(window: tauri::Window) -> io::Result<()> {
        let handle = window.hwnd().map_err(io::Error::other)?.0 as usize as u64;
        WINDOW
            .set(window)
            .map_err(|_| io::Error::other("Workspace foreground watcher already initialized"))?;
        HANDLE.store(handle, Ordering::Release);
        let (ready, receiver) = mpsc::sync_channel(1);
        let worker = thread::spawn(move || unsafe {
            let mut message: MSG = std::mem::zeroed();
            PeekMessageW(&mut message, ptr::null_mut(), 0, 0, PM_NOREMOVE);
            let hook = SetWinEventHook(
                EVENT_SYSTEM_FOREGROUND,
                EVENT_SYSTEM_FOREGROUND,
                ptr::null_mut(),
                Some(changed),
                std::process::id(),
                0,
                WINEVENT_OUTOFCONTEXT,
            );
            if hook.is_null() {
                let _ = ready.send(Err(io::Error::last_os_error()));
                return;
            }
            if ready.send(Ok(GetCurrentThreadId())).is_ok() {
                while GetMessageW(&mut message, ptr::null_mut(), 0, 0) > 0 {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
            UnhookWinEvent(hook);
        });
        match receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(Ok(id)) => {
                *WORKER
                    .lock()
                    .map_err(|_| io::Error::other("Workspace watcher state unavailable"))? =
                    Some((id, worker));
                Ok(())
            }
            Ok(Err(error)) => {
                let _ = worker.join();
                Err(error)
            }
            Err(_) => Err(io::Error::other(
                "Workspace foreground watcher startup timed out",
            )),
        }
    }
    pub fn shutdown() {
        HANDLE.store(0, Ordering::Release);
        if let Ok(mut worker) = WORKER.lock() {
            if let Some((id, join)) = worker.take() {
                if unsafe { PostThreadMessageW(id, WM_QUIT, 0, 0) } != 0 {
                    let _ = join.join();
                }
            }
        }
    }
    #[test]
    fn only_manager_foreground_events_can_restore_the_surface() {
        assert!(accepts(EVENT_SYSTEM_FOREGROUND, 17, 0, 0, 17));
        for (event, window, object, child, manager) in [
            (EVENT_SYSTEM_FOREGROUND, 18, 0, 0, 17),
            (EVENT_SYSTEM_FOREGROUND, 17, 1, 0, 17),
            (EVENT_SYSTEM_FOREGROUND, 17, 0, 1, 17),
            (EVENT_OBJECT_SHOW, 17, 0, 0, 17),
            (EVENT_SYSTEM_FOREGROUND, 0, 0, 0, 0),
        ] {
            assert!(!accepts(event, window, object, child, manager));
        }
    }
}

#[cfg(windows)]
fn queue_workspace_update(window: tauri::Window) {
    use std::sync::Mutex;
    use tauri::Emitter;
    // One worker drains the latest event; no stale visibility snapshots or thread per move.
    static PENDING: Mutex<(bool, Option<tauri::Window>)> = Mutex::new((false, None));
    let Ok(mut pending) = PENDING.lock() else {
        return;
    };
    pending.1 = Some(window);
    if pending.0 {
        return;
    }
    pending.0 = true;
    drop(pending);
    tauri::async_runtime::spawn_blocking(|| loop {
        let window = {
            let Ok(mut pending) = PENDING.lock() else {
                return;
            };
            match pending.1.take() {
                Some(window) => window,
                None => {
                    pending.0 = false;
                    return;
                }
            }
        };
        let result = window.hwnd().map_err(|e| e.to_string()).and_then(|handle| {
            crate::engine::workspace_surface::update(handle.0 as usize as u64, None)
                .map_err(|e| e.to_string())
        });
        if let Err(error) = result {
            let _ = window.emit("workspace-surface-error", error);
        }
    });
}

fn show_startup_error(message: &str) {
    let _ = MessageDialog::new()
        .set_title("Minecraft Instance Manager")
        .set_description(message)
        .set_level(MessageLevel::Info)
        .set_buttons(MessageButtons::Ok)
        .show();
}
