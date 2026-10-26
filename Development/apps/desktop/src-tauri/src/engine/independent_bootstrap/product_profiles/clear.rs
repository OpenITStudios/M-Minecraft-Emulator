//! Confirmed full-profile replacement; account/world payloads are never read or copied.
use super::*;
use std::fs;
use std::time::{Duration, Instant};
use windows_sys::Win32::Storage::FileSystem::*;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Transaction {
    schema: u32,
    owner: [u8; 32],
    root: Identity,
    slot: u32,
    stage: String,
    stage_id: Identity,
    old: Identity,
    old_hive: Identity,
    new: Identity,
    new_hive: Identity,
}

fn journal(root: &Path, slot: u32) -> PathBuf {
    root.join(format!(".clear-Minecraft-{slot}.json"))
}
fn profile(root: &Path, slot: u32) -> PathBuf {
    root.join(format!("Launchers/Minecraft-{slot}"))
}
fn same(a: &Identity, b: &Identity) -> bool {
    a.volume == b.volume && a.file == b.file
}

fn catalog(root: &Path) -> io::Result<Catalog> {
    let pins = pin_chain(root)?;
    let value: Catalog = read_json(&root.join(CATALOG))?;
    if value.schema != 2
        || value.hives.is_none()
        || value.owner != owner()?
        || !value.root.matches(pins.last().unwrap())?
    {
        return Err(invalid(
            "Full clear requires this project's verified file-backed launcher data",
        ));
    }
    Ok(value)
}

fn bound(store: &SlotStorage) -> io::Result<(PathBuf, Catalog)> {
    if !(1..=5).contains(&store.slot) {
        return Err(invalid("Unknown launcher"));
    }
    let root = store
        .private_path
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| invalid("Launcher root unavailable"))?
        .to_path_buf();
    let value = catalog(&root)?;
    let index = store.slot as usize - 1;
    if profile(&root, store.slot) != directory_path(&store.private)?
        || !value.profiles[index].matches(&store.private)?
        || !value.hives.as_ref().unwrap()[index]
            .matches_image(&pin_hive(&store.private_path.join("Registry/HKCU.dat"))?)?
    {
        return Err(invalid("Launcher identity changed; no data was removed"));
    }
    Ok((root, value))
}

fn transaction(root: &Path, slot: u32) -> io::Result<Option<Transaction>> {
    if !(1..=5).contains(&slot) {
        return Err(invalid("Unknown launcher"));
    }
    let value: Transaction = match read_json(&journal(root, slot)) {
        Ok(value) => value,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let current = catalog(root)?;
    if value.schema != 1
        || value.slot != slot
        || value.owner != current.owner
        || !same(&value.root, &current.root)
        || !value
            .stage
            .strip_prefix("MivubiEducationPilot_")
            .is_some_and(|id| id.len() == 32 && id.bytes().all(|b| b.is_ascii_hexdigit()))
    {
        return Err(invalid(
            "Full-clear transaction metadata is invalid; data was preserved",
        ));
    }
    Ok(Some(value))
}

pub(in super::super) fn pending(store: &SlotStorage) -> io::Result<bool> {
    // Ordinary session inspection needs only absence of the reserved journal.
    // Full catalog/hive validation remains mandatory whenever it exists or data changes.
    let root = store
        .private_path
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| invalid("Launcher root unavailable"))?;
    match fs::symlink_metadata(journal(root, store.slot)) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
        Ok(_) => {}
    }
    let (root, current) = bound(store)?;
    let Some(operation) = transaction(&root, store.slot)? else {
        return Ok(false);
    };
    let id = &current.profiles[store.slot as usize - 1];
    if !same(id, &operation.old) && !same(id, &operation.new) {
        return Err(invalid(
            "Full-clear transaction does not match this launcher",
        ));
    }
    Ok(true)
}

