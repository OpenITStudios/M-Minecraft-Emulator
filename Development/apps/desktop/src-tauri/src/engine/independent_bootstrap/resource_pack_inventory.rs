//! Read-only, bounded Minecraft Education profile resource-pack metadata.
//! A file on disk or a declaration in a world is NOT proof of loaded textures,
//! current world selection, GPU residency or reclaimable RAM.
use super::{directory_id, directory_path, image_id, open_directory};
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

// Selective inventory is read-only and on demand. A directory identity is
// evidence for a future review, not deletion authorization.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum InventoryCategory { Worlds, ResourcePacks, BehaviorPacks }

pub(super) struct InventoryItem {
    pub(super) category: InventoryCategory,
    pub(super) root: &'static str,
    pub(super) name: String,
    pub(super) identity: windows_sys::Win32::Storage::FileSystem::FILE_ID_INFO,
}

// Opaque display identity also binds the retained profile and selected slot.
// This is not an authorization token.
pub(super) fn inventory_identity(private: &File, slot: u32, item: &InventoryItem) -> io::Result<String> {
    use sha2::{Digest, Sha256};
    let profile = directory_id(private)?;
    let mut hash = Sha256::new();
    hash.update(b"LazyTester/selective-item/v1");
    hash.update(slot.to_le_bytes());
    for id in [&profile, &item.identity] {
        hash.update(id.VolumeSerialNumber.to_le_bytes());
        hash.update(id.FileId.Identifier);
    }
    for field in [item.root, item.name.as_str()] {
        hash.update((field.len() as u64).to_le_bytes());
        hash.update(field.as_bytes());
    }
    Ok(hash.finalize().iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(""))
}

pub(super) fn individual_inventory(private: &File, category: InventoryCategory,
    budget: InspectionBudget) -> io::Result<Vec<InventoryItem>> {
    let Some((base, _pins)) = base_path(private, budget)? else {
        if budget.exhausted() { return Err(io::Error::other("Inventory deadline exceeded")); }
        return Ok(Vec::new());
    };
    let (roots, marker, limit): (&[&'static str], &str, u32) = match category {
        InventoryCategory::Worlds => (&["minecraftWorlds"], "level.dat", MAX_WORLD_ENTRIES),
        InventoryCategory::ResourcePacks => (&["resource_packs", "development_resource_packs"], "manifest.json", MAX_PACK_ENTRIES),
        InventoryCategory::BehaviorPacks => (&["behavior_packs", "development_behavior_packs"], "manifest.json", MAX_PACK_ENTRIES),
    };
    let mut items = Vec::new();
    let mut visited = 0;
    for &root_name in roots {
        if budget.exhausted() { return Err(io::Error::other("Inventory deadline exceeded")); }
        let root = base.join(root_name);
        let _root_pin = match open_directory(&root, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY) {
            Ok(pin) => pin,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
        for entry in fs::read_dir(&root)? {
            visited += 1;
            if visited > limit || budget.exhausted() { return Err(io::Error::other("Inventory limit exceeded")); }
            let entry = entry?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path)?;
            if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
                return Err(io::Error::other("Linked inventory entry"));
            }
            if !metadata.is_dir() { continue; }
            let pin = open_directory(&path, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY)?;
            match fs::symlink_metadata(path.join(marker)) {
                Ok(meta) if meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 => return Err(io::Error::other("Linked inventory marker")),
                Ok(meta) if meta.is_file() => {},
                Ok(_) => continue,
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(error) => return Err(error),
            }
            let name = entry.file_name().into_string().map_err(|_| io::Error::other("Unsupported inventory name"))?;
            items.push(InventoryItem { category, root: root_name, name, identity: directory_id(&pin)? });
        }
    }
    if budget.exhausted() { return Err(io::Error::other("Inventory deadline exceeded")); }
    items.sort_by(|a,b| a.root.cmp(b.root).then_with(|| a.name.cmp(&b.name)));
    Ok(items)
}

// Hold every verified ancestor for the entire read-only inventory. A path
// returned without its pins is not a stable basis for a later cleanup review.
fn base_path(private: &File, budget: InspectionBudget) -> io::Result<Option<(std::path::PathBuf, Vec<File>)>> {
    let mut path = directory_path(private)?;
    let mut pins = vec![private.try_clone()?];
    for part in ["Roaming", "Minecraft Education Edition", "games", "com.mojang"] {
        if budget.exhausted() { return Err(io::Error::other("Resource pack inventory exceeded its time limit")); }
        path.push(part);
        match open_directory(&path, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY) {
            Ok(pin) => pins.push(pin),
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        }
    }
    if budget.exhausted() { return Err(io::Error::other("Resource pack inventory exceeded its time limit")); }
    Ok(Some((path, pins)))
}

