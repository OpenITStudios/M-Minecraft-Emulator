//! Explicit one-guest GPU-PV pilot only. The production provider remains VMware.
//! No VM creation, account state, snapshots, force-stop, or guest command routes.
//! A manually prepared disposable VM must match BOTH fixed identity and operator
//! consent before this narrow pilot sends start/graceful shutdown/open-display.
use super::{
    app_sandbox::{authenticated_session, normalized_disk_root, AsbSession},
    app_sandbox_registry::read_registry,
};
use crate::operation_lock::OperationLock;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    thread,
    time::{Duration, Instant},
};
use sysinfo::System;

pub(super) const PILOT_NAME: &str = "MVB-PILOT-01";
const RAM_MB: u64 = 4096;
const CPU_CORES: u64 = 2;
const WAIT_START: Duration = Duration::from_secs(90);
const WAIT_SHUTDOWN: Duration = Duration::from_secs(120);
const WAIT_DISPLAY: Duration = Duration::from_secs(10);
const MAX_REPLY_BYTES: u64 = 4096;
// Pilot allocation + host reserve, not a guarantee of gameplay performance.
const HOST_RAM_RESERVE_MB: u64 = 6 * 1024;
const HOST_FREE_DISK_FLOOR_GB: u64 = 12;
const VHDX_HEADER_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PilotAction {
    Inspect,
    Start,
    Shutdown,
    Display,
}

impl PilotAction {
    fn parse(args: &[String]) -> io::Result<Self> {
        match args {
            [x] if x == "inspect" => Ok(Self::Inspect),
            [x, flag] if flag == "--confirm-disposable-pilot" => match x.as_str() {
                "start" => Ok(Self::Start),
                "shutdown" => Ok(Self::Shutdown),
                "display" => Ok(Self::Display),
                _ => Err(invalid("unsupported pilot action")),
            },
            _ => Err(invalid(
                "pilot accepts inspect OR start|shutdown|display --confirm-disposable-pilot",
            )),
        }
    }
    fn route(self) -> Option<&'static str> {
        match self {
            Self::Start => Some("vms/MVB-PILOT-01/start"),
            Self::Shutdown => Some("vms/MVB-PILOT-01/shutdown"),
            Self::Display => Some("vms/MVB-PILOT-01/display"),
            Self::Inspect => None,
        }
    }
    fn action_name(self) -> &'static str {
        match self {
            Self::Inspect => "inspect",
            Self::Start => "start",
            Self::Shutdown => "shutdown",
            Self::Display => "display",
        }
    }
}

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}
fn unsupported(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, message)
}
fn unavailable(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::Other, message)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PilotVm {
    name: String,
    os_type: String,
    disk_directory: String,
    state: String,
    running: bool,
    building: bool,
    agent_online: bool,
    install_complete: bool,
    ram_mb: u64,
    cpu_cores: u64,
    gpu_mode: u8,
    network_mode: u8,
}

