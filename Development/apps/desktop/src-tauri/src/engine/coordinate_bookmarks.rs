// Manual coordinate bookmarks belong to Lazy Tester's project Data root.
// They are NOT Minecraft player/world telemetry, a game-save editor, or a
// process identity registry. No clipboard, commands, game hooks or world I/O.
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs,
    io::{self, Read, Write},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

const MAX_RECORDS: usize = 100;
const MAX_FILE_BYTES: u64 = 65_536;
const ORIGIN: &str = "USER_PASTED_UNVERIFIED";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Dimension {
    Unknown,
    Overworld,
    Nether,
    TheEnd,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Bookmark {
    pub id: String,
    pub slot: u8,
    pub name: String,
    pub world_label: String,
    pub dimension: Dimension,
    pub xyz: String,
    pub origin: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredBookmarks {
    schema: u8,
    items: Vec<Bookmark>,
}

impl Default for StoredBookmarks {
    fn default() -> Self {
        Self { schema: 1, items: Vec::new() }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "operation", deny_unknown_fields)]
pub enum BookmarkAction {
    List,
    Save {
        slot: u8,
        name: String,
        #[serde(rename = "worldLabel")]
        world_label: String,
        dimension: Dimension,
        xyz: String,
    },
    Delete { id: String },
}

#[derive(Debug, Serialize)]
pub struct BookmarkReport {
    schema: u8,
    items: Vec<Bookmark>,
}

fn invalid(why: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, why)
}

fn label_ok(value: &str) -> bool {
    !value.is_empty() && value.len() <= 64 && value.trim() == value
        && !value.chars().any(char::is_control)
}

fn xyz_ok(value: &str) -> bool {
    if value.is_empty() || value.len() > 128 || value.trim() != value {
        return false;
    }
    let parts: Vec<&str> = value.split(' ').collect();
    parts.len() == 3 && parts.iter().all(|part| {
        let unsigned = part.strip_prefix('-')
            .or_else(|| part.strip_prefix('+')).unwrap_or(part);
        let mut digits = unsigned.split('.');
        let integer = digits.next().unwrap_or("");
        let fraction = digits.next();
        if integer.is_empty() || !integer.bytes().all(|b| b.is_ascii_digit())
            || fraction.is_some_and(|x| x.is_empty() || !x.bytes().all(|b| b.is_ascii_digit()))
            || digits.next().is_some() {
            return false;
        }
        let Ok(whole) = integer.parse::<u64>() else { return false; };
        whole <= 1_000_000_000 && (whole < 1_000_000_000
            || fraction.is_none_or(|x| x.bytes().all(|b| b == b'0')))
    })
}

fn validate(stored: &StoredBookmarks) -> io::Result<()> {
    if stored.schema != 1 || stored.items.len() > MAX_RECORDS {
        return Err(invalid("Coordinate bookmark schema or capacity is invalid"));
    }
    let mut ids = HashSet::new();
    for item in &stored.items {
        if item.id.len() != 32 || !item.id.bytes().all(|b| b.is_ascii_hexdigit())
            || !ids.insert(&item.id) || !(1..=5).contains(&item.slot)
            || !label_ok(&item.name) || !label_ok(&item.world_label)
            || !xyz_ok(&item.xyz) || item.origin != ORIGIN {
            return Err(invalid("Saved coordinate bookmarks are invalid; existing data was preserved"));
        }
    }
    Ok(())
}

fn read_at(path: &Path) -> io::Result<StoredBookmarks> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(StoredBookmarks::default()),
        Err(error) => return Err(error),
        Ok(meta) => {
            if !meta.is_file() || meta.file_type().is_symlink() {
                return Err(invalid("Coordinate bookmarks must be an ordinary file"));
            }
            #[cfg(windows)]
            {
                use std::os::windows::fs::MetadataExt;
                if meta.file_attributes() & 0x400 != 0 {
                    return Err(invalid("Coordinate bookmarks may not be a reparse point"));
                }
            }
        }
    }
    let mut bytes = Vec::new();
    fs::File::open(path)?.take(MAX_FILE_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_FILE_BYTES {
        return Err(invalid("Saved coordinate bookmarks exceed the size limit"));
    }
    let stored: StoredBookmarks = serde_json::from_slice(&bytes)
        .map_err(|_| invalid("Saved coordinate bookmarks are malformed; existing data was preserved"))?;
    validate(&stored)?;
    Ok(stored)
}

