use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

#[cfg(windows)]
use std::os::windows::fs::MetadataExt;
#[cfg(windows)]
const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x00000400;

fn backup_path(path: &Path) -> io::Result<PathBuf> {
    let parent = path.parent().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "metadata path has no parent")
    })?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "metadata filename is invalid")
        })?;
    Ok(parent.join(format!(".{name}.bak")))
}

fn temporary_path(path: &Path) -> io::Result<PathBuf> {
    let parent = path.parent().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "metadata path has no parent")
    })?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "metadata filename is invalid")
        })?;
    Ok(parent.join(format!(".{name}.{}.tmp", std::process::id())))
}

fn unresolved_temporary(path: &Path) -> io::Result<Option<PathBuf>> {
    let parent = path.parent().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "metadata path has no parent")
    })?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "metadata filename is invalid")
        })?;
    let prefix = format!(".{name}.");
    for entry in fs::read_dir(parent)? {
        let entry = entry?;
        let filename = entry.file_name();
        let Some(filename) = filename.to_str() else {
            continue;
        };
        let Some(pid) = filename
            .strip_prefix(&prefix)
            .and_then(|value| value.strip_suffix(".tmp"))
        else {
            continue;
        };
        if pid.is_empty() || !pid.bytes().all(|byte| byte.is_ascii_digit()) {
            continue;
        }
        let candidate = entry.path();
        reject_unsafe_existing_file(&candidate)?;
        return Ok(Some(candidate));
    }
    Ok(None)
}

pub(crate) fn reject_unsafe_existing_file(path: &Path) -> io::Result<()> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    if metadata.file_type().is_symlink()
        || is_reparse_point(&metadata)
        || !metadata.file_type().is_file()
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("metadata path is not a regular file: {}", path.display()),
        ));
    }
    Ok(())
}

#[cfg(windows)]
fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_reparse_point(_metadata: &fs::Metadata) -> bool {
    false
}

// Two independent read-only queries may both observe a crash backup while
// holding shared runtime locks. Losing the rename/remove race is not metadata
// corruption if another reader already completed the identical recovery.
fn peer_completed_recovery(path: &Path, backup: &Path, error: io::Error) -> io::Result<()> {
    if error.kind() == io::ErrorKind::NotFound && !backup.exists() {
        reject_unsafe_existing_file(path)?;
        if path.is_file() {
            return Ok(());
        }
    }
    Err(error)
}

fn recover_interrupted_replace(path: &Path) -> io::Result<()> {
    let backup = backup_path(path)?;
    reject_unsafe_existing_file(path)?;
    reject_unsafe_existing_file(&backup)?;

    match (path.exists(), backup.exists()) {
        (false, true) => {
            fs::rename(&backup, path).or_else(|error| peer_completed_recovery(path, &backup, error))
        }
        (true, true) => {
            // A crash may leave an incomplete primary beside the original.
            // Preserve the last candidate backup if the primary is corrupt.
            let current = fs::read_to_string(path)?;
            if path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
            {
                serde_json::from_str::<serde_json::Value>(&current).map_err(|error| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "replacement JSON is invalid; recovery backup preserved at {}: {error}",
                            backup.display()
                        ),
                    )
                })?;
            }
            fs::remove_file(&backup).or_else(|error| peer_completed_recovery(path, &backup, error))
        }
        _ => Ok(()),
    }
}

pub(crate) fn read_text_recovering(path: &Path) -> io::Result<String> {
    recover_interrupted_replace(path)?;
    fs::read_to_string(path)
}

