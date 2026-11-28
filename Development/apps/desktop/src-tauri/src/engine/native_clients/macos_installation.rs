//! Explicit Mac installation candidate inspection, never game admission.
//! Only fixed Applications paths; no account/profile data, game execution,
//! publisher pin, updater operation, or arbitrary caller-supplied path.
//! On-demand AppKit discovery is observation only, never adoption.
use core_foundation::{base::{CFType, TCFType}, string::CFString, url::CFURL};
use security_framework_sys::code_signing::{
    kSecCSCheckAllArchitectures, kSecCSCheckNestedCode, kSecCSNoNetworkAccess,
    kSecCSSingleThreaded, kSecCSStrictValidate, SecRequirementCreateWithString,
    SecStaticCodeCheckValidity, SecStaticCodeCreateWithPath,
    SecRequirementRef, SecStaticCodeRef,
};
use serde::{Deserialize, Serialize};
use objc2::{rc::autoreleasepool, ClassType};
use objc2_app_kit::NSRunningApplication;
use objc2_foundation::{NSObjectProtocol, NSString};
use std::{
    collections::HashSet,
    fs::{self, Metadata, OpenOptions},
    io::{self, Cursor, Read},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::Path,
};

// Current Homebrew/original bundle names plus Microsoft's historical Intune
// sample install name. These are discovery hints, not product identity.
const CANDIDATES: [&str; 3] = ["Minecraft Education.app", "minecraft-edu.app", "Minecraft EE.app"];
// User-supplied 1.26.33.0 Info.plist establishes the expected metadata
// identity, not the cryptographically verified signer or loaded image.
const EXPECTED_BUNDLE_ID: &str = "com.microsoft.minecraft-edu";
const EXPECTED_EXECUTABLE: &str = "minecraft-edu";
// Installomator's reviewed minecraftedu label expects UBF8T346G9, also
// documented by Microsoft for other products. Not yet checked against
// the actual official Education DMG. Require the expected code identifier
// and Developer ID Application certificate OIDs (TN3127), not team alone.
// The code identifier must be corroborated against the actual signed binary.
const EXPECTED_TEAM_REQUIREMENT: &str = r#"identifier "com.microsoft.minecraft-edu" and anchor apple generic and certificate 1[field.1.2.840.113635.100.6.2.6] and certificate leaf[field.1.2.840.113635.100.6.1.13] and certificate leaf[subject.OU] = "UBF8T346G9""#;
const MAX_PLIST: u64 = 256 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
enum CandidateStatus {
    Missing,
    Candidate,
    Ambiguous,
    Unconfirmed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
enum SignatureObservation {
    NotChecked,
    ExpectedTeamMatched,
    Unconfirmed,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MacEducationInstallationReport {
    schema: u32,
    platform: &'static str,
    status: CandidateStatus,
    version: Option<String>,
    signature: SignatureObservation,
    publisher: &'static str,
    multiple_instances_prohibited: Option<bool>,
    observed_application_count: Option<u32>,
    runtime_admission: bool,
}

fn report(status: CandidateStatus, version: Option<String>) -> MacEducationInstallationReport {
    MacEducationInstallationReport {
        schema: 1,
        platform: "macos",
        status,
        version,
        signature: SignatureObservation::NotChecked,
        publisher: "NotPinned",
        multiple_instances_prohibited: None,
        observed_application_count: None,
        runtime_admission: false,
    }
}

#[derive(Deserialize)]
struct BundleMetadata {
    #[serde(rename = "CFBundlePackageType")]
    package_type: String,
    #[serde(rename = "CFBundleIdentifier")]
    identifier: String,
    #[serde(rename = "CFBundleExecutable")]
    executable: String,
    #[serde(rename = "CFBundleShortVersionString")]
    version: String,
    #[serde(rename = "CFBundleVersion")]
    build: String,
    #[serde(rename = "LSMultipleInstancesProhibited")]
    multiple_instances_prohibited: Option<bool>,
}

fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "Unconfirmed Mac application candidate")
}

