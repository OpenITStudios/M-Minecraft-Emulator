//! Private binding metadata for the installed app; never profile/account copying.
use super::*;
use crate::engine::{
    native_clients::ProfileBinding, native_clients_session::{existing_controller_gate, CONTROLLER_GATE_NAME},
};
use std::io::{Read, Write};
use windows_sys::Win32::Storage::FileSystem::{
    FileDispositionInfo, MoveFileExW, SetFileInformationByHandle, DELETE, FILE_DISPOSITION_INFO,
    MOVEFILE_WRITE_THROUGH,
};

const NAME: &str = "profile-binding-five.bin";
const SIZE: usize = 288;

// Retain every ordinary ancestor against replacement until reading/publishing ends.
pub(super) fn binding_path(create: bool) -> io::Result<(PathBuf, Vec<File>)> {
    let mut path = known_folder(&FOLDERID_LocalAppData)?;
    let mut pins = Vec::new();
    for ancestor in path.ancestors().collect::<Vec<_>>().into_iter().rev() {
        pins.push(open_directory(
            ancestor,
            FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY,
        )?);
    }
    for part in ["MinecraftInstanceManager", "NativePreview"] {
        path.push(part);
        if create {
            match std::fs::create_dir(&path) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error),
            }
        }
        pins.push(open_directory(
            &path,
            FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY,
        )?);
    }
    Ok((path.join(NAME), pins))
}
fn read_binding(path: &Path) -> io::Result<Vec<u8>> {
    let mut file = pin_image(path)?;
    if file.metadata()?.len() != SIZE as u64 {
        return Err(invalid("Invalid five-profile binding size"));
    }
    let mut bytes = vec![0; SIZE];
    file.read_exact(&mut bytes)?;
    Ok(bytes)
}
fn validate(bytes: &[u8]) -> io::Result<()> {
    let expected =
        REVIEWED_FIVE_REPLAY.ok_or_else(|| invalid("No reviewed five-profile policy"))?;
    let (logical, parent) = replay_roots()?;
    drop(reopen_slots_for::<5>(logical, &parent, bytes, expected)?);
    Ok(())
}

// Recover the reviewed pilot from directory identities, never account/world files
// or a developer's launcher path. This is not provisioning for a different PC.
fn identity_bytes(parent: &Path, name: &str) -> io::Result<Vec<u8>> {
    let suffix = name
        .strip_prefix("MivubiEducationPilot_")
        .ok_or_else(|| invalid("Invalid profile root name"))?;
    if suffix.len() != 32 || !suffix.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(invalid("Invalid profile root name"));
    }
    let root = parent.join(name);
    let mut pins = Vec::new();
    for ancestor in root.ancestors().collect::<Vec<_>>().into_iter().rev() {
        pins.push(open_directory(ancestor, FILE_READ_ATTRIBUTES)?);
    }
    let mut bytes = vec![0u8; SIZE];
    bytes[..4].copy_from_slice(&0x52504c31u32.to_le_bytes());
    bytes[4..8].copy_from_slice(&2u32.to_le_bytes());
    bytes[8..12].copy_from_slice(&(SIZE as u32).to_le_bytes());
    for (index, unit) in name.encode_utf16().enumerate() {
        bytes[16 + index * 2..18 + index * 2].copy_from_slice(&unit.to_le_bytes());
    }
    let mut put = |file: &File, at: usize| -> io::Result<()> {
        let id = directory_id(file)?;
        bytes[at..at + 8].copy_from_slice(&id.VolumeSerialNumber.to_le_bytes());
        bytes[at + 8..at + 24].copy_from_slice(&id.FileId.Identifier);
        Ok(())
    };
    put(
        pins.last().ok_or_else(|| invalid("Missing profile root"))?,
        144,
    )?;
    for slot in 1..=5 {
        let private = open_directory(&root.join(format!("MC{slot:02}")), FILE_READ_ATTRIBUTES)?;
        put(&private, 168 + (slot - 1) * 24)?;
        pins.push(private);
    }
    Ok(bytes)
}

