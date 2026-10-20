use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    time::Duration,
};

#[cfg(target_os = "windows")]
use std::ffi::OsStr;

use url::Url;

use crate::{
    client::{ClientId, ClientState},
    paths::{runtime_root, update_staging_root},
    persistence::{read_text_recovering, write_text_transactional},
    provider::current_platform_provider,
    schema::{inspect_runtime_schema, SchemaState},
};

const RELEASE_CHANNEL_JSON: &str = include_str!("../../distribution/release-channel.json");
const MAX_MANIFEST_BYTES: u64 = 128 * 1024;
const MAX_INSTALLER_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReleaseChannel {
    schema: u32,
    channel: String,
    repository: String,
    release_tag_prefix: String,
    platform: String,
    manifest_asset: String,
    manifest_endpoint: String,
    check_mode: String,
    apply_gate: String,
    self_update_runtime_enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct UpdateManifest {
    version: String,
    platforms: BTreeMap<String, PlatformManifest>,
}

#[derive(Debug, Clone, Deserialize)]
struct PlatformManifest {
    #[serde(rename = "authenticodeThumbprint")]
    authenticode_thumbprint: String,
    url: String,
    sha256: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UpdateState {
    UpToDate,
    UpdateAvailable,
    UpdateStaged,
    Unavailable,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheck {
    pub state: UpdateState,
    pub current_version: &'static str,
    pub latest_version: Option<String>,
    pub can_apply_now: bool,
    pub self_update_enabled: bool,
    pub staged_path: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StagedUpdate {
    pub version: String,
    pub platform: String,
    pub installer_path: String,
    pub sha256: String,
    pub authenticode_thumbprint: String,
}

pub fn check_update() -> io::Result<UpdateCheck> {
    let policy = release_channel()?;
    validate_policy(&policy)?;

    if policy.platform != current_platform() {
        return Ok(UpdateCheck {
            state: UpdateState::Unavailable,
            current_version: env!("CARGO_PKG_VERSION"),
            latest_version: None,
            can_apply_now: false,
            self_update_enabled: false,
            staged_path: None,
            reason: Some(format!(
                "release channel {} does not publish for {}",
                policy.platform,
                current_platform()
            )),
        });
    }

    let manifest = match fetch_manifest(&policy) {
        Ok(manifest) => manifest,
        Err(error) => {
            let (staged_path, reason) = match staged_update() {
                Ok(staged) => (
                    staged
                        .filter(|item| Path::new(&item.installer_path).is_file())
                        .map(|item| item.installer_path),
                    error.to_string(),
                ),
                Err(staged_error) => (
                    None,
                    format!("{error}; staged update metadata is unreadable: {staged_error}"),
                ),
            };
            return Ok(UpdateCheck {
                state: UpdateState::Unavailable,
                current_version: env!("CARGO_PKG_VERSION"),
                latest_version: None,
                can_apply_now: false,
                self_update_enabled: policy.self_update_runtime_enabled,
                staged_path,
                reason: Some(reason),
            });
        }
    };

    let latest = parse_version(&manifest.version)?;
    let current = parse_version(env!("CARGO_PKG_VERSION"))?;
    let staged = staged_update()?;
    let platform = manifest.platforms.get(current_platform()).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "update manifest has no package for this platform",
        )
    })?;
    validate_platform_manifest(&policy, &manifest.version, platform)?;
    let staged_matches = staged.as_ref().is_some_and(|item| {
        staged_update_matches(
            item,
            &manifest.version,
            current_platform(),
            platform,
            Path::new(&item.installer_path).is_file(),
        )
    });

    let state = if latest <= current {
        UpdateState::UpToDate
    } else if staged_matches {
        UpdateState::UpdateStaged
    } else {
        UpdateState::UpdateAvailable
    };

    let readiness = update_apply_readiness();

    let (can_apply_now, reason) = apply_permission(policy.self_update_runtime_enabled, readiness);

    Ok(UpdateCheck {
        state,
        current_version: env!("CARGO_PKG_VERSION"),
        latest_version: Some(manifest.version),
        can_apply_now,
        self_update_enabled: policy.self_update_runtime_enabled,
        staged_path: staged
            .filter(|_| staged_matches)
            .map(|item| item.installer_path),
        reason,
    })
}

// Staged is a concrete downloaded artifact matching the current manifest,
// not merely a leftover metadata record. This does not enable self-application.
fn staged_update_matches(
    staged: &StagedUpdate,
    version: &str,
    platform: &str,
    manifest: &PlatformManifest,
    installer_exists: bool,
) -> bool {
    let path_is_contained = staged_installer_path_is_contained(staged).unwrap_or(false);
    installer_exists
        && path_is_contained
        && staged.version == version
        && staged.platform == platform
        && staged.sha256.eq_ignore_ascii_case(&manifest.sha256)
        && staged
            .authenticode_thumbprint
            .eq_ignore_ascii_case(&manifest.authenticode_thumbprint)
}

fn staged_installer_path_is_contained(staged: &StagedUpdate) -> io::Result<bool> {
    let expected_root = update_staging_root()?.join(&staged.version);
    let installer = Path::new(&staged.installer_path);
    if !installer.is_absolute() {
        return Ok(false);
    }
    let Some(parent) = installer.parent() else {
        return Ok(false);
    };
    Ok(parent == expected_root)
}

pub fn stage_update() -> io::Result<StagedUpdate> {
    let policy = release_channel()?;
    validate_policy(&policy)?;

    if policy.platform != current_platform() {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("no update package is published for {}", current_platform()),
        ));
    }

    let manifest = fetch_manifest(&policy)?;
    let latest = parse_version(&manifest.version)?;
    let current = parse_version(env!("CARGO_PKG_VERSION"))?;
    if latest <= current {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Virtual Clients is already up to date",
        ));
    }

    let platform = manifest.platforms.get(current_platform()).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("manifest has no {} package", current_platform()),
        )
    })?;
    validate_platform_manifest(&policy, &manifest.version, platform)?;

    let version_root = update_staging_root()?.join(&manifest.version);
    fs::create_dir_all(&version_root)?;
    let installer_name = platform
        .url
        .rsplit('/')
        .next()
        .filter(|name| !name.is_empty() && !name.contains(['/', '\\']))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "installer URL is invalid"))?;
    let final_path = version_root.join(installer_name);
    let temporary = final_path.with_extension("download");

    let publisher_thumbprint = publisher_certificate_thumbprint()?;
    if !publisher_thumbprint.eq_ignore_ascii_case(&platform.authenticode_thumbprint) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "update manifest signer does not match the embedded publisher certificate",
        ));
    }

    if temporary.exists() {
        fs::remove_file(&temporary)?;
    }
    download_to(&platform.url, &temporary, MAX_INSTALLER_BYTES)?;
    let actual = sha256_file(&temporary)?;
    if !actual.eq_ignore_ascii_case(&platform.sha256) {
        let _ = fs::remove_file(&temporary);
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "update installer SHA-256 mismatch: expected {}, got {}",
                platform.sha256, actual
            ),
        ));
    }

    let actual_thumbprint = authenticode_thumbprint(&temporary)?;
    if !actual_thumbprint.eq_ignore_ascii_case(publisher_thumbprint) {
        let _ = fs::remove_file(&temporary);
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "update installer Authenticode signer does not match the embedded publisher certificate",
        ));
    }

    if final_path.exists() {
        fs::remove_file(&final_path)?;
    }
    fs::rename(&temporary, &final_path)?;

    let staged = StagedUpdate {
        version: manifest.version,
        platform: current_platform().to_string(),
        installer_path: final_path.display().to_string(),
        sha256: actual,
        authenticode_thumbprint: actual_thumbprint,
    };
    write_staged_update(&staged)?;
    let _ = cleanup_obsolete_update_versions(&version_root);
    Ok(staged)
}

