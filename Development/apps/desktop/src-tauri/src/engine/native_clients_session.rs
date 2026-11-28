//! Lazy Tester transport. The existing bootstrap owns games in one retained helper.
// Stable cross-build existence gate; keep the original Windows object name.
#[cfg(windows)]
pub(crate) const CONTROLLER_GATE_NAME: &str = "Local\\MivubiEducationStartupObservation";

use std::{
    io::{self, BufRead},
    sync::Mutex,
};

// Domain modules remain private; existing callers retain their session API paths.
mod agent_activity;
pub use self::agent_activity::{AgentActivityItem, AgentActivityReport, AgentProbeCounts};
mod controller_testing_runs;
pub use self::controller_testing_runs::{TestingRunInspection, TestingRunView};
pub(crate) use self::controller_testing_runs::{NativeTestingRunBook, valid_testing_id};
#[cfg(feature="mc02-native-input")]
mod controller_ai_input_leases;
#[cfg(feature="mc02-native-input")]
pub use self::controller_ai_input_leases::{AgentControlMode, AiInputFrame};
#[cfg(feature="mc02-native-input")]
pub(crate) use self::controller_ai_input_leases::{
    AiLeaseBook, Mc02BatchOrigin, NativeInputIntent, admit_native_input_frame,
    current_ai_filetime, valid_ai_writer_id, validate_ai_acquire_epoch,
    validate_ai_input_group, validate_ai_lease_slots, AI_WRITER_LEASE_MS,
};
mod session_protocol;
pub use self::session_protocol::{
    CacheStorage, DiskSpace, FullClearStorage, GameSettingsStorage, ProfileSession,
    ProfileStorage, ResourcePackStorage, SelectiveCategory, SelectiveInventory,
    SelectiveInventoryEntry, SelectiveReviewReceipt, SelectiveSelection,
    SessionRequest, SessionSnapshot, SettingsReport, WorldPackDeclarations,
};
pub(crate) use self::session_protocol::SessionReply;
use self::session_protocol::SESSION_FRAME_LIMIT;
#[cfg(windows)]
pub(super) use self::session_protocol::validate_request;

#[cfg(windows)]
pub(crate) mod pipe;

// The private one-shot receiver is the only local identity of an IPC
// request still in flight. Its bytes cannot be attributed to any later
// request, and it is never retried or published as a new response.
// Do not introduce a separate transaction owner or public wire token.
#[cfg(windows)]
struct PendingRequest {
    replies: std::sync::mpsc::Receiver<io::Result<SessionReply>>,
    worker: std::thread::JoinHandle<()>,
}

#[cfg(windows)]
impl PendingRequest {
    // Only this original client worker is targeted, never a game or the
    // controller actor. A cancel request does not prove completion.
    fn request_io_cancel(&self) {
        use std::os::windows::io::AsRawHandle;
        if !self.worker.is_finished() {
            unsafe { windows_sys::Win32::System::IO::CancelSynchronousIo(self.worker.as_raw_handle() as _) };
        }
    }
}

#[cfg(windows)]
enum PendingWorkerWait {
    // A response belongs to the unique original worker only.
    Completed(io::Result<SessionReply>),
    StillRunning,
    ResultChannelLost,
}
#[cfg(windows)]
fn classify_pending_worker_wait(
    outcome:Result<io::Result<SessionReply>,std::sync::mpsc::RecvTimeoutError>
)->PendingWorkerWait {
    match outcome {
        Ok(result)=>PendingWorkerWait::Completed(result),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout)=>
            PendingWorkerWait::StillRunning,
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected)=>
            PendingWorkerWait::ResultChannelLost,
    }
}
#[cfg(windows)]
struct Controller {
    pending: Option<PendingRequest>,
    uncertain: bool,
}
#[cfg(windows)]
impl Controller {
    // Only after the original receiver reports a result or disconnect.
    fn join_finished_request(&mut self) {
        let pending = self.pending.take().expect("Original pending request");
        let _ = pending.worker.join();
    }

