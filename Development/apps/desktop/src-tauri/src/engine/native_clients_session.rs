//! Lazy Tester transport. The existing bootstrap owns games in one retained helper.
use super::process_identity::OwnedProcess;
use serde::{Deserialize, Serialize};
use std::{
    io::{self, BufRead},
    sync::Mutex,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProfileSession {
    pub slot: u32,
    pub state: String,
    pub process: Option<OwnedProcess>,
    pub issue: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SessionSnapshot {
    pub schema: u32,
    pub profiles: Vec<ProfileSession>,
    pub storage: Option<Vec<ProfileStorage>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settings: Option<SettingsReport>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SettingsReport {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_audio: Option<super::manager_settings::BackgroundAudio>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_issue: Option<String>,
    #[serde(default = "default_graphics")]
    pub graphics_presets: [super::manager_settings::GraphicsPreset; 5],
    pub active_fps: u32,
    pub background_fps: u32,
    pub render_distance_chunks: u32,
    pub can_apply_preset: bool,
    pub changed_profiles: Option<u32>,
}
fn default_graphics() -> [super::manager_settings::GraphicsPreset; 5] { [super::manager_settings::GraphicsPreset::Low; 5] }
impl SettingsReport {
    pub fn policy(can_apply_preset: bool, changed_profiles: Option<u32>) -> io::Result<Self> {
        // Configured ceilings of the reviewed payload, never measured frame rates.
        let limits=super::manager_settings::load_performance()?;
        Ok(Self { background_audio:Some(super::manager_settings::load_audio()?), audio_issue:None, graphics_presets: super::manager_settings::load_graphics()?, active_fps: limits.active_fps, background_fps: limits.background_fps, render_distance_chunks: 12,
            can_apply_preset, changed_profiles })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProfileStorage {
    pub slot: u32,
    pub bytes: u64,
    pub files: u32,
    pub partial: bool,
    pub disk: Option<DiskSpace>,
    pub cache: CacheStorage,
    // Read-only on-disk pack directory census; NOT world activation or resident RAM.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_packs: Option<ResourcePackStorage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_settings: Option<GameSettingsStorage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_clear: Option<FullClearStorage>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResourcePackStorage {
    // Manifest-containing local directories; no assumption of activation.
    pub manifested: u32,
    pub partial: bool,
    pub issue: Option<String>,
    // Optional for backward-compatible older session replies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_declarations: Option<WorldPackDeclarations>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorldPackDeclarations {
    // Counts only world_resource_packs.json metadata, not loaded resources.
    pub worlds_scanned: u32,
    pub worlds_with_references: u32,
    pub pack_references: u32,
    pub partial: bool,
    pub issue: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FullClearStorage {
    pub can_clear: bool,
    pub pending: bool,
    pub confirmation_token: Option<String>,
    pub issue: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GameSettingsStorage {
    pub has_settings: bool,
    pub backup_available: bool,
    pub can_reset: bool,
    pub can_restore: bool,
    // Optional read-only profile option, no implied native version/behavior.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub improved_input_response_enabled: Option<bool>,
    // Only an explicit existing key is observed; current Education support is not inferred.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub texture_streaming_enabled: Option<bool>,
    pub issue: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DiskSpace {
    pub available_bytes: u64,
    pub total_bytes: u64,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CacheStorage {
    pub bytes: u64,
    pub files: u32,
    pub partial: bool,
    pub can_clear: bool,
    pub issue: Option<String>,
}
impl SessionSnapshot {
    pub fn stopped() -> Self {
        Self {
            schema: 1,
            storage: None,
            settings: None,
            profiles: (1..=5)
                .map(|slot| ProfileSession {
                    slot,
                    state: "STOPPED".into(),
                    process: None,
                    issue: None,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "operation", deny_unknown_fields)]
pub enum SessionRequest {
    Inspect,
    InspectSettings,
    ApplyPreset,
    ApplyGraphicsPreset { slot: u32, preset: super::manager_settings::GraphicsPreset },
    SavePerformance { #[serde(rename="activeFps")] active_fps:u32, #[serde(rename="backgroundFps")] background_fps:u32 },
    SaveAudio {mode:super::manager_settings::BackgroundAudio},
    InspectStorage,
    OpenProfileFolder { slot: u32 },
    ClearProfileCache { slot: u32 },
    ResetGameSettings { slot: u32 },
    RestoreGameSettings { slot: u32 },
    DisableImprovedInputResponse { slot: u32 },
    EnableTextureStreaming { slot: u32 },
    DisableTextureStreaming { slot: u32 },
    ClearProfileData {slot:u32, #[serde(rename="confirmationToken")] confirmation_token:String, #[serde(rename="confirmationName")] confirmation_name:String},
    Launch { slot: u32 },
    #[cfg(feature="mc02-native-input")]
    LaunchMc02Experiment { slot: u32 },
    #[cfg(feature="mc02-native-input")]
    LaunchAiExperiment {slot:u32,mode:AgentControlMode},
    #[cfg(feature="mc02-native-input")]
    Mc02Input {slot:u32,keys:u32,dx:i32,dy:i32,ttl_ms:u32,issued_at_ms:u64},
    #[cfg(feature="mc02-native-input")]
    AiLeasedInput {
        slot:u32,keys:u32,dx:i32,dy:i32,ttl_ms:u32,issued_at_ms:u64,
        owner_id:String,
    },
    // A single retained controller atomically admits all original game
    // slots for one ephemeral MCP writer before any movement packet.
    #[cfg(feature="mc02-native-input")]
    AiLeaseAcquire {slots:Vec<u32>,owner_id:String,mode:AgentControlMode},
    #[cfg(feature="mc02-native-input")]
    AiLeaseRelease {slots:Vec<u32>,owner_id:String},
    // One authenticated request for all per-client states due at the SAME
    // timestamp. This is a bounded group, not a promise of atomic Win32
    // delivery or simultaneous Minecraft polling.
    #[cfg(feature="mc02-native-input")]
    AiInputGroup {
        frames:Vec<AiInputFrame>,issued_at_ms:u64,deadline_at_ms:u64,
        owner_id:String,
    },
    CloseOne { slot: u32 },
    CloseAll,
}

#[cfg(feature="mc02-native-input")]
#[derive(Clone,Copy,Debug,Deserialize,Serialize)]
#[serde(deny_unknown_fields)]
pub struct AiInputFrame {
    pub slot:u32,pub keys:u32,pub dx:i32,pub dy:i32,pub ttl_ms:u32,
}

// Validate the COMPLETE private group before any controller-side mutation.
// Require strictly ascending, distinct native slots and fixed experimental
// pulse TTL. This cannot authorize a child that lacks its opted-in channel.
#[cfg(feature="mc02-native-input")]
pub(crate) fn validate_ai_input_group(
    frames:&[AiInputFrame],issued_at_ms:u64,deadline_at_ms:u64,
)->io::Result<()> {
    if frames.is_empty() || frames.len()>5 || issued_at_ms==0 ||
       deadline_at_ms<issued_at_ms ||
       deadline_at_ms-issued_at_ms>150 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,
            "Invalid experimental AI timestamp group"));
    }
    let mut previous_slot=0;
    for frame in frames {
        if frame.slot<=previous_slot || frame.ttl_ms!=350 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,
                "AI group must have distinct ascending slots and fixed TTL"));
        }
        admit_native_input_frame(frame.slot,frame.keys,frame.dx,frame.dy,
            frame.ttl_ms)?;
        previous_slot=frame.slot;
    }
    Ok(())
}

// A bounded, controller-owned writer lease for the feature-gated AI input
// experiment. The 32-hex owner ID is an ephemeral correlation ID supplied
// only via the local MCP child environment: NOT an OS authentication
// credential. Same-user malicious processes can imitate it; this prevents
// accidental independent MCP writers, not hostile same-user tampering.
// Native PID/birth/mode checks and per-process game input gates still apply.
#[cfg(feature="mc02-native-input")]
pub(crate) const AI_WRITER_LEASE_MS:u64=10_000;
#[cfg(feature="mc02-native-input")]
pub(crate) fn valid_ai_writer_id(value:&str)->bool {
    value.len()==32 && value.bytes().all(|b|b.is_ascii_hexdigit()&&
        !b.is_ascii_uppercase())
}
#[cfg(feature="mc02-native-input")]
pub(crate) fn validate_ai_lease_slots(slots:&[u32],owner_id:&str)->io::Result<()> {
    if !valid_ai_writer_id(owner_id)||slots.is_empty()||slots.len()>5||
       slots.windows(2).any(|p|p[0]>=p[1])||
       slots.iter().any(|slot|!(1..=5).contains(slot)) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,
            "Invalid experimental AI writer lease"));
    }
    Ok(())
}
#[cfg(feature="mc02-native-input")]
#[derive(Clone,Debug)]
struct AiWriterClaim {
    owner_id:String,pid:u32,birth:u64,until_ms:u64,neutral:bool,
}
#[cfg(feature="mc02-native-input")]
#[derive(Debug)]
pub(crate) struct AiLeaseBook {
    claims:[Option<AiWriterClaim>;5],
    legacy_until_ms:[u64;5],
}
#[cfg(feature="mc02-native-input")]
impl Default for AiLeaseBook {
    fn default()->Self {
        Self{claims:std::array::from_fn(|_|None),legacy_until_ms:[0;5]}
    }
}
#[cfg(feature="mc02-native-input")]
impl AiLeaseBook {
    // Original process identities are verified by the SAME Sessions actor
    // before this pure book is called. No partial acquisition on conflict.
    pub(crate) fn acquire(&mut self,targets:&[(u32,u32,u64)],
        owner:&str,now:u64)->io::Result<()> {
        let slots=targets.iter().map(|(slot,_,_)|*slot).collect::<Vec<_>>();
        validate_ai_lease_slots(&slots,owner)?;
        if now==0 || targets.iter().any(|(slot,pid,birth)| {
            *pid==0||*birth==0 ||
            self.legacy_until_ms[(*slot-1) as usize]>now ||
            self.claims[(*slot-1) as usize].as_ref().is_some_and(|held|
                held.until_ms>now && (held.owner_id!=owner ||
                    held.pid!=*pid||held.birth!=*birth))
        }) {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied,
                "AI writer lease conflict; existing game input preserved"));
        }
        for &(slot,pid,birth) in targets {
            let index=(slot-1) as usize;
            let neutral=self.claims[index].as_ref().is_none_or(|held|
                held.until_ms<=now || held.owner_id!=owner ||
                held.pid!=pid||held.birth!=birth || held.neutral);
            self.claims[index]=Some(AiWriterClaim {
                owner_id:owner.into(),pid,birth,
                until_ms:now.saturating_add(AI_WRITER_LEASE_MS),neutral,
            });
        }
        Ok(())
    }
    pub(crate) fn require(&self,slot:u32,owner:&str,
        pid:u32,birth:u64,now:u64)->io::Result<()> {
        if !valid_ai_writer_id(owner)||!(1..=5).contains(&slot)||now==0||
           !self.claims[(slot-1) as usize].as_ref().is_some_and(|held|
             held.owner_id==owner && held.pid==pid &&
             held.birth==birth && held.until_ms>now) {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied,
                "AI slot is not owned by this live writer lease"));
        }
        Ok(())
    }
    // Declare a write potentially ACTIVE BEFORE calling the exact
    // original process channel. A native I/O error can occur after the
    // packet was partially/fully published; never leave the lease marked
    // neutral based on the PREVIOUS acknowledged input state.
    pub(crate) fn begin_write(&mut self,slot:u32,owner:&str,
        pid:u32,birth:u64,now:u64)->io::Result<()> {
        self.require(slot,owner,pid,birth,now)?;
        let held=self.claims[(slot-1) as usize].as_mut().unwrap();
        held.neutral=false;
        held.until_ms=now.saturating_add(AI_WRITER_LEASE_MS);
        Ok(())
    }
    // Only an acknowledged native shared-state write may restore neutral.
    pub(crate) fn wrote(&mut self,slot:u32,owner:&str,
        pid:u32,birth:u64,now:u64,neutral:bool)->io::Result<()> {
        self.require(slot,owner,pid,birth,now)?;
        let held=self.claims[(slot-1) as usize].as_mut().unwrap();
        held.until_ms=now.saturating_add(AI_WRITER_LEASE_MS);
        held.neutral=neutral;
        Ok(())
    }
    pub(crate) fn release(&mut self,targets:&[(u32,u32,u64)],
        owner:&str,now:u64)->io::Result<()> {
        let slots=targets.iter().map(|(slot,_,_)|*slot).collect::<Vec<_>>();
        validate_ai_lease_slots(&slots,owner)?;
        for &(slot,pid,birth) in targets {
            self.require(slot,owner,pid,birth,now)?;
            if !self.claims[(slot-1) as usize].as_ref().unwrap().neutral {
                return Err(io::Error::new(io::ErrorKind::PermissionDenied,
                    "AI slot must be neutral before relinquishing writer lease"));
            }
        }
        for &(slot,_,_) in targets {self.claims[(slot-1) as usize]=None;}
        Ok(())
    }
    pub(crate) fn legacy_allowed(&self,slot:u32,now:u64)->io::Result<()> {
        if !(1..=5).contains(&slot)||now==0 ||
           self.claims[(slot-1) as usize].as_ref().is_some_and(|c|c.until_ms>now) {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied,
                "An AI slot is held by a different native writer"));
        }
        Ok(())
    }
    pub(crate) fn legacy_wrote(&mut self,slot:u32,now:u64,neutral:bool) {
        self.claims[(slot-1) as usize]=None;
        self.legacy_until_ms[(slot-1) as usize]=
            if neutral{0}else{now.saturating_add(500)};
    }
    pub(crate) fn forget_slot(&mut self,slot:u32) {
        if (1..=5).contains(&slot) {
            self.claims[(slot-1) as usize]=None;
            self.legacy_until_ms[(slot-1) as usize]=0;
        }
    }
}

