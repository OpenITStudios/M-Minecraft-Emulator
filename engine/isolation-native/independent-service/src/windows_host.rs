//! Windows SCM process host only. No CreateService, self-install,
//! service privileges, named pipes, or Minecraft launch.

use std::{ffi::OsString, sync::mpsc::{self, Receiver}, time::Duration};
use windows_service::{
    define_windows_service, service_dispatcher,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode,
        ServiceState, ServiceStatus, ServiceType,
    },
    service_control_handler::{
        self, ServiceControlHandlerResult, ServiceStatusHandle,
    },
};
use crate::lifecycle::{HostLifecycle, Phase, Signal};

const NAME: &str = "MIVUBIIndependentHost";
define_windows_service!(ffi_service_main, service_main);

pub(crate) fn run_dispatcher() {
    // A normal CLI invocation must NOT fall back to a local daemon.
    if service_dispatcher::start(NAME, ffi_service_main).is_err() {
        eprintln!("MIVUBI service host: SCM dispatch unavailable.");
        std::process::exit(1);
    }
}

fn service_main(_arguments: Vec<OsString>) {
    let _ = run_service();
}

/// A default service host remains inert: only the explicitly compiled
/// readonly-ipc-pilot subscribes to SCM session-change notifications.
fn running_controls() -> ServiceControlAccept {
    let controls = ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN;
    #[cfg(feature = "readonly-ipc-pilot")]
    {
        controls | ServiceControlAccept::SESSION_CHANGE
    }
    #[cfg(not(feature = "readonly-ipc-pilot"))]
    {
        controls
    }
}

fn service_status(phase: Phase) -> ServiceStatus {
    let pending = matches!(phase, Phase::Starting | Phase::Stopping);
    ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: match phase {
            Phase::Cold | Phase::Starting => ServiceState::StartPending,
            Phase::Running => ServiceState::Running,
            Phase::Stopping => ServiceState::StopPending,
            Phase::Stopped | Phase::Failed => ServiceState::Stopped,
        },
        controls_accepted: if phase == Phase::Running {
            running_controls()
        } else {
            ServiceControlAccept::empty()
        },
        exit_code: ServiceExitCode::Win32(if phase == Phase::Failed { 1 } else { 0 }),
        checkpoint: u32::from(pending),
        wait_hint: if pending { Duration::from_secs(5) } else { Duration::ZERO },
        process_id: None,
    }
}

fn report(
    state: &mut HostLifecycle,
    change: Signal,
    status: &ServiceStatusHandle,
) -> windows_service::Result<bool> {
    match state.advance(change) {
        Ok(phase) => {
            status.set_service_status(service_status(phase))?;
            Ok(true)
        }
        Err(_) => {
            // A programmer error cannot promote a service to RUNNING.
            status.set_service_status(service_status(Phase::Failed))?;
            Ok(false)
        }
    }
}

fn service_loop(
    status: &ServiceStatusHandle,
    stop_rx: Receiver<()>,
) -> windows_service::Result<()> {
    let mut state = HostLifecycle::new();
    if !report(&mut state, Signal::Begin, status)? {
        return Ok(());
    }
    // The opt-in pilot must bind its ACL-protected *service-owned* endpoint
    // before reporting RUNNING. Default builds remain inert.
    #[cfg(feature = "readonly-ipc-pilot")]
    let mut pilot = match crate::ipc_worker::ReadOnlyPipeWorker::bind() {
        Ok(value) => value,
        Err(_) => {
            let _ = report(&mut state, Signal::Fatal, status)?;
            return Ok(());
        }
    };
    if !report(&mut state, Signal::Ready, status)? {
        return Ok(());
    }
    // Publish STOP_PENDING as soon as the worker notices the SCM stop
    // signal and BEFORE waiting for overlapped I/O cancellation to drain.
    // The worker must not return success without announcing that state.
    #[cfg(feature = "readonly-ipc-pilot")]
    let clean_stop = {
        let mut announced = false;
        let outcome = pilot.run_until_stop(&stop_rx, &mut || {
            if !announced {
                let accepted = report(&mut state, Signal::RequestStop, status)
                    .map_err(|_| std::io::Error::other("SCM STOP_PENDING report failed"))?;
                if !accepted {
                    return Err(std::io::Error::other("Invalid SCM stop transition"));
                }
                announced = true;
            }
            Ok(())
        });
        outcome.is_ok() && announced
    };
    #[cfg(not(feature = "readonly-ipc-pilot"))]
    let clean_stop = stop_rx.recv().is_ok();
    if !clean_stop {
        let _ = report(&mut state, Signal::Fatal, status)?;
        return Ok(());
    }
    #[cfg(not(feature = "readonly-ipc-pilot"))]
    if !report(&mut state, Signal::RequestStop, status)? {
        return Ok(());
    }
    // The opt-in pilot returns only after all pending pipe I/O has
    // completed or cancellation has been acknowledged by Windows.
    // Production multi-worker IPC and per-client authentication are absent.
    let _ = report(&mut state, Signal::CompleteStop, status)?;
    Ok(())
}

