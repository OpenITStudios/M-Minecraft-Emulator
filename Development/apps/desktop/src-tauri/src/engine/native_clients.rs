// Read-only compile-time capability. This means the reviewed Win64 native
// control path is compiled, NOT that a game is installed, signed in, playable,
// or that five Minecraft instances have passed acceptance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum NativeBackendSupport {
    WindowsNative,
    Unsupported,
}
pub fn native_backend_support() -> NativeBackendSupport {
    if cfg!(all(windows, target_arch = "x86_64")) {
        NativeBackendSupport::WindowsNative
    } else {
        NativeBackendSupport::Unsupported
    }
}

// Explicit private startup experiment; ordinary product admission remains unchanged.
#[cfg(all(windows, target_arch = "x86_64"))]
#[path = "independent_bootstrap.rs"]
mod independent_bootstrap;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProfileBinding {
    pub schema: u32,
    pub required: bool,
    pub configured: bool,
    pub issue: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum DataLocationAction { Inspect, Open, Locate, Choose }
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ProfilePath { pub slot:u32, pub path:String }
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct DataLocation {
    pub schema:u32, pub default_path:String, pub path:Option<String>,
    pub profiles:Vec<ProfilePath>, pub can_locate:bool,
}
// Only the reviewed Windows product may inspect/modify its five saved profile
// identities. Non-product builds do not admit managed profile storage.
#[cfg(all(windows, target_arch = "x86_64", feature = "lazy-tester-package"))]
pub fn data_location(action: DataLocationAction) -> io::Result<DataLocation> {
    independent_bootstrap::product_profiles::location(action, !crate::app_bootstrap::startup_diagnostic_mode())
}

#[cfg(all(windows, not(all(target_arch = "x86_64", feature = "lazy-tester-package"))))]
pub fn data_location(action: DataLocationAction) -> io::Result<DataLocation> {
    let _ = action;
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Data locations require the Lazy Tester product build",
    ))
}

#[cfg(not(windows))]
fn unsupported_native_profiles() -> io::Error {
    io::Error::new(
        io::ErrorKind::Unsupported,
        "Minecraft Education profile management is not implemented on this operating system",
    )
}

#[cfg(not(windows))]
pub fn data_location(_: DataLocationAction) -> io::Result<DataLocation> {
    Err(unsupported_native_profiles())
}

#[cfg(all(windows, target_arch = "x86_64", feature = "lazy-tester-package"))]
pub fn profile_binding(import: bool) -> io::Result<ProfileBinding> {
    if crate::app_bootstrap::startup_diagnostic_mode() {
        if import {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "Startup diagnostic cannot import profiles"));
        }
        return Ok(independent_bootstrap::profile_binding::inspect_existing());
    }
    independent_bootstrap::profile_binding::run(import)
}

#[cfg(all(windows, not(all(target_arch = "x86_64", feature = "lazy-tester-package"))))]
pub fn profile_binding(import: bool) -> io::Result<ProfileBinding> {
    let _ = import;
    Ok(ProfileBinding { schema: 1, required: true, configured: false, issue: Some("Managed profiles require the Lazy Tester product build.".into()) })
}

// A non-Windows GUI must never show five profiles as ready merely because no
// profile binder exists. No account/world mutation or simulated readiness.
#[cfg(not(windows))]
pub fn profile_binding(_: bool) -> io::Result<ProfileBinding> {
    Err(unsupported_native_profiles())
}

// Internal same-executable mode, never a Tauri command or Launch admission.
pub fn maybe_run_native_entrypoint() -> bool {
    #[cfg(all(windows, target_arch = "x86_64"))]
    { independent_bootstrap::maybe_run_native_entrypoint() }
    #[cfg(not(all(windows, target_arch = "x86_64")))]
    { false }
}

use serde::{Deserialize, Serialize};
use std::{collections::HashSet, io};

#[cfg(windows)]
pub(super) fn system_powershell() -> io::Result<std::path::PathBuf> {
    use std::{ffi::OsString, os::windows::ffi::OsStringExt, path::PathBuf};
    use windows_sys::Win32::System::SystemInformation::GetSystemDirectoryW;

    // Use the actual Windows system directory, not PATH or the working
    // directory. A lookalike powershell.exe must never run our collector.
    let mut system_dir = [0_u16; 32768];
    let length = unsafe { GetSystemDirectoryW(system_dir.as_mut_ptr(), system_dir.len() as u32) };
    if length == 0 || length as usize >= system_dir.len() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Windows system directory could not be resolved",
        ));
    }
    let executable = PathBuf::from(OsString::from_wide(&system_dir[..length as usize]))
        .join("WindowsPowerShell")
        .join("v1.0")
        .join("powershell.exe");
    if !executable.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Windows system PowerShell is unavailable; refusing an untrusted executable search",
        ));
    }
    Ok(executable)
}

