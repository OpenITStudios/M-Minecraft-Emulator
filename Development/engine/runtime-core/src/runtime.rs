use crate::{
    capacity::current_host_capacity,
    client::{
        ActionAvailability, ClientId, ClientLifecycleActions, ClientState, ClientStatus,
        ConnectionHealth, DestructiveConfirmation, IdentityState, LifecycleBlocker,
    },
    diagnostics::{collect as collect_diagnostics, DiagnosticsReport},
    doctor::{doctor, DoctorReport},
    guest::{
        guest_agent_launch_compatible, query_guest_status, GuestStatus,
        GUEST_AGENT_PROTOCOL_VERSION,
    },
    identity::classify_identity_state,
    journal::{record_operation, OperationKind},
    lifecycle_admission::{
        evaluate_lifecycle_admission, require_lifecycle_admission, validate_power_state,
        LifecycleAction, LifecycleFacts,
    },
    operation_lock::OperationLock,
    paths::runtime_root,
    policy::{engine_policy, EnginePolicy, MAX_VIRTUAL_CLIENTS},
    profile::{
        client_lineage_parity, current_base_vmx_path, identity_fingerprint, load_base_profile,
        load_client_profile, native_minecraft_profile, profile_status,
        require_base_launch_protocol, require_base_matches_native, require_client_matches_native,
        write_client_profile, write_verified_base_profile, write_verified_client_identities,
        BaseProfile, BaseState, MinecraftProfile, ProfileParity, ProfileStatus,
    },
    provider::{
        base_state_for_path, cleanup_staging, current_platform_provider,
        ensure_guest_token_for_path, set_base_state_for_path, vmx_file_present, Provider,
    },
    resources::{current_host_pressure, HostPressure, StartDecision, VIRTUAL_MEMORY_LIMIT_MB},
    schema::{inspect_runtime_schema, SchemaState},
    startup::wait_until_start_is_safe,
    support::{capture_time_ms, write_support_bundle, EngineSnapshot, SupportBundleResult},
    update::{check_update, stage_update, StagedUpdate, UpdateCheck},
};
use serde::Serialize;
use std::{fs, io, thread, time::Duration};

#[derive(Debug, Default)]
pub struct VirtualClients;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatus {
    pub provider: Option<&'static str>,
    pub runtime_profile: ProfileStatus,
    pub pressure: HostPressure,
    pub clients: Vec<ClientStatus>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceView {
    pub requested_virtual_clients: usize,
    pub recommended_virtual_clients: usize,
    pub running_virtual_clients: usize,
    pub suspended_virtual_clients: usize,
    pub stopped_virtual_clients: usize,
    pub virtual_memory_limit_mb: u64,
    pub observed_working_set_mb: u64,
    pub observed_working_set_instances: usize,
    pub pressure: HostPressure,
}

fn require_base_finalization_state(state: Option<BaseState>) -> io::Result<()> {
    if state == Some(BaseState::Registered) {
        return Ok(());
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        "Base can be opened for finalization only while it is REGISTERED",
    ))
}

#[derive(Debug, Clone)]
struct ClientObservation {
    client: ClientId,
    state: ClientState,
    vm_identity: Option<String>,
    guest: Option<GuestStatus>,
}

fn collect_client_observations(provider: &dyn Provider) -> io::Result<Vec<ClientObservation>> {
    provider
        .observe_client_states()?
        .into_iter()
        .map(|(client, state)| {
            let vm_identity = if state == ClientState::NotProvisioned {
                None
            } else {
                provider.identity_key(client).ok().flatten()
            };
            let guest = (state == ClientState::Running)
                .then(|| guest_status_once(provider, client))
                .flatten();
            Ok(ClientObservation {
                client,
                state,
                vm_identity,
                guest,
            })
        })
        .collect()
}

fn observation_for(
    observations: &[ClientObservation],
    client: ClientId,
) -> io::Result<&ClientObservation> {
    observations
        .iter()
        .find(|observation| observation.client == client)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{} observation is missing", client.as_str()),
            )
        })
}

fn vm_identity_state_from_observations(
    client: ClientId,
    observations: &[ClientObservation],
) -> IdentityState {
    let Ok(current) = observation_for(observations, client) else {
        return IdentityState::Unknown;
    };
    let mut peers = Vec::new();

    for other in observations {
        if other.client == client || other.state == ClientState::NotProvisioned {
            continue;
        }
        peers.push(other.vm_identity.as_deref());
    }

    classify_identity_state(current.vm_identity.as_deref(), peers)
}

fn vm_identity_state(provider: &dyn Provider, client: ClientId) -> IdentityState {
    let current = provider.identity_key(client).ok().flatten();
    let mut peers = Vec::new();

    for other in ClientId::VIRTUAL {
        if other == client {
            continue;
        }

        match provider.status(other) {
            Ok(ClientState::NotProvisioned) => continue,
            Ok(_) => peers.push(provider.identity_key(other).ok().flatten()),
            Err(_) => peers.push(None),
        }
    }

    classify_identity_state(
        current.as_deref(),
        peers.iter().map(|identity| identity.as_deref()),
    )
}

fn collect_lifecycle_facts(
    provider: &dyn Provider,
    client: ClientId,
    can_start_virtual: bool,
) -> io::Result<LifecycleFacts> {
    let state = provider.status(client)?;
    let ready_snapshot = if state == ClientState::NotProvisioned {
        Some(false)
    } else {
        provider.has_ready(client).ok()
    };
    let root = runtime_root()?;
    let schema_ready = matches!(
        inspect_runtime_schema(&root).state,
        SchemaState::Ready | SchemaState::Missing
    );
    let profile = profile_status();
    let client_profile = load_client_profile(client).ok();
    let current_vm = provider.identity_key(client).ok().flatten();
    let saved_vm_identity_matches = client_profile.as_ref().map_or(true, |profile| {
        profile.saved_vm_identity_matches(current_vm.as_deref())
    });
    let identity_provenance_bound = client_profile
        .as_ref()
        .is_some_and(|profile| profile.identity_provenance_matches(current_vm.as_deref()));
    let client_compatible = client_lineage_parity(
        profile.native.as_ref(),
        profile.base.as_ref(),
        client_profile.as_ref(),
        state != ClientState::NotProvisioned,
    ) == ProfileParity::Match;
    // The provider owns the meaning of a prepared, stopped base/template.
    // Unreadable provenance is never promoted to first-time setup readiness.
    let base_finalized_and_stopped = provider.base_finalized_and_stopped().unwrap_or(false);

    let vm_identity = vm_identity_state(provider, client);
    let fresh_client_profile = client_profile.as_ref().is_some_and(|profile| {
        profile.verified_vm_identity.is_none() && profile.verified_windows_identity.is_none()
    });

    Ok(LifecycleFacts {
        state,
        ready_snapshot,
        schema_ready,
        base_compatible: profile.parity == ProfileParity::Match,
        client_compatible,
        saved_vm_identity_matches,
        vm_identity_duplicate: vm_identity == IdentityState::Duplicate,
        vm_identity_unique: vm_identity == IdentityState::Unique,
        fresh_client_profile,
        identity_provenance_bound,
        base_finalized_and_stopped,
        can_start: can_start_virtual,
    })
}

fn check_action_admission(
    provider: &dyn Provider,
    client: ClientId,
    action: LifecycleAction,
) -> io::Result<()> {
    let can_start_virtual = current_host_pressure().start_decision != StartDecision::Block;
    require_lifecycle_admission(
        client,
        action,
        &collect_lifecycle_facts(provider, client, can_start_virtual)?,
    )
}

fn restore_batch_state(
    provider: &dyn Provider,
    started: &[(ClientId, ClientState)],
) -> Vec<&'static str> {
    let mut failed = Vec::new();

    for (client, original_state) in started.iter().rev() {
        let result = match original_state {
            ClientState::Suspended => provider.suspend(*client).map(|_| ()),
            ClientState::Stopped => provider.stop(*client).map(|_| ()),
            _ => Ok(()),
        };

        if result.is_err() {
            failed.push(client.as_str());
        }
    }

    failed
}

// VM power batches are deliberately non-atomic. A failed provider command may
// already have changed its VM; never mask that with an implicit restart.
// Project observed states to the caller so a human can reconcile before retry.
// Inspect all existing Virtual lineage before attempting the first clone.
// Partial/missing provenance must never cause another guest to be provisioned.
fn missing_provision_targets(
    provider: &dyn Provider,
    mut verify_existing: impl FnMut(ClientId) -> io::Result<()>,
) -> io::Result<Vec<ClientId>> {
    let mut missing = Vec::new();
    for client in ClientId::VIRTUAL {
        if provider.status(client)? == ClientState::NotProvisioned {
            missing.push(client);
        } else {
            verify_existing(client).map_err(|error| io::Error::new(error.kind(), format!(
                "{} exists but its lineage is not usable: {error}. Existing VM/account data were preserved; no new clients were provisioned.",
                client.as_str()
            )))?;
        }
    }
    Ok(missing)
}

fn execute_non_atomic_power_batch(
    provider: &dyn Provider,
    targets: &[ClientId],
    action: &str,
    mut apply: impl FnMut(ClientId) -> io::Result<()>,
) -> io::Result<()> {
    for &client in targets {
        if let Err(error) = apply(client) {
            let observed = targets
                .iter()
                .map(|&target| {
                    let state = provider
                        .status(target)
                        .map(|state| format!("{state:?}"))
                        .unwrap_or_else(|read_error| format!("UNKNOWN({read_error})"));
                    format!("{}={state}", target.as_str())
                })
                .collect::<Vec<_>>()
                .join(", ");
            return Err(io::Error::new(
                error.kind(),
                format!(
                    "{action} batch interrupted at {}: {error}. Partial completion possible; observed VM states: {observed}. Inspect status before retrying.",
                    client.as_str()
                ),
            ));
        }
    }
    Ok(())
}

fn stop_targets(
    provider: &dyn Provider,
    targets: &[ClientId],
    mut admit: impl FnMut(ClientId) -> io::Result<()>,
) -> io::Result<()> {
    // Deterministic admission must succeed for the whole batch before mutation.
    // Recheck before each command to detect newly changed provider state.
    for &client in targets {
        admit(client)?;
    }
    execute_non_atomic_power_batch(provider, targets, "Stop", |client| {
        admit(client)?;
        provider.stop(client).map(|_| ())
    })
}

fn suspend_targets(
    provider: &dyn Provider,
    targets: &[ClientId],
    mut admit: impl FnMut(ClientId) -> io::Result<()>,
) -> io::Result<()> {
    for &client in targets {
        admit(client)?;
    }
    execute_non_atomic_power_batch(provider, targets, "Suspend", |client| {
        admit(client)?;
        provider.suspend(client).map(|_| ())
    })
}

