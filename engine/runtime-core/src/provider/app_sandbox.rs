//! Read-only App Sandbox v0.1.9 inspection for explicit CLI diagnostics.
//! This is NOT a second provider, lifecycle authority, or Minecraft readiness probe.
//! Never send mutations, copy tokens to logs, or infer GPU/render compatibility.
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs, io::Read, path::Path, time::Duration};

const MAX_DISCOVERY_BYTES: u64 = 4096;
const MAX_API_BYTES: u64 = 256 * 1024;
const EXPECTED_PRODUCT_VERSION: &str = "0.1.9";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppSandboxInspection {
    pub state: &'static str,
    pub reason: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<HostCapacity>,
    pub guests: Vec<GuestObservation>,
    pub game_and_gpu_verified: bool,
}

impl AppSandboxInspection {
    fn unavailable(reason: &'static str) -> Self {
        Self {
            state: "UNVERIFIED",
            reason,
            version: None,
            host: None,
            guests: Vec::new(),
            game_and_gpu_verified: false,
        }
    }
}

#[derive(Debug, Deserialize)]
struct Discovery {
    endpoint: String,
    port: u16,
    token: String,
    pid: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Version {
    product: String,
    version: String,
    api_version: String,
    host_os: String,
    capabilities: Capabilities,
}

#[derive(Debug, Deserialize)]
struct Capabilities {
    snapshots: bool,
    templates: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HostCapacity {
    host_cores: u64,
    host_ram_mb: u64,
    free_gb: u64,
    vm_cores: u64,
    vm_ram_mb: u64,
    vm_hdd_gb: u64,
}

#[derive(Debug, Deserialize)]
struct VmList {
    // Other users may have Linux VMs with longer hostnames or different
    // device modes. Only parse full state for OUR three canonical names.
    vms: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VmState {
    name: String,
    os_type: String,
    // App Sandbox status returns a directory, not active VHDX or MAC identity.
    disk_directory: String,
    state: String,
    running: bool,
    agent_online: bool,
    install_complete: bool,
    building: bool,
    ram_mb: u64,
    cpu_cores: u64,
    gpu_mode: u8,
    network_mode: u8,
    display_open: bool,
}

// This is VM power only, NEVER an Education readiness claim. Transitional
// states have no power-control implication and cannot authorize any action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum VmPowerState {
    PoweredOff,
    PoweredOn,
    Transitioning,
}

#[derive(Debug, Deserialize)]
struct DisplayState {
    open: bool,
    ready: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SshInfo {
    host: String,
    port: u16,
    #[serde(rename = "user")]
    _user: String,
    ssh_state: u8,
    enabled: bool,
    key_deployed: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SshObservation {
    // Only a forwarding/configuration signal. This is NOT remote-command
    // authorization and never establishes trusted host identity.
    key_forwarding_ready: bool,
    host_key_pin_present: bool,
    host_key_verified: bool,
    guest_status_observed: bool,
    minecraft_commands_enabled: bool,
}

fn parse_ssh_info(data: &[u8]) -> Result<SshObservation, &'static str> {
    let ssh: SshInfo = serde_json::from_slice(data).map_err(|_| "INVALID_PROVIDER_RESPONSE")?;
    if ssh.host != "127.0.0.1"
        || ssh.ssh_state > 4
        || (ssh.ssh_state == 4 && !ssh.key_deployed)
        || (ssh.ssh_state == 4 && (!ssh.enabled || ssh.port == 0))
    {
        return Err("INVALID_PROVIDER_RESPONSE");
    }
    Ok(SshObservation {
        key_forwarding_ready: ssh.enabled && ssh.ssh_state == 4 && ssh.key_deployed && ssh.port > 0,
        host_key_pin_present: false,
        host_key_verified: false,
        guest_status_observed: false,
        minecraft_commands_enabled: false,
    })
}

fn parse_display(data: &[u8]) -> Result<DisplayState, &'static str> {
    let display: DisplayState =
        serde_json::from_slice(data).map_err(|_| "INVALID_PROVIDER_RESPONSE")?;
    // Source: App Sandbox v0.1.9, headless.c GET /display:
    // ready = open || asb_vm_idd_ready(vm).
    if display.open && !display.ready {
        return Err("INVALID_PROVIDER_RESPONSE");
    }
    Ok(display)
}

// Reject unstable aliases, relative paths and parent traversals without
// opening or modifying VM storage. These keys are only same-request evidence,
// not HCS/VHDX identity or an ownership authorization.
pub(super) fn normalized_disk_root(raw: &str) -> Option<String> {
    let bytes = raw.as_bytes();
    if bytes.len() < 6
        || bytes.len() > 260
        || !bytes[0].is_ascii_alphabetic()
        || bytes[1] != b':'
        || bytes[2] != b'\\'
        || raw.contains('/')
        || raw.contains(['\0', '*', '?', '"', '<', '>', '|'])
    {
        return None;
    }
    let mut normalized = String::new();
    for segment in raw[3..].split('\\') {
        if segment.is_empty()
            || segment == "."
            || segment == ".."
            || segment.ends_with(' ')
            || segment.ends_with('.')
            || segment.contains(':')
        {
            return None;
        }
        if !normalized.is_empty() {
            normalized.push('\\');
        }
        normalized.push_str(&segment.to_lowercase());
    }
    Some(format!(
        "{}:\\{normalized}",
        (bytes[0] as char).to_ascii_lowercase()
    ))
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct SnapshotPointer {
    snap_index: i32,
    branch_index: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SnapshotState {
    current: SnapshotPointer,
    snapshots: Vec<serde_json::Value>,
    base_branches: Vec<serde_json::Value>,
}

fn parse_snapshot_pointer(data: &[u8]) -> Result<SnapshotPointer, &'static str> {
    let report: SnapshotState =
        serde_json::from_slice(data).map_err(|_| "INVALID_PROVIDER_RESPONSE")?;
    let position = report.current;
    if report.snapshots.len() > 128
        || report.base_branches.len() > 128
        || position.snap_index < -2
        || position.branch_index < -1
        || position.snap_index > 127
        || position.branch_index > 127
        || (position.snap_index == -1 && position.branch_index != -1)
    {
        return Err("INVALID_PROVIDER_RESPONSE");
    }
    Ok(position)
}

fn vm_power_state(state: &str) -> VmPowerState {
    match state {
        "stopped" => VmPowerState::PoweredOff,
        "installing" | "booting" | "online" => VmPowerState::PoweredOn,
        // The caller passes a parsed and validated upstream state.
        _ => VmPowerState::Transitioning,
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GuestObservation {
    name: String,
    state: String,
    power_state: VmPowerState,
    running: bool,
    agent_online: bool,
    display_open: bool,
    display_ready: bool,
    ssh: SshObservation,
    storage: StorageEvidence,
    windows_identity_observed: bool,
    windows_identity_unique_across_three: bool,
    // No App Sandbox lifecycle command is wired to the product runtime yet.
    // In particular, the upstream stop endpoint is a hard power-off.
    lifecycle_controls_enabled: bool,
    ram_mb: u64,
    cpu_cores: u64,
    gpu_mode: u8,
    network_mode: u8,
    ownership_verified: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StorageEvidence {
    // The HTTP diskDirectory is a SHARED PARENT, not the VM's active VHDX.
    // The separately read upstream vms.cfg adds MAC/VHDX paths but still does
    // not prove HCS disk attachment or account ownership.
    registry_record_observed: bool,
    unique_vm_directory: bool,
    persisted_mac_observed: bool,
    active_vhdx_path_observed: bool,
    snapshot: SnapshotPointer,
    active_vhdx_verified: bool,
}

fn parse_discovery(data: &[u8]) -> Result<Discovery, &'static str> {
    let entry: Discovery = serde_json::from_slice(data).map_err(|_| "INVALID_DISCOVERY")?;
    let expected = format!("http://127.0.0.1:{}", entry.port);
    if entry.port == 0
        || entry.endpoint != expected
        || entry.pid == 0
        || entry.token.len() != 40
        || !entry
            .token
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err("INVALID_DISCOVERY");
    }
    Ok(entry)
}

fn parse_version(data: &[u8]) -> Result<Version, &'static str> {
    let info: Version = serde_json::from_slice(data).map_err(|_| "INVALID_PROVIDER_RESPONSE")?;
    if info.product != "AppSandbox"
        || info.version != EXPECTED_PRODUCT_VERSION
        || info.api_version != "v1"
        || info.host_os != "Windows"
        || !info.capabilities.snapshots
        || !info.capabilities.templates
    {
        return Err("PROVIDER_INCOMPATIBLE");
    }
    Ok(info)
}

fn parse_host(data: &[u8]) -> Result<HostCapacity, &'static str> {
    let host: HostCapacity =
        serde_json::from_slice(data).map_err(|_| "INVALID_PROVIDER_RESPONSE")?;
    if host.host_cores == 0 || host.host_ram_mb == 0 {
        return Err("INVALID_PROVIDER_RESPONSE");
    }
    Ok(host)
}

fn parse_vm(data: &[u8]) -> Result<VmState, &'static str> {
    let vm: VmState = serde_json::from_slice(data).map_err(|_| "INVALID_PROVIDER_RESPONSE")?;
    if vm.name.is_empty()
        || vm.name.len() > 15
        || !vm
            .name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        || normalized_disk_root(&vm.disk_directory).is_none()
        || vm.cpu_cores == 0
        || vm.ram_mb < 512
        || vm.gpu_mode > 1
        || vm.network_mode > 3
        || !matches!(
            vm.state.as_str(),
            "building" | "stopped" | "installing" | "booting" | "online" | "stopping"
        )
    {
        return Err("INVALID_PROVIDER_RESPONSE");
    }
    // Validate invariants observed in src/app_win/headless.c::derive_state.
    let consistent = match vm.state.as_str() {
        "building" => vm.building,
        "stopped" => !vm.running && !vm.building,
        "installing" => vm.running && !vm.install_complete && !vm.building,
        "booting" => vm.running && vm.install_complete && !vm.agent_online && !vm.building,
        "online" => vm.running && vm.install_complete && vm.agent_online && !vm.building,
        "stopping" => vm.running && !vm.building,
        _ => false,
    };
    if !consistent {
        return Err("INVALID_PROVIDER_RESPONSE");
    }
    Ok(vm)
}

fn parse_vm_list(data: &[u8]) -> Result<Vec<VmState>, &'static str> {
    let raw: VmList = serde_json::from_slice(data).map_err(|_| "INVALID_PROVIDER_RESPONSE")?;
    if raw.vms.len() > 128 {
        return Err("INVALID_PROVIDER_RESPONSE");
    }
    let mut names = HashSet::new();
    let mut values = Vec::with_capacity(raw.vms.len());
    for record in raw.vms {
        let name = record
            .get("name")
            .and_then(serde_json::Value::as_str)
            .ok_or("INVALID_PROVIDER_RESPONSE")?;
        if name.is_empty() || !names.insert(name.to_ascii_lowercase()) {
            return Err("INVALID_PROVIDER_RESPONSE");
        }
        if is_tracked_guest(name) {
            // Validate tracked guests as strictly as the named status route.
            let encoded = serde_json::to_vec(&record).map_err(|_| "INVALID_PROVIDER_RESPONSE")?;
            values.push(parse_vm(&encoded)?);
        } else if ["Virtual-01", "Virtual-02", "Virtual-03"]
            .iter()
            .any(|target| name.eq_ignore_ascii_case(target))
        {
            // App Sandbox compares VM names case-insensitively. Treat a
            // differently-cased tracked name as an ambiguous ownership collision,
            // NOT as an absent client eligible for provisioning.
            return Err("TRACKED_NAME_COLLISION");
        }
    }
    Ok(values)
}

