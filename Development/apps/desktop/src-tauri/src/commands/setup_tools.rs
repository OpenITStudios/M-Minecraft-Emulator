use crate::engine::setup_tools;
use tauri::{path::BaseDirectory, AppHandle, Manager};

#[tauri::command]
pub async fn setup_open_guest_tools(app: AppHandle) -> Result<(), String> {
    let tools = app
        .path()
        .resolve("guest/windows", BaseDirectory::Resource)
        .map_err(|error| format!("Virtual Clients setup tools path is unavailable: {error}"))?;
    if !tools.is_dir() {
        return Err(format!("Packaged setup tools are missing: {}", tools.display()));
    }
    let guest_agent = tools.join("virtual-guest-agent.exe");
    if !guest_agent.is_file() {
        return Err(format!(
            "Packaged Guest Agent is missing from setup tools: {}",
            guest_agent.display()
        ));
    }

    tauri::async_runtime::spawn_blocking(move || setup_tools::open_folder(&tools))
        .await
        .map_err(|error| format!("Open setup tools task failed: {error}"))?
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn setup_open_base_location() -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(setup_tools::open_base_location)
        .await
        .map_err(|error| format!("Open Base location task failed: {error}"))?
        .map_err(|error| error.to_string())
}

