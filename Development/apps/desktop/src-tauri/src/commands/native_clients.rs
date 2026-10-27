use crate::engine::{
    instance_catalog::{
        admit_slot_operation, instance_slots, InstanceSlotDescriptor, SlotOperation,
    },
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

#[tauri::command]
pub async fn lazy_tester_data_location(action:native_clients::DataLocationAction) -> Result<native_clients::DataLocation,String> {
    if crate::app_bootstrap::startup_diagnostic_mode() && !matches!(action,native_clients::DataLocationAction::Inspect) {return Err("Startup diagnostic permits inspection only".into());}
    tauri::async_runtime::spawn_blocking(move||run_manager_operation(||native_clients::data_location(action)))
        .await.map_err(|_|"Data location task failed".to_string())?.map_err(|error|error.to_string())
}

#[tauri::command]
pub async fn lazy_tester_profile_binding(import: bool) -> Result<native_clients::ProfileBinding, String> {
    if crate::app_bootstrap::startup_diagnostic_mode() && import {return Err("Startup diagnostic cannot import profiles".into());}
    tauri::async_runtime::spawn_blocking(move || run_manager_operation(|| native_clients::profile_binding(import)))
        .await.map_err(|_| "Profile binding task failed".to_string())?.map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn lazy_tester_session(
    request: crate::engine::native_clients_session::SessionRequest,
) -> Result<crate::engine::native_clients_session::SessionSnapshot, String> {
    if crate::app_bootstrap::startup_diagnostic_mode() && !matches!(request,crate::engine::native_clients_session::SessionRequest::Inspect) {return Err("Startup diagnostic cannot launch or modify games".into());}
    tauri::async_runtime::spawn_blocking(move || run_manager_operation(|| {
        crate::engine::native_clients_session::request(request)
    })).await.map_err(|_| "Session task failed".to_string())?.map_err(|error| error.to_string())
}

// This receipt distinguishes a preflight failure (returned as Err before
// presentation mutation) from a still-running game AFTER its workspace was
// successfully released. It records no additional game ownership.
#[derive(serde::Serialize)]
#[serde(rename_all="camelCase")]
pub struct SelectedCloseReport {
    schema: u32,
    presentation_released: bool,
    snapshot: Option<crate::engine::native_clients_session::SessionSnapshot>,
    issue: Option<String>,
}

// This one command admits the entire selected-close sequence under the
// existing manager lock. The retained session controller owns lifecycle.
#[tauri::command]
pub async fn lazy_tester_close_one(
    slot:u32,
) -> Result<SelectedCloseReport,String> {
    if crate::app_bootstrap::startup_diagnostic_mode() {return Err("Startup diagnostic cannot close games".into());}
    if !(1..=5).contains(&slot) {return Err("Unknown Minecraft profile".into());}
    tauri::async_runtime::spawn_blocking(move||run_manager_operation(||{
        use crate::engine::native_clients_session::{request,SessionRequest};
        let snapshot=request(SessionRequest::Inspect)?;
        let selected=snapshot.profiles.iter().find(|profile|profile.slot==slot)
            .ok_or_else(||io::Error::new(io::ErrorKind::InvalidInput,"Unknown Minecraft profile"))?;
        if selected.state=="STOPPED" || selected.state=="ATTENTION" {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,"Minecraft session is not running"));
        }
        let recovery_issue=match crate::engine::workspace_surface::release_managed_slot(slot) {
            Ok(()) => None,
            Err((false,error)) => return Err(error),
            Err((true,error)) => Some(error),
        };
        if let Some(ref recovery)=recovery_issue {
            // A lost original OGLES HWND need not prevent the game's normal
            // close: only the target's own helper may grant the fallback, and
            // only after its terminal End + process exit are both observed.
            let quiesced=crate::engine::workspace_surface::allow_managed_slot_close_after_failed_restore(slot);
            match quiesced {
                Ok(true) => {}
                Ok(false) => return Ok(SelectedCloseReport {
                    schema:1,presentation_released:false,snapshot:None,
                    issue:Some(format!("Window recovery unconfirmed: {recovery}. Target helper still active or ambiguous; Minecraft close not requested.")),
                }),
                Err(error) => return Ok(SelectedCloseReport {
                    schema:1,presentation_released:false,snapshot:None,
                    issue:Some(format!("Window recovery unconfirmed: {recovery}. Helper identity unavailable: {error}; Minecraft close not requested.")),
                }),
            }
        }
        // All five slots share this SAME single-child normal-close controller.
        // No extra worker, PID/HWND selector or peer shutdown is permitted.
        let result=request(SessionRequest::CloseOne {slot});
        if let Some(recovery)=recovery_issue {
            return match result {
                Err(close) => Ok(SelectedCloseReport {
                    schema:1,presentation_released:false,snapshot:None,
                    issue:Some(format!("Window restoration unconfirmed: {recovery}. Normal Minecraft close was requested but remains pending: {close}")),
                }),
                Ok(snapshot) => match crate::engine::workspace_surface::release_exited_managed_slot_checkpoint(slot) {
                    Ok(()) => Ok(SelectedCloseReport {
                        schema:1,presentation_released:true,snapshot:Some(snapshot),issue:None,
                    }),
                    Err(cleanup) => Ok(SelectedCloseReport {
                        schema:1,presentation_released:false,snapshot:None,
                        issue:Some(format!("Minecraft closed but its native window checkpoint is still unconfirmed: {recovery}; {cleanup}")),
                    }),
                },
            };
        }
        Ok(SelectedCloseReport {
            schema:1,presentation_released:true,
            snapshot:result.as_ref().ok().cloned(),
            issue:result.err().map(|error|error.to_string()),
        })
    })).await.map_err(|_|"Selected Minecraft close task failed".to_string())?
        .map_err(|error|error.to_string())
}

