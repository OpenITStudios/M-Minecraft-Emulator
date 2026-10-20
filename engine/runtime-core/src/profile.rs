use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs, io, path::PathBuf};
#[cfg(target_os = "macos")]
use std::{path::Path, time::Duration};
#[cfg(target_os = "windows")]
use std::{path::Path, time::Duration};

use crate::{
    client::ClientId,
    guest::{guest_agent_launch_compatible, GUEST_AGENT_PROTOCOL_VERSION, GUEST_STATUS_SCHEMA},
    paths::{
        base_profile_path_for_version, base_vmx_path_for_version, client_profile_path,
        validate_version_segment,
    },
    persistence::{read_text_recovering, write_text_transactional},
};

pub const BASE_PROFILE_SCHEMA: u32 = 3;
pub const CLIENT_PROFILE_SCHEMA: u32 = 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MinecraftInstallType {
    Desktop,
    Store,
    AppBundle,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MinecraftProfile {
    pub version: String,
    pub install_type: MinecraftInstallType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BaseState {
    Registered,
    Finalizing,
    Finalized,
}

impl BaseState {
    pub(crate) fn as_vmx_str(self) -> &'static str {
        match self {
            Self::Registered => "REGISTERED",
            Self::Finalizing => "FINALIZING",
            Self::Finalized => "FINALIZED",
        }
    }

    pub(crate) fn from_vmx_str(value: &str) -> io::Result<Self> {
        match value {
            "REGISTERED" => Ok(Self::Registered),
            "FINALIZING" => Ok(Self::Finalizing),
            "FINALIZED" => Ok(Self::Finalized),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid Base lifecycle state: {value}"),
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseProfile {
    pub schema: u32,
    pub minecraft_version: String,
    pub guest_status_schema: u32,
    #[serde(default)]
    pub guest_agent_protocol: u32,
    #[serde(default)]
    pub base_generation_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientProfile {
    pub schema: u32,
    pub base_minecraft_version: String,
    #[serde(default)]
    pub base_generation_id: String,
    #[serde(default)]
    pub verified_vm_identity: Option<String>,
    #[serde(default)]
    pub verified_windows_identity: Option<String>,
}

pub(crate) fn identity_fingerprint(value: &str) -> String {
    Sha256::digest(value.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

fn new_base_generation_id() -> io::Result<String> {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::Other, error.to_string()))?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

fn valid_provenance_id(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|character| character.is_ascii_hexdigit())
}

impl ClientProfile {
    pub(crate) fn saved_vm_identity_matches(&self, current: Option<&str>) -> bool {
        match self.verified_vm_identity.as_deref() {
            None => true,
            Some(expected) => current.is_some_and(|value| identity_fingerprint(value) == expected),
        }
    }

    pub(crate) fn identity_provenance_matches(&self, current: Option<&str>) -> bool {
        self.verified_vm_identity
            .as_deref()
            .is_some_and(|value| !value.is_empty())
            && self
                .verified_windows_identity
                .as_deref()
                .is_some_and(|value| !value.is_empty())
            && self.saved_vm_identity_matches(current)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProfileParity {
    Match,
    Mismatch,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileStatus {
    pub native: Option<MinecraftProfile>,
    pub base: Option<BaseProfile>,
    pub parity: ProfileParity,
}

pub fn current_base_vmx_path() -> io::Result<PathBuf> {
    let native = native_minecraft_profile().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Native Minecraft Education version could not be detected",
        )
    })?;
    base_vmx_path_for_version(&native.version)
}

pub fn base_profile_path() -> io::Result<PathBuf> {
    let native = native_minecraft_profile().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Native Minecraft Education version could not be detected",
        )
    })?;
    base_profile_path_for_version(&native.version)
}

pub fn load_base_profile() -> io::Result<BaseProfile> {
    let path = base_profile_path()?;
    let raw = read_text_recovering(&path)?;
    let profile: BaseProfile = serde_json::from_str(&raw).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid Base profile {}: {error}", path.display()),
        )
    })?;

    if profile.schema != BASE_PROFILE_SCHEMA {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "unsupported Base profile schema {}; expected {}",
                profile.schema, BASE_PROFILE_SCHEMA
            ),
        ));
    }
    if !valid_provenance_id(&profile.base_generation_id) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Base profile baseGenerationId is invalid",
        ));
    }

    Ok(profile)
}