fn invalid_evidence(reason: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, reason)
}

#[cfg(target_os = "windows")]
const EDUCATION_DESKTOP_PREFLIGHT: &str = r#"
$ErrorActionPreference = 'Stop'
$sessionId = (Get-Process -Id $PID -ErrorAction Stop).SessionId
if ($sessionId -eq 0) { throw 'Minecraft must launch from an interactive Windows session.' }
$desktopVersions = @(
  foreach ($view in @([Microsoft.Win32.RegistryView]::Registry32, [Microsoft.Win32.RegistryView]::Registry64)) {
    $registry = [Microsoft.Win32.RegistryKey]::OpenBaseKey([Microsoft.Win32.RegistryHive]::LocalMachine, $view)
    try {
      $entry = $registry.OpenSubKey('SOFTWARE\Microsoft\Microsoft Studios\Minecraft Education Edition')
      if ($null -ne $entry) {
        try {
          $version = [string]$entry.GetValue('Version', '')
          if ($version -notmatch '^\d+(?:\.\d+){1,4}$') { throw 'Desktop version registration is invalid.' }
          $version
        } finally { $entry.Dispose() }
      }
    } finally { $registry.Dispose() }
  }
)
$storePresent = @(Get-AppxPackage -Name 'Microsoft.MinecraftEducationEdition' -ErrorAction Stop).Count -gt 0
if ($desktopVersions.Count -ne 1 -or $storePresent) {
  throw 'A single Minecraft Education Desktop installation is required; do not mix Desktop and Store editions.'
}
$apps = @(Get-StartApps -ErrorAction Stop | Where-Object {
    $_.Name -eq 'Minecraft Education' -or
    $_.Name -eq 'Minecraft: Education Edition' -or
    $_.Name -eq 'Minecraft Education Edition'
})
if ($apps.Count -ne 1) {
    throw 'Minecraft Education must have exactly one registered Start menu app. Check the official installation.'
}
$appId = [string]$apps[0].AppID
# A registered AppID is NOT an authoritative on-disk EXE path.
# Win32 Start apps can use a known-folder GUID followed by a relative path.
$simpleAppId = $appId -match '^[A-Za-z0-9_.!+{}-]+$'
$desktopAppId = $appId -match '^\{[0-9A-Fa-f]{8}(?:-[0-9A-Fa-f]{4}){3}-[0-9A-Fa-f]{12}\}(?:\\[A-Za-z0-9_][A-Za-z0-9_. -]*)+$'
if (-not $appId -or $appId.Length -gt 256 -or -not ($simpleAppId -or $desktopAppId)) {
    throw 'Registered Minecraft Education application identity is invalid.'
}
"#;

// The isolated preview resolves only the official Windows registration and
// Authenticode signature; it does not inspect legacy NativePilot preferences.
#[cfg(target_os = "windows")]
const OFFICIAL_TARGET_SELECTOR: &str = if cfg!(feature = "native-workspace-preview") {
    "Resolve-ManagerEducationTarget -RegisteredOnly"
} else {
    "Resolve-ManagerEducationTarget"
};

#[cfg(target_os = "windows")]
pub fn registered_education_target() -> io::Result<String> {
    resolve_registered_education_target(OFFICIAL_TARGET_SELECTOR)
}

#[cfg(target_os = "windows")]
fn resolve_registered_education_target(selector: &str) -> io::Result<String> {
    run_registered_education_query(selector, "\n$selectedTarget")
}

#[cfg(windows)]
fn run_registered_education_query(selector: &str, output: &str) -> io::Result<String> {
    let resolver =
        include_str!("../../../../../tooling/acceptance/windows/education-target.ps1");
    let script = [
        resolver,
        EDUCATION_DESKTOP_PREFLIGHT,
        "\n$selectedTarget=",
        selector,
        "\nAssert-ManagerRegisteredAppId $appId $selectedTarget",
        output,
    ].concat();
    virtual_clients_core::bounded_command_output(
        &system_powershell()?,
        ["-NoProfile", "-NonInteractive", "-Command", &script],
        std::time::Duration::from_secs(15),
    )
    .map(|s| s.trim().to_owned())
}

