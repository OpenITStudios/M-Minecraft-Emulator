//! Lazy Tester transport. The existing bootstrap owns games in one retained helper.
use super::owned_provider::OwnedProcess;
use serde::{Deserialize, Serialize};
use std::{
    io::{self, BufRead},
    sync::Mutex,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProfileSession {
    pub slot: u32,
    pub state: String,
    pub process: Option<OwnedProcess>,
    pub issue: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SessionSnapshot {
    pub schema: u32,
    pub profiles: Vec<ProfileSession>,
    pub storage: Option<Vec<ProfileStorage>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settings: Option<SettingsReport>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SettingsReport {
    #[serde(default = "default_graphics")]
    pub graphics_presets: [super::manager_settings::GraphicsPreset; 5],
    pub active_fps: u32,
    pub background_fps: u32,
    pub render_distance_chunks: u32,
    pub can_apply_preset: bool,
    pub changed_profiles: Option<u32>,
}
fn default_graphics() -> [super::manager_settings::GraphicsPreset; 5] { [super::manager_settings::GraphicsPreset::Low; 5] }
impl SettingsReport {
    pub fn policy(can_apply_preset: bool, changed_profiles: Option<u32>) -> io::Result<Self> {
        // Configured ceilings of the reviewed payload, never measured frame rates.
        let limits=super::manager_settings::load_performance()?;
        Ok(Self { graphics_presets: super::manager_settings::load_graphics()?, active_fps: limits.active_fps, background_fps: limits.background_fps, render_distance_chunks: 12,
            can_apply_preset, changed_profiles })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProfileStorage {
    pub slot: u32,
    pub bytes: u64,
    pub files: u32,
    pub partial: bool,
    pub disk: Option<DiskSpace>,
    pub cache: CacheStorage,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DiskSpace {
    pub available_bytes: u64,
    pub total_bytes: u64,
}
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CacheStorage {
    pub bytes: u64,
    pub files: u32,
    pub partial: bool,
    pub can_clear: bool,
    pub issue: Option<String>,
}
impl SessionSnapshot {
    pub fn stopped() -> Self {
        Self {
            schema: 1,
            storage: None,
            settings: None,
            profiles: (1..=5)
                .map(|slot| ProfileSession {
                    slot,
                    state: "STOPPED".into(),
                    process: None,
                    issue: None,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "operation", deny_unknown_fields)]
pub enum SessionRequest {
    Inspect,
    InspectSettings,
    ApplyPreset,
    ApplyGraphicsPreset { slot: u32, preset: super::manager_settings::GraphicsPreset },
    SavePerformance { #[serde(rename="activeFps")] active_fps:u32, #[serde(rename="backgroundFps")] background_fps:u32 },
    InspectStorage,
    OpenProfileFolder { slot: u32 },
    ClearProfileCache { slot: u32 },
    Launch { slot: u32 },
    CloseAll,
}

pub(crate) type SessionReply = Result<SessionSnapshot, String>;
#[cfg(windows)]
pub(crate) mod pipe;

#[cfg(windows)]
struct PendingRequest {
    replies: std::sync::mpsc::Receiver<io::Result<SessionReply>>,
    worker: std::thread::JoinHandle<()>,
}
#[cfg(windows)]
struct Controller {
    pending: Option<PendingRequest>,
    uncertain: bool,
}
#[cfg(windows)]
static CONTROLLER: Mutex<Controller> = Mutex::new(Controller {
    pending: None,
    uncertain: false,
});

#[cfg(windows)]
pub fn request(request: SessionRequest) -> io::Result<SessionSnapshot> {
    if let SessionRequest::SavePerformance {active_fps,background_fps}=request {
        super::manager_settings::PerformanceSettings {active_fps,background_fps}.validate()?;
    }
    if matches!(request, SessionRequest::ApplyGraphicsPreset { slot, .. } | SessionRequest::Launch { slot } | SessionRequest::OpenProfileFolder { slot } | SessionRequest::ClearProfileCache { slot } if !(1..=5).contains(&slot))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Unknown Minecraft profile",
        ));
    }
    let inspecting = matches!(request, SessionRequest::Inspect | SessionRequest::InspectSettings);
    let mut owner = CONTROLLER
        .lock()
        .map_err(|_| io::Error::other("Session transport unavailable"))?;
    if let Some(pending) = &owner.pending {
        match pending.replies.try_recv() {
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "The previous session request is still running; games are preserved",
                ))
            }
            _ => {
                let pending = owner.pending.take().unwrap();
                let _ = pending.worker.join();
            }
        }
    }
    if owner.uncertain && !inspecting {
        return Err(io::Error::other(
            "Previous request outcome is uncertain. Refresh session status before another action.",
        ));
    }
    let (sender, replies) = std::sync::mpsc::sync_channel(1);
    let worker = std::thread::spawn(move || {
        let _ = sender.send(transact(request));
    });
    owner.pending = Some(PendingRequest { replies, worker });
    let outcome = owner
        .pending
        .as_ref()
        .unwrap()
        .replies
        .recv_timeout(std::time::Duration::from_secs(45));
    match outcome {
        Ok(result) => {
            let pending = owner.pending.take().unwrap();
            let _ = pending.worker.join();
            match result {
                Ok(reply) => {
                    if inspecting && reply.is_ok() {
                        owner.uncertain = false;
                    }
                    reply.map_err(io::Error::other)
                }
                Err(error) => {
                    owner.uncertain = true;
                    Err(error)
                }
            }
        }
        Err(_) => {
            // Keep this exact worker/buffers/connection alive. Never start a replacement on timeout.
            owner.uncertain = true;
            Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "Session request is still pending; refresh to observe its eventual outcome",
            ))
        }
    }
}

#[cfg(windows)]
fn transact(request: SessionRequest) -> io::Result<SessionReply> {
    use std::{
        io::BufReader,
        os::windows::process::CommandExt,
        process::{Command, Stdio},
    };
    let endpoint = pipe::Endpoint::current()?;
    match endpoint.connect() {
        Ok(connection) => return pipe::exchange(&endpoint, connection, &request),
        Err(error) if error.raw_os_error() == Some(2) => {}
        Err(error) => return Err(error), // Busy/denied/forged is never permission to spawn.
    }
    // Another build or the historical private runner may retain the same profiles.
    // An absent compatible endpoint must not be presented as five stopped profiles.
    if existing_controller_gate("Local\\MivubiEducationStartupObservation")? {
        return Err(io::Error::new(io::ErrorKind::WouldBlock,
            "An existing Minecraft controller uses another build or is still starting. Its games are preserved; no replacement was started."));
    }
    // Closing an idle manager must never provision/start a new controller or game.
    if matches!(request,SessionRequest::CloseAll) {return Ok(Ok(SessionSnapshot::stopped()));}
    if matches!(request, SessionRequest::Inspect | SessionRequest::InspectSettings | SessionRequest::SavePerformance { .. }) {
        let mut snapshot = SessionSnapshot::stopped();
        if let SessionRequest::SavePerformance { active_fps, background_fps } = request {
            super::manager_settings::save_performance(super::manager_settings::PerformanceSettings {active_fps,background_fps})?;
        }
        if !matches!(request, SessionRequest::Inspect) {
            let idle = super::native_clients::inspect_native_workspace()?.processes.is_empty();
            snapshot.settings = Some(SettingsReport::policy(idle, None)?);
        }
        return Ok(Ok(snapshot));
    }
    // Explicit user action only. A fresh UI first tries the existing authenticated endpoint.
    let mut child = Command::new(std::env::current_exe()?)
        .arg("--lazy-tester-controller")
        .creation_flags(0x08000000)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let output = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("Missing controller startup output"))?;
    let mut reader = BufReader::new(output);
    let startup = read_message(&mut reader)?
        .ok_or_else(|| io::Error::other("Controller exited before startup"))?;
    let ready: Result<(), String> = serde_json::from_str(&startup).map_err(io::Error::other)?;
    if let Err(error) = ready {
        return Ok(Err(error));
    }
    // Dropping Child never kills the controller. It owns game lifetime independently of this UI.
    pipe::exchange(&endpoint, endpoint.connect()?, &request)
}

#[cfg(windows)]
pub(crate) fn existing_controller_gate(name: &str) -> io::Result<bool> {
    use std::os::windows::io::{FromRawHandle, OwnedHandle};
    use windows_sys::Win32::{
        Foundation::GetLastError,
        System::Threading::{OpenMutexW, SYNCHRONIZATION_SYNCHRONIZE},
    };
    let name: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
    let handle = unsafe { OpenMutexW(SYNCHRONIZATION_SYNCHRONIZE, 0, name.as_ptr()) };
    if !handle.is_null() {
        let _gate = unsafe { OwnedHandle::from_raw_handle(handle as _) };
        return Ok(true);
    }
    let error = unsafe { GetLastError() };
    if error == 2 {
        Ok(false)
    } else {
        Err(io::Error::from_raw_os_error(error as i32))
    }
}

#[cfg(not(windows))]
pub fn request(_: SessionRequest) -> io::Result<SessionSnapshot> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Minecraft sessions require Windows",
    ))
}

