//! Bounded profile size and disk observation, plus handle-based cleanup of one proven HTTP cache.
use super::{directory_path, image_id, invalid, open_directory, wide};
use crate::engine::native_clients_session::{CacheStorage, DiskSpace, ProfileStorage};
use std::{
    fs::{self, File, OpenOptions},
    io, mem,
    os::windows::{fs::{MetadataExt, OpenOptionsExt}, io::AsRawHandle},
    path::Path,
    time::Duration,
};
use virtual_clients_core::InspectionBudget;
use windows_sys::Win32::Storage::FileSystem::*;

// Exact observed Chromium profile. Never classify Default itself, Cookies, Local
// Storage, auth helpers, worlds, other TEMP contents or arbitrary "Cache" names.
const CACHE_PARTS: [&str; 6] = [
    "Local",
    "Temp",
    "minecraftpe",
    "codeBuilderV1",
    "Default",
    "Cache",
];
const MAX_FILES: usize = 2048;
const MAX_DIRECTORIES: usize = 256;

// STG-08: Windows FILETIME uses 100 ns ticks. Retain at least seven full
// days; future, missing and exactly-at-cutoff timestamps are not eligible.
const LOG_RETENTION_TICKS: u64 = 7 * 24 * 60 * 60 * 10_000_000;
fn expired_log(last_write: u64, now: u64) -> bool {
    last_write != 0 && now.checked_sub(last_write)
        .is_some_and(|age| age > LOG_RETENTION_TICKS)
}

// Internal candidate policy. The root and file class must come
// from a native-reviewed producer contract, never a UI path or extension
// search. A .log suffix is NOT sufficient to identify a producer's logs.
#[allow(dead_code)]
fn plan_expired_logs(root: &File, now: u64, budget: InspectionBudget, deleting: bool,
    is_owned_log: impl Fn(&std::ffi::OsStr) -> bool) -> io::Result<ExpiredLogPlan> {
    super::directory_id(root)?;
    let path = directory_path(root)?;
    let root = root.try_clone()?;
    let mut candidates = Vec::new();
    for (index, entry) in fs::read_dir(&path)?.enumerate() {
        if index >= MAX_FILES || budget.exhausted() {
            return Err(invalid("Log retention inspection limit reached"));
        }
        let entry = entry?;
        if !is_owned_log(&entry.file_name()) { continue; }
        let meta = fs::symlink_metadata(entry.path())?;
        if !meta.is_file() || meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(invalid("Unsupported log candidate"));
        }
        // A fresh file need not be opened for retention. Never rely on this
        // path metadata to admit an old candidate; check its handle below.
        if !expired_log(meta.last_write_time(), now) { continue; }
        // FILE_READ_DATA participates in Windows sharing checks; attributes
        // alone do not. Mutation adds DELETE and keeps exclusive ownership.
        let file = OpenOptions::new().access_mode(FILE_READ_ATTRIBUTES | FILE_READ_DATA | if deleting { DELETE } else { 0 })
            .share_mode(0).custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(entry.path())?;
        image_id(&file)?;
        let mut standard: FILE_STANDARD_INFO = unsafe { mem::zeroed() };
        if unsafe { GetFileInformationByHandleEx(file.as_raw_handle() as _,
            FileStandardInfo, &mut standard as *mut _ as _,
            mem::size_of_val(&standard) as u32) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let metadata = file.metadata()?;
        if standard.NumberOfLinks != 1 || standard.Directory || standard.DeletePending ||
            !metadata.is_file() || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(invalid("Unsupported or linked log candidate"));
        }
        if expired_log(metadata.last_write_time(), now) {
            candidates.push(file);
        }
    }
    if budget.exhausted() { return Err(invalid("Log retention deadline exceeded")); }
    Ok(ExpiredLogPlan { _root: root, candidates, budget, deleting })
}

#[allow(dead_code)]
struct ExpiredLogPlan {
    _root: File,
    candidates: Vec<File>,
    budget: InspectionBudget,
    deleting: bool,
}
#[allow(dead_code)]
impl ExpiredLogPlan {
    // Only a complete mutating plan is disposable. The caller must first
    // verify producer identity, root and native idle admission. No public
    // path, new file open or retry is accepted here.
    fn dispose(self) -> io::Result<()> {
        if !self.deleting || self.budget.exhausted() {
            return Err(invalid("Log disposition plan is not admitted"));
        }
        for file in self.candidates {
            mark_for_delete(&file).map_err(|_|io::Error::other("Log cleanup stopped; selected old logs may be partially removed. Refresh before retrying."))?;
        }
        Ok(())
    }
}


