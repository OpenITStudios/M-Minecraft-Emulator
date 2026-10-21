#![forbid(unsafe_code)]

mod base_preparation;
mod capacity;
mod client;
mod command;
mod contract;
mod diagnostics;
mod doctor;
mod error;
mod guest;
mod identity;
mod journal;
mod lifecycle_admission;
pub mod independent_isolation;
mod minecraft_runtime;
mod operation_lock;
mod paths;
mod persistence;
mod policy;
mod profile;
mod provider;
mod resources;
mod runtime;
mod schema;
mod startup;
mod support;
mod update;

pub use base_preparation::{inspect_base_preparation, BasePreparationReport};
pub use client::{
    ActionAvailability, ClientId, ClientLifecycleActions, ClientState, ClientStatus,
    ConnectionHealth, DestructiveConfirmation, IdentityState, LifecycleBlocker,
};
pub use command::{
    execute_public_command, execute_public_command_with_progress, OperationPhase,
    OperationProgress, PublicCommandResult,
};
pub use contract::{SuccessReport, PUBLIC_CONTRACT_SCHEMA};
pub use diagnostics::{
    DiagnosticsReport, HostDiagnostics, ProviderDiagnostics, VirtualHardwareDiagnostics,
};
pub use doctor::{
    DoctorClient, DoctorReport, HealthIssue, HealthIssueCode, HealthSeverity, SetupAction,
};
pub use error::{ErrorCode, ErrorReport};
pub use journal::{read_operation_history, OperationKind, OperationOutcome, OperationRecord};
pub use policy::{
    engine_policy, EnginePolicy, MAX_VIRTUAL_CLIENTS, READY_SNAPSHOT_NAME, VIRTUAL_VCPUS,
};
pub use profile::{
    BaseProfile, BaseState, MinecraftInstallType, MinecraftProfile, ProfileParity, ProfileStatus,
};
pub use runtime::{ResourceView, RuntimeStatus, VirtualClients};
pub use schema::{SchemaState, SchemaStatus, CURRENT_RUNTIME_SCHEMA};
pub use support::{EngineSnapshot, SupportBundleResult};
pub use update::{StagedUpdate, UpdateCheck, UpdateState};

#[doc(hidden)]
pub use guest::{
    GuestStatus, MinecraftLaunchResult, MinecraftLaunchState, GUEST_AGENT_PORT,
    GUEST_AGENT_PROTOCOL_VERSION, GUEST_STATUS_SCHEMA, MINECRAFT_LAUNCH_SCHEMA,
};

#[doc(hidden)]
pub fn guest_agent_minecraft_profile() -> Option<MinecraftProfile> {
    profile::native_minecraft_profile()
}

// Desktop, providers and Guest Agent share one bounded subprocess runner.
#[doc(hidden)]
pub fn bounded_command_output<I, S>(
    program: &std::path::Path,
    args: I,
    timeout: std::time::Duration,
) -> std::io::Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    provider::command_output_with_timeout(program, args, timeout)
}

// Pass large fixed scripts via a bounded stdin pipe rather than the Windows
// command line. The existing provider runner keeps the same timeout and
// output-size limits, and manages only its direct child.
#[doc(hidden)]
pub fn bounded_command_output_with_stdin<I, S>(
    program: &std::path::Path,
    args: I,
    timeout: std::time::Duration,
    input: &[u8],
) -> std::io::Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    provider::command_output_with_stdin(program, args, timeout, input)
}