#[derive(Debug, Deserialize)]
struct VmInventory {
    vms: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Display {
    open: bool,
    ready: bool,
}

#[derive(Debug, Deserialize)]
struct PilotConsent {
    schema: u32,
    vm_name: String,
    fingerprint: String,
    disposable: bool,
    windows_entitlement_confirmed: bool,
}

#[derive(Debug, Deserialize)]
struct Accepted {
    ok: bool,
    accepted: bool,
    action: String,
    name: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PilotReport {
    name: &'static str,
    action: &'static str,
    observed_vm_state: String,
    provider_acknowledged: bool,
    observed_completion: bool,
    operator_consent_valid: bool,
    // Enrollment fingerprint pins the pilot name, active VHDX path and MAC;
    // it is NOT a cryptographic attestation of disk contents or user account.
    enrollment_fingerprint: String,
    game_and_gpu_verified: bool,
}

fn root() -> io::Result<PathBuf> {
    let program_data = PathBuf::from(
        std::env::var_os("ProgramData").ok_or_else(|| invalid("ProgramData is missing"))?,
    );
    if !program_data.is_absolute() {
        return Err(invalid("ProgramData is invalid"));
    }
    Ok(program_data
        .join("M-Bedrock")
        .join("VirtualClients")
        .join("pilots")
        .join(PILOT_NAME))
}

fn ordinary_path(path: &Path, directory: bool) -> bool {
    let Ok(meta) = fs::symlink_metadata(path) else {
        return false;
    };
    let ordinary = if directory {
        meta.is_dir()
    } else {
        meta.file_type().is_file()
    };
    if !ordinary {
        return false;
    }
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::fs::MetadataExt;
        if meta.file_attributes() & 0x400 != 0 {
            return false;
        } // reparse point
    }
    true
}

fn safe_pilot_root(path: &Path) -> bool {
    let mut cursor = path.to_path_buf();
    // Do not silently follow a substituted pilot directory or ancestor.
    loop {
        if !ordinary_path(&cursor, true) {
            return false;
        }
        if cursor.file_name().is_some_and(|name| name == "M-Bedrock") {
            break;
        }
        if !cursor.pop() {
            return false;
        }
    }
    true
}

fn valid_vhdx_signature(signature: &[u8]) -> bool {
    // MS-VHDX 2.2.1: the eight-byte file type identifier at offset 0.
    signature == b"vhdxfile"
}

fn valid_start_disk(path: &Path) -> io::Result<()> {
    // Only before Start on a stopped VM: a running HCS guest may hold an
    // exclusive disk handle. Never make Shutdown depend on opening its VHDX.
    if !ordinary_path(path, false) {
        return Err(unsupported("pilot VHDX was replaced or redirected"));
    }
    let mut file =
        fs::File::open(path).map_err(|_| unsupported("pilot VHDX cannot be read safely"))?;
    if file
        .metadata()
        .map_err(|_| unavailable("pilot VHDX metadata unavailable"))?
        .len()
        < VHDX_HEADER_BYTES
    {
        return Err(unsupported("pilot VHDX is smaller than its header section"));
    }
    let mut signature = [0_u8; 8];
    file.read_exact(&mut signature)
        .map_err(|_| unsupported("pilot VHDX identifier cannot be read"))?;
    if !valid_vhdx_signature(&signature) {
        return Err(unsupported("pilot file has no VHDX identifier"));
    }
    Ok(())
}

fn enough_memory(available_bytes: u64) -> bool {
    available_bytes / (1024 * 1024) >= RAM_MB + HOST_RAM_RESERVE_MB
}

fn start_resource_guard(session: &AsbSession) -> io::Result<()> {
    if !session.disk_space_at_least_gb(HOST_FREE_DISK_FLOOR_GB) {
        return Err(unsupported(
            "insufficient free ProgramData disk space for pilot start",
        ));
    }
    let mut system = System::new();
    system.refresh_memory();
    if !enough_memory(system.available_memory()) {
        return Err(unsupported(
            "insufficient available host RAM for pilot start",
        ));
    }
    Ok(())
}

fn fingerprint(active_vhdx: &str, mac: &str) -> String {
    let binding = format!("{PILOT_NAME}|{active_vhdx}|{mac}");
    Sha256::digest(binding.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

fn read_consent(path: &Path, fingerprint: &str) -> bool {
    if !ordinary_path(path, false) {
        return false;
    }
    if fs::metadata(path).ok().is_none_or(|m| m.len() > 1024) {
        return false;
    }
    let Ok(bytes) = fs::read(path) else {
        return false;
    };
    if bytes.len() > 1024 {
        return false;
    }
    let Ok(record) = serde_json::from_slice::<PilotConsent>(&bytes) else {
        return false;
    };
    record.schema == 1
        && record.vm_name == PILOT_NAME
        && record.disposable
        && record.windows_entitlement_confirmed
        && record.fingerprint == fingerprint
}

fn read_vm(session: &AsbSession) -> io::Result<PilotVm> {
    let inventory: VmInventory = serde_json::from_slice(&session.get("vms").map_err(unavailable)?)
        .map_err(|_| invalid("pilot VM inventory is invalid"))?;
    let found: Vec<_> = inventory
        .vms
        .iter()
        .filter(|v| {
            v.get("name")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|name| name.eq_ignore_ascii_case(PILOT_NAME))
        })
        .collect();
    if found.len() != 1
        || found[0].get("name").and_then(serde_json::Value::as_str) != Some(PILOT_NAME)
    {
        return Err(unsupported(
            "pilot name is missing, ambiguous, or differently cased",
        ));
    }
    let raw = session.get("vms/MVB-PILOT-01").map_err(unavailable)?;
    let vm: PilotVm =
        serde_json::from_slice(&raw).map_err(|_| invalid("pilot VM status is invalid"))?;
    let inventory_parent = found[0]
        .get("diskDirectory")
        .and_then(serde_json::Value::as_str)
        .and_then(normalized_disk_root)
        .ok_or_else(|| invalid("pilot inventory storage is invalid"))?;
    if vm.name != PILOT_NAME
        || vm.os_type != "Windows"
        || normalized_disk_root(&vm.disk_directory).as_deref() != Some(inventory_parent.as_str())
    {
        return Err(unsupported("pilot VM is not the expected Windows machine"));
    }
    Ok(vm)
}

fn current_state_consistent(vm: &PilotVm) -> bool {
    match vm.state.as_str() {
        "stopped" => !vm.running && !vm.building,
        "installing" => vm.running && !vm.building && !vm.install_complete,
        "booting" => vm.running && !vm.building && vm.install_complete && !vm.agent_online,
        "online" => vm.running && !vm.building && vm.install_complete && vm.agent_online,
        "stopping" => vm.running && !vm.building,
        "building" => vm.building,
        _ => false,
    }
}

// A bare base disk can report (-1,-1) or (-2,-1) in upstream v0.1.9.
// Anything else or any recorded branches/snapshots blocks a pilot mutation.
fn snapshot_is_clean(value: &serde_json::Value) -> bool {
    let no_snapshots = value
        .get("snapshots")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|rows| rows.is_empty());
    let no_branches = value
        .get("baseBranches")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|rows| rows.is_empty());
    let cursor = value.get("current");
    let snap = cursor
        .and_then(|c| c.get("snapIndex"))
        .and_then(serde_json::Value::as_i64);
    let branch = cursor
        .and_then(|c| c.get("branchIndex"))
        .and_then(serde_json::Value::as_i64);
    no_snapshots && no_branches && branch == Some(-1) && matches!(snap, Some(-1) | Some(-2))
}