pub(crate) fn write_text_transactional(path: &Path, contents: &str) -> io::Result<()> {
    let parent = path.parent().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidInput, "metadata path has no parent")
    })?;
    fs::create_dir_all(parent)?;

    // A leftover temporary file may contain the only copy of a crashed write.
    // Do not delete or reuse it (including after PID reuse); leave both old
    // metadata and unfinished content available for explicit reconciliation.
    let temporary = temporary_path(path)?;
    if let Some(unresolved) = unresolved_temporary(path)? {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "interrupted metadata write preserved at {}; inspect before retrying",
                unresolved.display()
            ),
        ));
    }

    recover_interrupted_replace(path)?;
    let backup = backup_path(path)?;
    reject_unsafe_existing_file(&backup)?;

    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temporary)?;
    file.write_all(contents.as_bytes())?;
    file.sync_all()?;
    drop(file);

    if backup.exists() {
        fs::remove_file(&backup)?;
    }

    let had_existing = path.exists();
    if had_existing {
        fs::rename(path, &backup)?;
    }

    match fs::rename(&temporary, path) {
        Ok(()) => {
            if had_existing && backup.exists() {
                fs::remove_file(backup)?;
            }
            Ok(())
        }
        Err(error) => {
            // Preserve the complete new candidate after failed promotion.
            if had_existing && backup.exists() && !path.exists() {
                if let Err(restore_error) = fs::rename(&backup, path) {
                    return Err(io::Error::new(
                        error.kind(),
                        format!("metadata promotion failed ({error}); original backup and candidate preserved; restore failed ({restore_error})"),
                    ));
                }
            }
            Err(io::Error::new(
                error.kind(),
                format!(
                    "metadata promotion failed ({error}); temporary candidate preserved at {}",
                    temporary.display()
                ),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{backup_path, read_text_recovering, write_text_transactional};
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn transactional_write_replaces_existing_metadata() {
        let root = unique_temp_dir("replace");
        let path = root.join("state.json");

        write_text_transactional(&path, "one\n").unwrap();
        write_text_transactional(&path, "two\n").unwrap();

        assert_eq!(read_text_recovering(&path).unwrap(), "two\n");
        assert!(!backup_path(&path).unwrap().exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn read_recovers_interrupted_backup_state() {
        let root = unique_temp_dir("recover");
        fs::create_dir_all(&root).unwrap();
        let path = root.join("state.json");
        let backup = backup_path(&path).unwrap();

        fs::write(&backup, "old\n").unwrap();
        assert_eq!(read_text_recovering(&path).unwrap(), "old\n");
        assert!(path.is_file());
        assert!(!backup.exists());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn previous_process_temporary_metadata_blocks_replacement() {
        let root = unique_temp_dir("old-pid-temporary");
        fs::create_dir_all(&root).unwrap();
        let path = root.join("state.json");
        let current_pid = std::process::id();
        let previous_pid = if current_pid == 1 { 2 } else { current_pid - 1 };
        let temporary = root.join(format!(".state.json.{previous_pid}.tmp"));
        fs::write(&path, r#"{"original":true}"#).unwrap();
        fs::write(&temporary, r#"{"candidate":true}"#).unwrap();
        assert_eq!(
            write_text_transactional(&path, r#"{"replacement":true}"#)
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::AlreadyExists
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), r#"{"original":true}"#);
        assert_eq!(
            fs::read_to_string(&temporary).unwrap(),
            r#"{"candidate":true}"#
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unfinished_temporary_metadata_is_never_silently_deleted() {
        let root = unique_temp_dir("preserve-temporary");
        fs::create_dir_all(&root).unwrap();
        let path = root.join("state.json");
        let temporary = super::temporary_path(&path).unwrap();
        fs::write(&path, r#"{"account":"original"}"#).unwrap();
        fs::write(&temporary, r#"{"account":"unfinished"}"#).unwrap();

        let error = write_text_transactional(&path, r#"{"account":"replacement"}"#).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            r#"{"account":"original"}"#
        );
        assert_eq!(
            fs::read_to_string(&temporary).unwrap(),
            r#"{"account":"unfinished"}"#
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn corrupted_primary_keeps_backup_until_explicit_recovery() {
        let root = unique_temp_dir("preserve-backup");
        fs::create_dir_all(&root).unwrap();
        let path = root.join("state.json");
        let backup = backup_path(&path).unwrap();
        fs::write(&path, "{not valid json").unwrap();
        fs::write(&backup, r#"{"account":"original"}"#).unwrap();

        assert_eq!(
            read_text_recovering(&path).unwrap_err().kind(),
            std::io::ErrorKind::InvalidData
        );
        assert_eq!(
            write_text_transactional(&path, r#"{"account":"replacement"}"#)
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::InvalidData
        );
        assert_eq!(
            fs::read_to_string(&backup).unwrap(),
            r#"{"account":"original"}"#
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), "{not valid json");

        fs::remove_file(&path).unwrap();
        assert_eq!(
            read_text_recovering(&path).unwrap(),
            r#"{"account":"original"}"#
        );
        assert!(!backup.exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn read_discards_stale_backup_after_successful_replace() {
        let root = unique_temp_dir("stale-backup");
        fs::create_dir_all(&root).unwrap();
        let path = root.join("state.json");
        let backup = backup_path(&path).unwrap();

        fs::write(&path, r#"{"state":"new"}"#).unwrap();
        fs::write(&backup, r#"{"state":"old"}"#).unwrap();

        assert_eq!(read_text_recovering(&path).unwrap(), r#"{"state":"new"}"#);
        assert!(!backup.exists());

        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn metadata_symlink_is_rejected() {
        use std::os::unix::fs::symlink;
        let root = unique_temp_dir("symlink");
        fs::create_dir_all(&root).unwrap();
        let target = root.join("target.txt");
        let path = root.join("state.json");
        fs::write(&target, "private\n").unwrap();
        symlink(&target, &path).unwrap();

        assert!(read_text_recovering(&path).is_err());
        assert!(write_text_transactional(&path, "replacement\n").is_err());
        assert_eq!(fs::read_to_string(&target).unwrap(), "private\n");

        fs::remove_dir_all(root).unwrap();
    }

    fn unique_temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "m-bedrock-virtual-clients-persistence-{label}-{}-{nonce}",
            std::process::id()
        ))
    }
    #[test]
    fn peer_backup_recovery_is_idempotent_but_never_clobbers_a_remaining_backup() {
        let root = std::env::temp_dir().join(format!("vc-peer-recovery-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("metadata.json");
        let backup = super::backup_path(&path).unwrap();
        fs::write(&path, r#"{"complete":true}"#).unwrap();

        // The other reader already restored a regular primary and consumed .bak.
        super::peer_completed_recovery(
            &path,
            &backup,
            std::io::Error::from(std::io::ErrorKind::NotFound),
        )
        .unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), r#"{"complete":true}"#);

        // Still-present backup cannot be dismissed as a harmless losing reader.
        fs::write(&backup, r#"{"previous":true}"#).unwrap();
        assert!(super::peer_completed_recovery(
            &path,
            &backup,
            std::io::Error::from(std::io::ErrorKind::NotFound),
        )
        .is_err());
        assert_eq!(fs::read_to_string(&backup).unwrap(), r#"{"previous":true}"#);
        fs::remove_dir_all(root).unwrap();
    }
}