fn count_manifested(base: &Path, budget: InspectionBudget) -> io::Result<u32> {
    let mut manifested = 0;
    let mut visited = 0;
    for name in ["resource_packs", "development_resource_packs"] {
        // Five profiles share this deadline: do not open another root after
        // another storage scan has already consumed the remaining budget.
        if budget.exhausted() {
            return Err(io::Error::other("Resource pack inventory exceeded its time limit"));
        }
        let root = base.join(name);
        let _directory = match open_directory(&root, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY) {
            Ok(directory) => directory,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };
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
    // A final filesystem call can cross the deadline after the last entry.
    // Refuse a misleading complete count even if the directory is empty.
    if budget.exhausted() {
        return Err(io::Error::other("Resource pack inventory exceeded its time limit"));
    }
    Ok(manifested)
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Deserialize)]
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
fn read_pack_references(path: &Path) -> io::Result<Option<Vec<DeclaredPack>>> {
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
    Ok(Some(refs))
}

// Review evidence only: a known reference blocks pack removal; an incomplete
// observation must never be treated as proof that a pack is unused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct WorldPackReference {
    pub(super) world: String,
    pub(super) category: InventoryCategory,
    pub(super) uuid: String,
    pub(super) version: [u32; 3],
}

pub(super) fn world_pack_references(private: &File, budget: InspectionBudget)
    -> io::Result<Vec<WorldPackReference>> {
    let Some((base, _pins)) = base_path(private, budget)? else {
        if budget.exhausted() { return Err(io::Error::other("Reference review deadline exceeded")); }
        return Ok(Vec::new());
    };
    let root = base.join("minecraftWorlds");
    if budget.exhausted() { return Err(io::Error::other("Reference review deadline exceeded")); }
    let _root_pin = match open_directory(&root, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY) {
        Ok(pin) => pin,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            if budget.exhausted() { return Err(io::Error::other("Reference review deadline exceeded")); }
            return Ok(Vec::new());
        }
        Err(error) => return Err(error),
    };
    let mut references = Vec::new();
    let mut visited = 0;
    for entry in fs::read_dir(&root)? {
        visited += 1;
        if visited > MAX_WORLD_ENTRIES || budget.exhausted() {
            return Err(io::Error::other("World reference review exceeded its limit"));
        }
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(io::Error::other("Linked world in reference review"));
        }
        if !metadata.is_dir() { continue; }
        let _world_pin = open_directory(&path, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY)?;
        let world = entry.file_name().into_string()
            .map_err(|_| io::Error::other("Unsupported world name"))?;
        // Include even unclassified world directories: a missing level.dat
        // is not evidence that their declared dependencies can be ignored.
        for (category, filename) in [
            (InventoryCategory::ResourcePacks, "world_resource_packs.json"),
            (InventoryCategory::BehaviorPacks, "world_behavior_packs.json"),
        ] {
            if budget.exhausted() { return Err(io::Error::other("Reference review deadline exceeded")); }
            if let Some(declared) = read_pack_references(&path.join(filename))? {
                for reference in declared {
                    if references.len() >= MAX_WORLD_PACK_REFS as usize {
                        return Err(io::Error::other("Too many world pack references"));
                    }
                    references.push(WorldPackReference {
                        world: world.clone(), category,
                        uuid: reference.pack_id.to_ascii_lowercase(), version: reference.version,
                    });
                }
            }
        }
    }
    if budget.exhausted() { return Err(io::Error::other("Reference review deadline exceeded")); }
    Ok(references)
}

// Bounded evidence for installed pack headers and UUID dependencies.
// Unsupported manifest versions or dependency shapes block review rather
// than silently treating the pack as independent.
#[derive(serde::Deserialize)]
struct PackHeader {
    uuid: String,
    version: [u32; 3],
}
#[derive(serde::Deserialize)]
struct PackManifest {
    header: PackHeader,
    #[serde(default)]
    dependencies: Vec<ManifestDependency>,
}

// Typed fields reject duplicate keys instead of silently retaining the last
// UUID. Only the two currently understood dependency shapes are admitted.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestDependency {
    uuid: Option<String>,
    module_name: Option<String>,
    version: serde_json::Value,
}

pub(super) struct PackDependencyEvidence {
    pub(super) item: InventoryItem,
    pub(super) uuid: String,
    pub(super) version: [u32; 3],
    pub(super) dependencies: Vec<String>,
}

fn read_manifest(path: &Path) -> io::Result<PackManifest> {
    let file = OpenOptions::new().read(true).share_mode(FILE_SHARE_READ)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT).open(path)?;
    image_id(&file)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
        || metadata.len() > MAX_DECLARATIONS_BYTES as u64 {
        return Err(io::Error::other("Unsupported pack manifest"));
    }
    let mut bytes = Vec::new();
    file.take((MAX_DECLARATIONS_BYTES + 1) as u64).read_to_end(&mut bytes)?;
    if bytes.len() > MAX_DECLARATIONS_BYTES {
        return Err(io::Error::other("Pack manifest exceeded its size limit"));
    }
    let manifest: PackManifest = serde_json::from_slice(&bytes)
        .map_err(|_| io::Error::other("Unsupported pack manifest"))?;
    if !valid_uuid(&manifest.header.uuid) ||
        manifest.header.version.iter().any(|v| *v > 65535) ||
        manifest.dependencies.len() > MAX_WORLD_PACK_REFS as usize {
        return Err(io::Error::other("Invalid pack identity or dependencies"));
    }
    Ok(manifest)
}