pub(super) fn scan_profile_size(path: &Path, result: &mut ProfileStorage, budget: InspectionBudget) {
    scan_size(path, result, budget, 0);
}

// Metadata only, bounded, and no traversal through junctions/symlinks. Not a cache classifier.
fn scan_size(path: &Path, result: &mut ProfileStorage, budget: InspectionBudget, depth: usize) {
    if depth > 48 || result.files >= 100_000 || budget.exhausted() {
        result.partial = true;
        return;
    }
    let Ok(_pin) = open_directory(path, FILE_READ_ATTRIBUTES) else {
        result.partial = true;
        return;
    };
    let Ok(entries) = std::fs::read_dir(path) else {
        result.partial = true;
        return;
    };
    for entry in entries {
        if result.files >= 100_000 || budget.exhausted() {
            result.partial = true;
            break;
        }
        let Ok(entry) = entry else {
            result.partial = true;
            continue;
        };
        let Ok(metadata) = std::fs::symlink_metadata(entry.path()) else {
            result.partial = true;
            continue;
        };
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            result.partial = true;
            continue;
        }
        if metadata.is_dir() {
            scan_size(&entry.path(), result, budget, depth + 1);
        } else if metadata.is_file() {
            result.bytes = result.bytes.saturating_add(metadata.len());
            result.files += 1;
        }
    }
    // The last file observation may cross the budget without another loop
    // iteration. Such a five-profile size estimate remains incomplete.
    if budget.exhausted() { result.partial = true; }
}