// Private experimental launch mode: pinned in the original game owner.
// Hybrid reserves MC01 for a human. FiveAi treats MC01 as an opted-in AI;
// input frames cannot change a process's launch mode or game identity.
#[cfg(feature="mc02-native-input")]
#[derive(Clone,Copy,Debug,PartialEq,Eq,Deserialize,Serialize)]
#[serde(rename_all="kebab-case")]
pub enum AgentControlMode { HumanPlusFourAi, FiveAi }

// The historical Mc02Input wire name is retained, but exact native
// slot+channel binding is the final dispatch authority, not this range check.
// Normal product builds have NO native AI input.
#[cfg(feature="mc02-native-input")]
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum NativeInputIntent { Active, ReleaseAll }

#[cfg(feature="mc02-native-input")]
pub(crate) fn admit_native_input_frame(
    slot:u32,keys:u32,dx:i32,dy:i32,ttl_ms:u32,
)->io::Result<NativeInputIntent> {
    // This is ONLY range/value validation, not authorization: native
    // controller requires an opted-in channel on the exact original child.
    if !(1..=5).contains(&slot) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,
            "No authorized native input backend for requested slot"));
    }
    // Reject impossible opposing movement controls even when a caller
    // bypasses the MCP's action planner or sends its own IPC payload.
    // Bits: W=1, S=2, A=4, D=8, jump=16, sneak=32, attack=64, use=128.
    if keys>255 || (keys & 0x03)==0x03 || (keys & 0x0c)==0x0c ||
       !(-127..=127).contains(&dx) || !(-127..=127).contains(&dy) ||
       !(1..=500).contains(&ttl_ms) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,
            "Invalid experimental native input frame"));
    }
    Ok(if keys==0 && dx==0 && dy==0 {
        NativeInputIntent::ReleaseAll
    } else {
        NativeInputIntent::Active
    })
}