// Informational receipt only: no path or hash is exposed to the webview,
// and neither the registry version nor recipe match grants launch admission.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum EducationUpdaterState { Ready, Busy, Disabled, Missing, Unknown }

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EducationInstallationReport {
    pub schema: u32,
    pub registered_version: String,
    pub updater: EducationUpdaterState,
    // None means review intentionally deferred while the updater is busy.
    pub reviewed_image: Option<bool>,
}

#[cfg(all(windows, target_arch = "x86_64"))]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RegisteredEducationMetadata {
    path: String,
    registered_version: String,
    updater: EducationUpdaterState,
}

#[cfg(all(windows, target_arch = "x86_64"))]
fn registered_education_metadata() -> io::Result<RegisteredEducationMetadata> {
    let json = run_registered_education_query("Resolve-ManagerEducationTarget -RegisteredOnly",
        r#"
[pscustomobject]@{path=$selectedTarget;registeredVersion=$desktopVersions[0];updater=(Get-ManagerEducationUpdaterState)} | ConvertTo-Json -Compress
"#)?;
    let receipt: RegisteredEducationMetadata = serde_json::from_str(&json)
        .map_err(|_| invalid_evidence("Invalid Education installation metadata"))?;
    let version = &receipt.registered_version;
    if version.len() > 64 || !(2..=5).contains(&version.split('.').count())
        || !version.split('.').all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
        || receipt.path.is_empty() {
        return Err(invalid_evidence("Invalid Education installation metadata"));
    }
    Ok(receipt)
}

#[cfg(target_os = "macos")]
mod macos_installation;

#[cfg(target_os = "macos")]
pub use macos_installation::MacEducationInstallationReport;

// Keep the existing Windows JSON receipt byte shape (no variant tag).
// Mac is explicitly a candidate observation, not Windows updater state.
#[derive(Serialize)]
#[serde(untagged)]
pub enum EducationInstallationObservation {
    #[cfg(not(target_os = "macos"))]
    Windows(EducationInstallationReport),
    #[cfg(target_os = "macos")]
    Mac(MacEducationInstallationReport),
}

pub fn education_installation_observation() -> io::Result<EducationInstallationObservation> {
    #[cfg(target_os = "macos")]
    { Ok(EducationInstallationObservation::Mac(macos_installation::inspect())) }
    #[cfg(not(target_os = "macos"))]
    { education_installation_report().map(EducationInstallationObservation::Windows) }
}

pub fn education_installation_report() -> io::Result<EducationInstallationReport> {
    #[cfg(all(windows, target_arch = "x86_64"))]
    { independent_bootstrap::education_installation_report() }
    #[cfg(not(all(windows, target_arch = "x86_64")))]
    { Err(io::Error::new(io::ErrorKind::Unsupported, "Education installation inspection requires Windows x64")) }
}

#[cfg(not(target_os = "windows"))]
pub fn registered_education_target() -> io::Result<String> {
    Err(io::Error::other("Windows required"))
}

// Product-facing native workspace observation, independent of every retired
// isolation provider. Registered-path equality is NOT proof of an unboxed
// process, publisher identity, Microsoft sign-in or gameplay.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeWorkspaceObservation {
    pub schema: u32,
    pub evidence_class: String,
    pub processes: Vec<super::process_identity::OwnedProcess>,
}

// Lazy Tester owns five launcher slots. The registered-image observer also
// sees unrelated game candidates, so >5 still fails closed rather than
// silently truncating identities. This is an observation cap, not ownership.
const MAX_NATIVE_WORKSPACE_PROCESSES: usize = 5;

#[cfg(test)]
fn parse_native_workspace_observation(raw: &str) -> io::Result<NativeWorkspaceObservation> {
    let result: NativeWorkspaceObservation = serde_json::from_str(raw)
        .map_err(|_| invalid_evidence("Malformed native workspace observation"))?;
    validate_native_workspace_observation(result)
}

