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
#[cfg(windows)]
use self::controller_testing_runs::{valid_testing_digest, valid_testing_outcome};
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
pub(super) fn validate_request(request: &SessionRequest) -> io::Result<()> {
    if let SessionRequest::InspectAgentTarget {slot}=*request {
        if !(1..=5).contains(&slot) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,"Unknown Minecraft profile"));
        }
    }
    if let SessionRequest::ReviewSelectiveInventory { selections, .. } | SessionRequest::ConfirmSelectedWorlds { selections, .. } | SessionRequest::ConfirmSelectedPacks { selections, .. } = request {
        let mut identities = std::collections::HashSet::new();
        let mut names = std::collections::HashSet::new();
        if selections.is_empty() || selections.len() > 128 || selections.iter().any(|s| {
            let roots: &[&str] = match s.category {
                SelectiveCategory::Worlds => &["minecraftWorlds"],
                SelectiveCategory::ResourcePacks => &["resource_packs", "development_resource_packs"],
                SelectiveCategory::BehaviorPacks => &["behavior_packs", "development_behavior_packs"],
            };
            !roots.contains(&s.root.as_str()) || s.name.is_empty() || s.name.len() > 255 ||
                s.name == "." || s.name == ".." ||
                s.name.chars().any(|c| c.is_control() || matches!(c, '/' | '\\' | ':')) ||
                s.identity.len() != 64 ||
                !s.identity.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) ||
                !identities.insert(s.identity.as_str()) ||
                !names.insert((s.root.as_str(), s.name.to_ascii_lowercase()))
        }) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid selective review selection"));
        }
        // Bound the existing pipe contract before starting a transport worker.
        if serde_json::to_vec(request).map_err(io::Error::other)?.len() >= 32768 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Selection exceeds the session request limit"));
        }
    }
    if let SessionRequest::ConfirmSelectedPacks { slot, selections, confirmation_token } = request {
        if !(1..=5).contains(slot) || selections.iter().any(|s| s.category == SelectiveCategory::Worlds) || confirmation_token.len()!=64 || !confirmation_token.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid selected-pack confirmation"));
        }
    }
    if let SessionRequest::ConfirmSelectedWorlds { slot, selections, confirmation_token } = request {
        if !(1..=5).contains(slot) || selections.iter().any(|s| s.category != SelectiveCategory::Worlds) || confirmation_token.len()!=64 || !confirmation_token.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid selected-world confirmation"));
        }
    }
    match request {
        SessionRequest::TestingInspect {slot,owner_id} => {
            if !(1..=5).contains(slot)||!valid_testing_id(owner_id) {
                return Err(io::Error::new(io::ErrorKind::InvalidInput,
                    "Invalid read-only Testing inspection"));
            }
        }
        SessionRequest::TestingReserve {slot,owner_id,run_id,replace_run_id} => {
            if !(1..=5).contains(slot)||!valid_testing_id(owner_id)||
                !valid_testing_id(run_id)||
                replace_run_id.as_deref().is_some_and(|v|!valid_testing_id(v)){
                return Err(io::Error::new(io::ErrorKind::InvalidInput,
                    "Invalid Testing reservation"));
            }
        }
        SessionRequest::TestingFinalize {slot,owner_id,run_id,
            reported_status,report_digest} => {
            if !(1..=5).contains(slot)||!valid_testing_id(owner_id)||
                !valid_testing_id(run_id)||
                !valid_testing_outcome(reported_status)||
                !valid_testing_digest(report_digest) {
                return Err(io::Error::new(io::ErrorKind::InvalidInput,
                    "Invalid Testing final report reference"));
            }
        }
        _=>{}
    }
    if let SessionRequest::PublishAgentActivity {ref activity}=*request {
        if !activity.valid_ingress() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,
                "Invalid agent activity report"));
        }
    }
    if let SessionRequest::ClearProfileData {slot,ref confirmation_token,ref confirmation_name,..}=*request {
        if !(1..=5).contains(&slot) || confirmation_token.len()!=64 || !confirmation_token.bytes().all(|byte|byte.is_ascii_hexdigit()) || *confirmation_name!=format!("Minecraft - {slot}") {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,"Confirm the exact launcher name before clearing all data"));
        }
    }
    if let SessionRequest::SavePerformance {active_fps,background_fps}=*request {
        super::manager_settings::PerformanceSettings {active_fps,background_fps}.validate()?;
    }
    if matches!(*request, SessionRequest::ApplyGraphicsPreset { slot, .. } | SessionRequest::Launch { slot } | SessionRequest::CloseOne { slot } | SessionRequest::OpenProfileFolder { slot, .. } | SessionRequest::InspectProfileStorage { slot } | SessionRequest::ClearProfileCache { slot, .. } | SessionRequest::ClearProfileDownloadCache { slot } | SessionRequest::InspectSelectiveInventory { slot, .. } | SessionRequest::ReviewSelectiveInventory { slot, .. } | SessionRequest::ResetGameSettings { slot, .. } | SessionRequest::RestoreGameSettings { slot, .. } | SessionRequest::DisableImprovedInputResponse { slot } | SessionRequest::EnableTextureStreaming { slot } | SessionRequest::DisableTextureStreaming { slot } if !(1..=5).contains(&slot))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Unknown Minecraft profile",
        ));
    }
    #[cfg(feature="mc02-native-input")]
    if let SessionRequest::LaunchMc02Experiment {slot}=*request {
        if slot!=2 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,
                "Legacy experimental input launch only creates MC02"));
        }
    }
    #[cfg(feature="mc02-native-input")]
    if let SessionRequest::LaunchAiExperiment {slot,mode}=*request {
        if !(1..=5).contains(&slot) ||
           (mode==AgentControlMode::HumanPlusFourAi && slot==1) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,
                "Hybrid MC01 belongs to the human and has no AI channel"));
        }
    }
    #[cfg(feature="mc02-native-input")]
    if let SessionRequest::Mc02Input {
        slot,keys,dx,dy,ttl_ms,issued_at_ms,batch_origin,
    }=*request {
        // Native IPC must not trust CLI-provided control values or a
        // caller-supplied PID/birth claim. The actor separately verifies
        // this original-process witness before any legacy state change.
        admit_native_input_frame(slot,keys,dx,dy,ttl_ms)?;
        if issued_at_ms==0 || batch_origin.is_some_and(|pin|!pin.valid()) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,
                "Invalid native MC02 issue time or batch origin"));
        }
    }
    #[cfg(feature="mc02-native-input")]
    if let SessionRequest::AiInputGroup {
        ref frames,issued_at_ms,deadline_at_ms,
        ref owner_id,ref lease_epoch,
    } = *request {
        validate_ai_input_group(frames,issued_at_ms,deadline_at_ms)?;
        if !valid_ai_writer_id(owner_id)||!valid_ai_writer_id(lease_epoch) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,
                "Invalid AI writer or transaction fencing epoch"));
        }
    }
    #[cfg(feature="mc02-native-input")]
    if let SessionRequest::AiLeasedInput {
        slot,keys,dx,dy,ttl_ms,issued_at_ms,
        ref owner_id,ref lease_epoch,
    } = *request {
        admit_native_input_frame(slot,keys,dx,dy,ttl_ms)?;
        if issued_at_ms==0||!valid_ai_writer_id(owner_id)||
           !valid_ai_writer_id(lease_epoch) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,
                "Invalid leased AI input or transaction epoch"));
        }
    }
    #[cfg(feature="mc02-native-input")]
    if let SessionRequest::AiLeaseAcquire {
        ref slots,ref owner_id,mode,ref lease_epoch,neutral_only:_,
    }=*request {
        validate_ai_lease_slots(slots,owner_id)?;
        if !valid_ai_writer_id(lease_epoch) ||
           (mode==AgentControlMode::HumanPlusFourAi && slots.contains(&1)) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,
                "Invalid AI lease epoch or protected hybrid MC01"));
        }
    }
    #[cfg(feature="mc02-native-input")]
    if let SessionRequest::AiLeaseRelease {
        ref slots,ref owner_id,ref lease_epoch,
    }=*request {
        validate_ai_lease_slots(slots,owner_id)?;
        if !valid_ai_writer_id(lease_epoch) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,
                "Invalid AI release fencing epoch"));
        }
    }
    Ok(())
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
    let count = io::Read::take(input, 32769).read_until(b'\n', &mut bytes)?;
    if count == 0 {
        return Ok(None);
    }
    if count > 32768 || bytes.last() != Some(&b'\n') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid session message",
        ));
    }
    String::from_utf8(bytes).map(Some).map_err(io::Error::other)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn existence_lease_excludes_second_creator_until_original_handle_closes() {
        let name=format!(r"Local\LazyTester-GateFixture-{}",std::process::id());
        let acquire=||acquire_existence_gate(&name,io::Error::new(io::ErrorKind::WouldBlock,"held"));
        let first=acquire().unwrap();
        assert_eq!(acquire().unwrap_err().kind(),io::ErrorKind::WouldBlock);
        drop(first);
        assert!(acquire().is_ok());
    }

    #[cfg(windows)]
    #[test]
    fn pending_timeout_reconciliation_requires_fresh_session_inspect_not_settings() {
        // Pure admission policy fixture: no controller, pipe, Windows
        // game window or source test executor is started by this code.
        assert!(can_reconcile_pending_transport(&SessionRequest::Inspect));
        assert!(!can_reconcile_pending_transport(&SessionRequest::InspectSettings));
        assert!(!can_reconcile_pending_transport(&SessionRequest::InspectStorage));
        assert!(!can_reconcile_pending_transport(&SessionRequest::InspectProfileStorage {slot:1}));
        assert!(!can_reconcile_pending_transport(&SessionRequest::CloseOne{slot:2}));
        assert!(!can_reconcile_pending_transport(&SessionRequest::CloseAll));
        for read in [SessionRequest::InspectSettings, SessionRequest::InspectStorage,
            SessionRequest::InspectProfileStorage {slot:1}] {
            assert!(read_needs_reconciliation(&read,true));
            assert!(!read_needs_reconciliation(&read,false));
        }
        for other in [SessionRequest::Inspect,SessionRequest::InspectAgentTarget {slot:1},
            SessionRequest::CloseOne {slot:2},SessionRequest::CloseAll,
            SessionRequest::Launch {slot:3}] {
            assert!(!read_needs_reconciliation(&other,true));
        }
        #[cfg(feature="mc02-native-input")]
        {
            let owner="a".repeat(32);
            let epoch="b".repeat(30)+"e1";
            assert!(!can_reconcile_pending_transport(
                &SessionRequest::AiLeaseRelease {
                    slots:vec![2],owner_id:owner.clone(),lease_epoch:epoch.clone(),
                }));
            assert!(!can_reconcile_pending_transport(
                &SessionRequest::AiLeaseAcquire {
                    slots:vec![2],owner_id:owner,
                    lease_epoch:epoch,mode:AgentControlMode::HumanPlusFourAi,
                    neutral_only:true,
                }));
        }
    }

    #[cfg(windows)]
    #[test]
    fn pending_worker_outcomes_keep_one_shot_request_provenance() {
        use std::sync::mpsc::{self,RecvTimeoutError};
        use std::time::Duration;
        // Local channels only: no named pipe, controller, game or VM.
        let (tx,rx)=mpsc::sync_channel::<io::Result<SessionReply>>(1);
        assert!(matches!(
            classify_pending_worker_wait(rx.recv_timeout(Duration::ZERO)),
            PendingWorkerWait::StillRunning
        ));
        tx.send(Ok(Ok(SessionSnapshot::stopped()))).unwrap();
        assert!(matches!(
            classify_pending_worker_wait(rx.recv_timeout(Duration::ZERO)),
            PendingWorkerWait::Completed(Ok(Ok(_)))
        ));
        // The old channel's result never appears on another request's
        // distinct receiver, even if their invocation order overlaps.
        let (tx_old,rx_old)=mpsc::sync_channel::<io::Result<SessionReply>>(1);
        let (tx_new,rx_new)=mpsc::sync_channel::<io::Result<SessionReply>>(1);
        tx_old.send(Ok(Err("Original native refusal".into()))).unwrap();
        assert!(matches!(
            classify_pending_worker_wait(rx_new.recv_timeout(Duration::ZERO)),
            PendingWorkerWait::StillRunning
        ));
        assert!(matches!(
            classify_pending_worker_wait(rx_old.recv_timeout(Duration::ZERO)),
            PendingWorkerWait::Completed(Ok(Err(_)))
        ));
        // A transport error from the correct worker is NOT a successful
        // native reply, even if the worker did finish.
        tx_new.send(Err(io::Error::other("Original result unverified"))).unwrap();
        assert!(matches!(
            classify_pending_worker_wait(rx_new.recv_timeout(Duration::ZERO)),
            PendingWorkerWait::Completed(Err(_))
        ));
        let (tx_lost,rx_lost)=mpsc::sync_channel::<io::Result<SessionReply>>(1);
        drop(tx_lost);
        assert!(matches!(
            rx_lost.recv_timeout(Duration::ZERO),
            Err(RecvTimeoutError::Disconnected)
        ));
        // The dead worker's closed response channel is not a timeout.
        let (_sender,receiver)=mpsc::sync_channel::<io::Result<SessionReply>>(1);
        // Senders are dropped before checking to model a crashed worker.
        drop(_sender);
        assert!(matches!(
            classify_pending_worker_wait(receiver.recv_timeout(Duration::ZERO)),
            PendingWorkerWait::ResultChannelLost
        ));
    }

    #[cfg(windows)]
    #[test]
    fn previous_worker_drain_never_replays_or_clears_uncertainty() {
        let (sender, replies) = std::sync::mpsc::sync_channel(1);
        let worker = std::thread::spawn(|| {});
        let mut controller = Controller {
            pending: Some(PendingRequest { replies, worker }),
            uncertain: false,
        };
        assert_eq!(controller.drain_previous_request().unwrap_err().kind(),
            io::ErrorKind::WouldBlock);
        assert!(controller.pending.is_some());
        sender.send(Ok(Ok(SessionSnapshot::stopped()))).unwrap();
        controller.drain_previous_request().unwrap();
        assert!(controller.pending.is_none());
        assert!(controller.uncertain);
        controller.drain_previous_request().unwrap();
        assert!(controller.uncertain);
    }

    #[cfg(windows)]
    #[test]
    fn contended_transport_gate_refuses_a_second_request_without_waiting() {
        // Synthetic gate only: no Minecraft process, profile or controller IPC.
        let gate = Mutex::new(Controller { pending: None, uncertain: false });
        let retained = gate.lock().unwrap();
        let refusal = match admit_controller(&gate) {
            Ok(_) => panic!("Contended transport must never admit another request"),
            Err(error) => error,
        };
        assert_eq!(refusal.kind(), io::ErrorKind::WouldBlock);
        drop(retained);
        assert!(admit_controller(&gate).is_ok());
    }

    #[cfg(windows)]
    #[test]
    fn recovery_rejects_out_of_range_slots_before_controller_or_storage_access() {
        for slot in [0,6,u32::MAX] {
            for operation in [SessionRequest::ResetGameSettings {slot,selected_storage:None},SessionRequest::RestoreGameSettings {slot,selected_storage:None},SessionRequest::DisableImprovedInputResponse {slot},SessionRequest::EnableTextureStreaming {slot},SessionRequest::DisableTextureStreaming {slot},SessionRequest::CloseOne {slot}] {
                assert_eq!(request(operation).unwrap_err().kind(),io::ErrorKind::InvalidInput);
            }
        }
        assert!(serde_json::from_str::<SessionRequest>(r#"{"operation":"RestoreGameSettings","slot":1,"path":"outside"}"#).is_err());
        assert!(serde_json::from_str::<SessionRequest>(r#"{"operation":"CloseOne","slot":1,"pid":123}"#).is_err());
        assert!(matches!(serde_json::from_str::<SessionRequest>(r#"{"operation":"CloseOne","slot":3}"#),Ok(SessionRequest::CloseOne{slot:3})));
        for (slot,token,name) in [(0,"a".repeat(64),"Minecraft - 0"),(1,"bad".into(),"Minecraft - 1"),(1,"a".repeat(64),"Minecraft - 2")] {
            assert_eq!(request(SessionRequest::ClearProfileData {slot,confirmation_token:token,confirmation_name:name.into(),selected_storage:None}).unwrap_err().kind(),io::ErrorKind::InvalidInput);
        }
    }
    #[cfg(windows)]
    #[test]
    fn selected_storage_request_is_exact_and_bounded() {
        for slot in 1..=5 {
            let request=SessionRequest::InspectProfileStorage {slot};
            assert!(validate_request(&request).is_ok());
            assert!(matches!(serde_json::from_value::<SessionRequest>(serde_json::to_value(request).unwrap()),
                Ok(SessionRequest::InspectProfileStorage {slot: observed}) if observed==slot));
        }
        for slot in [0,6,u32::MAX] {
            assert_eq!(validate_request(&SessionRequest::InspectProfileStorage {slot}).unwrap_err().kind(),io::ErrorKind::InvalidInput);
        }
        assert!(serde_json::from_str::<SessionRequest>(r#"{"operation":"InspectProfileStorage"}"#).is_err());
        assert!(serde_json::from_str::<SessionRequest>(r#"{"operation":"InspectProfileStorage","slot":1,"path":"outside"}"#).is_err());
        assert!(matches!(serde_json::from_str::<SessionRequest>(r#"{"operation":"InspectStorage"}"#),Ok(SessionRequest::InspectStorage)));
    }
    #[cfg(windows)]
    #[test]
    fn selected_profile_receipt_scope_is_optional_and_strict() {
        for operation in ["OpenProfileFolder", "InspectSelectiveInventory", "ReviewSelectiveInventory"] {
            let mut legacy = serde_json::json!({"operation":operation,"slot":3});
            if operation == "InspectSelectiveInventory" { legacy["category"] = serde_json::json!("Worlds"); }
            if operation == "ReviewSelectiveInventory" {
                legacy["selections"] = serde_json::json!([{"category":"Worlds","root":"minecraftWorlds","name":"one","identity":"a".repeat(64)}]);
            }
            let request: SessionRequest = serde_json::from_value(legacy.clone()).unwrap();
            assert!(validate_request(&request).is_ok());
            assert_eq!(serde_json::to_value(request).unwrap(), legacy);
            for selected in [false, true] {
                let mut wire = legacy.clone();
                wire["selectedProfile"] = serde_json::json!(selected);
                let request: SessionRequest = serde_json::from_value(wire.clone()).unwrap();
                assert!(validate_request(&request).is_ok());
                assert_eq!(serde_json::to_value(request).unwrap(), wire);
            }
            let mut invalid = legacy.clone();
            invalid["selectedProfile"] = serde_json::json!("true");
            assert!(serde_json::from_value::<SessionRequest>(invalid).is_err());
            legacy["selectedProfile"] = serde_json::json!(true);
            legacy["slot"] = serde_json::json!(6);
            assert!(validate_request(&serde_json::from_value(legacy).unwrap()).is_err());
        }
        assert!(serde_json::from_value::<SessionRequest>(serde_json::json!({"operation":"Inspect","selectedProfile":true})).is_err());
        assert!(serde_json::from_value::<SessionRequest>(serde_json::json!({"operation":"Launch","slot":3,"selectedProfile":true})).is_err());
    }

    #[cfg(windows)]
    #[test]
    fn selected_mutation_storage_scope_is_optional_and_strict() {
        for operation in ["ClearProfileCache","ResetGameSettings","RestoreGameSettings","ClearProfileData"] {
            let mut legacy=serde_json::json!({"operation":operation,"slot":3});
            if operation=="ClearProfileData" {
                legacy["confirmationToken"]=serde_json::json!("a".repeat(64));
                legacy["confirmationName"]=serde_json::json!("Minecraft - 3");
            }
            let request:SessionRequest=serde_json::from_value(legacy.clone()).unwrap();
            assert!(validate_request(&request).is_ok());
            assert_eq!(serde_json::to_value(request).unwrap(),legacy);
            let mut selected=legacy.clone();
            selected["selectedStorage"]=serde_json::json!(true);
            let request:SessionRequest=serde_json::from_value(selected.clone()).unwrap();
            assert!(validate_request(&request).is_ok());
            assert_eq!(serde_json::to_value(request).unwrap(),selected);
            selected["selectedStorage"]=serde_json::json!("true");
            assert!(serde_json::from_value::<SessionRequest>(selected).is_err());
            legacy["slot"]=serde_json::json!(6);
            legacy["selectedStorage"]=serde_json::json!(true);
            let invalid:SessionRequest=serde_json::from_value(legacy).unwrap();
            assert_eq!(validate_request(&invalid).unwrap_err().kind(),io::ErrorKind::InvalidInput);
        }
        assert!(serde_json::from_value::<SessionRequest>(serde_json::json!({"operation":"InspectProfileStorage","slot":3,"selectedStorage":true})).is_err());
    }
    #[test]
    fn settings_report_describes_the_reviewed_policy_without_starting_a_controller() {
        let policy = SettingsReport::policy(false, None).unwrap();
        let saved = super::super::manager_settings::load_performance().unwrap();
        assert_eq!((policy.active_fps, policy.background_fps, policy.render_distance_chunks), (saved.active_fps, saved.background_fps, 12));
        let pacing = include_str!("../../../../../tooling/acceptance/windows/bootstrap-ordering/frame-pacing.h");
        let initial = super::super::manager_settings::PerformanceSettings::default();
        assert!(pacing.contains(&format!(
            "activeFps = {}, backgroundFps = {}", initial.active_fps, initial.background_fps
        )), "The first-run manager FPS limits must match the reviewed native pacer");
        let snapshot = SessionSnapshot::stopped();
        assert!(!serde_json::to_value(&snapshot).unwrap().as_object().unwrap().contains_key("settings"));
        assert!(serde_json::from_str::<SessionRequest>(r#"{"operation":"ApplyPreset"}"#).is_ok());
        assert!(serde_json::from_str::<SessionRequest>(r#"{"operation":"InspectSettings"}"#).is_ok());
        #[cfg(windows)]
        assert_eq!(request(SessionRequest::SavePerformance {active_fps:0,background_fps:10}).unwrap_err().kind(),io::ErrorKind::InvalidInput);
    }
    #[cfg(windows)]
    #[test]
    fn existing_incompatible_controller_gate_is_distinguished_from_absence() {
        use std::os::windows::io::{FromRawHandle, OwnedHandle};
        let name = format!(
            "Local\\LazyTester-disposable-gate-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        assert!(!existing_controller_gate(&name).unwrap());
        let wide: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
        let raw = unsafe {
            windows_sys::Win32::System::Threading::CreateMutexW(std::ptr::null(), 0, wide.as_ptr())
        };
        assert!(!raw.is_null());
        let handle = unsafe { OwnedHandle::from_raw_handle(raw as _) };
        assert!(existing_controller_gate(&name).unwrap());
        drop(handle);
        assert!(!existing_controller_gate(&name).unwrap());
    }
    #[cfg(windows)]
    #[test]
    fn passive_idle_hint_never_substitutes_for_pending_or_uncertain_inspect() {
        assert!(idle_hint_needs_read(true, || panic!("unsettled bypasses hint"), || panic!("no candidate scan")).unwrap());
        assert!(idle_hint_needs_read(false, || Ok(true), || panic!("existing controller requires full read")).unwrap());
        assert!(idle_hint_needs_read(false, || Ok(false), || Ok(true)).unwrap());
        let mut checks=0;
        assert!(idle_hint_needs_read(false, || {checks+=1;Ok(checks==2)}, || Ok(false)).unwrap());
        assert_eq!(checks,2);
        assert!(!idle_hint_needs_read(false, || Ok(false), || Ok(false)).unwrap());
        assert!(idle_hint_needs_read(false, || Ok(false), || Err(io::Error::other("snapshot failed"))).is_err());
        assert!(idle_hint_needs_read(false, || Err(io::Error::other("gate unavailable")), || Ok(false)).is_err());
    }
    #[test]
    fn missing_controller_inspect_and_settings_require_no_observed_games() {
        // Pure admission fixture: no Windows inventory, controller, game or
        // settings mutation. Each fallback operation shares this same gate.
        assert!(require_absent_controller_idle(0).is_ok());
        for observed in [1, 2, 5, 16] {
            let error = require_absent_controller_idle(observed).unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::WouldBlock);
            assert!(error.to_string().contains("per-slot state is unknown"));
        }
    }

    #[test]
    fn missing_controller_refuses_false_close_when_native_game_candidates_exist() {
        // Full manager shutdown may confirm an already-idle host, but a
        // specific slot needs its retained controller even when none is seen.
        assert!(absent_controller_close_reply(&SessionRequest::CloseAll, 0).is_ok());
        for slot in 1..=5 {
            let selected = SessionRequest::CloseOne {slot};
            assert!(absent_controller_close_reply(&selected, 0).unwrap_err().contains("No retained Minecraft controller"));
            assert!(absent_controller_close_reply(&selected, 1).unwrap_err().contains("no compatible Lazy Tester"));
        }
        assert!(absent_controller_close_reply(&SessionRequest::CloseAll, 1).is_err());
        assert!(absent_controller_close_reply(&SessionRequest::CloseAll, 5).is_err());
        // The established serialized operation remains unchanged.
        assert!(matches!(serde_json::from_str::<SessionRequest>(
            r#"{"operation":"CloseOne","slot":2}"#
        ), Ok(SessionRequest::CloseOne {slot:2})));
    }

    #[cfg(feature="mc02-native-input")]
    #[test]
    fn legacy_mc02_optional_batch_origin_preserves_one_shot_wire_and_rejects_bad_pin() {
        // Pure serde fixture. No controller, native IPC, Windows or game.
        let old=serde_json::json!({"operation":"Mc02Input",
            "slot":2,"keys":0,"dx":0,"dy":0,"ttl_ms":1,"issued_at_ms":100});
        let decoded:SessionRequest=serde_json::from_value(old.clone()).unwrap();
        assert!(matches!(decoded,SessionRequest::Mc02Input{
            batch_origin:None,..}));
        let one_shot=SessionRequest::Mc02Input{slot:2,keys:0,dx:0,dy:0,
            ttl_ms:1,issued_at_ms:100,batch_origin:None};
        assert_eq!(serde_json::to_value(one_shot).unwrap(),old,
            "older legacy one-shot IPC shape must remain unchanged");
        let pin=Mc02BatchOrigin{pid:202,birth:134_000_000_000_000_000,
            session:1};
        let pinned=SessionRequest::Mc02Input{slot:2,keys:1,dx:0,dy:0,
            ttl_ms:350,issued_at_ms:101,batch_origin:Some(pin)};
        let wire=serde_json::to_value(pinned).unwrap();
        assert!(matches!(serde_json::from_value::<SessionRequest>(
            wire.clone()).unwrap(),SessionRequest::Mc02Input{
            batch_origin:Some(found),..} if found==pin));
        for wrong in [
            serde_json::json!({"pid":0,"birth":pin.birth,"session":1}),
            serde_json::json!({"pid":pin.pid,"birth":0,"session":1}),
            serde_json::json!({"pid":pin.pid,"birth":pin.birth,"session":0}),
        ] {
            let invalid=Mc02BatchOrigin{
                pid:wrong["pid"].as_u64().unwrap() as u32,
                birth:wrong["birth"].as_u64().unwrap(),
                session:wrong["session"].as_u64().unwrap() as u32,
            };
            assert!(!invalid.valid());
        }
        let mut extra=wire;
        extra["batch_origin"]["unreviewed_owner"]=serde_json::json!(true);
        assert!(serde_json::from_value::<SessionRequest>(extra).is_err(),
            "private expected-process wire must deny unknown fields");
    }

    #[cfg(feature="mc02-native-input")]
    #[test]
    fn private_acquire_wire_requires_mode_epoch_and_recovery_intent() {
        // Reject old unfenced serialized requests BEFORE controller mutation.
        let valid=serde_json::json!({
            "operation":"AiLeaseAcquire",
            "slots":[2,3,4,5],
            "owner_id":"a".repeat(32),
            "mode":"human-plus-four-ai",
            "lease_epoch":"1".repeat(32),
            "neutral_only":false
        });
        assert!(matches!(serde_json::from_value::<SessionRequest>(valid.clone()),
            Ok(SessionRequest::AiLeaseAcquire {
                mode:AgentControlMode::HumanPlusFourAi,
                neutral_only:false,..})));
        for required in ["mode","owner_id","lease_epoch","neutral_only"] {
            let mut old=valid.clone();
            old.as_object_mut().unwrap().remove(required);
            assert!(serde_json::from_value::<SessionRequest>(old).is_err());
        }
        let mut bad=valid.clone();
        bad["mode"]=serde_json::json!("unrecognized");
        assert!(serde_json::from_value::<SessionRequest>(bad).is_err());
        let mut five=valid.clone();
        five["mode"]=serde_json::json!("five-ai");
        five["slots"]=serde_json::json!([1,2,3,4,5]);
        assert!(matches!(serde_json::from_value::<SessionRequest>(five),
            Ok(SessionRequest::AiLeaseAcquire {
                mode:AgentControlMode::FiveAi,..})));
    }

    #[cfg(feature="mc02-native-input")]
    #[test]
    fn private_input_receipt_is_not_a_session_roster_or_status_report() {
        let receipt=SessionSnapshot::empty_receipt();
        assert!(receipt.is_input_receipt());
        assert_eq!(serde_json::to_value(&receipt).unwrap(),
            serde_json::json!({"schema":1,"profiles":[],"storage":null}));
        let roundtrip:SessionSnapshot=serde_json::from_value(
            serde_json::to_value(&receipt).unwrap()).unwrap();
        assert!(roundtrip.is_input_receipt());
        assert!(!SessionSnapshot::stopped().is_input_receipt());
        let mut wrong=receipt;
        wrong.schema=2;
        assert!(!wrong.is_input_receipt());
        assert!(!SessionRequest::Inspect.expects_input_receipt());
        assert!(!SessionRequest::LaunchMc02Experiment{slot:2}.expects_input_receipt());
        assert!(!SessionRequest::LaunchAiExperiment{slot:2,
            mode:AgentControlMode::HumanPlusFourAi}.expects_input_receipt());
        assert!(SessionRequest::AiLeasedInput{slot:2,keys:0,dx:0,dy:0,
            ttl_ms:1,issued_at_ms:1,owner_id:"a".repeat(32),
            lease_epoch:"b".repeat(32)}.expects_input_receipt());
    }

    #[cfg(feature="mc02-native-input")]
    #[test]
    fn grouped_ai_input_admission_rejects_ambiguous_and_expired_deadlines() {
        // Pure source-authored fixture; no IPC, game windows or input sent.
        let frame=|slot|AiInputFrame{slot,keys:1,dx:0,dy:0,ttl_ms:350};
        for slots in [vec![2,3,4,5],vec![1,2,3,4,5]] {
            let valid=slots.into_iter().map(frame).collect::<Vec<_>>();
            assert!(validate_ai_input_group(&valid,1000,1150).is_ok());
            assert!(validate_ai_input_group(&valid,1000,1000).is_ok());
            assert!(validate_ai_input_group(&valid,1000,1151).is_err());
            assert!(validate_ai_input_group(&valid,1100,1099).is_err());
            assert!(validate_ai_input_group(&valid,0,100).is_err());
        }
        for slots in [vec![],vec![2,2],vec![3,2],vec![1,2,3,4,5,6],
            vec![0],vec![6]] {
            let invalid=slots.into_iter().map(frame).collect::<Vec<_>>();
            assert!(validate_ai_input_group(&invalid,1000,1100).is_err());
        }
        let mut invalid=frame(2);
        invalid.keys=3; // conflicting W/S must not enter the actor
        assert!(validate_ai_input_group(&[invalid],1000,1100).is_err());
        invalid=frame(2);
        invalid.ttl_ms=1;
        assert!(validate_ai_input_group(&[invalid],1000,1100).is_err());
        let json=r#"{"operation":"AiInputGroup","frames":[{"slot":2,
            "keys":1,"dx":0,"dy":0,"ttl_ms":350}],
            "issued_at_ms":1000,"deadline_at_ms":1100,
            "owner_id":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "lease_epoch":"11111111111111111111111111111111"}"#;
        assert!(matches!(serde_json::from_str::<SessionRequest>(json),
            Ok(SessionRequest::AiInputGroup{..})));
        let original:serde_json::Value=serde_json::from_str(json).unwrap();
        for required in ["owner_id","lease_epoch"] {
            let mut old=original.clone();
            old.as_object_mut().unwrap().remove(required);
            // Remove the actual JSON object field, not a textual
            // substring that might not exist due to whitespace.
            assert!(serde_json::from_value::<SessionRequest>(old).is_err());
        }
        assert!(serde_json::from_str::<SessionRequest>(
            &json.replace("\"ttl_ms\":350", "\"ttl_ms\":350,\"pid\":123")
        ).is_err());
    }
    #[cfg(feature="mc02-native-input")]
    #[test]
    fn per_client_input_admission_validates_five_slots_and_opposing_controls() {
        // Pure source fixture: does not create a retained controller, process
        // or input section and is NOT Minecraft gameplay evidence.
        for slot in [0,6,u32::MAX] {
            assert_eq!(admit_native_input_frame(slot,1,0,0,150)
                .unwrap_err().kind(),io::ErrorKind::InvalidInput);
            assert_eq!(admit_native_input_frame(slot,0,0,0,1)
                .unwrap_err().kind(),io::ErrorKind::InvalidInput);
        }
        for slot in 1..=5 {
            assert_eq!(admit_native_input_frame(slot,1,0,0,150).unwrap(),
                NativeInputIntent::Active);
            assert_eq!(admit_native_input_frame(slot,0,0,0,1).unwrap(),
                NativeInputIntent::ReleaseAll);
        }
        for keys in [3,12,15,255,256] {
            assert_eq!(admit_native_input_frame(2,keys,0,0,150)
                .unwrap_err().kind(),io::ErrorKind::InvalidInput);
        }
        for (keys,dx,dy,ttl) in [(1,128,0,150),(1,0,-128,150),
                                 (1,0,0,0),(1,0,0,501)] {
            assert!(admit_native_input_frame(2,keys,dx,dy,ttl).is_err());
        }
        assert_eq!(admit_native_input_frame(2,0,0,0,1).unwrap(),
            NativeInputIntent::ReleaseAll);
        assert_eq!(admit_native_input_frame(2,0,10,-10,150).unwrap(),
            NativeInputIntent::Active);
        assert_eq!(admit_native_input_frame(2,1|4|16|64,0,0,150).unwrap(),
            NativeInputIntent::Active);
    }
    #[cfg(windows)]
    #[test]
    fn agent_target_admission_and_receipt_are_exact_slot_only() {
        for slot in [0,6,u32::MAX] {
            assert!(validate_request(&SessionRequest::InspectAgentTarget{slot}).is_err());
        }
        for slot in 1..=5 {
            let request=SessionRequest::InspectAgentTarget{slot};
            assert!(request.is_agent_target());
            assert!(validate_request(&request).is_ok());
            assert!(!can_reconcile_pending_transport(&request));
            let mut snapshot=SessionSnapshot::stopped();
            assert!(!snapshot.is_agent_target_receipt(slot));
            snapshot.profiles.retain(|p|p.slot==slot);
            assert!(snapshot.is_agent_target_receipt(slot));
            assert!(!snapshot.is_agent_target_receipt(if slot==1{2}else{1}));
        }
        assert!(!SessionRequest::Inspect.is_agent_target());
        assert!(serde_json::from_str::<SessionRequest>(r#"{"operation":"InspectAgentTarget","slot":2,"pid":1}"#).is_err());
    }

    #[cfg(windows)]
    #[test]
    fn full_inspect_reconciliation_refuses_partial_duplicate_and_unrelated_receipts() {
        let full=SessionSnapshot::stopped();
        assert!(full.is_full_inspect_receipt());
        let mut partial=full.clone();
        partial.profiles.retain(|p|p.slot==2);
        assert!(partial.is_agent_target_receipt(2));
        assert!(!partial.is_full_inspect_receipt());
        let mut duplicate=full.clone();
        duplicate.profiles[4].slot=4;
        assert!(!duplicate.is_full_inspect_receipt());
        let mut reordered=full.clone();
        reordered.profiles.swap(0,1);
        assert!(!reordered.is_full_inspect_receipt());
        let mut wrong=full.clone();
        wrong.schema=2;
        assert!(!wrong.is_full_inspect_receipt());
        let mut storage=full;
        storage.storage=Some(Vec::new());
        assert!(!storage.is_full_inspect_receipt());
    }

    #[cfg(windows)]
    #[test]
    fn desktop_target_uses_exact_slot_unless_original_transport_needs_reconciliation() {
        for slot in 1..=5 {
            for uncertain in [false,true] {
                let selected=desktop_observation_request(SessionRequest::InspectAgentTarget{slot},true,uncertain);
                if uncertain {
                    assert!(matches!(selected,SessionRequest::Inspect));
                } else {
                    assert!(matches!(selected,SessionRequest::InspectAgentTarget{slot:observed} if observed==slot));
                }
                // The agent's own selected observation never reconciles.
                let agent=desktop_observation_request(SessionRequest::InspectAgentTarget{slot},false,uncertain);
                assert!(matches!(agent,SessionRequest::InspectAgentTarget{slot:observed} if observed==slot));
            }
        }
    }

    #[test]
    fn agent_request_classes_keep_private_routes_out_of_desktop_and_spawn() {
        for request in [
            SessionRequest::TestingInspect { slot: 1, owner_id: String::new() },
            SessionRequest::TestingReserve { slot: 1, owner_id: String::new(),
                run_id: String::new(), replace_run_id: None },
            SessionRequest::TestingFinalize { slot: 1, owner_id: String::new(),
                run_id: String::new(), reported_status: String::new(), report_digest: String::new() },
        ] {
            assert!(request.is_agent_report());
            assert!(!request.is_experimental_input());
        }
        for request in [SessionRequest::Inspect, SessionRequest::Launch { slot: 1 }] {
            assert!(!request.is_agent_report());
            assert!(!request.is_experimental_input());
        }
        #[cfg(feature="mc02-native-input")]
        for request in [
            SessionRequest::LaunchMc02Experiment { slot: 2 },
            SessionRequest::LaunchAiExperiment { slot: 2, mode: AgentControlMode::HumanPlusFourAi },
            SessionRequest::Mc02Input { slot: 2, keys: 0, dx: 0, dy: 0,
                ttl_ms: 350, issued_at_ms: 1, batch_origin: None },
            SessionRequest::AiLeasedInput { slot: 2, keys: 0, dx: 0, dy: 0,
                ttl_ms: 350, issued_at_ms: 1, owner_id: String::new(), lease_epoch: String::new() },
            SessionRequest::AiLeaseAcquire { slots: vec![2], owner_id: String::new(),
                mode: AgentControlMode::HumanPlusFourAi, lease_epoch: String::new(), neutral_only: true },
            SessionRequest::AiLeaseRelease { slots: vec![2], owner_id: String::new(), lease_epoch: String::new() },
            SessionRequest::AiInputGroup { frames: vec![], issued_at_ms: 1, deadline_at_ms: 2,
                owner_id: String::new(), lease_epoch: String::new() },
        ] {
            assert!(request.is_experimental_input());
            assert!(!request.is_agent_report());
        }
    }

    #[test]
    fn protocol_rejects_paths_unknown_operations_and_unbounded_input() {
        for input in [
            r#"{"operation":"Launch","slot":1,"path":"other.exe"}"#,
            r#"{"operation":"Delete","slot":1}"#,
            r#"{"operation":"SavePerformance","activeFps":60,"backgroundFps":5,"path":"other.json"}"#,
            r#"{"operation":"SavePerformance","activeFps":60.5,"backgroundFps":5}"#,
            r#"{"operation":"DisableImprovedInputResponse","slot":1,"enabled":false}"#,
            r#"{"operation":"EnableTextureStreaming","slot":1,"value":1}"#,
        ] {
            assert!(serde_json::from_str::<SessionRequest>(input).is_err());
        }
        assert!(matches!(serde_json::from_str::<SessionRequest>(r#"{"operation":"DisableImprovedInputResponse","slot":2}"#), Ok(SessionRequest::DisableImprovedInputResponse {slot:2})));
        assert!(matches!(serde_json::from_str::<SessionRequest>(r#"{"operation":"EnableTextureStreaming","slot":4}"#), Ok(SessionRequest::EnableTextureStreaming {slot:4})));
        assert!(matches!(serde_json::from_str::<SessionRequest>(r#"{"operation":"DisableTextureStreaming","slot":4}"#), Ok(SessionRequest::DisableTextureStreaming {slot:4})));
        assert!(read_message(&mut io::Cursor::new(vec![b'x'; 32769])).is_err());
        assert!(read_message(&mut io::Cursor::new(b"{}".to_vec())).is_err());
        assert_eq!(
            SessionSnapshot::stopped()
                .profiles
                .iter()
                .map(|p| p.slot)
                .collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5]
        );
    }
}
