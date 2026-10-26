mod app_sandbox;
mod app_sandbox_pilot;
mod app_sandbox_registry;
mod app_sandbox_ssh;
#[cfg(target_os = "macos")]
mod fusion;
mod process;
mod storage;
mod vmx;
#[cfg(target_os = "windows")]
mod workstation;

pub(crate) use app_sandbox_pilot::execute as app_sandbox_pilot_command;

pub(crate) use app_sandbox::inspect as inspect_app_sandbox;

pub(crate) use process::{
    command_output, command_output_with_stdin, command_output_with_timeout, wait_for_state,
};
pub(crate) use storage::{
    cleanup_staging, ensure_parent, has_suspend_state, promote_staging_vm, remove_vm_container,
    staging_residue_count,
};
pub(crate) use vmx::{
    apply_virtual_hardware_policy, base_state_for_path, ensure_guest_token_for_path,
    guest_token_for_path, read_vmx_memory, read_vmx_value, set_base_state_for_path,
    vm_identity_key, vmx_file_present,
};

use crate::{
    client::{ClientId, ClientState},
    guest::{launch_guest_minecraft, query_guest_status, GuestStatus},
    paths::{client_root, staging_root},
    profile::{current_base_vmx_path, BaseState},
};
use std::{
    io,
    net::IpAddr,
    path::{Path, PathBuf},
    time::Duration,
};
use sysinfo::System;

#[cfg(target_os = "macos")]
use fusion::VmwareFusionProvider;
#[cfg(target_os = "windows")]
use workstation::VmwareWorkstationProvider;

pub(crate) const DISK_STATE_TIMEOUT: Duration = Duration::from_secs(180);

pub trait Provider {
    fn id(&self) -> &'static str;
    fn version(&self) -> Option<String>;
    fn detect(&self) -> bool;
    fn provision(&self, client: ClientId) -> io::Result<ClientState>;
    fn reprovision(&self, client: ClientId) -> io::Result<ClientState>;
    fn memory_limit_mb(&self, client: ClientId) -> io::Result<u64>;
    // Every backend must provide its own resource evidence. The old default
    // inspected VMware processes and would silently misreport HCS GPU-PV.
    fn host_working_sets_mb(&self) -> io::Result<Vec<(ClientId, u64)>>;
    fn guest_tools_ready(&self, client: ClientId) -> io::Result<Option<bool>>;
    // Provider-owned guest transport. HCS must never inherit VMware VMX/IP/token
    // assumptions or promote its own agent-online signal to Minecraft readiness.
    fn guest_status(&self, client: ClientId, timeout: Duration) -> io::Result<Option<GuestStatus>>;
    fn launch_minecraft(&self, client: ClientId, timeout: Duration) -> io::Result<()>;
    // Identity stays UNKNOWN without a verified, independently unique UUID|MAC.
    fn identity_key(&self, client: ClientId) -> io::Result<Option<String>>;

    // A provider-neutral admission fact, rather than looking for .vmx from
    // runtime.rs. HCS templates will have a distinct provenance contract.
    fn base_finalized_and_stopped(&self) -> io::Result<bool>;
    fn network_mode(&self, client: ClientId) -> io::Result<Option<String>>;
    // Configuration only; NOT evidence of a working D3D11/OpenGL renderer.
    fn graphics_3d_enabled(&self, client: ClientId) -> io::Result<Option<bool>>;
    fn status(&self, client: ClientId) -> io::Result<ClientState>;
    // One read-only observation per request. Providers may batch external
    // status discovery; never cache across independent operations/mutations.
    fn observe_client_states(&self) -> io::Result<Vec<(ClientId, ClientState)>> {
        ClientId::VIRTUAL
            .into_iter()
            .map(|client| self.status(client).map(|state| (client, state)))
            .collect()
    }
    fn start(&self, client: ClientId) -> io::Result<ClientState>;
    fn suspend(&self, client: ClientId) -> io::Result<ClientState>;
    fn stop(&self, client: ClientId) -> io::Result<ClientState>;
    fn restart(&self, client: ClientId) -> io::Result<ClientState>;
    fn set_ready(&self, client: ClientId) -> io::Result<ClientState>;
    fn has_ready(&self, client: ClientId) -> io::Result<bool>;
    fn reset(&self, client: ClientId) -> io::Result<ClientState>;
    fn open(&self, client: ClientId) -> io::Result<ClientState>;
    fn open_vm_ui(&self, vmx: &Path) -> io::Result<()>;
    fn is_running_path(&self, vmx: &Path) -> io::Result<bool>;
    fn start_validation_vm(&self, vmx: &Path) -> io::Result<()>;
    fn stop_validation_vm(&self, vmx: &Path) -> io::Result<()>;
    fn guest_ip_for_path(&self, vmx: &Path) -> io::Result<Option<String>>;
}

