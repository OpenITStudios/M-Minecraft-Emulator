//! One retained bootstrap owner. Closing the UI detaches; games and FPS policy survive.
use super::*;
use crate::engine::{
    native_clients_session::{
        pipe::{Endpoint, Peer, Server},
        ProfileSession, ProfileStorage, SessionRequest, SessionSnapshot,
    },
    owned_provider::OwnedProcess,
};
use std::{io::Write, time::Duration};

struct Sessions {
    stores: Vec<Option<SlotStorage>>,
    running: Vec<Option<StartedEducationObservation<1>>>,
    issues: Vec<Option<String>>,
    windows: Vec<Option<OwnedProcess>>,
    path: String,
    image: File,
}
impl Sessions {
    fn prepare() -> io::Result<Self> {
        // Do not adopt arbitrary games or infer a profile from window titles.
        if !super::super::inspect_native_workspace()?
            .processes
            .is_empty()
        {
            return Err(io::Error::new(io::ErrorKind::AlreadyExists,
                "Existing Minecraft sessions are preserved. Close them normally before starting managed profiles."));
        }
        let (path, image) = registered_image(true)?;
        let stores = registered_slots::<5>(true)?.into_iter().map(Some).collect();
        // Validate the reviewed DLLs before the first settings write or game process.
        drop(BootstrapLibrary::reviewed()?);
        Ok(Self {
            stores,
            running: (0..5).map(|_| None).collect(),
            issues: vec![None; 5],
            windows: vec![None; 5],
            path,
            image,
        })
    }

    fn reap(&mut self) {
        for index in 0..5 {
            let Some(started) = self.running[index].as_ref() else {
                continue;
            };
            let code = match started.children[0].exit_code_after_exit() {
                Ok(code) => code,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => continue,
                Err(error) => {
                    self.issues[index] = Some(error.to_string());
                    continue;
                }
            };
            if code != 0 {
                self.issues[index] = Some(format!("Minecraft exited with code {code}"));
            }
            if let Err(error) = started.save_after_exit(1) {
                self.issues[index] = Some(format!("Could not save startup journal: {error}"));
            }
            let started = self.running[index].take().unwrap();
            let [mut child] = started.children;
            self.stores[index] = child.storage.take();
            self.windows[index] = None;
        }
    }

    fn launch(&mut self, slot: u32) -> io::Result<()> {
        if !(1..=5).contains(&slot) {
            return Err(invalid("Unknown Minecraft profile"));
        }
        let index = slot as usize - 1;
        self.reap();
        if self.running[index].is_some() {
            return Err(invalid(
                "Minecraft profile is already running or its outcome is uncertain",
            ));
        }
        let payload = BootstrapLibrary::reviewed()?;
        // Revalidate registration and exact game bytes before every launch.
        let (path, image) = registered_image(true)?;
        if path != self.path || !same_image(&image_id(&image)?, &image_id(&self.image)?) {
            return Err(invalid("Registered game changed during this session"));
        }
        let store = self.stores[index]
            .as_mut()
            .ok_or_else(|| invalid("Profile requires controller restart before retry"))?;
        store.singleton_caller_rva = 0x2cdc30;
        game_preset::apply_inactive(std::slice::from_ref(store))?;
        let child = create_unstarted_configured(
            Path::new(&self.path),
            self.image.try_clone()?,
            self.stores[index].take(),
            true,
            Some(&payload),
            #[cfg(test)]
            None,
        )?;
        // A pre-resume error cancels only this never-resumed child. Persistent files survive.
        payload.enroll(&child)?;
        let started = PreparedEducationEnrollment {
            _children: [child],
            _payload: payload,
        }
        .resume_for_authorized_observation()?;
        self.issues[index] = match &started.attempts[0] {
            StartupAttempt::Resumed => None,
            _ => Some("Resume outcome is uncertain; game preserved without retry".into()),
        };
        self.running[index] = Some(started);
        Ok(())
    }

    fn snapshot(&mut self) -> SessionSnapshot {
        self.reap();
        let profiles = (0..5)
            .map(|index| {
                let process = self.running[index].as_ref().and_then(|started| {
                    let child = &started.children[0];
                    child.identity().ok()?;
                    if let Some(previous) = &self.windows[index] {
                        if valid_retained_window(previous) {
                            let mut process = previous.clone();
                            process.responding = unsafe {
                                windows_sys::Win32::UI::WindowsAndMessaging::IsHungAppWindow(
                                    usize::from_str_radix(&process.window_handle[2..], 16).ok()?
                                        as _,
                                )
                            } == 0;
                            return Some(process);
                        }
                    }
                    window(child)
                });
                self.windows[index] = process.clone();
                let state = if self.running[index].is_some() {
                    match process.as_ref() {
                        Some(p) if p.responding => "RUNNING",
                        Some(_) => "NOT_RESPONDING",
                        None => "STARTING",
                    }
                } else if self.issues[index].is_some() || self.stores[index].is_none() {
                    "ATTENTION"
                } else {
                    "STOPPED"
                };
                ProfileSession {
                    slot: index as u32 + 1,
                    state: state.into(),
                    process,
                    issue: self.issues[index].clone(),
                }
            })
            .collect();
        SessionSnapshot {
            schema: 1,
            profiles,
            storage: None,
        }
    }