struct Preflight {
    vm: PilotVm,
    enrollment_fingerprint: String,
    consent: bool,
    registry_disk: String,
    registry_mac: String,
}

fn preflight(session: &AsbSession) -> io::Result<Preflight> {
    let vm = read_vm(session)?;
    if !current_state_consistent(&vm)
        || vm.building
        || vm.ram_mb != RAM_MB
        || vm.cpu_cores != CPU_CORES
        || vm.gpu_mode != 1
        || vm.network_mode != 1
    {
        return Err(unsupported(
            "pilot configuration or VM phase is not eligible",
        ));
    }
    let expected = root()?;
    if !safe_pilot_root(&expected) {
        return Err(unsupported("pilot folder or parent is missing or unsafe"));
    }
    let root_key = normalized_disk_root(&expected.to_string_lossy())
        .ok_or_else(|| invalid("pilot root is invalid"))?;
    let expected_vhdx = format!("{root_key}\\disk.vhdx");
    let parent = root_key
        .rsplit_once('\\')
        .ok_or_else(|| invalid("pilot root has no parent"))?
        .0;
    let before = read_registry().map_err(|_| unavailable("App Sandbox registry is unavailable"))?;
    let observed = before
        .vm(PILOT_NAME)
        .ok_or_else(|| unsupported("disposable pilot is not registered"))?;
    if observed.name != PILOT_NAME
        || observed.vm_root != root_key
        || observed.shared_parent != parent
        || observed.active_vhdx != expected_vhdx
        || normalized_disk_root(&vm.disk_directory).as_deref() != Some(parent)
        || observed.mac.is_none()
    {
        return Err(unsupported("pilot storage and MAC provenance do not match"));
    }
    let disk = expected.join("disk.vhdx");
    if !ordinary_path(&disk, false) || fs::metadata(&disk).ok().is_none_or(|m| m.len() < 4096) {
        return Err(unsupported(
            "pilot VHDX is unavailable, empty or redirected",
        ));
    }
    // No snapshot or differencing branches for the disposable one-guest pilot.
    let raw = session
        .get("vms/MVB-PILOT-01/snapshots")
        .map_err(unavailable)?;
    let snapshots: serde_json::Value =
        serde_json::from_slice(&raw).map_err(|_| invalid("pilot snapshots response is invalid"))?;
    if !snapshot_is_clean(&snapshots) {
        return Err(unsupported(
            "pilot has a snapshot branch; no operation was attempted",
        ));
    }
    let after = read_registry()
        .map_err(|_| unavailable("App Sandbox registry changed or is unavailable"))?;
    if before != after {
        return Err(unavailable(
            "App Sandbox registry changed during pilot preflight",
        ));
    }
    let mac = observed
        .mac
        .as_deref()
        .ok_or_else(|| unsupported("pilot MAC is absent"))?;
    let digest = fingerprint(&observed.active_vhdx, mac);
    let consent = read_consent(&expected.join("pilot-consent.json"), &digest);
    Ok(Preflight {
        vm,
        enrollment_fingerprint: digest,
        consent,
        registry_disk: observed.active_vhdx.clone(),
        registry_mac: mac.to_string(),
    })
}

