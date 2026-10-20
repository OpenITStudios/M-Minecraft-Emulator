use crate::{
    capacity::recommended_virtual_clients,
    client::{ClientId, ClientState},
    paths::runtime_root,
    profile::current_base_vmx_path,
    profile::{
        client_lineage_parity, load_client_profile, profile_status, BaseState, ProfileParity,
        ProfileStatus,
    },
    provider::{base_state_for_path, current_platform_provider, vmx_file_present},
    schema::{inspect_runtime_schema, SchemaStatus},
};
use serde::Serialize;
use sysinfo::System;
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DoctorClient {
    pub id: &'static str,
    pub provisioned: bool,
    // Internal provider observation; do not change Doctor's serialized JSON.
    #[serde(skip_serializing)]
    pub(crate) provider_state: ClientState,
    pub ready_snapshot: bool,
    pub lineage_parity: ProfileParity,
    pub identity_provenance: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SetupAction {
    RuntimeDataIncompatible,
    InstallProvider,
    InstallNativeMinecraft,
    PrepareBase,
    RegisterBase,
    FinalizeBase,
    RebuildBase,
    ProvisionVirtuals,
    ReprovisionVirtuals,
    VerifyIdentities,
    CreateReadySnapshots,
    Ready,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HealthSeverity {
    Blocker,
    Warning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HealthIssueCode {
    RuntimeDataIncompatible,
    ProviderUnavailable,
    NativeMinecraftUnavailable,
    BaseMissing,
    BaseRunning,
    BaseStateUnknown,
    BaseFinalizationInterrupted,
    BaseProfileMismatch,
    VirtualNotProvisioned,
    VirtualProviderStateUncertain,
    VirtualLineageMismatch,
    IdentityProofMissing,
    ReadySnapshotMissing,
    HostCapacityLimited,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthIssue {
    pub code: HealthIssueCode,
    pub severity: HealthSeverity,
    pub client: Option<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DoctorReport {
    pub platform: &'static str,
    pub provider: Option<&'static str>,
    pub logical_cpus: usize,
    pub total_memory_gb: f64,
    pub available_memory_gb: f64,
    pub max_recommended_virtual_clients: usize,
    pub base_vm_present: bool,
    pub base_vm_stopped: Option<bool>,
    pub base_state: Option<BaseState>,
    pub runtime_schema: SchemaStatus,
    pub runtime_profile: ProfileStatus,
    pub clients: Vec<DoctorClient>,
    pub next_setup_action: SetupAction,
    pub issues: Vec<HealthIssue>,
}

fn schema_allows_provisioning(status: &SchemaStatus) -> bool {
    matches!(
        status.state,
        crate::schema::SchemaState::Ready | crate::schema::SchemaState::Missing
    )
}

fn select_setup_action(
    runtime_schema: &SchemaStatus,
    provider_available: bool,
    native_available: bool,
    base_vm_present: bool,
    profile_parity: ProfileParity,
    base_state: Option<BaseState>,
    clients: &[DoctorClient],
) -> SetupAction {
    if matches!(
        runtime_schema.state,
        crate::schema::SchemaState::Invalid | crate::schema::SchemaState::NewerThanApp
    ) {
        SetupAction::RuntimeDataIncompatible
    } else if !provider_available {
        SetupAction::InstallProvider
    } else if !native_available {
        SetupAction::InstallNativeMinecraft
    } else if !base_vm_present {
        SetupAction::PrepareBase
    } else if profile_parity != ProfileParity::Match {
        if matches!(
            base_state,
            Some(BaseState::Finalized) | Some(BaseState::Finalizing)
        ) {
            SetupAction::RebuildBase
        } else {
            SetupAction::RegisterBase
        }
    } else if base_state == Some(BaseState::Finalizing) {
        SetupAction::RebuildBase
    } else if base_state != Some(BaseState::Registered) && base_state != Some(BaseState::Finalized)
    {
        SetupAction::RegisterBase
    } else if base_state == Some(BaseState::Registered) {
        SetupAction::FinalizeBase
    } else if clients
        .iter()
        .any(|client| client.provider_state == ClientState::Error)
    {
        // A partial clone or unreadable VM state must never initiate provisioning.
        SetupAction::ReprovisionVirtuals
    } else if clients.iter().any(|client| !client.provisioned) {
        SetupAction::ProvisionVirtuals
    } else if clients
        .iter()
        .any(|client| client.lineage_parity != ProfileParity::Match)
    {
        SetupAction::ReprovisionVirtuals
    } else if clients.iter().any(|client| !client.identity_provenance) {
        SetupAction::VerifyIdentities
    } else if clients.iter().any(|client| !client.ready_snapshot) {
        SetupAction::CreateReadySnapshots
    } else {
        SetupAction::Ready
    }
}

fn collect_health_issues(
    runtime_schema: &SchemaStatus,
    provider_available: bool,
    native_available: bool,
    base_vm_present: bool,
    base_vm_stopped: Option<bool>,
    base_state: Option<BaseState>,
    profile_parity: ProfileParity,
    clients: &[DoctorClient],
    max_recommended_virtual_clients: usize,
) -> Vec<HealthIssue> {
    let mut issues = Vec::new();
    let mut push = |code, severity, client| {
        issues.push(HealthIssue {
            code,
            severity,
            client,
        });
    };

    if matches!(
        runtime_schema.state,
        crate::schema::SchemaState::Invalid | crate::schema::SchemaState::NewerThanApp
    ) {
        push(
            HealthIssueCode::RuntimeDataIncompatible,
            HealthSeverity::Blocker,
            None,
        );
    }

    if !provider_available {
        push(
            HealthIssueCode::ProviderUnavailable,
            HealthSeverity::Blocker,
            None,
        );
    }

    if !native_available {
        push(
            HealthIssueCode::NativeMinecraftUnavailable,
            HealthSeverity::Blocker,
            None,
        );
    }

    if native_available {
        if !base_vm_present {
            push(HealthIssueCode::BaseMissing, HealthSeverity::Blocker, None);
        } else {
            if base_vm_stopped == Some(false) {
                push(HealthIssueCode::BaseRunning, HealthSeverity::Blocker, None);
            }

            match base_state {
                None => push(
                    HealthIssueCode::BaseStateUnknown,
                    HealthSeverity::Blocker,
                    None,
                ),
                Some(BaseState::Finalizing) => push(
                    HealthIssueCode::BaseFinalizationInterrupted,
                    HealthSeverity::Blocker,
                    None,
                ),
                Some(BaseState::Registered | BaseState::Finalized) => {}
            }

            if profile_parity != ProfileParity::Match {
                push(
                    HealthIssueCode::BaseProfileMismatch,
                    HealthSeverity::Blocker,
                    None,
                );
            }
        }
    }

    if provider_available {
        for client in clients {
            if client.provider_state == ClientState::Error {
                push(
                    HealthIssueCode::VirtualProviderStateUncertain,
                    HealthSeverity::Blocker,
                    Some(client.id),
                );
                continue;
            }
            if !client.provisioned {
                push(
                    HealthIssueCode::VirtualNotProvisioned,
                    HealthSeverity::Warning,
                    Some(client.id),
                );
                continue;
            }
            if client.lineage_parity != ProfileParity::Match {
                push(
                    HealthIssueCode::VirtualLineageMismatch,
                    HealthSeverity::Blocker,
                    Some(client.id),
                );
            }
            if !client.identity_provenance {
                push(
                    HealthIssueCode::IdentityProofMissing,
                    HealthSeverity::Warning,
                    Some(client.id),
                );
            }
            if !client.ready_snapshot {
                push(
                    HealthIssueCode::ReadySnapshotMissing,
                    HealthSeverity::Warning,
                    Some(client.id),
                );
            }
        }
    }

    if max_recommended_virtual_clients < ClientId::VIRTUAL.len() {
        push(
            HealthIssueCode::HostCapacityLimited,
            HealthSeverity::Warning,
            None,
        );
    }

    issues
}

pub fn doctor() -> DoctorReport {
    let provider = current_platform_provider();
    let base = current_base_vmx_path().ok();

    let mut system = System::new();
    system.refresh_memory();
    system.refresh_cpu_all();
    let logical_cpus = system.cpus().len();
    let total_memory_gb = system.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    let available_memory_gb = system.available_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    let max_recommended_virtual_clients =
        recommended_virtual_clients(total_memory_gb, logical_cpus);

    let base_vm_present = base
        .as_ref()
        .is_some_and(|path| vmx_file_present(path).unwrap_or(false));
    let base_vm_stopped = match (provider.as_ref(), base.as_ref()) {
        (Some(provider), Some(path)) if base_vm_present => {
            provider.is_running_path(path).ok().map(|running| !running)
        }
        _ => None,
    };

    let base_state = base
        .as_ref()
        .and_then(|path| base_state_for_path(path).ok())
        .flatten();

    let runtime_schema = runtime_root()
        .map(|root| inspect_runtime_schema(&root))
        .unwrap_or_else(|_| SchemaStatus {
            state: crate::schema::SchemaState::Invalid,
            schema: None,
        });
    let runtime_profile = profile_status();

    // Reuse a single VMware power-state list when all three can be read
    // together. A failed batch falls back to per-client diagnostics as before.
    let observed_states = provider
        .as_ref()
        .and_then(|provider| provider.observe_client_states().ok());
    let clients: Vec<DoctorClient> = ClientId::VIRTUAL
        .into_iter()
        .map(|client| {
            let state = observed_states
                .as_ref()
                .and_then(|states| {
                    states
                        .iter()
                        .find(|(id, _)| *id == client)
                        .map(|(_, state)| *state)
                })
                .or_else(|| {
                    provider
                        .as_ref()
                        .and_then(|provider| provider.status(client).ok())
                });
            // Missing evidence is UNKNOWN/ERROR, never a clean VM slot.
            let provider_state = state.unwrap_or(ClientState::Error);
            let provisioned = matches!(
                provider_state,
                ClientState::Stopped | ClientState::Suspended | ClientState::Running
            );
            let ready_snapshot = if provisioned {
                provider
                    .as_ref()
                    .and_then(|provider| provider.has_ready(client).ok())
                    .unwrap_or(false)
            } else {
                false
            };

            let client_profile = provisioned
                .then(|| load_client_profile(client).ok())
                .flatten();
            let lineage_parity = client_lineage_parity(
                runtime_profile.native.as_ref(),
                runtime_profile.base.as_ref(),
                client_profile.as_ref(),
                provisioned,
            );

            let current_vm_identity = provider
                .as_ref()
                .and_then(|provider| provider.identity_key(client).ok().flatten());
            let identity_provenance = load_client_profile(client).ok().is_some_and(|profile| {
                profile.identity_provenance_matches(current_vm_identity.as_deref())
            });

            DoctorClient {
                id: client.as_str(),
                provisioned,
                provider_state,
                ready_snapshot,
                lineage_parity,
                identity_provenance,
            }
        })
        .collect();

    let next_setup_action = select_setup_action(
        &runtime_schema,
        provider.is_some(),
        runtime_profile.native.is_some(),
        base_vm_present,
        runtime_profile.parity,
        base_state,
        &clients,
    );

    let issues = collect_health_issues(
        &runtime_schema,
        provider.is_some(),
        runtime_profile.native.is_some(),
        base_vm_present,
        base_vm_stopped,
        base_state,
        runtime_profile.parity,
        &clients,
        max_recommended_virtual_clients,
    );

    DoctorReport {
        platform: std::env::consts::OS,
        provider: provider.as_ref().map(|provider| provider.id()),
        logical_cpus,
        total_memory_gb,
        available_memory_gb,
        max_recommended_virtual_clients,
        base_vm_present,
        base_vm_stopped,
        base_state: base_state.clone(),
        runtime_schema,
        runtime_profile: runtime_profile.clone(),
        clients,
        next_setup_action,
        issues,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        collect_health_issues, schema_allows_provisioning, select_setup_action, DoctorClient,
        HealthIssueCode, HealthSeverity, SetupAction,
    };
    use crate::{
        guest::{GUEST_AGENT_PROTOCOL_VERSION, GUEST_STATUS_SCHEMA},
        profile::{
            BaseProfile, BaseState, ClientProfile, MinecraftInstallType, MinecraftProfile,
            ProfileParity, BASE_PROFILE_SCHEMA, CLIENT_PROFILE_SCHEMA,
        },
        schema::{SchemaState, SchemaStatus},
    };

    fn compatible_schema() -> SchemaStatus {
        SchemaStatus {
            state: SchemaState::Ready,
            schema: Some(crate::schema::CURRENT_RUNTIME_SCHEMA),
        }
    }

    fn client(ready_snapshot: bool, identity_provenance: bool) -> DoctorClient {
        DoctorClient {
            id: "Virtual-test",
            provisioned: true,
            provider_state: crate::client::ClientState::Stopped,
            ready_snapshot,
            lineage_parity: ProfileParity::Match,
            identity_provenance,
        }
    }

    #[test]
    fn doctor_lineage_requires_the_current_base_generation() {
        let native = MinecraftProfile {
            version: "1.21.120.0".into(),
            install_type: MinecraftInstallType::Desktop,
        };
        let base = BaseProfile {
            schema: BASE_PROFILE_SCHEMA,
            minecraft_version: native.version.clone(),
            guest_status_schema: GUEST_STATUS_SCHEMA,
            guest_agent_protocol: GUEST_AGENT_PROTOCOL_VERSION,
            base_generation_id: "a".repeat(64),
        };
        let current = ClientProfile {
            schema: CLIENT_PROFILE_SCHEMA,
            base_minecraft_version: native.version.clone(),
            base_generation_id: base.base_generation_id.clone(),
            verified_vm_identity: None,
            verified_windows_identity: None,
        };
        assert_eq!(
            crate::profile::client_lineage_parity(Some(&native), Some(&base), Some(&current), true),
            ProfileParity::Match
        );

        let stale = ClientProfile {
            base_generation_id: "b".repeat(64),
            ..current
        };
        assert_eq!(
            crate::profile::client_lineage_parity(Some(&native), Some(&base), Some(&stale), true),
            ProfileParity::Mismatch
        );
        assert_eq!(
            crate::profile::client_lineage_parity(Some(&native), Some(&base), Some(&stale), false),
            ProfileParity::Unknown
        );
    }

    #[test]
    fn health_issues_are_dependency_aware_and_machine_readable() {
        let issues = collect_health_issues(
            &compatible_schema(),
            false,
            true,
            true,
            Some(true),
            Some(BaseState::Finalized),
            ProfileParity::Match,
            &[],
            3,
        );
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, HealthIssueCode::ProviderUnavailable);
        assert_eq!(issues[0].severity, HealthSeverity::Blocker);

        let clients = vec![client(false, false)];
        let issues = collect_health_issues(
            &compatible_schema(),
            true,
            true,
            true,
            Some(true),
            Some(BaseState::Finalized),
            ProfileParity::Match,
            &clients,
            2,
        );
        assert!(issues
            .iter()
            .any(|issue| issue.code == HealthIssueCode::IdentityProofMissing));
        assert!(issues
            .iter()
            .any(|issue| issue.code == HealthIssueCode::ReadySnapshotMissing));
        assert!(issues
            .iter()
            .any(|issue| issue.code == HealthIssueCode::HostCapacityLimited));
    }

    #[test]
    fn provisioning_readiness_fails_closed_on_incompatible_schema() {
        let ready = SchemaStatus {
            state: SchemaState::Ready,
            schema: Some(crate::schema::CURRENT_RUNTIME_SCHEMA),
        };
        let missing = SchemaStatus {
            state: SchemaState::Missing,
            schema: None,
        };
        let invalid = SchemaStatus {
            state: SchemaState::Invalid,
            schema: None,
        };
        let newer = SchemaStatus {
            state: SchemaState::NewerThanApp,
            schema: Some(crate::schema::CURRENT_RUNTIME_SCHEMA + 1),
        };

        assert!(schema_allows_provisioning(&ready));
        assert!(schema_allows_provisioning(&missing));
        assert!(!schema_allows_provisioning(&invalid));
        assert!(!schema_allows_provisioning(&newer));
    }

    #[test]
    fn identity_proof_precedes_ready_snapshot_setup() {
        let clients = vec![
            client(false, false),
            client(false, false),
            client(false, false),
        ];
        assert_eq!(
            select_setup_action(
                &compatible_schema(),
                true,
                true,
                true,
                ProfileParity::Match,
                Some(BaseState::Finalized),
                &clients,
            ),
            SetupAction::VerifyIdentities
        );

        let clients = vec![
            client(false, true),
            client(false, true),
            client(false, true),
        ];
        assert_eq!(
            select_setup_action(
                &compatible_schema(),
                true,
                true,
                true,
                ProfileParity::Match,
                Some(BaseState::Finalized),
                &clients,
            ),
            SetupAction::CreateReadySnapshots
        );
    }

    #[test]
    fn base_state_routes_legacy_and_interrupted_setup_safely() {
        let clients = vec![
            client(false, false),
            client(false, false),
            client(false, false),
        ];

        assert_eq!(
            select_setup_action(
                &compatible_schema(),
                true,
                true,
                true,
                ProfileParity::Match,
                None,
                &clients,
            ),
            SetupAction::RegisterBase
        );

        assert_eq!(
            select_setup_action(
                &compatible_schema(),
                true,
                true,
                true,
                ProfileParity::Match,
                Some(BaseState::Registered),
                &clients,
            ),
            SetupAction::FinalizeBase
        );

        assert_eq!(
            select_setup_action(
                &compatible_schema(),
                true,
                true,
                true,
                ProfileParity::Match,
                Some(BaseState::Finalizing),
                &clients,
            ),
            SetupAction::RebuildBase
        );
    }

    #[test]
    fn stale_finalized_base_requires_rebuild_not_reregistration() {
        let clients = vec![
            client(false, false),
            client(false, false),
            client(false, false),
        ];

        assert_eq!(
            select_setup_action(
                &compatible_schema(),
                true,
                true,
                true,
                ProfileParity::Mismatch,
                Some(BaseState::Finalized),
                &clients,
            ),
            SetupAction::RebuildBase
        );
        assert_eq!(
            select_setup_action(
                &compatible_schema(),
                true,
                true,
                true,
                ProfileParity::Mismatch,
                Some(BaseState::Registered),
                &clients,
            ),
            SetupAction::RegisterBase
        );
    }

    #[test]
    fn setup_action_precedence_is_deterministic() {
        let ready_clients = vec![client(true, true), client(true, true), client(true, true)];

        let invalid_schema = SchemaStatus {
            state: SchemaState::Invalid,
            schema: None,
        };
        assert_eq!(
            select_setup_action(
                &invalid_schema,
                false,
                false,
                false,
                ProfileParity::Unknown,
                None,
                &[],
            ),
            SetupAction::RuntimeDataIncompatible
        );

        assert_eq!(
            select_setup_action(
                &compatible_schema(),
                false,
                false,
                false,
                ProfileParity::Unknown,
                None,
                &[],
            ),
            SetupAction::InstallProvider
        );

        assert_eq!(
            select_setup_action(
                &compatible_schema(),
                true,
                false,
                false,
                ProfileParity::Unknown,
                None,
                &[],
            ),
            SetupAction::InstallNativeMinecraft
        );

        assert_eq!(
            select_setup_action(
                &compatible_schema(),
                true,
                true,
                false,
                ProfileParity::Unknown,
                None,
                &[],
            ),
            SetupAction::PrepareBase
        );

        let mut missing_virtual = ready_clients.clone();
        missing_virtual[1].provisioned = false;
        assert_eq!(
            select_setup_action(
                &compatible_schema(),
                true,
                true,
                true,
                ProfileParity::Match,
                Some(BaseState::Finalized),
                &missing_virtual,
            ),
            SetupAction::ProvisionVirtuals
        );

        let mut stale_lineage = ready_clients.clone();
        stale_lineage[2].lineage_parity = ProfileParity::Unknown;
        assert_eq!(
            select_setup_action(
                &compatible_schema(),
                true,
                true,
                true,
                ProfileParity::Match,
                Some(BaseState::Finalized),
                &stale_lineage,
            ),
            SetupAction::ReprovisionVirtuals
        );

        let mut no_identity = ready_clients.clone();
        no_identity[0].identity_provenance = false;
        assert_eq!(
            select_setup_action(
                &compatible_schema(),
                true,
                true,
                true,
                ProfileParity::Match,
                Some(BaseState::Finalized),
                &no_identity,
            ),
            SetupAction::VerifyIdentities
        );

        let mut no_ready = ready_clients.clone();
        no_ready[0].ready_snapshot = false;
        assert_eq!(
            select_setup_action(
                &compatible_schema(),
                true,
                true,
                true,
                ProfileParity::Match,
                Some(BaseState::Finalized),
                &no_ready,
            ),
            SetupAction::CreateReadySnapshots
        );

        assert_eq!(
            select_setup_action(
                &compatible_schema(),
                true,
                true,
                true,
                ProfileParity::Match,
                Some(BaseState::Finalized),
                &ready_clients,
            ),
            SetupAction::Ready
        );
    }

    #[test]
    fn ready_requires_all_ready_snapshots() {
        let mut clients = vec![client(true, true), client(true, true), client(false, true)];
        assert_eq!(
            select_setup_action(
                &compatible_schema(),
                true,
                true,
                true,
                ProfileParity::Match,
                Some(BaseState::Finalized),
                &clients,
            ),
            SetupAction::CreateReadySnapshots
        );

        clients[2].ready_snapshot = true;
        assert_eq!(
            select_setup_action(
                &compatible_schema(),
                true,
                true,
                true,
                ProfileParity::Match,
                Some(BaseState::Finalized),
                &clients,
            ),
            SetupAction::Ready
        );
    }
    #[test]
    fn changed_or_unreadable_vm_identity_reopens_identity_verification() {
        let profile = crate::profile::ClientProfile {
            schema: crate::profile::CLIENT_PROFILE_SCHEMA,
            base_minecraft_version: "1.0.0".into(),
            base_generation_id: "c".repeat(64),
            verified_vm_identity: Some(crate::profile::identity_fingerprint("uuid-a|mac-a")),
            verified_windows_identity: Some("windows-proof".into()),
        };
        for observed in [Some("uuid-b|mac-a"), None] {
            let mut clients = vec![client(true, true), client(true, true), client(true, true)];
            clients[0].identity_provenance = profile.identity_provenance_matches(observed);
            assert_eq!(
                select_setup_action(
                    &compatible_schema(),
                    true,
                    true,
                    true,
                    ProfileParity::Match,
                    Some(BaseState::Finalized),
                    &clients,
                ),
                SetupAction::VerifyIdentities,
            );
        }
    }

    #[test]
    fn incomplete_vm_requires_review_not_new_provisioning() {
        let mut clients = vec![client(true, true), client(true, true), client(true, true)];
        clients[1].provider_state = crate::client::ClientState::Error;
        clients[1].provisioned = false;
        assert_eq!(
            select_setup_action(
                &compatible_schema(),
                true,
                true,
                true,
                ProfileParity::Match,
                Some(BaseState::Finalized),
                &clients,
            ),
            SetupAction::ReprovisionVirtuals,
        );
        let issues = collect_health_issues(
            &compatible_schema(),
            true,
            true,
            true,
            Some(true),
            Some(BaseState::Finalized),
            ProfileParity::Match,
            &clients,
            3,
        );
        assert!(issues.iter().any(|issue| {
            issue.client == Some("Virtual-test")
                && issue.code == HealthIssueCode::VirtualProviderStateUncertain
                && issue.severity == HealthSeverity::Blocker
        }));
    }
}