#[tauri::command]
pub async fn lazy_tester_present(
    window: tauri::WebviewWindow,
    slot: u32,
    viewport: crate::engine::workspace_surface::Viewport,
) -> Result<(), String> {
    #[cfg(windows)]
    let manager = window.hwnd().map_err(|error| error.to_string())?.0 as usize as u64;
    #[cfg(not(windows))]
    let manager = { let _ = window; 0 };
    tauri::async_runtime::spawn_blocking(move || run_manager_operation(|| {
        let snapshot = crate::engine::native_clients_session::request(
            crate::engine::native_clients_session::SessionRequest::Inspect)?;
        let target = snapshot.profiles.into_iter().find(|p| p.slot == slot)
            .and_then(|p| p.process).filter(|p| p.responding)
            .ok_or_else(|| io::Error::other("Minecraft has no verified responsive game window"))?;
        // Slot attribution comes from retained original bootstrap handles, never UI PID/title.
        let authority = "LAZY_TESTER_MANAGED_SESSION".to_string();
        let client = format!("{slot:02}");
        if crate::engine::workspace_surface::present_retained(manager, viewport, target.clone(), authority.clone(), client.clone())? {
            return Ok(());
        }
        crate::engine::workspace_surface::present_verified(manager, viewport, target, authority, client)
    })).await.map_err(|_| "Workspace task failed".to_string())?.map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn lazy_tester_previews(
    window: tauri::WebviewWindow,
    viewport: crate::engine::workspace_surface::Viewport,
    tiles: Vec<crate::engine::workspace_surface::PreviewTile>,
) -> Result<crate::engine::workspace_surface::PreviewReport, String> {
    let window = window.as_ref().window();
    tauri::async_runtime::spawn_blocking(move || run_manager_operation(|| {
        let snapshot = crate::engine::native_clients_session::request(
            crate::engine::native_clients_session::SessionRequest::Inspect)?;
        let profiles = snapshot.profiles.into_iter().filter_map(|profile|
            profile.process.map(|process|(profile.slot,process))).collect();
        crate::engine::workspace_surface::configure_previews(window,viewport,tiles,profiles)
    })).await.map_err(|_| "Preview task failed".to_string())?.map_err(|error|error.to_string())
}

#[tauri::command]
pub async fn lazy_tester_clear_previews() -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(|| run_manager_operation(
        crate::engine::workspace_surface::clear_previews))
        .await.map_err(|_|"Preview cleanup failed".to_string())?.map_err(|error|error.to_string())
}