// The same strict identity/duplicate validator applies to native Win32
// snapshots and imported JSON fixtures. OS-collected data is already typed:
// never serialize and deserialize it merely to reuse these invariants.
fn validate_native_workspace_observation(
    result: NativeWorkspaceObservation,
) -> io::Result<NativeWorkspaceObservation> {
    if result.schema != 1
        || result.evidence_class != "REGISTERED_IMAGE_PATH_CANDIDATE_ONLY"
        || result.processes.len() > MAX_NATIVE_WORKSPACE_PROCESSES
    {
        return Err(invalid_evidence("Too many or unsupported observed Minecraft processes"));
    }
    let mut pids = HashSet::new();
    let mut windows = HashSet::new();
    for process in &result.processes {
        let created = process.created_file_time.parse::<u64>()
            .map_err(|_| invalid_evidence("Invalid native process creation FILETIME"))?;
        let hwnd = process.window_handle.strip_prefix("0x")
            .and_then(|digits| u64::from_str_radix(digits, 16).ok());
        if process.pid == 0 || !pids.insert(process.pid) || created == 0
            || process.session_id == 0 || hwnd.is_none()
            || process.window_handle.len() > 18
            || hwnd.is_some_and(|number| number != 0 && !windows.insert(number))
        {
            return Err(invalid_evidence("Duplicate or invalid native process/window identity"));
        }
    }
    Ok(result)
}

/// Match a freshly reobserved OS process/window before presentation.
pub fn matches_native_workspace_target(
    observation: &NativeWorkspaceObservation,
    target: &super::process_identity::OwnedProcess,
) -> bool {
    target.responding && observation.processes.iter().any(|candidate| {
            candidate.same_window_identity(target)
                && candidate.window_handle != "0x0"
                && candidate.responding
        })
}

