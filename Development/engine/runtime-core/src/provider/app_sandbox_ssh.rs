//! Only pinned, read-only guest status over App Sandbox localhost SSH.
//! SSH host keys must be enrolled out-of-band using the intended guest console.
//! This never enrolls trust, sends a game launch, or mutates a VM.
use super::process::command_output_with_timeout;
use crate::guest::{GuestStatus, GUEST_AGENT_PROTOCOL_VERSION, GUEST_STATUS_SCHEMA};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};

#[derive(Debug, Default)]
pub(super) struct SshProof {
    pub pin_present: bool,
    pub host_key_verified: bool,
    pub guest_status_observed: bool,
    // Never serialize raw or hashed MachineGuid to UI, logs or Git.
    pub machine_identity: Option<String>,
}

const MAX_TRUST_BYTES: u64 = 8192;
const MAX_RESPONSE_BYTES: u64 = 16384;
const MAX_EXECUTION: Duration = Duration::from_secs(12);

fn alias(name: &str) -> Option<&'static str> {
    match name {
        "Virtual-01" => Some("mivubi-virtual-01"),
        "Virtual-02" => Some("mivubi-virtual-02"),
        "Virtual-03" => Some("mivubi-virtual-03"),
        _ => None,
    }
}

fn plain_file(path: &Path) -> bool {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return false;
    };
    if !metadata.file_type().is_file() {
        return false;
    }
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return false;
        }
    }
    true
}

fn parse_pins(text: &str) -> Option<HashMap<&str, &str>> {
    let mut entries = HashMap::new();
    let mut keys = HashSet::new();
    for line in text.lines() {
        let value = line.trim();
        if value.is_empty() || value.starts_with('#') {
            continue;
        }
        let items: Vec<_> = value.split_ascii_whitespace().collect();
        if items.len() != 3
            || ![
                "mivubi-virtual-01",
                "mivubi-virtual-02",
                "mivubi-virtual-03",
            ]
            .contains(&items[0])
            || items[1] != "ssh-ed25519"
            || !(48..=200).contains(&items[2].len())
            || !items[2]
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'+' || c == b'/' || c == b'=')
            || entries.insert(items[0], items[2]).is_some()
            || !keys.insert(items[2])
        {
            return None;
        }
    }
    Some(entries)
}

#[cfg(target_os = "windows")]
fn trust_file(name: &str) -> Option<PathBuf> {
    use std::os::windows::fs::MetadataExt;
    let required = alias(name)?;
    let root = PathBuf::from(std::env::var_os("LOCALAPPDATA")?);
    if !root.is_absolute() {
        return None;
    }
    let directory = root.join("M-Bedrock").join("VirtualClients").join("trust");
    let metadata = fs::symlink_metadata(&directory).ok()?;
    if !metadata.is_dir() || metadata.file_attributes() & 0x400 != 0 {
        return None;
    }
    let path = directory.join("app-sandbox-known-hosts");
    if !plain_file(&path) || fs::metadata(&path).ok()?.len() > MAX_TRUST_BYTES {
        return None;
    }
    let bytes = fs::read(&path).ok()?;
    if bytes.len() as u64 > MAX_TRUST_BYTES {
        return None;
    }
    let text = std::str::from_utf8(&bytes).ok()?;
    parse_pins(text)?.get(required)?;
    Some(path)
}

#[cfg(not(target_os = "windows"))]
fn trust_file(_name: &str) -> Option<PathBuf> {
    None
}

fn valid_user(user: &str) -> bool {
    !user.is_empty()
        && user.len() <= 20
        && user
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_' || c == b'.')
}

