use crate::{
    diagnostics::DiagnosticsReport,
    doctor::DoctorReport,
    journal::{read_operation_history, OperationRecord},
    paths::runtime_root,
    persistence::{reject_unsafe_existing_file, write_text_transactional},
    provider::staging_residue_count,
};
use serde::Serialize;
use std::{
    fs, io,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

const SUPPORT_BUNDLE_LIMIT: usize = 10;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineSnapshot {
    pub captured_at_unix_ms: u64,
    pub doctor: DoctorReport,
    pub diagnostics: DiagnosticsReport,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportBundle {
    pub schema: u32,
    pub app_version: &'static str,
    pub included_sections: [&'static str; 3],
    pub snapshot: EngineSnapshot,
    pub operation_history: Vec<OperationRecord>,
    pub staging_residue_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportBundleResult {
    pub captured_at_unix_ms: u64,
    pub path: String,
}

pub(crate) fn capture_time_ms() -> io::Result<u64> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?
        .as_millis();
    u64::try_from(millis)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "system time is out of range"))
}

pub(crate) fn write_support_bundle(snapshot: EngineSnapshot) -> io::Result<SupportBundleResult> {
    let root = runtime_root()?.join("support");
    fs::create_dir_all(&root)?;

    let captured_at_unix_ms = snapshot.captured_at_unix_ms;
    let path = root.join(format!(
        "support-{}-{}.json",
        captured_at_unix_ms,
        std::process::id()
    ));

    let bundle = SupportBundle {
        schema: 1,
        app_version: env!("CARGO_PKG_VERSION"),
        included_sections: ["snapshot", "operationHistory", "stagingResidueCount"],
        snapshot,
        operation_history: read_operation_history()?,
        staging_residue_count: staging_residue_count()?,
    };
    let json = serde_json::to_string_pretty(&bundle)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    write_text_transactional(&path, &format!("{json}\n"))?;
    let _ = prune_support_bundles(&root);

    Ok(SupportBundleResult {
        captured_at_unix_ms,
        path: path.display().to_string(),
    })
}

fn prune_support_bundles(root: &Path) -> io::Result<()> {
    let mut bundles = fs::read_dir(root)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            reject_unsafe_existing_file(path).is_ok()
                && path.is_file()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("support-") && name.ends_with(".json"))
        })
        .collect::<Vec<_>>();

    bundles.sort();
    if bundles.len() > SUPPORT_BUNDLE_LIMIT {
        let remove_count = bundles.len() - SUPPORT_BUNDLE_LIMIT;
        for path in bundles.into_iter().take(remove_count) {
            let _ = fs::remove_file(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::SUPPORT_BUNDLE_LIMIT;

    #[test]
    fn support_bundle_retention_is_bounded() {
        assert_eq!(SUPPORT_BUNDLE_LIMIT, 10);
    }
}