// VMware-specific metadata stays below the provider boundary. Future HCS
// implementations must not inherit VMX paths or VMware process heuristics.
fn vmware_base_finalized_and_stopped(provider: &dyn Provider) -> io::Result<bool> {
    let base = current_base_vmx_path()?;
    if base_state_for_path(&base)? != Some(BaseState::Finalized) {
        return Ok(false);
    }
    Ok(!provider.is_running_path(&base)?)
}

fn vmware_network_mode(client: ClientId) -> io::Result<Option<String>> {
    read_vmx_value(&client_vmx_path(client)?, "ethernet0.connectionType")
}

fn vmware_graphics_3d_enabled(client: ClientId) -> io::Result<Option<bool>> {
    Ok(read_vmx_value(&client_vmx_path(client)?, "mks.enable3d")?
        .map(|value| value.eq_ignore_ascii_case("TRUE")))
}

pub fn current_platform_provider() -> Option<Box<dyn Provider>> {
    #[cfg(target_os = "windows")]
    {
        let provider = VmwareWorkstationProvider::default();
        return provider
            .detect()
            .then(|| Box::new(provider) as Box<dyn Provider>);
    }

    #[cfg(target_os = "macos")]
    {
        let provider = VmwareFusionProvider::default();
        return provider
            .detect()
            .then(|| Box::new(provider) as Box<dyn Provider>);
    }

    #[allow(unreachable_code)]
    None
}

pub(crate) fn base_vmx_path() -> io::Result<PathBuf> {
    current_base_vmx_path()
}

pub(crate) fn client_vmx_path(client: ClientId) -> io::Result<PathBuf> {
    client_vmx_path_under(client, client_root()?)
}

pub(crate) fn staging_client_vmx_path(client: ClientId) -> io::Result<PathBuf> {
    let root = staging_root()?;
    let name = format!("{}-{}", client.as_str(), std::process::id());

    #[cfg(target_os = "windows")]
    {
        return Ok(root.join(&name).join(format!("{}.vmx", client.as_str())));
    }

    #[cfg(target_os = "macos")]
    {
        return Ok(root
            .join(format!("{name}.vmwarevm"))
            .join(format!("{}.vmx", client.as_str())));
    }

    #[allow(unreachable_code)]
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "unsupported platform",
    ))
}

fn client_vmx_path_under(client: ClientId, root: PathBuf) -> io::Result<PathBuf> {
    if client.is_native() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "native client has no VMX path",
        ));
    }

    let name = client.as_str();

    #[cfg(target_os = "windows")]
    {
        return Ok(root.join(name).join(format!("{name}.vmx")));
    }

    #[cfg(target_os = "macos")]
    {
        return Ok(root
            .join(format!("{name}.vmwarevm"))
            .join(format!("{name}.vmx")));
    }

    #[allow(unreachable_code)]
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "unsupported platform",
    ))
}

pub(crate) fn listed_as_running(list_output: &str, vmx: &Path) -> bool {
    let target = vmx.to_string_lossy();
    list_output
        .lines()
        .map(str::trim)
        .any(|line| line.eq_ignore_ascii_case(&target))
}

pub(crate) fn parse_guest_ip(output: &str) -> Option<String> {
    output.lines().find_map(|line| {
        let value = line.trim();
        value
            .parse::<IpAddr>()
            .ok()
            .map(|address| address.to_string())
    })
}

pub(crate) fn snapshot_list_contains(list_output: &str, name: &str) -> bool {
    list_output.lines().map(str::trim).any(|line| line == name)
}

// Legacy VMware Guest Agent endpoint stays inside provider ownership.
// The Windows HCS backend must supply a separately verified guest mechanism.
fn vmware_guest_connection(
    provider: &dyn Provider,
    client: ClientId,
) -> io::Result<Option<(String, String)>> {
    let vmx = client_vmx_path(client)?;
    let Some(ip) = provider.guest_ip_for_path(&vmx)? else {
        return Ok(None);
    };
    let token = guest_token_for_path(&vmx)?.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} VMware Guest Agent token is missing", client.as_str()),
        )
    })?;
    Ok(Some((ip, token)))
}

fn vmware_guest_status(
    provider: &dyn Provider,
    client: ClientId,
    timeout: Duration,
) -> io::Result<Option<GuestStatus>> {
    let Some((ip, token)) = vmware_guest_connection(provider, client)? else {
        return Ok(None);
    };
    query_guest_status(&ip, &token, timeout).map(Some)
}