#[tauri::command]
pub async fn desktop_inspect_native_workspace(
) -> Result<native_clients::NativeWorkspaceObservation, String> {
    tauri::async_runtime::spawn_blocking(|| {
        run_manager_operation(native_clients::inspect_native_workspace)
    })
    .await
    .map_err(|_| "Native Minecraft observation task failed".to_string())?
    .map_err(|error| error.to_string())
}

/// Read-only Windows login-helper process-family evidence for official
/// Minecraft game instances. No account data, tokens or broker IPC.
#[tauri::command]
pub async fn desktop_inspect_native_auth_family(
) -> Result<native_clients::NativeAuthFamilyObservation, String> {
    tauri::async_runtime::spawn_blocking(|| {
        run_manager_operation(native_clients::inspect_native_auth_family)
    })
    .await
    .map_err(|_| "Native Minecraft sign-in helper inspection task failed".to_string())?
    .map_err(|error| error.to_string())
}

/// Inspect Windows PSEC OS capability only. Not Minecraft launch admission,
/// filesystem redirection, registry/WAM isolation or a provider handshake.
#[tauri::command]
pub async fn desktop_inspect_native_psec_capabilities(
) -> Result<crate::engine::instance_identity::NativePsecInspection, String> {
    tauri::async_runtime::spawn_blocking(|| {
        run_manager_operation(crate::engine::instance_identity::inspect_native_psec_capabilities)
    })
    .await
    .map_err(|_| "Native Windows sandbox capability inspection task failed".to_string())?
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn desktop_compare_process_contexts(
    left: crate::engine::instance_identity::ProcessKey,
    right: crate::engine::instance_identity::ProcessKey,
) -> Result<crate::engine::instance_identity::ContextComparison, String> {
    tauri::async_runtime::spawn_blocking(move || {
        run_manager_operation(|| crate::engine::instance_identity::compare_contexts(&left, &right))
    })
    .await
    .map_err(|_| "Windows context inspection task failed".to_string())?
    .map_err(|error| error.to_string())
}

type SurfaceFailure = crate::engine::manager_settings::OperationFailure;
fn surface_failure(error: impl std::fmt::Display) -> SurfaceFailure {
    SurfaceFailure::from(io::Error::other(error.to_string()))
}

#[tauri::command]
pub async fn desktop_present_workspace(
    window: tauri::WebviewWindow,
    session_directory: String,
    client_id: String,
    target: crate::engine::owned_provider::OwnedProcess,
    viewport: crate::engine::workspace_surface::Viewport,
) -> Result<(), SurfaceFailure> {
    #[cfg(windows)]
    let manager = window.hwnd().map_err(surface_failure)?.0 as usize as u64;
    #[cfg(not(windows))]
    let manager = {
        let _ = window;
        0
    };
    tauri::async_runtime::spawn_blocking(move || {
        run_manager_operation(|| {
            use crate::engine::owned_provider::{OwnedOperation, OwnedRequest, OwnedTestBackend};
            crate::engine::instance_catalog::admit_owned_test_operation(
                &client_id,
                SlotOperation::Place,
            )?;
            if crate::engine::workspace_surface::present_retained(
                manager,
                viewport,
                target.clone(),
                session_directory.clone(),
                client_id.clone(),
            )? {
                return Ok(());
            }
            // MC01 works without retired Sandboxie, a fork or a VM.
            // Repeat process/window observation server-side before presentation.
            if client_id == "01" {
                let observed = native_clients::inspect_native_workspace()?;
                if !native_clients::matches_native_workspace_target(&observed, &target) {
                    return Err(io::Error::other(
                        "Native Minecraft process/window changed; refresh the registered game",
                    ));
                }
                return crate::engine::workspace_surface::present_verified(
                    manager, viewport, target, session_directory, client_id,
                );
            }
            let request = OwnedRequest {
                session_directory: session_directory.clone(),
                client_id: "02".into(),
                operation: OwnedOperation::Inspect,
                licensed: false,
                compatibility_confirmed: false,
                target: None,
            };
            let observation = OwnedTestBackend { request: &request }.execute()?;
            let slot = observation
                .clients
                .iter()
                .find(|slot| {
                    slot.client_id == client_id
                        && slot.status == "RUNNING"
                        && slot.settings_verified
                })
                .ok_or_else(|| io::Error::other("Client is not ready for the workspace"))?;
            let processes = &slot.processes;
            let matches = processes.len() == 1
                && processes.iter().any(|p| {
                    p.pid == target.pid
                        && p.created_file_time == target.created_file_time
                        && p.session_id == target.session_id
                        && p.window_handle == target.window_handle
                        && p.responding
                });
            if !matches {
                return Err(io::Error::other(
                    "Workspace process attribution changed; check clients again",
                ));
            }
            crate::engine::workspace_surface::present_verified(
                manager,
                viewport,
                target,
                session_directory,
                client_id,
            )
        })
    })
    .await
    .map_err(surface_failure)?
    .map_err(surface_failure)
}

/// Present an existing verified Windows game window only. This does not
/// provision, launch or attest an independent account/session provider.
#[tauri::command]
pub async fn desktop_present_observed_window(
    window: tauri::WebviewWindow,
    client_id: String,
    target: crate::engine::owned_provider::OwnedProcess,
    viewport: crate::engine::workspace_surface::Viewport,
) -> Result<(), SurfaceFailure> {
    #[cfg(windows)]
    let manager = window.hwnd().map_err(surface_failure)?.0 as usize as u64;
    #[cfg(not(windows))]
    let manager = {
        let _ = window;
        0
    };
    tauri::async_runtime::spawn_blocking(move || {
        run_manager_operation(|| {
            if !matches!(client_id.as_str(), "01" | "02" | "03" | "04") {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Unknown presentation-only window slot",
                ));
            }
            let authority = "OBSERVED_WINDOWS_PRESENTATION_ONLY".to_string();
            // Fast tab switching: a lease admitted against the registered
            // Windows game image retains exact PID/creation-FILETIME/HWND and
            // owner authority. The native surface revalidates its OS identity
            // on every presentation; do not spawn PowerShell again for it.
            if crate::engine::workspace_surface::present_retained(
                manager, viewport, target.clone(), authority.clone(), client_id.clone(),
            )? {
                return Ok(());
            }
            // First admission or a process/window replacement must recheck the
            // actual registered Minecraft executable from Windows. A UI slot
            // or a process name alone never authorizes an unverified window.
            let observed = native_clients::inspect_native_workspace()?;
            if !native_clients::matches_native_workspace_target(&observed, &target) {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "Observed Minecraft window identity changed; refresh before selecting",
                ));
            }
            crate::engine::workspace_surface::present_verified(
                manager, viewport, target, authority, client_id,
            )
        })
    })
    .await
    .map_err(surface_failure)?
    .map_err(surface_failure)
}