fn same_metadata(a: &Metadata, b: &Metadata) -> bool {
    (a.dev(), a.ino(), a.mode(), a.len(), a.mtime(), a.mtime_nsec(), a.ctime(), a.ctime_nsec())
        == (b.dev(), b.ino(), b.mode(), b.len(), b.mtime(), b.mtime_nsec(), b.ctime(), b.ctime_nsec())
}

// The fixed directory/metadata observations are not an atomic loaded-image
// pin. Reject symlinks and common replacement races; never use this for launch.
fn ordinary_directory(path: &Path) -> io::Result<Metadata> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_dir() || fs::canonicalize(path)? != path {
        return Err(invalid());
    }
    Ok(metadata)
}

fn ordinary_file(path: &Path) -> io::Result<Metadata> {
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() || fs::canonicalize(path)? != path {
        return Err(invalid());
    }
    Ok(metadata)
}

fn valid_version(value: &str) -> bool {
    !value.is_empty() && value.len() <= 64 && (1..=5).contains(&value.split('.').count())
        && value.split('.').all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
}

fn parse_metadata(bytes: &[u8]) -> io::Result<BundleMetadata> {
    let metadata: BundleMetadata = plist::from_reader(Cursor::new(bytes)).map_err(|_| invalid())?;
    if metadata.package_type != "APPL"
        || metadata.identifier != EXPECTED_BUNDLE_ID
        || metadata.executable != EXPECTED_EXECUTABLE
        || !valid_version(&metadata.version) || !valid_version(&metadata.build)
    {
        return Err(invalid());
    }
    Ok(metadata)
}

// Expected identifier + Developer ID team is stronger than a generic Apple
// anchor or team alone, but cannot establish the actual official distribution
// pin, notarization or launch admission. No generic fallback or second check.
// No private signing data is serialized. The caller owns a blocking,
// explicitly requested operation and repeats metadata checks afterwards.
fn observe_static_signature(path: &Path) -> SignatureObservation {
    let Some(url) = CFURL::from_path(path, true) else {
        return SignatureObservation::Unconfirmed;
    };
    let text = CFString::new(EXPECTED_TEAM_REQUIREMENT);
    let mut requirement: SecRequirementRef = std::ptr::null_mut();
    let mut code: SecStaticCodeRef = std::ptr::null_mut();
    // Each successful Create returns one owned CF reference. The local
    // CFType guards release exactly those references on every return path.
    unsafe {
        if SecRequirementCreateWithString(text.as_concrete_TypeRef(), 0, &mut requirement) != 0
            || requirement.is_null()
        {
            return SignatureObservation::Unconfirmed;
        }
        let _requirement_owner = CFType::wrap_under_create_rule(requirement.cast());
        if SecStaticCodeCreateWithPath(url.as_concrete_TypeRef(), 0, &mut code) != 0
            || code.is_null()
        {
            return SignatureObservation::Unconfirmed;
        }
        let _code_owner = CFType::wrap_under_create_rule(code.cast());
        let flags = kSecCSCheckAllArchitectures | kSecCSCheckNestedCode
            | kSecCSStrictValidate | kSecCSNoNetworkAccess | kSecCSSingleThreaded;
        if SecStaticCodeCheckValidity(code, flags, requirement) == 0 {
            SignatureObservation::ExpectedTeamMatched
        } else {
            SignatureObservation::Unconfirmed
        }
    }
}

