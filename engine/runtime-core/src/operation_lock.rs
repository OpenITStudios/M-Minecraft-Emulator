use crate::{paths::runtime_root, persistence::reject_unsafe_existing_file};
use std::{
    fs::{self, File, OpenOptions},
    io,
};

pub(crate) struct OperationLock {
    file: File,
}

impl OperationLock {
    fn open() -> io::Result<File> {
        let root = runtime_root()?;
        fs::create_dir_all(&root)?;
        let path = root.join(".operation.lock");
        reject_unsafe_existing_file(&path)?;
        OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(path)
    }

    pub(crate) fn acquire() -> io::Result<Self> {
        let file = Self::open()?;
        file.try_lock().map_err(io::Error::from).map_err(|error| {
            if error.kind() == io::ErrorKind::WouldBlock {
                io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "another Virtual Clients operation is already running",
                )
            } else {
                error
            }
        })?;
        Ok(Self { file })
    }

    pub(crate) fn acquire_shared() -> io::Result<Self> {
        let file = Self::open()?;
        file.try_lock_shared().map_err(io::Error::from).map_err(|error| {
            if error.kind() == io::ErrorKind::WouldBlock {
                io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "Virtual Clients state is changing; try again after the current operation finishes",
                )
            } else {
                error
            }
        })?;
        Ok(Self { file })
    }
}

impl Drop for OperationLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::OperationLock;
    use std::{
        env, fs, io,
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn cross_process_readers_writers_and_drop_keep_lock_ownership() {
        const MODE: &str = "MIVUBI_OPERATION_LOCK_TEST_MODE";
        let test_name =
            "operation_lock::tests::cross_process_readers_writers_and_drop_keep_lock_ownership";
        let run = |mode: &str, root: &std::path::Path| {
            let result = Command::new(env::current_exe().unwrap())
                .args(["--exact", test_name, "--nocapture"])
                .env(MODE, mode)
                .env("LOCALAPPDATA", root)
                .output()
                .unwrap();
            assert!(
                result.status.success(),
                "{mode}: {}",
                String::from_utf8_lossy(&result.stderr)
            );
        };
        let mode = env::var(MODE).ok();
        match mode.as_deref() {
            None => {
                let root = env::temp_dir().join(format!(
                    "operation-lock-{}-{}",
                    std::process::id(),
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_nanos()
                ));
                fs::create_dir_all(&root).unwrap();
                run("exercise", &root);
                fs::remove_dir_all(root).unwrap();
            }
            Some("exercise") => {
                let root = env::var_os("LOCALAPPDATA").unwrap();
                let root = std::path::Path::new(&root);
                let writer = OperationLock::acquire().unwrap();
                run("blocked-reader", root);
                run("blocked-writer", root);
                drop(writer);
                let reader = OperationLock::acquire_shared().unwrap();
                run("allowed-reader", root);
                run("blocked-writer", root);
                drop(reader);
                run("allowed-writer", root);
                // Child exit and Drop must release ownership for a fresh writer.
                assert!(OperationLock::acquire().is_ok());
            }
            Some("blocked-reader") => assert_eq!(
                OperationLock::acquire_shared().err().unwrap().kind(),
                io::ErrorKind::WouldBlock
            ),
            Some("blocked-writer") => assert_eq!(
                OperationLock::acquire().err().unwrap().kind(),
                io::ErrorKind::WouldBlock
            ),
            Some("allowed-reader") => {
                assert!(OperationLock::acquire_shared().is_ok());
            }
            Some("allowed-writer") => {
                assert!(OperationLock::acquire().is_ok());
            }
            Some(_) => panic!("Unknown lock test mode"),
        }
    }
}
