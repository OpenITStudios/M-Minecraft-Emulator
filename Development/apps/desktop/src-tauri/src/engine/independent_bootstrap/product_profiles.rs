//! Product data locations and five persistent stores. No account/world migration.
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
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Catalog {
    schema: u32,
    owner: [u8; 32],
    root: Identity,
    profiles: [Identity; 5],
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
    let app = executable
        .parent()
        .ok_or_else(|| invalid("Application folder unavailable"))?;
    Ok(
        if app
            .file_name()
            .is_some_and(|name| name.eq_ignore_ascii_case("App"))
        {
            app.parent()
                .ok_or_else(|| invalid("Application root unavailable"))?
                .to_path_buf()
        } else {
            app.to_path_buf()
        },
    ) // Existing flat candidates remain in place.
}
fn home() -> io::Result<PathBuf> {
    layout(&std::env::current_exe()?)
}
fn locator_path() -> io::Result<PathBuf> {
    Ok(std::env::current_exe()?
        .parent()
        .ok_or_else(|| invalid("Application folder unavailable"))?
        .join(LOCATOR))
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
    if catalog.schema != 1
        || catalog.owner != owner()?
        || !catalog.root.matches(pins.last().unwrap())?
    {
        return Err(invalid(
            "This data folder does not match the saved Windows user and directory identity",
        ));
    }
    disjoint(&logical, &directory_path(pins.last().unwrap())?)?;
    let _profiles = open_directory(
        &root.join("Profiles"),
        FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY,
    )?;
    let user_key_path = user_registry_path()?;
    let mut stores = Vec::new();
    for (index, identity) in catalog.profiles.iter().enumerate() {
        let slot = index as u32 + 1;
        let private = open_directory(
            &root.join(format!("Profiles/Minecraft-{slot}")),
            FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY,
        )?;
        if !identity.matches(&private)? {
            return Err(invalid(
                "A profile directory was replaced; existing data was preserved",
            ));
        }
        let private_path = directory_path(&private)?;
        let _roaming = open_directory(&private_path.join("Roaming"), FILE_READ_ATTRIBUTES)?;
        let key_path = wide(OsStr::new(&format!(
            "Software\\MivubiEducationPilot\\{}",
            registry_leaf(&directory_id(&private)?, slot)?
        )))?;
        let mut key = ptr::null_mut();
        let status = unsafe {
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                key_path.as_ptr(),
                0,
                KEY_ALL_ACCESS,
                &mut key,
            )
        };
        if status != 0 {
            return Err(io::Error::from_raw_os_error(status as i32));
        }
        let registry = RegistryKey(key);
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
fn open_mode(persist_location: bool) -> io::Result<Option<[SlotStorage; 5]>> {
    let (root, default) = resolve_location(&home()?, &locator_path()?)?;
    let (logical, _) = replay_roots()?;
    match open_at(logical, &root) {
        Ok(stores) => {
            // A successful setup may have stopped between catalog and locator
            // publication. Persist the pointer before admitting any game launch.
            if default && persist_location {
                write_json(
                    &locator_path()?,
                    &Locator {
                        schema: 1,
                        path: "Data".into(),
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
    Ok(std::fs::read_dir(root)?.next().is_none())
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
            if std::fs::read_dir(root)?.next().is_some() {
                return Err(invalid("Data folder is not empty. Locate existing data instead of creating new profiles."));
            }
        }
        Err(error) => return Err(error),
    }
    let pins = pin_chain(root)?;
    std::fs::create_dir(root.join("Profiles"))?;
    let _profiles = open_directory(
        &root.join("Profiles"),
        FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY,
    )?;
    let mut stores = Vec::new();
    for slot in 1..=5 {
        stores.push(create_private_slot_at(
            logical,
            &root.join(format!("Profiles/Minecraft-{slot}")),
            slot,
        )?);
    }
    let ids: Vec<Identity> = stores
        .iter()
        .map(|store| Identity::read(&store.private))
        .collect::<io::Result<_>>()?;
    let catalog = Catalog {
        schema: 1,
        owner: owner()?,
        root: Identity::read(pins.last().unwrap())?,
        profiles: ids
            .try_into()
            .map_err(|_| invalid("Missing product profile"))?,
    };
    write_json(&root.join(CATALOG), &catalog, false)?; // Complete identity record last; never overwrite partial data.
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
        let sentinel = data.join("Profiles/Minecraft-1/Roaming/world-fixture");
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
            data.join("Profiles/Minecraft-5"),
            data.join("Profiles/saved-5"),
        )
        .unwrap();
        std::fs::create_dir(data.join("Profiles/Minecraft-5")).unwrap();
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
        assert!(error.to_string().contains("profiles.json and all five profiles"));
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
