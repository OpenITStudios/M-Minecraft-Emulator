//! Read-only candidate for an app-owned Sandboxie-derived engine.
//! This module NEVER installs/loads a driver, chooses a runtime backend,
//! migrates an occupied box, or treats checksums as Authenticode trust.
//! The officially installed Sandboxie provider remains the only active route.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fs::{self, File},
    io::{self, Read},
    path::{Component, Path, PathBuf},
};

const PINNED_UPSTREAM: &str = "abd4c240031cfb46666daa74ca47ed4e48b561e4";
const EXPECTED_STATE: &str = "ARTIFACTS_STAGED_NOT_INSTALLABLE";
const PRIVATE_STATE: &str = "PRIVATE_CORE_STAGED_NOT_INSTALLABLE";
const OWNED_V1186_STATE: &str = "MIVUBI_V1186_CORE_STAGED_NOT_INSTALLABLE";
const OWNED_V1186_UPSTREAM: &str = "cdba7096240b93d79c5373aa81a75a1c5c71b883";
const OWNED_V1186_TREE: &str = "f690477c07a40810f479f53973334ceae7e0ab13";
const OWNED_V1186_MARKER: &str = "MIVUBI-owned v1.18.6 Sandboxie-derived core. NOT installable or co-installable with official Sandboxie. Do not install SbieDrv.sys or replace MinecraftClient02. See LICENSES and engine-provenance.json.\n";
const PRIVATE_SOURCE_FILES: &[&str] = &[
    "Sandboxie/common/my_version.h",
    "Sandboxie/core/drv/api.c",
    "Sandboxie/core/drv/box.c",
    "Sandboxie/core/drv/process_api.c",
    "Sandboxie/core/dll/sbieapi.c",
    "Sandboxie/core/svc/DriverAssist.cpp",
    "Sandboxie/core/svc/DriverAssistStart.cpp",
    "Sandboxie/apps/start/start.cpp",
    "Sandboxie/apps/ini/main.c",
    "Sandboxie/apps/ini/update.c",
    "Sandboxie/install/kmdutil/kmdutil.c",
];
const REQUIRED_FILES: &[&str] = &[
    "SbieDrv.sys",
    "SbieSvc.exe",
    "SbieDll.dll",
    "Start.exe",
    "SbieIni.exe",
    "kmdutil.exe",
    "SbieMsg.dll",
    "SandboxieRpcSs.exe",
    "SandboxieDcomLaunch.exe",
    "SandboxieCrypto.exe",
    "SandboxieBITS.exe",
    "SandboxieWUAU.exe",
    "SbieCtrl.exe",
    "SboxHostDll.dll",
    "32/SbieSvc.exe",
    "32/SbieDll.dll",
    "Templates.ini",
    "SbieSettings.ini",
    "Manifest0.txt",
    "Manifest1.txt",
    "Manifest2.txt",
    "LICENSES/LICENSE.Plus",
    "LICENSES/Sandboxie/COPYING",
    "LICENSES/SandboxiePlus/QSbieAPI/LICENSE",
];

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StagedArtifact {
    path: String,
    source: String,
    size_bytes: u64,
    sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct BuildStage {
    id: String,
    platform: String,
    state: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StagedManifest {
    schema: u32,
    state: String,
    upstream_repository: String,
    upstream_tag: String,
    upstream_commit: String,
    build_stages: Vec<BuildStage>,
    artifacts: Vec<StagedArtifact>,
    windows_kernel_driver_loaded: bool,
    minecraft_gameplay_verified: bool,
    sandboxie_external_dependency_removed: bool,
    public_distribution_approved: bool,
    // Optional ONLY on the private exclusive source build. The original
    // upstream payload must not claim the new fork identity.
    private_core_build: Option<bool>,
    private_fork_head: Option<String>,
    private_fork_tree: Option<String>,
    private_patch_sha256: Option<String>,
    source_build_marker: Option<String>,
    source_files_modified: Option<Vec<String>>,
    can_coexist_with_official_sandboxie: Option<bool>,
    // v1.18.6 owned payload only. Legacy v1.18.5 manifests must omit these.
    source_version: Option<String>,
    upstream_tree: Option<String>,
    owned_repo_head: Option<String>,
    owned_tree: Option<String>,
    native_modified_files: Option<u32>,
    build_report_sha256: Option<String>,
    compiler_sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnedCandidateInspection {
    pub schema: u32,
    pub state: String,
    pub upstream_commit: String,
    pub verified_artifact_count: u32,
    pub required_artifact_count: u32,
    pub provider_selected: bool,
    pub driver_load_verified: bool,
    pub gameplay_verified: bool,
    pub install_permitted: bool,
}

/// Staged payload entries must come from the exact source layout owned by
/// assemble-engine.mjs, never an arbitrary manifest-supplied path. This binds
/// names to source attribution but does NOT authenticate the build report.
fn expected_source_for_payload(path: &str) -> Option<String> {
    match path {
        "32/SbieSvc.exe" => Some("Sandboxie/Bin/Win32/SbieRelease/SbieSvc.exe".into()),
        "32/SbieDll.dll" => Some("Sandboxie/Bin/Win32/SbieRelease/SbieDll.dll".into()),
        "Templates.ini" | "SbieSettings.ini" | "Manifest0.txt" | "Manifest1.txt"
        | "Manifest2.txt" => Some(format!("Sandboxie/install/{path}")),
        "LICENSES/LICENSE.Plus" => Some("LICENSE.Plus".into()),
        "LICENSES/LICENSE.Classic" => Some("LICENSE.Classic".into()),
        "LICENSES/Sandboxie/COPYING" => Some("Sandboxie/COPYING".into()),
        "LICENSES/SandboxiePlus/QSbieAPI/LICENSE" => Some("SandboxiePlus/QSbieAPI/LICENSE".into()),
        "_debug/Win32/SbieSvc.pdb" | "_debug/Win32/SbieDll.pdb" => Some(format!(
            "Sandboxie/Bin/Win32/SbieRelease/{}",
            path.rsplit('/').next()?
        )),
        "_debug/x64/SbieDrv.pdb"
        | "_debug/x64/SbieSvc.pdb"
        | "_debug/x64/SbieDll.pdb"
        | "_debug/x64/Start.pdb"
        | "_debug/x64/SbieIni.pdb"
        | "_debug/x64/kmdutil.pdb"
        | "_debug/x64/SboxHostDll.pdb" => Some(format!(
            "Sandboxie/Bin/x64/SbieRelease/{}",
            path.rsplit('/').next()?
        )),
        _ if REQUIRED_FILES.contains(&path) => {
            Some(format!("Sandboxie/Bin/x64/SbieRelease/{path}"))
        }
        _ => None,
    }
}

fn expected_not_installable_marker(private: bool) -> &'static str {
    if private {
        "Private exclusive fork candidate, NOT installable or co-installable with official Sandboxie. Do not install SbieDrv.sys or replace Client 02. See LICENSES and engine-provenance.json.\n"
    } else {
        "Unmodified upstream candidate, not a signed or tested engine. Do not install SbieDrv.sys or replace Client 02. See LICENSES and engine-provenance.json.\n"
    }
}

fn rejected(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

// Windows has reparse-point types besides ordinary symbolic links (including
// junctions). Never treat those filesystem redirections as candidate files.
fn is_redirected_entry(meta: &fs::Metadata) -> bool {
    if meta.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
        if meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return true;
        }
    }
    false
}

// Manifest/marker paths are user-selected and untrusted. Bound allocation
// BEFORE parsing/inspection; checking fs::read().len() after allocation is late.
fn read_bounded_file(path: &Path, limit: usize) -> io::Result<Vec<u8>> {
    // Check the opened handle, rather than trusting only pre-open path metadata.
    // The path itself is prechecked for symbolic links by safe_relative_file.
    let mut file = File::open(path)?;
    let before = file.metadata()?;
    if !before.is_file() || before.len() > limit as u64 {
        return Err(rejected(
            "owned-engine metadata is not a bounded regular file",
        ));
    }
    let mut bytes = Vec::with_capacity(before.len() as usize);
    (&mut file).take(limit as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > limit {
        return Err(rejected("owned-engine metadata exceeds bounded size"));
    }
    let after = file.metadata()?;
    if after.len() != before.len() || after.modified()? != before.modified()? {
        return Err(rejected("owned-engine metadata changed while it was read"));
    }
    Ok(bytes)
}

fn safe_relative_file(root: &Path, relative: &str) -> io::Result<PathBuf> {
    if relative.is_empty() || relative.len() > 240 {
        return Err(rejected("empty or oversized owned-engine artifact path"));
    }
    let relative_path = Path::new(relative);
    if relative_path.is_absolute() || relative.contains('\\') {
        return Err(rejected("owned-engine artifact path is not portable"));
    }
    let mut joined = root.to_path_buf();
    let mut depth = 0;
    for component in relative_path.components() {
        let Component::Normal(part) = component else {
            return Err(rejected("owned-engine artifact path traversal refused"));
        };
        depth += 1;
        if depth > 6 {
            return Err(rejected("owned-engine artifact path depth exceeded"));
        }
        let part = part.to_string_lossy();
        if part.is_empty()
            || !part
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
        {
            return Err(rejected("owned-engine artifact path has unsafe characters"));
        }
        joined.push(part.as_ref());
        // Include broken symlinks: metadata() follows them and can lie by returning ENOENT.
        let metadata = fs::symlink_metadata(&joined)
            .map_err(|_| rejected("owned-engine artifact entry missing or inaccessible"))?;
        if is_redirected_entry(&metadata) {
            return Err(rejected("owned-engine artifact redirected path refused"));
        }
    }
    if depth == 0 {
        return Err(rejected("empty owned-engine artifact path"));
    }
    Ok(joined)
}

fn sha256_file(path: &Path, expected_size: u64) -> io::Result<String> {
    let mut file = File::open(path)?;
    let original = file.metadata()?;
    if !original.is_file() || original.len() != expected_size || expected_size == 0 {
        return Err(rejected("owned-engine artifact type or size changed"));
    }
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    let current = file.metadata()?;
    if current.len() != original.len() || current.modified()? != original.modified()? {
        return Err(rejected(
            "owned-engine artifact changed during hash verification",
        ));
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>())
}

// A read-only candidate must match the assembler's closed payload inventory.
// Manifest hashes alone are insufficient: an unlisted executable or redirecting
// directory could otherwise exist beside an apparently valid staged payload.
// This is a bounded snapshot, not an immutable filesystem or install permit.
fn verify_payload_inventory(root: &Path, declared: &HashSet<String>) -> io::Result<()> {
    let mut observed = HashSet::new();
    let mut directories = vec![(root.to_path_buf(), String::new(), 0_usize)];
    let mut entries_seen = 0_usize;
    while let Some((directory, prefix, depth)) = directories.pop() {
        if depth > 6 {
            return Err(rejected("owned-engine payload directory depth exceeded"));
        }
        for entry in fs::read_dir(directory)? {
            // A failed directory enumeration is not an absent entry: fail closed.
            let entry = entry?;
            entries_seen += 1;
            if entries_seen > 96 {
                return Err(rejected("owned-engine payload entry cap exceeded"));
            }
            let name_os = entry.file_name();
            let name = name_os
                .to_str()
                .ok_or_else(|| rejected("owned-engine payload contains non-Unicode name"))?;
            if name.is_empty()
                || !name
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
            {
                return Err(rejected("owned-engine payload name is not canonical"));
            }
            let relative = if prefix.is_empty() {
                name.to_string()
            } else {
                format!("{prefix}/{name}")
            };
            let meta = fs::symlink_metadata(entry.path())?;
            if is_redirected_entry(&meta) {
                return Err(rejected("owned-engine payload redirected entry refused"));
            }
            if meta.is_dir() {
                let directory_prefix = format!("{relative}/");
                if !declared
                    .iter()
                    .any(|path| path.starts_with(&directory_prefix))
                {
                    return Err(rejected("undeclared owned-engine directory"));
                }
                directories.push((entry.path(), relative, depth + 1));
            } else if meta.is_file() {
                if !declared.contains(&relative) || !observed.insert(relative) {
                    return Err(rejected("undeclared or duplicate owned-engine file"));
                }
            } else {
                return Err(rejected("owned-engine payload special file refused"));
            }
        }
    }
    if &observed != declared {
        return Err(rejected(
            "owned-engine payload inventory changed during inspection",
        ));
    }
    Ok(())
}

fn required_stages(stages: &[BuildStage], owned_v1186: bool) -> bool {
    let expected = if owned_v1186 {
        [
            ("win32-support", "Win32"),
            ("x64-user-core", "x64"),
            ("x64-wdk-kernel", "x64"),
        ]
    } else {
        [
            ("win32-support", "Win32"),
            ("x64-core", "x64"),
            ("x64-kernel", "x64"),
        ]
    };
    stages.len() == expected.len()
        && stages.iter().zip(expected).all(|(actual, (id, platform))| {
            actual.id == id && actual.platform == platform && actual.state == "COMPILED"
        })
}

/// Read-only checksum and layout inspection. A result is NEVER approval to
/// install the upstream kernel driver or launch a separate Minecraft account.
pub fn inspect_candidate(root: &Path) -> io::Result<OwnedCandidateInspection> {
    if root.as_os_str().is_empty() || !root.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Owned engine candidate requires an absolute directory",
        ));
    }
    let metadata = fs::symlink_metadata(root)?;
    if is_redirected_entry(&metadata) || !metadata.is_dir() {
        return Err(rejected(
            "owned-engine candidate root must be an ordinary directory",
        ));
    }
    let manifest_path = safe_relative_file(root, "engine-provenance.json")?;
    let raw = read_bounded_file(&manifest_path, 256 * 1024)?;
    let manifest: StagedManifest = serde_json::from_slice(&raw)
        .map_err(|_| rejected("invalid owned-engine provenance schema"))?;
    let private = manifest.state == PRIVATE_STATE;
    let owned_v1186 = manifest.state == OWNED_V1186_STATE;
    let hex = |value: &Option<String>, bytes: usize| {
        value.as_ref().is_some_and(|x| {
            x.len() == bytes
                && x.bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        })
    };
    let legacy_fields_absent = manifest.source_version.is_none()
        && manifest.upstream_tree.is_none()
        && manifest.owned_repo_head.is_none()
        && manifest.owned_tree.is_none()
        && manifest.native_modified_files.is_none()
        && manifest.build_report_sha256.is_none()
        && manifest.compiler_sha256.is_none();
    let private_provenance_correct = if owned_v1186 {
        manifest.private_core_build.is_none()
            && manifest.private_fork_head.is_none()
            && manifest.private_fork_tree.is_none()
            && manifest.private_patch_sha256.is_none()
            && manifest.source_build_marker.is_none()
            && manifest.source_files_modified.is_none()
            && manifest.can_coexist_with_official_sandboxie == Some(false)
            && manifest.source_version.as_deref() == Some("v1.18.6")
            && manifest.upstream_tree.as_deref() == Some(OWNED_V1186_TREE)
            && hex(&manifest.owned_repo_head, 40)
            && hex(&manifest.owned_tree, 40)
            && manifest.native_modified_files == Some(11)
            && hex(&manifest.build_report_sha256, 64)
            && hex(&manifest.compiler_sha256, 64)
    } else if private && legacy_fields_absent {
        manifest.private_core_build == Some(true)
            && manifest.private_fork_head.as_ref().is_some_and(|sha| {
                sha.len() == 40
                    && sha != PINNED_UPSTREAM
                    && sha
                        .bytes()
                        .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
            })
            && manifest.private_fork_tree.as_ref().is_some_and(|sha| {
                sha.len() == 40
                    && sha
                        .bytes()
                        .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
            })
            && manifest.private_patch_sha256.as_ref().is_some_and(|sha| {
                sha.len() == 64
                    && sha
                        .bytes()
                        .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
            })
            && manifest.source_build_marker.as_deref() == Some("MIVUBI_PRIVATE_ISOLATION_BUILD")
            && manifest.can_coexist_with_official_sandboxie == Some(false)
            && manifest
                .source_files_modified
                .as_ref()
                .is_some_and(|files| {
                    files.len() == PRIVATE_SOURCE_FILES.len()
                        && files
                            .iter()
                            .zip(PRIVATE_SOURCE_FILES.iter())
                            .all(|(actual, expected)| actual.as_str() == *expected)
                })
    } else {
        legacy_fields_absent
            && manifest.private_core_build.is_none()
            && manifest.private_fork_head.is_none()
            && manifest.private_fork_tree.is_none()
            && manifest.private_patch_sha256.is_none()
            && manifest.source_build_marker.is_none()
            && manifest.source_files_modified.is_none()
            && manifest.can_coexist_with_official_sandboxie.is_none()
    };
    if manifest.schema != (if owned_v1186 { 2 } else { 1 })
        || !(manifest.state == EXPECTED_STATE || private || owned_v1186)
        || !private_provenance_correct
        || manifest.upstream_repository != "sandboxie-plus/Sandboxie"
        || manifest.upstream_tag != (if owned_v1186 { "v1.18.6" } else { "v1.18.5" })
        || manifest.upstream_commit
            != (if owned_v1186 {
                OWNED_V1186_UPSTREAM
            } else {
                PINNED_UPSTREAM
            })
        || manifest.windows_kernel_driver_loaded
        || manifest.minecraft_gameplay_verified
        || manifest.sandboxie_external_dependency_removed
        || manifest.public_distribution_approved
        || !required_stages(&manifest.build_stages, owned_v1186)
        || !(REQUIRED_FILES.len()..=48).contains(&manifest.artifacts.len())
    {
        return Err(rejected(
            "owned-engine manifest makes unproven runtime claims",
        ));
    }

    let mut seen: HashSet<String> = HashSet::new();
    for artifact in &manifest.artifacts {
        if !seen.insert(artifact.path.clone()) {
            return Err(rejected("duplicate owned-engine artifact path"));
        }
        if expected_source_for_payload(&artifact.path).as_deref() != Some(artifact.source.as_str())
        {
            return Err(rejected(
                "owned-engine artifact path/source provenance mismatch",
            ));
        }
        if artifact.sha256.len() != 64
            || !artifact
                .sha256
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            return Err(rejected("invalid owned-engine artifact checksum"));
        }
        let path = safe_relative_file(root, &artifact.path)?;
        if sha256_file(&path, artifact.size_bytes)? != artifact.sha256 {
            return Err(rejected(
                "owned-engine artifact checksum does not match bytes",
            ));
        }
    }
    if !REQUIRED_FILES.iter().all(|name| seen.contains(*name))
        || (owned_v1186 && !seen.contains("LICENSES/LICENSE.Classic"))
    {
        return Err(rejected(
            "incomplete owned-engine driver/service/DLL, helpers or license closure",
        ));
    }
    // Explicit staging marker must accompany the purported uninstalled bundle.
    let marker = safe_relative_file(root, "NOT-INSTALLABLE.txt")?;
    let marker_bytes = read_bounded_file(&marker, 1024)?;
    let expected_marker = if owned_v1186 {
        OWNED_V1186_MARKER
    } else {
        expected_not_installable_marker(private)
    };
    if marker_bytes != expected_marker.as_bytes() {
        return Err(rejected(
            "owned-engine safety marker is missing or modified",
        ));
    }
    let mut declared = seen;
    declared.insert("engine-provenance.json".into());
    declared.insert("NOT-INSTALLABLE.txt".into());
    verify_payload_inventory(root, &declared)?;
    Ok(OwnedCandidateInspection {
        schema: 1,
        state: "CANDIDATE_INTEGRITY_VERIFIED_NOT_INSTALLABLE".into(),
        upstream_commit: manifest.upstream_commit.clone(),
        verified_artifact_count: manifest.artifacts.len() as u32,
        required_artifact_count: (REQUIRED_FILES.len() + usize::from(owned_v1186)) as u32,
        provider_selected: false,
        driver_load_verified: false,
        gameplay_verified: false,
        install_permitted: false,
    })
}