    fn drain_previous_request(&mut self) -> io::Result<()> {
        if let Some(pending) = &self.pending {
            match pending.replies.try_recv() {
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    // Retry cancellation only, never the original request.
                    // The same worker remains retained until a result/disconnect.
                    pending.request_io_cancel();
                    return Err(io::Error::new(
                        io::ErrorKind::WouldBlock,
                        "The previous session request is still running; games are preserved",
                    ));
                }
                Ok(_) | Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    // A previous reply never becomes a new caller's receipt.
                    self.join_finished_request();
                    self.uncertain = true;
                }
            }
        }
        Ok(())
    }
}
#[cfg(windows)]
static CONTROLLER: Mutex<Controller> = Mutex::new(Controller {
    pending: None,
    uncertain: false,
});

#[cfg(windows)]
fn admit_controller(
    controller: &Mutex<Controller>,
) -> io::Result<std::sync::MutexGuard<'_, Controller>> {
    // The existing request owns this gate for the entire bounded IPC wait.
    // A second caller must never queue behind that wait (up to 45 seconds)
    // when Windows is trying to review a close or another manager action.
    // Refusal preserves the original worker and all retained game handles.
    match controller.try_lock() {
        Ok(owner) => Ok(owner),
        Err(std::sync::TryLockError::WouldBlock) => Err(io::Error::new(
            io::ErrorKind::WouldBlock,
            "Another session request is still in progress; games are preserved",
        )),
        Err(std::sync::TryLockError::Poisoned(_)) => {
            Err(io::Error::other("Session transport unavailable"))
        }
    }
}

// Only the original session Inspect route is allowed to clear the transport
// uncertainty gate, and only when its fresh reply succeeds. Settings and
// storage observations may request that reconciliation first; their own
// receipt never proves an older mutation succeeded.
#[cfg(windows)]
fn can_reconcile_pending_transport(request:&SessionRequest)->bool {
    matches!(request,SessionRequest::Inspect)
}


#[cfg(windows)]
pub fn request(request: SessionRequest) -> io::Result<SessionSnapshot> {
    request_inner(request, false)
}

// Only trusted Rust desktop presentation and capture use this entry.
// The generic webview session command still refuses the private wire route.
// Reuse the same actor, exact receipt and transport owner; never replay.
#[cfg(windows)]
pub(crate) fn request_desktop_target(slot:u32) -> io::Result<SessionSnapshot> {
    request_inner(SessionRequest::InspectAgentTarget{slot}, true)
}
#[cfg(not(windows))]
pub(crate) fn request_desktop_target(slot:u32) -> io::Result<SessionSnapshot> {
    request(SessionRequest::InspectAgentTarget{slot})
}

#[cfg(windows)]
fn desktop_observation_request(request:SessionRequest, desktop_target:bool, uncertain:bool) -> SessionRequest {
    if desktop_target && uncertain && request.is_agent_target() {
        SessionRequest::Inspect
    } else {
        request
    }
}

#[cfg(windows)]
fn read_needs_reconciliation(request:&SessionRequest, uncertain:bool)->bool {
    uncertain && matches!(request, SessionRequest::InspectSettings |
        SessionRequest::InspectStorage | SessionRequest::InspectProfileStorage { .. })
}

#[cfg(windows)]
fn request_inner(request: SessionRequest, desktop_target:bool) -> io::Result<SessionSnapshot> {
    validate_request(&request)?;
    let mut owner = admit_controller(&CONTROLLER)?;
    owner.drain_previous_request()?;
    // A prior uncertain mutation still requires the original full Inspect.
    // Decide after draining while holding the existing admission gate.
    let request=desktop_observation_request(request,desktop_target,owner.uncertain);
    let deadline=std::time::Instant::now()+std::time::Duration::from_secs(45);
    let result = (|| {
        if read_needs_reconciliation(&request,owner.uncertain) {
            // Both serial transactions share the original deadline. Never replay.
            request_locked(&mut owner,SessionRequest::Inspect,deadline)?;
        }
        request_locked(&mut owner,request,deadline)
    })();
    drop(owner); // Report publication must not hold the native request gate.
    super::report_store::checkpoint_if_due();
    result
}