    fn store(&self, slot: u32) -> io::Result<&SlotStorage> {
        let index = slot
            .checked_sub(1)
            .filter(|index| *index < 5)
            .ok_or_else(|| invalid("Unknown profile"))? as usize;
        self.stores[index]
            .as_ref()
            .or_else(|| {
                self.running[index]
                    .as_ref()
                    .and_then(|started| started.children[0].storage.as_ref())
            })
            .ok_or_else(|| invalid("Profile storage is unavailable"))
    }

    fn storage(&self) -> io::Result<Vec<ProfileStorage>> {
        (1..=5)
            .map(|slot| {
                let store = self.store(slot)?;
                let path = directory_path(&store.private)?;
                let mut result = ProfileStorage {
                    slot,
                    bytes: 0,
                    files: 0,
                    partial: false,
                };
                scan(&path, &mut result, std::time::Instant::now(), 0);
                Ok(result)
            })
            .collect()
    }

    fn open_folder(&self, slot: u32) -> io::Result<()> {
        let store = self.store(slot)?;
        let path = directory_path(&store.private)?;
        // Keep the validated directory pinned while opening its canonical location.
        let pinned = open_directory(&path, FILE_READ_ATTRIBUTES)?;
        if !same_image(&directory_id(&pinned)?, &directory_id(&store.private)?) {
            return Err(invalid("Profile directory changed"));
        }
        let path = wide(path.as_os_str())?;
        let outcome = unsafe {
            windows_sys::Win32::UI::Shell::ShellExecuteW(
                ptr::null_mut(),
                windows_sys::w!("open"),
                path.as_ptr(),
                ptr::null(),
                ptr::null(),
                1,
            )
        };
        if outcome as isize <= 32 {
            return Err(io::Error::other(
                "Windows could not open the profile folder",
            ));
        }
        Ok(())
    }
}

// Metadata only, bounded, and no traversal through junctions/symlinks. Not a cache classifier.
fn scan(path: &Path, result: &mut ProfileStorage, started: std::time::Instant, depth: usize) {
    if depth > 48 || result.files >= 100_000 || started.elapsed() > Duration::from_secs(2) {
        result.partial = true;
        return;
    }
    let Ok(_pin) = open_directory(path, FILE_READ_ATTRIBUTES) else {
        result.partial = true;
        return;
    };
    let Ok(entries) = std::fs::read_dir(path) else {
        result.partial = true;
        return;
    };
    for entry in entries {
        if result.files >= 100_000 || started.elapsed() > Duration::from_secs(2) {
            result.partial = true;
            break;
        }
        let Ok(entry) = entry else {
            result.partial = true;
            continue;
        };
        let Ok(metadata) = std::fs::symlink_metadata(entry.path()) else {
            result.partial = true;
            continue;
        };
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            result.partial = true;
            continue;
        }
        if metadata.is_dir() {
            scan(&entry.path(), result, started, depth + 1);
        } else if metadata.is_file() {
            result.bytes = result.bytes.saturating_add(metadata.len());
            result.files += 1;
        }
    }
}

fn valid_retained_window(target: &OwnedProcess) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::*;
    let Ok(number) = usize::from_str_radix(&target.window_handle[2..], 16) else {
        return false;
    };
    let hwnd = number as windows_sys::Win32::Foundation::HWND;
    let mut pid = 0;
    unsafe {
        IsWindow(hwnd) != 0
            && GetWindowThreadProcessId(hwnd, &mut pid) != 0
            && pid == target.pid
            && GetAncestor(hwnd, GA_ROOT) == hwnd
            && GetWindow(hwnd, GW_OWNER).is_null()
    }
}

fn window(child: &PendingEducationBootstrap) -> Option<OwnedProcess> {
    use windows_sys::Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{EnumWindows, GetWindowThreadProcessId},
    };
    struct Selection {
        pid: u32,
        hwnd: HWND,
        count: u32,
    }
    unsafe extern "system" fn select(hwnd: HWND, data: isize) -> i32 {
        let selection = &mut *(data as *mut Selection);
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == selection.pid && super::super::verify_live_client_02_window(hwnd, pid).is_ok() {
            selection.hwnd = hwnd;
            selection.count += 1;
        }
        1
    }
    child.identity().ok()?;
    let mut selected = Selection {
        pid: child.pid,
        hwnd: ptr::null_mut(),
        count: 0,
    };
    if unsafe { EnumWindows(Some(select), &mut selected as *mut _ as isize) } == 0
        || selected.count != 1
    {
        return None;
    }
    child.identity().ok()?;
    let slot = child.storage.as_ref()?.slot;
    let _ = label_slot_window(selected.hwnd, child.pid, slot);
    Some(OwnedProcess {
        pid: child.pid,
        created_file_time: child.created.to_string(),
        session_id: interactive_session(child.pid).ok()?,
        window_handle: format!("0x{:x}", selected.hwnd as usize),
        responding: true,
    })
}