// This observer performs real Windows process/window enumeration using native
// OS handles. Only the already-existing registered Education installation
// preflight calls PowerShell; enumeration never executes a script.
#[cfg(target_os = "windows")]
mod native_workspace_windows {
    use super::*;
    use std::{mem, ptr, os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle}};
    use windows_sys::Win32::{
        Foundation::{
            ERROR_NO_MORE_FILES,
            INVALID_HANDLE_VALUE, WAIT_TIMEOUT,
        },
        Globalization::{CompareStringOrdinal, CSTR_EQUAL},
        System::{
            Diagnostics::ToolHelp::{
                CreateToolhelp32Snapshot, Process32FirstW, Process32NextW,
                PROCESSENTRY32W, TH32CS_SNAPPROCESS,
            },
            RemoteDesktop::ProcessIdToSessionId,
            Threading::{
                OpenProcess,
                WaitForSingleObject, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE,
            },
        },
        UI::WindowsAndMessaging::{
            EnumWindows, GetAncestor, GetWindow, GetWindowThreadProcessId,
            IsHungAppWindow, IsWindowVisible, GA_ROOT, GW_OWNER,
        },
    };


    struct GameWindow {
        selected: windows_sys::Win32::Foundation::HWND,
        ambiguous: bool,
    }

    impl GameWindow {
        fn record(&mut self, hwnd: windows_sys::Win32::Foundation::HWND) {
            if !self.selected.is_null() && self.selected != hwnd {
                self.ambiguous = true;
            } else {
                self.selected = hwnd;
            }
        }
    }

    struct PinnedGame {
        pid: u32,
        process: OwnedHandle,
        created: u64,
        window: GameWindow,
    }

    unsafe extern "system" fn enum_game_windows(
        hwnd: windows_sys::Win32::Foundation::HWND,
        data: isize,
    ) -> i32 {
        // EnumWindows invokes this callback synchronously. The roster is
        // bounded, fully pinned and never resized during enumeration.
        let games = &mut *(data as *mut Vec<PinnedGame>);
        let mut window_pid = 0;
        if GetWindowThreadProcessId(hwnd, &mut window_pid) == 0 {
            return 1;
        }
        let Some(game) = games.iter_mut().find(|game| game.pid == window_pid) else {
            return 1;
        };
        if IsWindowVisible(hwnd) != 0
            && GetAncestor(hwnd, GA_ROOT) == hwnd
            && GetWindow(hwnd, GW_OWNER).is_null()
        {
            game.window.record(hwnd);
        }
        1
    }

    fn exact_windows_path(left: &[u16], right: &[u16]) -> bool {
        // Ordinal case-insensitive comparison, not locale-specific lowercase
        // conversion. Windows process and registered image paths must match.
        unsafe {
            CompareStringOrdinal(
                left.as_ptr(), left.len() as i32,
                right.as_ptr(), right.len() as i32, 1,
            ) == CSTR_EQUAL
        }
    }

    fn executable_name(entry: &PROCESSENTRY32W) -> io::Result<&[u16]> {
        let end = entry.szExeFile.iter().position(|value| *value == 0)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Unterminated Windows executable name"))?;
        Ok(&entry.szExeFile[..end])
    }

    fn pin_candidate(
        pid: u32,
        target_image: &[u16],
        current_session: u32,
    ) -> io::Result<Option<PinnedGame>> {
        // Scope Windows enumeration to this interactive session *before*
        // opening any other user's same-name game process. Fail closed if
        // session attribution itself cannot be obtained.
        let mut session = 0;
        if unsafe { ProcessIdToSessionId(pid, &mut session) } == 0 {
            return Err(io::Error::last_os_error());
        }
        if session != current_session {
            return Ok(None);
        }
        let handle = unsafe {
            OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE, 0, pid,
            )
        };
        if handle.is_null() {
            // An inaccessible same-session candidate is never silently
            // excluded or promoted to an official Minecraft process.
            return Err(io::Error::last_os_error());
        }
        let process = unsafe { OwnedHandle::from_raw_handle(handle as _) };
        // Session is checked again after pinning the process HANDLE below,
        // so a PID that changed between enumeration and open is not trusted.

        let image = crate::engine::windows_process_image::query(process.as_raw_handle() as _)?;
        if !exact_windows_path(&image, target_image) {
            return Ok(None);
        }

        let original_created = crate::engine::windows_process_time::creation_filetime(process.as_raw_handle() as _)?;
        if original_created == 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Missing Minecraft process creation time"));
        }

        Ok(Some(PinnedGame {
            pid,
            process,
            created: original_created,
            window: GameWindow { selected: ptr::null_mut(), ambiguous: false },
        }))
    }

    fn finish_observation(game: &PinnedGame, current_session: u32)
        -> io::Result<super::super::process_identity::OwnedProcess> {
        let PinnedGame { pid, process, created, window } = game;
        let pid = *pid;
        if window.ambiguous {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Multiple visible Minecraft root windows attributed to one process"));
        }
        // Recheck all process/window fields after enumeration. An exited or
        // PID-reused game must not be published as a live selection.
        let current_created = crate::engine::windows_process_time::creation_filetime(process.as_raw_handle() as _)?;
        let mut again_session = 0;
        if unsafe { ProcessIdToSessionId(pid, &mut again_session) } == 0 {
            return Err(io::Error::last_os_error());
        }
        if again_session != current_session || current_created != *created
            || unsafe { WaitForSingleObject(process.as_raw_handle() as _, 0) } != WAIT_TIMEOUT
        {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Minecraft process identity changed during native observation"));
        }
        if !window.selected.is_null() {
            // The shared enumeration can precede this game's final receipt.
            // Reuse the existing exact visible/unowned root-window check.
            super::verify_live_client_02_window(window.selected, pid)?;
        }
        let responding = !window.selected.is_null()
            && unsafe { IsHungAppWindow(window.selected) } == 0;
        if unsafe { WaitForSingleObject(process.as_raw_handle() as _, 0) } != WAIT_TIMEOUT {
            return Err(invalid_evidence("Minecraft process exited during window observation"));
        }
        Ok(super::super::process_identity::OwnedProcess {
            pid,
            created_file_time: created.to_string(),
            session_id: current_session,
            window_handle: format!("0x{:x}", window.selected as usize),
            responding,
        })
    }

    // One native candidate traversal. A true visitor result terminates the
    // scan early; errors are never treated as an empty inventory.
    fn visit_game_candidates(mut visit: impl FnMut(u32) -> io::Result<bool>) -> io::Result<bool> {
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
        if snapshot == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        let snapshot = unsafe { OwnedHandle::from_raw_handle(snapshot as _) };
        let mut entry: PROCESSENTRY32W = unsafe { mem::zeroed() };
        entry.dwSize = mem::size_of::<PROCESSENTRY32W>() as u32;
        if unsafe { Process32FirstW(snapshot.as_raw_handle() as _, &mut entry) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let expected_image: Vec<u16> = "Minecraft.Windows.exe".encode_utf16().collect();
        loop {
            if exact_windows_path(executable_name(&entry)?, &expected_image) && visit(entry.th32ProcessID)? {
                return Ok(true);
            }
            if unsafe { Process32NextW(snapshot.as_raw_handle() as _, &mut entry) } == 0 {
                if io::Error::last_os_error().raw_os_error() != Some(ERROR_NO_MORE_FILES as i32) {
                    return Err(io::Error::last_os_error());
                }
                break;
            }
        }
        Ok(false)
    }

    // Conservative name-only hint: any candidate (even from another session)
    // requests the complete observation. Absence is NOT a stopped snapshot,
    // installation verification, or mutation permission.
    pub(super) fn game_candidate_exists() -> io::Result<bool> {
        visit_game_candidates(|_| Ok(true))
    }

    pub(super) fn inspect_for_registered_target(target: &str) -> io::Result<NativeWorkspaceObservation> {
        // The caller first resolved the single signed registered Education
        // image. This loop performs no PowerShell, WAM or account-file I/O.
        let target_wide: Vec<u16> = target.encode_utf16().collect();
        let mut current_session = 0;
        if unsafe { ProcessIdToSessionId(std::process::id(), &mut current_session) } == 0
            || current_session == 0
        {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "Interactive Windows session required"));
        }

        let mut games = Vec::new();
        visit_game_candidates(|pid| {
            if let Some(game) = pin_candidate(pid, &target_wide, current_session)? {
                if games.len() == MAX_NATIVE_WORKSPACE_PROCESSES {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "More than five registered Minecraft processes"));
                }
                games.push(game);
            }
            Ok(false)
        })?;
        // No candidates means no window enumeration. With one to five
        // candidates, scan the desktop once while every process stays pinned.
        if !games.is_empty()
            && unsafe { EnumWindows(Some(enum_game_windows), &mut games as *mut Vec<PinnedGame> as isize) } == 0
        {
            return Err(io::Error::last_os_error());
        }
        let mut processes = games.iter().map(|game| finish_observation(game, current_session))
            .collect::<io::Result<Vec<_>>>()?;
        processes.sort_by_key(|process| process.pid);
        let observation = NativeWorkspaceObservation {
            schema: 1,
            evidence_class: "REGISTERED_IMAGE_PATH_CANDIDATE_ONLY".into(),
            processes,
        };
        // Preserve the same validator as the JSON input path without
        // allocating and parsing an unnecessary intermediate JSON string.
        validate_native_workspace_observation(observation)
    }

    pub(super) fn inspect() -> io::Result<NativeWorkspaceObservation> {
        let target = super::registered_education_target()?;
        inspect_for_registered_target(&target)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn window_selection_preserves_missing_and_ambiguous_results() {
            // Synthetic HWND values only; never passed to a Windows API.
            let first = 1usize as windows_sys::Win32::Foundation::HWND;
            let second = 2usize as windows_sys::Win32::Foundation::HWND;
            let mut a = GameWindow { selected: ptr::null_mut(), ambiguous: false };
            let mut b = GameWindow { selected: ptr::null_mut(), ambiguous: false };
            a.record(first);
            a.record(first);
            assert_eq!(a.selected, first);
            assert!(!a.ambiguous);
            assert!(b.selected.is_null());
            assert!(!b.ambiguous);
            b.record(second);
            a.record(second);
            a.record(first);
            assert!(a.ambiguous);
            assert_eq!(b.selected, second);
            assert!(!b.ambiguous);
        }
    }
}

