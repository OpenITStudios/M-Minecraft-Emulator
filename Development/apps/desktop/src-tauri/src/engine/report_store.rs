//! Single product desktop checkpoint. CLI/controller/helpers never enable it.
use serde::{Deserialize, Serialize};
use std::{fs::{self, File, OpenOptions}, io::{self, Read, Write},
    os::windows::{ffi::OsStrExt, fs::OpenOptionsExt}, path::Path,
    sync::{Mutex, atomic::{AtomicBool, Ordering}}};
use windows_sys::Win32::Storage::FileSystem::{
    FILE_READ_ATTRIBUTES, FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_SHARE_DELETE,
    FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
    FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT,
    MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
};
const MAX_BYTES: usize = 16 * 1024;
const MAX_UI_ROWS: usize = 48;
const MAX_JS_INTEGER: u64 = 9_007_199_254_740_991;
const UI_CODES: &[&str] = &["LT-APP-01", "LT-APP-02", "LT-PRO-01", "LT-CON-01", "LT-GAM-01", "LT-DIS-01", "LT-DIS-02", "LT-STO-01", "LT-STO-02", "LT-SET-01", "LT-SET-02", "LT-SET-03", "LT-GAM-02", "LT-DIS-03", "LT-GAM-03", "LT-DIS-04", "LT-PRO-02", "LT-STO-03", "LT-STO-04", "LT-STO-05", "LT-STO-06", "LT-SET-04", "LT-STO-07", "LT-APP-03", "LT-SET-05", "LT-SET-06", "LT-DIS-05", "LT-DIS-06", "LT-APP-04", "LT-GAM-04", "LT-APP-05", "LT-GAM-05", "LT-CAP-01", "LT-CAP-02", "LT-CAP-03", "LT-CAP-04", "LT-CAP-05", "LT-AGT-01", "LT-AGT-02", "LT-AGT-03", "LT-AGT-04", "LT-AGT-05", "LT-SET-07"];

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct UiEntry {
    c: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    s: Option<u32>,
    t: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    l: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    r: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    e: Option<u32>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UiSnapshot {
    v: u8,
    scope: String,
    issues: Vec<UiEntry>,
    #[serde(default)]
    truncated: bool,
}
fn invalid() -> io::Error { io::Error::new(io::ErrorKind::InvalidData, "Issue report checkpoint refused") }
fn parse_ui(text: &str) -> io::Result<UiSnapshot> {
    if text.len() > MAX_BYTES { return Err(invalid()); }
    let ui: UiSnapshot = serde_json::from_str(text).map_err(|_| invalid())?;
    if ui.v != 1 || ui.scope != "desktop-observed" || ui.issues.is_empty() {
        return Err(invalid());
    }
    validate_ui_rows(&ui.issues)?;
    Ok(ui)
}
fn validate_ui_rows(rows: &[UiEntry]) -> io::Result<()> {
    if rows.len() > MAX_UI_ROWS { return Err(invalid()); }
    for (i, row) in rows.iter().enumerate() {
        if !UI_CODES.contains(&row.c.as_str()) ||
            row.s.is_some_and(|s| !(1..=5).contains(&s)) ||
            row.t > MAX_JS_INTEGER || row.l.is_some_and(|t| t > MAX_JS_INTEGER) ||
            row.r.is_some_and(|t| t > MAX_JS_INTEGER) || row.n == Some(0) ||
            row.e == Some(0) || rows[..i].iter().any(|old| old.c == row.c && old.s == row.s) {
            return Err(invalid());
        }
    }
    Ok(())
}
#[derive(Serialize)]
struct Checkpoint<'a> {
    v: u8,
    scope: &'static str,
    issues: &'a [UiEntry],
    #[serde(skip_serializing_if = "is_false")]
    truncated: bool,
    native: Option<super::issue_report::NativeIssueReport>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredCheckpoint {
    v: u8,
    scope: String,
    issues: Vec<UiEntry>,
    #[serde(default)]
    truncated: bool,
    native: Option<super::issue_report::NativeIssueReport>,
}
fn validate_saved(bytes: &[u8]) -> io::Result<()> {
    let saved: StoredCheckpoint = serde_json::from_slice(bytes).map_err(|_| invalid())?;
    if saved.v != 2 || saved.scope != "desktop-process" ||
        (saved.issues.is_empty() && saved.native.is_none()) ||
        saved.native.as_ref().is_some_and(|native| !native.valid()) {
        return Err(invalid());
    }
    validate_ui_rows(&saved.issues)?;
    // Require the complete producer schema, including a present native
    // field, and reject explicit null/false optional fields it never emits.
    let actual: serde_json::Value = serde_json::from_slice(bytes).map_err(|_| invalid())?;
    let canonical = serde_json::to_value(Checkpoint { v: 2, scope: "desktop-process",
        issues: &saved.issues, truncated: saved.truncated, native: saved.native })
        .map_err(|_| invalid())?;
    if actual != canonical { return Err(invalid()); }
    Ok(())
}
fn is_false(value: &bool) -> bool { !*value }
struct Store { ui: Option<UiSnapshot> }
static STORE: Mutex<Store> = Mutex::new(Store { ui: None });
static ENABLED: AtomicBool = AtomicBool::new(false);
static DUE: AtomicBool = AtomicBool::new(false);
pub(crate) fn enable() { ENABLED.store(true, Ordering::Release); }
pub(crate) fn request_checkpoint() {
    if ENABLED.load(Ordering::Acquire) { DUE.store(true, Ordering::Release); }
}
pub(crate) fn checkpoint_if_due() {
    if DUE.swap(false, Ordering::AcqRel) { let _ = flush(); }
}
pub(crate) fn checkpoint_ui(text: Option<&str>) -> Result<(), &'static str> {
    if !ENABLED.load(Ordering::Acquire) { return Err("Issue report checkpoint unavailable"); }
    let ui = text.map(parse_ui).transpose().map_err(|_| "Invalid issue report checkpoint")?;
    let mut store = STORE.lock().map_err(|_| "Issue report checkpoint unavailable")?;
    // Explicit native-only copy must also checkpoint without clearing UI evidence.
    if let Some(ui) = ui { store.ui = Some(ui); }
    store.save().map_err(|_| "Issue report checkpoint unavailable")
}
pub(crate) fn flush() -> io::Result<()> {
    if !ENABLED.load(Ordering::Acquire) { return Ok(()); }
    STORE.lock().map_err(|_| invalid())?.save()
}
impl Store {
    fn save(&mut self) -> io::Result<()> {
        let native = super::issue_report::read().map_err(|_| invalid())?;
        if self.ui.is_none() && native.is_none() { return Ok(()); }
        let payload = Checkpoint { v: 2, scope: "desktop-process",
            issues: self.ui.as_ref().map_or(&[], |ui| ui.issues.as_slice()),
            truncated: self.ui.as_ref().is_some_and(|ui| ui.truncated), native };
        let bytes = serde_json::to_vec(&payload).map_err(io::Error::other)?;
        if bytes.len() > MAX_BYTES { return Err(invalid()); }
        let settings = super::manager_settings::product_settings()?;
        publish(settings.parent().ok_or_else(invalid)?, &bytes)
    }
}
fn pin_directory(path: &Path) -> io::Result<File> {
    let file = OpenOptions::new().access_mode(FILE_READ_ATTRIBUTES)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT).open(path)?;
    let attrs = super::windows_file_identity::attributes(&file)?;
    if attrs & FILE_ATTRIBUTE_DIRECTORY == 0 || attrs & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(invalid());
    }
    Ok(file)
}
fn existing_checkpoint(path: &Path) -> io::Result<Option<Vec<u8>>> {
    let mut file = match OpenOptions::new().read(true).share_mode(FILE_SHARE_READ | FILE_SHARE_DELETE)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT).open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let attrs = super::windows_file_identity::attributes(&file)?;
    let info = super::windows_file_identity::standard(&file)?;
    if attrs & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT) != 0 ||
        info.NumberOfLinks != 1 || info.DeletePending || info.EndOfFile < 0 ||
        info.EndOfFile as usize > MAX_BYTES { return Err(invalid()); }
    let mut bytes = Vec::new();
    Read::by_ref(&mut file).take((MAX_BYTES + 1) as u64).read_to_end(&mut bytes)?;
    if bytes.len() > MAX_BYTES { return Err(invalid()); }
    validate_saved(&bytes)?;
    Ok(Some(bytes))
}
fn publish(data: &Path, bytes: &[u8]) -> io::Result<()> {
    if !data.is_absolute() || bytes.is_empty() || bytes.len() > MAX_BYTES ||
        data.components().any(|part| matches!(part, std::path::Component::ParentDir |
            std::path::Component::CurDir)) { return Err(invalid()); }
    let mut pins = data.ancestors().collect::<Vec<_>>().into_iter().rev()
        .map(pin_directory).collect::<io::Result<Vec<_>>>()?;
    let directory = data.join("Reports");
    match fs::create_dir(&directory) {
        Ok(()) => {},
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {},
        Err(error) => return Err(error),
    }
    pins.push(pin_directory(&directory)?);
    let final_path = directory.join("issues.json");
    let pending = directory.join("issues.pending");
    // Compare the actual bounded file, not a cached digest. Explicit copy
    // cannot report success for a checkpoint removed or replaced externally.
    let previous = existing_checkpoint(&final_path)?;
    if previous.as_deref() == Some(bytes) { return Ok(()); }
    let replace = previous.is_some();
    // Fixed exclusive stage; interruption preserves both files and blocks
    // subsequent writes rather than deleting an unverified staging file.
    let mut file = OpenOptions::new().write(true).create_new(true).share_mode(0)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT).open(&pending)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    let from: Vec<u16> = pending.as_os_str().encode_wide().chain(Some(0)).collect();
    let to: Vec<u16> = final_path.as_os_str().encode_wide().chain(Some(0)).collect();
    let flags = MOVEFILE_WRITE_THROUGH | if replace { MOVEFILE_REPLACE_EXISTING } else { 0 };
    if unsafe { MoveFileExW(from.as_ptr(), to.as_ptr(), flags) } == 0 {
        return Err(io::Error::last_os_error());
    }
    drop(pins);
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strict_ui_checkpoint() {
        let valid = r#"{"v":1,"scope":"desktop-observed","issues":[{"c":"LT-CAP-01","s":2,"t":100,"n":2,"l":110}]}"#;
        assert!(parse_ui(valid).is_ok());
        for bad in [valid.replace("LT-CAP-01", "PRIVATE_PATH"),
            valid.replace("\"s\":2", "\"s\":9"), valid.replace("\"t\":100", "\"t\":100,\"raw\":\"secret\""),
            valid.replace("\"n\":2", "\"n\":0"), "x".repeat(MAX_BYTES + 1)] {
            assert!(parse_ui(&bad).is_err());
        }
    }
    #[test]
    fn prior_checkpoint_requires_complete_known_schema() {
        let valid = r#"{"v":2,"scope":"desktop-process","issues":[],"native":{"v":1,"issues":[{"c":"IPC_REPLY_TIMEOUT","op":"close_one","s":2,"t":100,"n":1}]}}"#;
        assert!(validate_saved(valid.as_bytes()).is_ok());
        for bad in [
            valid.replace("close_one", "unknown_op"),
            valid.replace("IPC_REPLY_TIMEOUT", "UNKNOWN_CODE"),
            valid.replace("\"n\":1", "\"n\":0"),
            valid.replace("\"t\":100", "\"t\":100,\"raw\":\"private\""),
            valid.replace("\"s\":2", "\"s\":null"),
            valid.replace("\"n\":1", "\"n\":2"),
            r#"{"v":2,"scope":"desktop-process","issues":[{"c":"LT-CON-01","t":1}]}"#.to_owned(),
            r#"{"v":2,"scope":"desktop-process","issues":[],"native":null}"#.to_owned(),
        ] { assert!(validate_saved(bad.as_bytes()).is_err()); }
    }
    #[test]
    fn bounded_publication_preserves_previous_and_refuses_interrupted_stage() {
        let root = tempfile::tempdir().unwrap();
        let first = br#"{"v":2,"scope":"desktop-process","issues":[{"c":"LT-CON-01","t":1}],"native":null}"#;
        let next = br#"{"v":2,"scope":"desktop-process","issues":[{"c":"LT-CON-01","t":1,"r":2}],"native":null}"#;
        publish(root.path(), first).unwrap();
        let path = root.path().join("Reports/issues.json");
        publish(root.path(), next).unwrap();
        assert_eq!(fs::read(&path).unwrap(), next);
        // An identical snapshot needs no staging. A deleted final must be recreated.
        fs::write(root.path().join("Reports/issues.pending"), b"interrupted").unwrap();
        publish(root.path(), next).unwrap();
        assert_eq!(fs::read(&path).unwrap(), next);
        assert!(publish(root.path(), first).is_err());
        assert_eq!(fs::read(&path).unwrap(), next);
        fs::remove_file(root.path().join("Reports/issues.pending")).unwrap();
        fs::remove_file(&path).unwrap();
        publish(root.path(), next).unwrap();
        assert_eq!(fs::read(&path).unwrap(), next);
        fs::write(&path, b"unrelated").unwrap();
        assert!(publish(root.path(), first).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"unrelated");
        let malformed = br#"{"v":2,"scope":"desktop-process","issues":[{"c":"PRIVATE","t":1}],"native":null}"#;
        fs::write(&path, malformed).unwrap();
        assert!(publish(root.path(), next).is_err());
        assert_eq!(fs::read(&path).unwrap(), malformed);
    }
}
