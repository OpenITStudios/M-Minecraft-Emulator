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
            ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN
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
    // Inert host only: no service-owned IO, privileged worker or game.
    if !report(&mut state, Signal::Ready, status)? {
        return Ok(());
    }
    if stop_rx.recv().is_err() {
        let _ = report(&mut state, Signal::Fatal, status)?;
        return Ok(());
    }
    if !report(&mut state, Signal::RequestStop, status)? {
        return Ok(());
    }
    // A future IPC worker must drain/cancel all its IO BEFORE CompleteStop.
    let _ = report(&mut state, Signal::CompleteStop, status)?;
    Ok(())
}

fn run_service() -> windows_service::Result<()> {
    // SCM callbacks are nonblocking. Multiple Stop/Shutdown controls
    // coalesce to one bounded channel message; no game operations occur.
    let (stop_tx, stop_rx) = mpsc::sync_channel::<()>(1);
    let handler = move |event| match event {
        ServiceControl::Stop | ServiceControl::Shutdown => {
            let _ = stop_tx.try_send(());
            ServiceControlHandlerResult::NoError
        }
        ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
        _ => ServiceControlHandlerResult::NotImplemented,
    };
    let status = service_control_handler::register(NAME, handler)?;
    let result = service_loop(&status, stop_rx);
    if result.is_err() {
        // Best effort terminal failure, never an implicit restart.
        let _ = status.set_service_status(service_status(Phase::Failed));
    }
    result
}