/// Activate one already-running official Minecraft game in Windows.
/// No isolated session or new launch authority is inferred from window focus.
#[tauri::command]
pub async fn desktop_focus_observed_native_window(
    target: crate::engine::owned_provider::OwnedProcess,
) -> Result<(), SurfaceFailure> {
    tauri::async_runtime::spawn_blocking(move || {
        run_manager_operation(|| crate::engine::workspace_surface::focus_observed_native_window(target))
    })
    .await
    .map_err(surface_failure)?
    .map_err(surface_failure)
}

#[tauri::command]
pub async fn desktop_update_workspace(
    window: tauri::WebviewWindow,
    viewport: crate::engine::workspace_surface::Viewport,
) -> Result<(), SurfaceFailure> {
    #[cfg(windows)]
    let manager = window.hwnd().map_err(surface_failure)?.0 as usize as u64;
    #[cfg(not(windows))]
    let manager = {
        let _ = window;
        0
    };
    tauri::async_runtime::spawn_blocking(move || {
        crate::engine::workspace_surface::update(manager, Some(viewport))
    })
    .await
    .map_err(surface_failure)?
    .map_err(surface_failure)
}

#[tauri::command]
pub async fn desktop_release_workspace(
    target: Option<crate::engine::owned_provider::OwnedProcess>,
) -> Result<(), SurfaceFailure> {
    tauri::async_runtime::spawn_blocking(move || {
        run_manager_operation(|| match target {
            Some(target) => crate::engine::workspace_surface::release_target(&target),
            None => crate::engine::workspace_surface::release(),
        })
    })
    .await
    .map_err(surface_failure)?
    .map_err(surface_failure)
}

