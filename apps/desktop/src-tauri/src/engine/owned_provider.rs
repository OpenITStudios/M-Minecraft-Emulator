//! Explicit developer adapter. Selection is not authority: every call checks Windows.
//! The embedded helper is developer-only and never installs a driver/service.
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, io, path::PathBuf};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub enum OwnedOperation {
    Inspect,
    Create,
    Launch,
    Focus,
    PlaceLeft,
    PlaceRight,
    Close,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OwnedProcess {
    pub pid: u32,
    pub created_file_time: String,
    pub session_id: u32,
    pub window_handle: String,
    pub responding: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OwnedRequest {
    pub session_directory: String,
    pub client_id: String,
    pub operation: OwnedOperation,
    pub licensed: bool,
    pub compatibility_confirmed: bool,
    pub target: Option<OwnedProcess>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OwnedSlot {
    pub client_id: String,
    #[serde(rename = "box")]
    pub box_name: String,
    pub profile_exists: bool,
    pub settings_verified: bool,
    pub boxed_process_count: u32,
    pub launch_allowed: bool,
    pub status: String,
    pub processes: Vec<OwnedProcess>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OwnedSnapshot {
    pub schema: u32,
    pub provider: String,
    pub operation: OwnedOperation,
    pub launcher_exit_code: Option<i32>,
    pub clients: Vec<OwnedSlot>,
    pub native_processes: Vec<OwnedProcess>,
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn validate_process(process: &OwnedProcess, require_window: bool) -> io::Result<()> {
    let timestamp = process.created_file_time.parse::<u64>().unwrap_or(0);
    let hwnd = process
        .window_handle
        .strip_prefix("0x")
        .and_then(|s| u64::from_str_radix(s, 16).ok());
    if process.pid == 0 || timestamp == 0 || hwnd.is_none() || (require_window && hwnd == Some(0)) {
        return Err(invalid("Invalid owned process/window identity"));
    }
    Ok(())
}

fn validate_request(request: &OwnedRequest) -> io::Result<()> {
    use super::instance_catalog::{admit_owned_test_operation, SlotOperation};
    let operation = match request.operation {
        OwnedOperation::Inspect => SlotOperation::Inspect,
        OwnedOperation::Create => SlotOperation::Provision,
        OwnedOperation::Launch => SlotOperation::Launch,
        OwnedOperation::Focus => SlotOperation::Focus,
        OwnedOperation::PlaceLeft | OwnedOperation::PlaceRight => SlotOperation::Place,
        OwnedOperation::Close => SlotOperation::Close,
    };
    admit_owned_test_operation(&request.client_id, operation)?;
    if (request.operation == OwnedOperation::Create && !request.compatibility_confirmed)
        || (request.operation == OwnedOperation::Launch && !request.licensed)
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Explicit compatibility/licensing confirmation is required",
        ));
    }
    if request.session_directory.is_empty()
        || request.session_directory.chars().any(char::is_control)
    {
        return Err(invalid("Invalid test session path"));
    }
    let needs_target = matches!(
        request.operation,
        OwnedOperation::Focus
            | OwnedOperation::PlaceLeft
            | OwnedOperation::PlaceRight
            | OwnedOperation::Close
    );
    if needs_target != request.target.is_some() {
        return Err(invalid(
            "Window operations require an explicit observed identity",
        ));
    }
    if let Some(target) = &request.target {
        validate_process(target, true)?;
    }
    Ok(())
}

fn parse_snapshot(text: &str, operation: OwnedOperation) -> io::Result<OwnedSnapshot> {
    let snapshot: OwnedSnapshot = serde_json::from_str(text.trim_start_matches('\u{feff}'))
        .map_err(|_| invalid("Malformed owned engine observation"))?;
    if snapshot.schema != 1
        || snapshot.provider != "OWNED_V1186_LOCAL_TEST"
        || snapshot.operation != operation
        || snapshot.clients.len() != 3
    {
        return Err(invalid("Unexpected owned provider observation"));
    }
    let mut pids = HashSet::new();
    if snapshot.native_processes.len() > 1 {
        return Err(invalid("Ambiguous native process identity"));
    }
    for process in &snapshot.native_processes {
        validate_process(process, false)?;
        pids.insert(process.pid);
    }
    for (index, slot) in snapshot.clients.iter().enumerate() {
        let id = format!("{:02}", index + 2);
        if slot.client_id != id
            || slot.box_name != format!("MivubiEdu{id}")
            || slot.processes.len() > 1
            || slot.boxed_process_count > 256
            || (slot.launch_allowed && (!slot.settings_verified || slot.boxed_process_count != 0))
            || !matches!(
                slot.status.as_str(),
                "ABSENT" | "BLOCKED" | "STOPPED" | "STARTING_OR_CLOSING" | "RUNNING"
            )
            || (slot.settings_verified && !slot.profile_exists)
            || (slot.status == "ABSENT" && (slot.profile_exists || slot.boxed_process_count != 0))
            || (slot.status == "STOPPED"
                && (!slot.settings_verified || slot.boxed_process_count != 0))
            || (slot.status == "RUNNING" && (!slot.settings_verified || slot.processes.len() != 1))
            || slot.processes.len() > slot.boxed_process_count as usize
        {
            return Err(invalid("Inconsistent owned slot observation"));
        }
        for process in &slot.processes {
            validate_process(process, slot.status == "RUNNING")?;
            if !pids.insert(process.pid) {
                return Err(invalid(
                    "Duplicate process attributed to different profiles",
                ));
            }
        }
    }
    Ok(snapshot)
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn checked_session(request: &OwnedRequest) -> io::Result<PathBuf> {
    let root = repository_root()
        .join("workspace/owned-tests")
        .canonicalize()?;
    let session = PathBuf::from(&request.session_directory).canonicalize()?;
    if session == root || !session.starts_with(&root) {
        return Err(invalid("Session is outside developer test storage"));
    }
    Ok(session)
}

pub struct OwnedTestBackend<'a> {
    pub request: &'a OwnedRequest,
}

impl OwnedTestBackend<'_> {
    pub fn execute(&self) -> io::Result<OwnedSnapshot> {
        validate_request(self.request)?;
        if !cfg!(debug_assertions) {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Local test provider is unavailable in production builds",
            ));
        }
        checked_session(self.request)?;
        self.execute_windows(false)
    }

    #[cfg(windows)]
    fn execute_windows(&self, helper_only: bool) -> io::Result<OwnedSnapshot> {
        use std::time::Duration;
        const SCRIPT: &str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../tooling/isolation-source/windows/owned-manager.ps1"
        ));
        const PROBE: &str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../tooling/isolation-source/windows/probe-v1186-runtime.ps1"
        ));
        let code = PROBE
            .split("Add-Type -TypeDefinition @'")
            .nth(1)
            .and_then(|s| s.split("'@").next())
            .ok_or_else(|| invalid("Missing canonical query code"))?;
        let quote = |s: &str| format!("'{}'", s.replace('\'', "''"));
        let json = serde_json::to_string(self.request).map_err(io::Error::other)?;
        let modules = "$PSModuleAutoLoadingPreference='None';foreach($n in @('Utility','Security','Management')){Import-Module (Join-Path $PSHOME ('Modules/Microsoft.PowerShell.'+$n+'/Microsoft.PowerShell.'+$n+'.psd1')) -ErrorAction Stop};";
        let resolver =
            include_str!("../../../../../tooling/isolation-source/windows/education-target.ps1");
        let script = format!("{modules}\n{resolver}\nAdd-Type -TypeDefinition @'\n{code}\n'@\n& {{\n{SCRIPT}\n}} -RequestJson {} -RepositoryRoot {} -ApplicationExecutable {} {}", quote(&json), quote(&repository_root().to_string_lossy()), quote(&std::env::current_exe()?.to_string_lossy()), if helper_only {"-HelperOnly"} else {""});
        const BOOTSTRAP: &str = "$ErrorActionPreference='Stop';$r=[IO.StreamReader]::new([Console]::OpenStandardInput(),[Text.Encoding]::UTF8);& ([ScriptBlock]::Create($r.ReadToEnd()))";
        let output = virtual_clients_core::bounded_command_output_with_stdin(
            &super::native_clients::system_powershell()?,
            ["-NoProfile", "-NonInteractive", "-Command", BOOTSTRAP],
            Duration::from_secs(145),
            script.as_bytes(),
        )?;
        parse_snapshot(&output, self.request.operation)
    }

    #[cfg(not(windows))]
    fn execute_windows(&self, _helper_only: bool) -> io::Result<OwnedSnapshot> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Owned engine requires Windows",
        ))
    }
}

