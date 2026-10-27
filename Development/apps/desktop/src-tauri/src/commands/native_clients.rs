use crate::engine::native_clients;
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


type SurfaceFailure = crate::engine::manager_settings::OperationFailure;

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
