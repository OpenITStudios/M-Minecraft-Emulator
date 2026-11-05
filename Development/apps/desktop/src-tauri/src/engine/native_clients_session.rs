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

// A bounded, short-lived UX mirror of an agent CLI report. Not a second
// game/QA owner, not proof of Windows desktop control or Minecraft input.
#[derive(Clone,Debug,Deserialize,Serialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct AgentProbeCounts {
    pub expected:u32,
    pub pass:u32,
    pub fail:u32,
    pub info:u32,
    pub look:u32,
}
#[derive(Clone,Debug,Deserialize,Serialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct AgentActivityItem {
    pub kind:String,
    pub slot:u32,
    pub status:String,
    pub importance:String,
    pub evidence:String,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub reported_outcome:Option<String>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub probe_counts:Option<AgentProbeCounts>,
}
#[derive(Clone,Debug,Deserialize,Serialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct AgentActivityReport {
    pub schema:u32,
    pub source:String,
    pub snapshot_at:String,
    pub display_hint:String,
    pub desktop_control:String,
    pub minecraft_input_effect:String,
    pub map_gameplay_acceptance:String,
    pub items:Vec<AgentActivityItem>,
    #[serde(default)]
    pub stale:bool,
}
impl AgentActivityReport {
    pub fn valid_ingress(&self)->bool {
        if self.schema!=1||self.stale||self.items.len()>15||
           self.source!="CURRENT_MCP_PROCESS_MEMORY_ONLY"||
           !(20..=40).contains(&self.snapshot_at.len())||
           !self.snapshot_at.ends_with('Z')||
           self.desktop_control!="NOT_OBSERVED_NO_AUTHENTICATED_DESKTOP_PROVIDER"||
           self.minecraft_input_effect!="NOT_VERIFIED"||
           self.map_gameplay_acceptance!="NOT_TESTED" {
            return false;
        }
        let mut seen=std::collections::HashSet::new();
        let mut attention=false;
        for item in &self.items {
            if !(1..=5).contains(&item.slot) ||
               !seen.insert((item.kind.as_str(),item.slot)) ||
               !matches!(item.importance.as_str(),"INFORMATION"|"ATTENTION") {
                return false;
            }
            attention|=item.importance=="ATTENTION";
            let allowed=match item.kind.as_str() {
                "AI_INPUT" => {
                    item.evidence=="LOCAL_MCP_RECOVERY_HOLD" &&
                      matches!(item.status.as_str(),
                        "NATIVE_RELEASE_UNCONFIRMED"|
                        "STARTUP_RECONCILIATION_REQUIRED")
                    || item.evidence=="LOCAL_MCP_REQUEST_ONLY" &&
                       item.status=="EXPERIMENTAL_INPUT_REQUEST_IN_FLIGHT"
                },
                "VISUAL_OBSERVATION" =>
                    item.evidence=="LOCAL_SAMPLER_ONLY"&&
                    matches!(item.status.as_str(),
                      "SAMPLER_ACTIVE_NO_RECENT_FRAME"|
                      "SAMPLER_ACTIVE_RECENT_FRAME"),
                "SKILLS_PROBE" => {
                    let expected=if item.status=="PROBE_REPORT_AVAILABLE" {
                        "ORIGINAL_PROBE_LOG_UNVERIFIED_ORIGIN"
                    } else if matches!(item.status.as_str(),
                        "LOCAL_QA_REVIEW_IN_PROGRESS"|
                        "LOCAL_QA_PREPARATION_IN_PROGRESS") {
                        "MCP_PROCESS_ONLY"
                    } else { "SOURCE_CHECKPOINT_ONLY" };
                    item.evidence==expected&&
                    matches!(item.status.as_str(),
                      "PROBE_REPORT_AVAILABLE"|
                      "LOCAL_QA_REVIEW_IN_PROGRESS"|
                      "LOCAL_QA_PREPARATION_IN_PROGRESS"|
                      "AWAITING_AUTHORIZED_COMMAND_OR_EVIDENCE"|
                      "CHECKPOINT_EXPIRED_OR_UNAVAILABLE")
                },
                _=>false,
            };
            let outcome_allowed=match item.reported_outcome.as_deref() {
                None=>item.status!="PROBE_REPORT_AVAILABLE",
                Some(value)=>item.kind=="SKILLS_PROBE" &&
                    item.status=="PROBE_REPORT_AVAILABLE" &&
                    matches!(value,"PROBE_REPORTED_PASS"|"FAIL"|
                      "NEEDS_REVIEW"|"INCOMPLETE"|"BLOCKED"|
                      "UNRECOGNIZED_TERMINAL_STATUS"),
            };
            // Counts may be forwarded only from a complete original
            // Probe status, never from partial/blocked reports or AI input.
            let counts_allowed=match item.probe_counts.as_ref() {
                None=>true,
                Some(counts)=>{
                    let outcome=item.reported_outcome.as_deref();
                    let completed=matches!(outcome,
                        Some("PROBE_REPORTED_PASS"|"FAIL"|"NEEDS_REVIEW"));
                    let total=u64::from(counts.pass)+u64::from(counts.fail)+
                        u64::from(counts.info)+u64::from(counts.look);
                    completed&&item.kind=="SKILLS_PROBE"&&
                        item.status=="PROBE_REPORT_AVAILABLE"&&
                        (1..=80).contains(&counts.expected)&&
                        [counts.pass,counts.fail,counts.info,counts.look]
                          .iter().all(|value|*value<=80)&&
                        total==u64::from(counts.expected)&&
                        match outcome {
                            Some("PROBE_REPORTED_PASS") =>
                                counts.pass==counts.expected,
                            Some("FAIL")=>counts.fail>0,
                            Some("NEEDS_REVIEW")=>
                                counts.fail==0&&counts.info+counts.look>0,
                            _=>false,
                        }
                }
            };
            let important=matches!(item.status.as_str(),
                "NATIVE_RELEASE_UNCONFIRMED"|
                "STARTUP_RECONCILIATION_REQUIRED"|
                "CHECKPOINT_EXPIRED_OR_UNAVAILABLE") ||
                (item.status=="PROBE_REPORT_AVAILABLE" &&
                 item.reported_outcome.as_deref()!=Some("PROBE_REPORTED_PASS"));
            if !allowed||!outcome_allowed||!counts_allowed||
               (item.importance=="ATTENTION")!=important{return false;}
        }
        self.display_hint==if attention{"ATTENTION"}
          else if self.items.is_empty(){"HIDDEN_NO_LOCAL_SIGNAL"}
          else {"ACTIVITY"}
    }
}


/* A correlation/fencing book in the retained ORIGINAL native controller.
   It does not execute QA, read Skills Probe evidence or certify game/world.
   One reservation and immutable FINAL reference per native MC slot.
   Controller restart loses this volatile book: it is NOT a durable archive. */