fn refuse_action(action: PilotAction, pre: &Preflight) -> io::Result<()> {
    if action == PilotAction::Inspect {
        return Ok(());
    }
    if !pre.consent {
        return Err(unsupported(
            "explicit disposable-pilot consent is missing or mismatched",
        ));
    }
    match action {
        PilotAction::Start if pre.vm.state == "stopped" => Ok(()),
        PilotAction::Shutdown if pre.vm.running && pre.vm.state != "stopping" => Ok(()),
        PilotAction::Display if pre.vm.state == "online" => Ok(()),
        _ => Err(unsupported(
            "pilot action is ineligible from current VM phase",
        )),
    }
}

fn assert_same_preflight(previous: &Preflight, current: &Preflight) -> io::Result<()> {
    if current.registry_disk != previous.registry_disk
        || current.registry_mac != previous.registry_mac
        || current.vm.state != previous.vm.state
        || current.enrollment_fingerprint != previous.enrollment_fingerprint
        || !current.consent
    {
        return Err(unavailable(
            "pilot provenance or phase changed before the operation",
        ));
    }
    Ok(())
}

fn post_fixed(session: &AsbSession, action: PilotAction) -> io::Result<()> {
    let route = action
        .route()
        .ok_or_else(|| invalid("pilot inspect cannot be submitted"))?;
    // Exact known route and VM name only. No delete, stop (hard-off),
    // reprovision, snapshot, arbitrary command, or redirect/proxy.
    let url = format!("{}/v1/{route}", session.endpoint);
    let header = format!("Bearer {}", session.token);
    let result = session
        .agent
        .post(&url)
        .header("Authorization", &header)
        .send_json(&serde_json::json!({}));
    let response = match result {
        Ok(response) => response,
        Err(_) => {
            return Err(unavailable(
                "pilot action outcome unknown; inspect before retrying",
            ))
        }
    };
    let code = response.status();
    let mut bytes = Vec::new();
    response
        .into_body()
        .into_reader()
        .take(MAX_REPLY_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| unavailable("pilot acknowledgement unreadable; inspect before retrying"))?;
    if bytes.len() as u64 > MAX_REPLY_BYTES {
        return Err(unavailable(
            "pilot acknowledgement invalid; inspect before retrying",
        ));
    }
    match action {
        PilotAction::Start | PilotAction::Shutdown => {
            let ack: Accepted = serde_json::from_slice(&bytes).map_err(|_| {
                unavailable("pilot acknowledgement malformed; inspect before retrying")
            })?;
            if code != 202
                || !ack.ok
                || !ack.accepted
                || ack.action != action.action_name()
                || ack.name != PILOT_NAME
            {
                return Err(unavailable(
                    "pilot action was not confirmed accepted; inspect before retrying",
                ));
            }
        }
        PilotAction::Display => {
            let body: serde_json::Value = serde_json::from_slice(&bytes)
                .map_err(|_| unavailable("pilot display acknowledgement malformed"))?;
            if code != 200
                || body.get("ok").and_then(serde_json::Value::as_bool) != Some(true)
                || body.get("displayOpen").and_then(serde_json::Value::as_bool) != Some(true)
            {
                return Err(unavailable("pilot display could not be confirmed"));
            }
        }
        PilotAction::Inspect => return Err(invalid("invalid pilot mutation")),
    }
    Ok(())
}