// Guest verification can fail after a VM was already reset/restarted. Never
// power it off as a hidden rollback; preserve the observed state in the error.
fn observed_post_action_failure(
    provider: &dyn Provider,
    client: ClientId,
    action: &str,
    error: io::Error,
) -> io::Error {
    let state = provider
        .status(client)
        .map(|value| format!("{value:?}"))
        .unwrap_or_else(|probe| format!("UNKNOWN({probe})"));
    io::Error::new(error.kind(), format!(
        "{action} failed for {}: {error}. Observed VM state: {state}. No automatic stop attempted; inspect status before retrying.",
        client.as_str()
    ))
}

fn with_rollback_context(error: io::Error, failed: &[&'static str]) -> io::Error {
    if failed.is_empty() {
        return error;
    }

    io::Error::new(
        error.kind(),
        format!(
            "{error}; rollback incomplete for {}. Run status to inspect the current state.",
            failed.join(", ")
        ),
    )
}

// Shared by production startup and deterministic provider tests. Every failure
// after the first mutation follows the same rollback path, including failures
// in admission, compatibility verification, and result collection.
fn execute_start_batch<T>(
    provider: &dyn Provider,
    targets: &[ClientId],
    mut prepare: impl FnMut(ClientId, ClientState) -> io::Result<()>,
    mut verify: impl FnMut(ClientId) -> io::Result<T>,
) -> io::Result<Vec<T>> {
    let mut changed = Vec::with_capacity(targets.len());
    let outcome: io::Result<Vec<T>> = (|| {
        let mut results = Vec::with_capacity(targets.len());
        for &client in targets {
            let original_state = provider.status(client)?;
            validate_power_state(client, LifecycleAction::Start, original_state, false)?;
            prepare(client, original_state)?;

            if original_state != ClientState::Running {
                // A provider may mutate the VM before returning a timeout/error.
                // Record responsibility before calling it so that case is covered.
                changed.push((client, original_state));
                provider.start(client)?;
            }
            match verify(client) {
                Ok(result) => results.push(result),
                Err(error) => {
                    // A newly started/resumed client that fails verification
                    // must be stopped, not left live or saved as a bad resume.
                    // Earlier successful resumes still return to Suspended.
                    if original_state != ClientState::Running {
                        if let Some((_, rollback_state)) = changed.last_mut() {
                            *rollback_state = ClientState::Stopped;
                        }
                    }
                    return Err(error);
                }
            }
        }
        Ok(results)
    })();

    match outcome {
        Ok(results) => Ok(results),
        Err(error) => {
            let failed = restore_batch_state(provider, &changed);
            Err(with_rollback_context(error, &failed))
        }
    }
}

// Setup reuses startup rollback but intentionally does not probe a guest that
// may still be waiting for user input in Windows OOBE.
fn execute_setup_start(
    provider: &dyn Provider,
    client: ClientId,
    prepare: impl FnMut(ClientId, ClientState) -> io::Result<()>,
) -> io::Result<ClientState> {
    let mut states =
        execute_start_batch(provider, &[client], prepare, |client| provider.open(client))?;
    states.pop().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::Other,
            "first-time setup returned no client state",
        )
    })
}

fn guest_status_once(provider: &dyn Provider, client: ClientId) -> Option<GuestStatus> {
    // A provider must return explicit Minecraft guest evidence. VM/agent online
    // alone cannot be substituted for the Education-specific GuestStatus.
    provider
        .guest_status(client, Duration::from_secs(1))
        .ok()
        .flatten()
}

fn lineage_parity(native: Option<&MinecraftProfile>, client: ClientId) -> ProfileParity {
    let base = load_base_profile().ok();
    let client_profile = load_client_profile(client).ok();
    client_lineage_parity(
        native,
        base.as_ref(),
        client_profile.as_ref(),
        client_profile.is_some(),
    )
}

fn version_parity(native: Option<&MinecraftProfile>, guest: Option<&GuestStatus>) -> ProfileParity {
    match (native, guest.and_then(|status| status.minecraft.as_ref())) {
        (Some(native), Some(guest)) if native.version == guest.version => ProfileParity::Match,
        (Some(_), Some(_)) => ProfileParity::Mismatch,
        _ => ProfileParity::Unknown,
    }
}

fn guest_probe_error_is_terminal(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::InvalidData
            | io::ErrorKind::InvalidInput
            | io::ErrorKind::PermissionDenied
            | io::ErrorKind::Other
    )
}

fn wait_for_guest_compatibility(
    provider: &dyn Provider,
    client: ClientId,
    timeout: Duration,
    enforce_verified_windows_identity: bool,
) -> io::Result<GuestStatus> {
    let native = native_minecraft_profile().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Native Minecraft Education version could not be detected",
        )
    })?;
    let started = std::time::Instant::now();

    loop {
        match provider.guest_status(client, Duration::from_secs(2)) {
            Ok(Some(status)) => {
                let guest = status.minecraft.as_ref().ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::NotFound,
                        format!(
                            "{} Guest Agent cannot detect Minecraft Education",
                            client.as_str()
                        ),
                    )
                })?;

                if !guest_agent_launch_compatible(status.protocol_version) {
                    return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!(
                                "{} Guest Agent protocol {} cannot launch Minecraft Education; backend launch protocol {} required",
                                client.as_str(),
                                status.protocol_version,
                                GUEST_AGENT_PROTOCOL_VERSION
                            ),
                        ));
                }

                if enforce_verified_windows_identity {
                    if let Some(expected_windows_identity) = load_client_profile(client)
                        .ok()
                        .and_then(|profile| profile.verified_windows_identity)
                    {
                        let current_windows_identity =
                                status.machine_identity.as_deref().ok_or_else(|| {
                                    io::Error::new(
                                        io::ErrorKind::InvalidData,
                                        format!(
                                            "{} Windows identity cannot be verified against saved provenance",
                                            client.as_str()
                                        ),
                                    )
                                })?;
                        if current_windows_identity != expected_windows_identity {
                            return Err(io::Error::new(
                                    io::ErrorKind::InvalidData,
                                    format!(
                                        "{} Windows identity changed after verification; reprovision or run verify-identities after resolving the identity change",
                                        client.as_str()
                                    ),
                                ));
                        }
                    }
                }

                if guest.version != native.version {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "{} Minecraft Education version {} does not match Native {}",
                            client.as_str(),
                            guest.version,
                            native.version
                        ),
                    ));
                }

                return Ok(status);
            }
            Ok(None) => {}
            Err(error) if guest_probe_error_is_terminal(&error) => {
                return Err(io::Error::new(
                    error.kind(),
                    format!("{} Guest Agent probe failed: {error}", client.as_str()),
                ));
            }
            Err(_) => {}
        }

        if started.elapsed() >= timeout {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!(
                    "{} Guest Agent did not become ready within {}s",
                    client.as_str(),
                    timeout.as_secs()
                ),
            ));
        }

        thread::sleep(Duration::from_secs(1));
    }
}

fn windows_identity_state(client: ClientId, observations: &[ClientObservation]) -> IdentityState {
    let Ok(current) = observation_for(observations, client) else {
        return IdentityState::Unknown;
    };
    let Some(identity) = current
        .guest
        .as_ref()
        .and_then(|status| status.machine_identity.as_deref())
    else {
        return IdentityState::Unknown;
    };

    for other in observations {
        if other.client == client {
            continue;
        }
        if other.state != ClientState::Running {
            return IdentityState::Unknown;
        }
        let Some(other_identity) = other
            .guest
            .as_ref()
            .and_then(|status| status.machine_identity.as_deref())
        else {
            return IdentityState::Unknown;
        };
        if other_identity == identity {
            return IdentityState::Duplicate;
        }
    }

    IdentityState::Unique
}

fn verify_identity_provenance(provider: &dyn Provider) -> io::Result<Vec<ClientStatus>> {
    let native = native_minecraft_profile();

    let mut vm_identities = Vec::with_capacity(3);
    let mut windows_identities = Vec::with_capacity(3);
    let mut guests = Vec::with_capacity(3);

    for client in ClientId::VIRTUAL {
        require_client_matches_native(client)?;
        if provider.status(client)? != ClientState::Running {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "all Virtual instances must be RUNNING before verify-identities",
            ));
        }

        let vm_identity = provider.identity_key(client)?.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{} VM identity is unavailable", client.as_str()),
            )
        })?;
        let guest = wait_for_guest_compatibility(provider, client, Duration::from_secs(30), false)?;
        let windows_identity = guest.machine_identity.clone().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{} Windows identity is unavailable", client.as_str()),
            )
        })?;

        vm_identities.push((client, vm_identity));
        windows_identities.push((client, windows_identity));
        guests.push((client, guest));
    }

    for left in 0..ClientId::VIRTUAL.len() {
        for right in (left + 1)..ClientId::VIRTUAL.len() {
            if classify_identity_state(
                Some(vm_identities[left].1.as_str()),
                [Some(vm_identities[right].1.as_str())],
            ) != IdentityState::Unique
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "{} and {} do not have independently unique VM UUID and MAC identities",
                        vm_identities[left].0.as_str(),
                        vm_identities[right].0.as_str()
                    ),
                ));
            }
            if windows_identities[left].1 == windows_identities[right].1 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "{} and {} share the same Windows identity",
                        windows_identities[left].0.as_str(),
                        windows_identities[right].0.as_str()
                    ),
                ));
            }
        }
    }

    for index in 0..ClientId::VIRTUAL.len() {
        write_verified_client_identities(
            ClientId::VIRTUAL[index],
            &identity_fingerprint(&vm_identities[index].1),
            &windows_identities[index].1,
        )?;
    }

    let working_sets = provider.host_working_sets_mb()?;
    let mut result = Vec::with_capacity(3);
    for (client, _) in guests {
        result.push(client_status(
            provider,
            &working_sets,
            native.as_ref(),
            client,
        )?);
    }
    Ok(result)
}

fn working_set_for(working_sets: &[(ClientId, u64)], client: ClientId) -> Option<u64> {
    working_sets
        .iter()
        .find_map(|(candidate, memory_mb)| (*candidate == client).then_some(*memory_mb))
}

