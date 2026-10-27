//! Human-readable helper names, with the same pinned executable file identity.
use serde::Deserialize;
use std::os::windows::io::AsRawHandle;
use std::{
    fs::{File, OpenOptions},
    io,
    os::windows::fs::OpenOptionsExt,
    path::{Path, PathBuf},
};
use windows_sys::Win32::{Foundation::HANDLE, Storage::FileSystem::*};

#[derive(Deserialize)]
struct Role {
    role: String,
    file: String,
    description: String,
}
fn roles() -> Vec<Role> {
    serde_json::from_str(include_str!("../../windows/process-roles.json"))
        .expect("Compiled process roles must be valid")
}
pub(crate) struct Image {
    pub path: PathBuf,
    _pins: Vec<File>,
}
fn pin(path: &Path) -> io::Result<(File, (u64, [u8; 16]))> {
    let file = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    let mut tag: FILE_ATTRIBUTE_TAG_INFO = unsafe { std::mem::zeroed() };
    let mut id: FILE_ID_INFO = unsafe { std::mem::zeroed() };
    let handle = file.as_raw_handle() as HANDLE;
    if unsafe {
        GetFileInformationByHandleEx(
            handle,
            FileAttributeTagInfo,
            (&mut tag as *mut FILE_ATTRIBUTE_TAG_INFO).cast(),
            std::mem::size_of_val(&tag) as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    if tag.FileAttributes & (FILE_ATTRIBUTE_REPARSE_POINT | FILE_ATTRIBUTE_DIRECTORY) != 0 {
        return Err(io::Error::other("Helper image must be an ordinary file"));
    }
    if unsafe {
        GetFileInformationByHandleEx(
            handle,
            FileIdInfo,
            (&mut id as *mut FILE_ID_INFO).cast(),
            std::mem::size_of_val(&id) as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok((file, (id.VolumeSerialNumber, id.FileId.Identifier)))
}
fn at(current: &Path, role: &str) -> io::Result<Image> {
    let role = roles()
        .into_iter()
        .find(|r| r.role == role)
        .ok_or_else(|| io::Error::other("Unknown helper role"))?;
    let path = current
        .parent()
        .ok_or_else(|| io::Error::other("Helper folder unavailable"))?
        .join(&role.file);
    let (main, id) = pin(current)?;
    let (alias, actual) = pin(&path)?;
    if actual != id {
        return Err(io::Error::other("Helper must be a hard link to this exact Lazy Tester executable; no copied or stale helper is accepted"));
    }
    Ok(Image {
        path,
        _pins: vec![main, alias],
    })
}
pub(crate) fn image(role: &str) -> io::Result<Image> {
    let current = std::env::current_exe()?;
    if !cfg!(feature = "lazy-tester-package") || cfg!(debug_assertions) || cfg!(test) {
        return Ok(Image {
            path: current,
            _pins: Vec::new(),
        });
    }
    at(&current, role)
}
pub(crate) fn same_main_image(parent: &Path, current: &Path) -> io::Result<bool> {
    let same = |a: &Path, b: &Path| {
        a.to_string_lossy()
            .eq_ignore_ascii_case(&b.to_string_lossy())
    };
    if parent
        .file_name()
        .is_none_or(|name| !name.eq_ignore_ascii_case("lazy-tester.exe"))
        || parent
            .parent()
            .zip(current.parent())
            .is_none_or(|(a, b)| !same(a, b))
        || !roles().iter().any(|r| {
            current
                .file_name()
                .is_some_and(|name| name.eq_ignore_ascii_case(&r.file))
        })
    {
        return Ok(false);
    }
    let (_parent, left) = pin(parent)?;
    let (_current, right) = pin(current)?;
    Ok(left == right)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn named_helpers_require_the_original_file_and_reject_identical_copies() {
        let root = std::env::temp_dir().join(format!(
            "lazy-roles-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        let main = root.join("lazy-tester.exe");
        std::fs::write(&main, b"MZfixture").unwrap();
        for role in roles() {
            assert_eq!(role.description.chars().count(), 11);
            let alias = root.join(&role.file);
            std::fs::copy(&main, &alias).unwrap();
            assert!(at(&main, &role.role).is_err());
            assert!(!same_main_image(&main, &alias).unwrap());
            std::fs::remove_file(&alias).unwrap();
            std::fs::hard_link(&main, &alias).unwrap();
            let lease = at(&main, &role.role).unwrap();
            assert_eq!(lease.path, alias);
            assert!(same_main_image(&main, &alias).unwrap());
            assert!(OpenOptions::new().write(true).open(&alias).is_err());
            assert!(!same_main_image(&root.join("other.exe"), &alias).unwrap());
            drop(lease);
        }
        assert!(at(&main, "unknown").is_err());
        assert!(root.starts_with(std::env::temp_dir()));
        std::fs::remove_dir_all(root).unwrap();
    }
}