fn inspect_bundle(path: &Path, verify: impl FnOnce(&Path) -> SignatureObservation)
    -> io::Result<(BundleMetadata, SignatureObservation)> {
    let bundle_before = ordinary_directory(path)?;
    let contents = path.join("Contents");
    let contents_before = ordinary_directory(&contents)?;
    let macos = contents.join("MacOS");
    let macos_before = ordinary_directory(&macos)?;
    let info = contents.join("Info.plist");
    let before = ordinary_file(&info)?;
    if !(1..=MAX_PLIST).contains(&before.len()) {
        return Err(invalid());
    }
    // Refuse a replaced leaf symlink or a blocking non-regular file.
    let mut file = OpenOptions::new().read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(&info)?;
    let opened = file.metadata()?;
    if !opened.is_file() || !same_metadata(&before, &opened) {
        return Err(invalid());
    }
    let mut bytes = Vec::new();
    (&mut file).take(MAX_PLIST + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 != before.len() || !same_metadata(&opened, &file.metadata()?) {
        return Err(invalid());
    }
    let metadata = parse_metadata(&bytes)?;
    let executable = macos.join(&metadata.executable);
    let image = ordinary_file(&executable)?;
    if image.len() == 0 || image.mode() & 0o111 == 0 {
        return Err(invalid());
    }
    let signature = verify(path);
    if !same_metadata(&bundle_before, &ordinary_directory(path)?)
        || !same_metadata(&contents_before, &ordinary_directory(&contents)?)
        || !same_metadata(&macos_before, &ordinary_directory(&macos)?)
        || !same_metadata(&before, &ordinary_file(&info)?)
        || !same_metadata(&image, &ordinary_file(&executable)?)
    {
        return Err(invalid());
    }
    // Static code validation is not an atomic loaded-image pin.
    Ok((metadata, signature))
}

fn inspect_at(applications: &Path, verify: impl FnOnce(&Path) -> SignatureObservation) -> MacEducationInstallationReport {
    if ordinary_directory(applications).is_err() {
        return report(CandidateStatus::Unconfirmed, None);
    }
    let mut present = Vec::new();
    for name in CANDIDATES {
        let path = applications.join(name);
        match fs::symlink_metadata(&path) {
            Ok(_) => present.push(path),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {},
            Err(_) => return report(CandidateStatus::Unconfirmed, None),
        }
    }
    match present.as_slice() {
        [] => report(CandidateStatus::Missing, None),
        [path] => match inspect_bundle(path, verify) {
            Ok((metadata, signature)) => MacEducationInstallationReport {
                signature,
                multiple_instances_prohibited: metadata.multiple_instances_prohibited,
                ..report(CandidateStatus::Candidate, Some(metadata.version))
            },
            Err(_) => report(CandidateStatus::Unconfirmed, None),
        },
        _ => report(CandidateStatus::Ambiguous, None),
    }
}

// Recheck the original retained AppKit object, never treat a fresh PID
// lookup as ownership. Apple explicitly requires isEqual for this comparison.
// This is still only a run-loop-sensitive observation, not an OS process
// handle, mapped-image proof, launch receipt or permission to close.
fn reobserve_application_pid(app: &NSRunningApplication) -> Option<libc::pid_t> {
    let pid = app.processIdentifier();
    if pid <= 0 || pid as u32 == std::process::id() || app.isTerminated() {
        return None;
    }
    let current = NSRunningApplication::runningApplicationWithProcessIdentifier(pid)?;
    if !app.isEqual(Some(current.as_super().as_super()))
        || current.isTerminated() || app.isTerminated()
        || current.processIdentifier() != pid || app.processIdentifier() != pid
    {
        return None;
    }
    Some(pid)
}

// Never report a smaller count by silently dropping an ambiguous app.
fn distinct_observation_count(pids: impl IntoIterator<Item = Option<libc::pid_t>>) -> Option<u32> {
    let mut distinct = HashSet::new();
    for observed in pids {
        let pid = observed?;
        if pid <= 0 || !distinct.insert(pid) {
            return None;
        }
    }
    u32::try_from(distinct.len()).ok()
}

// On-demand bundle-id discovery only. Keep all external apps unmanaged;
// release every temporary reference when the observation returns.
fn observe_application_count() -> Option<u32> {
    autoreleasepool(|_| {
        let identifier = NSString::from_str(EXPECTED_BUNDLE_ID);
        let applications = NSRunningApplication::runningApplicationsWithBundleIdentifier(&identifier);
        distinct_observation_count((0..applications.count()).map(|index| {
            let original = applications.objectAtIndex(index);
            reobserve_application_pid(&original)
        }))
    })
}

pub(super) fn inspect() -> MacEducationInstallationReport {
    let mut result = inspect_at(Path::new("/Applications"), observe_static_signature);
    result.observed_application_count = observe_application_count();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::{symlink, PermissionsExt};

    fn fixture(root: &Path, name: &str, exe: &str) {
        let contents = root.join(name).join("Contents");
        fs::create_dir_all(contents.join("MacOS")).unwrap();
        let xml = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0"><dict>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleIdentifier</key><string>com.microsoft.minecraft-edu</string>
<key>CFBundleExecutable</key><string>{exe}</string>
<key>CFBundleShortVersionString</key><string>1.26.33</string>
<key>CFBundleVersion</key><string>1.26.33.0</string>
</dict></plist>"#);
        fs::write(contents.join("Info.plist"), xml).unwrap();
        let file = contents.join("MacOS").join(EXPECTED_EXECUTABLE);
        fs::write(&file, b"synthetic metadata only").unwrap();
        fs::set_permissions(&file, fs::Permissions::from_mode(0o700)).unwrap();
    }

    // Test input is a synthetic signature result, not an OS trust receipt.
    fn unconfirmed(_: &Path) -> SignatureObservation {
        SignatureObservation::Unconfirmed
    }

    #[test]
    fn ambiguous_application_observations_do_not_report_partial_or_zero_count() {
        assert_eq!(distinct_observation_count([]), Some(0));
        assert_eq!(distinct_observation_count([Some(10), Some(11)]), Some(2));
        assert_eq!(distinct_observation_count([Some(10), Some(10)]), None);
        assert_eq!(distinct_observation_count([Some(10), None]), None);
        assert_eq!(distinct_observation_count([None]), None);
        assert_eq!(distinct_observation_count([Some(0)]), None);
        assert_eq!(distinct_observation_count([Some(-1)]), None);
    }

    #[test]
    fn fixed_candidates_never_grant_admission() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().canonicalize().unwrap();
        assert_eq!(inspect_at(&root, unconfirmed).status, CandidateStatus::Missing);
        fixture(&root, CANDIDATES[0], EXPECTED_EXECUTABLE);
        let observed = inspect_at(&root, unconfirmed);
        assert_eq!(observed.status, CandidateStatus::Candidate);
        assert_eq!(observed.version.as_deref(), Some("1.26.33"));
        assert!(!observed.runtime_admission);
        assert_eq!(observed.observed_application_count, None);
        let wire = serde_json::to_value(observed).unwrap();
        assert_eq!(wire, serde_json::json!({"schema":1,"platform":"macos","status":"Candidate",
            "version":"1.26.33","signature":"Unconfirmed","publisher":"NotPinned",
            "multipleInstancesProhibited":null,"observedApplicationCount":null,"runtimeAdmission":false}));
        // An expected identity/team observation still cannot pin the official
        // distribution or authorize launch, even with the singleton key absent.
        let mut matched = inspect_at(&root, |_| SignatureObservation::ExpectedTeamMatched);
        // Synthetic discovery count is not a managed-session receipt.
        matched.observed_application_count = Some(5);
        assert_eq!(serde_json::to_value(&matched).unwrap()["observedApplicationCount"], 5);
        assert_eq!(matched.signature, SignatureObservation::ExpectedTeamMatched);
        assert_eq!(matched.publisher, "NotPinned");
        assert!(!matched.runtime_admission);
        assert_eq!(matched.multiple_instances_prohibited, None);
        fixture(&root, CANDIDATES[1], EXPECTED_EXECUTABLE);
        let ambiguous = inspect_at(&root, |_| panic!("must not inspect ambiguous bundles"));
        assert_eq!(ambiguous.status, CandidateStatus::Ambiguous);
        assert_eq!(ambiguous.signature, SignatureObservation::NotChecked);
    }

    #[test]
    fn intune_install_name_is_a_candidate_and_conflicting_names_refuse() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().canonicalize().unwrap();
        fixture(&root, CANDIDATES[2], EXPECTED_EXECUTABLE);
        let observed = inspect_at(&root, |_| SignatureObservation::ExpectedTeamMatched);
        assert_eq!(observed.status, CandidateStatus::Candidate);
        assert_eq!(observed.publisher, "NotPinned");
        assert!(!observed.runtime_admission);
        fixture(&root, CANDIDATES[0], EXPECTED_EXECUTABLE);
        let conflict = inspect_at(&root, |_| panic!("multiple names must not verify a signature"));
        assert_eq!(conflict.status, CandidateStatus::Ambiguous);
        assert_eq!(conflict.signature, SignatureObservation::NotChecked);
    }

    #[test]
    fn exact_metadata_identity_is_required_before_signature_observation() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().canonicalize().unwrap();
        fixture(&root, CANDIDATES[0], EXPECTED_EXECUTABLE);
        let info = root.join(CANDIDATES[0]).join("Contents/Info.plist");
        let original = fs::read_to_string(&info).unwrap();
        for changed in [
            original.replace(EXPECTED_BUNDLE_ID, "com.microsoft.other"),
            original.replace("<string>minecraft-edu</string>", "<string>other-executable</string>"),
            original.replace("<key>CFBundleVersion</key><string>1.26.33.0</string>", ""),
            original.replace("<string>1.26.33.0</string>", "<string>invalid</string>"),
        ] {
            fs::write(&info, changed).unwrap();
            assert_eq!(inspect_at(&root, |_| panic!("incorrect metadata must not verify")).status,
                CandidateStatus::Unconfirmed);
        }
    }

    #[test]
    fn explicit_singleton_flag_and_replaced_metadata_are_not_launch_authority() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().canonicalize().unwrap();
        fixture(&root, CANDIDATES[0], EXPECTED_EXECUTABLE);
        let info = root.join(CANDIDATES[0]).join("Contents/Info.plist");
        let original = fs::read_to_string(&info).unwrap();
        let with_flag = original.replace("</dict>", "<key>LSMultipleInstancesProhibited</key><true/></dict>");
        fs::write(&info, &with_flag).unwrap();
        let observed = inspect_at(&root, |_| SignatureObservation::ExpectedTeamMatched);
        assert_eq!(observed.multiple_instances_prohibited, Some(true));
        assert!(!observed.runtime_admission);
        let changed = inspect_at(&root, |_| {
            fs::write(&info, &original).unwrap();
            SignatureObservation::ExpectedTeamMatched
        });
        assert_eq!(changed.status, CandidateStatus::Unconfirmed);
        assert_eq!(changed.signature, SignatureObservation::NotChecked);
        let malformed = with_flag.replace("<true/>", "<string>true</string>");
        fs::write(&info, malformed).unwrap();
        assert_eq!(inspect_at(&root, |_| panic!("invalid plist")).status, CandidateStatus::Unconfirmed);
    }

    #[test]
    fn invalid_executable_and_symlink_are_unconfirmed_not_missing() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().canonicalize().unwrap();
        fixture(&root, CANDIDATES[0], "../fixture");
        assert_eq!(inspect_at(&root, unconfirmed).status, CandidateStatus::Unconfirmed);
        let other = tempfile::tempdir().unwrap();
        let root = other.path().canonicalize().unwrap();
        symlink(temporary.path(), root.join(CANDIDATES[0])).unwrap();
        assert_eq!(inspect_at(&root, unconfirmed).status, CandidateStatus::Unconfirmed);
    }
}
