use crate::engine::native_clients;
use std::{io, sync::Mutex};

// One admission point for heavyweight MIVUBI desktop requests. The caller
// may not bypass this by invoking Tauri commands directly while another
// probe, file hash, dialog, launch or window operation is in flight.
// This limits manager-created CPU/RAM peaks, not Minecraft CPU or GPU usage.
static MANAGER_RESOURCE_LOCK: Mutex<()> = Mutex::new(());

// Compile-time backend and approved capture capabilities, not game readiness.
// No lifecycle lock, worker, disk access, process discovery or credentials.
#[derive(serde::Serialize)]
#[serde(rename_all="camelCase")]
pub struct StartupCapabilities {
    backend: native_clients::NativeBackendSupport,
    diagnostic: bool,
    screenshot: bool,
    recording: bool,
}

#[tauri::command]
pub fn lazy_tester_startup_capabilities() -> StartupCapabilities {
    let backend = native_clients::native_backend_support();
    let diagnostic = cfg!(all(windows, target_arch = "x86_64", feature = "lazy-tester-package"))
        && crate::app_bootstrap::startup_diagnostic_mode();
    let capture_allowed = backend == native_clients::NativeBackendSupport::WindowsNative && !diagnostic;
    StartupCapabilities {
        backend, diagnostic,
        screenshot: capture_allowed && crate::engine::capture::screenshot_available(),
        recording: capture_allowed && crate::engine::capture::recording_available(),
    }
}

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
    passive_idle: Option<bool>,
) -> Result<Option<crate::engine::native_clients_session::SessionSnapshot>, String> {
    let passive_idle=passive_idle.unwrap_or(false);
    if passive_idle && !matches!(request,crate::engine::native_clients_session::SessionRequest::Inspect | crate::engine::native_clients_session::SessionRequest::InspectSettings) {
        return Err("Passive idle hint is restricted to Inspect or InspectSettings".into());
    }
    if crate::app_bootstrap::startup_diagnostic_mode() && !matches!(request,crate::engine::native_clients_session::SessionRequest::Inspect) {return Err("Startup diagnostic cannot launch or modify games".into());}
    // Retired individual graphics controls cannot bypass the global desktop policy.
    if matches!(request, crate::engine::native_clients_session::SessionRequest::ApplyGraphicsPreset { .. }) {
        return Err("Individual graphics preferences are retired; all launchers use the optimized preset.".into());
    }
    // Optional per-profile performance experiments are no longer desktop controls.
    if matches!(request, crate::engine::native_clients_session::SessionRequest::DisableImprovedInputResponse { .. } | crate::engine::native_clients_session::SessionRequest::EnableTextureStreaming { .. } | crate::engine::native_clients_session::SessionRequest::DisableTextureStreaming { .. }) {
        return Err("Individual performance controls are retired.".into());
    }
    // Untrusted webview code may not forge agent-reported activity.
    if request.is_agent_report() {
        return Err("Desktop cannot publish agent activity or arbitrate Testing runs".into());
    }
    if request.is_agent_target() {
        return Err("Private target inspection is not a desktop session command".into());
    }
    // The desktop's normal session command must never be an alternate
    // admission path for experimental game hooks. Only the explicitly
    // opted-in, same-image native agent CLI may send these requests
    // through the already-authenticated controller transport.
    if request.is_experimental_input() {
        return Err("Experimental input is restricted to the opt-in native agent CLI".into());
    }
    tauri::async_runtime::spawn_blocking(move || run_manager_operation(|| {
        match &request {
            crate::engine::native_clients_session::SessionRequest::CloseOne{slot}=>crate::engine::capture::require_recording_idle(Some(*slot))?,
            crate::engine::native_clients_session::SessionRequest::CloseAll=>crate::engine::capture::require_recording_idle(None)?,
            _=>{},
        }
        // Null means no new observation; never fabricate a stopped receipt.
        // Startup diagnostic and every ordinary command retain full admission.
        if passive_idle && !crate::app_bootstrap::startup_diagnostic_mode() &&
            !crate::engine::native_clients_session::passive_idle_needs_inspect()? {
            return Ok(None);
        }
        crate::engine::native_clients_session::request(request).map(Some)
    })).await.map_err(|_| "Session task failed".to_string())?.map_err(|error| error.to_string())
}

// Read only the current process's bounded report, not a native session request.
#[tauri::command]
pub(crate) fn lazy_tester_issue_report() -> Result<Option<crate::engine::issue_report::NativeIssueReport>, &'static str> {
    crate::engine::issue_report::read()
}

// Fixed report destination, not caller-selected file access.
#[tauri::command]
pub(crate) async fn lazy_tester_checkpoint_issues(ui: String) -> Result<(), &'static str> {
    #[cfg(windows)]
    {
        tauri::async_runtime::spawn_blocking(move || crate::engine::report_store::checkpoint_ui(&ui))
            .await.map_err(|_| "Issue report checkpoint task unavailable")?
    }
    #[cfg(not(windows))]
    { let _ = ui; Err("Issue report checkpoint unavailable") }
}

