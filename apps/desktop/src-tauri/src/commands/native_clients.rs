use crate::engine::{
    instance_catalog::{admit_slot_operation, instance_slots, InstanceSlotDescriptor, SlotOperation},
    instance_discovery::{self, InstanceOverview},
    instance_focus::{self, InstanceFocusObservation},
    instance_focus_events::{self, ForegroundEventObserverStatus},
    native_clients,
};

use std::{io, sync::Mutex};

// One admission point for heavyweight MIVUBI desktop requests. The caller
// may not bypass this by invoking Tauri commands directly while another
// probe, file hash, dialog, launch or window operation is in flight.
// This limits manager-created CPU/RAM peaks, not Minecraft CPU or GPU usage.
static MANAGER_RESOURCE_LOCK: Mutex<()> = Mutex::new(());

// One tiny, synchronous read-only catalog, no resource lock or Windows helper.
#[tauri::command]
pub fn desktop_list_instance_slots() -> [InstanceSlotDescriptor; 4] {
    instance_slots()
}

fn admit_resource_operation<T>(
    lock: &Mutex<()>,
    operation: impl FnOnce() -> io::Result<T>,
) -> io::Result<T> {
    let _guard = lock.try_lock().map_err(|_| {
        io::Error::new(io::ErrorKind::WouldBlock,
            "Another MIVUBI desktop operation is running; retry after it finishes")
    })?;
    operation()
}

fn run_manager_operation<T>(operation: impl FnOnce() -> io::Result<T>) -> io::Result<T> {
    admit_resource_operation(&MANAGER_RESOURCE_LOCK, operation)
}

// Explicit inspection of all four stable IDs. The two existing Windows
// collectors run sequentially beneath ONE manager resource lock. No new
// inspector service, process monitor, or shared runtime state is created.
#[tauri::command]
pub async fn desktop_inspect_instance_overview() -> Result<InstanceOverview, String> {
    tauri::async_runtime::spawn_blocking(|| run_manager_operation(instance_discovery::inspect_instance_overview))
        .await
        .map_err(|error| format!("Four-slot inspection task failed: {error}"))?
        .map_err(|error| error.to_string())
}

// One opt-in WinEvent observer counts foreground transitions with no
// per-event PID/HWND/title retention, Minecraft interaction, or FPS mutation.
#[tauri::command]
pub fn desktop_read_focus_observer() -> Result<ForegroundEventObserverStatus, String> {
    instance_focus_events::read_observer().map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn desktop_start_focus_observer() -> Result<ForegroundEventObserverStatus, String> {
    tauri::async_runtime::spawn_blocking(|| run_manager_operation(
        instance_focus_events::start_observer
    ))
        .await
        .map_err(|error| format!("Focus observer startup task failed: {error}"))?
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn desktop_stop_focus_observer() -> Result<ForegroundEventObserverStatus, String> {
    tauri::async_runtime::spawn_blocking(|| run_manager_operation(
        instance_focus_events::stop_observer
    ))
        .await
        .map_err(|error| format!("Focus observer shutdown task failed: {error}"))?
        .map_err(|error| error.to_string())
}

// Validate the exact slot/operation *before* touching a Windows helper.
// This reuses the single slot catalog; no second lifecycle state store.
fn run_slot_operation<T>(
    id: &str,
    action: SlotOperation,
    operation: impl FnOnce() -> io::Result<T>,
) -> io::Result<T> {
    admit_slot_operation(id, action)?;
    run_manager_operation(operation)
}

#[tauri::command]
pub async fn desktop_open_native_minecraft() -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(|| run_slot_operation("01", SlotOperation::Launch, native_clients::open_native_minecraft))
        .await
        .map_err(|error| format!("Desktop launch task failed: {error}"))?
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn desktop_inspect_native_candidates() -> Result<native_clients::NativeProcessObservation, String> {
    tauri::async_runtime::spawn_blocking(|| run_slot_operation("01", SlotOperation::Inspect, native_clients::inspect_native_candidates))
        .await
        .map_err(|error| format!("Native inspection task failed: {error}"))?
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn desktop_select_education_exe() -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(|| run_slot_operation("02", SlotOperation::SelectExecutable, native_clients::select_education_exe))
        .await.map_err(|e| format!("File dialog failed: {e}"))?
        .map_err(|e| e.to_string())
}
#[tauri::command]
pub async fn desktop_request_sandbox_client_02(executable: String, licensed: bool) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || run_slot_operation("02", SlotOperation::Launch, || native_clients::request_sandbox_client_02(executable, licensed)))
        .await.map_err(|e| format!("Isolated launch failed: {e}"))?
        .map_err(|e| e.to_string())
}