fn dependency_uuids(manifest: &PackManifest) -> io::Result<Vec<String>> {
    let mut result = Vec::new();
    for entry in &manifest.dependencies {
        match (&entry.uuid, &entry.module_name) {
            (Some(uuid), None) if valid_uuid(uuid) => {
                let version: [u32; 3] = serde_json::from_value(entry.version.clone())
                    .map_err(|_| io::Error::other("Invalid pack dependency version"))?;
                if version.iter().any(|v| *v > 65535) {
                    return Err(io::Error::other("Invalid pack dependency version"));
                }
                result.push(uuid.to_ascii_lowercase());
            }
            (None, Some(module)) if module.starts_with("@minecraft/")
                && module.len() > "@minecraft/".len()
                && entry.version.as_str().is_some_and(|v| !v.trim().is_empty()) => {
                // Script module declaration, not an installed pack UUID.
                // This classification does not establish runtime support.
            }
            _ => return Err(io::Error::other("Unknown or ambiguous pack dependency shape")),
        }
    }
    Ok(result)
}

pub(super) fn pack_dependency_evidence(private: &File, budget: InspectionBudget)
    -> io::Result<Vec<PackDependencyEvidence>> {
    let Some((base, _pins)) = base_path(private, budget)? else {
        if budget.exhausted() { return Err(io::Error::other("Dependency review deadline exceeded")); }
        return Ok(Vec::new());
    };
    let mut evidence = Vec::new();
    let mut total_dependencies = 0;
    for category in [InventoryCategory::ResourcePacks, InventoryCategory::BehaviorPacks] {
        for item in individual_inventory(private, category, budget)? {
            if budget.exhausted() { return Err(io::Error::other("Dependency review deadline exceeded")); }
            let root = base.join(item.root);
            let _root_pin = open_directory(&root, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY)?;
            let path = root.join(&item.name);
            let pin = open_directory(&path, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY)?;
            if !super::same_image(&item.identity, &directory_id(&pin)?) {
                return Err(io::Error::other("Pack directory changed during review"));
            }
            let manifest = read_manifest(&path.join("manifest.json"))?;
            let dependencies = dependency_uuids(&manifest)?;
            total_dependencies += dependencies.len();
            if total_dependencies > MAX_WORLD_PACK_REFS as usize {
                return Err(io::Error::other("Too many pack dependencies"));
            }
            evidence.push(PackDependencyEvidence {
                item, uuid: manifest.header.uuid.to_ascii_lowercase(),
                version: manifest.header.version, dependencies,
            });
        }
    }
    if budget.exhausted() { return Err(io::Error::other("Dependency review deadline exceeded")); }
    Ok(evidence)
}

// This conventional metadata path is a candidate only: absence is not
// proof that no global pack activation exists on the target installation.
pub(super) fn global_pack_references(private: &File, budget: InspectionBudget)
    -> io::Result<Option<Vec<String>>> {
    let Some((base, _pins)) = base_path(private, budget)? else {
        return Ok(None);
    };
    if budget.exhausted() { return Err(io::Error::other("Global reference deadline exceeded")); }
    let refs = read_pack_references(&base.join("global_resource_packs.json"))?;
    if budget.exhausted() { return Err(io::Error::other("Global reference deadline exceeded")); }
    Ok(refs.map(|refs| refs.into_iter().map(|r| r.pack_id.to_ascii_lowercase()).collect()))
}

// The review retains native identities but grants no mutation authority.
// Caller selections must match fresh enumerated entries, never joined paths.
pub(super) struct ReviewSelection {
    pub(super) category: InventoryCategory,
    pub(super) root: String,
    pub(super) name: String,
}

pub(super) struct SelectiveReview {
    pub(super) profile_identity: windows_sys::Win32::Storage::FileSystem::FILE_ID_INFO,
    pub(super) items: Vec<InventoryItem>,
    pub(super) warnings: Vec<String>,
    pub(super) bytes: u64,
    pub(super) files: u32,
    pub(super) tree_digests: Vec<[u8; 32]>,
}