#[tauri::command]
pub async fn desktop_suspend_workspace() -> Result<bool, SurfaceFailure> {
    tauri::async_runtime::spawn_blocking(|| {
        run_manager_operation(crate::engine::workspace_surface::suspend)
    })
    .await
    .map_err(surface_failure)?
    .map_err(surface_failure)
}

#[tauri::command]
pub async fn desktop_manager_settings(
    settings: Option<crate::engine::manager_settings::ManagerSettings>,
    include_legacy: Option<bool>,
) -> Result<serde_json::Value, String> {
    tauri::async_runtime::spawn_blocking(move || {
        run_manager_operation(|| {
            if include_legacy == Some(true) {
                crate::engine::manager_settings::prepare_legacy(settings)
            } else {
                crate::engine::manager_settings::prepare(settings)
            }
        })
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn desktop_owned_test_operation(
    request: crate::engine::owned_provider::OwnedRequest,
) -> Result<
    crate::engine::owned_provider::OwnedSnapshot,
    crate::engine::manager_settings::OperationFailure,
> {
    tauri::async_runtime::spawn_blocking(move || {
        run_manager_operation(|| {
            crate::engine::owned_provider::OwnedTestBackend { request: &request }.execute()
        })
    })
    .await
    .map_err(|error| {
        crate::engine::manager_settings::OperationFailure::from(io::Error::other(error.to_string()))
    })?
    .map_err(crate::engine::manager_settings::OperationFailure::from)
}

#[tauri::command]
pub async fn desktop_select_owned_test_session() -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        run_manager_operation(|| {
            if !cfg!(debug_assertions) {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "Developer session selection is unavailable in production",
                ));
            }
            Ok(rfd::FileDialog::new()
                .set_title("Select the activated local engine test session")
                .pick_folder()
                .map(|p| p.to_string_lossy().into_owned()))
        })
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| error.to_string())
}

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
        io::Error::new(
            io::ErrorKind::WouldBlock,
            "Another MIVUBI desktop operation is running; retry after it finishes",
        )
    })?;
    operation()
}

fn run_manager_operation<T>(operation: impl FnOnce() -> io::Result<T>) -> io::Result<T> {
    admit_resource_operation(&MANAGER_RESOURCE_LOCK, operation)
}