#[cfg(target_os = "windows")]
pub fn inspect_native_workspace() -> io::Result<NativeWorkspaceObservation> {
    native_workspace_windows::inspect()
}

#[cfg(windows)]
pub(super) fn game_candidate_exists() -> io::Result<bool> {
    native_workspace_windows::game_candidate_exists()
}

// Existing bootstrap callers retain the process lifetime separately; recheck
// HWND ownership immediately before their native window operation.
#[cfg(windows)]
fn verify_live_client_02_window(
    hwnd: windows_sys::Win32::Foundation::HWND,
    expected_pid: u32,
) -> io::Result<()> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetAncestor, GetWindow, GetWindowThreadProcessId, IsWindow, IsWindowVisible,
        GA_ROOT, GW_OWNER,
    };
    let mut actual_pid = 0;
    if expected_pid == 0 || unsafe { IsWindow(hwnd) } == 0
        || unsafe { IsWindowVisible(hwnd) } == 0
        || unsafe { GetWindowThreadProcessId(hwnd, &mut actual_pid) } == 0
        || actual_pid != expected_pid || unsafe { GetAncestor(hwnd, GA_ROOT) } != hwnd
        || !unsafe { GetWindow(hwnd, GW_OWNER) }.is_null()
    {
        return Err(io::Error::new(io::ErrorKind::NotFound,
            "Minecraft game window changed, closed or became an owned popup; Refresh first"));
    }
    Ok(())
}