// Domain-separated review fingerprint, not a bearer capability. The caller
// must still perform fresh controller admission and filesystem preflight.
// Pack warnings remain blocking until native dependency coverage is verified.
impl SelectiveReview {
    pub(super) fn confirmation_token(&self, slot: u32) -> io::Result<String> {
        use sha2::{Digest, Sha256};
        if !(1..=5).contains(&slot) || self.items.is_empty() || self.items.len() != self.tree_digests.len() || !self.warnings.is_empty() {
            return Err(io::Error::other("Selective review is not eligible for confirmation"));
        }
        let mut hash = Sha256::new();
        hash.update(b"LazyTester/selective-review/v3");
        hash.update(self.bytes.to_le_bytes());
        hash.update(self.files.to_le_bytes());
        hash.update(slot.to_le_bytes());
        fn identity(hash: &mut Sha256, id: &windows_sys::Win32::Storage::FileSystem::FILE_ID_INFO) {
            hash.update(id.VolumeSerialNumber.to_le_bytes());
            hash.update(id.FileId.Identifier);
        }
        fn field(hash: &mut Sha256, value: &str) {
            hash.update((value.len() as u64).to_le_bytes());
            hash.update(value.as_bytes());
        }
        identity(&mut hash, &self.profile_identity);
        // Canonical ordering makes checkbox order irrelevant.
        let mut items: Vec<_> = self.items.iter().zip(&self.tree_digests).collect();
        items.sort_by(|(a,_),(b,_)| a.root.cmp(b.root).then_with(|| a.name.cmp(&b.name)));
        hash.update((items.len() as u64).to_le_bytes());
        for (item, digest) in items {
            hash.update([match item.category {
                InventoryCategory::Worlds => 1u8,
                InventoryCategory::ResourcePacks => 2,
                InventoryCategory::BehaviorPacks => 3,
            }]);
            field(&mut hash, item.root);
            field(&mut hash, &item.name);
            identity(&mut hash, &item.identity);
            hash.update(digest);
        }
        Ok(hash.finalize().iter().map(|byte| format!("{byte:02x}")).collect::<Vec<_>>().join(""))
    }
}

pub(super) fn review_selection(private: &File, selections: &[ReviewSelection],
    budget: InspectionBudget) -> io::Result<SelectiveReview> {
    if selections.is_empty() || selections.len() > 128 || budget.exhausted() {
        return Err(io::Error::other("Invalid or expired selective review"));
    }
    let profile_identity = directory_id(private)?;
    let mut items = Vec::new();
    for category in [InventoryCategory::Worlds, InventoryCategory::ResourcePacks, InventoryCategory::BehaviorPacks] {
        if !selections.iter().any(|s| s.category == category) { continue; }
        let inventory = individual_inventory(private, category, budget)?;
        for selection in selections.iter().filter(|s| s.category == category) {
            let item = inventory.iter().find(|item| item.root == selection.root.as_str()
                && item.name == selection.name)
                .ok_or_else(|| io::Error::other("Selection no longer matches inventory"))?;
            if items.iter().any(|previous: &InventoryItem| super::same_image(&previous.identity, &item.identity)) {
                return Err(io::Error::other("Duplicate selective review item"));
            }
            items.push(InventoryItem {
                category, root: item.root, name: item.name.clone(), identity: item.identity,
            });
        }
    }
    let mut warnings = Vec::new();
    if items.iter().any(|item| item.category != InventoryCategory::Worlds) {
        let packs = pack_dependency_evidence(private, budget)?;
        let worlds = world_pack_references(private, budget)?;
        let global = global_pack_references(private, budget)?;
        for item in items.iter().filter(|item| item.category != InventoryCategory::Worlds) {
            let pack = packs.iter().find(|pack| super::same_image(&item.identity, &pack.item.identity))
                .ok_or_else(|| io::Error::other("Selected pack identity changed during review"))?;
            // Compare UUID regardless of declared version: a mismatch is
            // not evidence that the selected installed pack is dispensable.
            if worlds.iter().any(|reference| reference.uuid == pack.uuid) {
                warnings.push(format!("{} / {}: referenced by a world", item.root, item.name));
            }
            if packs.iter().any(|other| other.dependencies.contains(&pack.uuid)) {
                warnings.push(format!("{} / {}: referenced by a pack", item.root, item.name));
            }
            match &global {
                Some(refs) if refs.contains(&pack.uuid) => warnings.push(format!("{} / {}: globally referenced", item.root, item.name)),
                None => warnings.push(format!("{} / {}: global references unknown", item.root, item.name)),
                _ => {}
            }
        }
        // Candidate global path and current manifest census are not yet
        // validated against the native installation. Never authorize deletion.
        warnings.push("Pack dependency coverage is not native-verified".into());
    }
    if budget.exhausted() { return Err(io::Error::other("Selective review deadline exceeded")); }
    // One deadline across the entire review, including selected tree sizes.
    // No partial total is returned on failure; no file contents are read.
    let (base, _pins) = base_path(private, budget)?
        .ok_or_else(|| io::Error::other("Selected profile data disappeared"))?;
    let mut extent = super::profile_storage::SelectedTreeExtent::default();
    let mut tree_digests = Vec::new();
    for item in &items {
        if budget.exhausted() { return Err(io::Error::other("Selective review deadline exceeded")); }
        let root = base.join(item.root);
        let _root_pin = open_directory(&root, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY)?;
        let (item_extent, digest) = super::profile_storage::inspect_selected_tree(
            &root.join(&item.name), &item.identity, budget)?;
        tree_digests.push(digest);
        extent.accumulate(item_extent)?;
    }
    if budget.exhausted() { return Err(io::Error::other("Selective review deadline exceeded")); }
    Ok(SelectiveReview { profile_identity, items, warnings, bytes: extent.bytes, files: extent.files, tree_digests })
}