/// One-shot helper in this same executable. It never starts a second UI/service.
pub fn maybe_run_profile_helper() -> bool {
    let args: Vec<_> = std::env::args_os().collect();
    if args.get(1).is_none_or(|s| s != "--owned-profile-create") {
        return false;
    }
    let run = || -> io::Result<()> {
        if !cfg!(debug_assertions) || args.len() != 3 {
            return Err(invalid("Invalid developer helper invocation"));
        }
        let input = PathBuf::from(&args[2]).canonicalize()?;
        if std::fs::metadata(&input)?.len() > 16384 {
            return Err(invalid("Oversized helper request"));
        }
        let request: OwnedRequest =
            serde_json::from_slice(&std::fs::read(&input)?).map_err(io::Error::other)?;
        validate_request(&request)?;
        let session = checked_session(&request)?;
        let operations = session.join("operations").canonicalize()?;
        if !operations.starts_with(&session)
            || request.operation != OwnedOperation::Create
            || !input.starts_with(&operations)
            || input.file_name().is_none_or(|n| n != "request.json")
        {
            return Err(invalid(
                "Helper only accepts a bounded profile-creation request",
            ));
        }
        let result = OwnedTestBackend { request: &request }.execute_windows(true);
        let answer = match result {
            Ok(result) => serde_json::json!({"ok":true,"error":"","result":result}),
            Err(error) => serde_json::json!({"ok":false,"error":error.to_string(),"result":null}),
        };
        let parent = input
            .parent()
            .ok_or_else(|| invalid("Missing operation directory"))?;
        let temp = parent.join("response.tmp");
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)?;
        file.write_all(&serde_json::to_vec(&answer).map_err(io::Error::other)?)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(temp, parent.join("response.json"))?;
        Ok(())
    };
    if let Err(error) = run() {
        eprintln!("Owned profile helper refused: {error}");
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request() -> OwnedRequest {
        OwnedRequest {
            session_directory: "C:\\test".into(),
            client_id: "02".into(),
            operation: OwnedOperation::Inspect,
            licensed: false,
            compatibility_confirmed: false,
            target: None,
        }
    }
    #[test]
    fn direct_ipc_rejects_foreign_slots_and_missing_window_identity() {
        for id in ["05", "2", "02 ", "MivubiEdu02"] {
            let mut r = request();
            r.client_id = id.into();
            assert!(validate_request(&r).is_err());
        }
        let mut r = request();
        r.operation = OwnedOperation::Close;
        assert!(validate_request(&r).is_err());
        for id in ["02", "03", "04"] {
            r = request();
            r.client_id = id.into();
            assert!(validate_request(&r).is_ok());
        }
    }
    #[test]
    fn unknown_fields_and_operations_are_not_authority() {
        assert!(serde_json::from_str::<OwnedRequest>(r#"{"sessionDirectory":"x","clientId":"02","operation":"Install","licensed":true,"target":null}"#).is_err());
        assert!(serde_json::from_str::<OwnedProcess>(r#"{"pid":2,"createdFileTime":"1","sessionId":1,"windowHandle":"0x1","responding":true,"owned":true}"#).is_err());
    }
    #[test]
    fn observation_rejects_missing_slots_and_false_provider() {
        for text in [
            r#"{"schema":1,"provider":"OFFICIAL","operation":"Inspect","launcherExitCode":null,"clients":[]}"#,
            r#"{"schema":1,"provider":"OWNED_V1186_LOCAL_TEST","operation":"Inspect","launcherExitCode":null,"clients":[]}"#,
        ] {
            assert!(parse_snapshot(text, OwnedOperation::Inspect).is_err());
        }
    }

    #[test]
    #[ignore = "requires an explicitly selected active Windows test session"]
    fn actual_owned_session_inspects_three_slots() {
        let mut r = request();
        r.session_directory =
            std::env::var("MIVUBI_TEST_OWNED_SESSION").expect("select a real active session");
        let result = OwnedTestBackend { request: &r }
            .execute()
            .expect("actual live provider inspection");
        assert_eq!(result.clients.len(), 3);
        assert!(result.clients[0].settings_verified);
        println!("ACTUAL_OWNED_MANAGER_INSPECTION: three slots verified; no launch/create/window operation executed");
    }
}
