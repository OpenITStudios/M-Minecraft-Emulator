use crate::{
    error::{ErrorCode, ErrorReport},
    paths::runtime_root,
    persistence::{read_text_recovering, reject_unsafe_existing_file, write_text_transactional},
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File, OpenOptions},
    io, thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const HISTORY_LIMIT: usize = 200;
const HISTORY_SCHEMA: u32 = 1;
const HISTORY_LOCK_ATTEMPTS: usize = 5;
const HISTORY_LOCK_INTERVAL: Duration = Duration::from_millis(20);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OperationKind {
    StageUpdate,
    RegisterBase,
    OpenBaseFinalization,
    Provision,
    Reprovision,
    VerifyIdentities,
    Start,
    Suspend,
    Stop,
    Restart,
    SetReady,
    Reset,
    Open,
    LaunchMinecraft,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OperationOutcome {
    Success,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationRecord {
    pub schema: u32,
    pub timestamp_unix_ms: u64,
    pub operation: OperationKind,
    pub target: Option<String>,
    pub outcome: OperationOutcome,
    pub error_code: Option<ErrorCode>,
    pub retryable: bool,
}

fn timestamp_ms() -> io::Result<u64> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?
        .as_millis();
    u64::try_from(millis)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "system time is out of range"))
}

fn history_path() -> io::Result<std::path::PathBuf> {
    Ok(runtime_root()?.join("operation-history.json"))
}

fn lock_history_with_deadline(file: &File) -> io::Result<()> {
    for attempt in 0..HISTORY_LOCK_ATTEMPTS {
        match file.try_lock().map_err(io::Error::from) {
            Ok(()) => return Ok(()),
            Err(error)
                if error.kind() == io::ErrorKind::WouldBlock
                    && attempt + 1 < HISTORY_LOCK_ATTEMPTS =>
            {
                thread::sleep(HISTORY_LOCK_INTERVAL);
            }
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::WouldBlock,
        "operation history is busy",
    ))
}

// Reading may recover .bak files; reads must not race a writer's rename.
fn with_history_lock<T>(operation: impl FnOnce() -> io::Result<T>) -> io::Result<T> {
    let root = runtime_root()?;
    fs::create_dir_all(&root)?;
    let lock_path = root.join(".operation-history.lock");
    reject_unsafe_existing_file(&lock_path)?;
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&lock_path)?;
    lock_history_with_deadline(&file)?;
    let outcome = operation();
    let release = file.unlock();
    match (outcome, release) {
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error),
        (Ok(value), Ok(())) => Ok(value),
    }
}

fn read_history_at(path: &std::path::Path) -> io::Result<Vec<OperationRecord>> {
    // Backup recovery must precede "missing history" classification.
    match read_text_recovering(path) {
        Ok(raw) => serde_json::from_str(&raw)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(error),
    }
}

fn read_operation_history_unlocked() -> io::Result<Vec<OperationRecord>> {
    read_history_at(&history_path()?)
}

pub fn read_operation_history() -> io::Result<Vec<OperationRecord>> {
    with_history_lock(read_operation_history_unlocked)
}

fn record_operation_inner<T>(
    operation: OperationKind,
    target: Option<String>,
    result: &io::Result<T>,
) -> io::Result<()> {
    let (outcome, error_code, retryable) = match result {
        Ok(_) => (OperationOutcome::Success, None, false),
        Err(error) => {
            let report = ErrorReport::from_io(error);
            (
                OperationOutcome::Failed,
                Some(report.code),
                report.retryable,
            )
        }
    };
    let timestamp_unix_ms = timestamp_ms()?;

    with_history_lock(|| {
        // Never discard a corrupt history file to manufacture a fresh record.
        let mut history = read_operation_history_unlocked()?;
        history.push(OperationRecord {
            schema: HISTORY_SCHEMA,
            timestamp_unix_ms,
            operation,
            target,
            outcome,
            error_code,
            retryable,
        });
        if history.len() > HISTORY_LIMIT {
            let remove = history.len() - HISTORY_LIMIT;
            history.drain(0..remove);
        }
        let json = serde_json::to_string_pretty(&history)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
        write_text_transactional(&history_path()?, &format!("{json}\n"))
    })
}

// The VM result must not be replayed because best-effort journaling failed.
// Emit only an error kind, never guest paths, user identifiers or tokens.
pub(crate) fn record_operation<T>(
    operation: OperationKind,
    target: Option<String>,
    result: &io::Result<T>,
) {
    if let Err(error) = record_operation_inner(operation, target, result) {
        eprintln!(
            "Virtual Clients operation history unavailable ({:?}); VM result unchanged",
            error.kind()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{
        read_history_at, OperationKind, OperationOutcome, OperationRecord, HISTORY_LIMIT,
        HISTORY_LOCK_ATTEMPTS, HISTORY_LOCK_INTERVAL,
    };
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn lock_contention_has_a_small_explicit_budget() {
        assert!(HISTORY_LOCK_ATTEMPTS > 1);
        assert!(HISTORY_LOCK_ATTEMPTS <= 5);
        assert!(HISTORY_LOCK_INTERVAL.as_millis() <= 20);
    }

    #[test]
    fn backup_only_history_is_restored_not_discarded() {
        let root = std::env::temp_dir().join(format!(
            "history-backup-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("operation-history.json");
        let backup = root.join(".operation-history.json.bak");
        let record = OperationRecord {
            schema: 1,
            timestamp_unix_ms: 7,
            operation: OperationKind::Start,
            target: Some("Virtual-01".into()),
            outcome: OperationOutcome::Success,
            error_code: None,
            retryable: false,
        };
        fs::write(&backup, serde_json::to_string(&vec![record]).unwrap()).unwrap();
        let restored = read_history_at(&path).unwrap();
        assert_eq!(restored.len(), 1);
        assert_eq!(restored[0].timestamp_unix_ms, 7);
        assert!(path.is_file());
        assert!(!backup.exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn history_contract_is_bounded_and_non_sensitive() {
        assert_eq!(HISTORY_LIMIT, 200);
        let record = OperationRecord {
            schema: 1,
            timestamp_unix_ms: 1,
            operation: OperationKind::Start,
            target: Some("count:3".into()),
            outcome: OperationOutcome::Success,
            error_code: None,
            retryable: false,
        };
        let json = serde_json::to_string(&record).unwrap();
        assert!(!json.contains("password"));
        assert!(!json.contains("token"));
        assert!(!json.contains("email"));
    }

    #[test]
    fn history_contention_returns_would_block_and_releases_for_retry() {
        use std::{fs::OpenOptions, io, time::Instant};
        let root = std::env::temp_dir().join(format!(
            "history-contention-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("history.lock");
        let owner = OpenOptions::new()
            .create_new(true)
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        let contender = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        owner.try_lock().unwrap();
        let started = Instant::now();
        let error = super::lock_history_with_deadline(&contender).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::WouldBlock);
        assert!(started.elapsed() >= HISTORY_LOCK_INTERVAL * (HISTORY_LOCK_ATTEMPTS as u32 - 1));
        owner.unlock().unwrap();
        super::lock_history_with_deadline(&contender).unwrap();
        contender.unlock().unwrap();
        drop(contender);
        drop(owner);
        fs::remove_dir_all(root).unwrap();
    }
}