fn write_at(path: &Path, value: &StoredBookmarks) -> io::Result<()> {
    validate(value)?;
    // Reject replaced/symlinked/corrupt existing data instead of resetting it.
    read_at(path)?;
    let parent = path.parent().ok_or_else(|| invalid("Coordinate data directory unavailable"))?;
    fs::create_dir_all(parent)?;
    let bytes = serde_json::to_vec_pretty(value).map_err(io::Error::other)?;
    if bytes.len() as u64 > MAX_FILE_BYTES {
        return Err(invalid("Saved coordinate bookmarks exceed the size limit"));
    }
    let clock = SystemTime::now().duration_since(UNIX_EPOCH).map_err(io::Error::other)?;
    let temp = path.with_extension(format!("{}-{}.tmp", std::process::id(), clock.as_nanos()));
    let result = (|| {
        let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&temp)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temp, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

fn new_id(existing: &[Bookmark]) -> io::Result<String> {
    for nonce in 0..16_u128 {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_err(io::Error::other)?.as_nanos();
        let value = (nanos << 16) ^ ((std::process::id() as u128) << 8) ^ nonce;
        let id = format!("{value:032x}");
        if !existing.iter().any(|bookmark| bookmark.id == id) {
            return Ok(id);
        }
    }
    Err(invalid("Could not allocate a unique bookmark identifier"))
}

fn handle_at(path: &Path, action: BookmarkAction) -> io::Result<BookmarkReport> {
    let mut stored = read_at(path)?;
    match action {
        BookmarkAction::List => {}
        BookmarkAction::Save { slot, name, world_label, dimension, xyz } => {
            if !(1..=5).contains(&slot) || !label_ok(&name)
                || !label_ok(&world_label) || !xyz_ok(&xyz) {
                return Err(invalid("Bookmark requires MC01-MC05, a name, a world label and exact XYZ"));
            }
            if stored.items.len() >= MAX_RECORDS {
                return Err(invalid("Saved coordinate bookmark limit reached (100)"));
            }
            let id = new_id(&stored.items)?;
            stored.items.push(Bookmark {
                id, slot, name, world_label, dimension, xyz, origin: ORIGIN.into(),
            });
            write_at(path, &stored)?;
        }
        BookmarkAction::Delete { id } => {
            if id.len() != 32 || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(invalid("Invalid coordinate bookmark identifier"));
            }
            let before = stored.items.len();
            stored.items.retain(|bookmark| bookmark.id != id);
            if stored.items.len() == before {
                return Err(invalid("Coordinate bookmark no longer exists"));
            }
            write_at(path, &stored)?;
        }
    }
    Ok(BookmarkReport { schema: 1, items: stored.items })
}

pub fn handle(action: BookmarkAction) -> io::Result<BookmarkReport> {
    // Reuses the existing symlink/reparse-checked, app-root Data authority.
    // No per-game Roaming, Minecraft world database or saved options are read.
    let path = super::manager_settings::product_coordinates()?.join("bookmarks.json");
    handle_at(&path, action)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn temp_path() -> std::path::PathBuf {
        std::env::temp_dir().join(format!("lazy-coordinate-{}-{}",
            std::process::id(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()))
            .join("bookmarks.json")
    }
    fn save(xyz: &str) -> BookmarkAction {
        BookmarkAction::Save {
            slot: 2, name: "QA Spawn".into(), world_label: "Test world".into(),
            dimension: Dimension::Unknown, xyz: xyz.into(),
        }
    }
    #[test]
    fn manual_bookmarks_are_explicit_bounded_and_durable() {
        let path = temp_path();
        assert!(handle_at(&path, BookmarkAction::List).unwrap().items.is_empty());
        assert!(!path.exists());
        assert!(handle_at(&path, save("-45.125 64 123.456")).is_ok());
        let found = handle_at(&path, BookmarkAction::List).unwrap();
        assert_eq!(found.items.len(), 1);
        assert_eq!(found.items[0].xyz, "-45.125 64 123.456");
        assert_eq!(found.items[0].origin, ORIGIN);
        assert_eq!(found.items[0].slot, 2);
        assert!(handle_at(&path, save("1000000000.000000001 64 3")).is_err());
        assert!(handle_at(&path, save("1e3 64 3")).is_err());
        assert_eq!(handle_at(&path, BookmarkAction::List).unwrap().items.len(), 1);
        let id = found.items[0].id.clone();
        assert!(handle_at(&path, BookmarkAction::Delete { id }).unwrap().items.is_empty());
        assert!(handle_at(&path, BookmarkAction::List).unwrap().items.is_empty());
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
    #[test]
    fn corrupt_existing_data_is_not_overwritten() {
        let path = temp_path();
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, b"{invalid-json").unwrap();
        assert!(handle_at(&path, save("1 2 3")).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"{invalid-json");
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
    #[test]
    fn exact_coordinate_bounds_and_negative_fraction_are_preserved() {
        assert!(xyz_ok("-0.0000001 64 -1000000000"));
        assert!(!xyz_ok("1000000000.1 64 3"));
        assert!(!xyz_ok("1 2 3\n/tp @s 4 5 6"));
        assert!(!xyz_ok("1  2 3"));
        assert!(!xyz_ok("1,2,3"));
    }
}
