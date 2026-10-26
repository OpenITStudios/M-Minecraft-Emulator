use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::persistence::{read_text_recovering, write_text_transactional};

pub const CURRENT_RUNTIME_SCHEMA: u32 = 1;
pub const SCHEMA_FILE: &str = "runtime-schema.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSchema {
    pub schema: u32,
    pub created_by: String,
    pub last_migrated_by: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SchemaState {
    Missing,
    Ready,
    NewerThanApp,
    Invalid,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaStatus {
    pub state: SchemaState,
    pub schema: Option<u32>,
}

pub fn schema_path(root: &Path) -> PathBuf {
    root.join(SCHEMA_FILE)
}

pub fn inspect_runtime_schema(root: &Path) -> SchemaStatus {
    let path = schema_path(root);
    // Crash recovery may have left only a .bak file, including a newer schema.
    let raw = match read_text_recovering(&path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return SchemaStatus {
                state: SchemaState::Missing,
                schema: None,
            }
        }
        Err(_) => {
            return SchemaStatus {
                state: SchemaState::Invalid,
                schema: None,
            }
        }
    };
    let Ok(schema) = serde_json::from_str::<RuntimeSchema>(&raw) else {
        return SchemaStatus {
            state: SchemaState::Invalid,
            schema: None,
        };
    };

    let state = if schema.schema == CURRENT_RUNTIME_SCHEMA {
        SchemaState::Ready
    } else if schema.schema > CURRENT_RUNTIME_SCHEMA {
        SchemaState::NewerThanApp
    } else {
        SchemaState::Invalid
    };

    SchemaStatus {
        state,
        schema: Some(schema.schema),
    }
}

pub fn ensure_runtime_schema(root: &Path) -> io::Result<RuntimeSchema> {
    fs::create_dir_all(root)?;
    let path = schema_path(root);

    match read_text_recovering(&path) {
        Ok(raw) => {
            let schema: RuntimeSchema = serde_json::from_str(&raw).map_err(|error| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("invalid runtime schema {}: {error}", path.display()),
                )
            })?;

            if schema.schema > CURRENT_RUNTIME_SCHEMA {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "runtime data schema {} is newer than this app supports ({})",
                        schema.schema, CURRENT_RUNTIME_SCHEMA
                    ),
                ));
            }

            if schema.schema < CURRENT_RUNTIME_SCHEMA {
                return migrate_runtime_schema(root, schema);
            }

            return Ok(schema);
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }

    let version = env!("CARGO_PKG_VERSION").to_string();
    let schema = RuntimeSchema {
        schema: CURRENT_RUNTIME_SCHEMA,
        created_by: version.clone(),
        last_migrated_by: version,
    };
    write_schema_atomically(&path, &schema)?;
    Ok(schema)
}

fn migrate_runtime_schema(_root: &Path, schema: RuntimeSchema) -> io::Result<RuntimeSchema> {
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        format!("no migration exists from runtime schema {}", schema.schema),
    ))
}

fn write_schema_atomically(path: &Path, schema: &RuntimeSchema) -> io::Result<()> {
    let json = serde_json::to_string_pretty(schema)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    write_text_transactional(path, &format!("{json}\n"))
}

#[cfg(test)]
mod tests {
    use super::{
        ensure_runtime_schema, inspect_runtime_schema, SchemaState, CURRENT_RUNTIME_SCHEMA,
        SCHEMA_FILE,
    };
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn missing_schema_is_explicit() {
        let root = unique_temp_dir("missing-schema");
        let status = inspect_runtime_schema(&root);
        assert_eq!(status.state, SchemaState::Missing);
    }

    #[test]
    fn current_newer_older_and_malformed_schema_are_distinct() {
        let root = unique_temp_dir("schema-states");
        fs::create_dir_all(&root).unwrap();
        let path = root.join(SCHEMA_FILE);

        fs::write(
            &path,
            format!(
                "{{\"schema\":{},\"createdBy\":\"0.1.0\",\"lastMigratedBy\":\"0.1.0\"}}",
                CURRENT_RUNTIME_SCHEMA
            ),
        )
        .unwrap();
        assert_eq!(inspect_runtime_schema(&root).state, SchemaState::Ready);

        fs::write(
            &path,
            format!(
                "{{\"schema\":{},\"createdBy\":\"0.1.0\",\"lastMigratedBy\":\"0.1.0\"}}",
                CURRENT_RUNTIME_SCHEMA + 1
            ),
        )
        .unwrap();
        assert_eq!(
            inspect_runtime_schema(&root).state,
            SchemaState::NewerThanApp
        );
        assert_eq!(
            ensure_runtime_schema(&root).unwrap_err().kind(),
            std::io::ErrorKind::InvalidData
        );

        if CURRENT_RUNTIME_SCHEMA > 0 {
            fs::write(
                &path,
                format!(
                    "{{\"schema\":{},\"createdBy\":\"0.1.0\",\"lastMigratedBy\":\"0.1.0\"}}",
                    CURRENT_RUNTIME_SCHEMA - 1
                ),
            )
            .unwrap();
            assert_eq!(inspect_runtime_schema(&root).state, SchemaState::Invalid);
            assert_eq!(
                ensure_runtime_schema(&root).unwrap_err().kind(),
                std::io::ErrorKind::InvalidData
            );
        }

        fs::write(&path, "{not-json").unwrap();
        assert_eq!(inspect_runtime_schema(&root).state, SchemaState::Invalid);

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn backup_only_newer_schema_blocks_initialization() {
        let root = unique_temp_dir("schema-backup-newer");
        fs::create_dir_all(&root).unwrap();
        let path = root.join(SCHEMA_FILE);
        let backup = root.join(format!(".{SCHEMA_FILE}.bak"));
        let content = format!(
            r#"{{"schema":{},"createdBy":"future","lastMigratedBy":"future"}}"#,
            CURRENT_RUNTIME_SCHEMA + 1
        );
        fs::write(&backup, &content).unwrap();
        assert_eq!(
            ensure_runtime_schema(&root).unwrap_err().kind(),
            std::io::ErrorKind::InvalidData
        );
        assert_eq!(
            inspect_runtime_schema(&root).state,
            SchemaState::NewerThanApp
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), content);
        assert!(!backup.exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn backup_only_current_schema_is_ready_not_missing() {
        let root = unique_temp_dir("schema-backup-current");
        fs::create_dir_all(&root).unwrap();
        let backup = root.join(format!(".{SCHEMA_FILE}.bak"));
        fs::write(
            &backup,
            format!(
                r#"{{"schema":{},"createdBy":"previous","lastMigratedBy":"previous"}}"#,
                CURRENT_RUNTIME_SCHEMA
            ),
        )
        .unwrap();
        assert_eq!(inspect_runtime_schema(&root).state, SchemaState::Ready);
        assert!(!backup.exists());
        assert!(root.join(SCHEMA_FILE).is_file());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn ensure_schema_creates_current_schema_once() {
        let root = unique_temp_dir("schema-create");
        let created = ensure_runtime_schema(&root).unwrap();
        assert_eq!(created.schema, CURRENT_RUNTIME_SCHEMA);

        let loaded = ensure_runtime_schema(&root).unwrap();
        assert_eq!(loaded, created);

        fs::remove_dir_all(root).unwrap();
    }

    fn unique_temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "m-bedrock-virtual-clients-{label}-{}-{nonce}",
            std::process::id()
        ))
    }
}
