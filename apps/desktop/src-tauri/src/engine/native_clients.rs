// Explicit private startup experiment; ordinary product admission remains unchanged.
#[cfg(all(windows, target_arch = "x86_64"))]
#[path = "independent_bootstrap.rs"]
mod independent_bootstrap;

// Internal same-executable mode, never a Tauri command or Launch admission.
pub fn maybe_run_education_startup_observation() -> bool {
    #[cfg(all(windows, target_arch = "x86_64"))]
    { independent_bootstrap::maybe_run_startup_observation() }
    #[cfg(not(all(windows, target_arch = "x86_64")))]
    { false }
}

use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    io,
    sync::{Mutex, MutexGuard},
    time::{Duration, Instant},
};

// One native launch request at a time. A registered game candidate already
// running must be selected, never launched again as if it were absent.
const NATIVE_LAUNCH_RETRY_GAP: Duration = Duration::from_secs(10);
static NATIVE_LAUNCH: Mutex<Option<Instant>> = Mutex::new(None);

fn acquire_native_launch() -> io::Result<MutexGuard<'static, Option<Instant>>> {
    let guard = NATIVE_LAUNCH.try_lock().map_err(|_| {
        io::Error::new(io::ErrorKind::WouldBlock, "Minecraft launch request already in progress")
    })?;
    if guard.as_ref().is_some_and(|last| last.elapsed() < NATIVE_LAUNCH_RETRY_GAP) {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Recent Minecraft launch request; refresh the native session before retrying",
        ));
    }
    Ok(guard)
}

// One backend admission boundary for explicitly requested Client 02 launches.
// This protects in-process callers without a worker, daemon or persisted state.
const CLIENT_02_LAUNCH_RETRY_GAP: Duration = Duration::from_secs(30);
static CLIENT_02_LAUNCH: Mutex<Option<Instant>> = Mutex::new(None);

// The retry gap starts from the helper's completed/failed outcome, not from
// its launch. That avoids consuming most of the safety window while the
// bounded PowerShell/Start.exe helper is still running. This only protects
// this app's explicit launches; external Sandboxie activity is not atomic.
fn client_02_launch_in_retry_gap(last: Option<Instant>, now: Instant) -> bool {
    last.is_some_and(|finished| {
        now.saturating_duration_since(finished) < CLIENT_02_LAUNCH_RETRY_GAP
    })
}

fn acquire_client_02_launch() -> io::Result<MutexGuard<'static, Option<Instant>>> {
    let guard = CLIENT_02_LAUNCH.try_lock().map_err(|_| {
        io::Error::new(
            io::ErrorKind::WouldBlock,
            "A Client 02 launch is already in progress",
        )
    })?;
    if client_02_launch_in_retry_gap(*guard, Instant::now()) {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Client 02 was recently requested; inspect before retrying after 30 seconds",
        ));
    }
    Ok(guard)
}

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

// Desktop-native observation only. PIDs and window handles are candidates,
// not verified Minecraft, Microsoft account, rendering or gameplay evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeProcessCandidate {
    pub pid: u32,
    pub window_handle: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeProcessObservation {
    pub schema: u32,
    pub evidence_class: String,
    pub interactive_session: bool,
    pub candidates: Vec<NativeProcessCandidate>,
}

fn invalid_evidence(reason: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, reason)
}

fn parse_native_observation(raw: &str) -> io::Result<NativeProcessObservation> {
    let report: NativeProcessObservation = serde_json::from_str(raw)
        .map_err(|_| invalid_evidence("native process observation has invalid JSON/schema"))?;
    if report.schema != 1
        || report.evidence_class != "NAME_MATCH_CANDIDATE_ONLY"
        || report.candidates.len() > 16
    {
        return Err(invalid_evidence("unsupported native candidate observation"));
    }
    let mut pids = HashSet::new();
    let mut handles = HashSet::new();
    for process in &report.candidates {
        if process.pid == 0 || !pids.insert(process.pid) {
            return Err(invalid_evidence(
                "native process candidates have invalid or duplicate PIDs",
            ));
        }
        if let Some(handle) = &process.window_handle {
            let bytes = handle.as_bytes();
            if !(3..=18).contains(&bytes.len())
                || !handle.starts_with("0x")
                || !bytes[2..].iter().all(u8::is_ascii_hexdigit)
            {
                return Err(invalid_evidence(
                    "native candidate window handle is malformed",
                ));
            }
            let numeric_handle = u64::from_str_radix(&handle[2..], 16)
                .map_err(|_| invalid_evidence("native candidate window handle is invalid"))?;
            if numeric_handle == 0 || !handles.insert(numeric_handle) {
                return Err(invalid_evidence(
                    "native candidate window handle is zero or duplicate",
                ));
            }
        }
    }
    Ok(report)
}

#[cfg(target_os = "windows")]
pub fn inspect_native_candidates() -> io::Result<NativeProcessObservation> {
    use std::time::Duration;

    // Current interactive Windows session only. Never read executable paths,
    // command lines, titles, world files, account IDs, registry tokens or WAM.
    // Minecraft.Windows is shared by other Minecraft editions: every PID/HWND
    // remains a name-matched candidate, never authenticated Minecraft proof.
    const SCRIPT: &str = r#"
$ErrorActionPreference = 'Stop'
$sessionId = (Get-Process -Id $PID -ErrorAction Stop).SessionId
$allProcesses = @(Get-Process -ErrorAction Stop)
$items = @(
  $allProcesses |
    Where-Object {
      $_.SessionId -eq $sessionId -and
      $_.ProcessName -match '^(Minecraft\.Windows|Minecraft.*Education.*|Education.*Minecraft.*)$'
    } |
    Sort-Object Id |
    ForEach-Object {
      $handle = $null
      if ($_.MainWindowHandle -ne [IntPtr]::Zero) {
        $handle = '0x' + $_.MainWindowHandle.ToInt64().ToString('x')
      }
      [pscustomobject]@{ pid = [int]$_.Id; windowHandle = $handle }
    }
)
if ($items.Count -gt 16) { throw 'Candidate process cap exceeded' }
[pscustomobject]@{
  schema = 1
  evidenceClass = 'NAME_MATCH_CANDIDATE_ONLY'
  interactiveSession = ($sessionId -ne 0)
  candidates = @($items)
} | ConvertTo-Json -Depth 4 -Compress
"#;

    let output = virtual_clients_core::bounded_command_output(
        &system_powershell()?,
        [
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            SCRIPT,
        ],
        Duration::from_secs(8),
    )?;
    parse_native_observation(&output)
}

#[cfg(not(target_os = "windows"))]
pub fn inspect_native_candidates() -> io::Result<NativeProcessObservation> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Native Minecraft process inspection requires Windows",
    ))
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
    let resolver =
        include_str!("../../../../../tooling/isolation-source/windows/education-target.ps1");
    let script = [
        resolver,
        EDUCATION_DESKTOP_PREFLIGHT,
        "\n$selectedTarget=",
        selector,
        "\nAssert-ManagerRegisteredAppId $appId $selectedTarget\n$selectedTarget",
    ].concat();
    virtual_clients_core::bounded_command_output(
        &system_powershell()?,
        ["-NoProfile", "-NonInteractive", "-Command", &script],
        std::time::Duration::from_secs(15),
    )
    .map(|s| s.trim().to_owned())
}

#[cfg(not(target_os = "windows"))]
pub fn registered_education_target() -> io::Result<String> {
    Err(io::Error::other("Windows required"))
}

#[cfg(target_os = "windows")]
pub fn open_native_minecraft() -> io::Result<()> {
    use std::time::Duration;

    let mut launch_guard = acquire_native_launch()?;
    if !inspect_native_workspace()?.processes.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Registered Minecraft already has a process. Select its window instead of relaunching.",
        ));
    }
    // Request the single registered official application as the current
    // interactive user. This does NOT create a guaranteed second instance.
    // Never guess EXE paths, clone game files or accept arbitrary arguments.
    const SCRIPT: &str = r#"
$explorer = Join-Path ([Environment]::GetFolderPath([Environment+SpecialFolder]::Windows)) 'explorer.exe'
if (-not (Test-Path -LiteralPath $explorer -PathType Leaf)) {
    throw 'Windows Explorer is unavailable; refusing an untrusted executable search'
}
Start-Process -FilePath $explorer -ArgumentList ('"shell:AppsFolder\' + $appId + '"') -ErrorAction Stop | Out-Null
"#;
    let resolver =
        include_str!("../../../../../tooling/isolation-source/windows/education-target.ps1");
    let script = [
        resolver,
        EDUCATION_DESKTOP_PREFLIGHT,
        "\n$selectedTarget=",
        OFFICIAL_TARGET_SELECTOR,
        "\nAssert-ManagerRegisteredAppId $appId $selectedTarget\n",
        SCRIPT,
    ]
    .concat();
    let outcome = virtual_clients_core::bounded_command_output(
        &system_powershell()?,
        [
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &script,
        ],
        Duration::from_secs(12),
    );
    if outcome.is_ok() {
        *launch_guard = Some(Instant::now());
    }
    outcome.map(|_| ()).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!("Minecraft Education launch request failed: {error}"),
        )
    })
}

#[cfg(not(target_os = "windows"))]
pub fn open_native_minecraft() -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "The four-account MVP currently targets a Windows PC.",
    ))
}


// Product-facing native Client 01 observation, independent of every retired
// isolation provider. Registered-path equality is NOT proof of an unboxed
// process, publisher identity, Microsoft sign-in or gameplay.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeWorkspaceObservation {
    pub schema: u32,
    pub evidence_class: String,
    pub processes: Vec<super::owned_provider::OwnedProcess>,
}

