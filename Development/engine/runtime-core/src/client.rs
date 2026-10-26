use crate::profile::ProfileParity;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClientId {
    Native,
    Virtual01,
    Virtual02,
    Virtual03,
}

impl ClientId {
    pub const ALL: [Self; 4] = [
        Self::Native,
        Self::Virtual01,
        Self::Virtual02,
        Self::Virtual03,
    ];

    pub const VIRTUAL: [Self; 3] = [Self::Virtual01, Self::Virtual02, Self::Virtual03];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Native => "Native",
            Self::Virtual01 => "Virtual-01",
            Self::Virtual02 => "Virtual-02",
            Self::Virtual03 => "Virtual-03",
        }
    }

    pub fn is_native(self) -> bool {
        self == Self::Native
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClientState {
    Manual,
    NotProvisioned,
    Stopped,
    Suspended,
    Running,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IdentityState {
    Unknown,
    Unique,
    Duplicate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LifecycleBlocker {
    ProviderUnavailable,
    NativeManaged,
    NotProvisioned,
    InvalidState,
    ReadySnapshotMissing,
    ReadySnapshotExists,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionAvailability {
    pub allowed: bool,
    pub blocker: Option<LifecycleBlocker>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientLifecycleActions {
    pub id: &'static str,
    pub start: ActionAvailability,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_setup: Option<ActionAvailability>,
    pub suspend: ActionAvailability,
    pub stop: ActionAvailability,
    pub open: ActionAvailability,
    pub restart: ActionAvailability,
    pub set_ready: ActionAvailability,
    pub reset: ActionAvailability,
    pub reprovision: ActionAvailability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DestructiveConfirmation {
    ReprovisionAccountState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConnectionHealth {
    VmOffline,
    VmRunning,
    GuestToolsReady,
    GuestAgentReady,
    InteractiveLauncherReady,
    MinecraftRunning,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientStatus {
    pub id: &'static str,
    pub native: bool,
    pub state: ClientState,
    pub ready_snapshot: Option<bool>,
    pub memory_limit_mb: Option<u64>,
    pub host_working_set_mb: Option<u64>,
    pub guest_tools_ready: Option<bool>,
    pub guest_agent_ready: Option<bool>,
    pub guest_agent_version: Option<String>,
    pub minecraft_version: Option<String>,
    pub minecraft_running: Option<bool>,
    pub interactive_launcher_ready: Option<bool>,
    pub connection_health: Option<ConnectionHealth>,
    pub lineage_parity: Option<ProfileParity>,
    pub version_parity: Option<ProfileParity>,
    pub vm_identity: Option<IdentityState>,
    pub windows_identity: Option<IdentityState>,
}