pub(crate) fn require_base_launch_protocol(protocol: u32) -> io::Result<()> {
    if guest_agent_launch_compatible(protocol) {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Base Guest Agent protocol {protocol} cannot launch Minecraft Education; update the guest Agent before registering this Base"
            ),
        ))
    }
}

pub fn write_verified_base_profile(
    native: &MinecraftProfile,
    guest_agent_protocol: u32,
) -> io::Result<BaseProfile> {
    // The writer must enforce the same invariant as registration: a status
    // endpoint is insufficient to create a reusable playable Base profile.
    require_base_launch_protocol(guest_agent_protocol)?;
    let base = base_vmx_path_for_version(&native.version)?;
    if !base.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Base VM is missing: {}", base.display()),
        ));
    }

    let profile = BaseProfile {
        schema: BASE_PROFILE_SCHEMA,
        minecraft_version: native.version.clone(),
        guest_status_schema: GUEST_STATUS_SCHEMA,
        guest_agent_protocol,
        base_generation_id: new_base_generation_id()?,
    };

    let path = base_profile_path_for_version(&native.version)?;
    let json = serde_json::to_string_pretty(&profile)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    write_text_transactional(&path, &format!("{json}\n"))?;

    Ok(profile)
}

pub fn remove_client_profile(client: ClientId) -> io::Result<()> {
    if client.is_native() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Native does not have Virtual lineage provenance",
        ));
    }
    let path = client_profile_path(client.as_str())?;
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

pub fn write_client_profile(
    client: ClientId,
    base_version: &str,
    base_generation_id: &str,
) -> io::Result<ClientProfile> {
    if client.is_native() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Native does not have Virtual lineage provenance",
        ));
    }

    let profile = ClientProfile {
        schema: CLIENT_PROFILE_SCHEMA,
        base_minecraft_version: base_version.to_string(),
        base_generation_id: base_generation_id.to_string(),
        verified_vm_identity: None,
        verified_windows_identity: None,
    };
    write_client_profile_data(client, &profile)?;
    Ok(profile)
}

fn write_client_profile_data(client: ClientId, profile: &ClientProfile) -> io::Result<()> {
    validate_client_profile_identities(profile)?;
    let path = client_profile_path(client.as_str())?;
    let json = serde_json::to_string_pretty(profile)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    write_text_transactional(&path, &format!("{json}\n"))
}

pub fn write_verified_client_identities(
    client: ClientId,
    vm_identity: &str,
    windows_identity: &str,
) -> io::Result<ClientProfile> {
    let mut profile = load_client_profile(client)?;
    profile.verified_vm_identity = Some(vm_identity.to_string());
    profile.verified_windows_identity = Some(windows_identity.to_string());
    write_client_profile_data(client, &profile)?;
    Ok(profile)
}

pub fn load_client_profile(client: ClientId) -> io::Result<ClientProfile> {
    let path = client_profile_path(client.as_str())?;
    let raw = read_text_recovering(&path)?;
    let profile: ClientProfile = serde_json::from_str(&raw).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid client profile {}: {error}", path.display()),
        )
    })?;
    if profile.schema != CLIENT_PROFILE_SCHEMA {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "unsupported client profile schema {}; expected {}",
                profile.schema, CLIENT_PROFILE_SCHEMA
            ),
        ));
    }

    validate_client_profile_identities(&profile)?;
    Ok(profile)
}

fn valid_identity_fingerprint(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|character| character.is_ascii_hexdigit())
}

fn validate_client_profile_identities(profile: &ClientProfile) -> io::Result<()> {
    if !valid_provenance_id(&profile.base_generation_id) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "client profile baseGenerationId is invalid",
        ));
    }

    for (name, value) in [
        (
            "verifiedVmIdentity",
            profile.verified_vm_identity.as_deref(),
        ),
        (
            "verifiedWindowsIdentity",
            profile.verified_windows_identity.as_deref(),
        ),
    ] {
        if let Some(value) = value {
            if !valid_identity_fingerprint(value) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("client profile {name} fingerprint is invalid"),
                ));
            }
        }
    }

    Ok(())
}