fn parse_native_workspace_observation(raw: &str) -> io::Result<NativeWorkspaceObservation> {
    let result: NativeWorkspaceObservation = serde_json::from_str(raw)
        .map_err(|_| invalid_evidence("Malformed native workspace observation"))?;
    if result.schema != 1
        || result.evidence_class != "REGISTERED_IMAGE_PATH_CANDIDATE_ONLY"
        || result.processes.len() > 4
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
    target: &super::owned_provider::OwnedProcess,
) -> bool {
    observation.processes.iter().any(|candidate| {
            candidate.pid == target.pid
                && candidate.created_file_time == target.created_file_time
                && candidate.session_id == target.session_id
                && candidate.window_handle == target.window_handle
                && candidate.window_handle != "0x0"
                && candidate.responding
        })
}

// Process-family evidence for the official game's transient sign-in helper.
// Parent-PID affinity is not authentication attestation and cannot authorize
// a new Minecraft session, WAM broker, filesystem route or launch.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeAuthHelperCandidate {
    pub pid: u32,
    pub created_file_time: String,
    pub same_windows_user: bool,
    pub same_profile_directory: bool,
    pub same_windows_session: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeAuthGameFamily {
    pub game_pid: u32,
    pub game_created_file_time: String,
    pub helper_candidates: Vec<NativeAuthHelperCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NativeAuthFamilyObservation {
    pub schema: u32,
    pub evidence_class: String,
    pub games: Vec<NativeAuthGameFamily>,
    pub unattributed_candidate_count: u32,
}

fn validate_native_auth_family(value: &NativeAuthFamilyObservation) -> io::Result<()> {
    if value.schema != 1
        || value.evidence_class != "REGISTERED_GAME_AUTH_HELPER_PARENT_CANDIDATES_ONLY"
        || value.games.len() > 4 || value.unattributed_candidate_count > 16
    {
        return Err(invalid_evidence("Invalid native Minecraft authentication family observation"));
    }
    let mut games = HashSet::new();
    let mut helpers = HashSet::new();
    for game in &value.games {
        if game.game_pid == 0 || !games.insert(game.game_pid)
            || game.game_created_file_time.parse::<u64>().ok().filter(|n| *n != 0).is_none()
            || game.helper_candidates.len() > 16
        {
            return Err(invalid_evidence("Invalid official game identity or authentication family size"));
        }
        for helper in &game.helper_candidates {
            if helper.pid == 0 || helper.pid == game.game_pid || !helpers.insert(helper.pid)
                || helper.created_file_time.parse::<u64>().ok().filter(|n| *n != 0).is_none()
                || !helper.same_windows_session
            {
                return Err(invalid_evidence("Invalid/duplicated authentication helper process identity"));
            }
        }
    }
    if helpers.len() + value.unattributed_candidate_count as usize > 16 {
        return Err(invalid_evidence("Too many same-session authentication helper candidates"));
    }
    Ok(())
}

// This observer performs real Windows process/window enumeration using native
// OS handles. Only the already-existing registered Education installation
// preflight calls PowerShell; enumeration never executes a script.
#[cfg(target_os = "windows")]
mod native_workspace_windows {
    use super::*;
    use std::{mem, ptr};
    use windows_sys::Win32::{
        Foundation::{
            CloseHandle, FILETIME, HANDLE, ERROR_NO_MORE_FILES,
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
                GetProcessTimes, OpenProcess, QueryFullProcessImageNameW,
                WaitForSingleObject, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE,
            },
        },
        UI::WindowsAndMessaging::{
            EnumWindows, GetAncestor, GetWindow, GetWindowThreadProcessId,
            IsHungAppWindow, IsWindowVisible, GA_ROOT, GW_OWNER,
        },
    };

    struct OwnedHandle(HANDLE);
    impl Drop for OwnedHandle {
        fn drop(&mut self) {
            // Every handle here is obtained exactly once from the OS.
            unsafe { CloseHandle(self.0); }
        }
    }

    struct GameWindow {
        pid: u32,
        selected: windows_sys::Win32::Foundation::HWND,
        ambiguous: bool,
    }

    unsafe extern "system" fn enum_game_windows(
        hwnd: windows_sys::Win32::Foundation::HWND,
        data: isize,
    ) -> i32 {
        let state = &mut *(data as *mut GameWindow);
        let mut window_pid = 0;
        GetWindowThreadProcessId(hwnd, &mut window_pid);
        if window_pid != state.pid
            || IsWindowVisible(hwnd) == 0
            || GetAncestor(hwnd, GA_ROOT) != hwnd
            || !GetWindow(hwnd, GW_OWNER).is_null()
        {
            return 1;
        }
        if !state.selected.is_null() && state.selected != hwnd {
            state.ambiguous = true;
        } else {
            state.selected = hwnd;
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

    fn observe_one(
        pid: u32,
        target_image: &[u16],
        current_session: u32,
    ) -> io::Result<Option<super::super::owned_provider::OwnedProcess>> {
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
        let process = OwnedHandle(handle);
        // Session is checked again after pinning the process HANDLE below,
        // so a PID that changed between enumeration and open is not trusted.

        let mut image = vec![0u16; 32768];
        let mut size = image.len() as u32;
        if unsafe { QueryFullProcessImageNameW(process.0, 0, image.as_mut_ptr(), &mut size) } == 0 {
            return Err(io::Error::last_os_error());
        }
        if size == 0 || !exact_windows_path(&image[..size as usize], target_image) {
            return Ok(None);
        }

        let mut created: FILETIME = unsafe { mem::zeroed() };
        let mut exited: FILETIME = unsafe { mem::zeroed() };
        let mut kernel: FILETIME = unsafe { mem::zeroed() };
        let mut user: FILETIME = unsafe { mem::zeroed() };
        if unsafe { GetProcessTimes(process.0, &mut created, &mut exited, &mut kernel, &mut user) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let original_created = (u64::from(created.dwHighDateTime) << 32)
            | u64::from(created.dwLowDateTime);
        if original_created == 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Missing Minecraft process creation time"));
        }

        let mut window = GameWindow {
            pid,
            selected: ptr::null_mut(),
            ambiguous: false,
        };
        if unsafe { EnumWindows(Some(enum_game_windows), &mut window as *mut GameWindow as isize) } == 0 {
            return Err(io::Error::last_os_error());
        }
        if window.ambiguous {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Multiple visible Minecraft root windows attributed to one process"));
        }
        // Recheck all process/window fields after enumeration. An exited or
        // PID-reused game must not be published as a live selection.
        let mut again: FILETIME = unsafe { mem::zeroed() };
        if unsafe { GetProcessTimes(process.0, &mut again, &mut exited, &mut kernel, &mut user) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let current_created = (u64::from(again.dwHighDateTime) << 32)
            | u64::from(again.dwLowDateTime);
        let mut again_session = 0;
        if unsafe { ProcessIdToSessionId(pid, &mut again_session) } == 0 {
            return Err(io::Error::last_os_error());
        }
        if again_session != current_session || current_created != original_created
            || unsafe { WaitForSingleObject(process.0, 0) } != WAIT_TIMEOUT
        {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Minecraft process identity changed during native observation"));
        }
        if !window.selected.is_null() {
            let mut window_pid = 0;
            unsafe { GetWindowThreadProcessId(window.selected, &mut window_pid); }
            if window_pid != pid {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Minecraft HWND owner changed during observation"));
            }
        }
        let responding = !window.selected.is_null()
            && unsafe { IsHungAppWindow(window.selected) } == 0;
        Ok(Some(super::super::owned_provider::OwnedProcess {
            pid,
            created_file_time: original_created.to_string(),
            session_id: current_session,
            window_handle: format!("0x{:x}", window.selected as usize),
            responding,
        }))
    }

    fn inspect_for_registered_target(target: &str) -> io::Result<NativeWorkspaceObservation> {
        // The caller first resolved the single signed registered Education
        // image. This loop performs no PowerShell, WAM or account-file I/O.
        let target_wide: Vec<u16> = target.encode_utf16().collect();
        let mut current_session = 0;
        if unsafe { ProcessIdToSessionId(std::process::id(), &mut current_session) } == 0
            || current_session == 0
        {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "Interactive Windows session required"));
        }

        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
        if snapshot == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        let snapshot = OwnedHandle(snapshot);
        let mut entry: PROCESSENTRY32W = unsafe { mem::zeroed() };
        entry.dwSize = mem::size_of::<PROCESSENTRY32W>() as u32;
        if unsafe { Process32FirstW(snapshot.0, &mut entry) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let expected_image: Vec<u16> = "Minecraft.Windows.exe".encode_utf16().collect();
        let mut processes = Vec::new();
        loop {
            if exact_windows_path(executable_name(&entry)?, &expected_image) {
                if let Some(process) = observe_one(entry.th32ProcessID, &target_wide, current_session)? {
                    processes.push(process);
                    if processes.len() > 4 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "More than four registered Minecraft processes"));
                    }
                }
            }
            if unsafe { Process32NextW(snapshot.0, &mut entry) } == 0 {
                if io::Error::last_os_error().raw_os_error() != Some(ERROR_NO_MORE_FILES as i32) {
                    return Err(io::Error::last_os_error());
                }
                break;
            }
        }
        processes.sort_by_key(|process| process.pid);
        let observation = NativeWorkspaceObservation {
            schema: 1,
            evidence_class: "REGISTERED_IMAGE_PATH_CANDIDATE_ONLY".into(),
            processes,
        };
        // Keep one canonical invariant validator for the native OS collector
        // and previously authored malformed/duplicate process regressions.
        let raw = serde_json::to_string(&observation).map_err(io::Error::other)?;
        parse_native_workspace_observation(&raw)
    }

    pub(super) fn inspect() -> io::Result<NativeWorkspaceObservation> {
        let target = super::registered_education_target()?;
        inspect_for_registered_target(&target)
    }

    /// One on-demand snapshot of actual Windows processes associated with
    /// the game's documented Minecraft.AdalServer.exe login helper. Names
    /// and Toolhelp parent PIDs are merely candidates; the helper could be
    /// transient, launched by another process, or replaced while inspecting.
    pub(super) fn inspect_auth_family() -> io::Result<NativeAuthFamilyObservation> {
        let target = super::registered_education_target()?;
        let before = inspect_for_registered_target(&target)?;
        let helper_path = std::path::Path::new(&target)
            .with_file_name("Minecraft.AdalServer.exe");
        let helper_image: Vec<u16> = helper_path.to_string_lossy().encode_utf16().collect();
        let expected_name: Vec<u16> = "Minecraft.AdalServer.exe".encode_utf16().collect();
        let mut session = 0;
        if unsafe { ProcessIdToSessionId(std::process::id(), &mut session) } == 0
            || session == 0
        {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied,
                "Interactive Windows session required to inspect sign-in helper processes"));
        }

        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
        if snapshot == INVALID_HANDLE_VALUE { return Err(io::Error::last_os_error()); }
        let snapshot = OwnedHandle(snapshot);
        let mut entry: PROCESSENTRY32W = unsafe { mem::zeroed() };
        entry.dwSize = mem::size_of::<PROCESSENTRY32W>() as u32;
        if unsafe { Process32FirstW(snapshot.0, &mut entry) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let mut games: Vec<NativeAuthGameFamily> = before.processes.iter()
            .map(|game| NativeAuthGameFamily {
                game_pid: game.pid,
                game_created_file_time: game.created_file_time.clone(),
                helper_candidates: Vec::new(),
            }).collect();
        let mut unattributed = 0u32;
        let mut seen_helpers = HashSet::new();
        loop {
            if exact_windows_path(executable_name(&entry)?, &expected_name) {
                let pid = entry.th32ProcessID;
                let mut candidate_session = 0;
                if unsafe { ProcessIdToSessionId(pid, &mut candidate_session) } == 0 {
                    return Err(io::Error::last_os_error());
                }
                // Never open handles to Minecraft sign-in helpers belonging
                // to a different Windows login session.
                if candidate_session == session {
                    if !seen_helpers.insert(pid) || seen_helpers.len() > 16 {
                        return Err(invalid_evidence("Ambiguous or excessive sign-in helpers"));
                    }
                    let parent_index = games.iter().position(|game| game.game_pid == entry.th32ParentProcessID);
                    if let Some(index) = parent_index {
                        // A verified directory match is still not an image
                        // SECTION signature or a game-authentication verdict.
                        if let Some(helper) = observe_one(pid, &helper_image, session)? {
                            let parent = &games[index];
                            let game_created = parent.game_created_file_time.parse::<u64>()
                                .map_err(|_| invalid_evidence("Invalid registered game creation time"))?;
                            let helper_created = helper.created_file_time.parse::<u64>()
                                .map_err(|_| invalid_evidence("Invalid sign-in helper creation time"))?;
                            if helper_created <= game_created {
                                unattributed += 1;
                            } else {
                                let context = super::super::instance_identity::compare_contexts(
                                    &super::super::instance_identity::ProcessKey {
                                        pid: parent.game_pid,
                                        created_file_time: parent.game_created_file_time.clone(),
                                    },
                                    &super::super::instance_identity::ProcessKey {
                                        pid: helper.pid,
                                        created_file_time: helper.created_file_time.clone(),
                                    },
                                )?;
                                games[index].helper_candidates.push(NativeAuthHelperCandidate {
                                    pid: helper.pid,
                                    created_file_time: helper.created_file_time,
                                    same_windows_user: context.same_windows_user,
                                    same_profile_directory: context.same_profile_directory,
                                    same_windows_session: context.same_windows_session,
                                });
                            }
                        } else {
                            // Same process NAME but wrong on-disk image path.
                            // Do not misattribute another installed program.
                            unattributed += 1;
                        }
                    } else {
                        // The game's helper can also be broker-launched and
                        // unrelated same-session processes can share a name.
                        unattributed += 1;
                    }
                }
            }
            if unsafe { Process32NextW(snapshot.0, &mut entry) } == 0 {
                if io::Error::last_os_error().raw_os_error() != Some(ERROR_NO_MORE_FILES as i32) {
                    return Err(io::Error::last_os_error());
                }
                break;
            }
        }

        // Parent PID relationships alone cannot authenticate live processes.
        // Recheck each official game PID/FILETIME/session/HWND after helper
        // identity comparisons to refuse a stale or replaced game process.
        let after = inspect_for_registered_target(&target)?;
        if before.processes.len() != after.processes.len()
            || before.processes.iter().any(|game| !after.processes.iter().any(|fresh|
                fresh.pid == game.pid
                && fresh.created_file_time == game.created_file_time
                && fresh.session_id == game.session_id
                && fresh.window_handle == game.window_handle))
        {
            return Err(io::Error::new(io::ErrorKind::NotFound,
                "Registered Minecraft process changed during authentication helper inspection"));
        }
        let report = NativeAuthFamilyObservation {
            schema: 1,
            evidence_class: "REGISTERED_GAME_AUTH_HELPER_PARENT_CANDIDATES_ONLY".into(),
            games,
            unattributed_candidate_count: unattributed,
        };
        validate_native_auth_family(&report)?;
        Ok(report)
    }
}