pub(super) fn disk(private: &File) -> io::Result<DiskSpace> {
    let path = directory_path(private)?;
    let path = wide(path.as_os_str())?;
    let (mut available, mut total) = (0u64, 0u64);
    if unsafe {
        GetDiskFreeSpaceExW(
            path.as_ptr(),
            &mut available,
            &mut total,
            std::ptr::null_mut(),
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    if total == 0 || available > total {
        return Err(invalid("Unsupported disk space metadata"));
    }
    Ok(DiskSpace {
        available_bytes: available,
        total_bytes: total,
    })
}

// Shared exclusive bounds: read-only review must not promise a selection
// that the aggregate mutation preflight would necessarily refuse.
fn within_tree_limits(files: usize, directories: usize, entries: usize) -> bool {
    files < MAX_FILES && directories < MAX_DIRECTORIES && entries < MAX_FILES + MAX_DIRECTORIES
}

#[derive(Default)]
pub(super) struct SelectedTreeExtent {
    pub(super) bytes: u64,
    pub(super) files: u32,
    directories: usize,
    entries: usize,
}
impl SelectedTreeExtent {
    pub(super) fn accumulate(&mut self, next: Self) -> io::Result<()> {
        let bytes = self.bytes.checked_add(next.bytes).ok_or_else(|| invalid("Selection size overflow"))?;
        let files = self.files.checked_add(next.files).ok_or_else(|| invalid("Selection file count overflow"))?;
        let directories = self.directories.checked_add(next.directories).ok_or_else(|| invalid("Selection directory count overflow"))?;
        let entries = self.entries.checked_add(next.entries).ok_or_else(|| invalid("Selection entry count overflow"))?;
        if !within_tree_limits(files as usize, directories, entries) {
            return Err(invalid("Selection exceeds aggregate tree inspection limits"));
        }
        *self = Self { bytes, files, directories, entries };
        Ok(())
    }
}

struct CachePlan {
    directories: Vec<File>,
    files: Vec<File>,
    bytes: u64,
    entries: usize,
    budget: InspectionBudget,
    deleting: bool,
    selected_tree: bool,
    tree_records: Vec<Vec<u8>>,
}
impl CachePlan {
    fn new(deleting: bool, budget: InspectionBudget) -> Self {
        Self {
            directories: Vec::new(),
            files: Vec::new(),
            bytes: 0,
            entries: 0,
            budget,
            deleting,
            selected_tree: false,
            tree_records: Vec::new(),
        }
    }
    fn bounded(&self, depth: usize) -> io::Result<()> {
        if depth > 8
            || !within_tree_limits(self.files.len(), self.directories.len(), self.entries)
            || self.budget.exhausted()
        {
            return Err(invalid(
                "Cache inspection limit reached; nothing was deleted",
            ));
        }
        Ok(())
    }
    fn visit(&mut self, path: &Path, depth: usize) -> io::Result<()> {
        self.visit_tree(path, path, depth)
    }
    fn visit_tree(&mut self, path: &Path, root: &Path, depth: usize) -> io::Result<()> {
        self.bounded(depth)?;
        let directory = if self.selected_tree && self.deleting {
            // Directory handles need DELETE access for a later bottom-up
            // disposition, but must still allow read_dir's read access.
            // No write sharing and no delete sharing for other handles.
            let pin = OpenOptions::new()
                .access_mode(FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY | DELETE)
                .share_mode(FILE_SHARE_READ)
                .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
                .open(path)?;
            super::directory_id(&pin)?;
            directory_path(&pin)?;
            pin
        } else {
            open_directory(path, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY)?
        };
        if self.selected_tree {
            self.record_tree_entry(path, root, &directory, true)?;
        }
        self.directories.push(directory);
        for entry in fs::read_dir(path)? {
            self.entries += 1;
            self.bounded(depth)?;
            let path = entry?.path();
            let metadata = fs::symlink_metadata(&path)?;
            if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
                return Err(invalid("Linked cache entry; nothing was deleted"));
            }
            if metadata.is_dir() {
                self.visit_tree(&path, root, depth + 1)?;
            } else {
                // Open the exact leaf without following a reparse. Cleanup takes exclusive
                // access to every candidate before marking even one for deletion.
                let file = OpenOptions::new()
                    .access_mode(FILE_READ_ATTRIBUTES | if self.deleting { DELETE } else { 0 })
                    .share_mode(if self.deleting {
                        0
                    } else {
                        FILE_SHARE_READ | FILE_SHARE_WRITE
                    })
                    .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
                    .open(&path)?;
                image_id(&file)?;
                let mut standard: FILE_STANDARD_INFO = unsafe { mem::zeroed() };
                if unsafe {
                    GetFileInformationByHandleEx(
                        file.as_raw_handle() as _,
                        FileStandardInfo,
                        &mut standard as *mut _ as _,
                        mem::size_of_val(&standard) as u32,
                    )
                } == 0
                {
                    return Err(io::Error::last_os_error());
                }
                if standard.NumberOfLinks != 1
                    || standard.Directory
                    || standard.DeletePending
                    || standard.EndOfFile < 0
                {
                    return Err(invalid(
                        "Unsupported or linked cache file; nothing was deleted",
                    ));
                }
                self.bytes = self
                    .bytes
                    .checked_add(standard.EndOfFile as u64)
                    .ok_or_else(|| invalid("Cache size overflow"))?;
                if self.selected_tree { self.record_tree_entry(&path, root, &file, false)?; }
                self.files.push(file);
            }
        }
        Ok(())
    }

    fn record_tree_entry(&mut self, path: &Path, root: &Path, file: &File, directory: bool) -> io::Result<()> {
        use std::os::windows::ffi::OsStrExt;
        let relative = path.strip_prefix(root).map_err(|_| invalid("Selected tree path escaped root"))?;
        let mut record = Vec::new();
        record.push(if directory { 1 } else { 2 });
        let name: Vec<u16> = relative.as_os_str().encode_wide().collect();
        record.extend_from_slice(&(name.len() as u64).to_le_bytes());
        for unit in name { record.extend_from_slice(&unit.to_le_bytes()); }
        let id = if directory { super::directory_id(file)? } else { image_id(file)? };
        record.extend_from_slice(&id.VolumeSerialNumber.to_le_bytes());
        record.extend_from_slice(&id.FileId.Identifier);
        // Directory timestamps can change on unrelated enumeration; file
        // size and last-write metadata detect ordinary same-size edits.
        if !directory {
            let metadata = file.metadata()?;
            record.extend_from_slice(&metadata.len().to_le_bytes());
            record.extend_from_slice(&metadata.last_write_time().to_le_bytes());
        }
        self.tree_records.push(record);
        Ok(())
    }

    fn tree_digest(&mut self, start: usize) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let records = &mut self.tree_records[start..];
        records.sort();
        let mut hash = Sha256::new();
        hash.update(b"LazyTester/selected-tree/v1");
        for record in records {
            hash.update((record.len() as u64).to_le_bytes());
            hash.update(record);
        }
        hash.finalize().into()
    }
}