fn run() -> io::Result<()> {
    use windows_sys::Win32::{
        Foundation::{GetLastError, ERROR_ALREADY_EXISTS},
        System::Threading::CreateMutexW,
    };
    crate::engine::workspace_surface::verify_same_executable_parent()?;
    crate::engine::dpi::configure()?;
    // Same gate as historical pair/five CLI: no parallel lifecycle authority.
    let name = wide(OsStr::new("Local\\MivubiEducationStartupObservation"))?;
    let raw = unsafe { CreateMutexW(ptr::null(), 0, name.as_ptr()) };
    if raw.is_null() {
        return Err(io::Error::last_os_error());
    }
    let existed = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
    let _gate = unsafe { OwnedHandle::from_raw_handle(raw as _) };
    if existed {
        return Err(invalid(
            "Another Minecraft controller is still active; existing games are preserved",
        ));
    }
    let mut sessions = Sessions::prepare()?;
    let server = Server::start(Endpoint::current()?)?;
    let mut manager: Option<Peer> = None;
    let startup = std::time::Instant::now();
    {
        let mut output = io::stdout().lock();
        serde_json::to_writer(&mut output, &Result::<(), String>::Ok(()))?;
        output.write_all(b"\n")?;
        output.flush()?;
    }
    loop {
        sessions.reap();
        let detached = manager.as_ref().map_or_else(
            || startup.elapsed() > Duration::from_secs(45),
            |peer| !peer.alive(),
        );
        if detached && sessions.running.iter().all(Option::is_none) {
            return Ok(());
        }
        let envelope = match server.requests.recv_timeout(Duration::from_millis(250)) {
            Ok(envelope) => envelope,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
            Err(_) => {
                // Even a failed transport must retain game/storage/FPS ownership until normal exit.
                if sessions.running.iter().all(Option::is_none) {
                    return Err(invalid("Session transport stopped"));
                }
                std::thread::sleep(Duration::from_millis(250));
                continue;
            }
        };
        if !envelope.peer.alive() {
            let _ = envelope
                .reply
                .send(Err("Manager exited before request admission".into()));
            continue;
        }
        manager = Some(envelope.peer);
        let reply = (|| {
            let mut storage = None;
            match envelope.request {
                SessionRequest::Inspect => {}
                SessionRequest::Launch { slot } => {
                    sessions.launch(slot).map_err(|error| error.to_string())?
                }
                SessionRequest::InspectStorage => {
                    storage = Some(sessions.storage().map_err(|error| error.to_string())?)
                }
                SessionRequest::OpenProfileFolder { slot } => sessions
                    .open_folder(slot)
                    .map_err(|error| error.to_string())?,
            }
            let mut snapshot = sessions.snapshot();
            snapshot.storage = storage;
            Ok(snapshot)
        })();
        let _ = envelope.reply.send(reply); // A disconnected UI never cancels an admitted launch.
    }
}

pub(super) fn maybe_run() -> bool {
    if std::env::args_os().nth(1).as_deref() != Some(OsStr::new("--lazy-tester-controller")) {
        return false;
    }
    let result = if std::env::args_os().count() == 2 {
        run()
    } else {
        Err(invalid("Controller accepts no extra arguments"))
    };
    if let Err(error) = result {
        let reply: Result<SessionSnapshot, String> = Err(error.to_string());
        let mut output = io::stdout().lock();
        let _ = serde_json::to_writer(&mut output, &reply);
        let _ = output.write_all(b"\n");
        let _ = output.flush();
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_scan_counts_files_and_reports_bounds_without_opening_contents() {
        let directory = std::env::temp_dir().join(format!(
            "lazy-tester-storage-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&directory).unwrap();
        std::fs::create_dir(directory.join("nested")).unwrap();
        std::fs::write(directory.join("a.bin"), [0u8; 17]).unwrap();
        std::fs::write(directory.join("nested/b.bin"), [0u8; 23]).unwrap();
        let mut result = ProfileStorage {
            slot: 5,
            bytes: 0,
            files: 0,
            partial: false,
        };
        scan(&directory, &mut result, std::time::Instant::now(), 0);
        assert_eq!((result.bytes, result.files, result.partial), (40, 2, false));
        scan(&directory, &mut result, std::time::Instant::now(), 49);
        assert_eq!((result.bytes, result.files, result.partial), (40, 2, true));
        // Only the unique, create-new disposable fixture directory created above.
        assert_eq!(directory.parent(), Some(std::env::temp_dir().as_path()));
        std::fs::remove_dir_all(directory).unwrap();
    }
}
