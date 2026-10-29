//! Bounded, read-only inventory of resource pack manifest directories.
//! A pack present on disk is NOT proof that a world uses it or retains its textures.
use super::{directory_path, open_directory};
use crate::engine::native_clients_session::ResourcePackStorage;
use std::{fs, io};
use std::os::windows::fs::MetadataExt;
use virtual_clients_core::InspectionBudget;
use windows_sys::Win32::Storage::FileSystem::{
    FILE_ATTRIBUTE_REPARSE_POINT, FILE_LIST_DIRECTORY, FILE_READ_ATTRIBUTES,
};

const MAX_ENTRIES: u32 = 128;

fn inspect_inner(private: &fs::File, budget: InspectionBudget) -> io::Result<u32> {
    let mut path = directory_path(private)?;
    // The same canonical Education profile subtree as options.txt; no worlds,
    // credentials, third-party directories or resource file contents are read.
    for part in ["Roaming", "Minecraft Education Edition", "games", "com.mojang"] {
        path.push(part);
        match open_directory(&path, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY) {
            Ok(_) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(0),
            Err(e) => return Err(e),
        }
    }
    let mut manifested = 0;
    let mut visited = 0;
    for name in ["resource_packs", "development_resource_packs"] {
        let root = path.join(name);
        match open_directory(&root, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY) {
            Ok(_directory) => {}
            Err(e) if e.kind() == io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e),
        }
        for entry in fs::read_dir(&root)? {
            visited += 1;
            if visited > MAX_ENTRIES || budget.exhausted() {
                return Err(io::Error::other("Resource pack inventory is partial (entry/time limit)"));
            }
            let entry = entry?;
            let path = entry.path();
            let meta = fs::symlink_metadata(&path)?;
            if meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
                return Err(io::Error::other("Resource pack inventory contains a link; scanning stopped"));
            }
            if !meta.is_dir() { continue; }
            // Refuse junctions and linked child directories before looking for a
            // manifest; never recurse into pack assets or world resources.
            let _pin = open_directory(&path, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY)?;
            let manifest = path.join("manifest.json");
            match fs::symlink_metadata(&manifest) {
                Ok(m) if m.is_file() && m.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT == 0 => {
                    manifested += 1;
                }
                Ok(m) if m.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 =>
                    return Err(io::Error::other("Resource pack manifest is linked; scanning stopped")),
                Ok(_) => {}
                Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                Err(e) => return Err(e),
            }
        }
    }
    Ok(manifested)
}

pub(super) fn inspect(private: &fs::File, budget: InspectionBudget) -> ResourcePackStorage {
    match inspect_inner(private, budget) {
        Ok(manifested) => ResourcePackStorage { manifested, partial: false, issue: None },
        Err(error) => ResourcePackStorage {
            // Never report an unknown partial count as a reliable complete number.
            manifested: 0, partial: true, issue: Some(error.to_string()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn count_manifest_directories_without_reading_pack_content_or_worlds() {
        use windows_sys::Win32::Storage::FileSystem::{FILE_LIST_DIRECTORY, FILE_READ_ATTRIBUTES};
        let parent = super::super::fresh_root(&std::env::temp_dir()).unwrap();
        let pinned = open_directory(&parent, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY).unwrap();
        let empty = inspect(&pinned, InspectionBudget::new(std::time::Duration::from_secs(2)));
        assert_eq!(empty.manifested, 0);
        assert!(!empty.partial);
        let base = parent.join("Roaming").join("Minecraft Education Edition")
            .join("games").join("com.mojang");
        let packs = base.join("resource_packs");
        fs::create_dir_all(packs.join("one")).unwrap();
        fs::create_dir_all(packs.join("not-a-pack")).unwrap();
        fs::write(packs.join("one/manifest.json"),b"{}").unwrap();
        fs::create_dir_all(base.join("development_resource_packs/two")).unwrap();
        fs::write(base.join("development_resource_packs/two/manifest.json"),b"{}").unwrap();
        fs::create_dir_all(base.join("minecraftWorlds/unchanged")).unwrap();
        fs::write(base.join("minecraftWorlds/unchanged/level.dat"),b"keep").unwrap();
        let state = inspect(&pinned, InspectionBudget::new(std::time::Duration::from_secs(2)));
        assert_eq!(state.manifested,2);
        assert!(!state.partial);
        assert_eq!(fs::read(base.join("minecraftWorlds/unchanged/level.dat")).unwrap(),b"keep");
        drop(pinned);
        fs::remove_dir_all(parent).unwrap();
    }
}