#[cfg(target_os = "windows")]
pub fn inspect_native_auth_family() -> io::Result<NativeAuthFamilyObservation> {
    native_workspace_windows::inspect_auth_family()
}

#[cfg(not(target_os = "windows"))]
pub fn inspect_native_auth_family() -> io::Result<NativeAuthFamilyObservation> {
    Err(io::Error::new(io::ErrorKind::Unsupported,
        "Minecraft sign-in helper inspection requires Windows"))
}

#[cfg(target_os = "windows")]
pub fn inspect_native_workspace() -> io::Result<NativeWorkspaceObservation> {
    native_workspace_windows::inspect()
}

#[cfg(not(target_os = "windows"))]
pub fn inspect_native_workspace() -> io::Result<NativeWorkspaceObservation> {
    Err(io::Error::new(io::ErrorKind::Unsupported, "Minecraft workspace requires Windows"))
}

#[cfg(test)]
mod native_workspace_tests {
    use super::*;

    #[test]
    fn registered_native_candidate_requires_a_fresh_live_window() {
        let raw = r#"{"schema":1,"evidenceClass":"REGISTERED_IMAGE_PATH_CANDIDATE_ONLY","processes":[{"pid":41,"createdFileTime":"134000000000000001","sessionId":1,"windowHandle":"0x123","responding":true}]}"#;
        let observed = parse_native_workspace_observation(raw).unwrap();
        let target = observed.processes[0].clone();
        assert!(matches_native_workspace_target(&observed, &target));
        for changed in [
            super::super::owned_provider::OwnedProcess { pid: 42, ..target.clone() },
            super::super::owned_provider::OwnedProcess { created_file_time: "134000000000000002".into(), ..target.clone() },
            super::super::owned_provider::OwnedProcess { session_id: 2, ..target.clone() },
            super::super::owned_provider::OwnedProcess { window_handle: "0x124".into(), ..target.clone() },
            super::super::owned_provider::OwnedProcess { responding: false, ..target.clone() },
        ] {
            assert!(!matches_native_workspace_target(&observed, &changed));
        }
        let empty = parse_native_workspace_observation(
            r#"{"schema":1,"evidenceClass":"REGISTERED_IMAGE_PATH_CANDIDATE_ONLY","processes":[]}"#,
        ).unwrap();
        assert!(!matches_native_workspace_target(&empty, &target));
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

// The canonical Windows evidence collector is embedded into this development
// pilot. One implementation owns Sandboxie box/membership/config observations;
// its data never proves Microsoft account isolation or a playable Minecraft.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProcessIdentityCandidate {
    pub pid: u32,
    // Decimal FILETIME, carried as a string to preserve full precision in JS.
    // None means start time was inaccessible; it is not a verified identity.
    pub created_file_time: Option<String>,
    pub window_handle: Option<String>,
    pub in_selected_box: Option<bool>,
    // Microsoft signed Education product image candidate, not native/authenticated.
    pub image_evidence: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SandboxieClientObservation {
    pub schema: u32,
    pub box_name: String,
    pub stage: String,
    pub box_configured: String,
    pub box_settings_source: String,
    pub auto_delete: String,
    pub auto_remove: String,
    pub credential_host_access: String,
    pub membership_status: String,
    pub interactive_session: bool,
    pub process_evidence_class: String,
    pub candidate_process_count: u32,
    pub process_candidates: Vec<ProcessIdentityCandidate>,
    pub official_image_outside_selected_box_count: u32,
    pub direct_pid_query: String,
    pub signed_image_other_box_count: u32,
    pub signed_image_unknown_box_count: u32,
    pub isolated_candidate_processes: u32,
    pub native_visible_candidates: u32,
    pub isolated_visible_candidates: u32,
    pub interpretation: String,
}

fn parse_sandboxie_observation(raw: &str) -> io::Result<SandboxieClientObservation> {
    let report: SandboxieClientObservation = serde_json::from_str(raw)
        .map_err(|_| invalid_evidence("invalid Sandboxie collector summary"))?;
    let one_of = |value: &str, allowed: &[&str]| allowed.contains(&value);
    if report.schema != 6
        || report.box_name != CLIENT_02_BOX_NAME
        || report.stage != "INSPECT"
        || report.process_evidence_class != "NAME_MATCH_CANDIDATE_ONLY"
        || report.interpretation != "CONFIG_AND_PROCESS_CANDIDATES_ONLY_NO_AUTHENTICATION_PROOF"
        || !one_of(&report.box_configured, &["YES", "NO", "UNKNOWN"])
        || !one_of(
            &report.box_settings_source,
            &["BOX_SECTION_ENUMERATED", "UNKNOWN"],
        )
        || !one_of(&report.auto_delete, &["ON", "OFF", "UNKNOWN"])
        || !one_of(&report.auto_remove, &["ON", "OFF", "UNKNOWN"])
        || !one_of(&report.credential_host_access, &["ON", "OFF", "UNKNOWN"])
        || !one_of(&report.membership_status, &["OBSERVED", "UNKNOWN"])
        || (report.membership_status == "OBSERVED" && report.box_configured != "YES")
        || (report.box_settings_source == "BOX_SECTION_ENUMERATED"
            && report.box_configured != "YES")
        || (report.credential_host_access != "UNKNOWN"
            && report.box_settings_source != "BOX_SECTION_ENUMERATED")
        || report.candidate_process_count > 16
        || report.process_candidates.len() != report.candidate_process_count as usize
        || report.official_image_outside_selected_box_count > report.candidate_process_count
        || !one_of(
            &report.direct_pid_query,
            &["NOT_REQUESTED", "UNAVAILABLE", "QUERIED"],
        )
        || report.signed_image_other_box_count > report.official_image_outside_selected_box_count
        || report.signed_image_unknown_box_count > report.official_image_outside_selected_box_count
        || report
            .signed_image_other_box_count
            .saturating_add(report.signed_image_unknown_box_count)
            != report.official_image_outside_selected_box_count
        || (report.direct_pid_query == "NOT_REQUESTED"
            && report.official_image_outside_selected_box_count != 0)
        || (report.direct_pid_query != "NOT_REQUESTED"
            && report.official_image_outside_selected_box_count == 0)
        || (report.direct_pid_query != "QUERIED" && report.signed_image_other_box_count != 0)
        || report.isolated_candidate_processes > report.candidate_process_count
        || report.isolated_visible_candidates > report.isolated_candidate_processes
        || report
            .native_visible_candidates
            .saturating_add(report.isolated_visible_candidates)
            > report.candidate_process_count
        || (report.membership_status == "UNKNOWN"
            && (report.native_visible_candidates != 0
                || report.isolated_visible_candidates != 0
                || report.isolated_candidate_processes != 0))
    {
        return Err(invalid_evidence(
            "untrusted or contradictory Sandboxie process observation",
        ));
    }
    // Reconcile the identity list with the published candidate counts.
    // No PID/HWND snapshot is proof of Education sign-in or active gameplay.
    let mut pids = HashSet::new();
    let mut windows = HashSet::new();
    let mut boxed = 0_u32;
    let mut boxed_visible = 0_u32;
    let mut outside_visible = 0_u32;
    let mut official_outside = 0_u32;
    for candidate in &report.process_candidates {
        if candidate.pid == 0 || !pids.insert(candidate.pid) {
            return Err(invalid_evidence("duplicate or zero process identity PID"));
        }
        if let Some(time) = &candidate.created_file_time {
            let n = time
                .parse::<u64>()
                .map_err(|_| invalid_evidence("invalid process FILETIME"))?;
            if n == 0 || n.to_string() != *time {
                return Err(invalid_evidence("noncanonical process FILETIME"));
            }
        }
        if report.membership_status == "UNKNOWN" && candidate.in_selected_box.is_some() {
            return Err(invalid_evidence(
                "unknown Sandboxie membership cannot attribute a PID",
            ));
        }
        if candidate.in_selected_box == Some(true) {
            boxed += 1;
        }
        match candidate.image_evidence.as_str() {
            "UNKNOWN" => (),
            "SIGNED_EDUCATION_IMAGE_CANDIDATE" => {
                if candidate.window_handle.is_none()
                    || !matches!(candidate.in_selected_box, Some(true) | Some(false))
                    || (candidate.in_selected_box == Some(true)
                        && candidate.created_file_time.is_none())
                {
                    return Err(invalid_evidence(
                        "signed Education image requires a visible attributed process and boxed creation time"));
                }
                // This original counter remains OUTSIDE-only.
                if candidate.in_selected_box == Some(false) {
                    official_outside += 1;
                }
            }
            _ => return Err(invalid_evidence("unrecognized executable image evidence")),
        }
        if let Some(hwnd) = &candidate.window_handle {
            let raw = hwnd.as_bytes();
            if !(3..=18).contains(&raw.len())
                || !hwnd.starts_with("0x")
                || !raw[2..].iter().all(u8::is_ascii_hexdigit)
            {
                return Err(invalid_evidence("invalid process identity HWND"));
            }
            let handle = u64::from_str_radix(&hwnd[2..], 16)
                .map_err(|_| invalid_evidence("invalid process identity HWND"))?;
            if handle == 0 || !windows.insert(handle) {
                return Err(invalid_evidence("duplicate or zero process HWND"));
            }
            if candidate.in_selected_box == Some(true) {
                boxed_visible += 1;
            }
            if candidate.in_selected_box == Some(false) {
                outside_visible += 1;
            }
        }
    }
    if boxed != report.isolated_candidate_processes
        || official_outside != report.official_image_outside_selected_box_count
        || boxed_visible != report.isolated_visible_candidates
        || outside_visible != report.native_visible_candidates
    {
        return Err(invalid_evidence(
            "process candidate identities contradict counters",
        ));
    }
    Ok(report)
}

// Explicit launch admission uses the one canonical, freshly observed box
// snapshot. UNKNOWN membership never means an empty sandbox.
fn admit_client_02_launch_snapshot(observation: &SandboxieClientObservation) -> io::Result<()> {
    if !observation.interactive_session
        || observation.box_configured != "YES"
        || observation.box_settings_source != "BOX_SECTION_ENUMERATED"
        || observation.auto_delete != "OFF"
        || observation.auto_remove != "OFF"
        || observation.credential_host_access != "OFF"
    {
        return Err(io::Error::new(io::ErrorKind::PermissionDenied,
            "Client 02 preflight blocked: verify interactive Windows, persistent Sandboxie box and credential-bypass settings"));
    }
    if observation.membership_status != "OBSERVED" {
        return Err(io::Error::new(io::ErrorKind::WouldBlock,
            "Client 02 launch blocked: Sandboxie PID membership or Windows process identity is UNKNOWN; check Sandboxie service, process-inspection permissions and other Windows sessions, then Refresh"));
    }
    if observation.isolated_candidate_processes > 0 {
        return Err(io::Error::new(io::ErrorKind::AlreadyExists,
            "Client 02 already contains a Minecraft process candidate, possibly without a window; inspect before retrying"));
    }
    Ok(())
}

// The provider boundary owns inspection and explicit launch requests.
// Official Sandboxie stays the sole active backend; no data migration,
// automatic launch retry or runtime provider selector is introduced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IsolationInspection {
    StatusWithImageProof,
    FastLaunchPreflight,
    TargetedImageProof(u32),
}

trait IsolationBackend {
    fn inspect(&self, mode: IsolationInspection) -> io::Result<SandboxieClientObservation>;
    // This acknowledges a launcher request only, not Minecraft gameplay.
    fn request_launch(&self, executable: String, licensed: bool) -> io::Result<()>;
}

struct OfficialSandboxieBackend;
const ACTIVE_ISOLATION_BACKEND: OfficialSandboxieBackend = OfficialSandboxieBackend;

impl IsolationBackend for OfficialSandboxieBackend {
    fn inspect(&self, mode: IsolationInspection) -> io::Result<SandboxieClientObservation> {
        if matches!(mode, IsolationInspection::TargetedImageProof(0)) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Targeted inspection requires a nonzero process PID",
            ));
        }
        #[cfg(windows)]
        {
            match mode {
                IsolationInspection::StatusWithImageProof => {
                    inspect_sandbox_client_02_mode(true, None)
                }
                IsolationInspection::FastLaunchPreflight => {
                    inspect_sandbox_client_02_mode(false, None)
                }
                IsolationInspection::TargetedImageProof(pid) => {
                    inspect_sandbox_client_02_mode(false, Some(pid))
                }
            }
        }
        #[cfg(not(windows))]
        {
            let _ = mode;
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Windows is required",
            ))
        }
    }

    fn request_launch(&self, executable: String, licensed: bool) -> io::Result<()> {
        launch_with_official_sandboxie(executable, licensed)
    }
}