fn is_tracked_guest(name: &str) -> bool {
    matches!(name, "Virtual-01" | "Virtual-02" | "Virtual-03")
}

fn read_discovery_at(path: &Path) -> Result<Discovery, &'static str> {
    let metadata = fs::symlink_metadata(path).map_err(|_| "DISCOVERY_UNAVAILABLE")?;
    if !metadata.file_type().is_file() || metadata.len() > MAX_DISCOVERY_BYTES {
        return Err("INVALID_DISCOVERY");
    }
    let data = fs::read(path).map_err(|_| "DISCOVERY_UNAVAILABLE")?;
    if data.len() as u64 > MAX_DISCOVERY_BYTES {
        return Err("INVALID_DISCOVERY");
    }
    parse_discovery(&data)
}

#[cfg(target_os = "windows")]
fn discover() -> Result<Discovery, &'static str> {
    let root = std::env::var_os("ProgramData").ok_or("DISCOVERY_UNAVAILABLE")?;
    let root = Path::new(&root);
    if !root.is_absolute() {
        return Err("INVALID_DISCOVERY");
    }
    // Do not follow a redirected AppSandbox directory or host.json file.
    let folder = root.join("AppSandbox");
    if fs::symlink_metadata(&folder)
        .map_err(|_| "DISCOVERY_UNAVAILABLE")?
        .file_type()
        .is_symlink()
    {
        return Err("INVALID_DISCOVERY");
    }
    read_discovery_at(&folder.join("host.json"))
}