// Internal read-only Settings receipt: this path was already resolved and
// signature-checked by Sessions::prepare() or its last launch. Skip a new PowerShell registered-app
// preflight for an observation only. All mutations still use the normal
// registered_education_target/inspect_native_workspace admission and the
// original OS process checks; this hint never authorizes a settings write.
#[cfg(target_os = "windows")]
fn inspect_native_workspace_for_pinned_settings(
    validated_image: &str,
) -> io::Result<NativeWorkspaceObservation> {
    if validated_image.is_empty() || validated_image.contains('\0') {
        return Err(invalid_evidence("Pinned Education image path unavailable"));
    }
    native_workspace_windows::inspect_for_registered_target(validated_image)
}

#[cfg(not(target_os = "windows"))]
pub fn inspect_native_workspace() -> io::Result<NativeWorkspaceObservation> {
    Err(io::Error::new(io::ErrorKind::Unsupported, "Minecraft workspace requires Windows"))
}

#[cfg(test)]
mod native_workspace_tests {
    use super::*;

    #[test]
    fn backend_report_is_compiled_support_only_never_an_account_or_game_check() {
        let expected = if cfg!(all(windows, target_arch = "x86_64")) {
            "WindowsNative"
        } else {
            "Unsupported"
        };
        let actual = serde_json::to_value(native_backend_support()).unwrap();
        assert_eq!(actual, expected);
    }

    #[cfg(not(windows))]
    #[test]
    fn unsupported_platform_never_claims_profile_or_data_readiness() {
        for import in [false, true] {
            assert_eq!(
                profile_binding(import).unwrap_err().kind(),
                io::ErrorKind::Unsupported,
            );
        }
        for action in [
            DataLocationAction::Inspect,
            DataLocationAction::Open,
            DataLocationAction::Locate,
            DataLocationAction::Choose,
        ] {
            assert_eq!(
                data_location(action).unwrap_err().kind(),
                io::ErrorKind::Unsupported,
            );
        }
        assert!(!maybe_run_native_entrypoint());
    }

    #[test]
    fn registered_native_candidate_requires_a_fresh_live_window() {
        let raw = r#"{"schema":1,"evidenceClass":"REGISTERED_IMAGE_PATH_CANDIDATE_ONLY","processes":[{"pid":41,"createdFileTime":"134000000000000001","sessionId":1,"windowHandle":"0x123","responding":true}]}"#;
        let observed = parse_native_workspace_observation(raw).unwrap();
        let target = observed.processes[0].clone();
        assert!(matches_native_workspace_target(&observed, &target));
        for changed in [
            super::super::process_identity::OwnedProcess { pid: 42, ..target.clone() },
            super::super::process_identity::OwnedProcess { created_file_time: "134000000000000002".into(), ..target.clone() },
            super::super::process_identity::OwnedProcess { session_id: 2, ..target.clone() },
            super::super::process_identity::OwnedProcess { window_handle: "0x124".into(), ..target.clone() },
            super::super::process_identity::OwnedProcess { responding: false, ..target.clone() },
        ] {
            assert!(!matches_native_workspace_target(&observed, &changed));
        }
        let empty = parse_native_workspace_observation(
            r#"{"schema":1,"evidenceClass":"REGISTERED_IMAGE_PATH_CANDIDATE_ONLY","processes":[]}"#,
        ).unwrap();
        assert!(!matches_native_workspace_target(&empty, &target));
    }