fn read_display(session: &AsbSession) -> io::Result<Display> {
    let data = session
        .get("vms/MVB-PILOT-01/display")
        .map_err(unavailable)?;
    serde_json::from_slice(&data).map_err(|_| invalid("pilot display status is invalid"))
}

fn completion(
    session: &AsbSession,
    action: PilotAction,
    baseline: &Preflight,
) -> io::Result<String> {
    let limit = match action {
        PilotAction::Start => WAIT_START,
        PilotAction::Shutdown => WAIT_SHUTDOWN,
        PilotAction::Display => WAIT_DISPLAY,
        PilotAction::Inspect => return Err(invalid("inspect does not await a mutation")),
    };
    let since = Instant::now();
    loop {
        let vm = read_vm(session)
            .map_err(|_| unavailable("pilot state unknown after accepted action"))?;
        let matched = match action {
            // A received HTTP 202 and an OS boot phase are NOT an online guest.
            // Even online is only the App Sandbox agent, not Minecraft gameplay.
            PilotAction::Start => {
                vm.running && vm.state == "online" && current_state_consistent(&vm)
            }
            PilotAction::Shutdown => !vm.running && vm.state == "stopped",
            PilotAction::Display => {
                let display = read_display(session).map_err(|_| {
                    unavailable("pilot display state unknown after accepted action")
                })?;
                display.open && display.ready && vm.running
            }
            PilotAction::Inspect => false,
        };
        if matched {
            // Reconcile the registry, MAC, direct VHDX path, snapshot pointer,
            // consent and current host configuration before reporting success.
            // A changed or unreadable identity after POST is UNKNOWN, not PASS.
            let after = preflight(session)
                .map_err(|_| unavailable("pilot identity could not be revalidated after action"))?;
            if after.registry_disk != baseline.registry_disk
                || after.registry_mac != baseline.registry_mac
                || after.enrollment_fingerprint != baseline.enrollment_fingerprint
                || !after.consent
                || after.vm.state != vm.state
            {
                return Err(unavailable("pilot identity changed after accepted action"));
            }
            return Ok(vm.state);
        }
        if since.elapsed() >= limit {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "pilot completion not observed; do not retry before inspecting current VM state",
            ));
        }
        thread::sleep(Duration::from_secs(2));
    }
}