#[cfg(not(target_os = "windows"))]
fn discover() -> Result<Discovery, &'static str> {
    Err("WINDOWS_HOST_REQUIRED")
}

fn get_json(
    agent: &ureq::Agent,
    endpoint: &str,
    route: &str,
    token: Option<&str>,
) -> Result<Vec<u8>, &'static str> {
    let url = format!("{endpoint}/v1/{route}");
    let mut request = agent.get(&url);
    if let Some(token) = token {
        let authorization = format!("Bearer {token}");
        request = request.header("Authorization", &authorization);
    }
    let response = match request.call() {
        Ok(response) => response,
        Err(ureq::Error::StatusCode(401)) => return Err("PROVIDER_AUTH_FAILED"),
        Err(ureq::Error::StatusCode(404)) => return Err("VM_NOT_FOUND"),
        Err(ureq::Error::StatusCode(_)) => return Err("PROVIDER_HTTP_ERROR"),
        Err(_) => return Err("PROVIDER_UNREACHABLE"),
    };
    if response.status() != 200 {
        return Err("PROVIDER_HTTP_ERROR");
    }
    let mut bytes = Vec::new();
    response
        .into_body()
        .into_reader()
        .take(MAX_API_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "PROVIDER_UNREACHABLE")?;
    if bytes.len() as u64 > MAX_API_BYTES {
        return Err("INVALID_PROVIDER_RESPONSE");
    }
    Ok(bytes)
}