#[cfg(windows)]
fn issue_context(request: &SessionRequest) -> super::issue_report::Context {
    use SessionRequest::*;
    use super::issue_report::Operation;
    let (operation, slot) = match request {
        Inspect => (Operation::Inspect, None),
        InspectAgentTarget {slot,..} => (Operation::Target, Some(*slot)),
        PublishAgentActivity {..} => (Operation::AgentActivity, None),
        TestingReserve {slot,..} => (Operation::TestingReserve, Some(*slot)),
        TestingFinalize {slot,..} => (Operation::TestingFinalize, Some(*slot)),
        TestingInspect {slot,..} => (Operation::TestingInspect, Some(*slot)),
        InspectSettings => (Operation::InspectSettings, None),
        ApplyPreset => (Operation::ApplyPreset, None),
        ApplyGraphicsPreset {slot,..} => (Operation::ApplyGraphicsPreset, Some(*slot)),
        SavePerformance {..} => (Operation::SavePerformance, None),
        SaveAudio {..} => (Operation::SaveAudio, None),
        SaveStoragePreferences {..} => (Operation::SaveStoragePreferences, None),
        InspectStorage => (Operation::InspectStorage, None),
        InspectProfileStorage {slot,..} => (Operation::InspectProfileStorage, Some(*slot)),
        InspectSelectiveInventory {slot,..} => (Operation::InspectSelectiveInventory, Some(*slot)),
        ReviewSelectiveInventory {slot,..} => (Operation::ReviewSelectiveInventory, Some(*slot)),
        ConfirmSelectedWorlds {slot,..} => (Operation::ConfirmSelectedWorlds, Some(*slot)),
        ConfirmSelectedPacks {slot,..} => (Operation::ConfirmSelectedPacks, Some(*slot)),
        OpenProfileFolder {slot,..} => (Operation::OpenProfileFolder, Some(*slot)),
        ClearProfileCache {slot,..} => (Operation::ClearProfileCache, Some(*slot)),
        ClearProfileDownloadCache {slot,..} => (Operation::ClearProfileDownloadCache, Some(*slot)),
        ResetGameSettings {slot,..} => (Operation::ResetGameSettings, Some(*slot)),
        RestoreGameSettings {slot,..} => (Operation::RestoreGameSettings, Some(*slot)),
        DisableImprovedInputResponse {slot,..} => (Operation::DisableImprovedInputResponse, Some(*slot)),
        EnableTextureStreaming {slot,..} => (Operation::EnableTextureStreaming, Some(*slot)),
        DisableTextureStreaming {slot,..} => (Operation::DisableTextureStreaming, Some(*slot)),
        ClearProfileData {slot,..} => (Operation::ClearProfileData, Some(*slot)),
        Launch {slot,..} => (Operation::Launch, Some(*slot)),
        #[cfg(feature="mc02-native-input")]
        LaunchMc02Experiment {slot,..} => (Operation::LaunchMc02Experiment, Some(*slot)),
        #[cfg(feature="mc02-native-input")]
        LaunchAiExperiment {slot,..} => (Operation::LaunchAiExperiment, Some(*slot)),
        #[cfg(feature="mc02-native-input")]
        Mc02Input {slot,..} => (Operation::Mc02Input, Some(*slot)),
        #[cfg(feature="mc02-native-input")]
        AiLeasedInput {slot,..} => (Operation::AiLeasedInput, Some(*slot)),
        #[cfg(feature="mc02-native-input")]
        AiLeaseAcquire {..} => (Operation::AiLeaseAcquire, None),
        #[cfg(feature="mc02-native-input")]
        AiLeaseRelease {..} => (Operation::AiLeaseRelease, None),
        #[cfg(feature="mc02-native-input")]
        AiInputGroup {..} => (Operation::AiInputGroup, None),
        CloseOne {slot,..} => (Operation::CloseOne, Some(*slot)),
        CloseAll => (Operation::CloseAll, None),
    };
    super::issue_report::Context { operation, slot }
}