// An adapter error is propagated verbatim, never interpreted as zero PIDs.
fn inspect_with_backend(
    backend: &impl IsolationBackend,
    mode: IsolationInspection,
) -> io::Result<SandboxieClientObservation> {
    backend.inspect(mode)
}

// One explicit request per invocation: never retry an ambiguous helper result.
fn request_with_backend(
    backend: &impl IsolationBackend,
    executable: String,
    licensed: bool,
) -> io::Result<()> {
    backend.request_launch(executable, licensed)
}

// The same collector owns both paths. Explicit UI Refresh checks candidate
// executable provenance; Focus/Place/Launch inspect only the process/box
// evidence they need, without repeated Authenticode verification.
#[cfg(windows)]
fn inspect_sandbox_client_02_mode(
    image_proof: bool,
    boxed_image_pid: Option<u32>,
) -> io::Result<SandboxieClientObservation> {
    use std::time::Duration;
    const INSPECT: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../tooling/acceptance/windows/collect-sandboxie-feasibility.ps1"
    ));
    let script = if image_proof {
        ["& {\n", INSPECT, "\n} -SummaryOnly -ImageProof"].concat()
    } else if let Some(pid) = boxed_image_pid.filter(|pid| *pid != 0) {
        // A decimal u32 is the only interpolated value, not executable path.
        [
            "& {\n",
            INSPECT,
            "\n} -SummaryOnly -BoxedImageProofPid ",
            &pid.to_string(),
        ]
        .concat()
    } else {
        ["& {\n", INSPECT, "\n} -SummaryOnly"].concat()
    };
    // Passing the whole embedded collector as -Command exceeds Windows'
    // 32,767-character CreateProcess command-line limit. Stream it as
    // UTF-8 through stdin and parse via an ASCII-only fixed bootstrap.
    // The source stays embedded and read-only; no .ps1 file is written.
    const BOOTSTRAP: &str = "$r=[System.IO.StreamReader]::new([Console]::OpenStandardInput(),[System.Text.Encoding]::UTF8);& ([ScriptBlock]::Create($r.ReadToEnd()))";
    let output = virtual_clients_core::bounded_command_output_with_stdin(
        &system_powershell()?,
        [
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            BOOTSTRAP,
        ],
        Duration::from_secs(30),
        script.as_bytes(),
    )?;
    parse_sandboxie_observation(&output)
}

