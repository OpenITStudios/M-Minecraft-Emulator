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
        // Legacy VM/Tauri routes are deliberately not registered in the
        // native-first pilot. Source and account-bearing guest data remain.
        // Re-enabling a route requires an explicit product decision.
        .invoke_handler(tauri::generate_handler![
            commands::native_clients::desktop_present_workspace,
            commands::native_clients::desktop_update_workspace,
            commands::native_clients::desktop_release_workspace,
            commands::native_clients::desktop_manager_settings,
            commands::native_clients::desktop_owned_test_operation,
            commands::native_clients::desktop_select_owned_test_session,
            commands::native_clients::desktop_list_instance_slots,
            commands::native_clients::desktop_inspect_instance_overview,
            commands::native_clients::desktop_open_native_minecraft,
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
    let _ = crate::engine::workspace_surface::release();
    // Unhook from the worker's installation thread before exit.
    crate::engine::instance_focus_events::shutdown();
    result.expect("failed to run Minecraft Instance Manager desktop runtime");
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