#[cfg(windows)]
fn request_locked(owner:&mut Controller, request:SessionRequest, deadline:std::time::Instant)
    -> io::Result<SessionSnapshot> {
    let selected_target=match &request {
        SessionRequest::InspectAgentTarget{slot}=>Some(*slot),_=>None,
    };
    let inspecting = matches!(request, SessionRequest::Inspect | SessionRequest::InspectSettings) || selected_target.is_some();
    // Read-only observations remain available while a previous mutation is uncertain;
    // only a complete session Inspect may reconcile that uncertainty.
    let reconciling = can_reconcile_pending_transport(&request);
    #[cfg(feature="mc02-native-input")]
    let input_receipt_expected=request.expects_input_receipt();
    #[cfg(feature="mc02-native-input")]
    let neutral_mc02_release = matches!(request,
        SessionRequest::Mc02Input {keys:0,dx:0,dy:0,..} |
        SessionRequest::AiLeasedInput {keys:0,dx:0,dy:0,..});
    #[cfg(not(feature="mc02-native-input"))]
    let neutral_mc02_release = false;
    // A failed heartbeat IPC may leave MC02 controls held until native
    // TTL expires. Do not let that uncertainty suppress a best-effort
    // exact-owner neutral write from the SAME batch CLI. An in-flight
    // request is still refused by the pending-worker gate above: never
    // race an unresolved movement write with emergency release. All
    // active input, lifecycle and other mutations stay blocked.
    // The retained session actor still verifies original MC02
    // PID/birth/session and its opt-in channel before clearing state.
    if owner.uncertain && !inspecting && !neutral_mc02_release {
        return Err(io::Error::other(
            "Previous request outcome is uncertain. Refresh session status before another action.",
        ));
    }
    if deadline<=std::time::Instant::now() {
        return Err(io::Error::new(io::ErrorKind::TimedOut,
            "Session observation budget expired before the next request"));
    }
    use super::issue_report::{self, Code};
    let context = issue_context(&request);
    let (sender, replies) = std::sync::mpsc::sync_channel(1);
    // A failed OS thread allocation must not panic while the transport
    // mutex is held. No worker or mutation was admitted on spawn failure.
    let worker = std::thread::Builder::new().spawn(move || {
        let _ = sender.send(transact(request));
    }).map_err(|error| {
        issue_report::note(Code::IpcWorkerStartFailed, context, error.raw_os_error());
        error
    })?;
    owner.pending = Some(PendingRequest { replies, worker });
    let outcome = classify_pending_worker_wait(
        owner.pending.as_ref().unwrap().replies
            .recv_timeout(deadline.saturating_duration_since(std::time::Instant::now())));
    match outcome {
        PendingWorkerWait::Completed(result) => {
            owner.join_finished_request();
            match result {
                Ok(reply) => {
                    // Validate even cleanup-only calls which discard the
                    // returned value. Never accept an input ACK as an Inspect
                    // result either. A wrong receipt may follow publication;
                    // keep the existing uncertainty/recovery gate.
                    #[cfg(feature="mc02-native-input")]
                    if reply.as_ref().is_ok_and(|snapshot|
                       input_receipt_expected!=snapshot.is_input_receipt()) {
                        owner.uncertain=true;
                        issue_report::note(Code::IpcInputReceiptInvalid, context, None);
                        return Err(io::Error::new(io::ErrorKind::BrokenPipe,
                            "Invalid native input receipt; outcome uncertain"));
                    }
                    if let Some(slot)=selected_target {
                        if reply.as_ref().is_ok_and(|snapshot|!snapshot.is_agent_target_receipt(slot)) {
                            owner.uncertain=true;
                            issue_report::note(Code::IpcTargetReceiptInvalid, context, None);
                            return Err(io::Error::new(io::ErrorKind::BrokenPipe,
                                "Invalid selected target receipt; outcome uncertain"));
                        }
                    }
                    if reconciling {
                        if reply.as_ref().is_ok_and(|snapshot|!snapshot.is_full_inspect_receipt()) {
                            owner.uncertain=true;
                            issue_report::note(Code::IpcInspectReceiptInvalid, context, None);
                            return Err(io::Error::new(io::ErrorKind::BrokenPipe,
                                "Invalid full Inspect receipt; outcome uncertain"));
                        }
                        if reply.is_ok() {
                            if owner.uncertain {
                                // Only the original valid full Inspect clears uncertainty.
                                // Report its reconciliation, not the prior mutation outcome.
                                issue_report::reconcile_transport();
                            }
                            owner.uncertain = false;
                        }
                    }
                    reply.map_err(io::Error::other)
                }
                Err(error) => {
                    owner.uncertain = true;
                    issue_report::note(Code::IpcTransportFailed, context, error.raw_os_error());
                    Err(error)
                }
            }
        }
        PendingWorkerWait::StillRunning => {
            // The unique original worker may still execute its request.
            // Retain its receiver, buffers, peer pin and thread. Never
            // start a second worker or blindly resend its mutation.
            owner.uncertain = true;
            if let Some(pending)=&owner.pending { pending.request_io_cancel(); }
            issue_report::note(Code::IpcReplyTimeout, context, None);
            Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "Session request is still pending; refresh to observe its eventual outcome",
            ))
        }
        PendingWorkerWait::ResultChannelLost => {
            // A terminated sender is NOT a running timeout. No complete
            // original reply was received, so the mutation outcome is
            // unknown even though its local worker can now be joined.
            // The next safe Inspect still observes current native state;
            // no old reply can be synthesized or retried here.
            owner.join_finished_request();
            owner.uncertain = true;
            issue_report::note(Code::IpcReplyLost, context, None);
            Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "Original session request worker returned no result; outcome uncertain",
            ))
        }
    }
}