/// The SCM callback owns no logon identity and grants no new capability.
/// For the SOURCE-ONLY pilot, *every* session-change control invalidates
/// the whole diagnostic pipe by requesting service STOP, including logon,
/// logoff, lock, unlock, console/RDP connect/disconnect. Ignoring the
/// notification's numeric SessionId intentionally prevents converting a
/// control event into authentication or a new per-logon DACL grant.
///
/// This is a conservative kill switch, NOT production per-session
/// revocation. SCM can miss events before service registration; the
/// existing per-request WTS/token recheck remains mandatory.
fn route_control(
    event: ServiceControl,
    stop_tx: &mpsc::SyncSender<()>,
) -> ServiceControlHandlerResult {
    match event {
        ServiceControl::Stop | ServiceControl::Shutdown => {
            let _ = stop_tx.try_send(());
            ServiceControlHandlerResult::NoError
        }
        #[cfg(feature = "readonly-ipc-pilot")]
        ServiceControl::SessionChange(_) => {
            // Channel full means an earlier stop/revocation is already
            // queued. Do not block the SCM control-handler thread.
            let _ = stop_tx.try_send(());
            ServiceControlHandlerResult::NoError
        }
        ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
        _ => ServiceControlHandlerResult::NotImplemented,
    }
}

fn run_service() -> windows_service::Result<()> {
    // SCM callbacks are nonblocking. STOP, SHUTDOWN and (pilot-only)
    // SESSION_CHANGE coalesce to one bounded message; no account, game,
    // token inspection or pipe cancellation runs on the SCM callback thread.
    let (stop_tx, stop_rx) = mpsc::sync_channel::<()>(1);
    let handler = move |event| route_control(event, &stop_tx);
    let status = service_control_handler::register(NAME, handler)?;
    let result = service_loop(&status, stop_rx);
    if result.is_err() {
        // Best effort terminal failure, never an implicit restart.
        let _ = status.set_service_status(service_status(Phase::Failed));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_change_is_accepted_only_by_opt_in_service() {
        let running = service_status(Phase::Running);
        assert!(running.controls_accepted.contains(
            ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN
        ));
        assert_eq!(
            running.controls_accepted.contains(ServiceControlAccept::SESSION_CHANGE),
            cfg!(feature = "readonly-ipc-pilot"),
        );
        for phase in [Phase::Starting, Phase::Stopping, Phase::Stopped, Phase::Failed] {
            assert!(service_status(phase).controls_accepted.is_empty());
        }
    }

    #[cfg(feature = "readonly-ipc-pilot")]
    #[test]
    fn any_session_change_coalesces_with_stop_and_shutdown() {
        use windows_service::service::{
            SessionChangeParam, SessionChangeReason, SessionNotification,
        };
        let (send, recv) = mpsc::sync_channel::<()>(1);
        let event = ServiceControl::SessionChange(SessionChangeParam {
            reason: SessionChangeReason::SessionLogoff,
            notification: SessionNotification { size: 8, session_id: 7 },
        });
        assert!(matches!(
            route_control(event, &send),
            ServiceControlHandlerResult::NoError
        ));
        // A repeated or concurrent stop cannot fill an unbounded queue.
        assert!(matches!(
            route_control(ServiceControl::Stop, &send),
            ServiceControlHandlerResult::NoError
        ));
        assert!(recv.try_recv().is_ok());
        assert!(recv.try_recv().is_err());
    }
}