fn cleanup_obsolete_update_versions(keep: &Path) -> io::Result<()> {
    let root = update_staging_root()?;
    if !root.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() && path != keep {
            let _ = fs::remove_dir_all(path);
        }
    }
    Ok(())
}

fn fetch_manifest(policy: &ReleaseChannel) -> io::Result<UpdateManifest> {
    let bytes = fetch_bytes(&policy.manifest_endpoint, MAX_MANIFEST_BYTES)?;
    serde_json::from_slice(&bytes).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid update manifest: {error}"),
        )
    })
}

fn update_agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(12)))
        .tls_config(
            ureq::tls::TlsConfig::builder()
                .provider(ureq::tls::TlsProvider::NativeTls)
                .root_certs(ureq::tls::RootCerts::PlatformVerifier)
                .build(),
        )
        .build()
        .into()
}

fn fetch_bytes(url: &str, max_bytes: u64) -> io::Result<Vec<u8>> {
    if !url.starts_with("https://") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "update URL must use HTTPS",
        ));
    }

    let agent = update_agent();
    let response = agent.get(url).call().map_err(|error| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("update download failed: {error}"),
        )
    })?;

    let mut bytes = Vec::new();
    response
        .into_body()
        .into_reader()
        .take(max_bytes + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max_bytes {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "update payload exceeded the allowed size",
        ));
    }
    Ok(bytes)
}

