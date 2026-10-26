//! Private binding metadata for the installed app; never profile/account copying.
use super::*;
use crate::engine::{
    native_clients::ProfileBinding, native_clients_session::existing_controller_gate,
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

fn connect_reviewed_existing() -> io::Result<ProfileBinding> {
    let existing = binding_path(false).and_then(|(path, _pins)| read_binding(&path));
    match existing {
        Ok(_) => return Ok(status()), // Invalid existing bindings are preserved.
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    if existing_controller_gate("Local\\MivubiEducationStartupObservation")? {
        return Ok(status());
    }
    let expected =
        REVIEWED_FIVE_REPLAY.ok_or_else(|| invalid("No reviewed five-profile policy"))?;
    let (_, parent) = replay_roots()?;
    let Some(bytes) = discover_reviewed(&parent, expected)? else {
        return Ok(status());
    };
    validate(&bytes)?; // Includes existing Roaming/Local/TEMP and HKCU validation.
    if existing_controller_gate("Local\\MivubiEducationStartupObservation")? {
        return Ok(status());
    }
    let (_, pins) = binding_path(true)?;
    publish(
        pins.last()
            .ok_or_else(|| invalid("Missing binding directory"))?,
        &bytes,
    )?;
    Ok(status())
}

pub(super) fn has_legacy_binding() -> io::Result<bool> {
    match binding_path(false) {
        Ok((path, _pins)) => path.try_exists(),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
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
pub(super) fn inspect_existing() -> ProfileBinding {
    match super::product_profiles::inspect_existing() {
        Ok(Some(_)) => ProfileBinding { schema:1,required:true,configured:true,issue:None },
        Ok(None) => status(),
        Err(error) => ProfileBinding {schema:1,required:true,configured:false,issue:Some(format!("The saved data folder could not be verified: {error}. Existing data was preserved."))},
    }
}
fn status() -> ProfileBinding {
    let metadata = (|| {
        let (path, _pins) = binding_path(false)?;
        read_binding(&path)
    })();
    binding_status(metadata, validate)
}
fn binding_status(metadata: io::Result<Vec<u8>>, validate: impl FnOnce(&[u8])->io::Result<()>) -> ProfileBinding {
    let (configured, issue) = match metadata {
        Ok(bytes) => match validate(&bytes) {
            Ok(()) => (true,None),
            Err(error) => (false,Some(format!("Saved launcher data could not be reopened: {error}. Existing data was preserved. Use Locate existing data or reconnect the previous launcher."))),
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound =>
            (false, Some("No saved launcher connection was found. Set up a data folder or connect existing launcher data.".into())),
        Err(error) => (false, Some(format!("The saved launcher connection could not be read: {error}. Your data has been preserved."))),
    };
    ProfileBinding {
        schema: 1,
        required: true,
        configured,
        issue,
    }
}
pub(crate) fn run(import: bool) -> io::Result<ProfileBinding> {
    if !import {
        if super::product_profiles::open()?.is_some() {
            return Ok(ProfileBinding {
                schema: 1,
                required: true,
                configured: true,
                issue: None,
            });
        }
        let legacy = connect_reviewed_existing()?;
        if legacy.configured || has_legacy_binding()? {
            return Ok(legacy);
        }
        super::product_profiles::prepare()?;
        return Ok(ProfileBinding {
            schema: 1,
            required: true,
            configured: true,
            issue: None,
        });
    }
    if super::product_profiles::open()?.is_some() {
        return Err(invalid(
            "Product profiles already exist; previous-launcher import cannot replace them",
        ));
    }
    if existing_controller_gate("Local\\MivubiEducationStartupObservation")? {
        return Err(invalid(
            "Close all Minecraft sessions and the previous controller normally before connecting profiles",
        ));
    }
    let Some(folder) = rfd::FileDialog::new()
        .set_title("Select the previous Minecraft launcher folder")
        .pick_folder()
    else {
        return Ok(status());
    };
    let source = folder.join("independent-bootstrap/replay-profile-five.bin");
    let bytes = read_binding(&source)?;
    validate(&bytes)?; // Existing reviewed IDs and existing HKCU stores; no fresh data.
    if existing_controller_gate("Local\\MivubiEducationStartupObservation")? {
        return Err(invalid(
            "The controller started; profile connection was cancelled without changing your data",
        ));
    }
    let (_, pins) = binding_path(true)?;
    publish(
        pins.last()
            .ok_or_else(|| invalid("Missing binding directory"))?,
        &bytes,
    )?;
    Ok(status())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_connection_and_missing_saved_data_have_distinct_recovery_messages() {
        let missing=binding_status(Err(io::Error::from(io::ErrorKind::NotFound)),|_|panic!("missing metadata cannot validate"));
        assert!(!missing.configured);assert!(missing.issue.unwrap().contains("No saved launcher connection"));
        let stale=binding_status(Ok(vec![0;SIZE]),|_|Err(io::Error::new(io::ErrorKind::NotFound,"Minecraft - 2: temporary folder is unavailable")));
        let issue=stale.issue.unwrap();assert!(!stale.configured);assert!(issue.contains("Minecraft - 2"));assert!(issue.contains("Existing data was preserved"));assert!(!issue.contains("new PC"));
        let valid=binding_status(Ok(vec![0;SIZE]),|_|Ok(()));assert!(valid.configured);assert!(valid.issue.is_none());
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
        assert!(existing_private_temporary(&pin).is_err());
        assert!(!fixture.0.join("Local/Temp").exists());
        drop(private_temporary(&pin).unwrap()); // Explicit preparation is separate.
        assert!(existing_private_temporary(&pin).is_ok());
    }
}