// Reuse the existing bounded handle traversal for a read-only selected tree.
// The caller must retain the verified ancestor chain and exact item pin.
// Never pass user-supplied paths directly; this does not grant DELETE access.
pub(super) fn inspect_selected_tree(path: &Path, expected: &FILE_ID_INFO,
    budget: InspectionBudget) -> io::Result<(SelectedTreeExtent, [u8; 32])> {
    let pin = open_directory(path, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY)?;
    if !super::same_image(expected, &super::directory_id(&pin)?) {
        return Err(invalid("Selected directory changed before tree inspection"));
    }
    let mut plan = CachePlan::new(false, budget);
    plan.selected_tree = true;
    plan.visit(path, 0)?;
    plan.bounded(0)?;
    let digest = plan.tree_digest(0);
    Ok((SelectedTreeExtent { bytes: plan.bytes, files: plan.files.len() as u32,
        directories: plan.directories.len(), entries: plan.entries }, digest))
}

// Exclusive selected-tree preflight. Retain category and profile
// ancestor pins separately from deletion candidates. No disposition occurs
// here, including when this plan is dropped after an error.
pub(super) struct SelectedTreePlan {
    _ancestors: Vec<File>,
    plan: CachePlan,
}
impl SelectedTreePlan {
    pub(super) fn totals(&self) -> (u64, u32) {
        (self.plan.bytes, self.plan.files.len() as u32)
    }

    // Consume only handles admitted by the complete preflight. No reopening
    // paths, recursive removal, attribute overrides or fallback on failure.
    // Kept internal until the controller's confirmation admission is wired.
    pub(super) fn dispose(self) -> io::Result<()> {
        let Self { _ancestors, mut plan } = self;
        plan.bounded(0)?;
        if !plan.deleting || !plan.selected_tree || plan.directories.is_empty() {
            return Err(invalid("Invalid selected-world deletion plan"));
        }
        // FILE_DISPOSITION_INFO removes on handle close. Close each leaf
        // before attempting any directory, then consume directories in
        // reverse preorder so children close before parents.
        // Any failure after the first disposition may leave a partial result.
        let partial = || io::Error::other("Selected data cleanup stopped; selected data may be partially removed. Refresh inventory before retrying.");
        for file in plan.files.drain(..) {
            mark_for_delete(&file).map_err(|_| partial())?;
            drop(file);
        }
        while let Some(directory) = plan.directories.pop() {
            mark_for_delete(&directory).map_err(|_| partial())?;
            drop(directory);
        }
        // Keep non-target ancestors alive until every disposition finishes.
        drop(_ancestors);
        Ok(())
    }
}