fn discover_reviewed(parent: &Path, expected: [u8; 32]) -> io::Result<Option<Vec<u8>>> {
    use sha2::{Digest, Sha256};
    let _parent = open_directory(parent, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY)?;
    for (index, entry) in std::fs::read_dir(parent)?.enumerate() {
        if index >= 1024 {
            return Err(invalid("Profile discovery exceeded its directory limit"));
        }
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if !name.starts_with("MivubiEducationPilot_") {
            continue;
        }
        // Unrelated, partial, replaced and linked roots can never satisfy the
        // original reviewed digest. Do not repair them or inspect their contents.
        let Ok(bytes) = identity_bytes(parent, name) else {
            continue;
        };
        if <[u8; 32]>::from(Sha256::digest(&bytes)) == expected {
            return Ok(Some(bytes));
        }
    }
    Ok(None)
}

// Only explicit migration opens reviewed historical profiles. Discovery never
// publishes a binding or provisions a missing profile.
pub(super) fn migration_sources() -> io::Result<[SlotStorage; 5]> {
    let expected = REVIEWED_FIVE_REPLAY.ok_or_else(|| invalid("No reviewed five-profile policy"))?;
    let (logical, parent) = replay_roots()?;
    let bytes = match binding_path(false).and_then(|(path, _pins)| read_binding(&path)) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound =>
            discover_reviewed(&parent, expected)?.ok_or_else(|| invalid("No reviewed five-profile migration source; original data was preserved"))?,
        Err(error) => return Err(error),
    };
    reopen_slots_for::<5>(logical, &parent, &bytes, expected)
}
fn path_present(path: &Path) -> io::Result<bool> {
    match std::fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}