// A short-lived, never-serialized connection to the upstream single-owner daemon.
// Both diagnostics and the explicitly invoked disposable pilot use this one
// discovery/version/auth boundary; no token or agent is persisted or cached.
pub(super) struct AsbSession {
    pub(super) agent: ureq::Agent,
    pub(super) endpoint: String,
    pub(super) token: String,
    pub(super) version: String,
    pub(super) host: HostCapacity,
}

fn provider_agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .proxy(None)
        .max_redirects(0)
        .max_redirects_will_error(false)
        .timeout_connect(Some(Duration::from_secs(2)))
        .timeout_recv_response(Some(Duration::from_secs(3)))
        .timeout_recv_body(Some(Duration::from_secs(3)))
        .timeout_send_request(Some(Duration::from_secs(3)))
        .timeout_send_body(Some(Duration::from_secs(3)))
        .build()
        .into()
}

pub(super) fn authenticated_session() -> Result<AsbSession, &'static str> {
    let discovery = discover()?;
    let agent = provider_agent();
    let version_bytes = get_json(&agent, &discovery.endpoint, "version", None)?;
    let version = parse_version(&version_bytes)?;
    // Version is deliberately unauthenticated upstream. Prove the bearer
    // session with an authenticated response before making pilot decisions.
    let host_json = get_json(&agent, &discovery.endpoint, "host", Some(&discovery.token))?;
    let host = parse_host(&host_json)?;
    Ok(AsbSession {
        agent,
        endpoint: discovery.endpoint,
        token: discovery.token,
        version: version.version,
        host,
    })
}

impl AsbSession {
    // The upstream host info reports free space on the ProgramData volume.
    // It is not a guarantee of available space for the eventual dynamic VHDX.
    pub(super) fn disk_space_at_least_gb(&self, required: u64) -> bool {
        self.host.free_gb >= required
    }

    pub(super) fn get(&self, route: &str) -> Result<Vec<u8>, &'static str> {
        get_json(&self.agent, &self.endpoint, route, Some(&self.token))
    }
}