// Only the desktop's passive idle lane may ask whether a fresh read is
// necessary. An uncertain or pending receipt must always use full Inspect.
// A negative hint creates no SessionSnapshot and clears no uncertainty.
#[cfg(windows)]
fn idle_hint_needs_read(unsettled: bool, mut controller_exists: impl FnMut() -> io::Result<bool>,
    candidate_exists: impl FnOnce() -> io::Result<bool>) -> io::Result<bool> {
    if unsettled || controller_exists()? || candidate_exists()? {
        return Ok(true);
    }
    // A controller may have started during the candidate snapshot.
    controller_exists()
}

#[cfg(windows)]
pub(crate) fn passive_idle_needs_inspect() -> io::Result<bool> {
    let owner=admit_controller(&CONTROLLER)?;
    idle_hint_needs_read(owner.uncertain || owner.pending.is_some(),
        || existing_controller_gate(CONTROLLER_GATE_NAME),
        super::native_clients::game_candidate_exists)
}

#[cfg(not(windows))]
pub(crate) fn passive_idle_needs_inspect() -> io::Result<bool> {
    Ok(true) // Retain the original unsupported-backend refusal.
}

// Without the retained controller, even a successful IPC round trip cannot
// assign an observed game to MC01–MC05. Never publish all STOPPED (or change
// global settings) while independently observed Minecraft processes remain.
// A failed/partial OS inventory already fails closed at the caller.
fn require_absent_controller_idle(registered_process_count: usize) -> io::Result<()> {
    if registered_process_count != 0 {
        return Err(io::Error::new(io::ErrorKind::WouldBlock,
            "Minecraft processes remain but no compatible Lazy Tester controller is available. Their per-slot state is unknown; stopped status and settings changes were refused. Existing games are preserved."));
    }
    Ok(())
}

// A disappeared controller does not prove that its resumed game exited.
// Without retained per-slot ownership, CloseOne MUST NOT report success even
// when no official Minecraft process is visible. CloseAll can still confirm
// an already-idle manager exit. This changes no wire request/response schema.
fn absent_controller_close_reply(request:&SessionRequest,registered_process_count:usize)->SessionReply {
    if registered_process_count!=0 {
        return Err("Minecraft processes remain but no compatible Lazy Tester controller is available. Games are preserved; reconnect the correct controller before retrying.".into());
    }
    match request {
        SessionRequest::CloseAll => Ok(SessionSnapshot::stopped()),
        SessionRequest::CloseOne {..} => Err("No retained Minecraft controller can verify the selected profile. CloseOne was not performed; refresh session status.".into()),
        _ => Err("Unrecognized close request without a controller".into()),
    }
}