// Shared Win32 disposition primitive. It cannot grant DELETE access or
// traverse a path. The caller owns scope, preflight and error reporting.
fn mark_for_delete(file: &File) -> io::Result<()> {
    let info = FILE_DISPOSITION_INFO { DeleteFile: true };
    // SAFETY: fixed-size disposition structure and a live retained handle.
    if unsafe {
        SetFileInformationByHandle(
            file.as_raw_handle() as _, FileDispositionInfo,
            &info as *const _ as _, mem::size_of_val(&info) as u32,
        )
    } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

pub(super) fn preflight_selected_worlds(private: &File,
    review: &super::resource_pack_inventory::SelectiveReview,
    budget: InspectionBudget) -> io::Result<SelectedTreePlan> {
    use super::resource_pack_inventory::InventoryCategory;
    if review.items.iter().any(|item| item.category != InventoryCategory::Worlds) {
        return Err(invalid("World cleanup cannot target packs"));
    }
    preflight_selected_trees(private, review, budget)
}

pub(super) fn preflight_selected_packs(private: &File,
    review: &super::resource_pack_inventory::SelectiveReview,
    budget: InspectionBudget) -> io::Result<SelectedTreePlan> {
    use super::resource_pack_inventory::InventoryCategory;
    if review.items.iter().any(|item| item.category == InventoryCategory::Worlds) {
        return Err(invalid("Pack cleanup cannot target worlds"));
    }
    preflight_selected_trees(private, review, budget)
}

// Shared root-bound planner. Not an admission API: pack review remains blocked
// by unverified reference coverage; callers must also enforce native gates.
fn preflight_selected_trees(private: &File,
    review: &super::resource_pack_inventory::SelectiveReview,
    budget: InspectionBudget) -> io::Result<SelectedTreePlan> {
    use super::resource_pack_inventory::InventoryCategory;
    if review.items.is_empty() || !review.warnings.is_empty() ||
        review.items.iter().any(|item| !match item.category {
            InventoryCategory::Worlds => item.root == "minecraftWorlds",
            InventoryCategory::ResourcePacks => matches!(item.root,"resource_packs"|"development_resource_packs"),
            InventoryCategory::BehaviorPacks => matches!(item.root,"behavior_packs"|"development_behavior_packs"),
        }) ||
        !super::same_image(&review.profile_identity, &super::directory_id(private)?) {
        return Err(invalid("Selected world review is no longer eligible"));
    }
    let mut plan = CachePlan::new(true, budget);
    plan.selected_tree = true;
    plan.bounded(0)?;
    let mut path = directory_path(private)?;
    let mut ancestors = vec![private.try_clone()?];
    for part in ["Roaming", "Minecraft Education Edition", "games", "com.mojang"] {
        plan.bounded(0)?;
        path.push(part);
        ancestors.push(open_directory(&path, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY)?);
    }
    if review.tree_digests.len() != review.items.len() { return Err(invalid("Incomplete selected tree review")); }
    let mut roots = std::collections::HashSet::new();
    for (item, expected_digest) in review.items.iter().zip(&review.tree_digests) {
        plan.bounded(0)?;
        let root = path.join(item.root);
        if roots.insert(item.root) {
            ancestors.push(open_directory(&root, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY)?);
        }
        // Names came from the reviewed inventory; refuse non-single components
        // defensively rather than allowing any caller to expand the scope.
        let mut components = Path::new(&item.name).components();
        if !matches!(components.next(), Some(std::path::Component::Normal(_))) ||
            components.next().is_some() || item.name.contains(':') {
            return Err(invalid("Unsupported selected item name"));
        }
        let before = plan.directories.len();
        let record_start = plan.tree_records.len();
        plan.visit(&root.join(&item.name), 0)?;
        if !super::same_image(&item.identity, &super::directory_id(&plan.directories[before])?) {
            return Err(invalid("Selected item changed before preflight"));
        }
        if plan.tree_digest(record_start) != *expected_digest {
            return Err(invalid("Selected item tree changed; review again"));
        }
    }
    plan.bounded(0)?;
    if (plan.bytes, plan.files.len() as u32) != (review.bytes, review.files) {
        return Err(invalid("Selected item totals changed; review again"));
    }
    Ok(SelectedTreePlan { _ancestors: ancestors, plan })
}

#[derive(Clone, Copy)]
pub(super) enum CacheTarget { Web, Download }

impl CacheTarget {
    fn parts(self) -> io::Result<&'static [&'static str]> {
        match self {
            Self::Web => Ok(&CACHE_PARTS),
            // No native-reviewed download producer/root in the current source.
            // Do not substitute app Data/Cache, the HTTP cache, or pack roots.
            Self::Download => Err(io::Error::new(io::ErrorKind::Unsupported,
                "Minecraft download cache location is not verified")),
        }
    }
}

fn plan(private: &File, deleting: bool, budget: InspectionBudget) -> io::Result<CachePlan> {
    plan_cache(private, CacheTarget::Web, deleting, budget)
}