pub fn inspect_candidate_path(path: String) -> io::Result<OwnedCandidateInspection> {
    inspect_candidate(Path::new(&path))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct OwnedHostProbe {
    schema: u32,
    evidence_class: String,
    interactive_session: bool,
    inspection_status: String,
    service_registration: String,
    driver_registration: String,
    provider_selected: bool,
    trusted_driver_loaded: bool,
    gameplay_verified: bool,
}

/// A read-only Windows SCM namespace observation. A service name is not
/// trustworthy attribution to our payload; it may belong to official Sandboxie.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnedHostInspection {
    pub schema: u32,
    pub state: String,
    pub interactive_session: bool,
    pub service_registration: String,
    pub driver_registration: String,
    pub service_namespace_available: bool,
    pub provider_selected: bool,
    pub trusted_driver_loaded: bool,
    pub gameplay_verified: bool,
    pub activation_allowed: bool,
}

// Legacy JSON fixture parser retained only for regression tests. Production
// readiness calls interpret_host_probe directly with native SCM results.
#[cfg(test)]
fn parse_host_probe(raw: &str) -> io::Result<OwnedHostInspection> {
    if raw.len() > 4096 {
        return Err(rejected("owned host service probe exceeded bounded size"));
    }
    let probe: OwnedHostProbe = serde_json::from_str(raw)
        .map_err(|_| rejected("owned host service probe has invalid schema"))?;
    interpret_host_probe(probe)
}