#[cfg(windows)]
fn transact(request: SessionRequest) -> io::Result<SessionReply> {
    use std::{
        io::BufReader,
        os::windows::process::CommandExt,
        process::{Command, Stdio},
    };
    let endpoint = pipe::Endpoint::current()?;
    match endpoint.connect() {
        Ok(connection) => return pipe::exchange(&endpoint, connection, &request),
        Err(error) if error.raw_os_error() == Some(2) => {}
        Err(error) => return Err(error), // Busy/denied/forged is never permission to spawn.
    }
    // Another build or the historical private runner may retain the same profiles.
    // An absent compatible endpoint must not be presented as five stopped profiles.
    if existing_controller_gate(CONTROLLER_GATE_NAME)? {
        return Err(io::Error::new(io::ErrorKind::WouldBlock,
            "An existing Minecraft controller uses another build or is still starting. Its games are preserved; no replacement was started."));
    }
    // No compatible controller means no per-slot attribution. Never adopt
    // another game or infer selected CloseOne success from an empty process
    // inventory. Only idle CloseAll preserves its original stopped response.
    if matches!(request,SessionRequest::CloseAll | SessionRequest::CloseOne {..}) {
        let observation=super::native_clients::inspect_native_workspace()?;
        return Ok(absent_controller_close_reply(&request,observation.processes.len()));
    }
    if matches!(request, SessionRequest::Inspect | SessionRequest::InspectSettings | SessionRequest::SavePerformance { .. } | SessionRequest::SaveAudio {..} | SessionRequest::SaveStoragePreferences {..}) {
        // A standalone settings save must exclude concurrent controller
        // startup across desktop and CLI. Retain the original startup gate.
        let _settings_gate = if matches!(request, SessionRequest::SavePerformance { .. } |
            SessionRequest::SaveAudio {..} | SessionRequest::SaveStoragePreferences {..}) {
            Some(acquire_existence_gate(CONTROLLER_GATE_NAME,
                io::Error::new(io::ErrorKind::WouldBlock,"Controller startup or settings save in progress"))?)
        } else { None };
        // The old fallback returned five STOPPED profiles on Inspect without
        // checking whether a game survived the controller. Such a false idle
        // receipt could also clear the caller's pending-request uncertainty.
        // Reuse the existing signed registered-game inventory before creating
        // a stopped snapshot or touching global settings.
        let observation = super::native_clients::inspect_native_workspace()?;
        require_absent_controller_idle(observation.processes.len())?;
        let mut snapshot = SessionSnapshot::stopped();
        if let SessionRequest::SaveAudio {mode}=request {super::manager_settings::save_audio(mode)?;}
        if let SessionRequest::SaveStoragePreferences {preferences}=request {super::manager_settings::save_storage_preferences(preferences)?;}
        if let SessionRequest::SavePerformance { active_fps, background_fps } = request {
            super::manager_settings::save_performance(super::manager_settings::PerformanceSettings {active_fps,background_fps})?;
        }
        if !matches!(request, SessionRequest::Inspect) {
            // Read-only Settings shares the already admitted empty inventory.
            // There is no intervening mutation. Saves still reobserve after
            // their write; this hint never grants mutation authority.
            let idle = matches!(request, SessionRequest::InspectSettings)
                || super::native_clients::inspect_native_workspace()?.processes.is_empty();
            snapshot.settings = Some(SettingsReport::policy(idle, None)?);
        }
        return Ok(Ok(snapshot));
    }
    // A target read cannot create a controller or adopt another game.
    if request.is_agent_target() {
        return Err(io::Error::new(io::ErrorKind::NotFound,
            "No retained controller; agent target observation refused"));
    }
    // A report or Testing correlation cannot cause fresh game/profile
    // construction if the original controller disappeared after the
    // one-shot CLI's existence gate. Fail closed on this TOCTOU race.
    if request.is_agent_report() {
        return Err(io::Error::new(io::ErrorKind::NotFound,
            "No retained controller; status or Testing reservation refused"));
    }
    // Input operations require an already retained authenticated controller.
    // Do not invent a new owner, replay an old channel or adopt a game
    // merely because an input command mentions its numerical slot.
    if request.is_experimental_input() {
        return Err(io::Error::new(io::ErrorKind::NotFound,
            "No retained controller; experimental input refused"));
    }
    // Explicit user action only. A fresh UI first tries the existing authenticated endpoint.
    let executable=super::process_roles::image("sessions")?;
    let mut child = Command::new(&executable.path)
        .arg("--lazy-tester-controller")
        .creation_flags(0x08000000)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let output = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("Missing controller startup output"))?;
    let mut reader = BufReader::new(output);
    let startup = read_message(&mut reader)?
        .ok_or_else(|| io::Error::other("Controller exited before startup"))?;
    let ready: Result<(), String> = serde_json::from_str(&startup).map_err(io::Error::other)?;
    if let Err(error) = ready {
        return Ok(Err(error));
    }
    // Dropping Child never kills the controller. It owns game lifetime independently of this UI.
    pipe::exchange(&endpoint, endpoint.connect()?, &request)
}