pub fn client_lineage_parity(
    native: Option<&MinecraftProfile>,
    base: Option<&BaseProfile>,
    client: Option<&ClientProfile>,
    provisioned: bool,
) -> ProfileParity {
    if !provisioned {
        return ProfileParity::Unknown;
    }

    match (native, base, client) {
        (Some(native), Some(base), Some(client))
            if native.version == base.minecraft_version
                && native.version == client.base_minecraft_version
                && base.base_generation_id == client.base_generation_id =>
        {
            ProfileParity::Match
        }
        (Some(_), Some(_), Some(_)) => ProfileParity::Mismatch,
        _ => ProfileParity::Unknown,
    }
}

pub fn require_client_matches_native(client: ClientId) -> io::Result<ClientProfile> {
    let native = native_minecraft_profile().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Native Minecraft Education version could not be detected",
        )
    })?;
    let profile = load_client_profile(client).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!(
                "{} lineage cannot be proven: {error}. Reprovision this Virtual.",
                client.as_str()
            ),
        )
    })?;

    if profile.base_minecraft_version != native.version {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "{} was provisioned from Minecraft {} but Native is {}. Reprovision this Virtual.",
                client.as_str(),
                profile.base_minecraft_version,
                native.version
            ),
        ));
    }

    let base = load_base_profile().map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("registered Base lineage cannot be proven: {error}"),
        )
    })?;
    if profile.base_generation_id != base.base_generation_id {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "{} belongs to an older Base generation. Reprovision this Virtual from the current Base.",
                client.as_str()
            ),
        ));
    }

    Ok(profile)
}

pub fn native_minecraft_profile() -> Option<MinecraftProfile> {
    #[cfg(target_os = "windows")]
    {
        return windows_native_profile();
    }

    #[cfg(target_os = "macos")]
    {
        return macos_native_profile();
    }

    #[allow(unreachable_code)]
    None
}

fn base_profile_matches_native(native: &MinecraftProfile, base: &BaseProfile) -> bool {
    native.version == base.minecraft_version
        && base.guest_status_schema == GUEST_STATUS_SCHEMA
        && guest_agent_launch_compatible(base.guest_agent_protocol)
        && valid_provenance_id(&base.base_generation_id)
}

pub fn profile_status() -> ProfileStatus {
    let native = native_minecraft_profile();
    let base = load_base_profile().ok();

    let parity = match (&native, &base) {
        (Some(native), Some(base)) if base_profile_matches_native(native, base) => {
            ProfileParity::Match
        }
        (Some(_), Some(_)) => ProfileParity::Mismatch,
        _ => ProfileParity::Unknown,
    };

    ProfileStatus {
        native,
        base,
        parity,
    }
}

pub fn require_base_matches_native() -> io::Result<ProfileStatus> {
    let native = native_minecraft_profile().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Native Minecraft Education version could not be detected",
        )
    })?;
    let base = load_base_profile().map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("registered Base profile cannot be used: {error}"),
        )
    })?;

    if !base_profile_matches_native(&native, &base) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "registered Base profile does not match current Native/Guest Agent compatibility requirements (Native={}, Base={})",
                native.version, base.minecraft_version
            ),
        ));
    }

    Ok(ProfileStatus {
        native: Some(native),
        base: Some(base),
        parity: ProfileParity::Match,
    })
}