// This receipt distinguishes a refusal before managed-window mutation from
// an unconfirmed close AFTER the selected workspace was released. The
// retained session controller, never the UI, validates live game ownership.
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
        crate::engine::capture::require_recording_idle(Some(slot))?;
        use crate::engine::native_clients_session::{request,SessionRequest};
        // A preliminary Inspect would reap and sync journals from ALL slots
        // before the selected close. The original workspace owner only
        // releases exact, previously admitted leases for this selected slot.
        // The retained game controller below is the sole authority that
        // checks whether this slot is running before requesting normal exit.
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
                    issue:Some(format!("Window restoration unconfirmed: {recovery}. Minecraft close was not confirmed: {close}")),
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
        let (snapshot, issue) = match result {
            Ok(snapshot) => (Some(snapshot), None),
            Err(error) => (None, Some(error.to_string())),
        };
        Ok(SelectedCloseReport {
            schema:1,presentation_released:true,
            snapshot,
            issue,
        })
    })).await.map_err(|_|"Selected Minecraft close task failed".to_string())?
        .map_err(|error|error.to_string())
}

#[tauri::command]
pub async fn lazy_tester_screenshot(slot:u32,expected:crate::engine::process_identity::OwnedProcess)
    ->Result<Option<crate::engine::capture::ScreenshotReceipt>,String>{
    if crate::app_bootstrap::startup_diagnostic_mode(){return Err("Startup diagnostic cannot capture games".into());}
    tauri::async_runtime::spawn_blocking(move||run_manager_operation(||crate::engine::capture::save_screenshot(slot,expected)))
        .await.map_err(|_|"Screenshot task failed".to_string())?
        .map_err(|error|error.to_string())
}

#[tauri::command]
pub fn lazy_tester_recording_status()->Result<Vec<crate::engine::capture::RecordingReceipt>,String>{
    crate::engine::capture::recording_status().map_err(|e|e.to_string())
}
#[tauri::command]
pub async fn lazy_tester_start_recording(slot:u32,expected:crate::engine::process_identity::OwnedProcess)->Result<bool,String>{
    if crate::app_bootstrap::startup_diagnostic_mode(){return Err("Startup diagnostic cannot record games".into());}
    tauri::async_runtime::spawn_blocking(move||run_manager_operation(||crate::engine::capture::start_recording(slot,expected)))
        .await.map_err(|_|"Recording start task failed".to_string())?.map_err(|e|e.to_string())
}
#[tauri::command]
pub async fn lazy_tester_stop_recording(slot:u32)->Result<(),String>{
    if crate::app_bootstrap::startup_diagnostic_mode(){return Err("Startup diagnostic cannot record games".into());}
    tauri::async_runtime::spawn_blocking(move||run_manager_operation(||crate::engine::capture::stop_recording(slot)))
        .await.map_err(|_|"Recording stop task failed".to_string())?.map_err(|e|e.to_string())
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
        let snapshot = crate::engine::native_clients_session::request_desktop_target(slot)?;
        let target = snapshot.profiles.into_iter().find(|p| p.slot == slot && p.state=="RUNNING")
            .and_then(|p| p.process).filter(|p| p.responding)
            .ok_or_else(|| io::Error::other("Minecraft has no verified responsive game window"))?;
        // Slot attribution comes from retained original bootstrap handles, never UI PID/title.
        let authority = "LAZY_TESTER_MANAGED_SESSION".to_string();
        let client = format!("{slot:02}");
        // Fresh session admission already established the target. The native
        // presentation owner reuses a retained checkpoint or prepares one.
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
        // No source can be acquired when there are no requested tiles and
        // no retained workspace lease. Still run the native layout/clear
        // owner; a contended or retained lease requires fresh observation.
        let profiles = if tiles.is_empty() && !crate::engine::workspace_surface::is_active() {
            Vec::new()
        } else {
            let snapshot = crate::engine::native_clients_session::request(
                crate::engine::native_clients_session::SessionRequest::Inspect)?;
            snapshot.profiles.into_iter().filter_map(|profile|
                profile.process.map(|process|(profile.slot,process))).collect()
        };
        crate::engine::workspace_surface::configure_previews(window,viewport,tiles,profiles)
    })).await.map_err(|_| "Preview task failed".to_string())?.map_err(|error|error.to_string())
}

#[tauri::command]
pub async fn lazy_tester_clear_previews() -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(|| run_manager_operation(
        crate::engine::workspace_surface::clear_previews))
        .await.map_err(|_|"Preview cleanup failed".to_string())?.map_err(|error|error.to_string())
}


type SurfaceFailure = crate::engine::manager_settings::OperationFailure;

fn surface_failure(error: impl std::fmt::Display) -> SurfaceFailure {
    std::io::Error::other(error.to_string()).into()
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
    target: Option<crate::engine::process_identity::OwnedProcess>,
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
        crate::engine::capture::require_recording_idle(None)?;
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

#[cfg(test)]
mod resource_admission_tests {
    use super::admit_resource_operation;
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