fn plan_cache(private: &File, target: CacheTarget, deleting: bool, budget: InspectionBudget) -> io::Result<CachePlan> {
    // Resolve the compiled allowlist before touching the filesystem.
    let parts = target.parts()?;
    if parts.is_empty() { return Err(invalid("Cache root is empty")); }
    for part in parts {
        let mut components = Path::new(part).components();
        if !matches!(components.next(), Some(std::path::Component::Normal(_))) ||
            components.next().is_some() || part.contains(':') {
            return Err(invalid("Unsupported cache root component"));
        }
    }
    let mut plan = CachePlan::new(deleting, budget);
    // Both observation and deletion preflight must finish within their budget.
    plan.bounded(0)?;
    let mut path = directory_path(private)?;
    plan.directories.push(private.try_clone()?);
    for part in parts {
        plan.bounded(0)?;
        path.push(part);
        match open_directory(&path, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY) {
            Ok(pin) => plan.directories.push(pin),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                plan.bounded(0)?;
                return Ok(plan);
            }
            Err(error) => return Err(error),
        }
    }
    plan.visit(&path, 0)?;
    // A final OS call may consume the budget. Refuse incomplete preflight
    // before marking any file for deletion, even on the mutating path.
    plan.bounded(0)?;
    Ok(plan)
}

pub(super) fn inspect(private: &File, inactive: bool, budget: InspectionBudget) -> CacheStorage {
    inspect_cache(private, CacheTarget::Web, inactive, budget)
}

pub(super) fn inspect_cache(private: &File, target: CacheTarget, inactive: bool, budget: InspectionBudget) -> CacheStorage {
    if let Err(error) = target.parts() {
        return CacheStorage { bytes: 0, files: 0, partial: true, can_clear: false, issue: Some(error.to_string()) };
    }
    match plan_cache(private, target, false, budget) {
        Ok(plan) => CacheStorage {
            bytes: plan.bytes,
            files: plan.files.len() as u32,
            partial: false,
            can_clear: inactive && !plan.files.is_empty(),
            issue: if inactive {
                None
            } else {
                Some("Close this Minecraft session before clearing its cache.".into())
            },
        },
        Err(_) => CacheStorage {
            bytes: 0,
            files: 0,
            partial: true,
            can_clear: false,
            issue: Some(
                "Cache inspection was incomplete; cleanup is unavailable.".into(),
            ),
        },
    }
}

pub(super) fn clear(private: &File, inactive: bool) -> io::Result<()> {
    clear_cache_target(private, CacheTarget::Web, inactive)
}