#[cfg(target_os = "windows")]
fn windows_native_profile() -> Option<MinecraftProfile> {
    // This is called during host snapshots and in Guest Agent /status.
    // Keep both probes bounded; a stuck installer/registry service must not
    // indefinitely occupy the caller or a Guest Agent worker.
    let registry = crate::provider::command_output_with_timeout(
        Path::new("reg"),
        [
            "query",
            r"HKLM\SOFTWARE\Microsoft\Microsoft Studios\Minecraft Education Edition",
            "/v",
            "Version",
            "/reg:32",
        ],
        Duration::from_secs(4),
    )
    .ok();

    if let Some(text) = registry {
        if let Some(version) = parse_registry_version(&text) {
            return Some(MinecraftProfile {
                version,
                install_type: MinecraftInstallType::Desktop,
            });
        }
    }

    let store = crate::provider::command_output_with_timeout(
        Path::new("powershell.exe"),
        [
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "(Get-AppxPackage -Name Microsoft.MinecraftEducationEdition -ErrorAction SilentlyContinue | Sort-Object Version -Descending | Select-Object -First 1 -ExpandProperty Version).ToString()",
        ],
        Duration::from_secs(8),
    ).ok();

    store
        .and_then(|value| normalized_version(&value))
        .map(|version| MinecraftProfile {
            version,
            install_type: MinecraftInstallType::Store,
        })
}

#[cfg(target_os = "windows")]
fn parse_registry_version(output: &str) -> Option<String> {
    output.lines().find_map(|line| {
        let line = line.trim();
        if !line.starts_with("Version") {
            return None;
        }
        line.split_whitespace().last().and_then(normalized_version)
    })
}

#[cfg(target_os = "macos")]
fn macos_native_profile() -> Option<MinecraftProfile> {
    let version = crate::provider::command_output_with_timeout(
        Path::new("plutil"),
        [
            "-extract",
            "CFBundleShortVersionString",
            "raw",
            "-o",
            "-",
            "/Applications/Minecraft Education.app/Contents/Info.plist",
        ],
        Duration::from_secs(10),
    )
    .ok()?;

    normalized_version(&version).map(|version| MinecraftProfile {
        version,
        install_type: MinecraftInstallType::AppBundle,
    })
}