// CloseRequested must use the SAME existing manager admission gate as
// CloseOne/Launch/Present. Acquire it BEFORE restoring any native HWND so
// another action cannot mutate presentation between release and CloseAll.
// On contention we fail before modifying windows or game processes; the
// existing window-event error path preserves the open manager for retry.
pub(crate) fn close_all_managed() -> Result<(), (bool, io::Error)> {
    // Only one existing resource admission gate governs both Exit and
    // CloseOne. Record whether workspace release was ATTEMPTED (not whether
    // it succeeded): a failed release may already have moved native windows.
    // A gate refusal, by contrast, makes no game/window changes at all.
    let mut release_attempted = false;
    let outcome = run_manager_operation(|| {
        release_attempted = true;
        let release=crate::engine::workspace_surface::release_for_close();
        #[cfg(feature = "lazy-tester-package")]
        {
            let recovery_issue=release.err();
            if let Some(ref error)=recovery_issue {
                // The previous strict `?` created a circular dependency:
                // a live game with its HWND already gone made Restore fail,
                // which prevented us ever asking that game to close.
                // Never race a helper still performing native window writes.
                if !crate::engine::workspace_surface::allow_game_close_after_failed_restore()? {
                    return Err(io::Error::other(format!(
                        "Window restoration unconfirmed and recovery helper still active or unverified: {error}. Minecraft normal close was not requested."
                    )));
                }
            }
            let close=crate::engine::native_clients_session::request(
                crate::engine::native_clients_session::SessionRequest::CloseAll,
            ).map(|_| ());
            match (recovery_issue,close) {
                (Some(recovery),Err(error))=>Err(io::Error::other(format!(
                    "Window restoration unconfirmed: {recovery}. Minecraft normal close is also pending: {error}"
                ))),
                (_,result)=>result,
            }
        }
        #[cfg(not(feature = "lazy-tester-package"))]
        {
            release
        }
    });
    outcome.map_err(|error| (release_attempted, error))
}

// Explicit inspection of all four stable IDs. The two existing Windows
// collectors run sequentially beneath ONE manager resource lock. No new
// inspector service, process monitor, or shared runtime state is created.
#[tauri::command]
pub async fn desktop_inspect_instance_overview() -> Result<InstanceOverview, String> {
    tauri::async_runtime::spawn_blocking(|| {
        run_manager_operation(instance_discovery::inspect_instance_overview)
    })
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
    tauri::async_runtime::spawn_blocking(|| {
        run_manager_operation(instance_focus_events::start_observer)
    })
    .await
    .map_err(|error| format!("Focus observer startup task failed: {error}"))?
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn desktop_stop_focus_observer() -> Result<ForegroundEventObserverStatus, String> {
    tauri::async_runtime::spawn_blocking(|| {
        run_manager_operation(instance_focus_events::stop_observer)
    })
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
    tauri::async_runtime::spawn_blocking(|| {
        run_slot_operation(
            "01",
            SlotOperation::Launch,
            native_clients::open_native_minecraft,
        )
    })
    .await
    .map_err(|error| format!("Desktop launch task failed: {error}"))?
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn desktop_inspect_native_candidates(
) -> Result<native_clients::NativeProcessObservation, String> {
    tauri::async_runtime::spawn_blocking(|| {
        run_slot_operation(
            "01",
            SlotOperation::Inspect,
            native_clients::inspect_native_candidates,
        )
    })
    .await
    .map_err(|error| format!("Native inspection task failed: {error}"))?
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn desktop_select_education_exe() -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        run_slot_operation(
            "02",
            SlotOperation::SelectExecutable,
            native_clients::select_education_exe,
        )
    })
    .await
    .map_err(|e| format!("File dialog failed: {e}"))?
    .map_err(|e| e.to_string())
}
#[tauri::command]
pub async fn desktop_request_sandbox_client_02(
    executable: String,
    licensed: bool,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        run_slot_operation("02", SlotOperation::Launch, || {
            native_clients::request_sandbox_client_02(executable, licensed)
        })
    })
    .await
    .map_err(|e| format!("Isolated launch failed: {e}"))?
    .map_err(|e| e.to_string())
}