    #[test]
    fn typed_native_observation_reuses_the_exact_strict_json_identity_rules() {
        let raw = r#"{"schema":1,"evidenceClass":"REGISTERED_IMAGE_PATH_CANDIDATE_ONLY","processes":[{"pid":41,"createdFileTime":"134000000000000001","sessionId":1,"windowHandle":"0x0","responding":false}]}"#;
        let typed: NativeWorkspaceObservation = serde_json::from_str(raw).unwrap();
        assert_eq!(
            validate_native_workspace_observation(typed.clone()).unwrap().processes.len(),
            parse_native_workspace_observation(raw).unwrap().processes.len()
        );
        let mut bad = typed.clone();
        bad.processes.push(bad.processes[0].clone());
        assert!(validate_native_workspace_observation(bad).is_err());
        let mut invalid = typed;
        invalid.processes[0].pid = 0;
        assert!(validate_native_workspace_observation(invalid).is_err());
    }

    #[test]
    fn native_workspace_admits_five_distinct_game_processes_including_windowless() {
        let processes=(1_u32..=5).map(|slot|serde_json::json!({
            "pid":100+slot,
            "createdFileTime":(134000000000000001_u64+slot as u64).to_string(),
            "sessionId":1,
            "windowHandle":if slot==5{"0x0".to_string()}else{format!("0x{:x}",0x200+slot)},
            "responding":slot!=5,
        })).collect::<Vec<_>>();
        let report=serde_json::json!({
            "schema":1,
            "evidenceClass":"REGISTERED_IMAGE_PATH_CANDIDATE_ONLY",
            "processes":processes
        });
        let observed=parse_native_workspace_observation(&report.to_string()).unwrap();
        assert_eq!(observed.processes.len(),5);
        assert_eq!(observed.processes[4].window_handle,"0x0");
        assert!(!observed.processes[4].responding);
        for process in &observed.processes[..4] {
            assert!(matches_native_workspace_target(&observed,process));
        }
        // A sixth independently identified registered game must not be
        // ignored or misattributed to a permitted launcher.
        let mut excessive=report;
        excessive["processes"].as_array_mut().unwrap().push(serde_json::json!({
            "pid":106,
            "createdFileTime":"134000000000000106",
            "sessionId":1,
            "windowHandle":"0x206",
            "responding":true
        }));
        assert!(parse_native_workspace_observation(&excessive.to_string()).is_err());
    }

    #[test]
    fn native_workspace_refuses_ambiguous_or_forged_process_reports() {
        let valid = r#"{"pid":41,"createdFileTime":"134000000000000001","sessionId":1,"windowHandle":"0x123","responding":true}"#;
        let another = r#"{"pid":42,"createdFileTime":"134000000000000002","sessionId":1,"windowHandle":"0x124","responding":true}"#;
        let two = parse_native_workspace_observation(&format!(
            r#"{{"schema":1,"evidenceClass":"REGISTERED_IMAGE_PATH_CANDIDATE_ONLY","processes":[{valid},{another}]}}"#,
        )).unwrap();
        assert_eq!(two.processes.len(), 2);
        assert!(matches_native_workspace_target(&two, &two.processes[0]));
        assert!(matches_native_workspace_target(&two, &two.processes[1]));
        for malformed in [
            format!(r#"{{"schema":2,"evidenceClass":"REGISTERED_IMAGE_PATH_CANDIDATE_ONLY","processes":[{valid}]}}"#),
            format!(r#"{{"schema":1,"evidenceClass":"GAMEPLAY_VERIFIED","processes":[{valid}]}}"#),
            format!(r#"{{"schema":1,"evidenceClass":"REGISTERED_IMAGE_PATH_CANDIDATE_ONLY","processes":[{valid},{valid}]}}"#),
            format!(r#"{{"schema":1,"evidenceClass":"REGISTERED_IMAGE_PATH_CANDIDATE_ONLY","processes":[{valid},{{"pid":43,"createdFileTime":"134000000000000003","sessionId":1,"windowHandle":"0x123","responding":true}}]}}"#),
            format!(r#"{{"schema":1,"evidenceClass":"REGISTERED_IMAGE_PATH_CANDIDATE_ONLY","processes":[{valid},{another},{valid},{another},{valid}]}}"#),
            format!(r#"{{"schema":1,"evidenceClass":"REGISTERED_IMAGE_PATH_CANDIDATE_ONLY","processes":[{valid}],"account":"secret"}}"#),
            r#"{"schema":1,"evidenceClass":"REGISTERED_IMAGE_PATH_CANDIDATE_ONLY","processes":[{"pid":0,"createdFileTime":"1","sessionId":1,"windowHandle":"0x123","responding":true}]}"#.into(),
        ] {
            assert!(parse_native_workspace_observation(&malformed).is_err());
        }
    }
}