// One-shot read-only GetForegroundWindow snapshot, using the same full
// fresh Client 02 identity verification as the existing Focus/Place commands.
#[tauri::command]
pub async fn desktop_inspect_instance_focus(
    target: native_clients::ProcessIdentityCandidate,
) -> Result<InstanceFocusObservation, String> {
    tauri::async_runtime::spawn_blocking(move || run_slot_operation(
        "02", SlotOperation::Inspect, || instance_focus::inspect_instance_focus(target)
    ))
        .await
        .map_err(|error| format!("Foreground inspection task failed: {error}"))?
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn desktop_focus_observed_client_02(target: native_clients::ProcessIdentityCandidate) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || run_slot_operation("02", SlotOperation::Focus, || native_clients::focus_observed_client_02(target)))
        .await
        .map_err(|error| format!("Client 02 focus task failed: {error}"))?
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn desktop_place_observed_client_02(
    target: native_clients::ProcessIdentityCandidate,
    side: native_clients::Client02PlacementSide,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || run_slot_operation("02", SlotOperation::Place, || native_clients::place_observed_client_02(target, side)))
        .await
        .map_err(|error| format!("Client 02 placement task failed: {error}"))?
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn desktop_inspect_sandbox_client_02() -> Result<native_clients::SandboxieClientObservation, String> {
    tauri::async_runtime::spawn_blocking(|| run_slot_operation("02", SlotOperation::Inspect, native_clients::inspect_sandbox_client_02))
        .await
        .map_err(|error| format!("Sandboxie inspection task failed: {error}"))?
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn desktop_inspect_owned_engine_candidate(
    directory: String,
) -> Result<crate::engine::owned_isolation::OwnedCandidateInspection, String> {
    tauri::async_runtime::spawn_blocking(move || run_manager_operation(||
        native_clients::inspect_owned_engine_candidate(directory)))
        .await
        .map_err(|error| format!("Owned engine candidate inspection failed: {error}"))?
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn desktop_select_owned_engine_candidate_directory() -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(|| run_manager_operation(native_clients::select_owned_candidate_directory))
        .await
        .map_err(|error| format!("Owned candidate directory picker failed: {error}"))?
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn desktop_inspect_owned_engine_host(
) -> Result<crate::engine::owned_isolation::OwnedHostInspection, String> {
    tauri::async_runtime::spawn_blocking(|| run_manager_operation(native_clients::inspect_owned_engine_host))
        .await
        .map_err(|error| format!("Owned runtime host inspection failed: {error}"))?
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn desktop_discover_existing_virtuals(
) -> Result<crate::engine::virtual_inventory::VirtualBoxInventory, String> {
    tauri::async_runtime::spawn_blocking(|| run_manager_operation(
        crate::engine::virtual_inventory::discover_existing_virtuals))
        .await
        .map_err(|error| format!("Virtual discovery task failed: {error}"))?
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod resource_admission_tests {
    use super::{admit_resource_operation, run_slot_operation};
    use crate::engine::instance_catalog::SlotOperation;
    use std::{cell::Cell, io, sync::Mutex};

    #[test]
    fn simultaneous_heavy_request_fails_before_running_a_second_helper() {
        let gate = Mutex::new(());
        let existing = gate.lock().unwrap();
        let invoked = Cell::new(false);
        let error = admit_resource_operation(&gate, || {
            invoked.set(true);
            Ok(())
        }).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::WouldBlock);
        assert!(!invoked.get(), "must not spawn an overlapping Windows helper");
        drop(existing);
        assert_eq!(admit_resource_operation(&gate, || Ok(7)).unwrap(), 7);
    }

    #[test]
    fn future_or_invalid_slot_fails_before_any_resource_action() {
        let invoked = Cell::new(false);
        for id in ["03", "04", "", "02 ", "05"] {
            let error = run_slot_operation(id, SlotOperation::Launch, || {
                invoked.set(true);
                Ok(())
            }).unwrap_err();
            assert!(matches!(error.kind(),
                io::ErrorKind::Unsupported | io::ErrorKind::InvalidInput));
        }
        assert!(!invoked.get(), "no unavailable slot may run a helper");
        assert_eq!(run_slot_operation("02", SlotOperation::Inspect, || Ok("read-only"))
            .unwrap(), "read-only");
    }

    #[test]
    fn a_failed_helper_releases_admission_for_the_next_explicit_action() {
        let gate = Mutex::new(());
        let error = admit_resource_operation(&gate, || -> io::Result<()> {
            Err(io::Error::other("collector failed"))
        }).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::Other);
        assert_eq!(admit_resource_operation(&gate, || Ok("ready")).unwrap(), "ready");
    }
}