fn download_to(url: &str, path: &Path, max_bytes: u64) -> io::Result<()> {
    let bytes = fetch_bytes(url, max_bytes)?;
    fs::write(path, bytes)
}

fn publisher_certificate_thumbprint() -> io::Result<&'static str> {
    let value = option_env!("VIRTUAL_CLIENTS_PUBLISHER_CERT_THUMBPRINT").unwrap_or("");
    if value.len() != 40 || !value.chars().all(|character| character.is_ascii_hexdigit()) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "trusted publisher certificate is not configured in this build",
        ));
    }
    Ok(value)
}

#[cfg(target_os = "windows")]
fn authenticode_thumbprint(path: &Path) -> io::Result<String> {
    let script = r#"$s=Get-AuthenticodeSignature -LiteralPath $args[0]; if ($s.Status -ne 'Valid' -or $null -eq $s.SignerCertificate) { [Console]::Error.Write([string]$s.Status); exit 41 }; [Console]::Out.Write($s.SignerCertificate.Thumbprint)"#;
    let thumbprint = crate::provider::command_output_with_timeout(
        Path::new("powershell.exe"),
        [
            OsStr::new("-NoLogo"),
            OsStr::new("-NoProfile"),
            OsStr::new("-NonInteractive"),
            OsStr::new("-Command"),
            OsStr::new(script),
            path.as_os_str(),
        ],
        Duration::from_secs(15),
    )
    .map_err(|error| {
        if error.kind() == io::ErrorKind::TimedOut {
            error
        } else {
            io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("update installer Authenticode verification failed: {error}"),
            )
        }
    })?;
    if thumbprint.len() != 40
        || !thumbprint
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "update installer signer thumbprint is invalid",
        ));
    }
    Ok(thumbprint)
}

#[cfg(not(target_os = "windows"))]
fn authenticode_thumbprint(_path: &Path) -> io::Result<String> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Authenticode verification is available only on Windows",
    ))
}

fn sha256_file(path: &Path) -> io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>())
}

fn staged_metadata_path() -> io::Result<PathBuf> {
    Ok(update_staging_root()?.join("staged-update.json"))
}

fn staged_update() -> io::Result<Option<StagedUpdate>> {
    let path = staged_metadata_path()?;
    if !path.is_file() {
        return Ok(None);
    }
    let raw = read_text_recovering(&path)?;
    let staged = serde_json::from_str(&raw)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    Ok(Some(staged))
}

fn write_staged_update(staged: &StagedUpdate) -> io::Result<()> {
    let path = staged_metadata_path()?;
    let json = serde_json::to_string_pretty(staged)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    write_text_transactional(&path, &format!("{json}\n"))
}

fn validate_platform_manifest(
    policy: &ReleaseChannel,
    version: &str,
    platform: &PlatformManifest,
) -> io::Result<()> {
    parse_version(version)?;

    let parsed = Url::parse(&platform.url)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "update package URL is invalid"))?;
    let (owner, repository) = policy.repository.split_once('/').ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "release-channel repository identity is invalid",
        )
    })?;
    let expected_tag = format!("{}{}", policy.release_tag_prefix, version);
    let expected_prefix = format!("/{owner}/{repository}/releases/download/{expected_tag}/");
    let asset = parsed
        .path()
        .strip_prefix(&expected_prefix)
        .filter(|asset| {
            !asset.is_empty()
                && !asset.contains('/')
                && !asset.contains('\\')
                && !asset.contains('%')
        })
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "update package URL does not match the configured release channel",
            )
        })?;

    let expected_asset = format!("M-Bedrock-Virtual-Clients-{version}-windows-x86_64.exe");

    if parsed.scheme() != "https"
        || parsed.host_str() != Some("github.com")
        || parsed.port().is_some()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
        || asset != expected_asset
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "update package URL does not match the configured release channel",
        ));
    }

    if platform.authenticode_thumbprint.len() != 40
        || !platform
            .authenticode_thumbprint
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "update Authenticode thumbprint metadata is invalid",
        ));
    }

    if platform.sha256.len() != 64
        || !platform
            .sha256
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "update SHA-256 metadata is invalid",
        ));
    }

    Ok(())
}