fn client_status_from_observations(
    provider: &dyn Provider,
    working_sets: &[(ClientId, u64)],
    native: Option<&MinecraftProfile>,
    client: ClientId,
    observations: &[ClientObservation],
) -> io::Result<ClientStatus> {
    let observation = observation_for(observations, client)?;
    let state = observation.state;
    if state == ClientState::NotProvisioned {
        return Ok(ClientStatus {
            id: client.as_str(),
            native: false,
            state,
            ready_snapshot: Some(false),
            memory_limit_mb: None,
            host_working_set_mb: None,
            guest_tools_ready: None,
            guest_agent_ready: None,
            guest_agent_version: None,
            minecraft_version: None,
            minecraft_running: None,
            interactive_launcher_ready: None,
            connection_health: None,
            lineage_parity: Some(ProfileParity::Unknown),
            version_parity: Some(ProfileParity::Unknown),
            vm_identity: Some(IdentityState::Unknown),
            windows_identity: Some(IdentityState::Unknown),
        });
    }

    let guest = observation.guest.as_ref();
    let minecraft_version = guest
        .and_then(|status| status.minecraft.as_ref())
        .map(|minecraft| minecraft.version.clone());
    let parity = version_parity(native, guest);

    let guest_tools_ready = provider.guest_tools_ready(client).ok().flatten();
    let connection_health = connection_health(state, guest_tools_ready, guest);
    Ok(ClientStatus {
        id: client.as_str(),
        native: false,
        state,
        ready_snapshot: provider.has_ready(client).ok(),
        memory_limit_mb: provider.memory_limit_mb(client).ok(),
        host_working_set_mb: working_set_for(working_sets, client),
        guest_tools_ready,
        guest_agent_ready: guest.map(|_| true),
        guest_agent_version: guest.map(|status| status.agent_version.clone()),
        minecraft_version,
        minecraft_running: guest.and_then(|status| status.minecraft_running),
        interactive_launcher_ready: guest.and_then(|status| status.interactive_launcher_ready),
        connection_health,
        lineage_parity: Some(lineage_parity(native, client)),
        version_parity: Some(parity),
        vm_identity: Some(vm_identity_state_from_observations(client, observations)),
        windows_identity: Some(windows_identity_state(client, observations)),
    })
}

fn connection_health(
    state: ClientState,
    guest_tools_ready: Option<bool>,
    guest: Option<&GuestStatus>,
) -> Option<ConnectionHealth> {
    if state != ClientState::Running {
        return (!matches!(state, ClientState::NotProvisioned | ClientState::Error))
            .then_some(ConnectionHealth::VmOffline);
    }
    if guest.and_then(|status| status.minecraft_running) == Some(true) {
        return Some(ConnectionHealth::MinecraftRunning);
    }
    if guest.and_then(|status| status.interactive_launcher_ready) == Some(true) {
        return Some(ConnectionHealth::InteractiveLauncherReady);
    }
    if guest.is_some() {
        return Some(ConnectionHealth::GuestAgentReady);
    }
    if guest_tools_ready == Some(true) {
        return Some(ConnectionHealth::GuestToolsReady);
    }
    Some(ConnectionHealth::VmRunning)
}

fn client_status(
    provider: &dyn Provider,
    working_sets: &[(ClientId, u64)],
    native: Option<&MinecraftProfile>,
    client: ClientId,
) -> io::Result<ClientStatus> {
    let observations = collect_client_observations(provider)?;
    client_status_from_observations(provider, working_sets, native, client, &observations)
}

impl VirtualClients {
    pub fn doctor(&self) -> DoctorReport {
        doctor()
    }

    pub fn policy(&self) -> EnginePolicy {
        engine_policy()
    }

    pub fn lifecycle_actions(&self) -> io::Result<Vec<ClientLifecycleActions>> {
        let _lock = OperationLock::acquire_shared()?;
        let Some(provider) = current_platform_provider() else {
            return Ok(ClientId::VIRTUAL
                .into_iter()
                .map(|client| {
                    let blocked = || ActionAvailability {
                        allowed: false,
                        blocker: Some(LifecycleBlocker::ProviderUnavailable),
                        reason: Some("Virtualization provider is unavailable."),
                    };
                    ClientLifecycleActions {
                        id: client.as_str(),
                        start: blocked(),
                        start_setup: Some(blocked()),
                        suspend: blocked(),
                        stop: blocked(),
                        open: blocked(),
                        restart: blocked(),
                        set_ready: blocked(),
                        reset: blocked(),
                        reprovision: blocked(),
                    }
                })
                .collect());
        };

        let can_start_virtual = current_host_pressure().start_decision != StartDecision::Block;
        ClientId::VIRTUAL
            .into_iter()
            .map(|client| {
                let facts = collect_lifecycle_facts(provider.as_ref(), client, can_start_virtual)?;
                let action = |kind| evaluate_lifecycle_admission(client, kind, &facts);
                Ok(ClientLifecycleActions {
                    id: client.as_str(),
                    start: action(LifecycleAction::Start),
                    start_setup: facts
                        .fresh_client_profile
                        .then(|| action(LifecycleAction::StartSetup)),
                    suspend: action(LifecycleAction::Suspend),
                    stop: action(LifecycleAction::Stop),
                    open: action(LifecycleAction::Open),
                    restart: action(LifecycleAction::Restart),
                    set_ready: action(LifecycleAction::SetReady),
                    reset: action(LifecycleAction::Reset),
                    reprovision: action(LifecycleAction::Reprovision),
                })
            })
            .collect()
    }

    fn record<T>(
        &self,
        operation: OperationKind,
        target: Option<String>,
        result: io::Result<T>,
    ) -> io::Result<T> {
        record_operation(operation, target, &result);
        result
    }

    pub fn stage_update(&self) -> io::Result<StagedUpdate> {
        self.record(OperationKind::StageUpdate, None, self.stage_update_inner())
    }

    pub fn register_base(&self) -> io::Result<BaseProfile> {
        self.record(
            OperationKind::RegisterBase,
            None,
            self.register_base_inner(),
        )
    }

    pub fn open_base_for_finalization(&self) -> io::Result<()> {
        self.record(
            OperationKind::OpenBaseFinalization,
            Some("Base".to_string()),
            self.open_base_for_finalization_inner(),
        )
    }

    pub fn provision(&self) -> io::Result<Vec<ClientStatus>> {
        self.record(OperationKind::Provision, None, self.provision_inner())
    }

    pub fn reprovision(
        &self,
        client: ClientId,
        confirmation: DestructiveConfirmation,
    ) -> io::Result<ClientStatus> {
        self.record(
            OperationKind::Reprovision,
            Some(client.as_str().to_string()),
            self.reprovision_inner(client, confirmation),
        )
    }

    pub fn verify_identities(&self) -> io::Result<Vec<ClientStatus>> {
        self.record(
            OperationKind::VerifyIdentities,
            None,
            self.verify_identities_inner(),
        )
    }

    pub fn start(&self, count: usize) -> io::Result<Vec<ClientStatus>> {
        self.record(
            OperationKind::Start,
            Some(format!("count:{count}")),
            self.start_inner(count),
        )
    }

    pub fn start_setup(&self, client: ClientId) -> io::Result<ClientStatus> {
        self.record(
            OperationKind::Start,
            Some(format!("first-boot:{}", client.as_str())),
            self.start_setup_inner(client),
        )
    }

    pub fn start_client(&self, client: ClientId) -> io::Result<ClientStatus> {
        self.record(
            OperationKind::Start,
            Some(client.as_str().to_string()),
            self.start_client_inner(client),
        )
    }

    pub fn suspend(&self, client: Option<ClientId>) -> io::Result<Vec<ClientStatus>> {
        let target = client
            .map(|client| client.as_str().to_string())
            .or_else(|| Some("all".to_string()));
        self.record(OperationKind::Suspend, target, self.suspend_inner(client))
    }

    pub fn stop(&self, client: Option<ClientId>) -> io::Result<Vec<ClientStatus>> {
        let target = client
            .map(|client| client.as_str().to_string())
            .or_else(|| Some("all".to_string()));
        self.record(OperationKind::Stop, target, self.stop_inner(client))
    }

    pub fn restart(&self, client: ClientId) -> io::Result<ClientStatus> {
        self.record(
            OperationKind::Restart,
            Some(client.as_str().to_string()),
            self.restart_inner(client),
        )
    }

    pub fn set_ready(&self, client: ClientId) -> io::Result<ClientStatus> {
        self.record(
            OperationKind::SetReady,
            Some(client.as_str().to_string()),
            self.set_ready_inner(client),
        )
    }

    pub fn reset(&self, client: ClientId) -> io::Result<ClientStatus> {
        self.record(
            OperationKind::Reset,
            Some(client.as_str().to_string()),
            self.reset_inner(client),
        )
    }

    pub fn launch_minecraft(&self, client: ClientId) -> io::Result<ClientStatus> {
        self.record(
            OperationKind::LaunchMinecraft,
            Some(client.as_str().to_string()),
            self.launch_minecraft_inner(client),
        )
    }

    pub fn open(&self, client: ClientId) -> io::Result<ClientStatus> {
        self.record(
            OperationKind::Open,
            Some(client.as_str().to_string()),
            self.open_inner(client),
        )
    }

    pub fn check_update(&self) -> io::Result<UpdateCheck> {
        check_update()
    }

    fn stage_update_inner(&self) -> io::Result<StagedUpdate> {
        let _lock = OperationLock::acquire()?;
        stage_update()
    }

    pub fn diagnostics(&self) -> io::Result<DiagnosticsReport> {
        let _lock = OperationLock::acquire_shared()?;
        collect_diagnostics(self.status_unlocked()?)
    }

    pub fn snapshot(&self) -> io::Result<EngineSnapshot> {
        let _lock = OperationLock::acquire_shared()?;
        let captured_at_unix_ms = capture_time_ms()?;
        let runtime = self.status_unlocked()?;
        let diagnostics = collect_diagnostics(runtime)?;
        Ok(EngineSnapshot {
            captured_at_unix_ms,
            doctor: self.doctor(),
            diagnostics,
        })
    }

    pub fn support_bundle(&self) -> io::Result<SupportBundleResult> {
        write_support_bundle(self.snapshot()?)
    }