fn fixed_ssh_command(
    ssh: &Path,
    hosts: &Path,
    key: &Path,
    name: &str,
    user: &str,
    port: u16,
) -> Option<Command> {
    let alias = alias(name)?;
    if port == 0 || !valid_user(user) {
        return None;
    }
    let mut command = Command::new(ssh);
    command.arg("-F").arg("none").arg("-T");
    for option in [
        "BatchMode=yes",
        "StrictHostKeyChecking=yes",
        "UpdateHostKeys=no",
        "HostKeyAlgorithms=ssh-ed25519",
        "PreferredAuthentications=publickey",
        "PasswordAuthentication=no",
        "KbdInteractiveAuthentication=no",
        "IdentitiesOnly=yes",
        "IdentityAgent=none",
        "GlobalKnownHostsFile=none",
        "CheckHostIP=no",
        "VerifyHostKeyDNS=no",
        "ClearAllForwardings=yes",
        "ForwardAgent=no",
        "ProxyCommand=none",
        "ProxyJump=none",
        "ConnectionAttempts=1",
        "ConnectTimeout=3",
        "ServerAliveInterval=3",
        "ServerAliveCountMax=1",
        "LogLevel=ERROR",
    ] {
        command.arg("-o").arg(option);
    }
    command
        .arg("-o")
        .arg(format!("HostKeyAlias={alias}"))
        .arg("-o")
        .arg(format!("UserKnownHostsFile={}", hosts.display()))
        .arg("-i")
        .arg(key)
        .arg("-p")
        .arg(port.to_string())
        .arg(format!("{user}@127.0.0.1"))
        .arg(r#""C:\ProgramData\M-Bedrock\VirtualClients\virtual-guest-agent.exe" --ssh-status"#);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    Some(command)
}

fn ssh_status(command: Command) -> Option<GuestStatus> {
    // Use the canonical bounded child/pipe runner instead of joining an
    // unbounded reader when an SSH descendant retains the stdout handle.
    let output = command_output_with_timeout(
        Path::new(command.get_program()),
        command.get_args(),
        MAX_EXECUTION,
    )
    .ok()?;
    if output.len() as u64 > MAX_RESPONSE_BYTES {
        return None;
    }
    let status: GuestStatus = serde_json::from_str(&output).ok()?;
    if status.schema != GUEST_STATUS_SCHEMA
        || status.protocol_version != GUEST_AGENT_PROTOCOL_VERSION
        || status
            .machine_identity
            .as_deref()
            .is_none_or(|v| v.len() != 64 || !v.bytes().all(|c| c.is_ascii_hexdigit()))
    {
        return None;
    }
    Some(status)
}

#[cfg(target_os = "windows")]
fn host_ssh_binary() -> Option<PathBuf> {
    let root = PathBuf::from(std::env::var_os("SystemRoot")?);
    let path = root.join("System32").join("OpenSSH").join("ssh.exe");
    (root.is_absolute() && plain_file(&path)).then_some(path)
}
#[cfg(not(target_os = "windows"))]
fn host_ssh_binary() -> Option<PathBuf> {
    None
}

#[cfg(target_os = "windows")]
fn guest_client_key() -> Option<PathBuf> {
    let root = PathBuf::from(std::env::var_os("ProgramData")?);
    let path = root.join("AppSandbox").join("ssh").join("id_appsandbox");
    (root.is_absolute() && plain_file(&path)).then_some(path)
}
#[cfg(not(target_os = "windows"))]
fn guest_client_key() -> Option<PathBuf> {
    None
}

pub(super) fn probe_status(name: &str, user: &str, port: u16) -> SshProof {
    let Some(hosts) = trust_file(name) else {
        return SshProof::default();
    };
    let mut proof = SshProof {
        pin_present: true,
        ..SshProof::default()
    };
    let (Some(ssh), Some(key)) = (host_ssh_binary(), guest_client_key()) else {
        return proof;
    };
    let Some(command) = fixed_ssh_command(&ssh, &hosts, &key, name, user, port) else {
        return proof;
    };
    if let Some(status) = ssh_status(command) {
        proof.host_key_verified = true;
        proof.guest_status_observed = true;
        proof.machine_identity = status.machine_identity;
    }
    proof
}

#[cfg(test)]
mod tests {
    use super::{alias, fixed_ssh_command, parse_pins, valid_user};
    use std::path::Path;
    #[test]
    fn host_key_pins_are_exact_distinct_and_ed25519_only() {
        let one = "A".repeat(68);
        let two = "B".repeat(68);
        assert_eq!(
            parse_pins(&format!(
                "mivubi-virtual-01 ssh-ed25519 {one}\nmivubi-virtual-02 ssh-ed25519 {two}"
            ))
            .unwrap()
            .len(),
            2
        );
        assert!(parse_pins(&format!(
            "mivubi-virtual-01 ssh-ed25519 {one}\nmivubi-virtual-02 ssh-ed25519 {one}"
        ))
        .is_none());
        assert!(parse_pins(&format!("* ssh-ed25519 {one}")).is_none());
        assert!(parse_pins(&format!("mivubi-virtual-01 ssh-rsa {one}")).is_none());
        assert!(parse_pins(&format!(
            "mivubi-virtual-01 ssh-ed25519 {one}\nmivubi-virtual-01 ssh-ed25519 {two}"
        ))
        .is_none());
        assert_eq!(alias("virtual-01"), None);
    }
    #[test]
    fn remote_command_is_fixed_and_accepts_no_ssh_argument_injection() {
        assert!(!valid_user("admin@example"));
        assert!(!valid_user("-oProxyCommand=evil"));
        assert!(fixed_ssh_command(
            Path::new("ssh"),
            Path::new("hosts"),
            Path::new("key"),
            "Virtual-01",
            "Admin",
            2200
        )
        .is_some());
        assert!(fixed_ssh_command(
            Path::new("ssh"),
            Path::new("hosts"),
            Path::new("key"),
            "Virtual-01",
            "bad@x",
            2200
        )
        .is_none());
        assert!(fixed_ssh_command(
            Path::new("ssh"),
            Path::new("hosts"),
            Path::new("key"),
            "virtual-01",
            "Admin",
            2200
        )
        .is_none());
        assert!(fixed_ssh_command(
            Path::new("ssh"),
            Path::new("hosts"),
            Path::new("key"),
            "Virtual-01",
            "Admin",
            0
        )
        .is_none());
    }
}
