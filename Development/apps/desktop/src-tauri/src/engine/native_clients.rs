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
// Only the reviewed Windows package may inspect/modify its five saved profile
// identities. Keep the existing Windows source/preview fallback unchanged.
#[cfg(all(windows, target_arch = "x86_64", feature = "lazy-tester-package"))]
pub fn data_location(action: DataLocationAction) -> io::Result<DataLocation> {
    independent_bootstrap::product_profiles::location(action)
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
    independent_bootstrap::profile_binding::run(import)
}

#[cfg(all(windows, not(all(target_arch = "x86_64", feature = "lazy-tester-package"))))]
pub fn profile_binding(import: bool) -> io::Result<ProfileBinding> {
    let _ = import;
    Ok(ProfileBinding { schema: 1, required: false, configured: true, issue: None })
}

// A non-Windows GUI must never show five profiles as ready merely because no
// profile binder exists. No account/world mutation or simulated readiness.
#[cfg(not(windows))]
pub fn profile_binding(_: bool) -> io::Result<ProfileBinding> {
    Err(unsupported_native_profiles())
}

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
        include_str!("../../../../../tooling/acceptance/windows/education-target.ps1");
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
        include_str!("../../../../../tooling/acceptance/windows/education-target.ps1");
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
    pub processes: Vec<super::process_identity::OwnedProcess>,
}

// Lazy Tester owns five launcher slots. The registered-image observer also
// sees unrelated game candidates, so >5 still fails closed rather than
// silently truncating identities. This is an observation cap, not ownership.
const MAX_NATIVE_WORKSPACE_PROCESSES: usize = 5;

fn parse_native_workspace_observation(raw: &str) -> io::Result<NativeWorkspaceObservation> {
    let result: NativeWorkspaceObservation = serde_json::from_str(raw)
        .map_err(|_| invalid_evidence("Malformed native workspace observation"))?;
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

fn legacy_auth_helper_path(target: &std::path::Path) -> io::Result<std::path::PathBuf> {
    let helper = target.with_file_name("Minecraft.AdalServer.exe");
    let metadata = match std::fs::symlink_metadata(&helper) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Err(io::Error::new(io::ErrorKind::Unsupported,
                "This installation does not include the legacy Minecraft.AdalServer.exe helper. Authentication routing is UNKNOWN; this inspection covers that helper only."));
        }
        Err(error) => return Err(error),
    };
    if !metadata.file_type().is_file() {
        return Err(invalid_evidence("Legacy sign-in helper candidate is not an ordinary file"));
    }
    Ok(helper) // Existence is not signature, runtime attribution or enrollment approval.
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
    ) -> io::Result<Option<super::super::process_identity::OwnedProcess>> {
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
        Ok(Some(super::super::process_identity::OwnedProcess {
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
                    if processes.len() > MAX_NATIVE_WORKSPACE_PROCESSES {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "More than five registered Minecraft processes"));
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
    /// the documented legacy Minecraft.AdalServer.exe candidate, when installed. Names
    /// and Toolhelp parent PIDs are merely candidates; the helper could be
    /// transient, launched by another process, or replaced while inspecting.
    pub(super) fn inspect_auth_family() -> io::Result<NativeAuthFamilyObservation> {
        let target = super::registered_education_target()?;
        let helper_path = legacy_auth_helper_path(std::path::Path::new(&target))?;
        let before = inspect_for_registered_target(&target)?;
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
        assert!(!maybe_run_education_startup_observation());
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

