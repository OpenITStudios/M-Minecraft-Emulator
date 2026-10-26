//! Mixer control for retained controller-owned processes, on the controller thread.
use std::{
    collections::HashSet,
    io,
    time::{Duration, Instant},
};
use windows::{
    core::Interface,
    Win32::{Media::Audio::*, System::Com::*},
};

fn error(value: windows::core::Error) -> io::Error {
    io::Error::other(format!("Windows audio: {value}"))
}
struct Apartment;
impl Drop for Apartment {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}
struct Session {
    key: String,
    pid: u32,
    born: u64,
    volume: ISimpleAudioVolume,
    original_mute: bool,
    changed: bool,
}
impl Session {
    fn apply(&mut self, background: bool) -> io::Result<()> {
        // Never unmute a session that was already muted before this lease.
        let desired = self.original_mute || background;
        let current = unsafe { self.volume.GetMute() }.map_err(error)?.as_bool();
        if current != desired {
            // A foreground session never changed by us keeps any later user mute.
            if !background && !self.changed {
                return Ok(());
            }
            unsafe { self.volume.SetMute(desired, &CONTEXT) }.map_err(error)?;
            self.changed = desired != self.original_mute;
        }
        if !background && current == desired {
            self.changed = false;
        }
        Ok(())
    }
    fn restore(&mut self) -> io::Result<()> {
        if self.changed {
            // If another client changed the value, leave its choice alone.
            if unsafe { self.volume.GetMute() }.map_err(error)?.as_bool() {
                unsafe { self.volume.SetMute(self.original_mute, &CONTEXT) }.map_err(error)?;
            }
            self.changed = false;
        }
        Ok(())
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}
const CONTEXT: windows::core::GUID =
    windows::core::GUID::from_u128(0x978f5ab6_f1e5_489e_a3fb_f6148c53373b);

pub(super) struct Audio {
    // COM references must drop before this thread's apartment.
    sessions: Vec<Session>,
    enumerator: IMMDeviceEnumerator,
    scanned: Option<Instant>,
    targets: Vec<(u32, u64)>,
    _apartment: Apartment,
}
impl Audio {
    pub(super) fn new() -> io::Result<Self> {
        unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }
            .ok()
            .map_err(error)?;
        let apartment = Apartment;
        let enumerator =
            unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) }.map_err(error)?;
        Ok(Self {
            sessions: Vec::new(),
            enumerator,
            scanned: None,
            targets: Vec::new(),
            _apartment: apartment,
        })
    }
    pub(super) fn restore(&mut self) -> io::Result<()> {
        let mut failure = None;
        for session in &mut self.sessions {
            if let Err(e) = session.restore() {
                failure = Some(e);
            }
        }
        failure.map_or(Ok(()), Err)
    }
    pub(super) fn sync(
        &mut self,
        targets: &[(u32, u64)],
        foreground: Option<u32>,
        valid: impl Fn(u32, u64) -> bool,
    ) -> io::Result<()> {
        if targets != self.targets
            || self
                .scanned
                .is_none_or(|last| last.elapsed() >= Duration::from_secs(1))
        {
            self.discover(targets, &valid)?;
            self.targets = targets.to_vec();
            self.scanned = Some(Instant::now());
        }
        for session in &mut self.sessions {
            if valid(session.pid, session.born) {
                session.apply(foreground != Some(session.pid))?;
            }
        }
        Ok(())
    }
    fn discover(
        &mut self,
        targets: &[(u32, u64)],
        valid: &impl Fn(u32, u64) -> bool,
    ) -> io::Result<()> {
        let devices = unsafe {
            self.enumerator
                .EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)
        }
        .map_err(error)?;
        let count = unsafe { devices.GetCount() }.map_err(error)?;
        if count > 64 {
            return Err(io::Error::other("Too many audio endpoints"));
        }
        let mut seen = HashSet::new();
        let mut additions = Vec::new();
        for index in 0..count {
            let device = unsafe { devices.Item(index) }.map_err(error)?;
            let endpoint = take_string(unsafe { device.GetId() }.map_err(error)?)?;
            let manager: IAudioSessionManager2 =
                unsafe { device.Activate(CLSCTX_ALL, None) }.map_err(error)?;
            let sessions = unsafe { manager.GetSessionEnumerator() }.map_err(error)?;
            let count = unsafe { sessions.GetCount() }.map_err(error)?;
            if !(0..=256).contains(&count) {
                return Err(io::Error::other("Too many audio sessions"));
            }
            for index in 0..count {
                let control: IAudioSessionControl2 = unsafe { sessions.GetSession(index) }
                    .map_err(error)?
                    .cast()
                    .map_err(error)?;
                let mut pid = 0;
                // S_NO_SINGLE_PROCESS is success but cannot authorize cross-process mute.
                let result = unsafe {
                    (Interface::vtable(&control).GetProcessId)(
                        Interface::as_raw(&control),
                        &mut pid,
                    )
                };
                if result.0 != 0 || unsafe { control.IsSystemSoundsSession() }.0 != 1 {
                    continue;
                }
                let Some((_, born)) = targets.iter().find(|(owned, _)| *owned == pid) else {
                    continue;
                };
                if !valid(pid, *born) {
                    continue;
                }
                let instance =
                    take_string(unsafe { control.GetSessionInstanceIdentifier() }.map_err(error)?)?;
                let key = format!("{endpoint}\n{instance}");
                seen.insert(key.clone());
                if self
                    .sessions
                    .iter()
                    .any(|s| s.key == key && s.pid == pid && s.born == *born)
                {
                    continue;
                }
                let volume: ISimpleAudioVolume = control.cast().map_err(error)?;
                let original_mute = unsafe { volume.GetMute() }.map_err(error)?.as_bool();
                additions.push(Session {
                    key,
                    pid,
                    born: *born,
                    volume,
                    original_mute,
                    changed: false,
                });
            }
        }
        self.sessions
            .retain(|s| seen.contains(&s.key) && targets.contains(&(s.pid, s.born)));
        self.sessions.extend(additions);
        Ok(())
    }
}
fn take_string(value: windows::core::PWSTR) -> io::Result<String> {
    let result = unsafe { value.to_string() }
        .map_err(|_| io::Error::other("Invalid audio session identifier"));
    unsafe { CoTaskMemFree(Some(value.0 as _)) };
    result
}