pub(crate) fn execute(args: &[String]) -> io::Result<PilotReport> {
    let action = PilotAction::parse(args)?;
    let _lock = if action == PilotAction::Inspect {
        OperationLock::acquire_shared()?
    } else {
        OperationLock::acquire()?
    };
    let session = authenticated_session().map_err(unavailable)?;
    let before = preflight(&session)?;
    refuse_action(action, &before)?;
    if action == PilotAction::Inspect {
        return Ok(PilotReport {
            name: PILOT_NAME,
            action: "inspect",
            observed_vm_state: before.vm.state,
            provider_acknowledged: false,
            observed_completion: false,
            operator_consent_valid: before.consent,
            enrollment_fingerprint: before.enrollment_fingerprint,
            game_and_gpu_verified: false,
        });
    }
    if action == PilotAction::Display && !read_display(&session)?.ready {
        return Err(unsupported("pilot display driver has not reported ready"));
    }
    // Preflight directly before the single network mutation. Even if a later
    // request fails, do not repeat it or attempt forceful rollback.
    let again = preflight(&session)?;
    assert_same_preflight(&before, &again)?;
    if action == PilotAction::Start {
        valid_start_disk(&root()?.join("disk.vhdx"))?;
        start_resource_guard(&session)?;
    }
    // Low host RAM and locked guest disks must NEVER block graceful Shutdown.
    post_fixed(&session, action)?;
    let observed = completion(&session, action, &before)?;
    Ok(PilotReport {
        name: PILOT_NAME,
        action: action.action_name(),
        observed_vm_state: observed,
        provider_acknowledged: true,
        observed_completion: true,
        operator_consent_valid: true,
        enrollment_fingerprint: before.enrollment_fingerprint,
        game_and_gpu_verified: false,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        current_state_consistent, enough_memory, snapshot_is_clean, valid_vhdx_signature,
        PilotAction, PilotConsent, PilotVm, PILOT_NAME,
    };
    use serde_json::json;
    #[test]
    fn pilot_action_never_touches_production_names_or_force_stop() {
        assert_eq!(PILOT_NAME, "MVB-PILOT-01");
        assert_eq!(PilotAction::Start.route(), Some("vms/MVB-PILOT-01/start"));
        assert_eq!(
            PilotAction::Shutdown.route(),
            Some("vms/MVB-PILOT-01/shutdown")
        );
        assert_eq!(
            PilotAction::Display.route(),
            Some("vms/MVB-PILOT-01/display")
        );
        assert!(PilotAction::parse(&["start".into()]).is_err());
        assert!(
            PilotAction::parse(&["delete".into(), "--confirm-disposable-pilot".into()]).is_err()
        );
        assert!(PilotAction::parse(&["stop".into(), "--confirm-disposable-pilot".into()]).is_err());
        assert!(
            PilotAction::parse(&["reset".into(), "--confirm-disposable-pilot".into()]).is_err()
        );
        assert!(
            PilotAction::parse(&["Virtual-01".into(), "--confirm-disposable-pilot".into()])
                .is_err()
        );
    }
    #[test]
    fn online_requires_installed_guest_and_connected_agent() {
        let mut vm = PilotVm {
            name: PILOT_NAME.into(),
            os_type: "Windows".into(),
            disk_directory: r"C:\pilot".into(),
            state: "online".into(),
            running: true,
            building: false,
            agent_online: true,
            install_complete: true,
            ram_mb: 4096,
            cpu_cores: 2,
            gpu_mode: 1,
            network_mode: 1,
        };
        assert!(current_state_consistent(&vm));
        vm.agent_online = false;
        assert!(!current_state_consistent(&vm));
        vm.state = "booting".into();
        assert!(current_state_consistent(&vm));
        vm.install_complete = false;
        assert!(!current_state_consistent(&vm));
    }
    #[test]
    fn snapshot_is_clean_only_for_an_unbranched_base() {
        for (snap, branch, valid) in [
            (-1, -1, true),
            (-2, -1, true),
            (0, -1, false),
            (-1, 0, false),
        ] {
            let value = json!({"current":{"snapIndex":snap,"branchIndex":branch},
                "snapshots":[],"baseBranches":[]});
            assert_eq!(snapshot_is_clean(&value), valid);
        }
        assert!(!snapshot_is_clean(
            &json!({"current":{"snapIndex":-1,"branchIndex":-1},
            "snapshots":[{"index":0}],"baseBranches":[]})
        ));
        assert!(!snapshot_is_clean(
            &json!({"current":{"snapIndex":-2,"branchIndex":-1},
            "snapshots":[],"baseBranches":[{"index":0}]})
        ));
    }
    #[test]
    fn pilot_start_requires_vhdx_format_and_host_memory_headroom() {
        assert!(valid_vhdx_signature(b"vhdxfile"));
        assert!(!valid_vhdx_signature(b"NOT_VHDX"));
        let minimum = (4096_u64 + 6144) * 1024 * 1024;
        assert!(!enough_memory(minimum - 1));
        assert!(enough_memory(minimum));
    }

    #[test]
    fn operator_consent_is_not_an_automatic_authorization() {
        let record: PilotConsent = serde_json::from_str(
            r#"{"schema":1,"vm_name":"MVB-PILOT-01","fingerprint":"ab","disposable":true,"windows_entitlement_confirmed":true}"#
        ).unwrap();
        assert!(record.disposable && record.windows_entitlement_confirmed);
        assert!(PilotAction::parse(&["inspect".into()]).is_ok());
    }
}
