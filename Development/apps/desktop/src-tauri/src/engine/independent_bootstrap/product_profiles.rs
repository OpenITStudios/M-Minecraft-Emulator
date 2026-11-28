//! Product data locations, file-backed private registry and fixed opt-in legacy migration.
use super::*;
use crate::engine::native_clients::{DataLocation, DataLocationAction, ProfilePath};
use crate::engine::native_clients_session::existing_controller_gate;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use windows_sys::Win32::Storage::FileSystem::{
    MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
};

const CATALOG: &str = "profiles.json";
const LOCATOR: &str = "data-location.json";
const LIMIT: u64 = 16 * 1024;
pub(super) const MIGRATION_REQUIRED: &str = "Existing launcher data requires explicit migration. Close Lazy Tester and use --lazy-tester-migrate-data. Original data is preserved.";

#[path = "product_profiles/clear.rs"]
pub(super) mod clear;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Identity {
    volume: u64,
    file: [u8; 16],
}
impl Identity {
    fn read(file: &File) -> io::Result<Self> {
        let id = directory_id(file)?;
        Ok(Self {
            volume: id.VolumeSerialNumber,
            file: id.FileId.Identifier,
        })
    }
    fn matches(&self, file: &File) -> io::Result<bool> {
        let actual = Self::read(file)?;
        Ok(self.volume == actual.volume && self.file == actual.file)
    }
    fn matches_image(&self, file: &File) -> io::Result<bool> {
        let id = image_id(file)?;
        Ok(self.volume == id.VolumeSerialNumber && self.file == id.FileId.Identifier)
    }
}