fn vmware_launch_minecraft(
    provider: &dyn Provider,
    client: ClientId,
    timeout: Duration,
) -> io::Result<()> {
    let (ip, token) = vmware_guest_connection(provider, client)?.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotConnected,
            format!("{} VMware Guest Agent is unreachable", client.as_str()),
        )
    })?;
    launch_guest_minecraft(&ip, &token, timeout).map(|_| ())
}

pub(crate) fn guest_tools_state_ready(state: &str) -> bool {
    state.trim().eq_ignore_ascii_case("running")
}

pub(crate) fn host_working_sets_mb() -> Vec<(ClientId, u64)> {
    let system = System::new_all();
    let mut result = Vec::new();

    for client in ClientId::VIRTUAL {
        let Ok(vmx) = client_vmx_path(client) else {
            continue;
        };
        if !vmx.is_file() {
            continue;
        }

        let target = vmx.to_string_lossy().to_ascii_lowercase();
        let bytes = system
            .processes()
            .values()
            .filter(|process| {
                process
                    .cmd()
                    .iter()
                    .any(|arg| arg.to_string_lossy().to_ascii_lowercase().contains(&target))
            })
            .map(|process| process.memory())
            .sum::<u64>();

        if bytes > 0 {
            result.push((client, bytes / 1024 / 1024));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::vmx::{valid_guest_token, BASE_STATE_KEY, GUEST_TOKEN_KEY};
    use super::{
        apply_virtual_hardware_policy, guest_token_for_path, guest_tools_state_ready,
        listed_as_running, parse_guest_ip, snapshot_list_contains,
    };
    use crate::profile::BaseState;
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn detects_running_vm_from_vmrun_list() {
        let output = "Total running VMs: 1\nC:\\Lab\\Virtual-01\\Virtual-01.vmx\n";
        assert!(listed_as_running(
            output,
            std::path::Path::new(r"C:\Lab\Virtual-01\Virtual-01.vmx")
        ));
    }

    #[test]
    fn ready_snapshot_match_is_exact() {
        let output = "Total snapshots: 2\nQA_READY\nBefore Update\n";
        assert!(snapshot_list_contains(output, "QA_READY"));
        assert!(!snapshot_list_contains(output, "QA"));
    }

    #[test]
    fn tools_state_requires_running() {
        assert!(guest_tools_state_ready("running"));
        assert!(guest_tools_state_ready("Running\n"));
        assert!(!guest_tools_state_ready("installed"));
        assert!(!guest_tools_state_ready("not running"));
    }

    #[test]
    fn guest_ip_parser_requires_real_ip() {
        assert_eq!(
            parse_guest_ip("192.168.10.42\n"),
            Some("192.168.10.42".into())
        );
        assert_eq!(parse_guest_ip("Error: Tools not ready"), None);
    }

    #[test]
    fn vm_identity_requires_both_uuid_and_mac() {
        let root = unique_temp_dir("vm-identity");
        fs::create_dir_all(&root).unwrap();

        let vmx = root.join("Virtual-01.vmx");
        fs::write(
            &vmx,
            "uuid.bios = \"uuid-1\"\nethernet0.generatedAddress = \"00:50:56:AA:BB:CC\"\n",
        )
        .unwrap();
        assert_eq!(
            super::vm_identity_key(&vmx).unwrap().as_deref(),
            Some("uuid-1|00:50:56:AA:BB:CC")
        );

        fs::write(&vmx, "uuid.bios = \"uuid-1\"\n").unwrap();
        assert_eq!(super::vm_identity_key(&vmx).unwrap(), None);

        fs::write(&vmx, "ethernet0.generatedAddress = \"00:50:56:AA:BB:CC\"\n").unwrap();
        assert_eq!(super::vm_identity_key(&vmx).unwrap(), None);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn guest_token_format_is_strict() {
        assert!(valid_guest_token(&"a".repeat(64)));
        assert!(!valid_guest_token("short"));
        assert!(!valid_guest_token(&"z".repeat(64)));
    }

    #[test]
    fn cloned_vmx_normalizes_duplicate_keys_and_guest_token() {
        let root = unique_temp_dir("vmx-key-normalization");
        fs::create_dir_all(&root).unwrap();
        let vmx = root.join("Virtual-01.vmx");
        let base_token = "a".repeat(64);
        fs::write(
            &vmx,
            format!(
                r#"# Keep this comment
MEMSIZE="1024"
memsize = "2048"
memsizeExtra = "KEEP"
GUESTINFO.VIRTUALCLIENTS.TOKEN = "{base_token}"
guestinfo.virtualclients.token="{base_token}"
GuestInfo.VirtualClients.BaseState = "FINALIZED"
"#
            ),
        )
        .unwrap();

        assert_eq!(
            super::read_vmx_value(&vmx, "memsize").unwrap_err().kind(),
            std::io::ErrorKind::InvalidData
        );
        apply_virtual_hardware_policy(&vmx).unwrap();

        assert_eq!(
            super::read_vmx_value(&vmx, "memsize").unwrap().as_deref(),
            Some("4096")
        );
        assert_eq!(
            super::read_vmx_value(&vmx, "memsizeExtra")
                .unwrap()
                .as_deref(),
            Some("KEEP")
        );
        assert!(super::base_state_for_path(&vmx).unwrap().is_none());
        let rotated = guest_token_for_path(&vmx).unwrap().unwrap();
        assert!(valid_guest_token(&rotated));
        assert_ne!(rotated, base_token);

        let content = fs::read_to_string(&vmx).unwrap();
        for key in ["memsize", "guestinfo.virtualclients.token"] {
            assert_eq!(
                content
                    .lines()
                    .filter(|line| line
                        .split_once('=')
                        .is_some_and(|(name, _)| name.trim().eq_ignore_ascii_case(key)))
                    .count(),
                1
            );
        }
        assert!(content.contains("# Keep this comment"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn interrupted_vmx_metadata_write_recovers_before_presence_check() {
        let root = unique_temp_dir("vmx-rename-recovery");
        fs::create_dir_all(&root).unwrap();
        let path = root.join("Virtual-01.vmx");
        let backup = root.join(".Virtual-01.vmx.bak");
        fs::write(&backup, "memsize = \"4096\"\n").unwrap();

        assert!(super::vmx_file_present(&path).unwrap());
        assert!(path.is_file());
        assert!(!backup.exists());
        assert_eq!(
            super::read_vmx_value(&path, "memsize").unwrap().as_deref(),
            Some("4096")
        );

        super::set_base_state_for_path(&path, BaseState::Registered).unwrap();
        assert_eq!(
            super::base_state_for_path(&path).unwrap(),
            Some(BaseState::Registered)
        );
        assert_eq!(
            super::read_vmx_value(&path, "memsize").unwrap().as_deref(),
            Some("4096")
        );
        assert!(!backup.exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn base_state_is_single_canonical_vmx_key() {
        let root = unique_temp_dir("base-state");
        fs::create_dir_all(&root).unwrap();
        let vmx = root.join("Base.vmx");
        fs::write(&vmx, "config.version = \"8\"\n").unwrap();

        super::set_base_state_for_path(&vmx, BaseState::Registered).unwrap();
        assert_eq!(
            super::base_state_for_path(&vmx).unwrap(),
            Some(BaseState::Registered)
        );
        assert!(fs::read_to_string(&vmx).unwrap().contains(BASE_STATE_KEY));

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn virtual_hardware_policy_rotates_inherited_guest_tokens_per_clone() {
        let root = unique_temp_dir("guest-token-rotation");
        fs::create_dir_all(&root).unwrap();

        let inherited = "a".repeat(64);
        let base = root.join("Base.vmx");
        fs::write(
            &base,
            format!("{GUEST_TOKEN_KEY} = \"{inherited}\"\n{BASE_STATE_KEY} = \"FINALIZED\"\n"),
        )
        .unwrap();

        let mut virtual_tokens = Vec::new();
        for name in ["Virtual-01", "Virtual-02", "Virtual-03"] {
            let vmx = root.join(format!("{name}.vmx"));
            fs::copy(&base, &vmx).unwrap();
            apply_virtual_hardware_policy(&vmx).unwrap();

            let token = guest_token_for_path(&vmx).unwrap().unwrap();
            assert!(valid_guest_token(&token));
            assert_ne!(token, inherited);
            assert_eq!(super::base_state_for_path(&vmx).unwrap(), None);
            virtual_tokens.push(token);
        }

        assert_eq!(
            guest_token_for_path(&base).unwrap().as_deref(),
            Some(inherited.as_str())
        );
        assert_eq!(
            super::base_state_for_path(&base).unwrap(),
            Some(BaseState::Finalized)
        );
        assert_ne!(virtual_tokens[0], virtual_tokens[1]);
        assert_ne!(virtual_tokens[0], virtual_tokens[2]);
        assert_ne!(virtual_tokens[1], virtual_tokens[2]);

        fs::remove_dir_all(root).unwrap();
    }

    fn unique_temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "m-bedrock-virtual-clients-{label}-{}-{nonce}",
            std::process::id()
        ))
    }
}