fn interpret_host_probe(probe: OwnedHostProbe) -> io::Result<OwnedHostInspection> {
    let allowed = |s: &str| matches!(s, "PRESENT" | "ABSENT" | "UNKNOWN");
    let unknown = probe.service_registration == "UNKNOWN" || probe.driver_registration == "UNKNOWN";
    if probe.schema != 1
        || probe.evidence_class != "WINDOWS_SERVICE_NAME_ONLY_NO_OWNER_PROOF"
        || !allowed(&probe.service_registration)
        || !allowed(&probe.driver_registration)
        || (probe.inspection_status == "PARTIAL_UNKNOWN") != unknown
        || !matches!(
            probe.inspection_status.as_str(),
            "OBSERVED" | "PARTIAL_UNKNOWN"
        )
        || probe.provider_selected
        || probe.trusted_driver_loaded
        || probe.gameplay_verified
    {
        return Err(rejected(
            "owned host service probe is contradictory or overclaims trust",
        ));
    }
    // Even an empty SCM namespace has no signed driver, configured isolation
    // profile, trusted IPC protocol or proved disposable-host provenance.
    let empty_namespace =
        !unknown && probe.service_registration == "ABSENT" && probe.driver_registration == "ABSENT";
    let state = if !probe.interactive_session {
        "BLOCKED_NON_INTERACTIVE_SESSION"
    } else if probe.service_registration == "PRESENT" || probe.driver_registration == "PRESENT" {
        "BLOCKED_SERVICE_NAMESPACE_IN_USE"
    } else if unknown {
        "BLOCKED_SERVICE_PROBE_UNKNOWN"
    } else {
        "BLOCKED_UNVERIFIED_OWNED_DRIVER"
    };
    Ok(OwnedHostInspection {
        schema: 1,
        state: state.into(),
        interactive_session: probe.interactive_session,
        service_registration: probe.service_registration,
        driver_registration: probe.driver_registration,
        service_namespace_available: empty_namespace,
        provider_selected: false,
        trusted_driver_loaded: false,
        gameplay_verified: false,
        activation_allowed: false,
    })
}