fn read_world_declarations(path: &Path) -> io::Result<Option<u32>> {
    Ok(read_pack_references(path)?.map(|refs| refs.len() as u32))
}

fn scan_world_declarations(base: &Path, budget: InspectionBudget) -> io::Result<WorldPackDeclarations> {
    if budget.exhausted() {
        return Err(io::Error::other("World resource-pack inventory exceeded its time limit"));
    }
    let root = base.join("minecraftWorlds");
    let _root = match open_directory(&root, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY) {
        Ok(root) => root,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            // Missing root means zero only if this OS probe stayed in budget.
            if budget.exhausted() {
                return Err(io::Error::other("World resource-pack inventory exceeded its time limit"));
            }
            return Ok(WorldPackDeclarations {
                worlds_scanned: 0, worlds_with_references: 0, pack_references: 0,
                partial: false, issue: None,
            });
        },
        Err(error) => return Err(error),
    };
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
    if budget.exhausted() {
        return Err(io::Error::other("World resource-pack inventory exceeded its time limit"));
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

fn exhausted_report() -> ResourcePackStorage {
    let issue = "Resource pack inventory exceeded its shared inspection time budget";
    ResourcePackStorage {
        manifested: 0, partial: true, issue: Some(issue.into()),
        world_declarations: Some(unavailable(io::Error::other(issue))),
    }
}

pub(super) fn inspect(private: &File, budget: InspectionBudget) -> ResourcePackStorage {
    // Do not open resource-pack paths when the shared five-slot budget is
    // already exhausted. UNKNOWN is not evidence of zero installed packs.
    if budget.exhausted() { return exhausted_report(); }
    let base = match base_path(private, budget) {
        Ok(base) => base,
        Err(error) => return ResourcePackStorage {
            manifested: 0, partial: true, issue: Some(error.to_string()),
            world_declarations: None,
        },
    };
    // Ancestor probing may itself consume the remaining time. A missing
    // root after that point cannot be confidently reported as zero.
    if budget.exhausted() { return exhausted_report(); }
    let Some((base, _ancestor_pins)) = base else {
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
    fn manifest_dependency_review_distinguishes_pack_uuid_and_builtin_module() {
        let manifest: PackManifest = serde_json::from_str(r#"{"header":{"uuid":"AAAAAAAA-2222-3333-4444-555555555555","version":[1,0,0]},"dependencies":[{"uuid":"BBBBBBBB-2222-3333-4444-555555555555","version":[1,0,0]},{"module_name":"@minecraft/server","version":"1.0.0"}]}"#).unwrap();
        assert_eq!(dependency_uuids(&manifest).unwrap(), vec!["bbbbbbbb-2222-3333-4444-555555555555".to_string()]);
        let header = r#"{"header":{"uuid":"AAAAAAAA-2222-3333-4444-555555555555","version":[1,0,0]},"dependencies":["#;
        for dependency in [
            r#"{"unknown":true}"#,
            r#"{"uuid":"BBBBBBBB-2222-3333-4444-555555555555"}"#,
            r#"{"uuid":"BBBBBBBB-2222-3333-4444-555555555555","uuid":"CCCCCCCC-2222-3333-4444-555555555555","version":[1,0,0]}"#,
            r#"{"uuid":"BBBBBBBB-2222-3333-4444-555555555555","module_name":"@minecraft/server","version":[1,0,0]}"#,
            r#"{"uuid":"BBBBBBBB-2222-3333-4444-555555555555","version":[65536,0,0]}"#,
            r#"{"module_name":"@minecraft/","version":"1.0.0"}"#,
            r#"{"module_name":"@minecraft/server","version":""}"#,
        ] {
            let json = format!("{header}{dependency}]}}");
            assert!(serde_json::from_str::<PackManifest>(&json)
                .map_or(true, |manifest| dependency_uuids(&manifest).is_err()));
        }
    }

    #[test]
    fn confirmation_fingerprint_binds_slot_profile_and_selection() {
        let parent = super::super::fresh_root(&std::env::temp_dir()).unwrap();
        let private = open_directory(&parent, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY).unwrap();
        let root = parent.join("Roaming/Minecraft Education Edition/games/com.mojang/minecraftWorlds");
        for name in ["one", "two"] {
            let world = root.join(name);
            fs::create_dir_all(&world).unwrap();
            fs::write(world.join("level.dat"), b"fixture").unwrap();
        }
        let select = |name: &str| ReviewSelection {
            category: InventoryCategory::Worlds, root: "minecraftWorlds".into(), name: name.into(),
        };
        let budget = || InspectionBudget::new(Duration::from_secs(2));
        let mut review = review_selection(&private, &[select("one"), select("two")], budget()).unwrap();
        let token = review.confirmation_token(1).unwrap();
        assert_eq!(token.len(), 64);
        assert_ne!(token, review.confirmation_token(2).unwrap());
        assert!(review.confirmation_token(0).is_err());
        review.items.reverse();
        review.tree_digests.reverse();
        assert_eq!(token, review.confirmation_token(1).unwrap());
        assert_ne!(token, review_selection(&private, &[select("one")], budget()).unwrap().confirmation_token(1).unwrap());
        // Preserve the old directory so the filesystem cannot reuse its ID.
        fs::rename(root.join("one"), root.join("old")).unwrap();
        fs::create_dir(root.join("one")).unwrap();
        fs::write(root.join("one/level.dat"), b"fixture").unwrap();
        let changed = review_selection(&private, &[select("one"), select("two")], budget()).unwrap();
        assert_ne!(token, changed.confirmation_token(1).unwrap());
        review.warnings.push("Incomplete dependency coverage".into());
        assert!(review.confirmation_token(1).is_err());
        drop(private);
        fs::remove_dir_all(parent).unwrap();
    }

    #[test]
    fn selected_world_disposition_preserves_other_worlds_profiles_and_settings() {
        let parent = super::super::fresh_root(&std::env::temp_dir()).unwrap();
        let other = super::super::fresh_root(&std::env::temp_dir()).unwrap();
        let private = open_directory(&parent, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY).unwrap();
        let relative = "Roaming/Minecraft Education Edition/games/com.mojang";
        let base = parent.join(relative);
        let selected = base.join("minecraftWorlds/one");
        let sibling = base.join("minecraftWorlds/two");
        let foreign = other.join(relative).join("minecraftWorlds/one");
        for world in [&selected, &sibling, &foreign] {
            fs::create_dir_all(world.join("db")).unwrap();
            fs::write(world.join("level.dat"), b"fixture").unwrap();
            fs::write(world.join("db/data"), b"payload").unwrap();
        }
        fs::create_dir_all(base.join("minecraftpe")).unwrap();
        fs::write(base.join("minecraftpe/options.txt"), b"preserve").unwrap();
        let selection = ReviewSelection {
            category: InventoryCategory::Worlds, root: "minecraftWorlds".into(), name: "one".into(),
        };
        let budget = || InspectionBudget::new(Duration::from_secs(2));
        let review = review_selection(&private, &[selection], budget()).unwrap();
        let plan = super::super::profile_storage::preflight_selected_worlds(&private, &review, budget()).unwrap();
        plan.dispose().unwrap();
        assert!(!selected.exists());
        assert_eq!(fs::read(sibling.join("db/data")).unwrap(), b"payload");
        assert_eq!(fs::read(foreign.join("db/data")).unwrap(), b"payload");
        assert_eq!(fs::read(base.join("minecraftpe/options.txt")).unwrap(), b"preserve");
        drop(private);
        fs::remove_dir_all(parent).unwrap();
        fs::remove_dir_all(other).unwrap();
    }

    #[test]
    fn exclusive_world_preflight_never_deletes_on_drop_or_changed_totals() {
        let parent = super::super::fresh_root(&std::env::temp_dir()).unwrap();
        let private = open_directory(&parent, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY).unwrap();
        let world = parent.join("Roaming/Minecraft Education Edition/games/com.mojang/minecraftWorlds/one");
        fs::create_dir_all(&world).unwrap();
        let leaf = world.join("level.dat");
        fs::write(&leaf, b"fixture").unwrap();
        let selection = ReviewSelection {
            category: InventoryCategory::Worlds, root: "minecraftWorlds".into(), name: "one".into(),
        };
        let budget = || InspectionBudget::new(Duration::from_secs(2));
        let review = review_selection(&private, &[selection], budget()).unwrap();
        let plan = super::super::profile_storage::preflight_selected_worlds(&private, &review, budget()).unwrap();
        assert_eq!(plan.totals(), (7, 1));
        drop(plan);
        assert_eq!(fs::read(&leaf).unwrap(), b"fixture");
        fs::rename(&leaf, world.join("old.dat")).unwrap();
        fs::write(&leaf, b"fixture").unwrap();
        fs::remove_file(world.join("old.dat")).unwrap();
        assert!(super::super::profile_storage::preflight_selected_worlds(&private, &review, budget()).is_err());
        assert_eq!(fs::read(&leaf).unwrap(), b"fixture");
        fs::write(&leaf, b"changed size").unwrap();
        assert!(super::super::profile_storage::preflight_selected_worlds(&private, &review, budget()).is_err());
        assert_eq!(fs::read(&leaf).unwrap(), b"changed size");
        drop(private);
        fs::remove_dir_all(parent).unwrap();
    }

    #[test]
    fn selective_review_rejects_unknown_and_duplicate_items() {
        let parent = super::super::fresh_root(&std::env::temp_dir()).unwrap();
        let private = open_directory(&parent, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY).unwrap();
        let world = parent.join("Roaming/Minecraft Education Edition/games/com.mojang/minecraftWorlds/one");
        fs::create_dir_all(&world).unwrap();
        fs::write(world.join("level.dat"), b"fixture").unwrap();
        let selection = || ReviewSelection {
            category: InventoryCategory::Worlds, root: "minecraftWorlds".into(), name: "one".into(),
        };
        let budget = || InspectionBudget::new(Duration::from_secs(2));
        let review = review_selection(&private, &[selection()], budget()).unwrap();
        assert_eq!(review.items.len(), 1);
        assert!(review.warnings.is_empty());
        assert_eq!((review.bytes, review.files), (7, 1));
        fs::hard_link(world.join("level.dat"), parent.join("outside")).unwrap();
        assert!(review_selection(&private, &[selection()], budget()).is_err());
        fs::remove_file(parent.join("outside")).unwrap();
        assert!(super::super::same_image(&review.profile_identity, &directory_id(&private).unwrap()));
        assert!(review_selection(&private, &[selection(), selection()], budget()).is_err());
        let unknown = ReviewSelection { name: "../outside".into(), ..selection() };
        assert!(review_selection(&private, &[unknown], budget()).is_err());
        assert!(review_selection(&private, &[], budget()).is_err());
        assert!(review_selection(&private, &[selection()], InspectionBudget::new(Duration::ZERO)).is_err());
        drop(private);
        fs::remove_dir_all(parent).unwrap();
    }

    #[test]
    fn individual_inventory_classifies_only_marked_directories_and_retains_identity() {
        let parent = super::super::fresh_root(&std::env::temp_dir()).unwrap();
        let private = open_directory(&parent, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY).unwrap();
        let base = parent.join("Roaming/Minecraft Education Edition/games/com.mojang");
        let world = base.join("minecraftWorlds/one");
        let pack = base.join("resource_packs/one");
        fs::create_dir_all(&world).unwrap();
        fs::create_dir_all(&pack).unwrap();
        fs::create_dir_all(base.join("minecraftWorlds/unclassified")).unwrap();
        fs::write(world.join("level.dat"), b"fixture").unwrap();
        fs::write(pack.join("manifest.json"), b"{}").unwrap();
        let worlds = individual_inventory(&private, InventoryCategory::Worlds, InspectionBudget::new(Duration::from_secs(2))).unwrap();
        let packs = individual_inventory(&private, InventoryCategory::ResourcePacks, InspectionBudget::new(Duration::from_secs(2))).unwrap();
        assert_eq!(worlds.len(), 1);
        assert_eq!(packs.len(), 1);
        assert_eq!(worlds[0].name, "one");
        assert_eq!(worlds[0].category, InventoryCategory::Worlds);
        let world_pin = open_directory(&world, FILE_READ_ATTRIBUTES).unwrap();
        assert!(super::super::same_image(&worlds[0].identity, &directory_id(&world_pin).unwrap()));
        assert!(!super::super::same_image(&worlds[0].identity, &packs[0].identity));
        assert!(individual_inventory(&private, InventoryCategory::Worlds, InspectionBudget::new(Duration::ZERO)).is_err());
        drop(world_pin);
        drop(private);
        fs::remove_dir_all(parent).unwrap();
    }

    #[test]
    fn reference_review_reads_both_categories_and_refuses_invalid_declarations() {
        let parent = super::super::fresh_root(&std::env::temp_dir()).unwrap();
        let private = open_directory(&parent, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY).unwrap();
        let world = parent.join("Roaming/Minecraft Education Edition/games/com.mojang/minecraftWorlds/one");
        fs::create_dir_all(&world).unwrap();
        let resource = world.join("world_resource_packs.json");
        let behavior = world.join("world_behavior_packs.json");
        fs::write(&resource, br#"[{"pack_id":"AAAAAAAA-2222-3333-4444-555555555555","version":[1,0,0]}]"#).unwrap();
        fs::write(&behavior, br#"[{"pack_id":"BBBBBBBB-2222-3333-4444-555555555555","version":[2,0,0]}]"#).unwrap();
        let refs = world_pack_references(&private, InspectionBudget::new(Duration::from_secs(2))).unwrap();
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].category, InventoryCategory::ResourcePacks);
        assert_eq!(refs[0].uuid, "aaaaaaaa-2222-3333-4444-555555555555");
        assert_eq!(refs[1].category, InventoryCategory::BehaviorPacks);
        assert_eq!(refs[1].version, [2,0,0]);
        assert_eq!(refs[1].world, "one");
        fs::write(&behavior, b"invalid").unwrap();
        assert!(world_pack_references(&private, InspectionBudget::new(Duration::from_secs(2))).is_err());
        assert!(world_pack_references(&private, InspectionBudget::new(Duration::ZERO)).is_err());
        drop(private);
        fs::remove_dir_all(parent).unwrap();
    }

    #[test]
    fn behavior_inventory_is_separate_from_resource_inventory() {
        let parent = super::super::fresh_root(&std::env::temp_dir()).unwrap();
        let private = open_directory(&parent, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY).unwrap();
        let base = parent.join("Roaming/Minecraft Education Edition/games/com.mojang");
        for root in ["behavior_packs", "development_behavior_packs", "resource_packs"] {
            let pack = base.join(root).join("one");
            fs::create_dir_all(&pack).unwrap();
            fs::write(pack.join("manifest.json"), b"{}").unwrap();
        }
        let behavior = individual_inventory(&private, InventoryCategory::BehaviorPacks,
            InspectionBudget::new(Duration::from_secs(2))).unwrap();
        assert_eq!(behavior.len(), 2);
        assert!(behavior.iter().all(|item| item.category == InventoryCategory::BehaviorPacks));
        assert_eq!(behavior[0].root, "behavior_packs");
        assert_eq!(behavior[1].root, "development_behavior_packs");
        let resource = individual_inventory(&private, InventoryCategory::ResourcePacks,
            InspectionBudget::new(Duration::from_secs(2))).unwrap();
        assert_eq!(resource.len(), 1);
        assert!(!super::super::same_image(&behavior[0].identity, &resource[0].identity));
        let declaration = base.join("world_behavior_packs.json");
        fs::write(&declaration, br#"[{"pack_id":"11111111-2222-3333-4444-555555555555","version":[1,2,3]}]"#).unwrap();
        let refs = read_pack_references(&declaration).unwrap().unwrap();
        assert_eq!(refs[0].pack_id, "11111111-2222-3333-4444-555555555555");
        assert_eq!(refs[0].version, [1,2,3]);
        drop(private);
        fs::remove_dir_all(parent).unwrap();
    }

    #[test]
    fn inventory_ancestor_chain_is_retained_until_report_finishes() {
        let parent = super::super::fresh_root(&std::env::temp_dir()).unwrap();
        let private = open_directory(&parent, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY).unwrap();
        let base = parent.join("Roaming/Minecraft Education Edition/games/com.mojang");
        fs::create_dir_all(&base).unwrap();
        let (_, pins) = base_path(&private, InspectionBudget::new(Duration::from_secs(2)))
            .unwrap().unwrap();
        assert_eq!(pins.len(), 5);
        assert!(base_path(&private, InspectionBudget::new(Duration::ZERO)).is_err());
        drop(pins);
        drop(private);
        fs::remove_dir_all(parent).unwrap();
    }

    #[test]
    fn exhausted_shared_budget_skips_resource_disk_work_and_reports_unknown_not_zero() {
        let parent = super::super::fresh_root(&std::env::temp_dir()).unwrap();
        let private = open_directory(&parent, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY).unwrap();

        // No amount of missing directory metadata establishes zero once
        // another slot has exhausted the shared inspection deadline.
        let empty = inspect(&private, InspectionBudget::new(Duration::ZERO));
        assert_eq!(empty.manifested, 0);
        assert!(empty.partial);
        let world = empty.world_declarations.unwrap();
        assert_eq!(world.pack_references, 0);
        assert!(world.partial && world.issue.is_some());
        assert!(!parent.join("Roaming").exists());

        let base = parent.join("Roaming").join("Minecraft Education Edition")
            .join("games").join("com.mojang");
        let pack = base.join("resource_packs/installed");
        fs::create_dir_all(&pack).unwrap();
        fs::write(pack.join("manifest.json"), b"{}").unwrap();
        let saved_world = base.join("minecraftWorlds/one");
        fs::create_dir_all(&saved_world).unwrap();
        fs::write(saved_world.join("world_resource_packs.json"),
            br#"[{"pack_id":"11111111-2222-3333-4444-555555555555","version":[1,2,3]}]"#).unwrap();

        // A valid populated folder must not be read under an expired
        // deadline, nor advertised as an authoritative zero.
        let expired = InspectionBudget::new(Duration::ZERO);
        assert!(count_manifested(&base, expired).is_err());
        assert!(scan_world_declarations(&base, expired).is_err());
        let blocked = inspect(&private, expired);
        assert!(blocked.partial && blocked.issue.is_some());
        assert_eq!(blocked.manifested, 0);
        let world = blocked.world_declarations.unwrap();
        assert!(world.partial && world.issue.is_some());
        assert_eq!(world.pack_references, 0);

        let completed = inspect(&private, InspectionBudget::new(Duration::from_secs(2)));
        assert!(!completed.partial);
        assert_eq!(completed.manifested, 1);
        assert!(!completed.world_declarations.as_ref().unwrap().partial);
        assert_eq!(completed.world_declarations.unwrap().pack_references, 1);
        drop(private);
        fs::remove_dir_all(parent).unwrap();
    }

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