pub(super) fn clear_cache_target(private: &File, target: CacheTarget, inactive: bool) -> io::Result<()> {
    target.parts()?;
    if !inactive {
        return Err(invalid(
            "Close this Minecraft session normally before clearing its cache",
        ));
    }
    let plan = plan_cache(private, target, true, InspectionBudget::new(Duration::from_secs(2)))?;
    // No path-based delete or recursive directory removal. Retain ancestor pins
    // and exclusive file handles until all dispositions have been attempted.
    for file in &plan.files {
        if mark_for_delete(file).is_err() {
            return Err(io::Error::other("Cache cleanup stopped; some cache files may have been removed. Account and world data were not targeted."));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    #[test]
    fn retention_keeps_recent_boundary_future_and_unknown_timestamps() {
        let now = LOG_RETENTION_TICKS + 100;
        assert!(expired_log(99, now));
        assert!(!expired_log(100, now));
        assert!(!expired_log(101, now));
        assert!(!expired_log(0, now));
        assert!(!expired_log(now + 1, now));
        assert!(!expired_log(1, 0));
    }
    fn inspect_fixture(private: &File, inactive: bool) -> CacheStorage {
        inspect(private, inactive, InspectionBudget::new(Duration::from_secs(2)))
    }
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "lazy-cache-fixture-{}-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn cache(&self) -> PathBuf {
            CACHE_PARTS
                .iter()
                .fold(self.0.clone(), |p, part| p.join(part))
        }
        fn pin(&self) -> File {
            open_directory(&self.0, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY).unwrap()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            assert_eq!(self.0.parent(), Some(std::env::temp_dir().as_path()));
            fs::remove_dir_all(&self.0).unwrap();
        }
    }
    #[test]
    fn unresolved_download_cache_never_reuses_web_cache_or_touches_profile() {
        let fixture = Fixture::new();
        let cache = fixture.cache();
        fs::create_dir_all(&cache).unwrap();
        fs::write(cache.join("keep"), b"preserve").unwrap();
        let pin = fixture.pin();
        let report = inspect_cache(&pin, CacheTarget::Download, true, InspectionBudget::new(Duration::from_secs(2)));
        assert!(report.partial && !report.can_clear && report.issue.is_some());
        assert_eq!(clear_cache_target(&pin, CacheTarget::Download, true).unwrap_err().kind(), io::ErrorKind::Unsupported);
        assert_eq!(fs::read(cache.join("keep")).unwrap(), b"preserve");
    }
    #[test]
    fn clears_only_http_cache_preserving_auth_world_settings_and_other_temp() {
        let fixture = Fixture::new();
        let other = Fixture::new();
        fs::create_dir_all(other.cache()).unwrap();
        fs::write(other.cache().join("other-profile-cache"), b"other profile").unwrap();
        let other_lock = OpenOptions::new().read(true).share_mode(FILE_SHARE_READ)
            .open(other.cache().join("other-profile-cache")).unwrap();
        let cache = fixture.cache();
        fs::create_dir_all(cache.join("Cache_Data/index-dir")).unwrap();
        fs::write(cache.join("Cache_Data/a"), [0u8; 17]).unwrap();
        fs::write(cache.join("Cache_Data/index-dir/index"), [0u8; 23]).unwrap();
        let protected = [
            "Local/Temp/minecraftpe/codeBuilderV1/Default/Cookies",
            "Local/Temp/important.tmp",
            "Roaming/Minecraft Education Edition/games/com.mojang/minecraftWorlds/test/level.dat",
            "Roaming/Minecraft Education Edition/games/com.mojang/minecraftpe/options.txt",
        ];
        for name in protected {
            let path = fixture.0.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, b"preserve").unwrap();
        }
        let pin = fixture.pin();
        assert_eq!(
            (inspect_fixture(&pin, true).bytes, inspect_fixture(&pin, true).files),
            (40, 2)
        );
        assert!(!inspect_fixture(&pin, false).can_clear);
        assert!(clear(&pin, false).is_err());
        assert!(cache.join("Cache_Data/a").exists());
        let space = disk(&pin).unwrap();
        assert!(space.total_bytes > 0 && space.available_bytes <= space.total_bytes);
        clear(&pin, true).unwrap();
        assert_eq!(fs::read(other.cache().join("other-profile-cache")).unwrap(), b"other profile");
        drop(other_lock);
        assert_eq!(inspect_fixture(&pin, true).files, 0);
        for name in protected {
            assert_eq!(fs::read(fixture.0.join(name)).unwrap(), b"preserve");
        }
    }
    #[test]
    fn locked_or_hardlinked_cache_file_refuses_before_any_deletion() {
        let fixture = Fixture::new();
        let cache = fixture.cache();
        fs::create_dir_all(&cache).unwrap();
        fs::write(cache.join("a"), b"cache").unwrap();
        fs::write(cache.join("b"), b"locked").unwrap();
        let pin = fixture.pin();
        let lock = OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .open(cache.join("b"))
            .unwrap();
        assert!(clear(&pin, true).is_err());
        assert!(cache.join("a").exists());
        drop(lock);
        fs::hard_link(cache.join("b"), fixture.0.join("outside")).unwrap();
        assert!(!inspect_fixture(&pin, true).can_clear);
        assert!(clear(&pin, true).is_err());
        assert!(cache.join("a").exists());
        assert_eq!(fs::read(fixture.0.join("outside")).unwrap(), b"locked");
    }
    #[test]
    fn missing_cache_is_empty_and_reparse_ancestor_is_refused() {
        let fixture = Fixture::new();
        let pin = fixture.pin();
        assert_eq!(inspect_fixture(&pin, true).bytes, 0);
        clear(&pin, true).unwrap();
        fs::create_dir(fixture.0.join("outside")).unwrap();
        // Junction creation needs no elevation; cmd is used only to construct the
        // disposable test link, never for deletion or enumerated user paths.
        let status = std::process::Command::new("cmd.exe")
            .args(["/c", "mklink", "/J"])
            .arg(fixture.0.join("Local"))
            .arg(fixture.0.join("outside"))
            .output()
            .unwrap();
        assert!(status.status.success());
        assert!(inspect_fixture(&pin, true).partial);
        assert!(clear(&pin, true).is_err());
        fs::remove_dir(fixture.0.join("Local")).unwrap();
    }
    #[test]
    fn expired_read_only_budget_never_grants_cache_clear() {
        let fixture = Fixture::new();
        let cache = fixture.cache();
        fs::create_dir_all(&cache).unwrap();
        fs::write(cache.join("retained"), b"must survive").unwrap();
        let status = inspect(&fixture.pin(), true, InspectionBudget::new(Duration::ZERO));
        assert!(status.partial);
        assert!(!status.can_clear);
        assert!(cache.join("retained").exists());
    }

    #[test]
    fn expired_mutating_preflight_never_marks_files_for_deletion() {
        let fixture = Fixture::new();
        let cache = fixture.cache();
        fs::create_dir_all(&cache).unwrap();
        let leaf = cache.join("retained");
        fs::write(&leaf, b"must survive").unwrap();
        assert!(plan(&fixture.pin(), true, InspectionBudget::new(Duration::ZERO)).is_err());
        assert_eq!(fs::read(&leaf).unwrap(), b"must survive");
    }

    #[test]
    fn incomplete_plan_refuses_before_deleting_any_cache_file() {
        let fixture = Fixture::new();
        let cache = fixture.cache();
        fs::create_dir_all(&cache).unwrap();
        fs::write(cache.join("keep-until-plan-complete"), b"cache").unwrap();
        let mut deep = cache.clone();
        for _ in 0..10 {
            deep.push("nested");
        }
        fs::create_dir_all(deep).unwrap();
        let pin = fixture.pin();
        assert!(inspect_fixture(&pin, true).partial);
        assert!(clear(&pin, true).is_err());
        assert!(cache.join("keep-until-plan-complete").exists());
    }
}