// Windows ERROR_SERVICE_DOES_NOT_EXIST, not an ambiguous API failure.
// Access denied, invalid handle, SCM unavailable and other statuses stay
// UNKNOWN so no service-owner assumption can authorize a driver install.
const SERVICE_DOES_NOT_EXIST: u32 = 1060;

fn classify_service_open_error(error: u32) -> &'static str {
    if error == SERVICE_DOES_NOT_EXIST {
        "ABSENT"
    } else {
        "UNKNOWN"
    }
}

#[cfg(windows)]
fn read_native_service_registration(
    manager: windows_sys::Win32::System::Services::SC_HANDLE,
    name: &str,
) -> &'static str {
    use windows_sys::Win32::{
        Foundation::GetLastError,
        System::Services::{CloseServiceHandle, OpenServiceW, SERVICE_QUERY_STATUS},
    };
    let wide_name: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    let service = unsafe { OpenServiceW(manager, wide_name.as_ptr(), SERVICE_QUERY_STATUS) };
    if !service.is_null() {
        unsafe {
            CloseServiceHandle(service);
        }
        "PRESENT"
    } else {
        // Must read GetLastError immediately after the failed OpenServiceW.
        classify_service_open_error(unsafe { GetLastError() })
    }
}

/// Direct read-only Windows SCM inspection. No PowerShell/CIM child, no
/// WMI provider start and no repeated subprocess/stream worker allocation.
/// SERVICE_QUERY_STATUS does not start, stop or alter any registered service.
/// The result proves service NAMES only, never that a driver is our own.
#[cfg(windows)]
pub fn inspect_host_service_namespace() -> io::Result<OwnedHostInspection> {
    use windows_sys::Win32::System::{
        RemoteDesktop::ProcessIdToSessionId,
        Services::{CloseServiceHandle, OpenSCManagerW, SC_MANAGER_CONNECT},
        Threading::GetCurrentProcessId,
    };
    let mut session_id = 0_u32;
    let session_known =
        unsafe { ProcessIdToSessionId(GetCurrentProcessId(), &mut session_id) } != 0;
    let manager = unsafe { OpenSCManagerW(std::ptr::null(), std::ptr::null(), SC_MANAGER_CONNECT) };
    let (service, driver) = if manager.is_null() {
        ("UNKNOWN", "UNKNOWN")
    } else {
        let service = read_native_service_registration(manager, "SbieSvc");
        let driver = read_native_service_registration(manager, "SbieDrv");
        unsafe {
            CloseServiceHandle(manager);
        }
        (service, driver)
    };
    let unknown = service == "UNKNOWN" || driver == "UNKNOWN";
    interpret_host_probe(OwnedHostProbe {
        schema: 1,
        evidence_class: "WINDOWS_SERVICE_NAME_ONLY_NO_OWNER_PROOF".into(),
        interactive_session: session_known && session_id != 0,
        inspection_status: if unknown {
            "PARTIAL_UNKNOWN"
        } else {
            "OBSERVED"
        }
        .into(),
        service_registration: service.into(),
        driver_registration: driver.into(),
        provider_selected: false,
        trusted_driver_loaded: false,
        gameplay_verified: false,
    })
}

