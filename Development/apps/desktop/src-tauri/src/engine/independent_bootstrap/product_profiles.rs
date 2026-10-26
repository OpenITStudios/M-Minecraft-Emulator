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
        temporary: private_temporary(&private)?,
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
fn locator_path() -> io::Result<PathBuf> {
    Ok(home()?.join("Data/Settings").join(LOCATOR))
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
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok((home.join("Data"), true)),
        Err(error) => Err(error),
    }
}
fn disjoint(logical: &File, data: &Path) -> io::Result<()> {
    let logical = directory_path(logical)?
        .to_string_lossy()
        .to_ascii_lowercase();
    let data = data.to_string_lossy().to_ascii_lowercase();
    if Path::new(&logical).starts_with(&data) || Path::new(&data).starts_with(&logical) {
        return Err(invalid("Choose a data location outside Windows AppData"));
    }
    Ok(())
}
fn open_at(logical: File, root: &Path) -> io::Result<[SlotStorage; 5]> {
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
            temporary: existing_private_temporary(&private)?,
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
    open_mode(true)
}
pub(super) fn inspect_existing() -> io::Result<Option<[SlotStorage; 5]>> {
    open_mode(false)
}
fn recover_confirmed(root:&Path) {
    // Startup restores unfinished cutovers; large committed purges require Finish clear.
    // Inspection/Locate stay read-only and unrelated catalog-valid peers remain available.
    for slot in 1..=5 {let _=clear::rollback_uncommitted(root,slot);}
}
fn open_mode(persist_location: bool) -> io::Result<Option<[SlotStorage; 5]>> {
    let home = home()?;
    let locator = locator_path()?;
    let legacy = home.join("App").join(LOCATOR);
    let legacy_locator = !locator.try_exists()? && legacy.try_exists()?;
    let (root, default) = resolve_location(&home, if legacy_locator { &legacy } else { &locator })?;
    let (logical, _) = replay_roots()?;
    if persist_location {recover_confirmed(&root);}
    match open_at(logical, &root) {
        Ok(stores) => {
            // A successful setup may have stopped between catalog and locator
            // publication. Persist the pointer before admitting any game launch.
            if (default || legacy_locator) && persist_location {
                let settings = home.join("Data/Settings");
                std::fs::create_dir_all(&settings)?;
                write_json(
                    &locator_path()?,
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
            Ok(Some(stores))
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
    let mut tag: FILE_ATTRIBUTE_TAG_INFO = unsafe { mem::zeroed() };
    if unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle() as _,
            FileAttributeTagInfo,
            &mut tag as *mut _ as _,
            mem::size_of_val(&tag) as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
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
pub(super) fn migrate_reviewed() -> io::Result<()> {
    let _instance = crate::engine::app_instance::acquire().map_err(io::Error::other)?;
    require_idle()?;
    drop(registered_image(true)?);
    drop(BootstrapLibrary::reviewed()?);
    if open()?.is_some() {
        return Ok(());
    }
    let sources = registered_slots::<5>(true)?;
    migrate_sources(&home()?.join("Data"), &sources)?;
    drop(open()?.ok_or_else(|| invalid("Migrated profiles did not reopen"))?);
    Ok(())
}
fn migrate_sources(root: &Path, sources: &[SlotStorage]) -> io::Result<()> {
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
    for store in sources {
        for name in ["Roaming", "Local"] {
            check_copy_tree(&store.private_path.join(name))?;
        }
    }
    if !empty_default(&root)? {
        return Err(invalid(
            "Existing Data folder was preserved; migration requires an unused launcher destination",
        ));
    }
    if !root.try_exists()? {
        std::fs::create_dir(&root)?;
    }
    let _pins = pin_chain(&root)?;
    protect_data(&root)?;
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
    let settings = root.join("Settings");
    if !settings.try_exists()? {
        std::fs::create_dir(&settings)?;
    }
    for name in ["performance.json", "graphics.json"] {
        let source = root
            .parent()
            .ok_or_else(|| invalid("Project root unavailable"))?
            .join("App")
            .join(name);
        let destination = settings.join(name);
        if source.try_exists()? && !destination.try_exists()? {
            let _source = pin_image(&source)?;
            if unsafe {
                windows_sys::Win32::Storage::FileSystem::CopyFileW(
                    wide(source.as_os_str())?.as_ptr(),
                    wide(destination.as_os_str())?.as_ptr(),
                    1,
                )
            } == 0
            {
                return Err(io::Error::last_os_error());
            }
        }
    }
    publish_catalog(&root, &stores)?;
    drop(stores);
    Ok(())
}
fn require_idle() -> io::Result<()> {
    if existing_controller_gate("Local\\MivubiEducationStartupObservation")?
        || !super::super::inspect_native_workspace()?
            .processes
            .is_empty()
    {
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
    let (path, default) = resolve_location(&home()?, &locator_path()?)?;
    if !default {
        return Err(invalid(
            "The saved data folder is unavailable; locate it instead of creating new profiles",
        ));
    }
    let (logical, _) = replay_roots()?;
    initialize_at(&logical, &path)?;
    std::fs::create_dir_all(home()?.join("Data/Settings"))?;
    write_json(
        &locator_path()?,
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

pub(crate) fn location(action: DataLocationAction) -> io::Result<DataLocation> {
    let home = home()?;
    let locator = locator_path()?;
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
                if open()?.is_some() || super::profile_binding::has_legacy_binding()? {
                    return Err(invalid("Existing profile data was preserved. Use Locate data after moving the same data folder; creating a replacement is not allowed."));
                }
                drop(registered_image(true)?);
                drop(BootstrapLibrary::reviewed()?);
                let (logical, _) = replay_roots()?;
                initialize_at(&logical, &selected)?;
            }
            let (logical, _) = replay_roots()?;
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
    let (path, _) = resolve_location(&home, &locator)?;
    let (current, profiles) = match open()? {
        Some(stores) => (
            Some(path.clone()),
            stores
                .into_iter()
                .map(|store| ProfilePath {
                    slot: store.slot,
                    path: store.private_path.to_string_lossy().into_owned(),
                })
                .collect(),
        ),
        None => match registered_slots::<5>(true) {
            Ok(stores) => {
                let current = stores[0].private_path.parent().map(Path::to_path_buf);
                (
                    current,
                    stores
                        .into_iter()
                        .map(|store| ProfilePath {
                            slot: store.slot,
                            path: store.private_path.to_string_lossy().into_owned(),
                        })
                        .collect(),
                )
            }
            Err(_) => (None, Vec::new()),
        },
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
        migrate_sources(&target, &stores).unwrap();
        assert!(migrate_sources(&target, &stores).is_err());
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