fn normalized_version(value: &str) -> Option<String> {
    let value = value.trim();
    validate_version_segment(value).ok()?;
    Some(value.to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        base_profile_matches_native, normalized_version, BaseProfile, BaseState, ClientProfile,
        MinecraftInstallType, MinecraftProfile, BASE_PROFILE_SCHEMA, CLIENT_PROFILE_SCHEMA,
    };
    use crate::guest::{GUEST_AGENT_PROTOCOL_VERSION, GUEST_STATUS_SCHEMA};

    #[test]
    fn base_profile_rejects_status_only_guest_agents_before_registration() {
        assert!(super::require_base_launch_protocol(2).is_ok());
        for unsupported in [0, 1, 3, 99] {
            assert_eq!(
                super::require_base_launch_protocol(unsupported)
                    .unwrap_err()
                    .kind(),
                std::io::ErrorKind::InvalidData
            );
        }
    }

    #[test]
    fn version_normalization_is_strict() {
        assert_eq!(
            normalized_version("1.21.120.0\n"),
            Some("1.21.120.0".into())
        );
        assert_eq!(normalized_version(""), None);
        assert_eq!(normalized_version(".."), None);
        assert_eq!(normalized_version("1..21"), None);
        assert_eq!(normalized_version("version 1.21"), None);
    }

    #[test]
    fn base_state_vmx_contract_is_exact() {
        assert_eq!(BaseState::Registered.as_vmx_str(), "REGISTERED");
        assert_eq!(BaseState::Finalizing.as_vmx_str(), "FINALIZING");
        assert_eq!(BaseState::Finalized.as_vmx_str(), "FINALIZED");
        assert_eq!(
            BaseState::from_vmx_str("FINALIZED").unwrap(),
            BaseState::Finalized
        );
        assert_eq!(
            BaseState::from_vmx_str("READY").unwrap_err().kind(),
            std::io::ErrorKind::InvalidData
        );
    }

    #[test]
    fn base_profile_schema_round_trips() {
        let profile = BaseProfile {
            schema: BASE_PROFILE_SCHEMA,
            minecraft_version: "1.21.120.0".into(),
            guest_status_schema: GUEST_STATUS_SCHEMA,
            guest_agent_protocol: GUEST_AGENT_PROTOCOL_VERSION,
            base_generation_id: "a".repeat(64),
        };

        let json = serde_json::to_string(&profile).unwrap();
        let decoded: BaseProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, profile);
    }

    #[test]
    fn legacy_client_profile_without_base_generation_is_rejected() {
        let json = r#"{"schema":1,"baseMinecraftVersion":"1.21.120.0","createdBy":"0.1.0"}"#;
        let profile: ClientProfile = serde_json::from_str(json).unwrap();
        assert_ne!(profile.schema, CLIENT_PROFILE_SCHEMA);
        assert!(profile.base_generation_id.is_empty());
    }

    #[test]
    fn client_profile_identity_fingerprints_are_strict() {
        let mut profile = ClientProfile {
            schema: CLIENT_PROFILE_SCHEMA,
            base_minecraft_version: "1.21.120.0".into(),
            base_generation_id: "c".repeat(64),
            verified_vm_identity: Some("a".repeat(64)),
            verified_windows_identity: Some("b".repeat(64)),
        };
        super::validate_client_profile_identities(&profile).unwrap();

        profile.verified_windows_identity = Some("short".into());
        assert_eq!(
            super::validate_client_profile_identities(&profile)
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::InvalidData
        );
    }

    #[test]
    fn base_profile_parity_requires_current_protocol_schema_and_minecraft_version() {
        let native = MinecraftProfile {
            version: "1.21.120.0".into(),
            install_type: MinecraftInstallType::Desktop,
        };
        let current = BaseProfile {
            schema: BASE_PROFILE_SCHEMA,
            minecraft_version: native.version.clone(),
            guest_status_schema: GUEST_STATUS_SCHEMA,
            guest_agent_protocol: GUEST_AGENT_PROTOCOL_VERSION,
            base_generation_id: "a".repeat(64),
        };

        assert!(base_profile_matches_native(&native, &current));

        let stale_protocol = BaseProfile {
            guest_agent_protocol: GUEST_AGENT_PROTOCOL_VERSION + 1,
            ..current.clone()
        };
        assert!(!base_profile_matches_native(&native, &stale_protocol));

        let stale_schema = BaseProfile {
            guest_status_schema: GUEST_STATUS_SCHEMA + 1,
            ..current.clone()
        };
        assert!(!base_profile_matches_native(&native, &stale_schema));

        let wrong_version = BaseProfile {
            minecraft_version: "9.9.9".into(),
            ..current
        };
        assert!(!base_profile_matches_native(&native, &wrong_version));
    }

    #[test]
    fn native_profile_carries_install_type() {
        let profile = MinecraftProfile {
            version: "1.21.120.0".into(),
            install_type: MinecraftInstallType::Desktop,
        };
        assert_eq!(profile.version, "1.21.120.0");
    }
    #[test]
    fn stored_identity_provenance_requires_the_current_vm() {
        let mut profile = super::ClientProfile {
            schema: super::CLIENT_PROFILE_SCHEMA,
            base_minecraft_version: "1.0.0".into(),
            base_generation_id: "c".repeat(64),
            verified_vm_identity: Some(super::identity_fingerprint("uuid-a|mac-a")),
            verified_windows_identity: Some("windows-proof".into()),
        };
        assert!(profile.identity_provenance_matches(Some("uuid-a|mac-a")));
        assert!(!profile.identity_provenance_matches(Some("uuid-b|mac-a")));
        assert!(!profile.identity_provenance_matches(None));
        profile.verified_windows_identity = None;
        assert!(!profile.identity_provenance_matches(Some("uuid-a|mac-a")));
        profile.verified_windows_identity = Some(String::new());
        assert!(!profile.identity_provenance_matches(Some("uuid-a|mac-a")));
    }

    #[test]
    fn a_fresh_profile_is_not_identity_proof_but_has_no_saved_vm_mismatch() {
        let profile = super::ClientProfile {
            schema: super::CLIENT_PROFILE_SCHEMA,
            base_minecraft_version: "1.0.0".into(),
            base_generation_id: "c".repeat(64),
            verified_vm_identity: None,
            verified_windows_identity: None,
        };
        assert!(profile.saved_vm_identity_matches(None));
        assert!(!profile.identity_provenance_matches(Some("uuid-a|mac-a")));
        assert_eq!(
            super::identity_fingerprint("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