#[derive(Debug)]
struct ApplyReadiness {
    ready: bool,
    reason: Option<String>,
}

fn apply_permission(
    self_update_enabled: bool,
    readiness: ApplyReadiness,
) -> (bool, Option<String>) {
    if !self_update_enabled {
        return (
            false,
            Some(
                "runtime self-update is disabled until trusted signature verification is accepted"
                    .into(),
            ),
        );
    }

    (readiness.ready, readiness.reason)
}

fn update_apply_readiness() -> ApplyReadiness {
    let Ok(root) = runtime_root() else {
        return ApplyReadiness {
            ready: false,
            reason: Some("runtime root is unavailable".into()),
        };
    };

    let schema = inspect_runtime_schema(&root);
    if schema.state != SchemaState::Ready && schema.state != SchemaState::Missing {
        return ApplyReadiness {
            ready: false,
            reason: Some(format!("runtime schema is {:?}", schema.state)),
        };
    }

    let Some(provider) = current_platform_provider() else {
        return ApplyReadiness {
            ready: false,
            reason: Some("virtualization provider state is unavailable".into()),
        };
    };

    for client in ClientId::VIRTUAL {
        match provider.status(client) {
            Ok(ClientState::Stopped) | Ok(ClientState::NotProvisioned) => {}
            Ok(state) => {
                return ApplyReadiness {
                    ready: false,
                    reason: Some(format!("{} is {:?}", client.as_str(), state)),
                }
            }
            Err(error) => {
                return ApplyReadiness {
                    ready: false,
                    reason: Some(format!(
                        "{} state could not be read: {error}",
                        client.as_str()
                    )),
                }
            }
        }
    }

    ApplyReadiness {
        ready: true,
        reason: None,
    }
}

fn release_channel() -> io::Result<ReleaseChannel> {
    serde_json::from_str(RELEASE_CHANNEL_JSON)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn validate_policy(policy: &ReleaseChannel) -> io::Result<()> {
    let expected_manifest_endpoint = format!(
        "https://github.com/{}/releases/latest/download/{}",
        policy.repository, policy.manifest_asset
    );

    if policy.schema != 1
        || policy.channel != "stable"
        || policy.repository != "MIVUBI-STD/M-Minecraft-Emulator"
        || policy.release_tag_prefix != "virtual-clients-v"
        || policy.platform != "windows-x86_64"
        || policy.manifest_asset != "latest.json"
        || policy.manifest_endpoint != expected_manifest_endpoint
        || policy.check_mode != "startup-once"
        || policy.apply_gate != "all-virtuals-stopped"
        || policy.self_update_runtime_enabled
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "release-channel policy is invalid",
        ));
    }

    Ok(())
}

fn current_platform() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        return "windows-x86_64";
    }

    #[cfg(target_os = "macos")]
    {
        return "macos";
    }

    #[allow(unreachable_code)]
    "unsupported"
}

