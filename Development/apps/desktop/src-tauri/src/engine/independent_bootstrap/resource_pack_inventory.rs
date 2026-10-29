//! Read-only, bounded Minecraft Education profile resource-pack metadata.
//! A file on disk or a declaration in a world is NOT proof of loaded textures,
//! current world selection, GPU residency or reclaimable RAM.
use super::{directory_path, image_id, open_directory};
use crate::engine::native_clients_session::{ResourcePackStorage, WorldPackDeclarations};
use std::{
    fs::{self, File, OpenOptions}, io::{self, Read},
    os::windows::fs::{MetadataExt, OpenOptionsExt}, path::Path,
};
use virtual_clients_core::InspectionBudget;
use windows_sys::Win32::Storage::FileSystem::{
    FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_OPEN_REPARSE_POINT,
    FILE_LIST_DIRECTORY, FILE_READ_ATTRIBUTES, FILE_SHARE_DELETE, FILE_SHARE_READ,
};

const MAX_PACK_ENTRIES: u32 = 128;
const MAX_WORLD_ENTRIES: u32 = 128;
const MAX_WORLD_PACK_REFS: u32 = 256;
const MAX_DECLARATIONS_BYTES: usize = 32 * 1024;

fn base_path(private: &File) -> io::Result<Option<std::path::PathBuf>> {
    let mut path = directory_path(private)?;
    for part in ["Roaming", "Minecraft Education Edition", "games", "com.mojang"] {
        path.push(part);
        match open_directory(&path, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY) {
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        }
    }
    Ok(Some(path))
}

fn count_manifested(base: &Path, budget: InspectionBudget) -> io::Result<u32> {
    let mut manifested = 0;
    let mut visited = 0;
    for name in ["resource_packs", "development_resource_packs"] {
        let root = base.join(name);
        match open_directory(&root, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY) {
            Ok(_directory) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        }
        for entry in fs::read_dir(&root)? {
            visited += 1;
            if visited > MAX_PACK_ENTRIES || budget.exhausted() {
                return Err(io::Error::other("Resource pack inventory exceeded its entry/time limit"));
            }
            let path = entry?.path();
            let meta = fs::symlink_metadata(&path)?;
            if meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
                return Err(io::Error::other("Resource pack inventory contains a link"));
            }
            if !meta.is_dir() { continue; }
            // Check ordinary directory identity, never follow junctions.
            let _pin = open_directory(&path, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY)?;
            match fs::symlink_metadata(path.join("manifest.json")) {
                Ok(m) if m.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 =>
                    return Err(io::Error::other("Resource pack manifest is linked")),
                Ok(m) if m.is_file() => manifested += 1,
                Ok(_) => {},
                Err(error) if error.kind() == io::ErrorKind::NotFound => {},
                Err(error) => return Err(error),
            }
        }
    }
    Ok(manifested)
}

#[derive(serde::Deserialize)]
struct DeclaredPack {
    pack_id: String,
    version: [u32; 3],
}

fn valid_uuid(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 36 && bytes.iter().enumerate().all(|(index, byte)|
        if [8, 13, 18, 23].contains(&index) { *byte == b'-' }
        else { byte.is_ascii_hexdigit() })
}

// A world may reference resource packs using a small JSON manifest. This is
// not the world database, level.dat, player data, behavior packs or pack assets.
fn read_world_declarations(path: &Path) -> io::Result<Option<u32>> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
        Ok(meta) if meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 || !meta.is_file() =>
            return Err(io::Error::other("World resource-pack declaration is not an ordinary file")),
        Ok(meta) if meta.len() > MAX_DECLARATIONS_BYTES as u64 =>
            return Err(io::Error::other("World resource-pack declaration is oversized")),
        Ok(_) => {},
    }
    let file = OpenOptions::new().read(true).share_mode(FILE_SHARE_READ | FILE_SHARE_DELETE)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT).open(path)?;
    image_id(&file)?; // Refuse replacement by a reparse point after the metadata check.
    let mut bytes = Vec::new();
    file.take((MAX_DECLARATIONS_BYTES + 1) as u64).read_to_end(&mut bytes)?;
    if bytes.len() > MAX_DECLARATIONS_BYTES {
        return Err(io::Error::other("World resource-pack declaration exceeds its size limit"));
    }
    let refs: Vec<DeclaredPack> = serde_json::from_slice(&bytes)
        .map_err(|_| io::Error::other("World resource-pack declaration is not a supported JSON list"))?;
    if refs.len() > MAX_WORLD_PACK_REFS as usize || refs.iter().any(|entry|
        !valid_uuid(&entry.pack_id) || entry.version.iter().any(|version| *version > 65535)) {
        return Err(io::Error::other("World resource-pack declaration is invalid or too large"));
    }
    Ok(Some(refs.len() as u32))
}