pub(crate) type SessionReply = Result<SessionSnapshot, String>;
#[cfg(windows)]
pub(crate) mod pipe;

#[cfg(windows)]
struct PendingRequest {
    replies: std::sync::mpsc::Receiver<io::Result<SessionReply>>,
    worker: std::thread::JoinHandle<()>,
}
#[cfg(windows)]
struct Controller {
    pending: Option<PendingRequest>,
    uncertain: bool,
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

#[cfg(windows)]
pub fn request(request: SessionRequest) -> io::Result<SessionSnapshot> {
    if let SessionRequest::ClearProfileData {slot,ref confirmation_token,ref confirmation_name}=request {
        if !(1..=5).contains(&slot) || confirmation_token.len()!=64 || !confirmation_token.bytes().all(|byte|byte.is_ascii_hexdigit()) || *confirmation_name!=format!("Minecraft - {slot}") {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,"Confirm the exact launcher name before clearing all data"));
        }
    }
    if let SessionRequest::SavePerformance {active_fps,background_fps}=request {
        super::manager_settings::PerformanceSettings {active_fps,background_fps}.validate()?;
    }
    if matches!(request, SessionRequest::ApplyGraphicsPreset { slot, .. } | SessionRequest::Launch { slot } | SessionRequest::CloseOne { slot } | SessionRequest::OpenProfileFolder { slot } | SessionRequest::ClearProfileCache { slot } | SessionRequest::ResetGameSettings { slot } | SessionRequest::RestoreGameSettings { slot } | SessionRequest::DisableImprovedInputResponse { slot } | SessionRequest::EnableTextureStreaming { slot } | SessionRequest::DisableTextureStreaming { slot } if !(1..=5).contains(&slot))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Unknown Minecraft profile",
        ));
    }
    #[cfg(feature="mc02-native-input")]
    if let SessionRequest::LaunchMc02Experiment {slot}=request {
        if slot!=2 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,
                "Legacy experimental input launch only creates MC02"));
        }
    }
    #[cfg(feature="mc02-native-input")]
    if let SessionRequest::LaunchAiExperiment {slot,mode}=request {
        if !(1..=5).contains(&slot) ||
           (mode==AgentControlMode::HumanPlusFourAi && slot==1) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,
                "Hybrid MC01 belongs to the human and has no AI channel"));
        }
    }
    #[cfg(feature="mc02-native-input")]
    if let SessionRequest::Mc02Input {slot,keys,dx,dy,ttl_ms,issued_at_ms}=request {
        // Native IPC must not trust any client-side MCP/CLI button mask.
        // One admission rule covers both active writes and release-all.
        admit_native_input_frame(slot,keys,dx,dy,ttl_ms)?;
        if issued_at_ms==0 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,
                "Missing native input issue timestamp"));
        }
    }
    #[cfg(feature="mc02-native-input")]
    if let SessionRequest::AiInputGroup {
        ref frames,issued_at_ms,deadline_at_ms,ref owner_id,
    } = request {
        validate_ai_input_group(frames,issued_at_ms,deadline_at_ms)?;
        if !valid_ai_writer_id(owner_id) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,
                "Invalid AI writer lease ID"));
        }
    }
    #[cfg(feature="mc02-native-input")]
    if let SessionRequest::AiLeasedInput {
        slot,keys,dx,dy,ttl_ms,issued_at_ms,ref owner_id,
    } = request {
        admit_native_input_frame(slot,keys,dx,dy,ttl_ms)?;
        if issued_at_ms==0||!valid_ai_writer_id(owner_id) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,
                "Invalid leased AI input"));
        }
    }
    #[cfg(feature="mc02-native-input")]
    if let SessionRequest::AiLeaseAcquire {ref slots,ref owner_id,mode}=request {
        validate_ai_lease_slots(slots,owner_id)?;
        if mode==AgentControlMode::HumanPlusFourAi && slots.contains(&1) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,
                "Hybrid MC01 human slot cannot acquire AI writer lease"));
        }
    }
    #[cfg(feature="mc02-native-input")]
    if let SessionRequest::AiLeaseRelease {ref slots,ref owner_id}=request {
        validate_ai_lease_slots(slots,owner_id)?;
    }
    let inspecting = matches!(request, SessionRequest::Inspect | SessionRequest::InspectSettings);
    #[cfg(feature="mc02-native-input")]
    let neutral_mc02_release = matches!(request,
        SessionRequest::Mc02Input {keys:0,dx:0,dy:0,..} |
        SessionRequest::AiLeasedInput {keys:0,dx:0,dy:0,..});
    #[cfg(not(feature="mc02-native-input"))]
    let neutral_mc02_release = false;
    let mut owner = admit_controller(&CONTROLLER)?;
    if let Some(pending) = &owner.pending {
        match pending.replies.try_recv() {
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "The previous session request is still running; games are preserved",
                ))
            }
            _ => {
                let pending = owner.pending.take().unwrap();
                let _ = pending.worker.join();
            }
        }
    }
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
    let (sender, replies) = std::sync::mpsc::sync_channel(1);
    let worker = std::thread::spawn(move || {
        let _ = sender.send(transact(request));
    });
    owner.pending = Some(PendingRequest { replies, worker });
    let outcome = owner
        .pending
        .as_ref()
        .unwrap()
        .replies
        .recv_timeout(std::time::Duration::from_secs(45));
    match outcome {
        Ok(result) => {
            let pending = owner.pending.take().unwrap();
            let _ = pending.worker.join();
            match result {
                Ok(reply) => {
                    if inspecting && reply.is_ok() {
                        owner.uncertain = false;
                    }
                    reply.map_err(io::Error::other)
                }
                Err(error) => {
                    owner.uncertain = true;
                    Err(error)
                }
            }
        }
        Err(_) => {
            // Keep this exact worker/buffers/connection alive. Never start a replacement on timeout.
            owner.uncertain = true;
            Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "Session request is still pending; refresh to observe its eventual outcome",
            ))
        }
    }
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
    if existing_controller_gate("Local\\MivubiEducationStartupObservation")? {
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
    if matches!(request, SessionRequest::Inspect | SessionRequest::InspectSettings | SessionRequest::SavePerformance { .. } | SessionRequest::SaveAudio {..}) {
        let mut snapshot = SessionSnapshot::stopped();
        if let SessionRequest::SaveAudio {mode}=request {super::manager_settings::save_audio(mode)?;}
        if let SessionRequest::SavePerformance { active_fps, background_fps } = request {
            super::manager_settings::save_performance(super::manager_settings::PerformanceSettings {active_fps,background_fps})?;
        }
        if !matches!(request, SessionRequest::Inspect) {
            let idle = super::native_clients::inspect_native_workspace()?.processes.is_empty();
            snapshot.settings = Some(SettingsReport::policy(idle, None)?);
        }
        return Ok(Ok(snapshot));
    }
    // Input operations require an already retained authenticated controller.
    // Do not invent a new owner, replay an old channel or adopt a game
    // merely because an input command mentions its numerical slot.
    #[cfg(feature="mc02-native-input")]
    if matches!(request,SessionRequest::Mc02Input{..} |
        SessionRequest::AiInputGroup{..} |
        SessionRequest::AiLeasedInput{..} |
        SessionRequest::AiLeaseAcquire{..} |
        SessionRequest::AiLeaseRelease{..} |
        SessionRequest::LaunchMc02Experiment{..} |
        SessionRequest::LaunchAiExperiment{..}) {
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
            for operation in [SessionRequest::ResetGameSettings {slot},SessionRequest::RestoreGameSettings {slot},SessionRequest::DisableImprovedInputResponse {slot},SessionRequest::EnableTextureStreaming {slot},SessionRequest::DisableTextureStreaming {slot},SessionRequest::CloseOne {slot}] {
                assert_eq!(request(operation).unwrap_err().kind(),io::ErrorKind::InvalidInput);
            }
        }
        assert!(serde_json::from_str::<SessionRequest>(r#"{"operation":"RestoreGameSettings","slot":1,"path":"outside"}"#).is_err());
        assert!(serde_json::from_str::<SessionRequest>(r#"{"operation":"CloseOne","slot":1,"pid":123}"#).is_err());
        assert!(matches!(serde_json::from_str::<SessionRequest>(r#"{"operation":"CloseOne","slot":3}"#),Ok(SessionRequest::CloseOne{slot:3})));
        for (slot,token,name) in [(0,"a".repeat(64),"Minecraft - 0"),(1,"bad".into(),"Minecraft - 1"),(1,"a".repeat(64),"Minecraft - 2")] {
            assert_eq!(request(SessionRequest::ClearProfileData {slot,confirmation_token:token,confirmation_name:name.into()}).unwrap_err().kind(),io::ErrorKind::InvalidInput);
        }
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
    fn original_native_actor_lease_is_atomic_and_process_pinned() {
        // Pure book fixture: no Windows calls, Minecraft, IPC or input hooks.
        let a="a".repeat(32);
        let b="b".repeat(32);
        let four=vec![(2,102,1002),(3,103,1003),(4,104,1004),(5,105,1005)];
        let mut five=vec![(1,101,1001)];
        five.extend(four.iter().copied());
        let mut book=AiLeaseBook::default();
        assert!(book.acquire(&four,&a,1000).is_ok());
        for &(slot,pid,birth) in &four {
            assert!(book.require(slot,&a,pid,birth,1001).is_ok());
            assert!(book.require(slot,&b,pid,birth,1001).is_err());
            assert!(book.require(slot,&a,pid,birth+1,1001).is_err());
        }
        // A contested five-slot request must NOT steal idle MC01.
        assert!(book.acquire(&five,&b,1002).is_err());
        assert!(book.require(1,&b,101,1001,1002).is_err());
        // An outstanding key-down forbids releasing the claim to a rival.
        book.wrote(2,&a,102,1002,1100,false).unwrap();
        assert!(book.release(&four,&a,1101).is_err());
        assert!(book.acquire(&[(2,102,1002)],&b,1101).is_err());
        book.wrote(2,&a,102,1002,1110,true).unwrap();
        assert!(book.release(&four,&a,1111).is_ok());
        assert!(book.acquire(&five,&b,1200).is_ok());
        assert!(book.require(1,&b,101,1001,1201).is_ok());
        // Original process identity cannot be inherited by a new game.
        assert!(book.acquire(&[(2,999,1002)],&b,1201).is_err());
        book.forget_slot(2);
        assert!(book.acquire(&[(2,999,9002)],&a,1202).is_ok());
        assert!(book.require(2,&b,102,1002,1203).is_err());
        assert!(book.require(2,&a,999,9002,1203).is_ok());
        // A writer idle beyond expiry must re-acquire. Packet TTL
        // (350ms) is shorter than lease lifetime; no held key proof.
        assert!(book.require(2,&a,999,9002,
            1202+AI_WRITER_LEASE_MS).is_err());
        assert!(book.acquire(&[(2,999,9002)],&b,
            1203+AI_WRITER_LEASE_MS).is_ok());
    }

    #[cfg(feature="mc02-native-input")]
    #[test]
    fn writer_acquisition_wire_is_mode_bound_before_any_native_claim() {
        // No IPC, controller or Minecraft execution.
        let owner="a".repeat(32);
        let packet=serde_json::json!({
            "operation":"AiLeaseAcquire","slots":[2,3,4,5],
            "owner_id":owner,"mode":"human-plus-four-ai"
        });
        assert!(matches!(serde_json::from_value::<SessionRequest>(packet.clone()),
            Ok(SessionRequest::AiLeaseAcquire {
                mode:AgentControlMode::HumanPlusFourAi,..})));
        let mut missing=packet.clone();
        missing.as_object_mut().unwrap().remove("mode");
        assert!(serde_json::from_value::<SessionRequest>(missing).is_err());
        let mut unsupported=packet.clone();
        unsupported["mode"]=serde_json::json!("unknown");
        assert!(serde_json::from_value::<SessionRequest>(unsupported).is_err());
        let five=serde_json::json!({
            "operation":"AiLeaseAcquire","slots":[1,2,3,4,5],
            "owner_id":"a".repeat(32),"mode":"five-ai"
        });
        assert!(matches!(serde_json::from_value::<SessionRequest>(five),
            Ok(SessionRequest::AiLeaseAcquire {
                mode:AgentControlMode::FiveAi,..})));
    }
    #[cfg(feature="mc02-native-input")]
    #[test]
    fn interrupted_native_writer_never_releases_unconfirmed_input() {
        // Pure lease-book fixture only: the controller's fail-closed path
        // must not report safe neutral when the channel write fails.
        let owner="a".repeat(32);
        let group=vec![(2,202,2202),(3,203,2203),(4,204,2204),(5,205,2205)];
        let mut book=AiLeaseBook::default();
        book.acquire(&group,&owner,1000).unwrap();
        book.begin_write(3,&owner,203,2203,1010).unwrap();
        assert!(book.release(&group,&owner,1011).is_err());
        assert!(book.acquire(&[(3,203,2203)],&"b".repeat(32),1012).is_err());
        // Even if MC03's channel becomes unavailable, other original
        // channels that never had an uncertain write can be released.
        let safe=vec![(2,202,2202),(4,204,2204),(5,205,2205)];
        assert!(book.release(&safe,&owner,1020).is_ok());
        assert!(book.require(3,&owner,203,2203,1021).is_ok());
        book.begin_write(3,&owner,203,2203,1030).unwrap();
        // A known-good native neutral write restores releasability;
        // merely attempting ReleaseAll does NOT.
        assert!(book.release(&[(3,203,2203)],&owner,1031).is_err());
        book.wrote(3,&owner,203,2203,1032,true).unwrap();
        assert!(book.release(&[(3,203,2203)],&owner,1033).is_ok());
        // Explicit finite expiration (10 seconds) still prevents a
        // dead MCP from holding a process forever.
        book.acquire(&[(3,203,2203)],&owner,2000).unwrap();
        book.begin_write(3,&owner,203,2203,2001).unwrap();
        assert!(book.require(3,&owner,203,2203,2001+AI_WRITER_LEASE_MS).is_err());
        assert!(book.acquire(&[(3,203,2203)],&"b".repeat(32),
            2002+AI_WRITER_LEASE_MS).is_ok());
    }
    #[cfg(feature="mc02-native-input")]
    #[test]
    fn writer_lease_rejects_invalid_protocol_and_legacy_overlap() {
        let mut book=AiLeaseBook::default();
        let a="a".repeat(32);
        assert!(validate_ai_lease_slots(&[2,3,4,5],&a).is_ok());
        for id in ["bad".to_owned(),"A".repeat(32),"x".repeat(32)] {
            assert!(validate_ai_lease_slots(&[2],&id).is_err());
        }
        for slots in [vec![],vec![0],vec![6],vec![2,2],
            vec![3,2],vec![1,2,3,4,5,6]] {
            assert!(validate_ai_lease_slots(&slots,&a).is_err());
        }
        book.legacy_wrote(2,1000,false);
        assert!(book.acquire(&[(2,202,2002)],&a,1300).is_err());
        assert!(book.acquire(&[(2,202,2002)],&a,1501).is_ok());
        assert!(book.legacy_allowed(2,1502).is_err());
        // ReleaseAll is not a bypass around another live writer.
        assert!(book.legacy_allowed(2,1503).is_err());
        book.wrote(2,&a,202,2002,1510,true).unwrap();
        book.release(&[(2,202,2002)],&a,1511).unwrap();
        assert!(book.legacy_allowed(2,1512).is_ok());
        book.legacy_wrote(2,1512,true);
        assert!(book.acquire(&[(2,202,2002)],&a,1513).is_ok());
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
            "owner_id":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#;
        assert!(matches!(serde_json::from_str::<SessionRequest>(json),
            Ok(SessionRequest::AiInputGroup{..})));
        assert!(serde_json::from_str::<SessionRequest>(
            &json.replace(",\"owner_id\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"", "")
        ).is_err()); // Missing writer must not fall back to the old unleased group.
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