// Read-only snapshot of the original controller's in-memory Testing run
// reservation, never proof that the owning AI process is alive/dead.
#[derive(Clone,Debug,PartialEq,Eq,Deserialize,Serialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct TestingRunInspection {
    pub schema:u32,
    pub slot:u32,
    pub phase:String,
    pub run_id:Option<String>,
    pub owner_token_match:bool,
    pub reported_status:Option<String>,
    pub report_digest:Option<String>,
    pub owner_process_proof:String,
    pub operator_approval:String,
    pub transfer_allowed:bool,
    pub recovery_action:String,
    pub provenance:String,
}
#[derive(Clone,Debug,PartialEq,Eq,Deserialize,Serialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct TestingRunView {
    pub schema:u32,
    pub slot:u32,
    pub run_id:String,
    pub phase:String,
    pub owner_match:bool,
    // Serialize explicit null in a RESERVED receipt: the JS one-shot
    // adapter must distinguish "not finalized" from a malformed reply.
    #[serde(default)]
    pub reported_status:Option<String>,
    #[serde(default)]
    pub report_digest:Option<String>,
    pub provenance:String,
}
#[derive(Debug)]
struct TestingRecord {
    owner:String,
    view:TestingRunView,
    #[cfg(windows)]
    owner_process:Option<pipe::McpOwnerLifetime>,
}
// Stage-only recovery capability: NO production constructor or IPC/Tauri
// deserialization exists. These private markers must eventually be issued
// only by independently authenticated Windows operator presence and a
// native, revalidated immutable-archive decision in this controller.
// Neither a model-provided boolean nor a source-only MCP archive report
// can create either marker. The only constructors are in pure unit fixtures.
#[allow(dead_code)]
struct VerifiedTestingOperatorPresence(());
#[allow(dead_code)]
struct VerifiedNativeTestingArchive(());
#[allow(dead_code)]
struct TestingRecoveryAdmission {
    slot:u32,
    expected_run_id:String,
    expected_owner:String,
    new_run_id:String,
    new_owner:String,
    _operator:VerifiedTestingOperatorPresence,
    _archive:VerifiedNativeTestingArchive,
}
pub(crate) struct NativeTestingRunBook {
    slots:[Option<TestingRecord>;5],
}
impl Default for NativeTestingRunBook {
    fn default()->Self {
        Self {slots:std::array::from_fn(|_|None)}
    }
}
pub(crate) fn valid_testing_id(id:&str)->bool {
    id.len()==32 && id.bytes().all(|b|b.is_ascii_hexdigit()&&!b.is_ascii_uppercase())
}
fn valid_testing_digest(id:&str)->bool {
    id.len()==64 && id.bytes().all(|b|b.is_ascii_hexdigit()&&!b.is_ascii_uppercase())
}
fn valid_testing_outcome(outcome:&str)->bool {
    matches!(outcome,"PROBE_REPORTED_PASS"|"FAIL"|"NEEDS_REVIEW"|"INCOMPLETE"|"BLOCKED")
}
impl NativeTestingRunBook {
    pub(crate) fn inspect(
        &self,slot:u32,caller_token:&str,
    )->Result<TestingRunInspection,&'static str> {
        if !(1..=5).contains(&slot)||!valid_testing_id(caller_token) {
            return Err("TESTING_INSPECTION_INVALID");
        }
        let record=self.slots[(slot-1) as usize].as_ref();
        let (phase,run_id,owner_match,reported_status,digest,action)=
            if let Some(held)=record {
                (held.view.phase.clone(),Some(held.view.run_id.clone()),
                 held.owner==caller_token,held.view.reported_status.clone(),
                 held.view.report_digest.clone(),
                 if held.view.phase=="RESERVED" {
                    "REQUIRE_VERIFIED_OWNER_DEATH_AND_OPERATOR_APPROVAL"
                 } else {"READ_FINAL_REPORT_NO_OWNER_TRANSFER"})
            } else {
                // A controller restart ALSO produces NO_RECORD. Do not
                // infer there was never a prior run from this state.
                ("NO_RECORD".into(),None,false,None,None,
                 "CHECK_ARCHIVE_AND_CONTROLLER_EPOCH")
            };
        // A token match is not a process proof. Only a retained OS handle
        // can report its pinned parent lifetime; no takeover is permitted.
        #[cfg(windows)]
        let owner_proof=record.and_then(|held|held.owner_process.as_ref())
            .map_or("NOT_BOUND_TO_PROCESS_HANDLE",|pin|pin.status());
        #[cfg(not(windows))]
        let owner_proof="NOT_BOUND_TO_PROCESS_HANDLE";
        Ok(TestingRunInspection {
            schema:1,slot,phase,run_id,
            owner_token_match:owner_match,
            reported_status,report_digest:digest,
            owner_process_proof:owner_proof.into(),
            operator_approval:"NOT_AVAILABLE_IN_PROTOCOL".into(),
            transfer_allowed:false,
            recovery_action:action.into(),
            provenance:"RETAINED_CONTROLLER_MEMORY_ONLY_NOT_GAMEPLAY_PROOF".into(),
        })
    }
    // Internal pure transition; live native requests use reserve_from_peer.
    fn reserve(
        &mut self,slot:u32,owner:&str,run_id:&str,replace_run_id:Option<&str>,
    )->Result<TestingRunView,&'static str> {
        if !(1..=5).contains(&slot)||
           !valid_testing_id(owner)||!valid_testing_id(run_id)||
           replace_run_id.is_some_and(|s|!valid_testing_id(s)) {
            return Err("TESTING_RUN_INVALID_ID");
        }
        let entry=&mut self.slots[(slot-1) as usize];
        if let Some(held)=entry {
            let same=held.owner==owner;
            if same&&held.view.run_id==run_id&&replace_run_id.is_none() {
                let mut view=held.view.clone();
                view.owner_match=true;
                return Ok(view);
            }
            if same&&held.view.phase=="FINAL"&&
              replace_run_id==Some(held.view.run_id.as_str())&&
              run_id!=held.view.run_id {
                // Explicit restart after immutable terminal reference only.
                *entry=None;
            } else {
                let mut view=held.view.clone();
                view.owner_match=false; // Not permission to overwrite.
                return Ok(view);
            }
        } else if replace_run_id.is_some() {
            return Err("TESTING_PREVIOUS_RUN_UNAVAILABLE");
        }
        let view=TestingRunView {
            schema:1,slot,run_id:run_id.into(),
            phase:"RESERVED".into(),owner_match:true,
            reported_status:None,report_digest:None,
            provenance:"NATIVE_RUN_CORRELATION_ONLY_NOT_GAMEPLAY_PROOF".into(),
        };
        *entry=Some(TestingRecord{
            owner:owner.into(),view:view.clone(),
            #[cfg(windows)]
            owner_process:None,
        });
        Ok(view)
    }
    // Internal pure transition; live native requests use finalize_from_peer.
    fn finalize(
        &mut self,slot:u32,owner:&str,run_id:&str,
        outcome:&str,digest:&str,
    )->Result<TestingRunView,&'static str> {
        if !(1..=5).contains(&slot)||!valid_testing_id(owner)||
           !valid_testing_id(run_id)||!valid_testing_digest(digest)||
           !valid_testing_outcome(outcome) {
            return Err("TESTING_FINAL_REPORT_INVALID");
        }
        let record=self.slots[(slot-1) as usize].as_mut()
            .ok_or("TESTING_RESERVATION_REQUIRED")?;
        if record.owner!=owner||record.view.run_id!=run_id {
            return Err("TESTING_RUN_OWNER_MISMATCH");
        }
        if record.view.phase=="FINAL" {
            if record.view.reported_status.as_deref()==Some(outcome) &&
               record.view.report_digest.as_deref()==Some(digest) {
                return Ok(record.view.clone());
            }
            return Err("TESTING_FINAL_RESULT_IMMUTABLE");
        }
        record.view.phase="FINAL".into();
        record.view.reported_status=Some(outcome.into());
        record.view.report_digest=Some(digest.into());
        Ok(record.view.clone())
    }

    #[cfg(windows)]
    pub(crate) fn reserve_from_peer(
        &mut self,slot:u32,owner:&str,run_id:&str,
        replace_run_id:Option<&str>,peer:&pipe::Peer,
    )->Result<TestingRunView,&'static str> {
        if !(1..=5).contains(&slot) {return Err("TESTING_RUN_INVALID_ID");}
        let incoming=peer.mcp_parent()
            .map_err(|_|"TESTING_MCP_OWNER_PROCESS_UNVERIFIED")?;
        if let Some(held)=self.slots[(slot-1) as usize].as_ref() {
            if held.owner==owner && !held.owner_process.as_ref().is_some_and(
                |original|original.same(&incoming) &&
                  original.status()=="BOUND_PARENT_RUNNING_AT_INSPECTION") {
                return Err("TESTING_MCP_OWNER_PROCESS_CHANGED");
            }
        }
        if incoming.status()!="BOUND_PARENT_RUNNING_AT_INSPECTION" {
            return Err("TESTING_MCP_OWNER_PROCESS_UNVERIFIED");
        }
        let view=self.reserve(slot,owner,run_id,replace_run_id)?;
        if view.owner_match {
            // A new/owned reservation pins the same verified parent OS
            // process, rather than the disposable CLI or caller token.
            let held=self.slots[(slot-1) as usize].as_mut()
                .ok_or("TESTING_RESERVATION_REQUIRED")?;
            held.owner_process=Some(incoming);
        }
        Ok(view)
    }

    #[cfg(windows)]
    pub(crate) fn finalize_from_peer(
        &mut self,slot:u32,owner:&str,run_id:&str,
        outcome:&str,digest:&str,peer:&pipe::Peer,
    )->Result<TestingRunView,&'static str> {
        if !(1..=5).contains(&slot) {
            return Err("TESTING_FINAL_REPORT_INVALID");
        }
        let incoming=peer.mcp_parent()
            .map_err(|_|"TESTING_MCP_OWNER_PROCESS_UNVERIFIED")?;
        let held=self.slots[(slot-1) as usize].as_ref()
            .ok_or("TESTING_RESERVATION_REQUIRED")?;
        if held.owner!=owner || held.view.run_id!=run_id ||
           !held.owner_process.as_ref().is_some_and(
              |original|original.same(&incoming) &&
                original.status()=="BOUND_PARENT_RUNNING_AT_INSPECTION") ||
           incoming.status()!="BOUND_PARENT_RUNNING_AT_INSPECTION" {
            return Err("TESTING_MCP_OWNER_PROCESS_CHANGED");
        }
        self.finalize(slot,owner,run_id,outcome,digest)
    }
    // Compare-only portion of one prospective transfer. No ownership is
    // changed by a preflight and it does NOT authenticate the admission.
    // The actual internal CAS below independently pins the old + new OS
    // process lifetime just before replacing the ONE native slot record.
    #[allow(dead_code)]
    fn check_recovery_cas(
        &self,permit:&TestingRecoveryAdmission,
    )->Result<(),&'static str> {
        if !(1..=5).contains(&permit.slot)||
           !valid_testing_id(&permit.expected_run_id)||
           !valid_testing_id(&permit.expected_owner)||
           !valid_testing_id(&permit.new_run_id)||
           !valid_testing_id(&permit.new_owner)||
           permit.new_run_id==permit.expected_run_id||
           permit.new_owner==permit.expected_owner {
            return Err("TESTING_TRANSFER_INVALID_CONTRACT");
        }
        let current=self.slots[(permit.slot-1) as usize].as_ref()
            .ok_or("TESTING_TRANSFER_OLD_RUN_UNAVAILABLE")?;
        if current.view.run_id!=permit.expected_run_id||
           current.owner!=permit.expected_owner {
            return Err("TESTING_TRANSFER_STALE_EXPECTED_RUN");
        }
        if current.view.phase!="RESERVED"||
           current.view.reported_status.is_some()||
           current.view.report_digest.is_some() {
            return Err("TESTING_TRANSFER_FINAL_OR_INCONSISTENT");
        }
        Ok(())
    }

    // No live route currently calls this method. All markers are private,
    // have no production issuers, and cannot be built from wire, UI or MCP
    // values. It remains unreachable until independent native operator
    // authentication + archive reconciliation have been implemented.
    #[cfg(windows)]
    #[allow(dead_code)]
    fn transfer_with_sealed_admission(
        &mut self,permit:TestingRecoveryAdmission,
        successor:pipe::McpOwnerLifetime,
    )->Result<TestingRunView,&'static str> {
        self.check_recovery_cas(&permit)?;
        let index=(permit.slot-1) as usize;
        let old=self.slots[index].as_ref()
            .ok_or("TESTING_TRANSFER_OLD_RUN_UNAVAILABLE")?;
        let old_pin=old.owner_process.as_ref()
            .ok_or("TESTING_TRANSFER_OLD_PROCESS_NOT_PINNED")?;
        // A time threshold, a stale caller token, or an UNKNOWN process
        // state can NEVER release the original reservation.
        if old_pin.status()!="BOUND_PARENT_EXIT_CONFIRMED" {
            return Err("TESTING_TRANSFER_OLD_PROCESS_NOT_CONFIRMED_EXITED");
        }
        if successor.status()!="BOUND_PARENT_RUNNING_AT_INSPECTION"||
           successor.same(old_pin) {
            return Err("TESTING_TRANSFER_NEW_PROCESS_NOT_VERIFIED");
        }
        // No await, filesystem access, IPC request or guest action between
        // final checks and this single-slot replacement. The old run is
        // never reused; previous same-token finalize now fails closed.
        let view=TestingRunView {
            schema:1,slot:permit.slot,run_id:permit.new_run_id,
            phase:"RESERVED".into(),owner_match:true,
            reported_status:None,report_digest:None,
            provenance:"NATIVE_RUN_CORRELATION_ONLY_NOT_GAMEPLAY_PROOF".into(),
        };
        self.slots[index]=Some(TestingRecord {
            owner:permit.new_owner,view:view.clone(),
            owner_process:Some(successor),
        });
        Ok(view)
    }

}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SessionSnapshot {
    pub schema: u32,
    pub profiles: Vec<ProfileSession>,
    pub storage: Option<Vec<ProfileStorage>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settings: Option<SettingsReport>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub agent_activity:Option<AgentActivityReport>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub testing_run:Option<TestingRunView>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub testing_inspection:Option<TestingRunInspection>,
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
            agent_activity: None,
            testing_run: None,
            testing_inspection: None,
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
    // Status-only mirror; desktop Tauri requests cannot publish this.
    PublishAgentActivity {activity:AgentActivityReport},
    // Original controller reserves per-slot run correlation across separate
    // same-image MCP clients. Never starts Skills Probe or games.
    TestingReserve {slot:u32,owner_id:String,run_id:String,
        #[serde(default)] replace_run_id:Option<String>},
    TestingFinalize {slot:u32,owner_id:String,run_id:String,
        reported_status:String,report_digest:String},
    TestingInspect {slot:u32,owner_id:String},
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
        owner_id:String,lease_epoch:String,
    },
    // A single retained controller atomically admits all original game
    // slots for one ephemeral MCP writer before any movement packet.
    #[cfg(feature="mc02-native-input")]
    AiLeaseAcquire {
        slots:Vec<u32>,owner_id:String,mode:AgentControlMode,
        lease_epoch:String,neutral_only:bool,
    },
    #[cfg(feature="mc02-native-input")]
    AiLeaseRelease {slots:Vec<u32>,owner_id:String,lease_epoch:String},
    // One authenticated request for all per-client states due at the SAME
    // timestamp. This is a bounded group, not a promise of atomic Win32
    // delivery or simultaneous Minecraft polling.
    #[cfg(feature="mc02-native-input")]
    AiInputGroup {
        frames:Vec<AiInputFrame>,issued_at_ms:u64,deadline_at_ms:u64,
        owner_id:String,lease_epoch:String,
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

// P31 temporal binding prevents a delayed pre-relaunch acquisition from
// adopting a newer original Minecraft process. Birth is the retained
// Windows GetProcessTimes FILETIME, never a window title or MCP claim.
// The epoch is a 32-hex private correlator, not a signed credential;
// this is not resistance to a malicious same-user process.
#[cfg(feature="mc02-native-input")]
pub(crate) fn current_ai_filetime()->io::Result<u64> {
    const UNIX_TO_FILETIME_100NS:u128=116_444_736_000_000_000;
    let elapsed=std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_|io::Error::other("AI epoch clock unavailable"))?;
    u64::try_from(UNIX_TO_FILETIME_100NS+
        elapsed.as_nanos()/100)
        .map_err(|_|io::Error::other("AI epoch clock out of range"))
}
#[cfg(feature="mc02-native-input")]
pub(crate) fn validate_ai_acquire_epoch(
    epoch:&str,now_filetime:u64,targets:&[(u32,u32,u64)]
)->io::Result<()> {
    // CLI acquisition is at most six seconds, allow a bounded ten-second
    // window. One-second clock lead tolerance is fail-closed on larger drift.
    const MAX_AGE_100NS:u64=100_000_000;
    const MAX_FUTURE_100NS:u64=10_000_000;
    let issued=if valid_ai_writer_id(epoch)&&epoch.ends_with("e1") {
        u64::from_str_radix(&epoch[..16],16).ok()
    }else {None};
    if now_filetime==0||targets.is_empty()||
       !issued.is_some_and(|at|at!=0 &&
           at<=now_filetime.saturating_add(MAX_FUTURE_100NS) &&
           now_filetime<=at.saturating_add(MAX_AGE_100NS) &&
           targets.iter().all(|(_,pid,birth)|
               *pid!=0&&*birth!=0&&*birth<=at)) {
        return Err(io::Error::new(io::ErrorKind::PermissionDenied,
            "AI transaction predates original game or is stale"));
    }
    Ok(())
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
    owner_id:String,lease_epoch:String,pid:u32,birth:u64,
    until_ms:u64,neutral:bool,neutral_only:bool,
}
#[cfg(feature="mc02-native-input")]
#[derive(Debug)]
pub(crate) struct AiLeaseBook {
    claims:[Option<AiWriterClaim>;5],
    legacy_until_ms:[u64;5],
    // Packet TTL only bounds virtual polling; it cannot attest that the
    // opted-in game consumed a neutral release. Keep this per-slot
    // unresolved legacy input bit until a confirmed native neutral write
    // or original-process reap, just like the modern writer book.
    legacy_neutral_pending:[bool;5],
}
#[cfg(feature="mc02-native-input")]
impl Default for AiLeaseBook {
    fn default()->Self {
        Self{claims:std::array::from_fn(|_|None),legacy_until_ms:[0;5],
            legacy_neutral_pending:[false;5]}
    }
}
#[cfg(feature="mc02-native-input")]
impl AiLeaseBook {
    // The ephemeral lease_epoch is a per-TRANSACTION fencing value.
    // It is not OS authentication. A delayed CLI from a previous
    // lease/release (even with the same MCP writer ID) must never
    // touch a newly launched child after the original slot was reaped.
    pub(crate) fn acquire(&mut self,targets:&[(u32,u32,u64)],
        owner:&str,epoch:&str,now:u64,neutral_only:bool)->io::Result<()> {
        let slots=targets.iter().map(|(slot,_,_)|*slot).collect::<Vec<_>>();
        validate_ai_lease_slots(&slots,owner)?;
        if !valid_ai_writer_id(epoch) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,
                "Invalid AI transaction fencing epoch"));
        }
        if now==0 || targets.iter().any(|(slot,pid,birth)| {
            *pid==0||*birth==0 ||
            self.legacy_until_ms[(*slot-1) as usize]>now ||
            self.legacy_neutral_pending[(*slot-1) as usize] ||
            self.claims[(*slot-1) as usize].as_ref().is_some_and(|held|
                // Expiry retires WRITE permission, not the record of an
                // unconfirmed active input. Even the same writer must use
                // a NEW recovery-only epoch and native neutral ACK before
                // it may acquire a normal action lease on this slot.
                (!held.neutral && !neutral_only) ||
                (held.until_ms>now && (
                    held.owner_id!=owner || held.pid!=*pid ||
                    held.birth!=*birth ||
                    // A recovery-only epoch cannot become an active
                    // writer with the very same transaction identifier.
                    (held.lease_epoch==epoch && held.neutral_only &&
                     !neutral_only)
                )))
        }) {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied,
                "AI writer lease conflict; existing game input preserved"));
        }
        for &(slot,pid,birth) in targets {
            let index=(slot-1) as usize;
            // An expired claim is NEVER evidence of native neutral
            // delivery. Every recovery-only acquisition starts
            // non-neutral, even for an unclaimed slot, until a fresh
            // exact-owner neutral write has actually succeeded.
            // A normal claim can inherit only prior confirmed neutral
            // state; pending active input always requires recovery.
            let neutral=!neutral_only &&
                self.claims[index].as_ref().is_none_or(|held|held.neutral);
            self.claims[index]=Some(AiWriterClaim {
                owner_id:owner.into(),lease_epoch:epoch.into(),pid,birth,
                until_ms:now.saturating_add(AI_WRITER_LEASE_MS),
                neutral,neutral_only,
            });
        }
        Ok(())
    }
    pub(crate) fn require(&self,slot:u32,owner:&str,epoch:&str,
        pid:u32,birth:u64,now:u64)->io::Result<()> {
        if !valid_ai_writer_id(owner)||!valid_ai_writer_id(epoch)||
           !(1..=5).contains(&slot)||now==0||
           !self.claims[(slot-1) as usize].as_ref().is_some_and(|held|
             held.owner_id==owner && held.lease_epoch==epoch &&
             held.pid==pid && held.birth==birth && held.until_ms>now) {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied,
                "AI slot is not owned by this live writer lease"));
        }
        Ok(())
    }
    pub(crate) fn begin_write(&mut self,slot:u32,owner:&str,epoch:&str,
        pid:u32,birth:u64,now:u64,neutral:bool)->io::Result<()> {
        self.require(slot,owner,epoch,pid,birth,now)?;
        let held=self.claims[(slot-1) as usize].as_mut().unwrap();
        if held.neutral_only && !neutral {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied,
                "AI neutral-only recovery lease refuses active game input"));
        }
        // Native I/O may fail AFTER publishing a shared packet. Keep
        // the original lease non-neutral until a NEW neutral ACK.
        held.neutral=false;
        held.until_ms=now.saturating_add(AI_WRITER_LEASE_MS);
        Ok(())
    }
    pub(crate) fn wrote(&mut self,slot:u32,owner:&str,epoch:&str,
        pid:u32,birth:u64,now:u64,neutral:bool)->io::Result<()> {
        self.require(slot,owner,epoch,pid,birth,now)?;
        let held=self.claims[(slot-1) as usize].as_mut().unwrap();
        held.until_ms=now.saturating_add(AI_WRITER_LEASE_MS);
        held.neutral=neutral;
        Ok(())
    }
    pub(crate) fn release(&mut self,targets:&[(u32,u32,u64)],
        owner:&str,epoch:&str,now:u64)->io::Result<()> {
        let slots=targets.iter().map(|(slot,_,_)|*slot).collect::<Vec<_>>();
        validate_ai_lease_slots(&slots,owner)?;
        for &(slot,pid,birth) in targets {
            self.require(slot,owner,epoch,pid,birth,now)?;
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
           self.claims[(slot-1) as usize].as_ref().is_some_and(|c|
               c.until_ms>now || !c.neutral) {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied,
                "An AI slot is held by a different native writer"));
        }
        Ok(())
    }
    // The legacy MC02 native call can fail AFTER publishing a packet.
    // Mark that slot possibly active BEFORE entering native I/O so a
    // concurrent/following 4/5-AI lease cannot adopt it while the input
    // channel's bounded (<=500ms) TTL may still be alive. The retained
    // session actor calls legacy_allowed first and serializes both steps.
    pub(crate) fn legacy_begin_write(&mut self,slot:u32,now:u64) {
        // A failed native send can publish input before it errors.
        // Mark possibly active BEFORE the original channel I/O. Expiry
        // stops virtual polling but must not clear this recovery gate.
        self.legacy_neutral_pending[(slot-1) as usize]=true;
        self.legacy_until_ms[(slot-1) as usize]=now.saturating_add(500);
    }
    pub(crate) fn legacy_wrote(&mut self,slot:u32,now:u64,neutral:bool) {
        self.claims[(slot-1) as usize]=None;
        // Called only AFTER the existing exact-process shared-channel
        // write returns success. This confirms native transport only;
        // Minecraft key-up and Raw Input polling remain unverified.
        self.legacy_neutral_pending[(slot-1) as usize]=!neutral;
        self.legacy_until_ms[(slot-1) as usize]=
            if neutral{0}else{now.saturating_add(500)};
    }
    pub(crate) fn forget_slot(&mut self,slot:u32) {
        if (1..=5).contains(&slot) {
            self.claims[(slot-1) as usize]=None;
            self.legacy_until_ms[(slot-1) as usize]=0;
            self.legacy_neutral_pending[(slot-1) as usize]=false;
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

// An automatic Settings read can return a valid settings snapshot without
// reconciling the result of a timed-out launch, CloseOne, or native input.
// Only the original session Inspect route is allowed to clear the transport
// uncertainty gate, and only when its fresh reply succeeds. This remains
// read-only inspection, not an assertion that an older mutation succeeded.
#[cfg(windows)]
fn can_reconcile_pending_transport(request:&SessionRequest)->bool {
    matches!(request,SessionRequest::Inspect)
}

#[cfg(windows)]
pub fn request(request: SessionRequest) -> io::Result<SessionSnapshot> {
    match &request {
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
    if let SessionRequest::PublishAgentActivity {ref activity}=request {
        if !activity.valid_ingress() {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,
                "Invalid agent activity report"));
        }
    }
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
        ref frames,issued_at_ms,deadline_at_ms,
        ref owner_id,ref lease_epoch,
    } = request {
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
    } = request {
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
    }=request {
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
    }=request {
        validate_ai_lease_slots(slots,owner_id)?;
        if !valid_ai_writer_id(lease_epoch) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,
                "Invalid AI release fencing epoch"));
        }
    }
    let inspecting = matches!(request, SessionRequest::Inspect | SessionRequest::InspectSettings);
    // Both reads remain available while a previous mutation is uncertain;
    // only a complete session Inspect may reconcile that uncertainty.
    let reconciling = can_reconcile_pending_transport(&request);
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
            Ok(_) | Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                // This exact receiver belongs to the PREVIOUS worker,
                // not the current caller. A late native reply or broken
                // sender is drained and joined without replaying it,
                // pretending success or releasing the uncertainty gate.
                let pending = owner.pending.take().unwrap();
                let _ = pending.worker.join();
                owner.uncertain = true;
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
    let outcome = classify_pending_worker_wait(
        owner.pending.as_ref().unwrap().replies
            .recv_timeout(std::time::Duration::from_secs(45)));
    match outcome {
        PendingWorkerWait::Completed(result) => {
            let pending = owner.pending.take().unwrap();
            let _ = pending.worker.join();
            match result {
                Ok(reply) => {
                    if reconciling && reply.is_ok() {
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
        PendingWorkerWait::StillRunning => {
            // The unique original worker may still execute its request.
            // Retain its receiver, buffers, peer pin and thread. Never
            // start a second worker or blindly resend its mutation.
            owner.uncertain = true;
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
            let pending = owner.pending.take().unwrap();
            let _ = pending.worker.join();
            owner.uncertain = true;
            Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "Original session request worker returned no result; outcome uncertain",
            ))
        }
    }
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
        // The old fallback returned five STOPPED profiles on Inspect without
        // checking whether a game survived the controller. Such a false idle
        // receipt could also clear the caller's pending-request uncertainty.
        // Reuse the existing signed registered-game inventory before creating
        // a stopped snapshot or touching global settings.
        let observation = super::native_clients::inspect_native_workspace()?;
        require_absent_controller_idle(observation.processes.len())?;
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
    // A report or Testing correlation cannot cause fresh game/profile
    // construction if the original controller disappeared after the
    // one-shot CLI's existence gate. Fail closed on this TOCTOU race.
    if matches!(request,
        SessionRequest::PublishAgentActivity{..} |
        SessionRequest::TestingReserve{..} |
        SessionRequest::TestingFinalize{..} |
        SessionRequest::TestingInspect{..}) {
        return Err(io::Error::new(io::ErrorKind::NotFound,
            "No retained controller; status or Testing reservation refused"));
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

    #[test]
    fn staged_testing_recovery_cas_rejects_stale_or_finalized_slots() {
        // Pure contract fixtures, NOT authenticated operator approval.
        // No production constructor, native transfer request, or game action.
        fn fixture(slot:u32,old_run:&str,old_owner:&str,
                   new_run:&str,new_owner:&str)->TestingRecoveryAdmission {
            TestingRecoveryAdmission{
                slot,expected_run_id:old_run.into(),
                expected_owner:old_owner.into(),
                new_run_id:new_run.into(),new_owner:new_owner.into(),
                _operator:VerifiedTestingOperatorPresence(()),
                _archive:VerifiedNativeTestingArchive(()),
            }
        }
        let mut book=NativeTestingRunBook::default();
        let old="a".repeat(32);
        let other="b".repeat(32);
        let next="c".repeat(32);
        let old_run="d".repeat(32);
        let stale_run="9".repeat(32);
        let new_run="e".repeat(32);
        assert_eq!(
            book.check_recovery_cas(&fixture(2,&old_run,&old,&new_run,&next)),
            Err("TESTING_TRANSFER_OLD_RUN_UNAVAILABLE"));
        let owned=book.reserve(2,&old,&old_run,None).unwrap();
        let proposed=fixture(2,&old_run,&old,&new_run,&next);
        assert_eq!(book.check_recovery_cas(&proposed),Ok(()));
        assert_eq!(book.inspect(2,&other).unwrap().run_id.as_deref(),
            Some(old_run.as_str()),"Preflight cannot change ownership");
        for candidate in [
            fixture(0,&old_run,&old,&new_run,&next),
            fixture(6,&old_run,&old,&new_run,&next),
            fixture(2,&old_run,&old,&old_run,&next),
            fixture(2,&old_run,&old,&new_run,&old),
            fixture(2,&old_run,&old,"INVALID",&next),
        ] {
            assert_eq!(book.check_recovery_cas(&candidate),
                Err("TESTING_TRANSFER_INVALID_CONTRACT"));
        }
        for candidate in [
            fixture(2,&stale_run,&old,&new_run,&next),
            fixture(2,&old_run,&other,&new_run,&next),
        ] {
            assert_eq!(book.check_recovery_cas(&candidate),
                Err("TESTING_TRANSFER_STALE_EXPECTED_RUN"));
        }
        assert_eq!(book.inspect(2,&other).unwrap().phase,"RESERVED");
        assert_eq!(book.reserve(2,&old,&old_run,None).unwrap(),owned);
        book.finalize(2,&old,&old_run,"BLOCKED",&"f".repeat(64)).unwrap();
        assert_eq!(
            book.check_recovery_cas(&fixture(2,&old_run,&old,&new_run,&next)),
            Err("TESTING_TRANSFER_FINAL_OR_INCONSISTENT"));
        assert_eq!(book.inspect(2,&other).unwrap().phase,"FINAL");
        assert!(!book.inspect(2,&other).unwrap().transfer_allowed);
    }

    #[test]
    fn testing_inspection_is_read_only_and_never_authorizes_orphan_takeover() {
        // No controller/game/OS access. Token comparison is not PID proof.
        let mut book=NativeTestingRunBook::default();
        let owner="a".repeat(32);
        let other="b".repeat(32);
        let run="c".repeat(32);
        let digest="1".repeat(64);
        let empty=book.inspect(2,&other).unwrap();
        assert_eq!(empty.phase,"NO_RECORD");
        assert!(empty.run_id.is_none());
        assert!(!empty.transfer_allowed);
        assert_eq!(empty.recovery_action,"CHECK_ARCHIVE_AND_CONTROLLER_EPOCH");
        assert_eq!(empty.owner_process_proof,"NOT_BOUND_TO_PROCESS_HANDLE");
        assert_eq!(empty.operator_approval,"NOT_AVAILABLE_IN_PROTOCOL");
        assert_eq!(book.inspect(0,&owner),Err("TESTING_INSPECTION_INVALID"));
        assert_eq!(book.inspect(2,"INVALID"),Err("TESTING_INSPECTION_INVALID"));

        let reserved=book.reserve(2,&owner,&run,None).unwrap();
        let owner_view=book.inspect(2,&owner).unwrap();
        let other_view=book.inspect(2,&other).unwrap();
        assert_eq!(owner_view.phase,"RESERVED");
        assert!(owner_view.owner_token_match);
        assert!(!other_view.owner_token_match);
        assert_eq!(other_view.run_id.as_deref(),Some(run.as_str()));
        assert_eq!(other_view.reported_status,None);
        assert_eq!(other_view.report_digest,None);
        assert_eq!(other_view.recovery_action,
            "REQUIRE_VERIFIED_OWNER_DEATH_AND_OPERATOR_APPROVAL");
        assert!(!owner_view.transfer_allowed);
        assert!(!other_view.transfer_allowed);
        assert_eq!(book.reserve(2,&owner,&run,None).unwrap(),reserved,
            "Inspection must not mutate a reservation");
        assert!(!book.reserve(2,&other,&"d".repeat(32),None)
            .unwrap().owner_match);

        book.finalize(2,&owner,&run,"FAIL",&digest).unwrap();
        let final_view=book.inspect(2,&other).unwrap();
        assert_eq!(final_view.phase,"FINAL");
        assert_eq!(final_view.reported_status.as_deref(),Some("FAIL"));
        assert_eq!(final_view.report_digest.as_deref(),Some(digest.as_str()));
        assert_eq!(final_view.recovery_action,
            "READ_FINAL_REPORT_NO_OWNER_TRANSFER");
        assert!(!final_view.transfer_allowed);
        assert_eq!(book.finalize(2,&other,&run,"FAIL",&digest),
            Err("TESTING_RUN_OWNER_MISMATCH"));
    }

    #[test]
    fn testing_run_book_deduplicates_and_fences_terminal_outcome() {
        // Pure Rust source fixture; no Windows IPC or gameplay.
        let mut book=NativeTestingRunBook::default();
        let owner="a".repeat(32);
        let rival="b".repeat(32);
        let first="c".repeat(32);
        let other="d".repeat(32);
        let digest="1".repeat(64);
        let wrong="2".repeat(64);
        let reserved=book.reserve(2,&owner,&first,None).unwrap();
        assert_eq!(reserved.phase,"RESERVED");
        assert!(reserved.owner_match);
        assert_eq!(reserved.provenance,
            "NATIVE_RUN_CORRELATION_ONLY_NOT_GAMEPLAY_PROOF");
        assert_eq!(book.reserve(2,&owner,&first,None).unwrap(),reserved);
        let rival_view=book.reserve(2,&rival,&other,None).unwrap();
        assert_eq!(rival_view.run_id,first);
        assert!(!rival_view.owner_match);
        assert_eq!(book.finalize(2,&rival,&first,"FAIL",&digest),
            Err("TESTING_RUN_OWNER_MISMATCH"));
        let complete=book.finalize(
            2,&owner,&first,"PROBE_REPORTED_PASS",&digest).unwrap();
        assert_eq!(complete.phase,"FINAL");
        assert_eq!(complete.report_digest.as_deref(),Some(digest.as_str()));
        assert_eq!(book.finalize(2,&owner,&first,"FAIL",&wrong),
            Err("TESTING_FINAL_RESULT_IMMUTABLE"));
        assert_eq!(book.finalize(
            2,&owner,&first,"PROBE_REPORTED_PASS",&digest).unwrap(),complete);
        assert!(!book.reserve(2,&owner,&other,None).unwrap().owner_match);
        assert!(!book.reserve(2,&rival,&other,Some(&first)).unwrap().owner_match);
        let restarted=book.reserve(2,&owner,&other,Some(&first)).unwrap();
        assert!(restarted.owner_match);
        assert_eq!(restarted.phase,"RESERVED");
        assert_eq!(restarted.run_id,other);
        assert!(restarted.report_digest.is_none());
    }

    #[test]
    fn testing_run_book_refuses_invalid_claims_and_active_restart() {
        let mut book=NativeTestingRunBook::default();
        let a="a".repeat(32);
        let b="b".repeat(32);
        let sha="1".repeat(64);
        assert_eq!(book.reserve(0,&a,&b,None),Err("TESTING_RUN_INVALID_ID"));
        assert_eq!(book.reserve(2,&a,"UPPERCASE",None),
            Err("TESTING_RUN_INVALID_ID"));
        assert_eq!(book.reserve(2,&a,&b,Some(&b)),
            Err("TESTING_PREVIOUS_RUN_UNAVAILABLE"));
        assert!(book.reserve(2,&a,&b,None).unwrap().owner_match);
        assert!(!book.reserve(2,&a,&a,Some(&b)).unwrap().owner_match,
            "Even owner cannot restart while a run is reserved");
        assert_eq!(book.finalize(2,&a,&b,"PASS",&sha),
            Err("TESTING_FINAL_REPORT_INVALID"));
        assert_eq!(book.finalize(2,&a,&b,"BLOCKED",&sha)
            .unwrap().phase,"FINAL");
    }

    #[test]
    fn agent_activity_report_is_bounded_and_cannot_claim_desktop_control() {
        // Pure metadata only: no native controller, game or Tauri execution.
        let mut report=AgentActivityReport{
            schema:1,source:"CURRENT_MCP_PROCESS_MEMORY_ONLY".into(),
            snapshot_at:"2026-11-02T12:00:00.000Z".into(),
            display_hint:"ATTENTION".into(),
            desktop_control:"NOT_OBSERVED_NO_AUTHENTICATED_DESKTOP_PROVIDER".into(),
            minecraft_input_effect:"NOT_VERIFIED".into(),
            map_gameplay_acceptance:"NOT_TESTED".into(),stale:false,
            items:vec![AgentActivityItem{
                kind:"AI_INPUT".into(),slot:2,
                status:"NATIVE_RELEASE_UNCONFIRMED".into(),
                importance:"ATTENTION".into(),
                evidence:"LOCAL_MCP_RECOVERY_HOLD".into(),
                reported_outcome:None,
                probe_counts:None,
            }],
        };
        assert!(report.valid_ingress());
        report.desktop_control="AI_CONTROLLING_PC".into();
        assert!(!report.valid_ingress());
        report.desktop_control="NOT_OBSERVED_NO_AUTHENTICATED_DESKTOP_PROVIDER".into();
        report.items[0].importance="INFORMATION".into();
        assert!(!report.valid_ingress(),
            "Unconfirmed input release cannot be reduced to normal activity");
        report.items[0].importance="ATTENTION".into();
        report.map_gameplay_acceptance="PASS".into();
        assert!(!report.valid_ingress());
        report.map_gameplay_acceptance="NOT_TESTED".into();
        report.items[0].slot=6;
        assert!(!report.valid_ingress());
        report.items[0].slot=2;
        report.stale=true;
        assert!(!report.valid_ingress(),
            "Only the controller may add a stale notice after monotonic expiry");
        report.stale=false;
        report.display_hint="ACTIVITY".into();
        report.items[0]=AgentActivityItem{
            kind:"SKILLS_PROBE".into(),slot:2,
            status:"PROBE_REPORT_AVAILABLE".into(),
            importance:"INFORMATION".into(),
            evidence:"ORIGINAL_PROBE_LOG_UNVERIFIED_ORIGIN".into(),
            reported_outcome:Some("PROBE_REPORTED_PASS".into()),
            probe_counts:Some(AgentProbeCounts{
                expected:2,pass:2,fail:0,info:0,look:0,
            }),
        };
        assert!(report.valid_ingress());
        report.items[0].probe_counts.as_mut().unwrap().pass=3;
        assert!(!report.valid_ingress(),"Counts cannot exceed the reported total");
        report.items[0].probe_counts.as_mut().unwrap().pass=2;
        report.items[0].reported_outcome=Some("INCOMPLETE".into());
        report.items[0].importance="ATTENTION".into();
        report.display_hint="ATTENTION".into();
        assert!(!report.valid_ingress(),
            "Incomplete results may not carry completed Probe counts");
    }

    #[cfg(windows)]
    #[test]
    fn pending_timeout_reconciliation_requires_fresh_session_inspect_not_settings() {
        // Pure admission policy fixture: no controller, pipe, Windows
        // game window or source test executor is started by this code.
        assert!(can_reconcile_pending_transport(&SessionRequest::Inspect));
        assert!(!can_reconcile_pending_transport(&SessionRequest::InspectSettings));
        assert!(!can_reconcile_pending_transport(&SessionRequest::InspectStorage));
        assert!(!can_reconcile_pending_transport(&SessionRequest::CloseOne{slot:2}));
        assert!(!can_reconcile_pending_transport(&SessionRequest::CloseAll));
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
    fn temporal_epoch_fences_relaunched_child_before_native_acquire() {
        // Pure contract fixture: 100-ns Windows FILETIME witness. Does not
        // create any game process, IPC server, or input channel.
        let at=134_000_000_000_000_000u64;
        let token=|issued:u64|format!("{issued:016x}{}e1","a".repeat(14));
        let old=token(at);
        let fresh=token(at+2_000_000);
        let four=vec![
            (2,202,at-10_000_000),(3,203,at-10_000_000),
            (4,204,at-10_000_000),(5,205,at-10_000_000),
        ];
        let mut five=vec![(1,201,at-10_000_000)];
        five.extend(four.iter().copied());
        assert!(validate_ai_acquire_epoch(&old,at+1_000_000,&four).is_ok());
        assert!(validate_ai_acquire_epoch(&old,at+1_000_000,&five).is_ok());

        // One slot replaced after the old MCP request was minted.
        // Atomic multi-slot acquisition must fail for BOTH 4-AI and 5-AI.
        let replaced_four=vec![
            (2,902,at+1_000_000),(3,203,at-10_000_000),
            (4,204,at-10_000_000),(5,205,at-10_000_000),
        ];
        let mut replaced_five=vec![(1,201,at-10_000_000)];
        replaced_five.extend(replaced_four.iter().copied());
        assert!(validate_ai_acquire_epoch(&old,at+2_300_000,
            &replaced_four).is_err());
        assert!(validate_ai_acquire_epoch(&old,at+2_300_000,
            &replaced_five).is_err());
        assert!(validate_ai_acquire_epoch(&fresh,at+2_300_000,
            &replaced_four).is_ok());
        assert!(validate_ai_acquire_epoch(&fresh,at+2_300_000,
            &replaced_five).is_ok());

        // The original retained lease book still fences stale input and
        // release requests by exact owner, epoch, pid and process birth.
        let owner="b".repeat(32);
        let mut book=AiLeaseBook::default();
        book.acquire(&four,&owner,&old,1000,false).unwrap();
        book.forget_slot(2);
        assert!(book.require(2,&owner,&old,902,at+1_000_000,1001).is_err());
        book.acquire(&replaced_four,&owner,&fresh,1002,false).unwrap();
        assert!(book.require(2,&owner,&old,902,at+1_000_000,1003).is_err());
        assert!(book.require(2,&owner,&fresh,902,at+1_000_000,1003).is_ok());
        assert!(book.release(&[(2,902,at+1_000_000)],&owner,&old,1004).is_err());

        // Disabled, malformed, stale and far-future epochs fail closed.
        for invalid in [
            "1".repeat(32),
            "A".repeat(32),
            format!("{:016x}{}e2",at,"a".repeat(14)),
            token(at-120_000_000),
            token(at+20_000_000),
        ] {
            assert!(validate_ai_acquire_epoch(&invalid,at+1_000_000,
                &four).is_err());
        }
        assert!(validate_ai_acquire_epoch(&fresh,at+2_300_000,&[]).is_err());
    }

    #[cfg(feature="mc02-native-input")]
    #[test]
    fn fenced_writer_conflict_is_atomic_and_process_bound() {
        // Pure Rust source fixture; no Windows, IPC, or Minecraft execution.
        let a="a".repeat(32);
        let b="b".repeat(32);
        let e1="1".repeat(32);
        let e2="2".repeat(32);
        let four=vec![(2,102,1002),(3,103,1003),(4,104,1004),(5,105,1005)];
        let mut five=vec![(1,101,1001)];
        five.extend(four.iter().copied());
        let mut book=AiLeaseBook::default();
        assert!(book.acquire(&four,&a,&e1,1000,false).is_ok());
        for &(slot,pid,birth) in &four {
            assert!(book.require(slot,&a,&e1,pid,birth,1001).is_ok());
            assert!(book.require(slot,&a,&e2,pid,birth,1001).is_err());
            assert!(book.require(slot,&b,&e1,pid,birth,1001).is_err());
            assert!(book.require(slot,&a,&e1,pid,birth+1,1001).is_err());
        }
        // Another writer cannot claim idle MC01 while peers conflict.
        assert!(book.acquire(&five,&b,&e2,1002,false).is_err());
        assert!(book.require(1,&b,&e2,101,1001,1002).is_err());
        book.begin_write(2,&a,&e1,102,1002,1010,false).unwrap();
        assert!(book.release(&four,&a,&e1,1011).is_err());
        assert!(book.acquire(&[(2,102,1002)],&a,&e2,1012,false).is_err());
        // Emergency neutral takeover fences the old same-writer epoch.
        assert!(book.acquire(&four,&a,&e2,1020,true).is_ok());
        assert!(book.require(2,&a,&e1,102,1002,1021).is_err());
        assert!(book.release(&four,&a,&e2,1022).is_err());
        assert!(book.begin_write(2,&a,&e2,102,1002,1023,false).is_err());
        book.begin_write(2,&a,&e2,102,1002,1024,true).unwrap();
        book.wrote(2,&a,&e2,102,1002,1025,true).unwrap();
        // Recovery-only never assumes the other three are neutral just
        // because they were neutral on a previous epoch. Each gets an
        // explicit native neutral ACK before the group can be released.
        assert!(book.release(&four,&a,&e2,1025).is_err());
        for &(slot,pid,birth) in &four[1..] {
            book.begin_write(slot,&a,&e2,pid,birth,1025,true).unwrap();
            book.wrote(slot,&a,&e2,pid,birth,1025,true).unwrap();
        }
        assert!(book.release(&four,&a,&e2,1026).is_ok());
        assert!(book.acquire(&five,&b,&e1,1030,false).is_ok());
        assert!(book.require(1,&b,&e1,101,1001,1031).is_ok());
        // Process exit invalidates an original claim. A freshly claimed
        // replacement uses a DIFFERENT epoch, not a reused owner ID.
        assert!(book.acquire(&[(2,999,1002)],&b,&e2,1032,false).is_err());
        book.forget_slot(2);
        assert!(book.acquire(&[(2,999,9002)],&b,&e2,1033,false).is_ok());
        assert!(book.require(2,&b,&e1,102,1002,1034).is_err());
        assert!(book.require(2,&b,&e2,999,9002,1034).is_ok());
        assert!(book.release(&[(2,999,9002)],&b,&e1,1035).is_err());
        assert!(book.begin_write(2,&b,&e1,999,9002,1035,true).is_err());
        assert!(book.require(2,&b,&e2,999,9002,
            1033+AI_WRITER_LEASE_MS).is_err());
        assert!(book.acquire(&[(2,999,9002)],&a,&e1,
            1034+AI_WRITER_LEASE_MS,false).is_ok());
    }


    #[cfg(feature="mc02-native-input")]
    #[test]
    fn expired_active_writer_requires_exact_native_neutral_recovery_in_four_five_modes() {
        // Pure retained native lease-book fixture; no Windows game process,
        // input mapping, client CLI or Minecraft is created or executed.
        let first="a".repeat(32);
        let second="b".repeat(32);
        let e1="1".repeat(32);
        let e2="2".repeat(32);
        let e3="3".repeat(32);
        let four=vec![(2,202,2202),(3,203,2203),(4,204,2204),(5,205,2205)];
        let mut five=vec![(1,201,2201)];
        five.extend(four.iter().copied());
        for cohort in [&four,&five] {
            let mut book=AiLeaseBook::default();
            book.acquire(cohort,&first,&e1,1000,false).unwrap();
            // Exercise the REAL legacy MC02 collision in both cohort
            // shapes, not merely the second element of the slot list.
            let &(slot,pid,birth)=cohort.iter()
                .find(|(slot,_,_)|*slot==2).unwrap();
            book.begin_write(slot,&first,&e1,pid,birth,1010,false).unwrap();
            // The first input may have reached the game. There is no
            // matching native neutral ACK, not even after 10s expiry.
            let expired=1010+AI_WRITER_LEASE_MS+2;
            assert!(book.require(slot,&first,&e1,pid,birth,expired).is_err());
            assert!(book.acquire(cohort,&second,&e2,expired,false).is_err(),
                "a single unconfirmed slot denies the whole 4/5 cohort");
            assert!(book.acquire(&[(slot,pid,birth)],&first,&e2,expired,false).is_err(),
                "same MCP writer cannot convert expiry into neutral proof");
            assert!(book.acquire(&[(slot,pid,birth)],&second,&e2,expired,false).is_err());
            assert!(book.legacy_allowed(slot,expired).is_err(),
                "expired active MC02 claim may not unlock legacy input");
            let healthy=cohort.iter().copied()
                .filter(|(s,_,_)|*s!=slot).collect::<Vec<_>>();
            assert!(book.acquire(&healthy,&second,&e2,expired,false).is_ok(),
                "other original clients may be claimed independently");
            assert!(book.release(&healthy,&second,&e2,expired+1).is_ok(),
                "only prior neutral-ACKed normal claims may release");

            book.acquire(&[(slot,pid,birth)],&second,&e2,expired,true).unwrap();
            assert!(book.require(slot,&first,&e1,pid,birth,expired+1).is_err(),
                "old epoch is fenced after neutral-only takeover");
            assert!(book.release(&[(slot,pid,birth)],&second,&e2,expired+1).is_err(),
                "expiry and recovery acquire are not a neutral ACK");
            assert!(book.begin_write(slot,&second,&e2,pid,birth,expired+1,false).is_err(),
                "recovery-only writer may never send active input");
            book.begin_write(slot,&second,&e2,pid,birth,expired+2,true).unwrap();
            assert!(book.release(&[(slot,pid,birth)],&second,&e2,expired+3).is_err(),
                "a possibly published neutral write still needs its ACK");
            book.wrote(slot,&second,&e2,pid,birth,expired+3,true).unwrap();
            book.release(&[(slot,pid,birth)],&second,&e2,expired+4).unwrap();
            assert!(book.acquire(&[(slot,pid,birth)],&second,&e3,expired+5,false).is_ok(),
                "new action may start only after verified native neutral and release");
        }
    }

    #[cfg(feature="mc02-native-input")]
    #[test]
    fn recovery_only_new_claim_never_starts_preconfirmed_neutral() {
        let writer="b".repeat(32);
        let epoch="2".repeat(32);
        let target=[(3,303,3303)];
        let mut book=AiLeaseBook::default();
        book.acquire(&target,&writer,&epoch,1000,true).unwrap();
        assert!(book.release(&target,&writer,&epoch,1001).is_err(),
            "a first-time recovery must obtain its own native neutral ACK");
        book.begin_write(3,&writer,&epoch,303,3303,1002,true).unwrap();
        assert!(book.release(&target,&writer,&epoch,1003).is_err());
        book.wrote(3,&writer,&epoch,303,3303,1003,true).unwrap();
        book.release(&target,&writer,&epoch,1004).unwrap();

        // Only a confirmed original process exit can forget the stale
        // claim without writing to a possibly replaced game process.
        let mut exited=AiLeaseBook::default();
        exited.acquire(&target,&writer,&epoch,2000,false).unwrap();
        exited.begin_write(3,&writer,&epoch,303,3303,2001,false).unwrap();
        exited.forget_slot(3);
        assert!(exited.acquire(&[(3,909,9909)],&writer,
            &"3".repeat(32),2002,false).is_ok());
        assert!(exited.require(3,&writer,&epoch,909,9909,2003).is_err());
    }

    #[cfg(feature="mc02-native-input")]
    #[test]
    fn interrupted_native_write_remains_non_neutral_and_isolated() {
        let a="a".repeat(32);
        let e1="1".repeat(32);
        let e2="2".repeat(32);
        let targets=vec![(2,202,2202),(3,203,2203),(4,204,2204),(5,205,2205)];
        let mut book=AiLeaseBook::default();
        book.acquire(&targets,&a,&e1,1000,false).unwrap();
        book.begin_write(3,&a,&e1,203,2203,1010,false).unwrap();
        assert!(book.release(&targets,&a,&e1,1011).is_err());
        let peers=vec![(2,202,2202),(4,204,2204),(5,205,2205)];
        book.release(&peers,&a,&e1,1020).unwrap();
        assert!(book.require(3,&a,&e1,203,2203,1021).is_ok());
        book.begin_write(3,&a,&e1,203,2203,1030,true).unwrap();
        assert!(book.release(&[(3,203,2203)],&a,&e1,1031).is_err());
        book.wrote(3,&a,&e1,203,2203,1032,true).unwrap();
        book.release(&[(3,203,2203)],&a,&e1,1033).unwrap();
        book.acquire(&[(3,203,2203)],&a,&e2,2000,true).unwrap();
        book.begin_write(3,&a,&e2,203,2203,2001,true).unwrap();
        assert!(book.require(3,&a,&e2,203,2203,
            2001+AI_WRITER_LEASE_MS).is_err());
        assert!(book.acquire(&[(3,203,2203)],&"b".repeat(32),&e1,
            2002+AI_WRITER_LEASE_MS,true).is_ok());
    }

    #[cfg(feature="mc02-native-input")]
    #[test]
    fn fencing_protocol_rejects_invalid_ids_and_legacy_overlap() {
        let a="a".repeat(32);
        let e1="1".repeat(32);
        let e2="2".repeat(32);
        let mut book=AiLeaseBook::default();
        assert!(validate_ai_lease_slots(&[2,3,4,5],&a).is_ok());
        for invalid in ["short".to_owned(),"A".repeat(32),"x".repeat(32)] {
            assert!(validate_ai_lease_slots(&[2],&invalid).is_err());
            assert!(book.acquire(&[(2,202,2002)],&a,&invalid,1000,false).is_err());
        }
        for invalid_slots in [vec![],vec![2,2],vec![3,2],vec![0],vec![6],
            vec![1,2,3,4,5,6]] {
            assert!(validate_ai_lease_slots(&invalid_slots,&a).is_err());
        }
        book.legacy_wrote(2,1000,false);
        assert!(book.acquire(&[(2,202,2002)],&a,&e1,1300,false).is_err());
        assert!(book.acquire(&[(2,202,2002)],&a,&e1,1501,false).is_err(),
            "expired legacy pulse without native neutral ACK stays fenced");
        // Simulate the original controller's separately acknowledged
        // legacy neutral write. Only then may a modern writer acquire.
        book.legacy_wrote(2,1502,true);
        assert!(book.acquire(&[(2,202,2002)],&a,&e1,1503,false).is_ok());
        assert!(book.legacy_allowed(2,1504).is_err());
        book.begin_write(2,&a,&e1,202,2002,1505,false).unwrap();
        assert!(book.acquire(&[(2,202,2002)],&a,&e2,1506,true).is_ok());
        assert!(book.require(2,&a,&e1,202,2002,1507).is_err());
        book.begin_write(2,&a,&e2,202,2002,1508,true).unwrap();
        book.wrote(2,&a,&e2,202,2002,1509,true).unwrap();
        book.release(&[(2,202,2002)],&a,&e2,1510).unwrap();
        book.legacy_wrote(2,1511,true);
        assert!(book.acquire(&[(2,202,2002)],&a,&e1,1512,false).is_ok());
    }


    #[cfg(feature="mc02-native-input")]
    #[test]
    fn interrupted_legacy_mc02_send_fences_multi_writer_until_neutral_ack_or_reap() {
        // Pure lease-book simulation: legacy MC02 packet may already be
        // published when the native API reports an error. No game process,
        // Windows channel, MCP, or test runner is started by this fixture.
        let a="a".repeat(32);
        let e="1".repeat(32);
        let four=vec![(2,102,1002),(3,103,1003),(4,104,1004),(5,105,1005)];
        let five=vec![(1,101,1001),(2,102,1002),
                      (3,103,1003),(4,104,1004),(5,105,1005)];
        let mut book=AiLeaseBook::default();
        // legacy_allowed() admitted the exact original hybrid MC02 slot.
        book.legacy_allowed(2,1000).unwrap();
        book.legacy_begin_write(2,1000);
        // No legacy_wrote(): simulate a native failure after possible publish.
        // The all-slot claim is atomic; it never captures healthy peers.
        assert!(book.acquire(&four,&a,&e,1001,false).is_err());
        assert!(book.acquire(&five,&a,&e,1002,false).is_err());
        assert!(book.require(3,&a,&e,103,1003,1002).is_err());
        // Other selected slots can still be acquired/released separately.
        book.acquire(&[(3,103,1003)],&a,&e,1003,false).unwrap();
        book.release(&[(3,103,1003)],&a,&e,1004).unwrap();
        assert!(book.acquire(&four,&a,&e,1499,false).is_err());
        assert!(book.acquire(&four,&a,&e,1500,false).is_err(),
            "500ms packet TTL is not a native neutral acknowledgement");
        assert!(book.acquire(&five,&a,&e,10_000,false).is_err(),
            "a late takeover still requires native neutral ACK");
        book.legacy_allowed(2,10_001).unwrap();
        book.legacy_begin_write(2,10_001);
        book.legacy_wrote(2,10_002,true);
        assert!(book.acquire(&four,&a,&e,10_003,false).is_ok());

        // Explicit successful neutral input is the ONLY early clearance.
        let mut recovered=AiLeaseBook::default();
        recovered.legacy_allowed(2,2000).unwrap();
        recovered.legacy_begin_write(2,2000);
        assert!(recovered.acquire(&five,&a,&e,2001,false).is_err());
        recovered.legacy_wrote(2,2002,true);
        assert!(recovered.acquire(&five,&a,&e,2003,false).is_ok());

        // A failed neutral write must retain the potential activity fence.
        let mut uncertain=AiLeaseBook::default();
        uncertain.legacy_begin_write(2,3000);
        assert!(uncertain.acquire(&four,&a,&e,3001,false).is_err());
        uncertain.forget_slot(2); // Only native confirmed process reap.
        assert!(uncertain.acquire(&four,&a,&e,3002,false).is_ok());
    }


    #[cfg(feature="mc02-native-input")]
    #[test]
    fn legacy_packet_ttl_never_unlocks_four_or_five_ai_without_neutral_ack() {
        // Source-only ledger fixture: the native consumer may stop polling
        // at 500ms but there is no ACK that it received key-up. One legacy
        // MC02 channel is shared with each selected group of AI slots.
        let owner="a".repeat(32);
        let epoch="1".repeat(32);
        let four=vec![(2,202,2202),(3,203,2203),(4,204,2204),(5,205,2205)];
        let mut five=vec![(1,201,2201)];
        five.extend(four.iter().copied());
        for cohort in [&four,&five] {
            let mut book=AiLeaseBook::default();
            book.legacy_allowed(2,1000).unwrap();
            book.legacy_begin_write(2,1000);
            // A packet may have been published, then native I/O failed;
            // legacy_wrote is intentionally NOT called after that error.
            assert!(book.acquire(cohort,&owner,&epoch,1500,false).is_err());
            assert!(book.acquire(cohort,&owner,&epoch,15_000,false).is_err());
            assert!(book.acquire(&[(2,202,2202)],&owner,&epoch,
                15_001,true).is_err(),
                "recovery-only multi-AI cannot bypass unresolved legacy input");
            // Independently healthy original clients retain their lease
            // admission even when one legacy channel requires recovery.
            let peers=cohort.iter().copied()
                .filter(|(slot,_,_)|*slot!=2).collect::<Vec<_>>();
            book.acquire(&peers,&owner,&epoch,15_002,false).unwrap();
            book.release(&peers,&owner,&epoch,15_003).unwrap();

            // A legacy neutral attempt that fails after possible publish
            // must keep the unresolved fence, including after another TTL.
            book.legacy_begin_write(2,15_004);
            assert!(book.acquire(cohort,&owner,&epoch,16_000,false).is_err());
            book.legacy_wrote(2,16_001,true); // original channel ACK
            book.acquire(cohort,&owner,&epoch,16_002,false).unwrap();
            book.release(cohort,&owner,&epoch,16_003).unwrap();

            // Confirmed process reap clears only this slot and does not
            // admit a stale original process identity into a replacement.
            let mut exited=AiLeaseBook::default();
            exited.legacy_begin_write(2,1000);
            exited.forget_slot(2);
            let replacement=cohort.iter().copied().map(|(slot,pid,birth)|
                if slot==2{(slot,902,birth+1000)}else{(slot,pid,birth)}
            ).collect::<Vec<_>>();
            exited.acquire(&replacement,&owner,&epoch,1002,false).unwrap();
        }
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
