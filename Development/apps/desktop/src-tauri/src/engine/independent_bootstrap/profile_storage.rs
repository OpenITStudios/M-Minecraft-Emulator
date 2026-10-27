//! Metadata-only classification and handle-based cleanup of one proven HTTP cache.
use super::{directory_path, image_id, invalid, open_directory, wide};
use crate::engine::native_clients_session::{CacheStorage, DiskSpace};
use std::{
    fs::{self, File, OpenOptions},
    io, mem,
    os::windows::{fs::OpenOptionsExt, io::AsRawHandle},
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

struct CachePlan {
    directories: Vec<File>,
    files: Vec<File>,
    bytes: u64,
    entries: usize,
    budget: InspectionBudget,
    deleting: bool,
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
        }
    }
    fn bounded(&self, depth: usize) -> io::Result<()> {
        if depth > 8
            || self.files.len() >= MAX_FILES
            || self.directories.len() >= MAX_DIRECTORIES
            || self.entries >= MAX_FILES + MAX_DIRECTORIES
            || self.budget.exhausted()
        {
            return Err(invalid(
                "Cache inspection limit reached; nothing was deleted",
            ));
        }
        Ok(())
    }
    fn visit(&mut self, path: &Path, depth: usize) -> io::Result<()> {
        self.bounded(depth)?;
        self.directories.push(open_directory(
            path,
            FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY,
        )?);
        for entry in fs::read_dir(path)? {
            self.entries += 1;
            self.bounded(depth)?;
            let path = entry?.path();
            let metadata = fs::symlink_metadata(&path)?;
            if metadata.is_dir() {
                self.visit(&path, depth + 1)?;
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
                self.files.push(file);
            }
        }
        Ok(())
    }
}

fn plan(private: &File, deleting: bool, budget: InspectionBudget) -> io::Result<CachePlan> {
    let mut plan = CachePlan::new(deleting, budget);
    // Check the read-only deadline before every ancestor inspection. The
    // mutating clear path gets its own unchanged two-second local budget.
    if !deleting { plan.bounded(0)?; }
    let mut path = directory_path(private)?;
    plan.directories.push(private.try_clone()?);
    for part in CACHE_PARTS {
        if !deleting { plan.bounded(0)?; }
        path.push(part);
        match open_directory(&path, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY) {
            Ok(pin) => plan.directories.push(pin),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                if !deleting { plan.bounded(0)?; }
                return Ok(plan);
            }
            Err(error) => return Err(error),
        }
    }
    plan.visit(&path, 0)?;
    // A scan that consumed the budget on its final OS call is not a complete
    // observation and must never advertise cache cleanup eligibility.
    if !deleting { plan.bounded(0)?; }
    Ok(plan)
}

pub(super) fn inspect(private: &File, inactive: bool, budget: InspectionBudget) -> CacheStorage {
    match plan(private, false, budget) {
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
    if !inactive {
        return Err(invalid(
            "Close this Minecraft session normally before clearing its cache",
        ));
    }
    let plan = plan(private, true, InspectionBudget::new(Duration::from_secs(2)))?;
    // No path-based delete or recursive directory removal. Retain ancestor pins
    // and exclusive file handles until all dispositions have been attempted.
    for file in &plan.files {
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