fn scan_world_declarations(base: &Path, budget: InspectionBudget) -> io::Result<WorldPackDeclarations> {
    let root = base.join("minecraftWorlds");
    match open_directory(&root, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY) {
        Ok(_root) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(WorldPackDeclarations {
            worlds_scanned: 0, worlds_with_references: 0, pack_references: 0,
            partial: false, issue: None,
        }),
        Err(error) => return Err(error),
    }
    let mut scanned = 0;
    let mut with_references = 0;
    let mut references = 0;
    let mut visited = 0;
    for entry in fs::read_dir(&root)? {
        visited += 1;
        if visited > MAX_WORLD_ENTRIES || budget.exhausted() {
            return Err(io::Error::other("World resource-pack inventory exceeded its entry/time limit"));
        }
        let path = entry?.path();
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(io::Error::other("World resource-pack inventory contains a link"));
        }
        if !metadata.is_dir() { continue; }
        let _pin = open_directory(&path, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY)?;
        scanned += 1;
        if let Some(count) = read_world_declarations(&path.join("world_resource_packs.json"))? {
            references += count;
            if references > MAX_WORLD_PACK_REFS {
                return Err(io::Error::other("Too many declared world resource-pack references"));
            }
            if count > 0 { with_references += 1; }
        }
    }
    Ok(WorldPackDeclarations {
        worlds_scanned: scanned, worlds_with_references: with_references,
        pack_references: references, partial: false, issue: None,
    })
}

fn unavailable(error: io::Error) -> WorldPackDeclarations {
    // Fail closed: partial results are not authoritative counts.
    WorldPackDeclarations {
        worlds_scanned: 0, worlds_with_references: 0, pack_references: 0,
        partial: true, issue: Some(error.to_string()),
    }
}

pub(super) fn inspect(private: &File, budget: InspectionBudget) -> ResourcePackStorage {
    let base = match base_path(private) {
        Ok(base) => base,
        Err(error) => return ResourcePackStorage {
            manifested: 0, partial: true, issue: Some(error.to_string()),
            world_declarations: None,
        },
    };
    let Some(base) = base else {
        return ResourcePackStorage {
            manifested: 0, partial: false, issue: None,
            world_declarations: Some(WorldPackDeclarations {
                worlds_scanned: 0, worlds_with_references: 0, pack_references: 0,
                partial: false, issue: None,
            }),
        };
    };
    // Independent evidence: world-reference failure must not erase a good
    // manifest-folder count. Neither result implies loaded GPU/CPU memory.
    let (manifested, partial, issue) = match count_manifested(&base, budget) {
        Ok(manifested) => (manifested, false, None),
        Err(error) => (0, true, Some(error.to_string())),
    };
    let world_declarations = Some(match scan_world_declarations(&base, budget) {
        Ok(declarations) => declarations,
        Err(error) => unavailable(error),
    });
    ResourcePackStorage { manifested, partial, issue, world_declarations }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn report_declarations_without_reading_world_database_or_modifying_any_assets() {
        let parent = super::super::fresh_root(&std::env::temp_dir()).unwrap();
        let private = open_directory(&parent, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY).unwrap();
        let empty = inspect(&private, InspectionBudget::new(Duration::from_secs(2)));
        assert_eq!(empty.manifested, 0);
        assert_eq!(empty.world_declarations.as_ref().unwrap().pack_references, 0);
        let base = parent.join("Roaming").join("Minecraft Education Edition")
            .join("games").join("com.mojang");
        let packs = base.join("resource_packs");
        fs::create_dir_all(packs.join("installed")).unwrap();
        fs::write(packs.join("installed/manifest.json"), b"{}").unwrap();
        let worlds = base.join("minecraftWorlds");
        fs::create_dir_all(worlds.join("one/db")).unwrap();
        fs::create_dir_all(worlds.join("two")).unwrap();
        fs::write(worlds.join("one/db/untouched"), b"keep original bytes").unwrap();
        fs::write(worlds.join("one/world_resource_packs.json"),
            br#"[{"pack_id":"11111111-2222-3333-4444-555555555555","version":[1,2,3]}]"#).unwrap();
        let report = inspect(&private, InspectionBudget::new(Duration::from_secs(2)));
        assert_eq!(report.manifested, 1);
        assert!(!report.partial);
        let refs = report.world_declarations.unwrap();
        assert_eq!((refs.worlds_scanned, refs.worlds_with_references, refs.pack_references), (2, 1, 1));
        assert!(!refs.partial);
        assert_eq!(fs::read(worlds.join("one/db/untouched")).unwrap(), b"keep original bytes");
        drop(private);
        fs::remove_dir_all(parent).unwrap();
    }

    #[test]
    fn corrupt_or_oversized_world_metadata_is_unknown_without_erasing_pack_folder_count() {
        let parent = super::super::fresh_root(&std::env::temp_dir()).unwrap();
        let private = open_directory(&parent, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY).unwrap();
        let base = parent.join("Roaming").join("Minecraft Education Edition")
            .join("games").join("com.mojang");
        let pack = base.join("resource_packs/example");
        fs::create_dir_all(&pack).unwrap();
        fs::write(pack.join("manifest.json"), b"{}").unwrap();
        let world = base.join("minecraftWorlds/one");
        fs::create_dir_all(&world).unwrap();
        let oversized = vec![b'x'; MAX_DECLARATIONS_BYTES + 1];
        for bad in [b"not json".as_slice(), b"[{\"pack_id\":\"bad\",\"version\":[1,0,0]}]".as_slice(),
            oversized.as_slice()] {
            fs::write(world.join("world_resource_packs.json"), bad).unwrap();
            let report = inspect(&private, InspectionBudget::new(Duration::from_secs(2)));
            assert_eq!(report.manifested, 1);
            assert!(!report.partial);
            assert!(report.world_declarations.as_ref().unwrap().partial);
            assert_eq!(report.world_declarations.as_ref().unwrap().pack_references, 0);
        }
        drop(private);
        fs::remove_dir_all(parent).unwrap();
    }
}