pub(in super::super) fn confirmation(store: &SlotStorage) -> io::Result<String> {
    let (_, value) = bound(store)?;
    let index = store.slot as usize - 1;
    let bytes = serde_json::to_vec(&(
        value.owner,
        value.root,
        &value.profiles[index],
        &value.hives.as_ref().unwrap()[index],
        store.slot,
    ))
    .map_err(io::Error::other)?;
    Ok(Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

pub(in super::super) fn inspect(
    store: &SlotStorage,
    admission: io::Result<()>,
) -> crate::engine::native_clients_session::FullClearStorage {
    let mut report = crate::engine::native_clients_session::FullClearStorage {
        can_clear: false,
        pending: false,
        confirmation_token: None,
        issue: None,
    };
    let result = (|| {
        report.confirmation_token = Some(confirmation(store)?);
        report.pending = pending(store)?;
        admission?;
        report.can_clear = true;
        Ok::<_, io::Error>(())
    })();
    if let Err(error) = result {
        report.issue = Some(error.to_string());
    }
    report
}

pub(in super::super) fn run(store: SlotStorage, token: &str) -> io::Result<()> {
    if confirmation(&store)? != token {
        return Err(invalid(
            "Launcher data changed after confirmation; review it again",
        ));
    }
    let (root, _) = bound(&store)?;
    if transaction(&root, store.slot)?.is_some() {
        let slot = store.slot;
        let logical = store.logical.try_clone()?;
        drop(store);
        if recover(&root, slot)? {
            return Ok(());
        }
        let store = reopen(logical, &root, slot)?;
        return execute(store);
    }
    execute(store)
}

pub(in super::super) fn reopen(logical: File, root: &Path, slot: u32) -> io::Result<SlotStorage> {
    if !(1..=5).contains(&slot) {
        return Err(invalid("Unknown launcher"));
    }
    recover_confirmed(root);
    open_at(logical, root)?
        .into_iter()
        .nth(slot as usize - 1)
        .ok_or_else(|| invalid("Launcher could not be reopened"))
}

pub(in super::super) fn location(store: &SlotStorage) -> io::Result<PathBuf> {
    bound(store).map(|(root, _)| root)
}

fn deletion_directory(path: &Path) -> io::Result<File> {
    let file = OpenOptions::new()
        .access_mode(DELETE | FILE_READ_ATTRIBUTES | FILE_WRITE_ATTRIBUTES | FILE_LIST_DIRECTORY)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    directory_id(&file)?;
    Ok(file)
}

// Every payload is pinned with DELETE access and no read/write sharing before mutation.
// Reparse points/hardlinks/locked files abort preflight; no recursive path deletion.
struct DeletionPlan {
    files: Vec<(PathBuf, File)>,
    directories: Vec<File>,
    started: Instant,
}
impl DeletionPlan {
    fn read(path: &Path) -> io::Result<Self> {
        let mut plan = Self {
            files: Vec::new(),
            directories: Vec::new(),
            started: Instant::now(),
        };
        plan.visit(path, 0)?;
        Ok(plan)
    }
    fn visit(&mut self, path: &Path, depth: usize) -> io::Result<()> {
        if depth > 64
            || self.files.len() + self.directories.len() > 100_000
            || self.started.elapsed() > Duration::from_secs(15)
        {
            return Err(invalid(
                "Launcher data inspection limit reached; nothing was removed",
            ));
        }
        let directory = deletion_directory(path)?;
        for entry in fs::read_dir(path)? {
            if self.files.len() + self.directories.len() >= 100_000
                || self.started.elapsed() > Duration::from_secs(15)
            {
                return Err(invalid(
                    "Launcher data inspection limit reached; nothing was removed",
                ));
            }
            let path = entry?.path();
            if fs::symlink_metadata(&path)?.is_dir() {
                self.visit(&path, depth + 1)?;
            } else {
                let file = OpenOptions::new()
                    .access_mode(DELETE | FILE_READ_ATTRIBUTES | FILE_WRITE_ATTRIBUTES)
                    .share_mode(FILE_SHARE_DELETE)
                    .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
                    .open(&path)?;
                image_id(&file)?;
                let mut info: FILE_STANDARD_INFO = unsafe { mem::zeroed() };
                if unsafe {
                    GetFileInformationByHandleEx(
                        file.as_raw_handle() as _,
                        FileStandardInfo,
                        &mut info as *mut _ as _,
                        mem::size_of_val(&info) as u32,
                    )
                } == 0
                {
                    return Err(io::Error::last_os_error());
                }
                if info.NumberOfLinks != 1 || info.Directory || info.DeletePending {
                    return Err(invalid(
                        "Linked or unavailable launcher file; nothing was removed",
                    ));
                }
                self.files.push((path, file));
            }
        }
        self.directories.push(directory);
        Ok(())
    }
    fn root(&self) -> &File {
        self.directories.last().unwrap()
    }
    fn into_root(mut self) -> File {
        self.directories.pop().unwrap()
    }
    fn purge(self) -> io::Result<()> {
        for (_, file) in self.files {
            remove(file)?;
        }
        for directory in self.directories {
            remove(directory)?;
        }
        Ok(())
    }
}
fn remove(file: File) -> io::Result<()> {
    let mut basic: FILE_BASIC_INFO = unsafe { mem::zeroed() };
    if unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle() as _,
            FileBasicInfo,
            &mut basic as *mut _ as _,
            mem::size_of_val(&basic) as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    if basic.FileAttributes & FILE_ATTRIBUTE_READONLY != 0 {
        basic.FileAttributes &= !FILE_ATTRIBUTE_READONLY;
        if basic.FileAttributes == 0 {
            basic.FileAttributes = FILE_ATTRIBUTE_NORMAL;
        }
        if unsafe {
            SetFileInformationByHandle(
                file.as_raw_handle() as _,
                FileBasicInfo,
                &basic as *const _ as _,
                mem::size_of_val(&basic) as u32,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
    }
    let info = FILE_DISPOSITION_INFO { DeleteFile: true };
    if unsafe {
        SetFileInformationByHandle(
            file.as_raw_handle() as _,
            FileDispositionInfo,
            &info as *const _ as _,
            mem::size_of_val(&info) as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    drop(file);
    Ok(())
}
fn rename(file: &File, target: &Path) -> io::Result<()> {
    let name = wide(target.as_os_str())?;
    let bytes = mem::offset_of!(FILE_RENAME_INFO, FileName) + name.len() * 2;
    let mut storage = vec![0usize; bytes.div_ceil(mem::size_of::<usize>())];
    let info = storage.as_mut_ptr() as *mut FILE_RENAME_INFO;
    unsafe {
        (*info).Anonymous.ReplaceIfExists = false;
        (*info).RootDirectory = ptr::null_mut();
        (*info).FileNameLength = ((name.len() - 1) * 2) as u32;
        ptr::copy_nonoverlapping(name.as_ptr(), (*info).FileName.as_mut_ptr(), name.len());
        if SetFileInformationByHandle(
            file.as_raw_handle() as _,
            FileRenameInfo,
            info as _,
            bytes as u32,
        ) == 0
        {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}
fn matching(path: &Path, id: &Identity) -> io::Result<Option<File>> {
    match deletion_directory(path) {
        Ok(file) if id.matches(&file)? => Ok(Some(file)),
        Ok(_) => Err(invalid(
            "Full-clear directory identity changed; no further data was removed",
        )),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}
fn validate_profile(
    root: &Path,
    slot: u32,
    id: &Identity,
    hive: &Identity,
) -> io::Result<(File, File, RegistryKey)> {
    let path = profile(root, slot);
    let private = open_directory(&path, FILE_READ_ATTRIBUTES)?;
    let file = pin_hive(&path.join("Registry/HKCU.dat"))?;
    if !id.matches(&private)? || !hive.matches_image(&file)? {
        return Err(invalid("Full-clear profile identity is no longer valid"));
    }
    let registry = open_hive(&private, slot, false)?;
    Ok((private, file, registry))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Step {
    Prepared,
    Journal,
    OldMoved,
    NewMoved,
    Catalog,
    PurgeReady,
}

pub(in super::super) fn execute(store: SlotStorage) -> io::Result<()> {
    execute_probed(store, |_| Ok(()))
}
fn execute_probed(
    store: SlotStorage,
    mut probe: impl FnMut(Step) -> io::Result<()>,
) -> io::Result<()> {
    let (root, mut current) = bound(&store)?;
    if transaction(&root, store.slot)?.is_some() {
        return Err(invalid(
            "Finish the previous full-clear operation before starting another",
        ));
    }
    let slot = store.slot;
    let logical = store.logical.try_clone()?;
    let _pins = pin_chain(&root)?;
    let _profiles = open_directory(
        &root.join("Launchers"),
        FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY,
    )?;
    let old_path = profile(&root, slot);
    drop(store); // Release only this inactive launcher's hive and temporary-directory pins.
    let old = DeletionPlan::read(&old_path)?;
    let index = slot as usize - 1;
    let hive = old
        .files
        .iter()
        .find(|(path, _)| *path == old_path.join("Registry/HKCU.dat"))
        .ok_or_else(|| invalid("Original launcher registry file unavailable"))?;
    if !current.profiles[index].matches(old.root())?
        || !current.hives.as_ref().unwrap()[index].matches_image(&hive.1)?
    {
        return Err(invalid(
            "Original launcher changed before full-clear admission",
        ));
    }
    let stage = fresh_root(&root.join("Launchers"))?;
    let stage_id = Identity::read(&open_directory(&stage, FILE_READ_ATTRIBUTES)?)?;
    let stage_name = stage.file_name().unwrap().to_string_lossy().into_owned();
    let prepared = (|| {
        let fresh = create_file_slot(&logical, &stage.join("new"), slot)?;
        let operation = Transaction {
            schema: 1,
            owner: current.owner,
            root: Identity::read(_pins.last().unwrap())?,
            slot,
            stage: stage_name,
            stage_id,
            old: Identity::read(old.root())?,
            old_hive: Identity {
                volume: current.hives.as_ref().unwrap()[index].volume,
                file: current.hives.as_ref().unwrap()[index].file,
            },
            new: Identity::read(&fresh.private)?,
            new_hive: {
                let file = pin_hive(&fresh.private_path.join("Registry/HKCU.dat"))?;
                let id = image_id(&file)?;
                Identity {
                    volume: id.VolumeSerialNumber,
                    file: id.FileId.Identifier,
                }
            },
        };
        drop(fresh);
        probe(Step::Prepared)?;
        write_json(&journal(&root, slot), &operation, false)?;
        Ok::<_, io::Error>(operation)
    })();
    let operation = match prepared {
        Ok(value) => value,
        Err(error) => {
            if !journal(&root, slot).try_exists()? {
                DeletionPlan::read(&stage)?.purge()?;
            }
            return Err(error);
        }
    };
    probe(Step::Journal)?;
    // NTFS refuses a directory rename while descendants are open, even with delete
    // sharing. Retain its exact root handle; repeat full pinned preflight before purge.
    let old_root = old.into_root();
    rename(&old_root, &stage.join("old")).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("Full clear old-directory cutover: {error}"),
        )
    })?;
    probe(Step::OldMoved)?;
    let new = deletion_directory(&stage.join("new"))?;
    rename(&new, &old_path).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("Full clear new-directory cutover: {error}"),
        )
    })?;
    drop(new);
    probe(Step::NewMoved)?;
    current.profiles[index] = Identity {
        volume: operation.new.volume,
        file: operation.new.file,
    };
    current.hives.as_mut().unwrap()[index] = Identity {
        volume: operation.new_hive.volume,
        file: operation.new_hive.file,
    };
    write_json(&root.join(CATALOG), &current, true)?;
    probe(Step::Catalog)?;
    let _new = validate_profile(&root, slot, &operation.new, &operation.new_hive)?;
    drop(old_root);
    let deletion = DeletionPlan::read(&stage.join("old"))?;
    probe(Step::PurgeReady)?;
    deletion.purge()?;
    finish(&root, &stage, &operation)?;
    Ok(())
}

fn finish(root: &Path, stage: &Path, operation: &Transaction) -> io::Result<()> {
    if let Some(directory) = matching(stage, &operation.stage_id)? {
        remove(directory)?;
    }
    fs::remove_file(journal(root, operation.slot))?;
    Ok(())
}

// Before catalog commit: restore the original. After commit: purge only its old identity.
// Errors leave the journal, which blocks this launcher while unrelated valid ones remain usable.
pub(in super::super) fn recover(root: &Path, slot: u32) -> io::Result<bool> {
    recover_mode(root, slot, true)
}
pub(super) fn rollback_uncommitted(root: &Path, slot: u32) -> io::Result<()> {
    recover_mode(root, slot, false).map(|_| ())
}
fn recover_mode(root: &Path, slot: u32, purge_committed: bool) -> io::Result<bool> {
    let Some(operation) = transaction(root, slot)? else {
        return Ok(false);
    };
    let _pins = pin_chain(root)?;
    let _profiles = open_directory(
        &root.join("Launchers"),
        FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY,
    )?;
    let stage = root.join("Launchers").join(&operation.stage);
    let _stage = matching(&stage, &operation.stage_id)?
        .ok_or_else(|| invalid("Full-clear staging directory unavailable"))?;
    let current = catalog(root)?;
    let index = slot as usize - 1;
    let target = profile(root, slot);
    let committed = same(&current.profiles[index], &operation.new)
        && same(&current.hives.as_ref().unwrap()[index], &operation.new_hive);
    if committed && !purge_committed {
        return Ok(true);
    }
    if committed {
        let _current = validate_profile(root, slot, &operation.new, &operation.new_hive)?;
        if let Some(old) = matching(&stage.join("old"), &operation.old)? {
            drop(old);
            DeletionPlan::read(&stage.join("old"))?.purge()?;
        }
    } else if same(&current.profiles[index], &operation.old)
        && same(&current.hives.as_ref().unwrap()[index], &operation.old_hive)
    {
        match matching(&target, &operation.old) {
            Ok(Some(original)) => drop(original),
            Ok(None) => {}
            Err(_) => {
                let replacement = matching(&target, &operation.new)?
                    .ok_or_else(|| invalid("Unrecognized replacement launcher"))?;
                rename(&replacement, &stage.join("new"))?;
            }
        }
        if let Some(old) = matching(&stage.join("old"), &operation.old)? {
            rename(&old, &target)?;
        }
        let _original = validate_profile(root, slot, &operation.old, &operation.old_hive)?;
        if let Some(new) = matching(&stage.join("new"), &operation.new)? {
            drop(new);
            DeletionPlan::read(&stage.join("new"))?.purge()?;
        }
        if matching(&target, &operation.old)?.is_none() {
            return Err(invalid("Original launcher could not be restored"));
        }
    } else {
        return Err(invalid(
            "Full-clear catalog identities changed; data preserved",
        ));
    }
    drop(_stage);
    finish(root, &stage, &operation)?;
    Ok(committed)
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Fixture {
        root: PathBuf,
    }
    impl Fixture {
        fn new() -> Self {
            let root = fresh_root(&std::env::temp_dir()).unwrap();
            fs::create_dir(root.join("logical")).unwrap();
            let fixture = Self { root };
            initialize_at(&fixture.logical(), &fixture.data()).unwrap();
            fixture
        }
        fn data(&self) -> PathBuf {
            self.root.join("Data")
        }
        fn logical(&self) -> File {
            open_directory(&self.root.join("logical"), FILE_READ_ATTRIBUTES).unwrap()
        }
        fn stores(&self) -> Vec<SlotStorage> {
            recover_confirmed(&self.data());
            open_at(self.logical(), &self.data())
                .unwrap()
                .into_iter()
                .collect()
        }
        fn populate(&self, stores: &[SlotStorage]) {
            use windows_sys::Win32::System::Registry::*;
            for store in stores {
                fs::write(
                    store.private_path.join("Roaming/world-fixture"),
                    b"synthetic world",
                )
                .unwrap();
                fs::write(
                    store.private_path.join("Local/account-fixture"),
                    b"synthetic account",
                )
                .unwrap();
                let label = wide(OsStr::new("SyntheticAccountFixture")).unwrap();
                assert_eq!(
                    unsafe {
                        RegSetValueExW(
                            store.registry.0,
                            label.as_ptr(),
                            0,
                            REG_BINARY,
                            b"synthetic".as_ptr(),
                            9,
                        )
                    },
                    0
                );
            }
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            assert_eq!(self.root.parent(), Some(std::env::temp_dir().as_path()));
            fs::remove_dir_all(&self.root).unwrap();
        }
    }
    #[test]
    fn full_clear_replaces_only_selected_file_backed_profile_without_setup() {
        use windows_sys::Win32::System::Registry::*;
        let fixture = Fixture::new();
        let mut stores = fixture.stores();
        fixture.populate(&stores);
        let before = catalog(&fixture.data()).unwrap();
        let selected = stores.remove(3);
        execute(selected).unwrap();
        let after = catalog(&fixture.data()).unwrap();
        for index in 0..5 {
            assert_eq!(
                same(&before.profiles[index], &after.profiles[index]),
                index != 3
            );
            assert_eq!(
                same(
                    &before.hives.as_ref().unwrap()[index],
                    &after.hives.as_ref().unwrap()[index]
                ),
                index != 3
            );
        }
        let reopened = fixture.stores();
        for store in &reopened {
            let label = wide(OsStr::new("SyntheticAccountFixture")).unwrap();
            let present = unsafe {
                RegQueryValueExW(
                    store.registry.0,
                    label.as_ptr(),
                    ptr::null(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                )
            } == 0;
            assert_eq!(present, store.slot != 4);
            assert_eq!(
                store.private_path.join("Roaming/world-fixture").exists(),
                store.slot != 4
            );
            assert_eq!(
                store.private_path.join("Local/account-fixture").exists(),
                store.slot != 4
            );
            assert!(!pending(store).unwrap());
        }
        assert!(!journal(&fixture.data(), 4).exists());
        assert_eq!(fs::read_dir(fixture.data().join("Launchers")).unwrap().count(),5,"Old or staging profile remained after success");
        drop(reopened);
        drop(stores);
    }
    #[test]
    fn full_clear_recovers_each_cutover_boundary_by_catalog_identity() {
        for boundary in [
            Step::Prepared,
            Step::Journal,
            Step::OldMoved,
            Step::NewMoved,
            Step::Catalog,
        ] {
            let fixture = Fixture::new();
            let mut stores = fixture.stores();
            fixture.populate(&stores);
            let selected = stores.remove(1);
            let mut reached = false;
            let result = execute_probed(selected, |step| {
                if step == boundary {
                    reached = true;
                    Err(invalid("Synthetic interruption"))
                } else {
                    Ok(())
                }
            });
            assert!(result.is_err());
            assert!(
                reached,
                "Cutover did not reach the requested boundary: {:?}",
                result.err()
            );
            drop(stores);
            if boundary == Step::Catalog {
                recover(&fixture.data(), 2).unwrap();
            }
            let reopened = fixture.stores();
            assert_eq!(
                reopened[1]
                    .private_path
                    .join("Roaming/world-fixture")
                    .exists(),
                boundary != Step::Catalog
            );
            assert!(
                !pending(&reopened[1]).unwrap(),
                "Recovery did not finish its recorded identity"
            );
            assert_eq!(fs::read_dir(fixture.data().join("Launchers")).unwrap().count(),5,"Interrupted transaction left duplicate profiles");
            drop(reopened);
        }
    }
    #[test]
    fn full_clear_rejects_locked_and_hardlinked_payload_before_cutover() {
        let fixture = Fixture::new();
        let mut stores = fixture.stores();
        fixture.populate(&stores);
        let selected = stores.remove(2);
        let payload = selected.private_path.join("Roaming/world-fixture");
        let locked = OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .open(&payload)
            .unwrap();
        assert!(execute(selected).is_err());
        assert!(payload.exists());
        assert!(!journal(&fixture.data(), 3).exists());
        drop(locked);
        drop(stores);
        let mut stores = fixture.stores();
        fs::hard_link(&payload, fixture.root.join("outside-link")).unwrap();
        assert!(execute(stores.remove(2)).is_err());
        assert_eq!(
            fs::read(fixture.root.join("outside-link")).unwrap(),
            b"synthetic world"
        );
        assert!(!journal(&fixture.data(), 3).exists());
        drop(stores);
    }
    #[test]
    fn full_clear_pending_purge_blocks_selected_generation_and_resumes_only_old_data() {
        let fixture = Fixture::new();
        let mut stores = fixture.stores();
        fixture.populate(&stores);
        let selected = stores.remove(0);
        let old_token = confirmation(&selected).unwrap();
        let mut late_lock = None;
        let result = execute_probed(selected, |step| {
            if step == Step::PurgeReady {
                let operation = transaction(&fixture.data(), 1)?.unwrap();
                let path = fixture
                    .data()
                    .join("Launchers")
                    .join(operation.stage)
                    .join("old/late-file");
                fs::write(&path, b"synthetic delayed file")?;
                late_lock = Some(
                    OpenOptions::new()
                        .read(true)
                        .share_mode(FILE_SHARE_READ)
                        .open(path)?,
                );
            }
            Ok(())
        });
        assert!(result.is_err());
        drop(stores);
        let mut reopened = fixture.stores();
        assert!(pending(&reopened[0]).unwrap());
        assert!(inspect(&reopened[0], Ok(())).pending);
        assert!(!inspect(&reopened[0], Err(invalid("Running launcher"))).can_clear);
        let current_token = confirmation(&reopened[0]).unwrap();
        assert_ne!(old_token, current_token);
        assert!(run(reopened.remove(0), &old_token).is_err());
        drop(reopened);
        drop(late_lock);
        let mut reopened = fixture.stores();
        assert!(
            pending(&reopened[0]).unwrap(),
            "Startup silently resumed a large committed purge"
        );
        fs::write(
            reopened[0]
                .private_path
                .join("Roaming/new-profile-preference"),
            b"new generation",
        )
        .unwrap();
        run(reopened.remove(0), &current_token).unwrap();
        drop(reopened);
        let reopened = fixture.stores();
        assert!(!pending(&reopened[0]).unwrap());
        assert!(
            reopened[0]
                .private_path
                .join("Roaming/new-profile-preference")
                .exists(),
            "Finish clear deleted the new profile generation"
        );
        assert!(!reopened[0]
            .private_path
            .join("Roaming/world-fixture")
            .exists());
        for store in &reopened[1..] {
            assert!(store.private_path.join("Roaming/world-fixture").exists());
        }
        drop(reopened);
    }
    #[test]
    fn full_clear_refuses_junctions_and_malformed_recovery_metadata() {
        let fixture = Fixture::new();
        let mut stores = fixture.stores();
        fixture.populate(&stores);
        let selected = stores.remove(4);
        let link = selected.private_path.join("Roaming/outside-link");
        let outside = fixture.root.join("outside");
        fs::create_dir(&outside).unwrap();
        fs::write(outside.join("keep"), b"untouched").unwrap();
        let output = std::process::Command::new("cmd.exe")
            .args(["/c", "mklink", "/J"])
            .arg(&link)
            .arg(&outside)
            .output()
            .unwrap();
        assert!(output.status.success());
        assert!(execute(selected).is_err());
        assert_eq!(fs::read(outside.join("keep")).unwrap(), b"untouched");
        fs::remove_dir(link).unwrap();
        drop(stores);
        fs::write(journal(&fixture.data(), 5), b"invalid transaction").unwrap();
        assert!(recover(&fixture.data(), 5).is_err());
        let mut stores = fixture.stores();
        assert!(pending(&stores[4]).is_err());
        assert!(run(stores.remove(4), &"0".repeat(64)).is_err());
        for store in &stores {
            assert!(store.private_path.join("Roaming/world-fixture").exists());
        }
        drop(stores);
    }
    #[test]
    fn full_clear_readonly_inspection_never_resumes_deletion() {
        let fixture = Fixture::new();
        let mut stores = fixture.stores();
        fixture.populate(&stores);
        assert!(
            execute_probed(stores.remove(0), |step| if step == Step::Catalog {
                Err(invalid("Synthetic interruption"))
            } else {
                Ok(())
            })
            .is_err()
        );
        drop(stores);
        let operation = transaction(&fixture.data(), 1).unwrap().unwrap();
        let old = fixture
            .data()
            .join("Launchers")
            .join(operation.stage)
            .join("old/Roaming/world-fixture");
        let before = fs::read(journal(&fixture.data(), 1)).unwrap();
        let inspected = open_at(fixture.logical(), &fixture.data()).unwrap();
        assert!(pending(&inspected[0]).unwrap());
        assert!(old.exists(), "Read-only opening removed old data");
        assert_eq!(fs::read(journal(&fixture.data(), 1)).unwrap(), before);
        drop(inspected);
        recover(&fixture.data(), 1).unwrap();
        drop(fixture.stores());
        assert!(!old.exists());
    }
    #[test]
    fn full_clear_deletion_handles_readonly_payloads_without_reading_contents() {
        let fixture = Fixture::new();
        let path = fixture.root.join("readonly-fixture");
        fs::create_dir(&path).unwrap();
        let file = path.join("readonly-file");
        fs::write(&file, b"synthetic").unwrap();
        let mut permissions = fs::metadata(&file).unwrap().permissions();
        permissions.set_readonly(true);
        fs::set_permissions(&file, permissions).unwrap();
        let result = DeletionPlan::read(&path).and_then(DeletionPlan::purge);
        if file.exists() {
            let mut permissions = fs::metadata(&file).unwrap().permissions();
            permissions.set_readonly(false);
            fs::set_permissions(&file, permissions).unwrap();
        }
        assert!(
            result.is_ok(),
            "Read-only fixture cleanup failed: {:?}",
            result.err()
        );
        assert!(!path.exists());
    }
}