pub(super) fn foreground() -> Option<u32> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowThreadProcessId,
    };
    let hwnd = unsafe { GetForegroundWindow() };
    let mut pid = 0;
    if !hwnd.is_null()
        && unsafe { GetWindowThreadProcessId(hwnd, &mut pid) } != 0
        && unsafe { GetForegroundWindow() } == hwnd
    {
        Some(pid)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, Write};
    fn silent_session(enumerator: &IMMDeviceEnumerator) -> (IAudioClient, ISimpleAudioVolume) {
        let device = unsafe { enumerator.GetDefaultAudioEndpoint(eRender, eConsole) }.unwrap();
        let client: IAudioClient = unsafe { device.Activate(CLSCTX_ALL, None) }.unwrap();
        let format = unsafe { client.GetMixFormat() }.unwrap();
        let guid = unsafe { CoCreateGuid() }.unwrap();
        let initialized = unsafe {
            client.Initialize(
                AUDCLNT_SHAREMODE_SHARED,
                0,
                1_000_000,
                0,
                format,
                Some(&guid),
            )
        };
        unsafe { CoTaskMemFree(Some(format as _)) };
        initialized.unwrap();
        let volume: ISimpleAudioVolume = unsafe { client.GetService() }.unwrap();
        (client, volume)
    }
    #[test]
    #[ignore = "subprocess fixture used only by the explicit Windows audio test"]
    fn audio_fixture_child() {
        if std::env::var("LAZY_AUDIO_SYNTHETIC_CHILD").as_deref() != Ok("1") {
            return;
        }
        unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }
            .ok()
            .unwrap();
        let _apartment = Apartment;
        let enumerator: IMMDeviceEnumerator =
            unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) }.unwrap();
        let (_client, volume) = silent_session(&enumerator);
        unsafe { volume.SetMute(false, &CONTEXT) }.unwrap();
        println!("AUDIO_FIXTURE_READY");
        std::io::stdout().flush().unwrap();
        for command in std::io::stdin().lock().lines() {
            if command.unwrap() == "EXIT" {
                break;
            }
            println!(
                "AUDIO_FIXTURE_MUTED={}",
                unsafe { volume.GetMute() }.unwrap().as_bool()
            );
            std::io::stdout().flush().unwrap();
        }
    }
    #[test]
    #[ignore = "creates only silent synthetic audio sessions on the default Windows output"]
    fn windows_audio_preserves_user_mute_and_scopes_verified_targets() {
        use std::process::{Command, Stdio};
        let mut child=Command::new(std::env::current_exe().unwrap()).args(["--exact","engine::native_clients::independent_bootstrap::session_audio::tests::audio_fixture_child","--ignored","--nocapture"])
            .env("LAZY_AUDIO_SYNTHETIC_CHILD","1").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
        let mut input = child.stdin.take().unwrap();
        let mut output = std::io::BufReader::new(child.stdout.take().unwrap());
        let mut line = String::new();
        loop {
            line.clear();
            assert!(output.read_line(&mut line).unwrap() > 0);
            if line.trim() == "AUDIO_FIXTURE_READY" {
                break;
            }
        }
        unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }
            .ok()
            .unwrap();
        let _apartment = Apartment;
        let mut audio = Audio::new().unwrap();
        let mut clients = Vec::new();
        let mut volumes = Vec::new();
        for original in [false, true] {
            let (client, volume) = silent_session(&audio.enumerator);
            unsafe { volume.SetMute(original, &CONTEXT) }.unwrap();
            clients.push(client);
            volumes.push(volume);
        }
        let pid = std::process::id();
        let targets = [(pid, 123)];
        // No retained identity, no writes even when the PID matches.
        audio.sync(&targets, None, |_, _| false).unwrap();
        assert!(!unsafe { volumes[0].GetMute() }.unwrap().as_bool());
        assert!(unsafe { volumes[1].GetMute() }.unwrap().as_bool());
        writeln!(input, "GET").unwrap();
        input.flush().unwrap();
        line.clear();
        output.read_line(&mut line).unwrap();
        assert_eq!(
            line.trim(),
            "AUDIO_FIXTURE_MUTED=false",
            "Other process audio must remain unchanged"
        );
        audio.scanned = None;
        audio
            .sync(&targets, None, |p, b| p == pid && b == 123)
            .unwrap();
        assert!(
            audio.sessions.len() >= 2,
            "Synthetic sessions must actually be discovered"
        );
        assert!(unsafe { volumes[0].GetMute() }.unwrap().as_bool());
        assert!(unsafe { volumes[1].GetMute() }.unwrap().as_bool());
        audio
            .sync(&targets, Some(pid), |p, b| p == pid && b == 123)
            .unwrap();
        assert!(!unsafe { volumes[0].GetMute() }.unwrap().as_bool());
        assert!(unsafe { volumes[1].GetMute() }.unwrap().as_bool());
        // A later foreground user mute is not cleared by repeated policy updates.
        unsafe { volumes[0].SetMute(true, &CONTEXT) }.unwrap();
        audio.sync(&targets, Some(pid), |_, _| true).unwrap();
        assert!(unsafe { volumes[0].GetMute() }.unwrap().as_bool());
        unsafe { volumes[0].SetMute(false, &CONTEXT) }.unwrap();
        audio.sync(&targets, None, |_, _| true).unwrap();
        audio.restore().unwrap();
        assert!(!unsafe { volumes[0].GetMute() }.unwrap().as_bool());
        assert!(unsafe { volumes[1].GetMute() }.unwrap().as_bool());
        // Empty ownership removes leases and restores only our own mute.
        audio.sync(&targets, None, |_, _| true).unwrap();
        audio.sync(&[], None, |_, _| false).unwrap();
        assert!(!unsafe { volumes[0].GetMute() }.unwrap().as_bool());
        assert!(unsafe { volumes[1].GetMute() }.unwrap().as_bool());
        drop(audio);
        drop(volumes);
        drop(clients);
        writeln!(input, "EXIT").unwrap();
        drop(input);
        assert!(child.wait().unwrap().success());
    }
}