const HIVE_IDENTITY: &str = "LazyTesterProfileIdentity";
fn hive_identity(private: &File, slot: u32) -> io::Result<[u8; 28]> {
    let id = directory_id(private)?;
    let mut bytes = [0; 28];
    bytes[..8].copy_from_slice(&id.VolumeSerialNumber.to_le_bytes());
    bytes[8..24].copy_from_slice(&id.FileId.Identifier);
    bytes[24..].copy_from_slice(&slot.to_le_bytes());
    Ok(bytes)
}
fn pin_hive(path: &Path) -> io::Result<File> {
    let file = OpenOptions::new()
        .access_mode(FILE_READ_ATTRIBUTES)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    image_id(&file)?;
    Ok(file)
}
pub(super) fn open_hive(private: &File, slot: u32, fresh: bool) -> io::Result<RegistryKey> {
    use windows_sys::Win32::System::Registry::{
        RegLoadAppKeyW, RegQueryValueExW, RegSetValueExW, REG_BINARY,
    };
    let directory = directory_path(private)?.join("Registry");
    if fresh {
        std::fs::create_dir(&directory)?;
    }
    let _pin = open_directory(&directory, FILE_READ_ATTRIBUTES)?;
    let path = directory.join("HKCU.dat");
    let before = if fresh {
        if path.try_exists()? {
            return Err(invalid("Registry file already exists"));
        }
        None
    } else {
        Some(pin_hive(&path)?)
    };
    let mut raw = ptr::null_mut();
    let status = unsafe {
        RegLoadAppKeyW(
            wide(path.as_os_str())?.as_ptr(),
            &mut raw,
            KEY_ALL_ACCESS,
            0,
            0,
        )
    };
    if status != 0 {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Minecraft - {slot}: registry file cannot be opened (Windows error {status})"),
        ));
    }
    let key = RegistryKey(raw);
    let after = pin_hive(&path).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("Loaded registry file metadata unavailable: {e}"),
        )
    })?;
    if let Some(before) = before {
        if !same_image(&image_id(&before)?, &image_id(&after)?) {
            return Err(invalid("Registry file changed during opening"));
        }
    }
    let expected = hive_identity(private, slot)?;
    let label = wide(OsStr::new(HIVE_IDENTITY))?;
    if fresh {
        let status = unsafe {
            RegSetValueExW(
                key.0,
                label.as_ptr(),
                0,
                REG_BINARY,
                expected.as_ptr(),
                expected.len() as u32,
            )
        };
        if status != 0 {
            return Err(io::Error::from_raw_os_error(status as i32));
        }
    } else {
        let mut actual = [0u8; 28];
        let mut size = actual.len() as u32;
        let mut kind = 0;
        let status = unsafe {
            RegQueryValueExW(
                key.0,
                label.as_ptr(),
                ptr::null(),
                &mut kind,
                actual.as_mut_ptr(),
                &mut size,
            )
        };
        if status != 0 || kind != REG_BINARY || size != 28 || actual != expected {
            return Err(invalid("Registry file does not belong to this launcher"));
        }
    }
    Ok(key)
}
fn protect_data(root: &Path) -> io::Result<()> {
    use windows_sys::Win32::Security::{
        Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW, SetFileSecurityW,
        DACL_SECURITY_INFORMATION, PROTECTED_DACL_SECURITY_INFORMATION,
    };
    let user = user_registry_path()?;
    let end = user
        .iter()
        .position(|c| *c == 0)
        .ok_or_else(|| invalid("Invalid user identity"))?;
    let native = String::from_utf16(&user[..end]).map_err(|_| invalid("Invalid user identity"))?;
    let sid = native
        .strip_prefix("\\REGISTRY\\USER\\")
        .filter(|sid| {
            sid.starts_with("S-1-")
                && sid
                    .bytes()
                    .all(|b| b.is_ascii_digit() || b == b'S' || b == b'-')
        })
        .ok_or_else(|| invalid("Unsupported Windows user identity"))?;
    let sddl = wide(OsStr::new(&format!(
        "D:P(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)(A;OICI;FA;;;{sid})"
    )))?;
    let mut descriptor = ptr::null_mut();
    if unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            1,
            &mut descriptor,
            ptr::null_mut(),
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let result = unsafe {
        SetFileSecurityW(
            wide(root.as_os_str())?.as_ptr(),
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            descriptor,
        )
    };
    let error = if result == 0 {
        Some(io::Error::last_os_error())
    } else {
        None
    };
    unsafe { windows_sys::Win32::Foundation::LocalFree(descriptor) };
    error.map_or(Ok(()), Err)
}
fn create_file_slot(logical: &File, path: &Path, slot: u32) -> io::Result<SlotStorage> {
    std::fs::create_dir(path)?;
    let private = open_directory(path, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY)?;
    for name in ["Roaming", "Local"] {
        std::fs::create_dir(path.join(name))?;
    }
    let registry = open_hive(&private, slot, true)?;
    Ok(SlotStorage {
        slot,
        logical: logical.try_clone()?,
        temporary: open_private_temporary(&private, PrivateTemporaryMode::CreateIfMissing)?,
        private,
        registry,
        user_key_path: user_registry_path()?,
        private_path: path.to_path_buf(),
        singleton_caller_rva: 0,
    })
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Catalog {
    schema: u32,
    owner: [u8; 32],
    root: Identity,
    profiles: [Identity; 5],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    hives: Option<[Identity; 5]>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Locator {
    schema: u32,
    path: String,
}

fn owner() -> io::Result<[u8; 32]> {
    let path = user_registry_path()?;
    let bytes: Vec<u8> = path.into_iter().flat_map(u16::to_le_bytes).collect();
    Ok(Sha256::digest(bytes).into())
}
fn layout(executable: &Path) -> io::Result<PathBuf> {
    crate::engine::manager_settings::application_root(executable)
}
fn home() -> io::Result<PathBuf> {
    layout(&std::env::current_exe()?)
}
fn locator_path(home: &Path) -> PathBuf {
    home.join("Data/Settings").join(LOCATOR)
}
fn pin_chain(path: &Path) -> io::Result<Vec<File>> {
    if !path.is_absolute()
        || path.components().any(|part| {
            matches!(
                part,
                std::path::Component::ParentDir | std::path::Component::CurDir
            )
        })
    {
        return Err(invalid(
            "An absolute data location without traversal is required",
        ));
    }
    path.ancestors()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(|part| open_directory(part, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY))
        .collect()
}
fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> io::Result<T> {
    let mut file = pin_image(path)?;
    if file.metadata()?.len() > LIMIT {
        return Err(invalid("Data metadata is too large"));
    }
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(LIMIT + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > LIMIT {
        return Err(invalid("Data metadata is too large"));
    }
    serde_json::from_slice(&bytes)
        .map_err(|_| invalid("Data metadata is invalid; existing files were preserved"))
}
fn write_json<T: Serialize>(path: &Path, value: &T, replace: bool) -> io::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| invalid("Metadata parent unavailable"))?;
    let _pins = pin_chain(parent)?;
    let bytes = serde_json::to_vec_pretty(value).map_err(io::Error::other)?;
    if bytes.len() as u64 > LIMIT {
        return Err(invalid("Data metadata is too large"));
    }
    let mut guid: windows_sys::core::GUID = unsafe { mem::zeroed() };
    if unsafe { CoCreateGuid(&mut guid) } < 0 {
        return Err(invalid("Cannot allocate metadata transaction"));
    }
    let temp = parent.join(format!(
        ".location-{:08x}{:04x}{:04x}-{}.tmp",
        guid.data1,
        guid.data2,
        guid.data3,
        std::process::id()
    ));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .share_mode(0)
        .open(&temp)?;
    let result = file.write_all(&bytes).and_then(|()| file.sync_all());
    drop(file);
    let result = result.and_then(|()| {
        let flags = MOVEFILE_WRITE_THROUGH
            | if replace {
                MOVEFILE_REPLACE_EXISTING
            } else {
                0
            };
        if unsafe {
            MoveFileExW(
                wide(temp.as_os_str())?.as_ptr(),
                wide(path.as_os_str())?.as_ptr(),
                flags,
            )
        } == 0
        {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    });
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    } // Only our create-new temporary.
    result
}
fn resolve_location(home: &Path, locator: &Path) -> io::Result<(PathBuf, bool)> {
    // Missing is distinct from an existing invalid/dangling pointer.
    let metadata = match std::fs::symlink_metadata(locator) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound =>
            return Ok((home.join("Data"), true)),
        Err(error) => return Err(error),
    };
    if !metadata.is_file() || std::os::windows::fs::MetadataExt::file_attributes(&metadata)
        & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(invalid("Saved data location is not an ordinary file"));
    }
    let _parents = pin_chain(locator.parent().ok_or_else(|| invalid("Metadata parent unavailable"))?)?;
    match read_json::<Locator>(locator) {
        Ok(value) if value.schema == 1 && value.path == "Data" => Ok((home.join("Data"), false)),
        Ok(value) if value.schema == 1 && Path::new(&value.path).is_absolute() => {
            let path = PathBuf::from(value.path);
            if !path.starts_with(home.join("Data")) {
                return Err(invalid(
                    "Launcher data must stay inside this project's Data folder",
                ));
            }
            let _pins = pin_chain(&path)?;
            Ok((path, false))
        }
        Ok(_) => Err(invalid("Saved data location is invalid; it was preserved")),
        Err(error) => Err(error), // A selected pointer that disappears must not become fresh setup.
    }
}
fn disjoint(logical: &File, data: &Path) -> io::Result<()> {
    let logical = directory_path(logical)?
        .to_string_lossy()
        .to_ascii_lowercase();
    let data = data.to_string_lossy().to_ascii_lowercase();
    if Path::new(&logical).starts_with(&data) || Path::new(&data).starts_with(&logical) {
        return Err(invalid("Data destination overlaps a protected source directory"));
    }
    Ok(())
}
fn open_at(logical: File, root: &Path) -> io::Result<[SlotStorage; 5]> {
    open_catalog(logical, root, 2)
}
// Schema 1 is admitted only by the explicit migration route.
fn open_catalog(logical: File, root: &Path, expected_schema: u32) -> io::Result<[SlotStorage; 5]> {
    let pins = pin_chain(root)?;
    let catalog: Catalog = read_json(&root.join(CATALOG))?;
    if !matches!(catalog.schema, 1 | 2)
        || (catalog.schema == 2) != catalog.hives.is_some()
        || catalog.owner != owner()?
        || !catalog.root.matches(pins.last().unwrap())?
    {
        return Err(invalid(
            "This data folder does not match the saved Windows user and directory identity",
        ));
    }
    if catalog.schema != expected_schema {
        return if expected_schema == 2 && catalog.schema == 1 {
            Err(io::Error::new(io::ErrorKind::Unsupported, MIGRATION_REQUIRED))
        } else {
            Err(invalid("Unexpected migration source schema; existing data was preserved"))
        };
    }
    disjoint(&logical, &directory_path(pins.last().unwrap())?)?;
    let profiles = if catalog.schema == 2 {
        "Launchers"
    } else {
        "Profiles"
    };
    let _profiles = open_directory(
        &root.join(profiles),
        FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY,
    )?;
    let user_key_path = user_registry_path()?;
    let mut stores = Vec::new();
    for (index, identity) in catalog.profiles.iter().enumerate() {
        let slot = index as u32 + 1;
        let private = open_directory(
            &root.join(format!("{profiles}/Minecraft-{slot}")),
            FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY,
        )?;
        if !identity.matches(&private)? {
            return Err(invalid(
                "A profile directory was replaced; existing data was preserved",
            ));
        }
        let private_path = directory_path(&private)?;
        let _roaming = open_directory(&private_path.join("Roaming"), FILE_READ_ATTRIBUTES)?;
        let registry = if let Some(hives) = &catalog.hives {
            let hive = pin_hive(&private_path.join("Registry/HKCU.dat"))?;
            if !hives[index].matches_image(&hive)? {
                return Err(invalid(
                    "A registry file was replaced; existing data was preserved",
                ));
            }
            open_hive(&private, slot, false)?
        } else {
            open_existing_registry_store(&directory_id(&private)?, slot)?
        };
        stores.push(SlotStorage {
            slot,
            logical: logical.try_clone()?,
            temporary: open_private_temporary(&private, PrivateTemporaryMode::ExistingOnly)?,
            private,
            registry,
            user_key_path,
            private_path,
            singleton_caller_rva: 0,
        });
    }
    stores
        .try_into()
        .map_err(|_| invalid("Missing product profile"))
}
pub(super) fn open() -> io::Result<Option<[SlotStorage; 5]>> {
    Ok(open_mode(&home()?, true)?.map(|(_, stores)| stores))
}
pub(super) fn inspect_existing() -> io::Result<Option<[SlotStorage; 5]>> {
    Ok(open_mode(&home()?, false)?.map(|(_, stores)| stores))
}
fn recover_confirmed(root:&Path) {
    // Startup restores unfinished cutovers; large committed purges require Finish clear.
    // Inspection/Locate stay read-only and unrelated catalog-valid peers remain available.
    for slot in 1..=5 {let _=clear::rollback_uncommitted(root,slot);}
}
// Canonical locator wins; an existing legacy locator is never fresh setup.
fn resolve_saved_location(home: &Path) -> io::Result<(PathBuf, bool, bool)> {
    let (root, missing) = resolve_location(home, &locator_path(home))?;
    if !missing { return Ok((root, false, false)); }
    let (root, missing) = resolve_location(home, &home.join("App").join(LOCATOR))?;
    Ok((root, missing, !missing))
}
fn open_mode(home: &Path, persist_location: bool) -> io::Result<Option<(PathBuf, [SlotStorage; 5])>> {
    let (root, default, legacy_locator) = resolve_saved_location(home)?;
    let logical = logical_appdata_root()?;
    if persist_location {recover_confirmed(&root);}
    match open_at(logical, &root) {
        Ok(stores) => {
            // A successful setup may have stopped between catalog and locator
            // publication. Persist the pointer before admitting any game launch.
            if (default || legacy_locator) && persist_location {
                let settings = home.join("Data/Settings");
                std::fs::create_dir_all(&settings)?;
                write_json(
                    &locator_path(home),
                    &Locator {
                        schema: 1,
                        path: if root == home.join("Data") {
                            "Data".into()
                        } else {
                            root.to_string_lossy().into_owned()
                        },
                    },
                    false,
                )?;
            }
            Ok(Some((root, stores)))
        }
        Err(error)
            if default && error.kind() == io::ErrorKind::NotFound && empty_default(&root)? =>
        {
            Ok(None)
        }
        Err(error) => Err(error), // Missing mounted data or a partial setup never becomes fresh profiles.
    }
}
fn empty_default(root: &Path) -> io::Result<bool> {
    if !root.try_exists()? {
        return Ok(true);
    }
    let _pins = pin_chain(root)?;
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        if entry.file_name() != "Settings" && entry.file_name() != "Cache" {
            return Ok(false);
        }
        drop(open_directory(&entry.path(), FILE_READ_ATTRIBUTES)?);
    }
    Ok(true)
}
fn initialize_at(logical: &File, root: &Path) -> io::Result<()> {
    let parent = root
        .parent()
        .ok_or_else(|| invalid("Data parent unavailable"))?;
    let _parent = pin_chain(parent)?;
    disjoint(logical, root)?;
    match std::fs::create_dir(root) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            let _pin = pin_chain(root)?;
            if !empty_default(root)? {
                return Err(invalid("Data folder is not empty. Locate existing data instead of creating new profiles."));
            }
        }
        Err(error) => return Err(error),
    }
    let _pins = pin_chain(root)?;
    protect_data(root).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("Data privacy permissions unavailable: {e}"),
        )
    })?;
    std::fs::create_dir(root.join("Launchers"))?;
    let _profiles = open_directory(
        &root.join("Launchers"),
        FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY,
    )?;
    let mut stores = Vec::new();
    for slot in 1..=5 {
        stores.push(
            create_file_slot(
                logical,
                &root.join(format!("Launchers/Minecraft-{slot}")),
                slot,
            )
            .map_err(|e| {
                io::Error::new(
                    e.kind(),
                    format!("Minecraft - {slot}: storage creation failed: {e}"),
                )
            })?,
        );
    }
    publish_catalog(root, &stores)
}
fn publish_catalog(root: &Path, stores: &[SlotStorage]) -> io::Result<()> {
    let pins = pin_chain(root)?;
    let ids: Vec<Identity> = stores
        .iter()
        .map(|store| Identity::read(&store.private))
        .collect::<io::Result<_>>()?;
    let catalog = Catalog {
        schema: 2,
        owner: owner()?,
        root: Identity::read(pins.last().unwrap())?,
        profiles: ids
            .try_into()
            .map_err(|_| invalid("Missing product profile"))?,
        hives: Some(
            stores
                .iter()
                .map(|store| {
                    let file = pin_hive(&store.private_path.join("Registry/HKCU.dat"))?;
                    let id = image_id(&file)?;
                    Ok(Identity {
                        volume: id.VolumeSerialNumber,
                        file: id.FileId.Identifier,
                    })
                })
                .collect::<io::Result<Vec<_>>>()?
                .try_into()
                .map_err(|_| invalid("Missing registry file"))?,
        ),
    };
    write_json(&root.join(CATALOG), &catalog, false)?; // Complete identity record last; never overwrite partial data.
    Ok(())
}