fn inspect_inner() -> Result<AppSandboxInspection, &'static str> {
    let session = authenticated_session()?;
    let version = session.version.clone();
    let host = session.host.clone();
    let list_bytes = session.get("vms")?;
    let records = parse_vm_list(&list_bytes)?;
    let mut guests = Vec::new();
    // This file is mutable and occasionally rewritten by the upstream host.
    // Inaccessible metadata gives UNKNOWN, not permission to replace anything.
    let registry_before = super::app_sandbox_registry::read_registry().ok();
    let mut observed_vm_roots = HashSet::new();
    let mut windows_identities = HashSet::new();
    let mut windows_identity_count = 0usize;
    for item in records {
        if !is_tracked_guest(&item.name) {
            continue;
        }
        let route = format!("vms/{}", item.name);
        let status_bytes = session.get(&route)?;
        let status = parse_vm(&status_bytes)?;
        if status.name != item.name
            || status.os_type != "Windows"
            || normalized_disk_root(&status.disk_directory)
                != normalized_disk_root(&item.disk_directory)
        {
            return Err("INVALID_PROVIDER_RESPONSE");
        }
        let registry_vm = registry_before.as_ref().and_then(|reg| reg.vm(&item.name));
        if let Some(registry_vm) = registry_vm {
            if registry_vm.name != item.name
                || normalized_disk_root(&status.disk_directory).as_deref()
                    != Some(registry_vm.shared_parent.as_str())
            {
                return Err("VM_REGISTRY_STATUS_MISMATCH");
            }
            if !observed_vm_roots.insert(registry_vm.vm_root.clone()) {
                return Err("DUPLICATE_VM_ROOT");
            }
        }
        let snapshot_route = format!("vms/{}/snapshots", item.name);
        let snapshot_bytes = session.get(&snapshot_route)?;
        let snapshot = parse_snapshot_pointer(&snapshot_bytes)?;
        let display_route = format!("vms/{}/display", item.name);
        let display_bytes = session.get(&display_route)?;
        let display = parse_display(&display_bytes)?;
        let ssh_route = format!("vms/{}/sshInfo", item.name);
        let ssh_bytes = session.get(&ssh_route)?;
        let mut ssh = parse_ssh_info(&ssh_bytes)?;
        let mut windows_identity_observed = false;
        if ssh.key_forwarding_ready {
            let info: SshInfo =
                serde_json::from_slice(&ssh_bytes).map_err(|_| "INVALID_PROVIDER_RESPONSE")?;
            // No TOFU: only a separately enrolled host key permits the
            // fixed, bounded and read-only --ssh-status command.
            let proof = super::app_sandbox_ssh::probe_status(&item.name, &info._user, info.port);
            ssh.host_key_pin_present = proof.pin_present;
            ssh.host_key_verified = proof.host_key_verified;
            ssh.guest_status_observed = proof.guest_status_observed;
            if let Some(identity) = proof.machine_identity {
                windows_identity_observed = true;
                if !windows_identities.insert(identity) {
                    return Err("DUPLICATE_WINDOWS_MACHINE_IDENTITY");
                }
                windows_identity_count += 1;
            }
        }
        // Status, display and SSH reads are not an atomic snapshot. Keep the
        // latter as a direct, read-only observation; neither allows controls.
        // VM names are observations, not trusted bindings to our account disks.
        guests.push(GuestObservation {
            name: status.name,
            power_state: vm_power_state(&status.state),
            state: status.state,
            running: status.running,
            agent_online: status.agent_online,
            display_open: display.open,
            display_ready: display.ready,
            ssh,
            storage: StorageEvidence {
                registry_record_observed: registry_vm.is_some(),
                unique_vm_directory: registry_vm.is_some(),
                persisted_mac_observed: registry_vm.is_some_and(|vm| vm.mac.is_some()),
                active_vhdx_path_observed: registry_vm.is_some(),
                snapshot,
                active_vhdx_verified: false,
            },
            windows_identity_observed,
            windows_identity_unique_across_three: false,
            lifecycle_controls_enabled: false,
            ram_mb: status.ram_mb,
            cpu_cores: status.cpu_cores,
            gpu_mode: status.gpu_mode,
            network_mode: status.network_mode,
            ownership_verified: false,
        });
    }
    // Detect registry writes or branch changes while API/SSH observations run.
    if let Some(before) = &registry_before {
        if super::app_sandbox_registry::read_registry().as_ref() != Ok(before) {
            return Err("VM_REGISTRY_CHANGED_DURING_INSPECTION");
        }
    }
    // Three different MachineGuid hashes are a necessary condition for account
    // isolation, not proof of stable MAC/VHDX identity or Windows entitlements.
    if guests.len() == 3 && windows_identity_count == 3 {
        for guest in &mut guests {
            guest.windows_identity_unique_across_three = true;
        }
    }
    Ok(AppSandboxInspection {
        state: "OBSERVED",
        reason: "VM_API_ONLY_NO_GAME_OR_GPU_PROOF",
        version: Some(version),
        host: Some(host),
        guests,
        game_and_gpu_verified: false,
    })
}

