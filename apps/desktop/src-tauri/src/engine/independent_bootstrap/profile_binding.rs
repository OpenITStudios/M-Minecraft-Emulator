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
fn status() -> ProfileBinding {
    let result = (|| {
        let (path, _pins) = binding_path(false)?;
        validate(&read_binding(&path)?)
    })();
    let (configured, issue) = match result {
        Ok(()) => (true, None),
        Err(error) if error.kind() == io::ErrorKind::NotFound =>
            (false, Some("Pilih folder launcher lama yang memuat lima profil Minecraft. Akun dan dunia tidak dipindahkan.".into())),
        Err(_) => (false, Some("Pengikat profil tidak cocok atau profil asli tidak tersedia. Data lama tetap dipertahankan.".into())),
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
        return Ok(status());
    }
    if existing_controller_gate("Local\\MivubiEducationStartupObservation")? {
        return Err(invalid(
            "Tutup semua Minecraft dan pengendali lama secara normal sebelum menghubungkan profil",
        ));
    }
    let Some(folder) = rfd::FileDialog::new()
        .set_title("Pilih folder launcher Minecraft lama")
        .pick_folder()
    else {
        return Ok(status());
    };
    let source = folder.join("independent-bootstrap/replay-profile-five.bin");
    let bytes = read_binding(&source)?;
    validate(&bytes)?; // Existing reviewed IDs and existing HKCU stores; no fresh data.
    if existing_controller_gate("Local\\MivubiEducationStartupObservation")? {
        return Err(invalid(
            "Pengendali mulai berjalan; pengaitan dibatalkan tanpa mengubah profil",
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