// Fresh setup must not replace even an unreviewed/partial historical root.
fn pilot_present(parent: &Path) -> io::Result<bool> {
    let _pin = open_directory(parent, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY)?;
    for (index, entry) in std::fs::read_dir(parent)?.enumerate() {
        if index >= 1024 { return Err(invalid("Profile discovery exceeded its directory limit")); }
        if entry?.file_name().to_string_lossy().to_ascii_lowercase().starts_with("mivubieducationpilot_") {
            return Ok(true);
        }
    }
    Ok(false)
}
pub(super) fn has_legacy_source() -> io::Result<bool> {
    match binding_path(false) {
        Ok((path, _pins)) if path_present(&path)? => return Ok(true),
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    let exe = std::env::current_exe()?;
    let payload = exe.parent().ok_or_else(|| invalid("Missing replay payload root"))?.join("independent-bootstrap");
    for name in ["replay-profile-pair.bin", "replay-profile-five.bin"] {
        if path_present(&payload.join(name))? { return Ok(true); }
    }
    pilot_present(&known_folder(&FOLDERID_Profile)?)
}

// Create-once, same-byte idempotent publication. Never replace an existing binding.
fn publish(directory: &File, bytes: &[u8]) -> io::Result<()> {
    if bytes.len() != SIZE {
        return Err(invalid("Invalid binding size"));
    }
    let root = directory_path(directory)?;
    let target = root.join(NAME);
    match read_binding(&target) {
        Ok(current) if current == bytes => return Ok(()),
        Ok(_) => {
            return Err(invalid(
                "Existing profile binding differs; it was preserved",
            ))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    let temporary = root.join(format!(
        ".binding-{}-{}.tmp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_nanos()
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .access_mode(
            windows_sys::Win32::Storage::FileSystem::FILE_GENERIC_WRITE
                | FILE_READ_ATTRIBUTES
                | DELETE,
        )
        .share_mode(FILE_SHARE_READ | windows_sys::Win32::Storage::FileSystem::FILE_SHARE_DELETE)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .create_new(true)
        .open(&temporary)?;
    image_id(&file)?;
    let result = (|| {
        file.write_all(bytes)?;
        file.sync_all()?;
        let from = wide(temporary.as_os_str())?;
        let to = wide(target.as_os_str())?;
        // Without REPLACE_EXISTING: a concurrent binding always wins intact.
        if unsafe { MoveFileExW(from.as_ptr(), to.as_ptr(), MOVEFILE_WRITE_THROUGH) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    })();
    if result.is_err() {
        let info = FILE_DISPOSITION_INFO { DeleteFile: true };
        // Only the retained create-new temporary file, never the target/path.
        unsafe {
            SetFileInformationByHandle(
                file.as_raw_handle() as _,
                FileDispositionInfo,
                &info as *const _ as _,
                mem::size_of_val(&info) as u32,
            );
        }
    }
    result
}
fn ready() -> ProfileBinding {
    ProfileBinding { schema:1, required:true, configured:true, issue:None }
}
fn unavailable(issue: &str) -> ProfileBinding {
    ProfileBinding { schema:1, required:true, configured:false, issue:Some(issue.into()) }
}
fn migration_required() -> ProfileBinding {
    unavailable(super::product_profiles::MIGRATION_REQUIRED)
}
fn is_migration_required(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::Unsupported && error.to_string() == super::product_profiles::MIGRATION_REQUIRED
}
pub(in crate::engine::native_clients) fn inspect_existing() -> ProfileBinding {
    if !cfg!(feature = "lazy-tester-package") {
        return unavailable("Managed profiles require the Lazy Tester product build.");
    }
    match super::product_profiles::inspect_existing() {
        Ok(Some(_)) => ready(),
        Ok(None) => unavailable("No current profile catalog was found. Existing data was not changed."),
        Err(error) if is_migration_required(&error) => migration_required(),
        Err(_) => unavailable("The saved data folder could not be verified. Existing data was preserved."),
    }
}
pub(crate) fn run(import: bool) -> io::Result<ProfileBinding> {
    match super::product_profiles::open() {
        Ok(Some(_)) if import => return Err(invalid("Product profiles already exist; import cannot replace them")),
        Ok(Some(_)) => return Ok(ready()),
        Err(error) if is_migration_required(&error) => return Ok(migration_required()),
        Err(error) => return Err(error),
        Ok(None) => {}
    }
    if !import {
        if has_legacy_source()? { return Ok(migration_required()); }
        super::product_profiles::prepare()?;
        return Ok(ready());
    }
    // Explicit prior-launcher selection records only the migration source,
    // never a second active runtime binding.
    if existing_controller_gate(CONTROLLER_GATE_NAME)? {
        return Err(invalid("Close all Minecraft sessions and the previous controller before selecting a migration source"));
    }
    let Some(folder) = rfd::FileDialog::new()
        .set_title("Select the previous Minecraft launcher folder")
        .pick_folder()
    else {
        return Ok(inspect_existing());
    };
    let source = folder.join("independent-bootstrap/replay-profile-five.bin");
    let bytes = read_binding(&source)?;
    validate(&bytes)?;
    if existing_controller_gate(CONTROLLER_GATE_NAME)? {
        return Err(invalid("The controller started; source selection was cancelled without changing your data"));
    }
    let (_, pins) = binding_path(true)?;
    publish(pins.last().ok_or_else(|| invalid("Missing binding directory"))?, &bytes)?;
    Ok(migration_required())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn old_profile_presence_is_not_runtime_readiness_or_fresh_setup() {
        let report = migration_required();
        assert!(!report.configured);
        assert!(report.required);
        assert!(report.issue.unwrap().contains("--lazy-tester-migrate-data"));
        let fixture = Fixture::new();
        assert!(!pilot_present(&fixture.0).unwrap());
        let root = fresh_root(&fixture.0).unwrap();
        // Even incomplete two-slot data must block fresh setup.
        std::fs::create_dir(root.join("MC01")).unwrap();
        assert!(pilot_present(&fixture.0).unwrap());
        assert!(migration_sources().is_err()); // Empty reviewed test policy; no host lookup.
    }
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path = fresh_root(&std::env::temp_dir()).unwrap();
            Self(path)
        }
        fn pin(&self) -> File {
            open_directory(&self.0, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY).unwrap()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            assert_eq!(self.0.parent(), Some(std::env::temp_dir().as_path()));
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }
    #[test]
    fn discovery_uses_only_the_reviewed_directory_identities_and_preserves_files() {
        use sha2::{Digest, Sha256};
        let fixture = Fixture::new();
        let root = fresh_root(&fixture.0).unwrap();
        for slot in 1..=5 {
            std::fs::create_dir(root.join(format!("MC{slot:02}"))).unwrap();
        }
        let name = root.file_name().unwrap().to_str().unwrap();
        let bytes = identity_bytes(&fixture.0, name).unwrap();
        let expected = Sha256::digest(&bytes).into();
        let sentinel = root.join("MC01/account-world-fixture");
        std::fs::write(&sentinel, b"untouched").unwrap();
        assert_eq!(
            discover_reviewed(&fixture.0, expected).unwrap(),
            Some(bytes)
        );
        assert!(discover_reviewed(&fixture.0, [0; 32]).unwrap().is_none());
        assert!(identity_bytes(&fixture.0, "../outside").is_err());
        // Hold the old directory under a different name so its file ID cannot
        // be recycled for the replacement. Never operate on host profile data.
        std::fs::rename(root.join("MC05"), root.join("saved-MC05")).unwrap();
        std::fs::create_dir(root.join("MC05")).unwrap();
        assert!(discover_reviewed(&fixture.0, expected).unwrap().is_none());
        assert_eq!(std::fs::read(sentinel).unwrap(), b"untouched");
        assert!(std::fs::read_dir(&fixture.0).unwrap().count() == 1);
    }
    #[test]
    fn publication_is_complete_idempotent_and_never_replaces_other_metadata() {
        let fixture = Fixture::new();
        let pin = fixture.pin();
        let bytes = vec![17; SIZE];
        publish(&pin, &bytes).unwrap();
        assert_eq!(read_binding(&fixture.0.join(NAME)).unwrap(), bytes);
        publish(&pin, &bytes).unwrap();
        assert!(publish(&pin, &vec![23; SIZE]).is_err());
        assert_eq!(read_binding(&fixture.0.join(NAME)).unwrap(), bytes);
        assert_eq!(std::fs::read_dir(&fixture.0).unwrap().count(), 1);
    }
    #[test]
    fn invalid_source_sizes_and_existing_invalid_metadata_do_not_publish() {
        let fixture = Fixture::new();
        let pin = fixture.pin();
        assert!(publish(&pin, &[0; 287]).is_err());
        std::fs::write(fixture.0.join(NAME), b"invalid").unwrap();
        assert!(read_binding(&fixture.0.join(NAME)).is_err());
        assert!(publish(&pin, &[0; SIZE]).is_err());
        assert_eq!(std::fs::read(fixture.0.join(NAME)).unwrap(), b"invalid");
        assert!(validate(&[0; SIZE]).is_err()); // Test policy cannot authorize host profiles.
    }
    #[test]
    fn inspecting_existing_temporary_storage_never_creates_a_missing_directory() {
        let fixture = Fixture::new();
        std::fs::create_dir(fixture.0.join("Local")).unwrap();
        let pin = fixture.pin();
        assert!(open_private_temporary(&pin, PrivateTemporaryMode::ExistingOnly).is_err());
        assert!(!fixture.0.join("Local/Temp").exists());
        drop(open_private_temporary(&pin, PrivateTemporaryMode::CreateIfMissing).unwrap()); // Explicit preparation is separate.
        assert!(open_private_temporary(&pin, PrivateTemporaryMode::ExistingOnly).is_ok());
    }
}