// CopyFileW/SHCopyKeyW copy the same launcher's existing storage opaquely;
// account values and world contents never enter diagnostics or a second profile.
fn portable_cache_junction(base: &Path, path: &Path) -> io::Result<bool> {
    if !base
        .file_name()
        .is_some_and(|name| name.eq_ignore_ascii_case("Local"))
        || !path.strip_prefix(base).ok().is_some_and(|relative| {
            relative
                .to_string_lossy()
                .replace('\\', "/")
                .eq_ignore_ascii_case("Microsoft/Windows/INetCache/Content.IE5")
        })
    {
        return Ok(false);
    }
    let file = OpenOptions::new()
        .access_mode(FILE_READ_ATTRIBUTES)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
        .open(path)?;
    let tag = crate::engine::windows_file_identity::attribute_tag(&file)?;
    Ok(tag.ReparseTag == 0xa0000003 && tag.FileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0)
}
fn check_copy_tree(path: &Path) -> io::Result<()> {
    check_copy_tree_at(path, path)
}
fn check_copy_tree_at(base: &Path, path: &Path) -> io::Result<()> {
    use std::os::windows::fs::MetadataExt;
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let metadata = std::fs::symlink_metadata(entry.path())?;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            if portable_cache_junction(base, &entry.path())? {
                continue;
            }
            return Err(invalid(
                "Existing launcher data contains a link; migration was not started",
            ));
        }
        if metadata.is_dir() {
            check_copy_tree_at(base, &entry.path())?;
        } else if !metadata.is_file() {
            return Err(invalid("Unsupported launcher data entry"));
        }
    }
    Ok(())
}
fn copy_tree(source: &Path, destination: &Path) -> io::Result<()> {
    copy_tree_at(source, source, destination)
}
fn copy_tree_at(base: &Path, source: &Path, destination: &Path) -> io::Result<()> {
    use windows_sys::Win32::Storage::FileSystem::CopyFileW;
    let _source = pin_chain(source)?;
    let _destination = pin_chain(destination)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        let metadata = std::fs::symlink_metadata(entry.path())?;
        if std::os::windows::fs::MetadataExt::file_attributes(&metadata)
            & FILE_ATTRIBUTE_REPARSE_POINT
            != 0
        {
            if portable_cache_junction(base, &entry.path())? {
                // Do not follow/copy Windows' external WinINet cache target.
                std::fs::create_dir(&target)?;
                continue;
            }
            return Err(invalid("Launcher data changed during migration"));
        }
        if metadata.is_dir() {
            match std::fs::create_dir(&target) {
                Ok(()) => {}
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                    drop(open_directory(&target, FILE_READ_ATTRIBUTES)?);
                }
                Err(e) => return Err(e),
            }
            copy_tree_at(base, &entry.path(), &target)?;
        } else if metadata.is_file() {
            let source = pin_hive(&entry.path())?;
            if unsafe {
                CopyFileW(
                    wide(entry.path().as_os_str())?.as_ptr(),
                    wide(target.as_os_str())?.as_ptr(),
                    1,
                )
            } == 0
            {
                return Err(io::Error::last_os_error());
            }
            if source.metadata()?.len() != std::fs::metadata(target)?.len() {
                return Err(invalid("Copied launcher file size differs"));
            }
        } else {
            return Err(invalid("Unsupported launcher data entry"));
        }
    }
    Ok(())
}
pub(super) fn migrate_reviewed() -> io::Result<usize> {
    let _instance = crate::engine::app_instance::acquire().map_err(io::Error::other)?;
    // Retain the SAME existence gate throughout source copy and locator cutover.
    let _gate = acquire_startup_gate(crate::engine::native_clients_session::CONTROLLER_GATE_NAME,
        invalid("Close Minecraft and its session controller normally before migrating"))?;
    require_no_games()?;
    drop(registered_image(true)?);
    drop(BootstrapLibrary::reviewed()?);
    let home = home()?;
    let selected = resolve_saved_location(&home)?;
    let logical = logical_appdata_root()?;
    let (sources, target, app_preferences) = match open_at(logical.try_clone()?, &selected.0) {
        Ok(_) => return Ok(0), // Already canonical; no data migrated.
        Err(error) if error.kind() == io::ErrorKind::Unsupported && error.to_string() == MIGRATION_REQUIRED => {
            let sources = open_catalog(logical, &selected.0, 1)?;
            // Keep the original catalog and profile directories intact.
            let target = selected.0.join("Migrated");
            match std::fs::symlink_metadata(&target) {
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
                Ok(_) => return Err(invalid("Migration destination already exists; preserve it and Locate only a verified complete catalog")),
            }
            (sources, target, None)
        }
        Err(error) if selected.1 && error.kind() == io::ErrorKind::NotFound && empty_default(&selected.0)? =>
            (super::profile_binding::migration_sources()?, selected.0.clone(), Some(home.join("App"))),
        Err(error) => return Err(error),
    };
    migrate_sources(&target, &sources, app_preferences.as_deref())?;
    require_no_games()?;
    publish_migrated_location(&home, &selected, &target, sources[0].logical.try_clone()?)?;
    Ok(sources.len())
}
fn publish_migrated_location(home: &Path, selected: &(PathBuf, bool, bool), target: &Path, logical: File) -> io::Result<()> {
    let _pins = pin_chain(target)?;
    // Never switch the active pointer before all five new hive/directory IDs reopen.
    let _stores = open_at(logical, target)?; // Retain validated directories/hives through publication.
    if &resolve_saved_location(home)? != selected {
        return Err(invalid("Saved data location changed during migration; both sources were preserved"));
    }
    std::fs::create_dir_all(home.join("Data/Settings"))?;
    write_json(&locator_path(home), &Locator {
        schema: 1,
        path: if target == home.join("Data") { "Data".into() } else { target.to_string_lossy().into_owned() },
    }, !selected.1 && !selected.2)
}
fn migrate_sources(root: &Path, sources: &[SlotStorage], app_preferences: Option<&Path>) -> io::Result<()> {
    use windows_sys::Win32::System::Registry::{RegFlushKey, RegSetValueExW, REG_BINARY};
    if sources.len() != 5
        || sources
            .iter()
            .enumerate()
            .any(|(i, store)| store.slot != i as u32 + 1)
    {
        return Err(invalid(
            "Migration requires the original five launcher stores",
        ));
    }
    let parent = root.parent().ok_or_else(|| invalid("Missing migration destination parent"))?;
    let parent_pins = pin_chain(parent)?;
    let destination = directory_path(parent_pins.last().unwrap())?
        .join(root.file_name().ok_or_else(|| invalid("Missing migration destination name"))?);
    let root = destination.as_path();
    disjoint(&sources[0].logical, root)?;
    let mut source_pins = Vec::new();
    for store in sources {
        disjoint(&store.private, root)?;
        let pins = pin_chain(&store.private_path)?;
        if !same_image(&directory_id(pins.last().unwrap())?, &directory_id(&store.private)?) {
            return Err(invalid("Migration source directory changed"));
        }
        source_pins.extend(pins);
        for name in ["Roaming", "Local"] {
            let path = store.private_path.join(name);
            source_pins.push(open_directory(&path, FILE_READ_ATTRIBUTES)?);
            check_copy_tree(&path)?;
        }
    }
    // Schema-1 cutover is create-new only. The replay-to-default Data
    // destination alone may already contain ordinary Settings/Cache.
    if app_preferences.is_none() {
        std::fs::create_dir(root)?;
    } else {
        match std::fs::create_dir(root) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    let _pins = pin_chain(root)?;
    if !empty_default(root)? {
        return Err(invalid("Existing Data folder was preserved; migration requires an unused launcher destination"));
    }
    protect_data(root)?;
    std::fs::create_dir(root.join("Launchers"))?;
    let mut stores = Vec::new();
    for source in sources {
        let target = create_file_slot(
            &source.logical,
            &root.join(format!("Launchers/Minecraft-{}", source.slot)),
            source.slot,
        )?;
        for name in ["Roaming", "Local"] {
            copy_tree(
                &source.private_path.join(name),
                &target.private_path.join(name),
            )?;
        }
        // App hives require one uniform descriptor; retain the destination ACL.
        let status = unsafe {
            windows_sys::Win32::UI::Shell::SHCopyKeyW(
                source.registry.0,
                ptr::null(),
                target.registry.0,
                0,
            )
        };
        if status != 0 {
            return Err(io::Error::from_raw_os_error(status as i32));
        }
        let marker = hive_identity(&target.private, target.slot)?;
        let status = unsafe {
            RegSetValueExW(
                target.registry.0,
                wide(OsStr::new(HIVE_IDENTITY))?.as_ptr(),
                0,
                REG_BINARY,
                marker.as_ptr(),
                marker.len() as u32,
            )
        };
        if status != 0 {
            return Err(io::Error::from_raw_os_error(status as i32));
        }
        let status = unsafe { RegFlushKey(target.registry.0) };
        if status != 0 {
            return Err(io::Error::from_raw_os_error(status as i32));
        }
        stores.push(target);
    }
    // Preferences remain in the manager's Data/Settings. Copy only on the
    // original replay-to-default-root migration, never into a nested profile root.
    if let Some(app) = app_preferences {
        let settings = root.join("Settings");
        if !settings.try_exists()? {
            std::fs::create_dir(&settings)?;
        }
        for name in ["performance.json", "graphics.json"] {
            let source = app.join(name);
            let destination = settings.join(name);
            if source.try_exists()? && !destination.try_exists()? {
                let _source = pin_image(&source)?;
                if unsafe {
                    windows_sys::Win32::Storage::FileSystem::CopyFileW(
                        wide(source.as_os_str())?.as_ptr(),
                        wide(destination.as_os_str())?.as_ptr(),
                        1,
                    )
                } == 0 {
                    return Err(io::Error::last_os_error());
                }
            }
        }
    }
    publish_catalog(&root, &stores)?;
    drop(stores);
    Ok(())
}
fn require_idle() -> io::Result<()> {
    if existing_controller_gate(crate::engine::native_clients_session::CONTROLLER_GATE_NAME)? {
        return Err(invalid("Close Minecraft and its session controller normally before changing data locations"));
    }
    require_no_games()
}
fn require_no_games() -> io::Result<()> {
    if !super::super::inspect_native_workspace()?.processes.is_empty() {
        return Err(invalid(
            "Close Minecraft and its session controller normally before changing data locations",
        ));
    }
    Ok(())
}
pub(super) fn prepare() -> io::Result<()> {
    require_idle()?;
    drop(registered_image(true)?);
    drop(BootstrapLibrary::reviewed()?);
    let home = home()?;
    let (path, default, _) = resolve_saved_location(&home)?;
    if !default {
        return Err(invalid(
            "The saved data folder is unavailable; locate it instead of creating new profiles",
        ));
    }
    let logical = logical_appdata_root()?;
    initialize_at(&logical, &path)?;
    std::fs::create_dir_all(home.join("Data/Settings"))?;
    write_json(
        &locator_path(&home),
        &Locator {
            schema: 1,
            path: "Data".into(),
        },
        false,
    )
}

fn verify_selected_location(logical: File, root: &Path) -> io::Result<()> {
    open_at(logical, root).map(drop).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            io::Error::new(error.kind(), "Select an existing Lazy Tester Data folder containing profiles.json and all five profiles. Empty folders and older pilot folders cannot be reconnected here. Your current data location has not changed.")
        } else {
            error
        }
    })
}

