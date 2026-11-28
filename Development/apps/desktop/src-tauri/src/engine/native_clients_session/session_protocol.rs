//! Session IPC request/reply schema and receipt predicates; no transport or controller state.
use crate::engine::{manager_settings, process_identity::OwnedProcess};
use serde::{Deserialize, Serialize};
use std::io;
use super::agent_activity::AgentActivityReport;
use super::controller_testing_runs::{TestingRunInspection, TestingRunView};
#[cfg(feature="mc02-native-input")]
use super::controller_ai_input_leases::{AgentControlMode, AiInputFrame, Mc02BatchOrigin};

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
    pub selective_inventory: Option<SelectiveInventory>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selective_review: Option<SelectiveReviewReceipt>,
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
    pub storage_preferences: Option<manager_settings::StoragePreferences>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_audio: Option<manager_settings::BackgroundAudio>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audio_issue: Option<String>,
    #[serde(default = "default_graphics")]
    pub graphics_presets: [manager_settings::GraphicsPreset; 5],
    pub active_fps: u32,
    pub background_fps: u32,
    pub render_distance_chunks: u32,
    pub can_apply_preset: bool,
    pub changed_profiles: Option<u32>,
}
fn default_graphics() -> [manager_settings::GraphicsPreset; 5] { [manager_settings::GraphicsPreset::Low; 5] }
impl SettingsReport {
    pub fn policy(can_apply_preset: bool, changed_profiles: Option<u32>) -> io::Result<Self> {
        // Configured ceilings of the reviewed payload, never measured frame rates.
        let (limits, audio, graphics, storage_preferences)=manager_settings::load_report_preferences()?;
        Ok(Self { storage_preferences, background_audio:Some(audio), audio_issue:None, graphics_presets: graphics, active_fps: limits.active_fps, background_fps: limits.background_fps, render_distance_chunks: 12,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub download_cache: Option<CacheStorage>,
    // Read-only on-disk pack directory census; NOT world activation or resident RAM.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_packs: Option<ResourcePackStorage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub game_settings: Option<GameSettingsStorage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_clear: Option<FullClearStorage>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum SelectiveCategory { Worlds, ResourcePacks, BehaviorPacks }

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SelectiveInventoryEntry {
    pub root: String,
    pub name: String,
    pub identity: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SelectiveInventory {
    pub slot: u32,
    pub category: SelectiveCategory,
    pub items: Vec<SelectiveInventoryEntry>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SelectiveSelection {
    pub category: SelectiveCategory,
    pub root: String,
    pub name: String,
    pub identity: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SelectiveReviewReceipt {
    pub slot: u32,
    pub bytes: u64,
    pub files: u32,
    pub items: Vec<SelectiveSelection>,
    pub warnings: Vec<String>,
    pub confirmation_token: Option<String>,
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
    #[cfg(windows)]
    pub(super) fn is_agent_target_receipt(&self,slot:u32)->bool {
        self.schema==1 && self.profiles.len()==1 && self.profiles[0].slot==slot &&
        self.storage.is_none() && self.selective_inventory.is_none() &&
        self.selective_review.is_none() && self.settings.is_none() &&
        self.agent_activity.is_none() && self.testing_run.is_none() && self.testing_inspection.is_none()
    }

    // Only a complete ordinary Inspect receipt may reconcile a previous
    // uncertain transport. In particular, an exact-slot presentation
    // receipt or input acknowledgement cannot clear that gate.
    #[cfg(windows)]
    pub(super) fn is_full_inspect_receipt(&self)->bool {
        self.schema==1 && self.profiles.len()==5 &&
        self.profiles.iter().enumerate().all(|(index,profile)|profile.slot==index as u32+1) &&
        self.storage.is_none() && self.selective_inventory.is_none() &&
        self.selective_review.is_none() && self.settings.is_none() &&
        self.testing_run.is_none() && self.testing_inspection.is_none()
    }

    // Private receipts start without a roster. The caller attaches only
    // the operation-specific result; never fabricate STOPPED entries.
    pub(crate) fn empty_receipt() -> Self {
        Self {
            schema:1, profiles:Vec::new(), storage:None,
            selective_inventory:None, selective_review:None, settings:None,
            agent_activity:None, testing_run:None, testing_inspection:None,
        }
    }

    #[cfg(feature="mc02-native-input")]
    pub(super) fn is_input_receipt(&self)->bool {
        self.schema==1 && self.profiles.is_empty() && self.storage.is_none() &&
        self.selective_inventory.is_none() && self.selective_review.is_none() &&
        self.settings.is_none() && self.agent_activity.is_none() &&
        self.testing_run.is_none() && self.testing_inspection.is_none()
    }

    pub fn stopped() -> Self {
        Self {
            schema: 1,
            storage: None,
            selective_inventory: None,
            selective_review: None,
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
    // Private exact-slot observation for the same-image agent CLI.
    InspectAgentTarget { slot: u32 },
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
    ApplyGraphicsPreset { slot: u32, preset: manager_settings::GraphicsPreset },
    SavePerformance { #[serde(rename="activeFps")] active_fps:u32, #[serde(rename="backgroundFps")] background_fps:u32 },
    SaveAudio {mode:manager_settings::BackgroundAudio},
    SaveStoragePreferences { preferences: manager_settings::StoragePreferences },
    InspectStorage,
    InspectProfileStorage { slot: u32 },
    InspectSelectiveInventory { slot: u32, category: SelectiveCategory, #[serde(default, rename="selectedProfile", skip_serializing_if="Option::is_none")] selected_profile: Option<bool> },
    ReviewSelectiveInventory { slot: u32, selections: Vec<SelectiveSelection>, #[serde(default, rename="selectedProfile", skip_serializing_if="Option::is_none")] selected_profile: Option<bool> },
    ConfirmSelectedWorlds { slot: u32, selections: Vec<SelectiveSelection>, #[serde(rename="confirmationToken")] confirmation_token: String },
    ConfirmSelectedPacks { slot: u32, selections: Vec<SelectiveSelection>, #[serde(rename="confirmationToken")] confirmation_token: String },
    OpenProfileFolder { slot: u32, #[serde(default, rename="selectedProfile", skip_serializing_if="Option::is_none")] selected_profile: Option<bool> },
    ClearProfileCache { slot: u32, #[serde(default, rename="selectedStorage", skip_serializing_if="Option::is_none")] selected_storage: Option<bool> },
    ClearProfileDownloadCache { slot: u32 },
    ResetGameSettings { slot: u32, #[serde(default, rename="selectedStorage", skip_serializing_if="Option::is_none")] selected_storage: Option<bool> },
    RestoreGameSettings { slot: u32, #[serde(default, rename="selectedStorage", skip_serializing_if="Option::is_none")] selected_storage: Option<bool> },
    DisableImprovedInputResponse { slot: u32 },
    EnableTextureStreaming { slot: u32 },
    DisableTextureStreaming { slot: u32 },
    ClearProfileData {slot:u32, #[serde(rename="confirmationToken")] confirmation_token:String, #[serde(rename="confirmationName")] confirmation_name:String, #[serde(default, rename="selectedStorage", skip_serializing_if="Option::is_none")] selected_storage: Option<bool>},
    Launch { slot: u32 },
    #[cfg(feature="mc02-native-input")]
    LaunchMc02Experiment { slot: u32 },
    #[cfg(feature="mc02-native-input")]
    LaunchAiExperiment {slot:u32,mode:AgentControlMode},
    #[cfg(feature="mc02-native-input")]
    Mc02Input {
        slot:u32,keys:u32,dx:i32,dy:i32,ttl_ms:u32,issued_at_ms:u64,
        // Only the legacy batch opts in: immutable original PID/birth/
        // Windows session observed ONCE before its first pulse.
        // Absence retains the established one-shot wire shape exactly.
        #[serde(default,skip_serializing_if="Option::is_none")]
        batch_origin:Option<Mc02BatchOrigin>,
    },
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

impl SessionRequest {
    // These requests belong to an existing controller and never to the desktop.
    pub(crate) fn is_agent_report(&self) -> bool {
        matches!(self, Self::PublishAgentActivity { .. } |
            Self::TestingReserve { .. } | Self::TestingFinalize { .. } |
            Self::TestingInspect { .. })
    }

    pub(crate) fn is_agent_target(&self)->bool {
        matches!(self,Self::InspectAgentTarget { .. })
    }

    // Launch is still a lifecycle operation and must return a full roster.
    #[cfg(feature="mc02-native-input")]
    pub(crate) fn expects_input_receipt(&self)->bool {
        matches!(self, Self::Mc02Input { .. } | Self::AiInputGroup { .. } |
            Self::AiLeasedInput { .. } | Self::AiLeaseAcquire { .. } |
            Self::AiLeaseRelease { .. })
    }

    pub(crate) fn is_experimental_input(&self) -> bool {
        #[cfg(feature="mc02-native-input")]
        {
            self.expects_input_receipt() || matches!(self,
                Self::LaunchMc02Experiment { .. } | Self::LaunchAiExperiment { .. })
        }
        #[cfg(not(feature="mc02-native-input"))]
        { false }
    }
}

pub(crate) type SessionReply = Result<SessionSnapshot, String>;