pub(super) fn read_message(input: &mut impl io::BufRead) -> io::Result<Option<String>> {
    let mut bytes = Vec::new();
    let count = io::Read::take(input, 32769).read_until(b'\n', &mut bytes)?;
    if count == 0 {
        return Ok(None);
    }
    if count > 32768 || bytes.last() != Some(&b'\n') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid session message",
        ));
    }
    String::from_utf8(bytes).map(Some).map_err(io::Error::other)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn settings_report_describes_the_reviewed_policy_without_starting_a_controller() {
        let policy = SettingsReport::policy(false, None).unwrap();
        assert_eq!((policy.active_fps, policy.background_fps, policy.render_distance_chunks), (144, 10, 12));
        let pacing = include_str!("../../../../../tooling/acceptance/windows/bootstrap-ordering/frame-pacing.h");
        assert!(pacing.contains("activeFps = 144, backgroundFps = 10"));
        let snapshot = SessionSnapshot::stopped();
        assert!(!serde_json::to_value(&snapshot).unwrap().as_object().unwrap().contains_key("settings"));
        assert!(serde_json::from_str::<SessionRequest>(r#"{"operation":"ApplyPreset"}"#).is_ok());
        assert!(serde_json::from_str::<SessionRequest>(r#"{"operation":"InspectSettings"}"#).is_ok());
        #[cfg(windows)]
        assert_eq!(request(SessionRequest::SavePerformance {active_fps:0,background_fps:10}).unwrap_err().kind(),io::ErrorKind::InvalidInput);
    }
    #[cfg(windows)]
    #[test]
    fn existing_incompatible_controller_gate_is_distinguished_from_absence() {
        use std::os::windows::io::{FromRawHandle, OwnedHandle};
        let name = format!(
            "Local\\LazyTester-disposable-gate-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        assert!(!existing_controller_gate(&name).unwrap());
        let wide: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
        let raw = unsafe {
            windows_sys::Win32::System::Threading::CreateMutexW(std::ptr::null(), 0, wide.as_ptr())
        };
        assert!(!raw.is_null());
        let handle = unsafe { OwnedHandle::from_raw_handle(raw as _) };
        assert!(existing_controller_gate(&name).unwrap());
        drop(handle);
        assert!(!existing_controller_gate(&name).unwrap());
    }
    #[test]
    fn protocol_rejects_paths_unknown_operations_and_unbounded_input() {
        for input in [
            r#"{"operation":"Launch","slot":1,"path":"other.exe"}"#,
            r#"{"operation":"Delete","slot":1}"#,
            r#"{"operation":"SavePerformance","activeFps":60,"backgroundFps":5,"path":"other.json"}"#,
            r#"{"operation":"SavePerformance","activeFps":60.5,"backgroundFps":5}"#,
        ] {
            assert!(serde_json::from_str::<SessionRequest>(input).is_err());
        }
        assert!(read_message(&mut io::Cursor::new(vec![b'x'; 32769])).is_err());
        assert!(read_message(&mut io::Cursor::new(b"{}".to_vec())).is_err());
        assert_eq!(
            SessionSnapshot::stopped()
                .profiles
                .iter()
                .map(|p| p.slot)
                .collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5]
        );
    }
}