pub(crate) fn location(action: DataLocationAction, persist_location: bool) -> io::Result<DataLocation> {
    // The hidden startup diagnostic is an observation, not setup/recovery.
    if !persist_location && !matches!(action, DataLocationAction::Inspect) {
        return Err(invalid("Startup diagnostic permits inspection only"));
    }
    let home = home()?;
    let locator = locator_path(&home);
    if matches!(
        action,
        DataLocationAction::Locate | DataLocationAction::Choose
    ) {
        require_idle()?;
        if let Some(selected) = rfd::FileDialog::new()
            .set_title(if matches!(action, DataLocationAction::Locate) {
                "Locate your Lazy Tester Data folder"
            } else {
                "Choose an empty folder for Minecraft data"
            })
            .pick_folder()
        {
            let pins = pin_chain(&selected)?;
            let selected = directory_path(pins.last().unwrap())?;
            if !selected.starts_with(home.join("Data")) {
                return Err(invalid(
                    "Keep launcher data inside this project's Data folder",
                ));
            }
            if matches!(action, DataLocationAction::Choose) {
                if open()?.is_some() || super::profile_binding::has_legacy_source()? {
                    return Err(invalid("Existing profile data was preserved. Use Locate data after moving the same data folder; creating a replacement is not allowed."));
                }
                drop(registered_image(true)?);
                drop(BootstrapLibrary::reviewed()?);
                let logical = logical_appdata_root()?;
                initialize_at(&logical, &selected)?;
            }
            let logical = logical_appdata_root()?;
            verify_selected_location(logical, &selected)?;
            require_idle()?;
            let relative = selected == home.join("Data");
            write_json(
                &locator,
                &Locator {
                    schema: 1,
                    path: if relative {
                        "Data".into()
                    } else {
                        selected.to_string_lossy().into_owned()
                    },
                },
                true,
            )?;
        }
    }
    // Report the exact catalog root from the same successful open.
    // A diagnostic must not roll back a cutover or publish a locator.
    let (current, profiles) = match open_mode(&home, persist_location)? {
        Some((root, stores)) => (
            Some(root),
            stores
                .into_iter()
                .map(|store| ProfilePath {
                    slot: store.slot,
                    path: store.private_path.to_string_lossy().into_owned(),
                })
                .collect(),
        ),
        None => (None, Vec::new()),
    };
    if matches!(action, DataLocationAction::Open) {
        let path = current
            .as_ref()
            .ok_or_else(|| invalid("No data location is configured"))?;
        let _pins = pin_chain(path)?;
        let wide = wide(path.as_os_str())?;
        if unsafe {
            windows_sys::Win32::UI::Shell::ShellExecuteW(
                ptr::null_mut(),
                windows_sys::w!("open"),
                wide.as_ptr(),
                ptr::null(),
                ptr::null(),
                1,
            )
        } as isize
            <= 32
        {
            return Err(invalid("Windows could not open the data folder"));
        }
    }
    Ok(DataLocation {
        schema: 1,
        default_path: home.join("Data").to_string_lossy().into_owned(),
        path: current.map(|path| path.to_string_lossy().into_owned()),
        profiles,
        can_locate: !existing_controller_gate("Local\\MivubiEducationStartupObservation")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn file_hive_reopens_without_hkcu_and_rejects_missing_or_foreign_identity() {
        let fixture = Fixture::new();
        let logical = fixture.logical();
        let data = fixture.root.join("Data");
        initialize_at(&logical, &data).unwrap();
        let stores = open_at(logical.try_clone().unwrap(), &data).unwrap();
        let private = stores[0].private.try_clone().unwrap();
        assert!(open_hive(&private, 2, false).is_err());
        drop(open_hive(&private, 1, false).unwrap());
        let hive = stores[0].private_path.join("Registry/HKCU.dat");
        drop(stores);
        std::fs::rename(&hive, hive.with_file_name("preserved.dat")).unwrap();
        assert!(open_hive(&private, 1, false).is_err());
        assert!(!hive.exists());
    }
    #[test]
    fn migration_preserves_sources_and_reopens_all_five_with_new_file_identities() {
        let fixture = Fixture::new();
        let logical = fixture.logical();
        let source = fixture.root.join("Original");
        initialize_at(&logical, &source).unwrap();
        let stores = open_at(logical.try_clone().unwrap(), &source).unwrap();
        for store in &stores {
            std::fs::write(
                store.private_path.join("Roaming/world-fixture"),
                format!("world-{}", store.slot),
            )
            .unwrap();
        }
        let target = fixture.root.join("Data");
        migrate_sources(&target, &stores, None).unwrap();
        assert!(migrate_sources(&target, &stores, None).is_err());
        let moved = open_at(logical, &target).unwrap();
        for (old, new) in stores.iter().zip(&moved) {
            assert!(!same_image(
                &directory_id(&old.private).unwrap(),
                &directory_id(&new.private).unwrap()
            ));
            assert_eq!(
                std::fs::read(old.private_path.join("Roaming/world-fixture")).unwrap(),
                std::fs::read(new.private_path.join("Roaming/world-fixture")).unwrap()
            );
        }
    }
    #[test]
    fn schema_one_is_migration_only_and_locator_cutover_preserves_original_catalog() {
        let mut fixture = Fixture::new();
        let logical = fixture.logical();
        let root = fixture.root.join("Data");
        std::fs::create_dir_all(root.join("Profiles")).unwrap();
        let mut original = Vec::new();
        for slot in 1..=5 {
            let store = super::super::profile_storage_handles::create_private_slot_at(
                &logical, &root.join(format!("Profiles/Minecraft-{slot}")), slot).unwrap();
            fixture.keys.push(registry_leaf(&directory_id(&store.private).unwrap(), slot).unwrap());
            std::fs::write(store.private_path.join("Roaming/world-fixture"), [slot as u8]).unwrap();
            original.push(store);
        }
        let catalog = Catalog {
            schema: 1, owner: owner().unwrap(),
            root: Identity::read(&open_directory(&root, FILE_READ_ATTRIBUTES).unwrap()).unwrap(),
            profiles: original.iter().map(|store| Identity::read(&store.private).unwrap())
                .collect::<Vec<_>>().try_into().ok().unwrap(),
            hives: None,
        };
        write_json(&root.join(CATALOG), &catalog, false).unwrap();
        let original_catalog = std::fs::read(root.join(CATALOG)).unwrap();
        drop(original);
        assert!(matches!(open_at(logical.try_clone().unwrap(), &root), Err(error)
            if error.kind() == io::ErrorKind::Unsupported));
        let selected = resolve_saved_location(&fixture.root).unwrap();
        let sources = open_catalog(logical.try_clone().unwrap(), &root, 1).unwrap();
        let target = root.join("Migrated");
        let overlapping = sources[0].private_path.join("Roaming").join("must-not-create");
        assert!(migrate_sources(&overlapping, &sources, None).is_err());
        assert!(!overlapping.exists());
        std::fs::create_dir(&target).unwrap();
        assert!(migrate_sources(&target, &sources, None).is_err());
        assert!(std::fs::read_dir(&target).unwrap().next().is_none());
        std::fs::remove_dir(&target).unwrap(); // Disposable fixture only.
        // A missing/partial target must never publish a new pointer.
        assert!(publish_migrated_location(&fixture.root, &selected, &target,
            logical.try_clone().unwrap()).is_err());
        assert!(!locator_path(&fixture.root).exists());
        migrate_sources(&target, &sources, None).unwrap();
        assert!(migrate_sources(&target, &sources, None).is_err());
        assert_eq!(resolve_saved_location(&fixture.root).unwrap(), selected);
        publish_migrated_location(&fixture.root, &selected, &target, logical.try_clone().unwrap()).unwrap();
        assert_eq!(resolve_saved_location(&fixture.root).unwrap().0, target);
        assert_eq!(std::fs::read(root.join(CATALOG)).unwrap(), original_catalog);
        let current = open_at(logical, &target).unwrap();
        for (old, new) in sources.iter().zip(&current) {
            assert_eq!(std::fs::read(old.private_path.join("Roaming/world-fixture")).unwrap(),
                std::fs::read(new.private_path.join("Roaming/world-fixture")).unwrap());
            assert!(!same_image(&directory_id(&old.private).unwrap(), &directory_id(&new.private).unwrap()));
        }
    }
    #[test]
    fn migration_replaces_only_the_known_wininet_junction_without_following_it() {
        let fixture = Fixture::new();
        let local = fixture.root.join("Local");
        let external = fixture.root.join("external");
        std::fs::create_dir(&external).unwrap();
        std::fs::write(external.join("outside-fixture"), b"must stay outside").unwrap();
        let link = local.join("Microsoft/Windows/INetCache/Content.IE5");
        std::fs::create_dir_all(link.parent().unwrap()).unwrap();
        let status = std::process::Command::new("cmd.exe")
            .args(["/d", "/c", "mklink", "/J"])
            .arg(&link)
            .arg(&external)
            .output()
            .unwrap();
        assert!(status.status.success());
        check_copy_tree(&local).unwrap();
        let destination = fixture.root.join("Copied");
        std::fs::create_dir(&destination).unwrap();
        copy_tree(&local, &destination).unwrap();
        let copied = destination.join("Microsoft/Windows/INetCache/Content.IE5");
        assert!(std::fs::read_dir(&copied).unwrap().next().is_none());
        assert_eq!(
            std::fs::read(external.join("outside-fixture")).unwrap(),
            b"must stay outside"
        );
        assert!(!portable_cache_junction(&local, &local.join("Unknown")).unwrap());
        let unknown = std::process::Command::new("cmd.exe")
            .args(["/d", "/c", "mklink", "/J"])
            .arg(local.join("Unknown"))
            .arg(&external)
            .output()
            .unwrap();
        assert!(unknown.status.success());
        assert!(check_copy_tree(&local).is_err());
    }
    struct Fixture {
        root: PathBuf,
        keys: Vec<String>,
    }
    impl Fixture {
        fn new() -> Self {
            Self {
                root: fresh_root(&std::env::temp_dir()).unwrap(),
                keys: Vec::new(),
            }
        }
        fn logical(&self) -> File {
            let path = self.root.join("logical");
            std::fs::create_dir(&path).unwrap();
            open_directory(&path, FILE_READ_ATTRIBUTES).unwrap()
        }
        fn remember(&mut self, data: &Path) {
            if read_json::<Catalog>(&data.join(CATALOG)).unwrap().schema == 2 {
                return;
            }
            for slot in 1..=5 {
                let file = open_directory(
                    &data.join(format!("Profiles/Minecraft-{slot}")),
                    FILE_READ_ATTRIBUTES,
                )
                .unwrap();
                self.keys
                    .push(registry_leaf(&directory_id(&file).unwrap(), slot).unwrap());
            }
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            for key in &self.keys {
                let key = wide(OsStr::new(&format!(
                    "Software\\MivubiEducationPilot\\{key}"
                )))
                .unwrap();
                assert_eq!(
                    unsafe {
                        windows_sys::Win32::System::Registry::RegDeleteTreeW(
                            HKEY_CURRENT_USER,
                            key.as_ptr(),
                        )
                    },
                    0
                );
            }
            assert_eq!(self.root.parent(), Some(std::env::temp_dir().as_path()));
            std::fs::remove_dir_all(&self.root).unwrap();
        }
    }
    #[test]
    fn creates_five_stores_reopens_them_and_never_resets_existing_data() {
        let mut fixture = Fixture::new();
        let logical = fixture.logical();
        let data = fixture.root.join("Data");
        initialize_at(&logical, &data).unwrap();
        fixture.remember(&data);
        verify_selected_location(logical.try_clone().unwrap(), &data).unwrap();
        let first = open_at(logical.try_clone().unwrap(), &data).unwrap();
        assert_eq!(
            first.iter().map(|store| store.slot).collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5]
        );
        let ids = first
            .iter()
            .map(|store| directory_id(&store.private).unwrap().FileId.Identifier)
            .collect::<Vec<_>>();
        drop(first);
        let sentinel = data.join("Launchers/Minecraft-1/Roaming/world-fixture");
        std::fs::write(&sentinel, b"keep").unwrap();
        assert!(initialize_at(&logical, &data).is_err());
        let again = open_at(logical.try_clone().unwrap(), &data).unwrap();
        assert_eq!(
            again
                .iter()
                .map(|store| directory_id(&store.private).unwrap().FileId.Identifier)
                .collect::<Vec<_>>(),
            ids
        );
        drop(again);
        assert_eq!(std::fs::read(&sentinel).unwrap(), b"keep");
        let moved = fixture.root.join("MovedData");
        std::fs::rename(&data, &moved).unwrap();
        let relocated = open_at(logical.try_clone().unwrap(), &moved).unwrap();
        assert_eq!(relocated[0].slot, 1);
        drop(relocated);
        std::fs::rename(&moved, &data).unwrap();
        std::fs::rename(
            data.join("Launchers/Minecraft-5"),
            data.join("Launchers/saved-5"),
        )
        .unwrap();
        std::fs::create_dir(data.join("Launchers/Minecraft-5")).unwrap();
        assert!(open_at(logical, &data).is_err());
    }
    #[test]
    fn selected_incomplete_folders_explain_recovery_without_creating_metadata() {
        let fixture = Fixture::new();
        let logical = fixture.logical();
        let data = fixture.root.join("Data");
        std::fs::create_dir(&data).unwrap();
        let error = verify_selected_location(logical.try_clone().unwrap(), &data).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::NotFound);
        assert!(error
            .to_string()
            .contains("profiles.json and all five profiles"));
        assert!(empty_default(&data).unwrap());
        let marker = data.join("partial-profile-marker");
        std::fs::write(&marker, b"keep").unwrap();
        assert!(verify_selected_location(logical, &data).is_err());
        assert_eq!(std::fs::read(&marker).unwrap(), b"keep");
        assert!(!data.join(CATALOG).exists());
        assert!(!data.join(LOCATOR).exists());
    }
    #[test]
    fn canonical_and_legacy_locator_share_one_resolution_without_fresh_fallback() {
        let fixture = Fixture::new();
        let home = &fixture.root;
        let data = home.join("Data");
        let saved = data.join("Saved");
        std::fs::create_dir_all(&saved).unwrap();
        std::fs::create_dir_all(home.join("App")).unwrap();
        std::fs::create_dir_all(data.join("Settings")).unwrap();
        let canonical = locator_path(home);
        let legacy = home.join("App").join(LOCATOR);
        assert_eq!(resolve_saved_location(home).unwrap(), (data.clone(), true, false));
        write_json(&legacy, &Locator { schema: 1, path: saved.to_string_lossy().into_owned() }, false).unwrap();
        assert_eq!(resolve_saved_location(home).unwrap(), (saved, false, true));
        assert!(!canonical.exists());
        write_json(&canonical, &Locator { schema: 1, path: "Data".into() }, false).unwrap();
        assert_eq!(resolve_saved_location(home).unwrap(), (data, false, false));
        std::fs::write(&canonical, b"corrupt").unwrap();
        assert!(resolve_saved_location(home).is_err());
        assert_eq!(std::fs::read(&canonical).unwrap(), b"corrupt");
        std::fs::remove_file(&canonical).unwrap();
        // A non-file canonical pointer is invalid, not permission to use legacy.
        std::fs::create_dir(&canonical).unwrap();
        assert!(resolve_saved_location(home).is_err());
        assert!(canonical.is_dir());
        std::fs::remove_dir(&canonical).unwrap();
        std::fs::write(&legacy, b"corrupt").unwrap();
        assert!(resolve_saved_location(home).is_err());
        assert!(!canonical.exists());
    }
    #[test]
    fn relative_default_follows_home_and_custom_missing_paths_never_fall_back() {
        let fixture = Fixture::new();
        let empty = fixture.root.join("Data");
        std::fs::create_dir(&empty).unwrap();
        assert!(empty_default(&empty).unwrap());
        std::fs::write(empty.join("preserved"), b"keep").unwrap();
        assert!(!empty_default(&empty).unwrap());
        let app = fixture.root.join("App");
        std::fs::create_dir(&app).unwrap();
        let locator = app.join(LOCATOR);
        assert_eq!(layout(&app.join("lazy-tester.exe")).unwrap(), fixture.root);
        write_json(
            &locator,
            &Locator {
                schema: 1,
                path: "Data".into(),
            },
            false,
        )
        .unwrap();
        let moved = fixture.root.join("moved-home");
        assert_eq!(
            resolve_location(&moved, &locator).unwrap().0,
            moved.join("Data")
        );
        assert!(!resolve_location(&moved, &locator).unwrap().1);
        write_json(
            &locator,
            &Locator {
                schema: 1,
                path: fixture
                    .root
                    .join("missing-drive")
                    .to_string_lossy()
                    .into_owned(),
            },
            true,
        )
        .unwrap();
        assert!(resolve_location(&fixture.root, &locator).is_err());
        write_json(
            &locator,
            &Locator {
                schema: 1,
                path: "../other".into(),
            },
            true,
        )
        .unwrap();
        assert!(resolve_location(&fixture.root, &locator).is_err());
        std::fs::write(&locator, b"corrupt").unwrap();
        assert!(resolve_location(&fixture.root, &locator).is_err());
        assert_eq!(std::fs::read(&locator).unwrap(), b"corrupt");
    }
}