/// Explicit one-shot diagnostic; never used by normal UI snapshot polling.
pub(crate) fn inspect() -> AppSandboxInspection {
    match inspect_inner() {
        Ok(report) => report,
        Err(reason) => AppSandboxInspection::unavailable(reason),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        normalized_disk_root, parse_discovery, parse_display, parse_host, parse_snapshot_pointer,
        parse_ssh_info, parse_version, parse_vm, parse_vm_list, vm_power_state, VmPowerState,
    };
    use serde_json::json;

    #[test]
    fn discovery_is_literal_loopback_with_a_distinct_40_hex_token() {
        let valid = json!({"endpoint":"http://127.0.0.1:8787","port":8787,"pid":412,"token":"a".repeat(40)});
        assert!(parse_discovery(&serde_json::to_vec(&valid).unwrap()).is_ok());
        for altered in [
            json!({"endpoint":"http://localhost:8787","port":8787,"pid":412,"token":"a".repeat(40)}),
            json!({"endpoint":"http://127.0.0.1:8788","port":8787,"pid":412,"token":"a".repeat(40)}),
            json!({"endpoint":"http://127.0.0.1:8787/evil","port":8787,"pid":412,"token":"a".repeat(40)}),
            json!({"endpoint":"http://127.0.0.1:8787","port":8787,"pid":412,"token":"z".repeat(40)}),
            json!({"endpoint":"http://127.0.0.1:8787","port":8787,"pid":412,"token":"a".repeat(64)}),
        ] {
            assert!(parse_discovery(&serde_json::to_vec(&altered).unwrap()).is_err());
        }
    }

    #[test]
    fn version_pinned_and_host_requires_positive_capacity() {
        let v = json!({"product":"AppSandbox","version":"0.1.9","apiVersion":"v1","hostOs":"Windows","capabilities":{"snapshots":true,"templates":true}});
        assert!(parse_version(&serde_json::to_vec(&v).unwrap()).is_ok());
        let mut wrong = v.clone();
        wrong["apiVersion"] = json!("v2");
        assert!(parse_version(&serde_json::to_vec(&wrong).unwrap()).is_err());
        let host = json!({"hostCores":28,"hostRamMb":32768,"freeGb":100,"vmCores":6,"vmRamMb":12288,"vmHddGb":180});
        assert_eq!(
            parse_host(&serde_json::to_vec(&host).unwrap())
                .unwrap()
                .vm_ram_mb,
            12288
        );
        let mut bad = host.clone();
        bad["hostCores"] = json!(0);
        assert!(parse_host(&serde_json::to_vec(&bad).unwrap()).is_err());
    }

    fn vm(name: &str) -> serde_json::Value {
        json!({"name":name,"osType":"Windows","diskDirectory":format!(r"C:\Users\QA\AppSandbox\{}", name),"state":"online","running":true,"agentOnline":true,"installComplete":true,"building":false,"ramMb":4096,"cpuCores":2,"gpuMode":1,"networkMode":1,"displayOpen":true})
    }

    #[test]
    fn vm_states_and_duplicate_names_are_not_silently_accepted() {
        let record = vm("Virtual-01");
        assert!(parse_vm(&serde_json::to_vec(&record).unwrap()).is_ok());
        let mut bad = record.clone();
        bad["state"] = json!("online");
        bad["agentOnline"] = json!(false);
        assert!(parse_vm(&serde_json::to_vec(&bad).unwrap()).is_err());
        let mut bad = record.clone();
        bad["gpuMode"] = json!(2);
        assert!(parse_vm(&serde_json::to_vec(&bad).unwrap()).is_err());
        let list = json!({"vms":[vm("Virtual-01"),vm("Virtual-02"),vm("Virtual-03")]});
        assert_eq!(
            parse_vm_list(&serde_json::to_vec(&list).unwrap())
                .unwrap()
                .len(),
            3
        );
        let duplicates = json!({"vms":[vm("Virtual-01"),vm("virtual-01")]});
        assert!(parse_vm_list(&serde_json::to_vec(&duplicates).unwrap()).is_err());
        // Linux guests with 63-character names are unrelated to our Windows
        // sessions; they must not make the read-only inventory fail.
        let foreign = json!({"vms":[vm("Virtual-01"),{"name":"unrelated-linux-guest-with-a-long-hostname-okay","osType":"Linux","state":"online","gpuMode":2}]});
        assert_eq!(
            parse_vm_list(&serde_json::to_vec(&foreign).unwrap())
                .unwrap()
                .len(),
            1
        );
        let collision = json!({"vms":[vm("virtual-01")]});
        assert_eq!(
            parse_vm_list(&serde_json::to_vec(&collision).unwrap()).unwrap_err(),
            "TRACKED_NAME_COLLISION"
        );
    }

    #[test]
    fn power_phase_never_conflates_transitions_with_a_playable_guest() {
        for (upstream, expected) in [
            ("building", VmPowerState::Transitioning),
            ("installing", VmPowerState::PoweredOn),
            ("booting", VmPowerState::PoweredOn),
            ("online", VmPowerState::PoweredOn),
            ("stopping", VmPowerState::Transitioning),
            ("stopped", VmPowerState::PoweredOff),
        ] {
            assert_eq!(vm_power_state(upstream), expected);
        }
    }

    #[test]
    fn read_only_display_state_is_not_gameplay_evidence() {
        let display = parse_display(br#"{"open":false,"ready":true}"#).unwrap();
        assert!(!display.open && display.ready);
        assert!(parse_display(br#"{"open":true,"ready":false}"#).is_err());
        assert!(parse_display(br#"{"open":"yes","ready":true}"#).is_err());
        let phase = vm_power_state("online");
        assert_eq!(phase, VmPowerState::PoweredOn);
        let guest = super::GuestObservation {
            name: "Virtual-01".to_string(),
            state: "online".to_string(),
            power_state: phase,
            running: true,
            agent_online: true,
            display_open: false,
            display_ready: true,
            ssh: parse_ssh_info(br#"{"host":"127.0.0.1","port":24000,"user":"Administrator","sshState":4,"enabled":true,"keyDeployed":true}"#).unwrap(),
            storage: super::StorageEvidence {
                registry_record_observed: false,
                unique_vm_directory: false,
                persisted_mac_observed: false,
                active_vhdx_path_observed: false,
                snapshot: parse_snapshot_pointer(br#"{"current":{"snapIndex":-1,"branchIndex":-1},"snapshots":[],"baseBranches":[]}"#).unwrap(),
                active_vhdx_verified: false,
            },
            windows_identity_observed: false,
            windows_identity_unique_across_three: false,
            lifecycle_controls_enabled: false,
            ram_mb: 4096,
            cpu_cores: 2,
            gpu_mode: 1,
            network_mode: 1,
            ownership_verified: false,
        };
        let json = serde_json::to_value(guest).unwrap();
        assert_eq!(json["lifecycleControlsEnabled"], false);
        assert_eq!(json["ownershipVerified"], false);
        assert_eq!(json["powerState"], "POWERED_ON");
        assert!(json.get("minecraftRunning").is_none());
        assert_eq!(json["ssh"]["keyForwardingReady"], true);
        assert_eq!(json["ssh"]["hostKeyPinPresent"], false);
        assert_eq!(json["ssh"]["hostKeyVerified"], false);
        assert_eq!(json["ssh"]["guestStatusObserved"], false);
        assert_eq!(json["ssh"]["minecraftCommandsEnabled"], false);
        assert_eq!(json["storage"]["activeVhdxVerified"], false);
        assert_eq!(json["storage"]["uniqueVmDirectory"], false);
        assert_eq!(json["storage"]["persistedMacObserved"], false);
        assert_eq!(json["windowsIdentityUniqueAcrossThree"], false);
    }

    #[test]
    fn ssh_forwarding_is_never_equated_with_authenticated_commands() {
        let ready = br#"{"host":"127.0.0.1","port":24000,"user":"Administrator","sshState":4,"enabled":true,"keyDeployed":true}"#;
        let parsed = parse_ssh_info(ready).unwrap();
        assert!(parsed.key_forwarding_ready);
        assert!(!parsed.host_key_verified);
        assert!(!parsed.minecraft_commands_enabled);
        for bad in [
            br#"{"host":"example.com","port":24000,"user":"Administrator","sshState":4,"enabled":true,"keyDeployed":true}"#.as_slice(),
            br#"{"host":"127.0.0.1","port":24000,"user":"Administrator","sshState":4,"enabled":true,"keyDeployed":false}"#.as_slice(),
            br#"{"host":"127.0.0.1","port":0,"user":"Administrator","sshState":4,"enabled":true,"keyDeployed":true}"#.as_slice(),
        ] {
            assert!(parse_ssh_info(bad).is_err());
        }
        let disabled = br#"{"host":"127.0.0.1","port":0,"user":"Administrator","sshState":0,"enabled":false,"keyDeployed":false}"#;
        assert!(!parse_ssh_info(disabled).unwrap().key_forwarding_ready);
    }

    #[test]
    fn disk_root_checks_are_non_authorizing_and_fail_closed() {
        assert_eq!(
            normalized_disk_root(r"C:\Users\QA\Vm1").unwrap(),
            r"c:\users\qa\vm1"
        );
        for bad in [
            r"C:relative",
            r"..\Vm1",
            r"C:\Vm1\..\Other",
            r"C:\Vm1/Other",
            r"\\server\share",
            r"C:\VM\.",
            r"C:\VM\",
        ] {
            assert!(normalized_disk_root(bad).is_none(), "{bad}");
        }
        let mut candidate = vm("Virtual-01");
        candidate["diskDirectory"] = json!(r"C:\Users\QA\Vm1");
        assert!(parse_vm(&serde_json::to_vec(&candidate).unwrap()).is_ok());
        candidate["diskDirectory"] = json!(r"C:\Users\QA\Vm1\..\Other");
        assert!(parse_vm(&serde_json::to_vec(&candidate).unwrap()).is_err());
    }

    #[test]
    fn snapshot_cursor_is_observational_and_rejects_inconsistent_positions() {
        let initial =
            br#"{"current":{"snapIndex":-1,"branchIndex":-1},"snapshots":[],"baseBranches":[]}"#;
        let base = parse_snapshot_pointer(initial).unwrap();
        assert_eq!(base.snap_index, -1);
        assert_eq!(base.branch_index, -1);
        assert!(parse_snapshot_pointer(
            br#"{"current":{"snapIndex":-1,"branchIndex":0},"snapshots":[],"baseBranches":[]}"#
        )
        .is_err());
        // Source can legitimately point at the bare base VHDX (-2/-1).
        // This is only a cursor, never a recoverable QA_READY checkpoint.
        assert!(parse_snapshot_pointer(
            br#"{"current":{"snapIndex":-2,"branchIndex":-1},"snapshots":[],"baseBranches":[]}"#
        )
        .is_ok());
        let branch = br#"{"current":{"snapIndex":-2,"branchIndex":0},"snapshots":[],"baseBranches":[{"index":0,"name":"safe"}]}"#;
        assert_eq!(parse_snapshot_pointer(branch).unwrap().snap_index, -2);
    }

    #[test]
    fn no_minecraft_or_gpu_playability_promise_is_serialized() {
        let report = super::AppSandboxInspection::unavailable("PROVIDER_UNREACHABLE");
        let output = serde_json::to_value(report).unwrap();
        assert_eq!(output["state"], "UNVERIFIED");
        assert_eq!(output["gameAndGpuVerified"], false);
        assert!(output.get("token").is_none());
    }

    #[test]
    fn http_migration_preserves_status_mapping_no_redirects_and_bounded_body() {
        use std::{
            io::{Read, Write},
            net::TcpListener,
            thread,
        };
        let agent = super::provider_agent();
        assert!(agent.config().proxy().is_none());
        assert_eq!(agent.config().max_redirects(), 0);
        assert_eq!(
            agent.config().timeouts().connect,
            Some(std::time::Duration::from_secs(2))
        );
        for (status, body, expected) in [
            (200, b"{}".to_vec(), Ok(b"{}".to_vec())),
            (401, vec![], Err("PROVIDER_AUTH_FAILED")),
            (404, vec![], Err("VM_NOT_FOUND")),
            (500, vec![], Err("PROVIDER_HTTP_ERROR")),
            (302, vec![], Err("PROVIDER_HTTP_ERROR")),
            (
                200,
                vec![b'x'; super::MAX_API_BYTES as usize + 1],
                Err("INVALID_PROVIDER_RESPONSE"),
            ),
        ] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let endpoint = format!("http://{}", listener.local_addr().unwrap());
            let server = thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                stream
                    .set_read_timeout(Some(std::time::Duration::from_secs(3)))
                    .unwrap();
                let mut request = Vec::new();
                loop {
                    let mut byte = [0];
                    stream.read_exact(&mut byte).unwrap();
                    request.push(byte[0]);
                    if request.ends_with(b"\r\n\r\n") {
                        break;
                    }
                    assert!(request.len() < 4096);
                }
                assert!(String::from_utf8(request)
                    .unwrap()
                    .to_ascii_lowercase()
                    .contains("authorization: bearer fixture-token"));
                write!(stream, "HTTP/1.1 {status} Fixture\r\nContent-Length: {}\r\nLocation: /redirect-forbidden\r\nConnection: close\r\n\r\n", body.len()).unwrap();
                let _ = stream.write_all(&body);
            });
            assert_eq!(
                super::get_json(&agent, &endpoint, "host", Some("fixture-token")),
                expected
            );
            server.join().unwrap();
        }
    }
}