    fn register_base_inner(&self) -> io::Result<BaseProfile> {
        let _lock = OperationLock::acquire()?;
        let provider = current_platform_provider().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "virtualization provider is unavailable",
            )
        })?;
        let native = native_minecraft_profile().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "Native Minecraft Education version could not be detected",
            )
        })?;
        let base = current_base_vmx_path()?;
        if !vmx_file_present(&base)? {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Base VM is missing: {}", base.display()),
            ));
        }
        if provider.is_running_path(&base)? {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Base must be stopped before live verification",
            ));
        }

        match base_state_for_path(&base)? {
            Some(BaseState::Finalized) => {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "Base is already FINALIZED and must not be booted for re-registration",
                ))
            }
            Some(BaseState::Finalizing) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Base is in FINALIZING state; inspect the Base before continuing",
                ))
            }
            _ => {}
        }

        let token = ensure_guest_token_for_path(&base)?;
        provider.start_validation_vm(&base)?;

        let proof = (|| -> io::Result<GuestStatus> {
            let started = std::time::Instant::now();
            loop {
                if let Some(ip) = provider.guest_ip_for_path(&base)? {
                    match query_guest_status(&ip, &token, Duration::from_secs(2)) {
                        Ok(status) => {
                            // Do not mark a Base REGISTERED if its Agent can
                            // report status but cannot launch the actual game.
                            require_base_launch_protocol(status.protocol_version)?;

                            let minecraft = status.minecraft.as_ref().ok_or_else(|| {
                                io::Error::new(
                                    io::ErrorKind::NotFound,
                                    "Base Guest Agent cannot detect Minecraft Education",
                                )
                            })?;
                            if minecraft.version != native.version {
                                return Err(io::Error::new(
                                    io::ErrorKind::InvalidData,
                                    format!(
                                        "Base Minecraft Education version {} does not match Native {}",
                                        minecraft.version, native.version
                                    ),
                                ));
                            }
                            return Ok(status);
                        }
                        Err(error) if guest_probe_error_is_terminal(&error) => {
                            return Err(io::Error::new(
                                error.kind(),
                                format!("Base Guest Agent probe failed: {error}"),
                            ));
                        }
                        Err(_) => {}
                    }
                }

                if started.elapsed() >= Duration::from_secs(120) {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "Base Guest Agent did not become ready within 120s",
                    ));
                }
                thread::sleep(Duration::from_secs(1));
            }
        })();

        let stop_result = provider.stop_validation_vm(&base);

        let proof = match (proof, stop_result) {
            (Ok(proof), Ok(())) => proof,
            (Err(error), Ok(())) => return Err(error),
            (Ok(_), Err(stop_error)) => {
                return Err(io::Error::new(
                    stop_error.kind(),
                    format!("Base proof succeeded but Base could not be stopped: {stop_error}"),
                ))
            }
            (Err(proof_error), Err(stop_error)) => {
                return Err(io::Error::new(
                    proof_error.kind(),
                    format!("{proof_error}; Base stop also failed: {stop_error}"),
                ))
            }
        };

        set_base_state_for_path(&base, BaseState::Registered)?;
        write_verified_base_profile(&native, proof.protocol_version)
    }

    fn open_base_for_finalization_inner(&self) -> io::Result<()> {
        require_base_matches_native()?;
        let _lock = OperationLock::acquire()?;
        let provider = current_platform_provider().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "virtualization provider is unavailable",
            )
        })?;
        let base = current_base_vmx_path()?;
        if !vmx_file_present(&base)? {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Base VM is missing: {}", base.display()),
            ));
        }
        require_base_finalization_state(base_state_for_path(&base)?)?;

        provider.start_validation_vm(&base)?;
        if let Err(open_error) = provider.open_vm_ui(&base) {
            let stop_error = provider.stop_validation_vm(&base).err();
            return Err(io::Error::new(
                open_error.kind(),
                match stop_error {
                    Some(stop_error) => format!(
                        "Base started but VMware UI could not open: {open_error}; rollback stop also failed: {stop_error}"
                    ),
                    None => format!("VMware UI could not open for Base finalization: {open_error}"),
                },
            ));
        }

        Ok(())
    }

    fn provision_inner(&self) -> io::Result<Vec<ClientStatus>> {
        let profile = require_base_matches_native()?;
        let native = profile.native.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "Native Minecraft Education version could not be detected",
            )
        })?;
        let base_generation_id = profile
            .base
            .as_ref()
            .map(|base| base.base_generation_id.clone())
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "registered Base generation is unavailable",
                )
            })?;
        let base = current_base_vmx_path()?;
        if base_state_for_path(&base)? != Some(BaseState::Finalized) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Base must be FINALIZED with finalize-base.ps1 before provisioning",
            ));
        }

        let _lock = OperationLock::acquire()?;
        cleanup_staging()?;
        let provider = current_platform_provider().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "virtualization provider is unavailable",
            )
        })?;

        // Full-target preflight before creating the first missing Virtual.
        let missing = missing_provision_targets(provider.as_ref(), |client| {
            require_client_matches_native(client).map(|_| ())
        })?;
        for client in missing {
            provider.provision(client)?;
            if let Err(error) = write_client_profile(client, &native.version, &base_generation_id) {
                return Err(io::Error::new(
                    error.kind(),
                    format!(
                        "{} VM was created but its lineage profile was not committed; existing VM files were preserved and further provisioning is blocked until recovery: {error}",
                        client.as_str()
                    ),
                ));
            }
        }

        let working_sets = provider.host_working_sets_mb()?;
        let native_profile = native_minecraft_profile();
        let mut result = Vec::with_capacity(3);
        for client in ClientId::VIRTUAL {
            result.push(client_status(
                provider.as_ref(),
                &working_sets,
                native_profile.as_ref(),
                client,
            )?);
        }
        Ok(result)
    }

    fn reprovision_inner(
        &self,
        client: ClientId,
        confirmation: DestructiveConfirmation,
    ) -> io::Result<ClientStatus> {
        if client.is_native() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Native cannot be reprovisioned",
            ));
        }
        if confirmation != DestructiveConfirmation::ReprovisionAccountState {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "reprovision requires explicit account-state destruction confirmation",
            ));
        }
        // This operation previously removed the saved profile and the entire
        // account-bearing VM before the new clone existed. Until a recoverable
        // cutover is implemented, reject it without touching disks or profiles.
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Reprovision is temporarily disabled until VM and account data can be preserved on failure.",
        ))
    }

    fn verify_identities_inner(&self) -> io::Result<Vec<ClientStatus>> {
        require_base_matches_native()?;
        let _lock = OperationLock::acquire()?;
        let provider = current_platform_provider().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "virtualization provider is unavailable",
            )
        })?;
        verify_identity_provenance(provider.as_ref())
    }

    pub fn status(&self) -> io::Result<RuntimeStatus> {
        let _lock = OperationLock::acquire_shared()?;
        self.status_unlocked()
    }

    fn status_unlocked(&self) -> io::Result<RuntimeStatus> {
        let provider = current_platform_provider();
        let mut clients = Vec::with_capacity(ClientId::ALL.len());

        let native_profile = native_minecraft_profile();
        clients.push(ClientStatus {
            id: ClientId::Native.as_str(),
            native: true,
            state: ClientState::Manual,
            ready_snapshot: None,
            memory_limit_mb: None,
            host_working_set_mb: None,
            guest_tools_ready: None,
            guest_agent_ready: None,
            guest_agent_version: None,
            minecraft_version: native_profile
                .as_ref()
                .map(|profile| profile.version.clone()),
            minecraft_running: None,
            interactive_launcher_ready: None,
            connection_health: None,
            lineage_parity: None,
            version_parity: None,
            vm_identity: None,
            windows_identity: None,
        });

        if let Some(provider) = provider.as_ref() {
            let working_sets = provider.host_working_sets_mb()?;
            let observations = collect_client_observations(provider.as_ref())?;
            for client in ClientId::VIRTUAL {
                clients.push(client_status_from_observations(
                    provider.as_ref(),
                    &working_sets,
                    native_profile.as_ref(),
                    client,
                    &observations,
                )?);
            }
        } else {
            for client in ClientId::VIRTUAL {
                clients.push(ClientStatus {
                    id: client.as_str(),
                    native: false,
                    state: ClientState::Error,
                    ready_snapshot: Some(false),
                    memory_limit_mb: None,
                    host_working_set_mb: None,
                    guest_tools_ready: None,
                    guest_agent_ready: None,
                    guest_agent_version: None,
                    minecraft_version: None,
                    minecraft_running: None,
                    interactive_launcher_ready: None,
                    connection_health: None,
                    lineage_parity: Some(ProfileParity::Unknown),
                    version_parity: Some(ProfileParity::Unknown),
                    vm_identity: Some(IdentityState::Unknown),
                    windows_identity: Some(IdentityState::Unknown),
                });
            }
        }

        Ok(RuntimeStatus {
            provider: provider.as_ref().map(|provider| provider.id()),
            runtime_profile: profile_status(),
            pressure: current_host_pressure(),
            clients,
        })
    }

    pub fn resources(&self, count: usize) -> io::Result<ResourceView> {
        if !(1..=MAX_VIRTUAL_CLIENTS).contains(&count) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "virtual client count must be within the configured engine policy",
            ));
        }

        let _lock = OperationLock::acquire_shared()?;
        let capacity = current_host_capacity();

        let provider = current_platform_provider().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "virtualization provider is unavailable",
            )
        })?;

        let mut running = 0;
        let mut suspended = 0;
        let mut stopped = 0;
        let working_sets = provider.host_working_sets_mb()?;
        let mut observed_working_set_mb = 0;
        let mut observed_working_set_instances = 0;

        for client in ClientId::VIRTUAL.into_iter().take(count) {
            match provider.status(client)? {
                ClientState::Running => running += 1,
                ClientState::Suspended => suspended += 1,
                ClientState::Stopped => stopped += 1,
                ClientState::NotProvisioned => {
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        format!("{} is not provisioned", client.as_str()),
                    ));
                }
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("{} is not in a resource-manageable state", client.as_str()),
                    ));
                }
            }

            if let Some(memory_mb) = working_set_for(&working_sets, client) {
                observed_working_set_mb += memory_mb;
                observed_working_set_instances += 1;
            }
        }

        Ok(ResourceView {
            requested_virtual_clients: count,
            recommended_virtual_clients: capacity.recommended_virtual_clients,
            running_virtual_clients: running,
            suspended_virtual_clients: suspended,
            stopped_virtual_clients: stopped,
            virtual_memory_limit_mb: VIRTUAL_MEMORY_LIMIT_MB,
            observed_working_set_mb,
            observed_working_set_instances,
            pressure: current_host_pressure(),
        })
    }

    fn start_inner(&self, count: usize) -> io::Result<Vec<ClientStatus>> {
        if !(1..=MAX_VIRTUAL_CLIENTS).contains(&count) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "virtual client count must be within the configured engine policy",
            ));
        }
        let targets: Vec<ClientId> = ClientId::VIRTUAL.into_iter().take(count).collect();
        self.start_targets(targets)
    }

    fn start_client_inner(&self, client: ClientId) -> io::Result<ClientStatus> {
        if client.is_native() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Native lifecycle is managed manually on the host",
            ));
        }
        let mut result = self.start_targets(vec![client])?;
        result.pop().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Other,
                "selected Virtual start returned no client status",
            )
        })
    }

    fn start_setup_inner(&self, client: ClientId) -> io::Result<ClientStatus> {
        let _lock = OperationLock::acquire()?;
        let provider = current_platform_provider().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "virtualization provider is unavailable",
            )
        })?;
        check_action_admission(provider.as_ref(), client, LifecycleAction::StartSetup)?;
        // Reuse the same mutation/rollback owner. Missing Guest Agent during
        // interactive Windows first boot is not a failed daily-start verification.
        let state = execute_setup_start(provider.as_ref(), client, |client, original_state| {
            check_action_admission(provider.as_ref(), client, LifecycleAction::StartSetup)?;
            if original_state != ClientState::Running {
                wait_until_start_is_safe()?;
            }
            Ok(())
        })?;
        // Return power observations only. Do not write identity provenance or
        // claim guest/version readiness before verify-identities completes.
        Ok(ClientStatus {
            id: client.as_str(),
            native: false,
            state,
            ready_snapshot: Some(false),
            memory_limit_mb: provider.memory_limit_mb(client).ok(),
            host_working_set_mb: None,
            guest_tools_ready: None,
            guest_agent_ready: None,
            guest_agent_version: None,
            minecraft_version: None,
            minecraft_running: None,
            interactive_launcher_ready: None,
            connection_health: None,
            lineage_parity: Some(ProfileParity::Match),
            version_parity: Some(ProfileParity::Unknown),
            vm_identity: Some(vm_identity_state(provider.as_ref(), client)),
            windows_identity: Some(IdentityState::Unknown),
        })
    }

    fn start_targets(&self, targets: Vec<ClientId>) -> io::Result<Vec<ClientStatus>> {
        let _lock = OperationLock::acquire()?;
        let provider = current_platform_provider().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "virtualization provider is unavailable",
            )
        })?;

        // Preflight and live admission use the same policy as UI availability.
        for &client in &targets {
            check_action_admission(provider.as_ref(), client, LifecycleAction::Start)?;
        }

        let native_profile = native_minecraft_profile();
        // Keep game launch and window opening inside the same rollback boundary
        // as VM startup. A pre-existing RUNNING client is never stopped here.
        execute_start_batch(
            provider.as_ref(),
            &targets,
            |client, original_state| {
                check_action_admission(provider.as_ref(), client, LifecycleAction::Start)?;
                // Apply one bounded resource wait to every new/resumed start,
                // including single-client calls from the desktop Start All.
                if original_state != ClientState::Running {
                    wait_until_start_is_safe()?;
                }
                Ok(())
            },
            |client| {
                check_action_admission(provider.as_ref(), client, LifecycleAction::Start)?;
                wait_for_guest_compatibility(
                    provider.as_ref(),
                    client,
                    Duration::from_secs(90),
                    true,
                )?;
                crate::minecraft_runtime::ensure_running(provider.as_ref(), client).map_err(
                    |error| {
                        io::Error::new(
                            error.kind(),
                            format!(
                                "{} VM started, but Minecraft Education launch failed: {error}",
                                client.as_str()
                            ),
                        )
                    },
                )?;
                provider.open(client).map_err(|error| {
                    io::Error::new(error.kind(), format!(
                        "{} Minecraft launch was requested, but VMware window could not be opened: {error}",
                        client.as_str()
                    ))
                })?;
                let working_sets = provider.host_working_sets_mb()?;
                let status = client_status(
                    provider.as_ref(),
                    &working_sets,
                    native_profile.as_ref(),
                    client,
                )?;
                Ok(status)
            },
        )
    }

    fn suspend_inner(&self, client: Option<ClientId>) -> io::Result<Vec<ClientStatus>> {
        if client.is_some_and(ClientId::is_native) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Native cannot be suspended by Virtual Clients",
            ));
        }

        let _lock = OperationLock::acquire()?;
        let provider = current_platform_provider().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "virtualization provider is unavailable",
            )
        })?;

        let targets: Vec<ClientId> = match client {
            Some(client) => vec![client],
            None => ClientId::VIRTUAL
                .into_iter()
                .filter(|client| provider.status(*client).ok() != Some(ClientState::NotProvisioned))
                .collect(),
        };

        let mut result = Vec::with_capacity(targets.len());
        suspend_targets(provider.as_ref(), &targets, |client| {
            check_action_admission(provider.as_ref(), client, LifecycleAction::Suspend)
        })?;

        let working_sets = provider.host_working_sets_mb()?;
        let native_profile = native_minecraft_profile();
        for client in targets {
            result.push(client_status(
                provider.as_ref(),
                &working_sets,
                native_profile.as_ref(),
                client,
            )?);
        }
        Ok(result)
    }

    fn stop_inner(&self, client: Option<ClientId>) -> io::Result<Vec<ClientStatus>> {
        if client.is_some_and(ClientId::is_native) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Native lifecycle is managed manually on the host",
            ));
        }

        let _lock = OperationLock::acquire()?;
        let provider = current_platform_provider().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "virtualization provider is unavailable",
            )
        })?;

        let targets: Vec<ClientId> = match client {
            Some(client) => vec![client],
            None => ClientId::VIRTUAL
                .into_iter()
                .filter(|client| provider.status(*client).ok() != Some(ClientState::NotProvisioned))
                .collect(),
        };

        let mut result = Vec::with_capacity(targets.len());
        stop_targets(provider.as_ref(), &targets, |client| {
            check_action_admission(provider.as_ref(), client, LifecycleAction::Stop)
        })?;

        let working_sets = provider.host_working_sets_mb()?;
        let native_profile = native_minecraft_profile();
        for client in targets {
            result.push(client_status(
                provider.as_ref(),
                &working_sets,
                native_profile.as_ref(),
                client,
            )?);
        }
        Ok(result)
    }

    fn restart_inner(&self, client: ClientId) -> io::Result<ClientStatus> {
        if client.is_native() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Native cannot be VM-restarted",
            ));
        }

        let _lock = OperationLock::acquire()?;
        let provider = current_platform_provider().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "virtualization provider is unavailable",
            )
        })?;
        check_action_admission(provider.as_ref(), client, LifecycleAction::Restart)?;
        provider.restart(client).map_err(|error| {
            observed_post_action_failure(provider.as_ref(), client, "Restart", error)
        })?;
        if let Err(error) =
            wait_for_guest_compatibility(provider.as_ref(), client, Duration::from_secs(90), true)
        {
            return Err(observed_post_action_failure(
                provider.as_ref(),
                client,
                "Restart",
                error,
            ));
        }
        if let Err(error) = crate::minecraft_runtime::ensure_running(provider.as_ref(), client) {
            return Err(io::Error::new(
                error.kind(),
                format!(
                    "{} is running, but Minecraft Education could not be opened: {error}",
                    client.as_str()
                ),
            ));
        }
        if let Err(error) = provider.open(client) {
            return Err(io::Error::new(
                error.kind(),
                format!("{} and Minecraft Education are running, but the client window could not be opened: {error}", client.as_str()),
            ));
        }
        let working_sets = provider.host_working_sets_mb()?;
        let native_profile = native_minecraft_profile();
        client_status(
            provider.as_ref(),
            &working_sets,
            native_profile.as_ref(),
            client,
        )
    }

    fn set_ready_inner(&self, client: ClientId) -> io::Result<ClientStatus> {
        if client.is_native() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Native does not use QA_READY snapshots",
            ));
        }

        let _lock = OperationLock::acquire()?;
        let provider = current_platform_provider().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "virtualization provider is unavailable",
            )
        })?;
        check_action_admission(provider.as_ref(), client, LifecycleAction::SetReady)?;
        provider.set_ready(client)?;
        let working_sets = provider.host_working_sets_mb()?;
        let native_profile = native_minecraft_profile();
        client_status(
            provider.as_ref(),
            &working_sets,
            native_profile.as_ref(),
            client,
        )
    }

    fn reset_inner(&self, client: ClientId) -> io::Result<ClientStatus> {
        if client.is_native() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Native does not use VM clean-state reset",
            ));
        }

        let _lock = OperationLock::acquire()?;
        let provider = current_platform_provider().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "virtualization provider is unavailable",
            )
        })?;
        check_action_admission(provider.as_ref(), client, LifecycleAction::Reset)?;
        provider.reset(client).map_err(|error| {
            observed_post_action_failure(provider.as_ref(), client, "Reset", error)
        })?;
        if let Err(error) =
            wait_for_guest_compatibility(provider.as_ref(), client, Duration::from_secs(90), true)
        {
            return Err(observed_post_action_failure(
                provider.as_ref(),
                client,
                "Reset",
                error,
            ));
        }
        if let Err(error) = crate::minecraft_runtime::ensure_running(provider.as_ref(), client) {
            return Err(io::Error::new(
                error.kind(),
                format!(
                    "{} is running, but Minecraft Education could not be opened: {error}",
                    client.as_str()
                ),
            ));
        }
        if let Err(error) = provider.open(client) {
            return Err(io::Error::new(
                error.kind(),
                format!("{} and Minecraft Education are running, but the client window could not be opened: {error}", client.as_str()),
            ));
        }
        let working_sets = provider.host_working_sets_mb()?;
        let native_profile = native_minecraft_profile();
        client_status(
            provider.as_ref(),
            &working_sets,
            native_profile.as_ref(),
            client,
        )
    }

    fn launch_minecraft_inner(&self, client: ClientId) -> io::Result<ClientStatus> {
        if client.is_native() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Native Minecraft is launched by the desktop app",
            ));
        }
        let _lock = OperationLock::acquire()?;
        let provider = current_platform_provider().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "virtualization provider is unavailable",
            )
        })?;
        if provider.status(client)? != ClientState::Running {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Virtual client must be running before Minecraft can be launched",
            ));
        }
        crate::minecraft_runtime::ensure_running(provider.as_ref(), client)?;
        let working_sets = provider.host_working_sets_mb()?;
        let native_profile = native_minecraft_profile();
        client_status(
            provider.as_ref(),
            &working_sets,
            native_profile.as_ref(),
            client,
        )
    }

    fn open_inner(&self, client: ClientId) -> io::Result<ClientStatus> {
        if client.is_native() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Native is opened manually on the host",
            ));
        }

        let _lock = OperationLock::acquire()?;
        let provider = current_platform_provider().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "virtualization provider is unavailable",
            )
        })?;
        check_action_admission(provider.as_ref(), client, LifecycleAction::Open)?;
        provider.open(client)?;
        let working_sets = provider.host_working_sets_mb()?;
        let native_profile = native_minecraft_profile();
        client_status(
            provider.as_ref(),
            &working_sets,
            native_profile.as_ref(),
            client,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{
        classify_identity_state, connection_health, guest_probe_error_is_terminal,
        missing_provision_targets, restore_batch_state, stop_targets, suspend_targets,
        validate_power_state, vm_identity_state, ConnectionHealth, LifecycleAction,
    };
    use crate::profile::BaseState;
    use crate::{
        client::{ClientId, ClientState, IdentityState},
        guest::GuestStatus,
        provider::Provider,
    };
    use std::{cell::RefCell, io, path::Path};

    #[derive(Debug)]
    struct FakeProvider {
        states: RefCell<[ClientState; 3]>,
        identities: [Option<&'static str>; 3],
        fail_start: Option<ClientId>,
        fail_after_start: bool,
        fail_open: bool,
        fail_stop: Option<ClientId>,
        fail_suspend: Option<ClientId>,
        fail_after_suspend: bool,
        identity_error: Option<ClientId>,
    }

    impl FakeProvider {
        fn new(states: [ClientState; 3], identities: [Option<&'static str>; 3]) -> Self {
            Self {
                states: RefCell::new(states),
                identities,
                fail_start: None,
                fail_after_start: false,
                fail_open: false,
                fail_stop: None,
                fail_suspend: None,
                fail_after_suspend: false,
                identity_error: None,
            }
        }

        fn index(client: ClientId) -> io::Result<usize> {
            match client {
                ClientId::Virtual01 => Ok(0),
                ClientId::Virtual02 => Ok(1),
                ClientId::Virtual03 => Ok(2),
                ClientId::Native => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Native has no provider state",
                )),
            }
        }

        fn state(&self, client: ClientId) -> io::Result<ClientState> {
            Ok(self.states.borrow()[Self::index(client)?])
        }

        fn set_state(&self, client: ClientId, state: ClientState) -> io::Result<ClientState> {
            self.states.borrow_mut()[Self::index(client)?] = state;
            Ok(state)
        }

        fn unsupported<T>() -> io::Result<T> {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "not used by runtime simulation",
            ))
        }
    }

    impl Provider for FakeProvider {
        fn id(&self) -> &'static str {
            "fake"
        }

        fn version(&self) -> Option<String> {
            Some("test".into())
        }

        fn detect(&self) -> bool {
            true
        }

        fn provision(&self, _client: ClientId) -> io::Result<ClientState> {
            Self::unsupported()
        }

        fn reprovision(&self, _client: ClientId) -> io::Result<ClientState> {
            Self::unsupported()
        }

        fn memory_limit_mb(&self, _client: ClientId) -> io::Result<u64> {
            Ok(4096)
        }

        fn host_working_sets_mb(&self) -> io::Result<Vec<(ClientId, u64)>> {
            Ok(Vec::new())
        }

        fn base_finalized_and_stopped(&self) -> io::Result<bool> {
            Ok(false)
        }

        fn network_mode(&self, _client: ClientId) -> io::Result<Option<String>> {
            Ok(None)
        }

        fn graphics_3d_enabled(&self, _client: ClientId) -> io::Result<Option<bool>> {
            Ok(None)
        }

        fn guest_tools_ready(&self, _client: ClientId) -> io::Result<Option<bool>> {
            Ok(Some(true))
        }

        fn guest_status(
            &self,
            _client: ClientId,
            _timeout: std::time::Duration,
        ) -> io::Result<Option<crate::guest::GuestStatus>> {
            Ok(None)
        }

        fn launch_minecraft(
            &self,
            _client: ClientId,
            _timeout: std::time::Duration,
        ) -> io::Result<()> {
            Self::unsupported()
        }

        fn identity_key(&self, client: ClientId) -> io::Result<Option<String>> {
            if self.identity_error == Some(client) {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "injected identity read failure",
                ));
            }
            Ok(self.identities[Self::index(client)?].map(str::to_string))
        }

        fn status(&self, client: ClientId) -> io::Result<ClientState> {
            self.state(client)
        }

        fn start(&self, client: ClientId) -> io::Result<ClientState> {
            if self.fail_start == Some(client) {
                if self.fail_after_start {
                    self.set_state(client, ClientState::Running)?;
                }
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "injected start failure",
                ));
            }
            self.set_state(client, ClientState::Running)
        }

        fn suspend(&self, client: ClientId) -> io::Result<ClientState> {
            if self.fail_suspend == Some(client) {
                if self.fail_after_suspend {
                    self.set_state(client, ClientState::Suspended)?;
                }
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "injected suspend failure",
                ));
            }
            self.set_state(client, ClientState::Suspended)
        }

        fn stop(&self, client: ClientId) -> io::Result<ClientState> {
            if self.fail_stop == Some(client) {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "injected stop failure",
                ));
            }
            self.set_state(client, ClientState::Stopped)
        }

        fn restart(&self, client: ClientId) -> io::Result<ClientState> {
            self.set_state(client, ClientState::Running)
        }

        fn set_ready(&self, client: ClientId) -> io::Result<ClientState> {
            self.state(client)
        }

        fn has_ready(&self, _client: ClientId) -> io::Result<bool> {
            Ok(false)
        }

        fn reset(&self, client: ClientId) -> io::Result<ClientState> {
            self.set_state(client, ClientState::Running)
        }

        fn open(&self, client: ClientId) -> io::Result<ClientState> {
            if self.fail_open {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "injected UI open failure",
                ));
            }
            self.state(client)
        }

        fn open_vm_ui(&self, _vmx: &Path) -> io::Result<()> {
            Self::unsupported()
        }

        fn is_running_path(&self, _vmx: &Path) -> io::Result<bool> {
            Self::unsupported()
        }

        fn start_validation_vm(&self, _vmx: &Path) -> io::Result<()> {
            Self::unsupported()
        }

        fn stop_validation_vm(&self, _vmx: &Path) -> io::Result<()> {
            Self::unsupported()
        }

        fn guest_ip_for_path(&self, _vmx: &Path) -> io::Result<Option<String>> {
            Self::unsupported()
        }
    }

    #[test]
    fn selected_start_target_is_not_prefix_batch() {
        let selected = ClientId::Virtual02;
        let targets = vec![selected];
        assert_eq!(targets, vec![ClientId::Virtual02]);
        assert_ne!(
            targets,
            ClientId::VIRTUAL.into_iter().take(2).collect::<Vec<_>>()
        );
    }

    #[test]
    fn base_finalization_open_requires_registered_state() {
        assert!(super::require_base_finalization_state(Some(BaseState::Registered)).is_ok());
        assert!(super::require_base_finalization_state(Some(BaseState::Finalizing)).is_err());
        assert!(super::require_base_finalization_state(Some(BaseState::Finalized)).is_err());
        assert!(super::require_base_finalization_state(None).is_err());
    }

    #[test]
    fn guest_probe_protocol_errors_fail_fast() {
        for kind in [
            io::ErrorKind::InvalidData,
            io::ErrorKind::InvalidInput,
            io::ErrorKind::PermissionDenied,
            io::ErrorKind::Other,
        ] {
            assert!(guest_probe_error_is_terminal(&io::Error::new(
                kind, "terminal"
            )));
        }

        for kind in [
            io::ErrorKind::TimedOut,
            io::ErrorKind::ConnectionRefused,
            io::ErrorKind::WouldBlock,
            io::ErrorKind::NotConnected,
        ] {
            assert!(!guest_probe_error_is_terminal(&io::Error::new(
                kind, "retry"
            )));
        }
    }

    #[test]
    fn vm_identity_classification_is_fail_closed() {
        assert_eq!(
            classify_identity_state(
                Some("uuid-a|mac-a"),
                [Some("uuid-b|mac-b"), Some("uuid-c|mac-c")]
            ),
            IdentityState::Unique
        );
        assert_eq!(
            classify_identity_state(
                Some("uuid-a|mac-a"),
                [Some("uuid-a|mac-a"), Some("uuid-c|mac-c")]
            ),
            IdentityState::Duplicate
        );
        assert_eq!(
            classify_identity_state(Some("uuid-a|mac-a"), [Some("uuid-b|mac-b"), None]),
            IdentityState::Unknown
        );
        assert_eq!(
            classify_identity_state(None, [Some("uuid-b|mac-b"), Some("uuid-c|mac-c")]),
            IdentityState::Unknown
        );
    }

    #[test]
    fn fake_provider_proves_identity_states_without_vmware() {
        let provider = FakeProvider::new(
            [
                ClientState::Stopped,
                ClientState::Stopped,
                ClientState::Stopped,
            ],
            [
                Some("uuid-a|mac-a"),
                Some("uuid-b|mac-b"),
                Some("uuid-c|mac-c"),
            ],
        );
        assert_eq!(
            vm_identity_state(&provider, ClientId::Virtual01),
            IdentityState::Unique
        );

        let duplicate = FakeProvider::new(
            [
                ClientState::Stopped,
                ClientState::Stopped,
                ClientState::Stopped,
            ],
            [
                Some("uuid-a|mac-a"),
                Some("uuid-a|mac-a"),
                Some("uuid-c|mac-c"),
            ],
        );
        assert_eq!(
            vm_identity_state(&duplicate, ClientId::Virtual01),
            IdentityState::Duplicate
        );

        let unreadable = FakeProvider {
            identity_error: Some(ClientId::Virtual02),
            ..FakeProvider::new(
                [
                    ClientState::Stopped,
                    ClientState::Stopped,
                    ClientState::Stopped,
                ],
                [
                    Some("uuid-a|mac-a"),
                    Some("uuid-b|mac-b"),
                    Some("uuid-c|mac-c"),
                ],
            )
        };
        assert_eq!(
            vm_identity_state(&unreadable, ClientId::Virtual01),
            IdentityState::Unknown
        );
    }

    #[test]
    fn provisioning_preflight_rejects_bad_existing_profile_before_any_clone() {
        let provider = FakeProvider::new(
            [
                ClientState::NotProvisioned,
                ClientState::Stopped,
                ClientState::NotProvisioned,
            ],
            [None, None, None],
        );
        let error = missing_provision_targets(&provider, |client| {
            if client == ClientId::Virtual02 {
                Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid saved lineage",
                ))
            } else {
                Ok(())
            }
        })
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(error
            .to_string()
            .contains("no new clients were provisioned"));
        assert_eq!(
            *provider.states.borrow(),
            [
                ClientState::NotProvisioned,
                ClientState::Stopped,
                ClientState::NotProvisioned
            ]
        );
        assert_eq!(
            missing_provision_targets(&provider, |_| Ok(())).unwrap(),
            vec![ClientId::Virtual01, ClientId::Virtual03]
        );
    }

    #[test]
    fn stop_batch_reports_partial_completion_without_restarting_guests() {
        let provider = FakeProvider {
            fail_stop: Some(ClientId::Virtual02),
            ..FakeProvider::new(
                [
                    ClientState::Running,
                    ClientState::Running,
                    ClientState::Running,
                ],
                [None, None, None],
            )
        };
        let targets = [
            ClientId::Virtual01,
            ClientId::Virtual02,
            ClientId::Virtual03,
        ];
        let error = stop_targets(&provider, &targets, |_| Ok(())).unwrap_err();
        assert!(error.to_string().contains("Partial completion possible"));
        assert!(error.to_string().contains("Virtual-01=Stopped"));
        assert!(error.to_string().contains("Virtual-02=Running"));
        assert!(error.to_string().contains("Virtual-03=Running"));
        assert_eq!(
            provider.state(ClientId::Virtual01).unwrap(),
            ClientState::Stopped
        );
        assert_eq!(
            provider.state(ClientId::Virtual03).unwrap(),
            ClientState::Running
        );
    }

    #[test]
    fn stop_batch_admission_failure_preserves_unattempted_clients() {
        let provider = FakeProvider::new(
            [
                ClientState::Running,
                ClientState::Running,
                ClientState::Running,
            ],
            [None, None, None],
        );
        let targets = [
            ClientId::Virtual01,
            ClientId::Virtual02,
            ClientId::Virtual03,
        ];
        let error = stop_targets(&provider, &targets, |client| {
            if client == ClientId::Virtual02 {
                Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "injected admission failure",
                ))
            } else {
                Ok(())
            }
        })
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(
            *provider.states.borrow(),
            [ClientState::Running; 3],
            "preflight must not mutate any VM when target admission fails"
        );
    }

    #[test]
    fn suspended_stop_preflight_preserves_all_other_vm_sessions() {
        let provider = FakeProvider::new(
            [
                ClientState::Running,
                ClientState::Suspended,
                ClientState::Running,
            ],
            [None, None, None],
        );
        let result = stop_targets(&provider, &ClientId::VIRTUAL, |client| {
            validate_power_state(
                client,
                LifecycleAction::Stop,
                provider.state(client)?,
                false,
            )
        });
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidInput);
        assert_eq!(
            *provider.states.borrow(),
            [
                ClientState::Running,
                ClientState::Suspended,
                ClientState::Running
            ]
        );
    }

    #[test]
    fn suspend_batch_reports_partial_states_without_implicit_restart() {
        let targets = ClientId::VIRTUAL;
        // The provider can fail before OR after mutating the second VM.
        for mutated_before_error in [false, true] {
            let provider = FakeProvider {
                fail_suspend: Some(ClientId::Virtual02),
                fail_after_suspend: mutated_before_error,
                ..FakeProvider::new([ClientState::Running; 3], [None, None, None])
            };
            let error = suspend_targets(&provider, &targets, |_| Ok(())).unwrap_err();
            assert!(error
                .to_string()
                .contains("Suspend batch interrupted at Virtual-02"));
            assert!(error.to_string().contains("Partial completion possible"));
            assert!(error.to_string().contains("Virtual-01=Suspended"));
            assert!(error.to_string().contains(if mutated_before_error {
                "Virtual-02=Suspended"
            } else {
                "Virtual-02=Running"
            }));
            assert!(error.to_string().contains("Virtual-03=Running"));
            assert_eq!(
                provider.state(ClientId::Virtual01).unwrap(),
                ClientState::Suspended
            );
            assert_eq!(
                provider.state(ClientId::Virtual03).unwrap(),
                ClientState::Running
            );
        }
    }

    #[test]
    fn suspend_batch_admission_failure_preserves_unattempted_clients() {
        let provider = FakeProvider::new([ClientState::Running; 3], [None, None, None]);
        let error = suspend_targets(&provider, &ClientId::VIRTUAL, |client| {
            if client == ClientId::Virtual02 {
                Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "admission rejected",
                ))
            } else {
                Ok(())
            }
        })
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(
            *provider.states.borrow(),
            [ClientState::Running; 3],
            "suspend preflight must not mutate any VM on admission failure"
        );
    }

    #[test]
    fn post_action_failure_preserves_observed_vm_state() {
        let provider = FakeProvider::new(
            [
                ClientState::Running,
                ClientState::Suspended,
                ClientState::Stopped,
            ],
            [None, None, None],
        );
        for (client, action, state) in [
            (ClientId::Virtual01, "Restart", "Running"),
            (ClientId::Virtual02, "Reset", "Suspended"),
        ] {
            let error = super::observed_post_action_failure(
                &provider,
                client,
                action,
                io::Error::new(io::ErrorKind::TimedOut, "guest not ready"),
            );
            assert_eq!(error.kind(), io::ErrorKind::TimedOut);
            assert!(error
                .to_string()
                .contains(&format!("Observed VM state: {state}")));
            assert!(error.to_string().contains("No automatic stop attempted"));
        }
        assert_eq!(
            *provider.states.borrow(),
            [
                ClientState::Running,
                ClientState::Suspended,
                ClientState::Stopped
            ]
        );
    }

    #[test]
    fn fake_provider_proves_batch_rollback_to_original_states() {
        let provider = FakeProvider::new(
            [
                ClientState::Running,
                ClientState::Running,
                ClientState::Stopped,
            ],
            [
                Some("uuid-a|mac-a"),
                Some("uuid-b|mac-b"),
                Some("uuid-c|mac-c"),
            ],
        );

        let failed = restore_batch_state(
            &provider,
            &[
                (ClientId::Virtual01, ClientState::Stopped),
                (ClientId::Virtual02, ClientState::Suspended),
            ],
        );

        assert!(failed.is_empty());
        assert_eq!(
            provider.state(ClientId::Virtual01).unwrap(),
            ClientState::Stopped
        );
        assert_eq!(
            provider.state(ClientId::Virtual02).unwrap(),
            ClientState::Suspended
        );
        assert_eq!(
            provider.state(ClientId::Virtual03).unwrap(),
            ClientState::Stopped
        );
    }

    #[test]
    fn fake_provider_surfaces_incomplete_rollback_without_touching_other_clients() {
        let provider = FakeProvider {
            fail_stop: Some(ClientId::Virtual01),
            ..FakeProvider::new(
                [
                    ClientState::Running,
                    ClientState::Running,
                    ClientState::Stopped,
                ],
                [
                    Some("uuid-a|mac-a"),
                    Some("uuid-b|mac-b"),
                    Some("uuid-c|mac-c"),
                ],
            )
        };

        let failed = restore_batch_state(
            &provider,
            &[
                (ClientId::Virtual01, ClientState::Stopped),
                (ClientId::Virtual02, ClientState::Suspended),
            ],
        );

        assert_eq!(failed, vec!["Virtual-01"]);
        assert_eq!(
            provider.state(ClientId::Virtual01).unwrap(),
            ClientState::Running
        );
        assert_eq!(
            provider.state(ClientId::Virtual02).unwrap(),
            ClientState::Suspended
        );
        assert_eq!(
            provider.state(ClientId::Virtual03).unwrap(),
            ClientState::Stopped
        );
    }
    #[test]
    fn vm_identity_rejects_a_collision_in_either_component() {
        for peer in ["uuid-b|mac-a", "uuid-a|mac-b", "UUID-A|MAC-B"] {
            assert_eq!(
                classify_identity_state(Some("uuid-a|mac-a"), [Some(peer)]),
                IdentityState::Duplicate
            );
        }
        assert_eq!(
            classify_identity_state(Some("uuid-a|mac-a"), [Some("uuid-b|mac-b")]),
            IdentityState::Unique
        );
        for invalid in ["missing-separator", "|mac-a", "uuid-a|", "a|b|c"] {
            assert_eq!(
                classify_identity_state(Some(invalid), [Some("uuid-b|mac-b")]),
                IdentityState::Unknown
            );
        }
    }

    fn stopped_provider() -> FakeProvider {
        FakeProvider::new(
            [ClientState::Stopped; 3],
            [
                Some("uuid-a|mac-a"),
                Some("uuid-b|mac-b"),
                Some("uuid-c|mac-c"),
            ],
        )
    }

    #[test]
    fn start_batch_launch_failure_restores_new_vm_and_preserves_existing_running() {
        let provider = FakeProvider::new(
            [
                ClientState::Running,
                ClientState::Stopped,
                ClientState::Stopped,
            ],
            [None, None, None],
        );
        let error = super::execute_start_batch(
            &provider,
            &ClientId::VIRTUAL,
            |_, _| Ok(()),
            |client| {
                if client == ClientId::Virtual02 {
                    Err(io::Error::new(
                        io::ErrorKind::Other,
                        "injected Minecraft launch failure",
                    ))
                } else {
                    provider.state(client)
                }
            },
        )
        .unwrap_err();
        assert!(error.to_string().contains("Minecraft launch failure"));
        assert_eq!(
            *provider.states.borrow(),
            [
                ClientState::Running,
                ClientState::Stopped,
                ClientState::Stopped
            ]
        );
    }

    #[test]
    fn start_batch_window_failure_rolls_back_prior_successful_vm() {
        let provider = stopped_provider();
        let error = super::execute_start_batch(
            &provider,
            &ClientId::VIRTUAL,
            |_, _| Ok(()),
            |client| {
                if client == ClientId::Virtual02 {
                    provider.open(client)?;
                    Ok(provider.state(client)?)
                } else if client == ClientId::Virtual03 {
                    Err(io::Error::new(
                        io::ErrorKind::Other,
                        "injected window open failure",
                    ))
                } else {
                    provider.state(client)
                }
            },
        )
        .unwrap_err();
        assert!(error.to_string().contains("window open failure"));
        assert_eq!(*provider.states.borrow(), [ClientState::Stopped; 3]);
    }

    #[test]
    fn start_batch_rolls_back_when_later_admission_fails() {
        let provider = stopped_provider();
        let error = super::execute_start_batch(
            &provider,
            &ClientId::VIRTUAL,
            |client, _| {
                if client == ClientId::Virtual02 {
                    Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "lineage changed",
                    ))
                } else {
                    Ok(())
                }
            },
            |client| provider.state(client),
        )
        .unwrap_err();
        assert!(error.to_string().contains("lineage changed"));
        assert_eq!(*provider.states.borrow(), [ClientState::Stopped; 3]);
    }

    #[test]
    fn start_batch_rolls_back_result_collection_failure() {
        let provider = FakeProvider::new(
            [
                ClientState::Suspended,
                ClientState::Stopped,
                ClientState::Stopped,
            ],
            [
                Some("uuid-a|mac-a"),
                Some("uuid-b|mac-b"),
                Some("uuid-c|mac-c"),
            ],
        );
        let result = super::execute_start_batch(
            &provider,
            &ClientId::VIRTUAL,
            |_, _| Ok(()),
            |client| {
                if client == ClientId::Virtual02 {
                    Err(io::Error::new(io::ErrorKind::Other, "status unavailable"))
                } else {
                    provider.state(client)
                }
            },
        );
        assert!(result.is_err());
        assert_eq!(
            *provider.states.borrow(),
            [
                ClientState::Suspended,
                ClientState::Stopped,
                ClientState::Stopped
            ]
        );
    }

    #[test]
    fn start_batch_preserves_preexisting_running_client_on_verification_failure() {
        let provider = FakeProvider::new(
            [
                ClientState::Stopped,
                ClientState::Running,
                ClientState::Stopped,
            ],
            [
                Some("uuid-a|mac-a"),
                Some("uuid-b|mac-b"),
                Some("uuid-c|mac-c"),
            ],
        );
        let result = super::execute_start_batch(
            &provider,
            &ClientId::VIRTUAL,
            |_, _| Ok(()),
            |client| {
                if client == ClientId::Virtual02 {
                    Err(io::Error::new(io::ErrorKind::InvalidData, "guest mismatch"))
                } else {
                    provider.state(client)
                }
            },
        );
        assert!(result.is_err());
        assert_eq!(
            *provider.states.borrow(),
            [
                ClientState::Stopped,
                ClientState::Running,
                ClientState::Stopped
            ]
        );
    }

    #[test]
    fn start_batch_tracks_provider_mutation_before_a_start_error() {
        let provider = FakeProvider {
            fail_start: Some(ClientId::Virtual02),
            fail_after_start: true,
            ..stopped_provider()
        };
        let result = super::execute_start_batch(
            &provider,
            &ClientId::VIRTUAL,
            |_, _| Ok(()),
            |client| provider.state(client),
        );
        assert!(result.is_err());
        assert_eq!(*provider.states.borrow(), [ClientState::Stopped; 3]);
    }

    #[test]
    fn start_batch_reconciles_a_timeout_that_mutated_provider_state() {
        let provider = FakeProvider {
            fail_start: Some(ClientId::Virtual02),
            fail_after_start: true,
            ..stopped_provider()
        };
        let error = super::execute_start_batch(
            &provider,
            &ClientId::VIRTUAL,
            |_, _| Ok(()),
            |client| provider.state(client),
        )
        .unwrap_err();
        assert!(error.to_string().contains("injected start failure"));
        assert_eq!(*provider.states.borrow(), [ClientState::Stopped; 3]);
    }

    #[test]
    fn start_batch_never_rolls_back_a_preexisting_running_client() {
        let provider = FakeProvider {
            fail_start: Some(ClientId::Virtual03),
            fail_after_start: true,
            ..FakeProvider::new(
                [
                    ClientState::Running,
                    ClientState::Stopped,
                    ClientState::Stopped,
                ],
                [
                    Some("uuid-a|mac-a"),
                    Some("uuid-b|mac-b"),
                    Some("uuid-c|mac-c"),
                ],
            )
        };
        let result = super::execute_start_batch(
            &provider,
            &ClientId::VIRTUAL,
            |_, _| Ok(()),
            |client| provider.state(client),
        );
        assert!(result.is_err());
        assert_eq!(
            *provider.states.borrow(),
            [
                ClientState::Running,
                ClientState::Stopped,
                ClientState::Stopped
            ]
        );
    }

    #[test]
    fn connection_health_is_a_strict_evidence_projection() {
        let guest = |minecraft_running, interactive_launcher_ready| GuestStatus {
            schema: crate::guest::GUEST_STATUS_SCHEMA,
            protocol_version: crate::guest::GUEST_AGENT_PROTOCOL_VERSION,
            agent_version: "test".into(),
            minecraft: None,
            minecraft_running,
            interactive_launcher_ready,
            machine_identity: None,
        };

        assert_eq!(
            connection_health(ClientState::Stopped, None, None),
            Some(ConnectionHealth::VmOffline)
        );
        assert_eq!(
            connection_health(ClientState::Running, None, None),
            Some(ConnectionHealth::VmRunning)
        );
        assert_eq!(
            connection_health(ClientState::Running, Some(true), None),
            Some(ConnectionHealth::GuestToolsReady)
        );
        assert_eq!(
            connection_health(
                ClientState::Running,
                Some(true),
                Some(&guest(Some(false), Some(false)))
            ),
            Some(ConnectionHealth::GuestAgentReady)
        );
        assert_eq!(
            connection_health(
                ClientState::Running,
                Some(true),
                Some(&guest(Some(false), Some(true)))
            ),
            Some(ConnectionHealth::InteractiveLauncherReady)
        );
        assert_eq!(
            connection_health(
                ClientState::Running,
                Some(true),
                Some(&guest(Some(true), Some(false)))
            ),
            Some(ConnectionHealth::MinecraftRunning)
        );
        assert_eq!(
            connection_health(ClientState::NotProvisioned, None, None),
            None
        );
        assert_eq!(connection_health(ClientState::Error, None, None), None);
    }

    #[test]
    fn start_rollback_never_restarts_a_previously_running_client() {
        let provider = FakeProvider::new(
            [
                ClientState::Stopped,
                ClientState::Stopped,
                ClientState::Stopped,
            ],
            [
                Some("uuid-a|mac-a"),
                Some("uuid-b|mac-b"),
                Some("uuid-c|mac-c"),
            ],
        );
        let failed = restore_batch_state(&provider, &[(ClientId::Virtual01, ClientState::Running)]);
        assert!(failed.is_empty());
        assert_eq!(
            provider.state(ClientId::Virtual01).unwrap(),
            ClientState::Stopped
        );
        assert_eq!(
            provider.state(ClientId::Virtual02).unwrap(),
            ClientState::Stopped
        );
        assert_eq!(
            provider.state(ClientId::Virtual03).unwrap(),
            ClientState::Stopped
        );
    }

    #[test]
    fn start_batch_reports_rollback_failure_and_keeps_the_original_error() {
        let provider = FakeProvider {
            fail_start: Some(ClientId::Virtual02),
            fail_stop: Some(ClientId::Virtual01),
            ..stopped_provider()
        };
        let error = super::execute_start_batch(
            &provider,
            &ClientId::VIRTUAL,
            |_, _| Ok(()),
            |client| provider.state(client),
        )
        .unwrap_err();
        let message = error.to_string();
        assert!(message.contains("injected start failure"));
        assert!(message.contains("rollback incomplete for Virtual-01"));
        assert_eq!(
            *provider.states.borrow(),
            [
                ClientState::Running,
                ClientState::Stopped,
                ClientState::Stopped
            ]
        );
    }

    #[test]
    fn start_batch_rolls_back_a_later_invalid_state() {
        let provider = FakeProvider::new(
            [
                ClientState::Stopped,
                ClientState::Error,
                ClientState::Stopped,
            ],
            [
                Some("uuid-a|mac-a"),
                Some("uuid-b|mac-b"),
                Some("uuid-c|mac-c"),
            ],
        );
        assert!(super::execute_start_batch(
            &provider,
            &ClientId::VIRTUAL,
            |_, _| Ok(()),
            |client| provider.state(client),
        )
        .is_err());
        assert_eq!(
            provider.state(ClientId::Virtual01).unwrap(),
            ClientState::Stopped
        );
        assert_eq!(
            provider.state(ClientId::Virtual02).unwrap(),
            ClientState::Error
        );
    }

    #[test]
    fn start_batch_success_changes_only_selected_clients() {
        let provider = stopped_provider();
        let result = super::execute_start_batch(
            &provider,
            &[ClientId::Virtual02],
            |_, _| Ok(()),
            |client| provider.state(client),
        )
        .unwrap();
        assert_eq!(result, vec![ClientState::Running]);
        assert_eq!(
            *provider.states.borrow(),
            [
                ClientState::Stopped,
                ClientState::Running,
                ClientState::Stopped
            ]
        );
    }

    #[test]
    fn start_batch_stops_a_resumed_client_that_fails_verification() {
        let provider = FakeProvider::new(
            [
                ClientState::Suspended,
                ClientState::Stopped,
                ClientState::Stopped,
            ],
            [
                Some("uuid-a|mac-a"),
                Some("uuid-b|mac-b"),
                Some("uuid-c|mac-c"),
            ],
        );
        let result: io::Result<Vec<ClientState>> = super::execute_start_batch(
            &provider,
            &[ClientId::Virtual01],
            |_, _| Ok(()),
            |_| Err(io::Error::new(io::ErrorKind::InvalidData, "guest mismatch")),
        );
        assert!(result.is_err());
        assert_eq!(*provider.states.borrow(), [ClientState::Stopped; 3]);
    }

    #[test]
    fn setup_start_returns_running_without_a_guest_agent() {
        let provider = stopped_provider();
        for client in ClientId::VIRTUAL {
            assert_eq!(
                super::execute_setup_start(&provider, client, |_, _| Ok(())).unwrap(),
                ClientState::Running,
            );
        }
        assert_eq!(*provider.states.borrow(), [ClientState::Running; 3]);
    }

    #[test]
    fn setup_open_failure_stops_only_a_newly_started_client() {
        let provider = FakeProvider {
            fail_open: true,
            ..stopped_provider()
        };
        assert!(super::execute_setup_start(&provider, ClientId::Virtual01, |_, _| Ok(())).is_err());
        assert_eq!(*provider.states.borrow(), [ClientState::Stopped; 3]);
        provider
            .set_state(ClientId::Virtual01, ClientState::Running)
            .unwrap();
        assert!(super::execute_setup_start(&provider, ClientId::Virtual01, |_, _| Ok(())).is_err());
        assert_eq!(
            provider.state(ClientId::Virtual01).unwrap(),
            ClientState::Running
        );
    }

    #[test]
    fn setup_admission_failure_does_not_start_a_client() {
        let provider = stopped_provider();
        assert!(
            super::execute_setup_start(&provider, ClientId::Virtual01, |_, _| {
                Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "not a fresh client",
                ))
            })
            .is_err()
        );
        assert_eq!(*provider.states.borrow(), [ClientState::Stopped; 3]);
    }
}