pub fn inspect_sandbox_client_02() -> io::Result<SandboxieClientObservation> {
    inspect_with_backend(
        &ACTIVE_ISOLATION_BACKEND,
        IsolationInspection::StatusWithImageProof,
    )
}

#[cfg(windows)]
fn inspect_sandbox_client_02_fast() -> io::Result<SandboxieClientObservation> {
    inspect_with_backend(
        &ACTIVE_ISOLATION_BACKEND,
        IsolationInspection::FastLaunchPreflight,
    )
}

// Focus is a user-requested, non-destructive window operation. Never use a
// previously observed PID/HWND as authority without fresh Sandboxie membership,
// process creation time and window-owner checks.
fn verify_fresh_focus_target(
    target: &ProcessIdentityCandidate,
    fresh: &SandboxieClientObservation,
) -> io::Result<(u64, u64)> {
    let invalid =
        || {
            io::Error::new(io::ErrorKind::InvalidInput,
        "Client 02 focus requires a visible boxed process with known identity; Refresh first")
        };
    // UI eligibility requires exactly one boxed Minecraft candidate.
    // Repeat that requirement against the NEW snapshot: another game process
    // may have appeared since Check status, even if the old PID/HWND survives.
    if target.pid == 0
        || target.in_selected_box != Some(true)
        || !fresh.interactive_session
        || fresh.membership_status != "OBSERVED"
        || fresh.isolated_candidate_processes != 1
        || fresh.isolated_visible_candidates != 1
    {
        return Err(invalid());
    }
    let key = super::instance_identity::parse_window_identity(
        target.pid,
        target.created_file_time.as_deref(),
        target.window_handle.as_deref(),
    )
    .ok_or_else(invalid)?;
    if !fresh.process_candidates.iter().any(|candidate| {
        candidate.pid == target.pid
            && candidate.created_file_time == target.created_file_time
            && candidate.window_handle == target.window_handle
            && candidate.in_selected_box == Some(true)
            && candidate.image_evidence == "SIGNED_EDUCATION_IMAGE_CANDIDATE"
    }) {
        // The caller's imageEvidence is untrusted. The backend must obtain
        // a new signed-file check for this exact PID/time/window.
        return Err(io::Error::new(io::ErrorKind::NotFound,
            "Client 02 identity, window or Microsoft-signed Education image could not be verified; Refresh first"));
    }
    Ok((key.created_file_time, key.hwnd))
}

// Compute a single client slot inside a monitor's usable work area. This
// deliberately does not infer, move or claim ownership of Client 01.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Client02PlacementSide {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Client02PlacementRect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

fn client_02_half_slot(
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
    side: Client02PlacementSide,
) -> io::Result<Client02PlacementRect> {
    let width = right
        .checked_sub(left)
        .ok_or_else(|| invalid_evidence("invalid monitor work-area width"))?;
    let height = bottom
        .checked_sub(top)
        .ok_or_else(|| invalid_evidence("invalid monitor work-area height"))?;
    if width < 2 || height < 1 {
        return Err(invalid_evidence(
            "monitor work area is too small for window placement",
        ));
    }
    let left_width = width / 2;
    let (x, slot_width) = match side {
        Client02PlacementSide::Left => (left, left_width),
        Client02PlacementSide::Right => (
            left.checked_add(left_width)
                .ok_or_else(|| invalid_evidence("monitor placement overflow"))?,
            width - left_width,
        ),
    };
    Ok(Client02PlacementRect {
        x,
        y: top,
        width: slot_width,
        height,
    })
}

// GA_ROOT checks parents, not an owned top-level popup. Require an unowned,
// visible root belonging to this exact PID; do not read window titles or
// claim that a matching name establishes official Minecraft/gameplay.
// HWND may be reused, so recheck immediately before each Win32 mutation.
#[cfg(windows)]
fn verify_live_client_02_window(
    hwnd: windows_sys::Win32::Foundation::HWND,
    expected_pid: u32,
) -> io::Result<()> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetAncestor, GetWindow, GetWindowThreadProcessId, IsWindow, IsWindowVisible, GA_ROOT,
        GW_OWNER,
    };
    let mut actual_pid = 0_u32;
    if unsafe { IsWindow(hwnd) } == 0
        || unsafe { IsWindowVisible(hwnd) } == 0
        || unsafe { GetWindowThreadProcessId(hwnd, &mut actual_pid) } == 0
        || actual_pid != expected_pid
        || unsafe { GetAncestor(hwnd, GA_ROOT) } != hwnd
        || !unsafe { GetWindow(hwnd, GW_OWNER) }.is_null()
    {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Client 02 game window changed, closed or became an owned popup; Refresh first",
        ));
    }
    Ok(())
}

// One ownership boundary for both Focus and Place. Keep the process handle
// alive across the entire Win32 action to prevent PID reuse. Recheck the
// observed box membership and HWND owner immediately before calling the OS.
#[cfg(windows)]
pub(super) fn with_verified_client_02_window(
    target: ProcessIdentityCandidate,
    action: impl FnOnce(windows_sys::Win32::Foundation::HWND) -> io::Result<()>,
) -> io::Result<()> {
    use windows_sys::Win32::{
        Foundation::{CloseHandle, FILETIME},
        System::Threading::{GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION},
    };
    // An explicit operation never bypasses an in-progress launch or another
    // window mutation in the same manager.
    let _guard = CLIENT_02_LAUNCH.try_lock().map_err(|_| {
        io::Error::new(
            io::ErrorKind::WouldBlock,
            "Another Client 02 operation is in progress",
        )
    })?;
    // A targeted, read-only signed-file probe is run only for this boxed
    // PID. The manual outside-box ImageProof remains a different mode.
    let fresh = inspect_with_backend(
        &ACTIVE_ISOLATION_BACKEND,
        IsolationInspection::TargetedImageProof(target.pid),
    )?;
    let (expected_time, hwnd_number) = verify_fresh_focus_target(&target, &fresh)?;
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, target.pid) };
    if process.is_null() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Cannot inspect Client 02 process identity; action refused",
        ));
    }

    let outcome = (|| -> io::Result<()> {
        let empty = || FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        };
        let mut created = empty();
        let mut exited = empty();
        let mut kernel = empty();
        let mut user = empty();
        if unsafe { GetProcessTimes(process, &mut created, &mut exited, &mut kernel, &mut user) }
            == 0
        {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Cannot recheck Client 02 creation time",
            ));
        }
        let actual_time =
            (u64::from(created.dwHighDateTime) << 32) | u64::from(created.dwLowDateTime);
        if actual_time != expected_time {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Client 02 PID was reused or process identity changed; Refresh first",
            ));
        }

        // Avoid silent truncation of an HWND recorded as a 64-bit integer.
        let native_handle = usize::try_from(hwnd_number)
            .map_err(|_| invalid_evidence("Client 02 HWND exceeds native pointer width"))?;
        let hwnd = native_handle as *mut core::ffi::c_void;
        verify_live_client_02_window(hwnd, target.pid)?;
        action(hwnd)
    })();
    unsafe {
        CloseHandle(process);
    }
    outcome
}

#[cfg(windows)]
pub fn focus_observed_client_02(target: ProcessIdentityCandidate) -> io::Result<()> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        IsIconic, SetForegroundWindow, ShowWindow, SW_RESTORE,
    };
    let process_id = target.pid;
    with_verified_client_02_window(target, |hwnd| {
        if unsafe { IsIconic(hwnd) } != 0 {
            unsafe {
                ShowWindow(hwnd, SW_RESTORE);
            }
            if unsafe { IsIconic(hwnd) } != 0 {
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "Windows did not restore Client 02; restore the game window manually",
                ));
            }
        }
        // Restore may replace the window; never focus a stale HWND.
        verify_live_client_02_window(hwnd, process_id)?;
        if unsafe { SetForegroundWindow(hwnd) } == 0 {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Windows refused to focus Client 02; select its window or use Alt+Tab",
            ));
        }
        Ok(())
    })
}

#[cfg(not(windows))]
pub fn focus_observed_client_02(_target: ProcessIdentityCandidate) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Focusing Client 02 requires Windows",
    ))
}

#[cfg(windows)]
pub fn place_observed_client_02(
    target: ProcessIdentityCandidate,
    side: Client02PlacementSide,
) -> io::Result<()> {
    use windows_sys::Win32::{
        Graphics::Gdi::{
            GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
        },
        UI::WindowsAndMessaging::{
            IsIconic, IsZoomed, SetWindowPos, SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE,
            SWP_NOOWNERZORDER, SWP_NOZORDER,
        },
    };
    let process_id = target.pid;
    with_verified_client_02_window(target, |hwnd| {
        // Never silently change Minecraft's maximized/fullscreen or minimized
        // state. The operator can restore the window explicitly first.
        if unsafe { IsIconic(hwnd) } != 0 || unsafe { IsZoomed(hwnd) } != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Restore Client 02 to a normal window before arranging",
            ));
        }
        let monitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
        if monitor.is_null() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Cannot locate the monitor containing Client 02",
            ));
        }
        let mut info = MONITORINFO::default();
        info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        if unsafe { GetMonitorInfoW(monitor, &mut info) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let work = info.rcWork;
        let slot = client_02_half_slot(work.left, work.top, work.right, work.bottom, side)?;
        // Monitor/window state can change while querying the work area.
        // Revalidate the entire window and layout target before the move.
        verify_live_client_02_window(hwnd, process_id)?;
        if unsafe { IsIconic(hwnd) } != 0 || unsafe { IsZoomed(hwnd) } != 0 {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "Client 02 window state changed before placement; restore and Refresh",
            ));
        }
        if unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) } != monitor {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "Client 02 moved to another monitor; Refresh before placing",
            ));
        }
        // Async placement avoids waiting indefinitely on a game window's
        // input queue. Success means the request was accepted, not that the
        // game rendered or the final geometry is already confirmed.
        let flags = SWP_ASYNCWINDOWPOS | SWP_NOACTIVATE | SWP_NOZORDER | SWP_NOOWNERZORDER;
        if unsafe {
            SetWindowPos(
                hwnd,
                std::ptr::null_mut(),
                slot.x,
                slot.y,
                slot.width,
                slot.height,
                flags,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    })
}

