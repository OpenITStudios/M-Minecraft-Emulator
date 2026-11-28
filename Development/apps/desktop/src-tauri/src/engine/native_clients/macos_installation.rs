//! Explicit Mac installation candidate inspection, never game admission.
//! Only fixed Applications paths; no account/profile data, game execution,
//! signature claim, updater operation, or arbitrary caller-supplied path.
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, Metadata, OpenOptions},
    io::{self, Cursor, Read},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::Path,
};

const CANDIDATES: [&str; 2] = ["Minecraft Education.app", "minecraft-edu.app"];
const MAX_PLIST: u64 = 256 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
enum CandidateStatus {
    Missing,
    Candidate,
    Ambiguous,
    Unconfirmed,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MacEducationInstallationReport {
    schema: u32,
    platform: &'static str,
    status: CandidateStatus,
    version: Option<String>,
    runtime_admission: bool,
}

fn report(status: CandidateStatus, version: Option<String>) -> MacEducationInstallationReport {
    MacEducationInstallationReport {
        schema: 1,
        platform: "macos",
        status,
        version,
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

fn valid_executable(value: &str) -> bool {
    !value.is_empty() && value.len() <= 128
        && value.as_bytes()[0].is_ascii_alphanumeric()
        && value.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-' | b' '))
}

fn parse_metadata(bytes: &[u8]) -> io::Result<BundleMetadata> {
    let metadata: BundleMetadata = plist::from_reader(Cursor::new(bytes)).map_err(|_| invalid())?;
    if metadata.package_type != "APPL"
        || metadata.identifier.is_empty() || metadata.identifier.len() > 255
        || !metadata.identifier.as_bytes()[0].is_ascii_alphanumeric()
        || !metadata.identifier.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-'))
        || !valid_executable(&metadata.executable) || !valid_version(&metadata.version)
    {
        return Err(invalid());
    }
    Ok(metadata)
}

fn inspect_bundle(path: &Path) -> io::Result<String> {
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
    if !same_metadata(&bundle_before, &ordinary_directory(path)?)
        || !same_metadata(&contents_before, &ordinary_directory(&contents)?)
        || !same_metadata(&macos_before, &ordinary_directory(&macos)?)
        || !same_metadata(&before, &ordinary_file(&info)?)
        || !same_metadata(&image, &ordinary_file(&executable)?)
    {
        return Err(invalid());
    }
    // Metadata may be imitated. Publisher and loaded image are unverified.
    Ok(metadata.version)
}

fn inspect_at(applications: &Path) -> MacEducationInstallationReport {
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
        [path] => match inspect_bundle(path) {
            Ok(version) => report(CandidateStatus::Candidate, Some(version)),
            Err(_) => report(CandidateStatus::Unconfirmed, None),
        },
        _ => report(CandidateStatus::Ambiguous, None),
    }
}

pub(super) fn inspect() -> MacEducationInstallationReport {
    inspect_at(Path::new("/Applications"))
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
<key>CFBundleIdentifier</key><string>com.example.fixture</string>
<key>CFBundleExecutable</key><string>{exe}</string>
<key>CFBundleShortVersionString</key><string>26.33</string>
</dict></plist>"#);
        fs::write(contents.join("Info.plist"), xml).unwrap();
        let file = contents.join("MacOS/fixture");
        fs::write(&file, b"synthetic metadata only").unwrap();
        fs::set_permissions(&file, fs::Permissions::from_mode(0o700)).unwrap();
    }

    #[test]
    fn fixed_candidates_never_grant_admission() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().canonicalize().unwrap();
        assert_eq!(inspect_at(&root).status, CandidateStatus::Missing);
        fixture(&root, CANDIDATES[0], "fixture");
        let observed = inspect_at(&root);
        assert_eq!(observed.status, CandidateStatus::Candidate);
        assert_eq!(observed.version.as_deref(), Some("26.33"));
        assert!(!observed.runtime_admission);
        let wire = serde_json::to_value(observed).unwrap();
        assert_eq!(wire, serde_json::json!({"schema":1,"platform":"macos","status":"Candidate",
            "version":"26.33","runtimeAdmission":false}));
        fixture(&root, CANDIDATES[1], "fixture");
        assert_eq!(inspect_at(&root).status, CandidateStatus::Ambiguous);
    }

    #[test]
    fn invalid_executable_and_symlink_are_unconfirmed_not_missing() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().canonicalize().unwrap();
        fixture(&root, CANDIDATES[0], "../fixture");
        assert_eq!(inspect_at(&root).status, CandidateStatus::Unconfirmed);
        let other = tempfile::tempdir().unwrap();
        let root = other.path().canonicalize().unwrap();
        symlink(temporary.path(), root.join(CANDIDATES[0])).unwrap();
        assert_eq!(inspect_at(&root).status, CandidateStatus::Unconfirmed);
    }
}
