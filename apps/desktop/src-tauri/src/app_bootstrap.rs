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

    tauri::Builder::default()
        // Legacy VM/Tauri routes are deliberately not registered in the
        // native-first pilot. Source and account-bearing guest data remain.
        // Re-enabling a route requires an explicit product decision.
        .invoke_handler(tauri::generate_handler![
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
            commands::native_clients::desktop_inspect_client_02_foreground,
            commands::native_clients::desktop_focus_observed_client_02,
            commands::native_clients::desktop_place_observed_client_02,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Minecraft Instance Manager desktop runtime");
}

fn show_startup_error(message: &str) {
    let _ = MessageDialog::new()
        .set_title("Minecraft Instance Manager")
        .set_description(message)
        .set_level(MessageLevel::Info)
        .set_buttons(MessageButtons::Ok)
        .show();
}