#[cfg(not(windows))]
pub fn place_observed_client_02(
    _target: ProcessIdentityCandidate,
    _side: Client02PlacementSide,
) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Arranging Client 02 requires Windows",
    ))
}

// Client 02 only. The sandbox is provisioned and licensed outside the manager.
const CLIENT_02_BOX_NAME: &str = "MinecraftClient02";
// This is a bounded launch REQUEST, not an independent-account verdict.
fn validate_education_path(path: &str) -> io::Result<()> {
    let p = std::path::Path::new(path);
    let valid = p.file_name().and_then(|s| s.to_str()).is_some_and(|name| {
        matches!(
            name.to_ascii_lowercase().as_str(),
            "minecraft.windows.exe" | "minecrafteducation.exe" | "minecrafteducationedition.exe"
        )
    });
    if path.is_empty()
        || path.len() > 1024
        || path.chars().any(char::is_control)
        || !p.is_absolute()
        || !valid
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Select the installed Minecraft Education Windows EXE, not a shortcut or installer",
        ));
    }
    Ok(())
}

#[cfg(windows)]
pub fn select_education_exe() -> io::Result<Option<String>> {
    // A generic *.exe picker can return Notepad or an unrelated installer.
    // Check the SAME filename/path policy as Launch at selection time, so
    // the UI cannot present an arbitrary EXE as Minecraft Education.
    // Authentic Microsoft publisher/product proof is repeated at Launch.
    rfd::FileDialog::new()
        .set_title("Select Minecraft Education executable")
        .add_filter("Windows executable", &["exe"])
        .pick_file()
        .map(|path| {
            let executable = path.to_str().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Selected executable path cannot be represented safely",
                )
            })?;
            validate_education_path(executable)?;
            if !path.is_file() {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Selected Minecraft Education executable is unavailable",
                ));
            }
            Ok(executable.to_owned())
        })
        .transpose()
}

#[cfg(not(windows))]
pub fn select_education_exe() -> io::Result<Option<String>> {
    Err(io::Error::new(io::ErrorKind::Unsupported, "Windows only"))
}

pub fn request_sandbox_client_02(executable: String, licensed: bool) -> io::Result<()> {
    request_with_backend(&ACTIVE_ISOLATION_BACKEND, executable, licensed)
}

// Inspection only: a valid staged candidate is still neither loadable nor
// enabled. No state or driver operations are performed by this command.
pub fn inspect_owned_engine_candidate(
    directory: String,
) -> io::Result<crate::engine::owned_isolation::OwnedCandidateInspection> {
    crate::engine::owned_isolation::inspect_candidate_path(directory)
}

// Read-only SCM namespace probe: does not select or install an owned backend.
pub fn inspect_owned_engine_host() -> io::Result<crate::engine::owned_isolation::OwnedHostInspection>
{
    crate::engine::owned_isolation::inspect_host_service_namespace()
}

#[cfg(windows)]
pub fn select_owned_candidate_directory() -> io::Result<Option<String>> {
    Ok(rfd::FileDialog::new()
        .set_title("Select staged owned isolation engine (read-only)")
        .pick_folder()
        .map(|p| p.to_string_lossy().into_owned()))
}

#[cfg(not(windows))]
pub fn select_owned_candidate_directory() -> io::Result<Option<String>> {
    Err(io::Error::new(io::ErrorKind::Unsupported, "Windows only"))
}

// Original official launch implementation: policy, admission, lock, and
// Microsoft executable signature checks remain unchanged.
#[cfg(windows)]
fn launch_with_official_sandboxie(executable: String, licensed: bool) -> io::Result<()> {
    use std::{path::Path, time::Duration};
    validate_education_path(&executable)?;
    if !licensed {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Confirm licensing for Sandboxie Plus in this environment",
        ));
    }
    if !Path::new(&executable).is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Selected Minecraft Education EXE was not found",
        ));
    }
    // UI buttons are not an IPC concurrency guarantee. Hold the backend
    // guard across preflight and launch to prevent overlapping requests.
    let mut launch_guard = acquire_client_02_launch()?;
    let observation = inspect_sandbox_client_02_fast()?;
    // A fresh backend inspection is authoritative regardless of UI state.
    admit_client_02_launch_snapshot(&observation)?;
    let escaped = executable.replace('\'', "''");
    let box_name = CLIENT_02_BOX_NAME;
    let desktop_preflight = EDUCATION_DESKTOP_PREFLIGHT;
    let script = format!(
        r#"
{desktop_preflight}
$game = '{escaped}'
$box = '{box_name}'
# Sandboxie's GUI normally receives a working directory for sandboxed
# run commands; pin this to the signed installed game EXE directory.
$gameFolder = [System.IO.Path]::GetDirectoryName($game)
if (-not (Test-Path -LiteralPath $gameFolder -PathType Container)) {{
    throw 'Selected Education executable directory is unavailable'
}}
$product = [Diagnostics.FileVersionInfo]::GetVersionInfo($game).ProductName
$signature = Get-AuthenticodeSignature -LiteralPath $game
$publisher = if ($null -ne $signature.SignerCertificate) {{
    [string]$signature.SignerCertificate.Subject
}} else {{ '' }}
if ($product -notmatch '(?i)(Minecraft.*Education|Education.*Minecraft)' -or
    $signature.Status -ne 'Valid' -or
    $publisher -notmatch '(?i)(^|,\s*)O=Microsoft Corporation(,|$)') {{
    throw 'Selected executable must be a valid Microsoft Corporation-signed Minecraft Education Desktop build'
}}
# Search the same trusted Windows installation roots as the canonical
# read-only collector, avoiding preflight/launch path inconsistencies.
$roots = @(
    [Environment]::GetEnvironmentVariable('ProgramW6432'),
    [Environment]::GetEnvironmentVariable('ProgramFiles'),
    [Environment]::GetEnvironmentVariable('ProgramFiles(x86)')
) | Where-Object {{ $_ }} | Select-Object -Unique
$install = @($roots | ForEach-Object {{ Join-Path $_ 'Sandboxie-Plus' }} |
    Where-Object {{
        (Test-Path -LiteralPath (Join-Path $_ 'Start.exe') -PathType Leaf) -and
        (Test-Path -LiteralPath (Join-Path $_ 'SbieIni.exe') -PathType Leaf)
    }}) | Select-Object -First 1
if (-not $install) {{ throw "Install Sandboxie Plus and create $box first" }}
$start = Join-Path $install 'Start.exe'
$ini = Join-Path $install 'SbieIni.exe'
if ((Get-AuthenticodeSignature -LiteralPath $start).Status -ne 'Valid' -or
    (Get-AuthenticodeSignature -LiteralPath $ini).Status -ne 'Valid') {{
    throw 'Sandboxie tools must be signed and trusted'
}}
if ((Get-Service SbieSvc -ErrorAction Stop).Status -ne 'Running') {{
    throw 'Sandboxie service not running'
}}
$enabled = @(& $ini query $box Enabled 2>$null)
if ($LASTEXITCODE -ne 0 -or $enabled.Count -ne 1 -or $enabled[0].Trim() -ne 'y') {{
    throw "Persistent box $box must exist and be enabled"
}}
$names = @(& $ini query $box '*' 2>$null)
if ($LASTEXITCODE -ne 0 -or $names.Count -lt 1 -or
    $names.Count -gt 128 -or $names -notcontains 'Enabled') {{
    throw 'Could not verify persistent box settings'
}}
foreach ($key in @('AutoDelete','AutoRemove')) {{
    if ($names -contains $key) {{
        $value = @(& $ini query $box $key 2>$null)
        if ($LASTEXITCODE -ne 0 -or $value.Count -ne 1 -or $value[0].Trim() -ne 'n') {{
            throw 'AutoDelete and AutoRemove must remain OFF'
        }}
    }}
}}
# Sandboxie's GUI Start.exe may return before a child actually launches.
# Never interpret this request as process membership, login or gameplay.
$launcher = Start-Process -FilePath $start -WorkingDirectory $gameFolder -ArgumentList @(('/box:' + $box), ('"' + $game + '"')) -PassThru -ErrorAction Stop
try {{
    if (-not $launcher.WaitForExit(8000)) {{
        # Start.exe is not Minecraft. Leave game and box untouched when
        # the helper's outcome is ambiguous, but release our process handle.
        throw 'Sandboxie Start.exe did not confirm the launch request within 8 seconds; game state remains UNKNOWN'
    }}
    if ($launcher.ExitCode -ne 0) {{
        throw "Sandboxie Start.exe rejected the launch request (exit code $($launcher.ExitCode))"
    }}
}} finally {{
    $launcher.Dispose()
}}
"#
    );
    // Even a failed/timed-out Start.exe request may already have spawned the
    // game. Keep the in-process lock throughout the attempt and reserve the
    // entire retry gap AFTER helper completion, including ambiguous failures.
    let launch_result = virtual_clients_core::bounded_command_output(
        &system_powershell()?,
        [
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            script.as_str(),
        ],
        Duration::from_secs(20),
    );
    *launch_guard = Some(Instant::now());
    launch_result?;
    Ok(())
}

#[cfg(not(windows))]
fn launch_with_official_sandboxie(_executable: String, _licensed: bool) -> io::Result<()> {
    Err(io::Error::new(io::ErrorKind::Unsupported, "Windows only"))
}

#[cfg(test)]
mod tests {
    use super::{
        acquire_client_02_launch, parse_native_observation, parse_sandboxie_observation,
        validate_education_path,
    };
    use serde_json::json;

    fn fixture() -> serde_json::Value {
        json!({
            "schema": 1,
            "evidenceClass": "NAME_MATCH_CANDIDATE_ONLY",
            "interactiveSession": true,
            "candidates": [
                {"pid": 123, "windowHandle": "0xAA"},
                {"pid": 456, "windowHandle": null}
            ]
        })
    }