#[cfg(test)]
mod size_tests {
    use super::*;

    #[test]
    fn selected_review_uses_same_aggregate_bounds_as_preflight() {
        let mut extent = SelectedTreeExtent { bytes: 1, files: 1, directories: MAX_DIRECTORIES - 2, entries: 1 };
        extent.accumulate(SelectedTreeExtent { bytes: 2, files: 1, directories: 1, entries: 1 }).unwrap();
        assert_eq!((extent.bytes, extent.files, extent.directories), (3, 2, MAX_DIRECTORIES - 1));
        assert!(extent.accumulate(SelectedTreeExtent { directories: 1, ..Default::default() }).is_err());
        assert_eq!(extent.directories, MAX_DIRECTORIES - 1);
        let mut extent = SelectedTreeExtent { files: MAX_FILES as u32 - 1, ..Default::default() };
        assert!(extent.accumulate(SelectedTreeExtent { files: 1, ..Default::default() }).is_err());
    }

    #[test]
    fn metadata_scan_counts_files_and_reports_bounds_without_opening_contents() {
        let directory = std::env::temp_dir().join(format!(
            "lazy-tester-storage-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&directory).unwrap();
        std::fs::create_dir(directory.join("nested")).unwrap();
        std::fs::write(directory.join("a.bin"), [0u8; 17]).unwrap();
        std::fs::write(directory.join("nested/b.bin"), [0u8; 23]).unwrap();
        let mut result = ProfileStorage {
            slot: 5,
            bytes: 0,
            files: 0,
            partial: false,
            disk: None,
            cache: Default::default(),
            download_cache: None,
            game_settings: None,
            full_clear: None,
            resource_packs: None,
        };
        scan_size(&directory, &mut result, InspectionBudget::new(Duration::from_secs(2)), 0);
        assert_eq!((result.bytes, result.files, result.partial), (40, 2, false));
        scan_size(&directory, &mut result, InspectionBudget::new(Duration::from_secs(2)), 49);
        assert_eq!((result.bytes, result.files, result.partial), (40, 2, true));
        // Expired shared admission never pretends a zero-byte snapshot is complete.
        let mut expired = result.clone();
        expired.bytes = 0;
        expired.files = 0;
        expired.partial = false;
        scan_size(&directory, &mut expired, InspectionBudget::new(Duration::ZERO), 0);
        assert_eq!((expired.bytes, expired.files, expired.partial), (0, 0, true));
        // Only the unique, create-new disposable fixture directory created above.
        assert_eq!(directory.parent(), Some(std::env::temp_dir().as_path()));
        std::fs::remove_dir_all(directory).unwrap();
    }
}