fn parse_version(value: &str) -> io::Result<(u64, u64, u64)> {
    let parts: Vec<&str> = value.split('.').collect();
    if parts.len() != 3 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid semantic version: {value}"),
        ));
    }

    let major = parts[0]
        .parse()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid major version"))?;
    let minor = parts[1]
        .parse()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid minor version"))?;
    let patch = parts[2]
        .parse()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid patch version"))?;
    Ok((major, minor, patch))
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::{
        apply_permission, parse_version, release_channel, validate_platform_manifest,
        validate_policy, ApplyReadiness, PlatformManifest,
    };

    #[test]
    fn release_channel_is_valid() {
        let policy = release_channel().unwrap();
        validate_policy(&policy).unwrap();
        assert!(!policy.self_update_runtime_enabled);
    }

    #[test]
    fn policy_cannot_enable_runtime_self_update_by_configuration_only() {
        let mut policy = release_channel().unwrap();
        policy.self_update_runtime_enabled = true;
        assert_eq!(
            validate_policy(&policy).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }

    #[test]
    fn semantic_version_order_is_numeric() {
        assert!(parse_version("0.10.0").unwrap() > parse_version("0.9.9").unwrap());
        assert!(parse_version("1.0.0").unwrap() > parse_version("0.99.99").unwrap());
    }

    #[test]
    fn disabled_self_update_never_reports_apply_ready() {
        let (ready, reason) = apply_permission(
            false,
            ApplyReadiness {
                ready: true,
                reason: None,
            },
        );
        assert!(!ready);
        assert!(reason.unwrap().contains("disabled"));
    }

    #[test]
    fn release_url_requires_exact_version_tag_boundary() {
        let policy = release_channel().unwrap();
        let valid = PlatformManifest {
            authenticode_thumbprint: "a".repeat(40),
            url: "https://github.com/MIVUBI-STD/M-Minecraft-Emulator/releases/download/virtual-clients-v1.2.3/M-Bedrock-Virtual-Clients-1.2.3-windows-x86_64.exe".into(),
            sha256: "a".repeat(64),
        };
        validate_platform_manifest(&policy, "1.2.3", &valid).unwrap();
        let foreign_repository = PlatformManifest {
            url: "https://github.com/MIVUBI-STD/M-Bedrock-Dev/releases/download/virtual-clients-v1.2.3/M-Bedrock-Virtual-Clients-1.2.3-windows-x86_64.exe".into(),
            ..valid.clone()
        };
        assert!(validate_platform_manifest(&policy, "1.2.3", &foreign_repository).is_err());

        let prefix_confusion = PlatformManifest {
            url: "https://github.com/MIVUBI-STD/M-Minecraft-Emulator/releases/download/virtual-clients-v1.2.30/M-Bedrock-Virtual-Clients-1.2.3-windows-x86_64.exe".into(),
            ..valid.clone()
        };
        assert!(validate_platform_manifest(&policy, "1.2.3", &prefix_confusion).is_err());

        let wrong_asset = PlatformManifest {
            url: "https://github.com/MIVUBI-STD/M-Minecraft-Emulator/releases/download/virtual-clients-v1.2.3/Other-Installer.exe".into(),
            ..valid
        };
        assert!(validate_platform_manifest(&policy, "1.2.3", &wrong_asset).is_err());
    }
    #[test]
    fn staging_requires_an_existing_matching_installer() {
        let manifest = PlatformManifest {
            url: "https://github.com/example/installer.exe".into(),
            sha256: "a".repeat(64),
            authenticode_thumbprint: "b".repeat(40),
        };
        let staged = super::StagedUpdate {
            version: "1.2.3".into(),
            platform: "windows-x86_64".into(),
            installer_path: super::update_staging_root()
                .unwrap()
                .join("1.2.3")
                .join("installer.exe")
                .to_string_lossy()
                .into_owned(),
            sha256: "A".repeat(64),
            authenticode_thumbprint: "B".repeat(40),
        };
        assert!(super::staged_update_matches(
            &staged,
            "1.2.3",
            "windows-x86_64",
            &manifest,
            true
        ));
        assert!(!super::staged_update_matches(
            &staged,
            "1.2.3",
            "windows-x86_64",
            &manifest,
            false
        ));
        assert!(!super::staged_update_matches(
            &staged,
            "1.2.4",
            "windows-x86_64",
            &manifest,
            true
        ));
        assert!(!super::staged_update_matches(
            &staged, "1.2.3", "macos", &manifest, true
        ));
        let outside_staging = super::StagedUpdate {
            installer_path: "installer.exe".into(),
            ..staged.clone()
        };
        assert!(!super::staged_update_matches(
            &outside_staging,
            "1.2.3",
            "windows-x86_64",
            &manifest,
            true
        ));
        let changed_hash = PlatformManifest {
            sha256: "c".repeat(64),
            ..manifest.clone()
        };
        assert!(!super::staged_update_matches(
            &staged,
            "1.2.3",
            "windows-x86_64",
            &changed_hash,
            true
        ));
        let changed_signer = PlatformManifest {
            authenticode_thumbprint: "d".repeat(40),
            ..manifest
        };
        assert!(!super::staged_update_matches(
            &staged,
            "1.2.3",
            "windows-x86_64",
            &changed_signer,
            true
        ));
    }

    #[test]
    fn update_transport_keeps_platform_trust_and_https_requirement() {
        let agent = super::update_agent();
        assert_eq!(
            agent.config().tls_config().provider(),
            ureq::tls::TlsProvider::NativeTls
        );
        assert!(matches!(
            agent.config().tls_config().root_certs(),
            ureq::tls::RootCerts::PlatformVerifier
        ));
        assert_eq!(
            agent.config().timeouts().global,
            Some(std::time::Duration::from_secs(12))
        );
        assert_eq!(
            super::fetch_bytes("http://127.0.0.1/never-requested", 8)
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::InvalidInput
        );
    }
}