    #[cfg(windows)]
    #[test]
    fn system_powershell_is_absolute_and_from_windows_system_directory() {
        let tool = super::system_powershell()
            .expect("system PowerShell must be available for the Windows pilot");
        assert!(tool.is_absolute());
        assert!(tool.is_file());
        assert!(tool.ends_with(std::path::Path::new(
            "WindowsPowerShell/v1.0/powershell.exe"
        )));
    }

    struct FakeInspection {
        value: Option<super::SandboxieClientObservation>,
        failure: Option<std::io::ErrorKind>,
        probes: std::cell::Cell<u32>,
    }
    impl super::IsolationBackend for FakeInspection {
        fn inspect(
            &self,
            mode: super::IsolationInspection,
        ) -> std::io::Result<super::SandboxieClientObservation> {
            assert_eq!(mode, super::IsolationInspection::FastLaunchPreflight);
            self.probes.set(self.probes.get() + 1);
            if let Some(error) = self.failure {
                return Err(std::io::Error::new(error, "unknown inspection outcome"));
            }
            self.value.clone().ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "no complete sandbox observation",
                )
            })
        }

        fn request_launch(&self, _executable: String, _licensed: bool) -> std::io::Result<()> {
            Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "inspection fake never launches",
            ))
        }
    }

    #[test]
    fn adapter_propagates_timeout_and_incomplete_evidence_without_fallback() {
        let fake = FakeInspection {
            value: None,
            failure: Some(std::io::ErrorKind::TimedOut),
            probes: std::cell::Cell::new(0),
        };
        assert_eq!(
            super::inspect_with_backend(&fake, super::IsolationInspection::FastLaunchPreflight)
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::TimedOut
        );
        assert_eq!(fake.probes.get(), 1);

        let invalid = FakeInspection {
            value: None,
            failure: None,
            probes: std::cell::Cell::new(0),
        };
        assert_eq!(
            super::inspect_with_backend(&invalid, super::IsolationInspection::FastLaunchPreflight)
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::InvalidData
        );
        assert_eq!(invalid.probes.get(), 1);
    }

    #[test]
    fn official_adapter_rejects_zero_pid_before_os_image_proof() {
        use super::IsolationBackend;
        assert_eq!(
            super::ACTIVE_ISOLATION_BACKEND
                .inspect(super::IsolationInspection::TargetedImageProof(0))
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::InvalidInput
        );
    }

    struct FakeLaunch {
        attempts: std::cell::Cell<u32>,
        received: std::cell::RefCell<Option<(String, bool)>>,
    }

    impl super::IsolationBackend for FakeLaunch {
        fn inspect(
            &self,
            _mode: super::IsolationInspection,
        ) -> std::io::Result<super::SandboxieClientObservation> {
            Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "launch fake cannot inspect",
            ))
        }

        fn request_launch(&self, executable: String, licensed: bool) -> std::io::Result<()> {
            self.attempts.set(self.attempts.get() + 1);
            *self.received.borrow_mut() = Some((executable, licensed));
            Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "ambiguous launch outcome",
            ))
        }
    }

    #[test]
    fn adapter_launch_dispatches_once_without_retry_on_timeout() {
        let fake = FakeLaunch {
            attempts: std::cell::Cell::new(0),
            received: std::cell::RefCell::new(None),
        };
        let path = "C:/Program Files/Minecraft Education/Minecraft.Windows.exe".to_string();
        let error = super::request_with_backend(&fake, path.clone(), true).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::TimedOut);
        assert_eq!(fake.attempts.get(), 1);
        assert_eq!(*fake.received.borrow(), Some((path, true)));
    }

    #[test]
    fn placement_slot_keeps_all_pieces_inside_monitor_work_area() {
        use super::{client_02_half_slot, Client02PlacementSide};
        let left = client_02_half_slot(-1280, 40, 641, 1120, Client02PlacementSide::Left).unwrap();
        let right =
            client_02_half_slot(-1280, 40, 641, 1120, Client02PlacementSide::Right).unwrap();
        assert_eq!(left.x, -1280);
        assert_eq!(left.y, 40);
        assert_eq!(left.width, 960);
        assert_eq!(right.x, -320);
        assert_eq!(right.width, 961);
        assert_eq!(left.height, 1080);
        assert_eq!(right.x + right.width, 641);
        assert!(client_02_half_slot(30, 0, 31, 800, Client02PlacementSide::Left).is_err());
        assert!(client_02_half_slot(10, 40, 10, 400, Client02PlacementSide::Right).is_err());
        assert!(
            client_02_half_slot(i32::MIN, 0, i32::MAX, 800, Client02PlacementSide::Right).is_err()
        );
    }

    #[test]
    fn focus_requires_exact_fresh_box_process_and_window_identity() {
        let report = serde_json::json!({
            "schema":6, "boxName":"MinecraftClient02", "stage":"INSPECT",
            "boxConfigured":"YES", "boxSettingsSource":"BOX_SECTION_ENUMERATED",
            "autoDelete":"OFF", "autoRemove":"OFF", "credentialHostAccess":"OFF",
            "membershipStatus":"OBSERVED", "interactiveSession":true,
            "processEvidenceClass":"NAME_MATCH_CANDIDATE_ONLY",
            "candidateProcessCount":1,
            "processCandidates":[{
                "pid":202, "createdFileTime":"133400000000000001",
                "windowHandle":"0x200", "inSelectedBox":true, "imageEvidence":"UNKNOWN"
            }],
            "officialImageOutsideSelectedBoxCount":0,
            "directPidQuery":"NOT_REQUESTED",
            "signedImageOtherBoxCount":0, "signedImageUnknownBoxCount":0,
            "isolatedCandidateProcesses":1, "nativeVisibleCandidates":0,
            "isolatedVisibleCandidates":1,
            "interpretation":"CONFIG_AND_PROCESS_CANDIDATES_ONLY_NO_AUTHENTICATION_PROOF"
        });
        let mut fresh = parse_sandboxie_observation(&report.to_string()).unwrap();
        let target = fresh.process_candidates[0].clone();
        // The UI sends a name-matched UNKNOWN candidate. It cannot grant
        // itself image authority; an unsigned fresh snapshot is rejected.
        assert!(super::verify_fresh_focus_target(&target, &fresh).is_err());
        fresh.process_candidates[0].image_evidence = "SIGNED_EDUCATION_IMAGE_CANDIDATE".into();
        // The fresh backend proof is for the exact same PID/time/HWND;
        // caller-supplied imageEvidence remains ignored.
        assert_eq!(
            super::verify_fresh_focus_target(&target, &fresh).unwrap().0,
            133400000000000001
        );

        // The selected PID/time/HWND still matches, but a SECOND boxed
        // candidate appeared after the UI snapshot. Focus and Place must
        // recheck unique eligibility instead of trusting the old window.
        let mut extra_boxed = fresh.clone();
        let mut newcomer = target.clone();
        newcomer.pid = 203;
        newcomer.created_file_time = Some("133400000000000003".into());
        newcomer.window_handle = None;
        newcomer.image_evidence = "UNKNOWN".into();
        extra_boxed.process_candidates.push(newcomer.clone());
        extra_boxed.candidate_process_count = 2;
        extra_boxed.isolated_candidate_processes = 2;
        assert!(parse_sandboxie_observation(&serde_json::to_string(&extra_boxed).unwrap()).is_ok());
        assert!(super::verify_fresh_focus_target(&target, &extra_boxed).is_err());

        // Both windows might be visible by the time the action reaches Rust.
        let mut extra_window = extra_boxed.clone();
        extra_window.process_candidates[1].window_handle = Some("0x300".into());
        extra_window.isolated_visible_candidates = 2;
        assert!(
            parse_sandboxie_observation(&serde_json::to_string(&extra_window).unwrap()).is_ok()
        );
        assert!(super::verify_fresh_focus_target(&target, &extra_window).is_err());

        // A distinct outside-box native candidate is NOT a second boxed
        // client, so it does not invalidate the verified boxed target.
        let mut native_outside = fresh.clone();
        newcomer.in_selected_box = Some(false);
        newcomer.window_handle = Some("0x300".into());
        native_outside.process_candidates.push(newcomer);
        native_outside.candidate_process_count = 2;
        native_outside.native_visible_candidates = 1;
        assert!(
            parse_sandboxie_observation(&serde_json::to_string(&native_outside).unwrap()).is_ok()
        );
        assert!(super::verify_fresh_focus_target(&target, &native_outside).is_ok());

        let mut changed = target.clone();
        changed.created_file_time = Some("133400000000000002".into());
        assert!(super::verify_fresh_focus_target(&changed, &fresh).is_err());
        changed = target.clone();
        changed.window_handle = Some("0x201".into());
        assert!(super::verify_fresh_focus_target(&changed, &fresh).is_err());
        changed = target.clone();
        changed.in_selected_box = None;
        assert!(super::verify_fresh_focus_target(&changed, &fresh).is_err());
        changed = target.clone();
        changed.created_file_time = None;
        assert!(super::verify_fresh_focus_target(&changed, &fresh).is_err());
        let mut unknown = fresh.clone();
        unknown.membership_status = "UNKNOWN".into();
        assert!(super::verify_fresh_focus_target(&target, &unknown).is_err());
    }

    #[test]
    fn client_02_retry_gap_uses_helper_completion_timestamp() {
        let now = std::time::Instant::now();
        let gap = super::CLIENT_02_LAUNCH_RETRY_GAP;
        assert!(!super::client_02_launch_in_retry_gap(None, now));
        assert!(super::client_02_launch_in_retry_gap(Some(now), now));
        assert!(super::client_02_launch_in_retry_gap(
            Some(now - (gap - std::time::Duration::from_secs(1))),
            now
        ));
        assert!(!super::client_02_launch_in_retry_gap(Some(now - gap), now));
    }

    #[test]
    fn backend_rejects_overlapping_client_02_launches() {
        let first = acquire_client_02_launch().expect("first request admitted");
        let second = acquire_client_02_launch()
            .err()
            .expect("concurrent request refused");
        assert_eq!(second.kind(), std::io::ErrorKind::WouldBlock);
        drop(first);
        assert!(acquire_client_02_launch().is_ok());
    }

    #[test]
    fn client_02_launch_requires_observed_empty_box() {
        let empty = serde_json::json!({
            "schema":6, "boxName":"MinecraftClient02", "stage":"INSPECT",
            "boxConfigured":"YES", "boxSettingsSource":"BOX_SECTION_ENUMERATED",
            "autoDelete":"OFF", "autoRemove":"OFF", "credentialHostAccess":"OFF",
            "membershipStatus":"OBSERVED", "interactiveSession":true,
            "processEvidenceClass":"NAME_MATCH_CANDIDATE_ONLY",
            "candidateProcessCount":0, "processCandidates":[],
            "officialImageOutsideSelectedBoxCount":0,
            "directPidQuery":"NOT_REQUESTED", "signedImageOtherBoxCount":0,
            "signedImageUnknownBoxCount":0, "isolatedCandidateProcesses":0,
            "nativeVisibleCandidates":0, "isolatedVisibleCandidates":0,
            "interpretation":"CONFIG_AND_PROCESS_CANDIDATES_ONLY_NO_AUTHENTICATION_PROOF"
        });
        let observed = parse_sandboxie_observation(&empty.to_string()).unwrap();
        assert!(super::admit_client_02_launch_snapshot(&observed).is_ok());
        let mut unknown = empty.clone();
        unknown["membershipStatus"] = serde_json::json!("UNKNOWN");
        let unknown = parse_sandboxie_observation(&unknown.to_string()).unwrap();
        assert_eq!(
            super::admit_client_02_launch_snapshot(&unknown)
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::WouldBlock
        );
        let mut occupied = observed.clone();
        occupied.isolated_candidate_processes = 1;
        assert_eq!(
            super::admit_client_02_launch_snapshot(&occupied)
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::AlreadyExists
        );
        let mut unsafe_box = observed.clone();
        unsafe_box.credential_host_access = "UNKNOWN".into();
        assert_eq!(
            super::admit_client_02_launch_snapshot(&unsafe_box)
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::PermissionDenied
        );
    }

    #[test]
    fn sandboxie_summary_rejects_fabricated_membership_and_credentials() {
        let base = serde_json::json!({
            "schema":6, "boxName":"MinecraftClient02", "stage":"INSPECT",
            "boxConfigured":"YES", "boxSettingsSource":"BOX_SECTION_ENUMERATED",
            "autoDelete":"OFF", "autoRemove":"OFF", "credentialHostAccess":"OFF",
            "membershipStatus":"OBSERVED", "interactiveSession":true,
            "processEvidenceClass":"NAME_MATCH_CANDIDATE_ONLY",
            "candidateProcessCount":2,
            "processCandidates":[
                {"pid":101, "createdFileTime":"133400000000000000", "windowHandle":"0x100", "inSelectedBox":false, "imageEvidence":"SIGNED_EDUCATION_IMAGE_CANDIDATE"},
                {"pid":202, "createdFileTime":"133400000000000001", "windowHandle":"0x200", "inSelectedBox":true, "imageEvidence":"UNKNOWN"}
            ],
            "officialImageOutsideSelectedBoxCount":1,
            "directPidQuery":"QUERIED",
            "signedImageOtherBoxCount":0, "signedImageUnknownBoxCount":1,
            "isolatedCandidateProcesses":1,
            "nativeVisibleCandidates":1, "isolatedVisibleCandidates":1,
            "interpretation":"CONFIG_AND_PROCESS_CANDIDATES_ONLY_NO_AUTHENTICATION_PROOF"
        });
        assert!(parse_sandboxie_observation(&base.to_string()).is_ok());
        let mut wrong = base.clone();
        wrong["boxName"] = serde_json::json!("AnotherBox");
        assert!(parse_sandboxie_observation(&wrong.to_string()).is_err());
        let mut wrong = base.clone();
        wrong["membershipStatus"] = serde_json::json!("UNKNOWN");
        assert!(parse_sandboxie_observation(&wrong.to_string()).is_err());
        let mut wrong = base.clone();
        wrong["credentialHostAccess"] = serde_json::json!("VERIFIED_GAMEPLAY");
        assert!(parse_sandboxie_observation(&wrong.to_string()).is_err());
        let mut wrong = base.clone();
        wrong["isolatedCandidateProcesses"] = serde_json::json!(0);
        assert!(parse_sandboxie_observation(&wrong.to_string()).is_err());
        let mut wrong = base.clone();
        wrong["isolatedCandidateProcesses"] = serde_json::json!(3);
        assert!(parse_sandboxie_observation(&wrong.to_string()).is_err());
        let mut wrong = base.clone();
        wrong["processCandidates"][1]["createdFileTime"] = serde_json::json!("invalid");
        assert!(parse_sandboxie_observation(&wrong.to_string()).is_err());
        let mut wrong = base.clone();
        wrong["processCandidates"][1]["pid"] = serde_json::json!(101);
        assert!(parse_sandboxie_observation(&wrong.to_string()).is_err());
        let mut wrong = base.clone();
        wrong["processCandidates"][1]["inSelectedBox"] = serde_json::json!(false);
        assert!(parse_sandboxie_observation(&wrong.to_string()).is_err());
        let mut wrong = base.clone();
        wrong["officialImageOutsideSelectedBoxCount"] = serde_json::json!(0);
        assert!(parse_sandboxie_observation(&wrong.to_string()).is_err());
        let mut wrong = base.clone();
        wrong["signedImageOtherBoxCount"] = serde_json::json!(2);
        assert!(parse_sandboxie_observation(&wrong.to_string()).is_err());
        let mut wrong = base.clone();
        wrong["directPidQuery"] = serde_json::json!("NATIVE_VERIFIED");
        assert!(parse_sandboxie_observation(&wrong.to_string()).is_err());
        let mut unknown = base.clone();
        unknown["directPidQuery"] = serde_json::json!("UNAVAILABLE");
        assert!(parse_sandboxie_observation(&unknown.to_string()).is_ok());
        let mut other = base.clone();
        other["signedImageOtherBoxCount"] = serde_json::json!(1);
        other["signedImageUnknownBoxCount"] = serde_json::json!(0);
        assert!(parse_sandboxie_observation(&other.to_string()).is_ok());
        other["directPidQuery"] = serde_json::json!("UNAVAILABLE");
        assert!(parse_sandboxie_observation(&other.to_string()).is_err());
        let mut boxed_signed = base.clone();
        boxed_signed["processCandidates"][1]["imageEvidence"] =
            serde_json::json!("SIGNED_EDUCATION_IMAGE_CANDIDATE");
        // Signed boxed images do not alter the outside-only counter.
        assert!(parse_sandboxie_observation(&boxed_signed.to_string()).is_ok());
        boxed_signed["processCandidates"][1]["createdFileTime"] = serde_json::Value::Null;
        assert!(parse_sandboxie_observation(&boxed_signed.to_string()).is_err());
        let mut wrong = base.clone();
        wrong["processCandidates"][0]["imageEvidence"] = serde_json::json!("AUTHENTICATED_NATIVE");
        assert!(parse_sandboxie_observation(&wrong.to_string()).is_err());
        let mut missing_time = base.clone();
        missing_time["processCandidates"][1]["createdFileTime"] = serde_json::Value::Null;
        assert!(parse_sandboxie_observation(&missing_time.to_string()).is_ok());
    }

    #[test]
    fn accepts_only_non_authoritative_two_process_observation() {
        let parsed = parse_native_observation(&fixture().to_string()).unwrap();
        assert_eq!(parsed.candidates.len(), 2);
        assert_eq!(parsed.candidates[0].pid, 123);
        assert_eq!(parsed.candidates[1].window_handle, None);
        assert_eq!(parsed.evidence_class, "NAME_MATCH_CANDIDATE_ONLY");
    }

    #[test]
    fn rejects_duplicate_pids_handles_and_invalid_evidence_class() {
        let mutations: Vec<Box<dyn Fn(&mut serde_json::Value)>> = vec![
            Box::new(|r| r["candidates"][1]["pid"] = json!(123)),
            Box::new(|r| r["candidates"][1]["windowHandle"] = json!("0xaa")),
            Box::new(|r| r["candidates"][1]["windowHandle"] = json!("0x00aa")),
            Box::new(|r| r["candidates"][0]["pid"] = json!(0)),
            Box::new(|r| r["candidates"][0]["windowHandle"] = json!("0x0")),
            Box::new(|r| r["evidenceClass"] = json!("GAMEPLAY_VERIFIED")),
            Box::new(|r| r["schema"] = json!(2)),
        ];
        for mutate in mutations {
            let mut report = fixture();
            mutate(&mut report);
            assert!(parse_native_observation(&report.to_string()).is_err());
        }
    }

    #[test]
    fn rejects_unbounded_and_extra_private_data() {
        let mut report = fixture();
        report["candidates"] = json!((1..=17)
            .map(|pid| json!({
                "pid": pid, "windowHandle": null
            }))
            .collect::<Vec<_>>());
        assert!(parse_native_observation(&report.to_string()).is_err());

        let mut report = fixture();
        report["candidates"][0]["accountEmail"] = json!("private@example.com");
        assert!(parse_native_observation(&report.to_string()).is_err());
    }

    #[test]
    fn rejects_non_education_exe_and_unsafe_paths() {
        for bad in [
            "",
            "notepad.exe",
            "C:/Tools/notepad.exe",
            "C:/Tools/Minecraft.Windows.cmd",
            "../Minecraft.Windows.exe",
            "C:/Minecraft.Windows.exe\nother",
        ] {
            assert!(validate_education_path(bad).is_err());
        }
    }

    #[test]
    fn empty_observation_is_not_a_game_failure() {
        let mut report = fixture();
        report["candidates"] = json!([]);
        let parsed = parse_native_observation(&report.to_string()).unwrap();
        assert!(parsed.candidates.is_empty());
        assert!(parsed.interactive_session);
    }
}