#[cfg(not(windows))]
pub fn inspect_host_service_namespace() -> io::Result<OwnedHostInspection> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Owned host service inspection requires interactive Windows",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    #[ignore = "requires an explicitly selected real owned v1.18.6 staged payload"]
    fn actual_staged_owned_payload_remains_not_installable() {
        let directory = std::env::var("MIVUBI_TEST_OWNED_PAYLOAD")
            .expect("set MIVUBI_TEST_OWNED_PAYLOAD to the absolute staged payload directory");
        let inspection = inspect_candidate_path(directory).expect("inspect actual staged payload");
        assert_eq!(inspection.upstream_commit, OWNED_V1186_UPSTREAM);
        assert_eq!(inspection.verified_artifact_count, 25);
        assert_eq!(inspection.required_artifact_count, 25);
        assert_eq!(
            inspection.state,
            "CANDIDATE_INTEGRITY_VERIFIED_NOT_INSTALLABLE"
        );
        assert!(!inspection.provider_selected);
        assert!(!inspection.driver_load_verified);
        assert!(!inspection.gameplay_verified);
        assert!(!inspection.install_permitted);
        let host =
            inspect_host_service_namespace().expect("read-only Windows service namespace probe");
        assert!(!host.activation_allowed);
        assert!(!host.provider_selected);
        assert!(!host.trusted_driver_loaded);
        assert!(!host.gameplay_verified);
        println!(
            "Candidate integrity verified: 25 files. Host service={}, driver={}, state={}; activation refused.",
            host.service_registration, host.driver_registration, host.state
        );
    }

    struct TempCandidate(PathBuf);
    impl TempCandidate {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "mivubi-owned-candidate-{}-{unique}",
                std::process::id()
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn add_file(&self, path: &str, content: &[u8]) -> serde_json::Value {
            let target = self.0.join(path);
            fs::create_dir_all(target.parent().unwrap()).unwrap();
            fs::write(&target, content).unwrap();
            let mut hash = Sha256::new();
            hash.update(content);
            serde_json::json!({
                "path": path,
                "source": expected_source_for_payload(path).expect("canonical fixture path"),
                "sizeBytes": content.len(),
                "sha256": hash.finalize().iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
            })
        }
        fn populate(&self) {
            let mut files = Vec::new();
            for path in REQUIRED_FILES {
                files.push(self.add_file(path, format!("fixture:{path}").as_bytes()));
            }
            fs::write(
                self.0.join("NOT-INSTALLABLE.txt"),
                expected_not_installable_marker(false),
            )
            .unwrap();
            let provenance = serde_json::json!({
                "schema": 1, "state": EXPECTED_STATE, "upstreamRepository": "sandboxie-plus/Sandboxie",
                "upstreamTag": "v1.18.5", "upstreamCommit": PINNED_UPSTREAM,
                "buildStages": [
                    {"id":"win32-support","platform":"Win32","state":"COMPILED"},
                    {"id":"x64-core","platform":"x64","state":"COMPILED"},
                    {"id":"x64-kernel","platform":"x64","state":"COMPILED"}
                ],
                "artifacts": files,
                "windowsKernelDriverLoaded": false,
                "minecraftGameplayVerified": false,
                "sandboxieExternalDependencyRemoved": false,
                "publicDistributionApproved": false
            });
            fs::write(
                self.0.join("engine-provenance.json"),
                serde_json::to_vec(&provenance).unwrap(),
            )
            .unwrap();
        }
    }
    impl Drop for TempCandidate {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn host_probe(service: &str, driver: &str, interactive: bool) -> String {
        let unknown = service == "UNKNOWN" || driver == "UNKNOWN";
        serde_json::json!({
            "schema": 1,
            "evidenceClass": "WINDOWS_SERVICE_NAME_ONLY_NO_OWNER_PROOF",
            "interactiveSession": interactive,
            "inspectionStatus": if unknown { "PARTIAL_UNKNOWN" } else { "OBSERVED" },
            "serviceRegistration": service,
            "driverRegistration": driver,
            "providerSelected": false,
            "trustedDriverLoaded": false,
            "gameplayVerified": false
        })
        .to_string()
    }

    #[test]
    fn native_scm_error_classification_is_strict_and_non_destructive() {
        assert_eq!(classify_service_open_error(1060), "ABSENT");
        // Error 5 = access denied, 6 = invalid handle, 123 = bad name,
        // 0 = non-error/unspecified; NONE proves service absence.
        for error in [0, 5, 6, 123, 87, 1722, u32::MAX] {
            assert_eq!(classify_service_open_error(error), "UNKNOWN");
        }
    }

    #[test]
    fn direct_scm_results_keep_the_legacy_public_host_contract_blocked() {
        for (service, driver, interactive, expected) in [
            ("ABSENT", "ABSENT", true, "BLOCKED_UNVERIFIED_OWNED_DRIVER"),
            (
                "PRESENT",
                "ABSENT",
                true,
                "BLOCKED_SERVICE_NAMESPACE_IN_USE",
            ),
            (
                "ABSENT",
                "PRESENT",
                true,
                "BLOCKED_SERVICE_NAMESPACE_IN_USE",
            ),
            ("UNKNOWN", "ABSENT", true, "BLOCKED_SERVICE_PROBE_UNKNOWN"),
            ("ABSENT", "ABSENT", false, "BLOCKED_NON_INTERACTIVE_SESSION"),
        ] {
            let probe = OwnedHostProbe {
                schema: 1,
                evidence_class: "WINDOWS_SERVICE_NAME_ONLY_NO_OWNER_PROOF".into(),
                interactive_session: interactive,
                inspection_status: if service == "UNKNOWN" || driver == "UNKNOWN" {
                    "PARTIAL_UNKNOWN"
                } else {
                    "OBSERVED"
                }
                .into(),
                service_registration: service.into(),
                driver_registration: driver.into(),
                provider_selected: false,
                trusted_driver_loaded: false,
                gameplay_verified: false,
            };
            let result = interpret_host_probe(probe).unwrap();
            assert_eq!(result.state, expected);
            assert!(
                !result.activation_allowed
                    && !result.trusted_driver_loaded
                    && !result.provider_selected
                    && !result.gameplay_verified
            );
        }
    }

    #[test]
    fn no_service_names_still_does_not_authorize_install_or_launch() {
        let result = parse_host_probe(&host_probe("ABSENT", "ABSENT", true)).unwrap();
        assert_eq!(result.state, "BLOCKED_UNVERIFIED_OWNED_DRIVER");
        assert!(result.service_namespace_available);
        assert!(
            !result.activation_allowed
                && !result.provider_selected
                && !result.trusted_driver_loaded
                && !result.gameplay_verified
        );
    }

    #[test]
    fn service_and_driver_names_block_an_ambiguous_owned_runtime() {
        for (service, driver) in [
            ("PRESENT", "ABSENT"),
            ("ABSENT", "PRESENT"),
            ("PRESENT", "PRESENT"),
        ] {
            let result = parse_host_probe(&host_probe(service, driver, true)).unwrap();
            assert_eq!(result.state, "BLOCKED_SERVICE_NAMESPACE_IN_USE");
            assert!(!result.activation_allowed);
        }
    }

    #[test]
    fn unknown_or_noninteractive_host_evidence_blocks_activation() {
        let result = parse_host_probe(&host_probe("UNKNOWN", "ABSENT", true)).unwrap();
        assert_eq!(result.state, "BLOCKED_SERVICE_PROBE_UNKNOWN");
        assert!(!result.service_namespace_available);
        let result = parse_host_probe(&host_probe("ABSENT", "ABSENT", false)).unwrap();
        assert_eq!(result.state, "BLOCKED_NON_INTERACTIVE_SESSION");
        assert!(!result.activation_allowed);
    }

    #[test]
    fn rejects_fabricated_service_provenance_and_true_ready_fields() {
        let raw = host_probe("ABSENT", "ABSENT", true);
        for (field, value) in [
            ("providerSelected", true),
            ("trustedDriverLoaded", true),
            ("gameplayVerified", true),
        ] {
            let mut data: serde_json::Value = serde_json::from_str(&raw).unwrap();
            data[field] = serde_json::json!(value);
            assert!(parse_host_probe(&data.to_string()).is_err());
        }
        let mut data: serde_json::Value = serde_json::from_str(&raw).unwrap();
        data["inspectionStatus"] = serde_json::json!("PARTIAL_UNKNOWN");
        assert!(parse_host_probe(&data.to_string()).is_err());
        data["inspectionStatus"] = serde_json::json!("OBSERVED");
        data["serviceRegistration"] = serde_json::json!("RUNNING");
        assert!(parse_host_probe(&data.to_string()).is_err());
    }

    #[test]
    fn owned_v1186_package_uses_real_stage_identity_and_remains_uninstallable() {
        let c = TempCandidate::new();
        c.populate();
        let path = c.0.join("engine-provenance.json");
        let mut data: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        let classic = c.add_file("LICENSES/LICENSE.Classic", b"GPL classic notice fixture");
        data["artifacts"].as_array_mut().unwrap().push(classic);
        data["schema"] = serde_json::json!(2);
        data["state"] = serde_json::json!(OWNED_V1186_STATE);
        data["sourceVersion"] = serde_json::json!("v1.18.6");
        data["upstreamTag"] = serde_json::json!("v1.18.6");
        data["upstreamCommit"] = serde_json::json!(OWNED_V1186_UPSTREAM);
        data["upstreamTree"] = serde_json::json!(OWNED_V1186_TREE);
        data["ownedRepoHead"] = serde_json::json!("a".repeat(40));
        data["ownedTree"] = serde_json::json!("b".repeat(40));
        data["nativeModifiedFiles"] = serde_json::json!(11);
        data["buildReportSha256"] = serde_json::json!("c".repeat(64));
        data["compilerSha256"] = serde_json::json!("d".repeat(64));
        data["canCoexistWithOfficialSandboxie"] = serde_json::json!(false);
        data["buildStages"][1]["id"] = serde_json::json!("x64-user-core");
        data["buildStages"][2]["id"] = serde_json::json!("x64-wdk-kernel");
        fs::write(c.0.join("NOT-INSTALLABLE.txt"), OWNED_V1186_MARKER).unwrap();
        fs::write(&path, serde_json::to_vec(&data).unwrap()).unwrap();
        let inspected = inspect_candidate(&c.0).unwrap();
        assert_eq!(inspected.upstream_commit, OWNED_V1186_UPSTREAM);
        assert_eq!(inspected.verified_artifact_count, 25);
        assert_eq!(inspected.required_artifact_count, 25);
        assert!(
            !inspected.provider_selected
                && !inspected.install_permitted
                && !inspected.driver_load_verified
                && !inspected.gameplay_verified
        );
        for (key, wrong) in [
            ("nativeModifiedFiles", serde_json::json!(10)),
            ("sourceVersion", serde_json::json!("v1.18.5")),
            ("compilerSha256", serde_json::json!("invalid")),
            ("upstreamTree", serde_json::json!("not-pinned")),
            ("canCoexistWithOfficialSandboxie", serde_json::json!(true)),
        ] {
            let saved = data[key].clone();
            data[key] = wrong;
            fs::write(&path, serde_json::to_vec(&data).unwrap()).unwrap();
            assert!(inspect_candidate(&c.0).is_err(), "accepted invalid {key}");
            data[key] = saved;
        }
        data["buildStages"][1]["id"] = serde_json::json!("x64-core");
        fs::write(&path, serde_json::to_vec(&data).unwrap()).unwrap();
        assert!(inspect_candidate(&c.0).is_err());
    }

    #[test]
    fn private_exclusive_candidate_is_inspected_but_never_selected_or_installable() {
        let c = TempCandidate::new();
        c.populate();
        let path = c.0.join("engine-provenance.json");
        let mut data: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        data["state"] = serde_json::json!(PRIVATE_STATE);
        fs::write(
            c.0.join("NOT-INSTALLABLE.txt"),
            expected_not_installable_marker(true),
        )
        .unwrap();
        data["privateCoreBuild"] = serde_json::json!(true);
        data["privateForkHead"] = serde_json::json!("a".repeat(40));
        data["privateForkTree"] = serde_json::json!("b".repeat(40));
        data["privatePatchSha256"] = serde_json::json!("c".repeat(64));
        data["sourceBuildMarker"] = serde_json::json!("MIVUBI_PRIVATE_ISOLATION_BUILD");
        data["sourceFilesModified"] = serde_json::json!(PRIVATE_SOURCE_FILES);
        data["canCoexistWithOfficialSandboxie"] = serde_json::json!(false);
        fs::write(&path, serde_json::to_vec(&data).unwrap()).unwrap();
        let valid = inspect_candidate(&c.0).unwrap();
        assert_eq!(valid.state, "CANDIDATE_INTEGRITY_VERIFIED_NOT_INSTALLABLE");
        assert!(
            !valid.provider_selected && !valid.install_permitted && !valid.driver_load_verified
        );
        data["privatePatchSha256"] = serde_json::json!("not-a-digest");
        fs::write(&path, serde_json::to_vec(&data).unwrap()).unwrap();
        assert!(inspect_candidate(&c.0).is_err());
        data["privatePatchSha256"] = serde_json::json!("c".repeat(64));
        data["privateForkHead"] = serde_json::json!(PINNED_UPSTREAM);
        fs::write(&path, serde_json::to_vec(&data).unwrap()).unwrap();
        assert!(inspect_candidate(&c.0).is_err());
        data["privateForkHead"] = serde_json::json!("a".repeat(40));
        data["privateForkTree"] = serde_json::json!("bad-tree");
        fs::write(&path, serde_json::to_vec(&data).unwrap()).unwrap();
        assert!(inspect_candidate(&c.0).is_err());
        data["privateForkTree"] = serde_json::json!("b".repeat(40));
        data["canCoexistWithOfficialSandboxie"] = serde_json::json!(true);
        fs::write(path, serde_json::to_vec(&data).unwrap()).unwrap();
        assert!(inspect_candidate(&c.0).is_err());
    }

    #[test]
    fn original_upstream_candidate_must_not_claim_private_fork_identity() {
        let c = TempCandidate::new();
        c.populate();
        let path = c.0.join("engine-provenance.json");
        let mut data: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        data["privateCoreBuild"] = serde_json::json!(true);
        fs::write(path, serde_json::to_vec(&data).unwrap()).unwrap();
        assert!(inspect_candidate(&c.0).is_err());
    }

    #[test]
    fn candidate_source_fixture_validates_hashes_but_never_proves_driver_load() {
        let c = TempCandidate::new();
        c.populate();
        let result = inspect_candidate(&c.0).unwrap();
        assert_eq!(result.verified_artifact_count, REQUIRED_FILES.len() as u32);
        assert!(
            !result.provider_selected
                && !result.driver_load_verified
                && !result.gameplay_verified
                && !result.install_permitted
        );
    }

    #[test]
    fn candidate_rejects_swapped_source_attribution_even_when_bytes_match() {
        let c = TempCandidate::new();
        c.populate();
        let path = c.0.join("engine-provenance.json");
        let mut data: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        let artifacts = data["artifacts"].as_array_mut().unwrap();
        let row = artifacts
            .iter_mut()
            .find(|row| row["path"] == "SbieDrv.sys")
            .unwrap();
        row["source"] = serde_json::json!("Sandboxie/Bin/x64/SbieRelease/Start.exe");
        fs::write(&path, serde_json::to_vec(&data).unwrap()).unwrap();
        assert!(inspect_candidate(&c.0).is_err());
    }

    #[test]
    fn candidate_rejects_unexpected_payload_entry_even_with_matching_checksum() {
        let c = TempCandidate::new();
        c.populate();
        let bytes = b"unknown fixture file";
        fs::write(c.0.join("unexpected.exe"), bytes).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let unexpected = serde_json::json!({
            "path": "unexpected.exe",
            "source": "Sandboxie/Bin/x64/SbieRelease/unexpected.exe",
            "sizeBytes": bytes.len(),
            "sha256": hasher.finalize().iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
        });
        let path = c.0.join("engine-provenance.json");
        let mut data: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        data["artifacts"].as_array_mut().unwrap().push(unexpected);
        fs::write(&path, serde_json::to_vec(&data).unwrap()).unwrap();
        assert!(inspect_candidate(&c.0).is_err());
    }

    #[test]
    fn candidate_safety_marker_content_must_match_exact_payload_state() {
        let c = TempCandidate::new();
        c.populate();
        fs::write(
            c.0.join("NOT-INSTALLABLE.txt"),
            b"not an installation marker",
        )
        .unwrap();
        assert!(inspect_candidate(&c.0).is_err());
    }

    #[test]
    fn candidate_must_include_qsbieapi_license_attribution() {
        let c = TempCandidate::new();
        c.populate();
        fs::remove_file(c.0.join("LICENSES/SandboxiePlus/QSbieAPI/LICENSE")).unwrap();
        assert!(inspect_candidate(&c.0).is_err());
    }

    #[test]
    fn optional_debug_symbols_preserve_expected_source_provenance() {
        let c = TempCandidate::new();
        c.populate();
        let extra = c.add_file("_debug/x64/SbieDrv.pdb", b"synthetic symbol file");
        let path = c.0.join("engine-provenance.json");
        let mut data: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        data["artifacts"].as_array_mut().unwrap().push(extra);
        fs::write(&path, serde_json::to_vec(&data).unwrap()).unwrap();
        let valid = inspect_candidate(&c.0).unwrap();
        assert_eq!(
            valid.verified_artifact_count,
            (REQUIRED_FILES.len() + 1) as u32
        );
        assert!(!valid.install_permitted);
    }

    #[test]
    fn oversized_manifest_is_refused_before_deserializing() {
        let c = TempCandidate::new();
        c.populate();
        fs::write(
            c.0.join("engine-provenance.json"),
            vec![b'X'; 256 * 1024 + 1],
        )
        .unwrap();
        assert!(inspect_candidate(&c.0).is_err());
    }

    #[test]
    fn oversized_safety_marker_is_refused_without_unbounded_allocation() {
        let c = TempCandidate::new();
        c.populate();
        fs::write(c.0.join("NOT-INSTALLABLE.txt"), vec![b'X'; 1025]).unwrap();
        assert!(inspect_candidate(&c.0).is_err());
    }

    #[test]
    fn candidate_rejects_unlisted_file_without_manifest_entry() {
        let c = TempCandidate::new();
        c.populate();
        fs::write(c.0.join("not-listed.exe"), b"unexpected extra file").unwrap();
        assert!(inspect_candidate(&c.0).is_err());
    }

    #[test]
    fn candidate_rejects_unlisted_nested_directory_or_file() {
        let c = TempCandidate::new();
        c.populate();
        let extra_dir = c.0.join("LICENSES/other");
        fs::create_dir(&extra_dir).unwrap();
        fs::write(extra_dir.join("extra.txt"), b"extra notice").unwrap();
        assert!(inspect_candidate(&c.0).is_err());
    }

    #[test]
    fn candidate_rejects_unlisted_nested_file_in_allowed_directory() {
        let c = TempCandidate::new();
        c.populate();
        fs::write(c.0.join("32/unlisted.dll"), b"unknown helper").unwrap();
        assert!(inspect_candidate(&c.0).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn candidate_rejects_unlisted_symlink_at_inventory_scan() {
        use std::os::unix::fs::symlink;
        let c = TempCandidate::new();
        c.populate();
        symlink(c.0.join("Start.exe"), c.0.join("redirect.exe")).unwrap();
        assert!(inspect_candidate(&c.0).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn candidate_rejects_symlinked_root_directory() {
        use std::os::unix::fs::symlink;
        let c = TempCandidate::new();
        c.populate();
        let alias = c.0.with_extension("alias");
        symlink(&c.0, &alias).unwrap();
        assert!(inspect_candidate(&alias).is_err());
        fs::remove_file(alias).unwrap();
    }

    #[test]
    fn candidate_checksum_mismatch_is_rejected() {
        let c = TempCandidate::new();
        c.populate();
        fs::write(c.0.join("SbieDrv.sys"), b"tampered binary").unwrap();
        assert!(inspect_candidate(&c.0).is_err());
    }

    #[test]
    fn candidate_missing_license_is_rejected() {
        let c = TempCandidate::new();
        c.populate();
        fs::remove_file(c.0.join("LICENSES/Sandboxie/COPYING")).unwrap();
        assert!(inspect_candidate(&c.0).is_err());
    }

    #[test]
    fn candidate_runtime_claim_is_rejected() {
        let c = TempCandidate::new();
        c.populate();
        let path = c.0.join("engine-provenance.json");
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        manifest["windowsKernelDriverLoaded"] = true.into();
        fs::write(path, serde_json::to_vec(&manifest).unwrap()).unwrap();
        assert!(inspect_candidate(&c.0).is_err());
    }
}