// One-shot read-only GetForegroundWindow snapshot, using the same full
// fresh Client 02 identity verification as the existing Focus/Place commands.
#[tauri::command]
pub async fn desktop_inspect_instance_focus(
    target: native_clients::ProcessIdentityCandidate,
) -> Result<InstanceFocusObservation, String> {
    tauri::async_runtime::spawn_blocking(move || {
        run_slot_operation("02", SlotOperation::Inspect, || {
            instance_focus::inspect_instance_focus(target)
        })
    })
    .await
    .map_err(|error| format!("Foreground inspection task failed: {error}"))?
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn desktop_focus_observed_client_02(
    target: native_clients::ProcessIdentityCandidate,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        run_slot_operation("02", SlotOperation::Focus, || {
            native_clients::focus_observed_client_02(target)
        })
    })
    .await
    .map_err(|error| format!("Client 02 focus task failed: {error}"))?
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn desktop_place_observed_client_02(
    target: native_clients::ProcessIdentityCandidate,
    side: native_clients::Client02PlacementSide,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        run_slot_operation("02", SlotOperation::Place, || {
            native_clients::place_observed_client_02(target, side)
        })
    })
    .await
    .map_err(|error| format!("Client 02 placement task failed: {error}"))?
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn desktop_inspect_sandbox_client_02(
) -> Result<native_clients::SandboxieClientObservation, String> {
    tauri::async_runtime::spawn_blocking(|| {
        run_slot_operation(
            "02",
            SlotOperation::Inspect,
            native_clients::inspect_sandbox_client_02,
        )
    })
    .await
    .map_err(|error| format!("Sandboxie inspection task failed: {error}"))?
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn desktop_inspect_owned_engine_candidate(
    directory: String,
) -> Result<crate::engine::owned_isolation::OwnedCandidateInspection, String> {
    tauri::async_runtime::spawn_blocking(move || {
        run_manager_operation(|| native_clients::inspect_owned_engine_candidate(directory))
    })
    .await
    .map_err(|error| format!("Owned engine candidate inspection failed: {error}"))?
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn desktop_select_owned_engine_candidate_directory() -> Result<Option<String>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        run_manager_operation(native_clients::select_owned_candidate_directory)
    })
    .await
    .map_err(|error| format!("Owned candidate directory picker failed: {error}"))?
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn desktop_inspect_owned_engine_host(
) -> Result<crate::engine::owned_isolation::OwnedHostInspection, String> {
    tauri::async_runtime::spawn_blocking(|| {
        run_manager_operation(native_clients::inspect_owned_engine_host)
    })
    .await
    .map_err(|error| format!("Owned runtime host inspection failed: {error}"))?
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn desktop_discover_existing_virtuals(
) -> Result<crate::engine::virtual_inventory::VirtualBoxInventory, String> {
    tauri::async_runtime::spawn_blocking(|| {
        run_manager_operation(crate::engine::virtual_inventory::discover_existing_virtuals)
    })
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
        })
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::WouldBlock);
        assert!(
            !invoked.get(),
            "must not spawn an overlapping Windows helper"
        );
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
            })
            .unwrap_err();
            assert!(matches!(
                error.kind(),
                io::ErrorKind::Unsupported | io::ErrorKind::InvalidInput
            ));
        }
        assert!(!invoked.get(), "no unavailable slot may run a helper");
        assert_eq!(
            run_slot_operation("02", SlotOperation::Inspect, || Ok("read-only")).unwrap(),
            "read-only"
        );
    }

    #[test]
    fn contending_close_sequence_refuses_before_native_window_mutation() {
        let gate = Mutex::new(());
        let currently_closing_one = gate.lock().unwrap();
        let restored = Cell::new(false);
        let result = admit_resource_operation(&gate, || {
            restored.set(true);
            Ok(())
        });
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::WouldBlock);
        assert!(!restored.get(), "Full exit must not restore any HWND while a slot close owns admission");
        drop(currently_closing_one);
        admit_resource_operation(&gate, || {
            restored.set(true);
            Ok(())
        }).unwrap();
        assert!(restored.get());
    }

    #[test]
    fn a_failed_helper_releases_admission_for_the_next_explicit_action() {
        let gate = Mutex::new(());
        let error = admit_resource_operation(&gate, || -> io::Result<()> {
            Err(io::Error::other("collector failed"))
        })
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::Other);
        assert_eq!(
            admit_resource_operation(&gate, || Ok("ready")).unwrap(),
            "ready"
        );
    }
}