// Named-object existence lease, not thread-owned mutex locking.
#[cfg(windows)]
pub(crate) fn acquire_existence_gate(name: &str, already_exists: io::Error)
    -> io::Result<std::os::windows::io::OwnedHandle> {
    use std::os::windows::io::{FromRawHandle, OwnedHandle};
    use windows_sys::Win32::{Foundation::{GetLastError, ERROR_ALREADY_EXISTS},
        System::Threading::CreateMutexW};
    let name: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
    let raw = unsafe { CreateMutexW(std::ptr::null(), 0, name.as_ptr()) };
    if raw.is_null() { return Err(io::Error::last_os_error()); }
    let existed = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
    let gate = unsafe { OwnedHandle::from_raw_handle(raw as _) };
    if existed { return Err(already_exists); }
    Ok(gate)
}

// Shared per-slot exclusion, not target identity or launch authority.
#[cfg(windows)]
pub(crate) fn acquire_recording_close_gate(slot:u32)
    -> io::Result<std::os::windows::io::OwnedHandle> {
    if !(1..=5).contains(&slot) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,"Unknown Minecraft profile"));
    }
    acquire_existence_gate(&format!(r"Local\LazyTester-RecordingClose-{slot}"),
        io::Error::new(io::ErrorKind::WouldBlock,
            "Minecraft recording or close is in progress for this profile"))
}

#[cfg(windows)]
pub(crate) fn existing_controller_gate(name: &str) -> io::Result<bool> {
    use std::os::windows::io::{FromRawHandle, OwnedHandle};
    use windows_sys::Win32::{
        Foundation::GetLastError,
        System::Threading::{OpenMutexW, SYNCHRONIZATION_SYNCHRONIZE},
    };
    let name: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
    let handle = unsafe { OpenMutexW(SYNCHRONIZATION_SYNCHRONIZE, 0, name.as_ptr()) };
    if !handle.is_null() {
        let _gate = unsafe { OwnedHandle::from_raw_handle(handle as _) };
        return Ok(true);
    }
    let error = unsafe { GetLastError() };
    if error == 2 {
        Ok(false)
    } else {
        Err(io::Error::from_raw_os_error(error as i32))
    }
}

#[cfg(not(windows))]
pub fn request(_: SessionRequest) -> io::Result<SessionSnapshot> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Minecraft sessions require Windows",
    ))
}

pub(super) fn read_message(input: &mut impl io::BufRead) -> io::Result<Option<String>> {
    let mut bytes = Vec::new();
    let count = io::Read::take(input, (SESSION_FRAME_LIMIT + 1) as u64).read_until(b'\n', &mut bytes)?;
    if count == 0 {
        return Ok(None);
    }
    if count > SESSION_FRAME_LIMIT || bytes.last() != Some(&b'\n') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid session message",
        ));
    }
    String::from_utf8(bytes).map(Some).map_err(io::Error::other)
}

#[cfg(test)]
mod tests;
