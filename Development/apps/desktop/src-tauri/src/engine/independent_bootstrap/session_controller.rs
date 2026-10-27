//! One retained bootstrap owner. Full exit requests normal closure of its own games.
use super::*;
use crate::engine::{
    native_clients_session::{
        pipe::{Endpoint, Peer, Server},
        ProfileSession, ProfileStorage, SessionRequest, SessionSnapshot, SettingsReport,
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
    exiting: bool,
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
            exiting: false,
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
        if self.exiting {return Err(invalid("Minecraft sessions are closing. Finish closing them before starting another game."));}
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
        #[cfg(feature="lazy-tester-package")]
        if product_profiles::clear::pending(self.store(slot)?)? {
            return Err(invalid("Finish clearing this launcher's old data in Data Storage before starting it"));
        }
        let payload = BootstrapLibrary::reviewed()?;
        crate::engine::manager_settings::load_performance()?;
        // Recheck current registration/signature/path identity and exact bytes
        // against the original retained image without a redundant initial resolve.
        validate_registered_image(&self.path,&self.image,true)?;
        let store = self.stores[index]
            .as_mut()
            .ok_or_else(|| invalid("Profile is unavailable after unconfirmed startup cleanup; existing processes are preserved"))?;
        store.singleton_caller_rva = 0x2cdc30;
        game_preset::apply_inactive(std::slice::from_ref(store))?;
        // Keep the exact validated store until ownership reaches the started
        // transaction. Cloning handles neither reopens paths nor seeds a profile.
        let cleanup = UnstartedCleanup::new();
        let attempt = (|| {
            let child = create_unstarted_tracked(
                Path::new(&self.path),
                self.image.try_clone()?,
                Some(self.stores[index].as_ref().unwrap().duplicate()?),
                true,
                Some(&payload),
                Some(cleanup.clone()),
                #[cfg(test)]
                None,
            )?;
            payload.enroll(&child)?;
            PreparedEducationEnrollment {
                _children: [child],
                _payload: payload,
            }
            .resume_for_authorized_observation()
        })();
        let started = match attempt {
            Ok(started) => started,
            Err(error) => {
                let issue = if cleanup.retry_safe() {
                    error.to_string()
                } else {
                    // Never restore retry permission merely because an error
                    // returned: cancellation must confirm this child's exit.
                    self.stores[index] = None;
                    format!("Startup cleanup is unconfirmed; retry disabled and existing processes preserved: {error}")
                };
                self.issues[index] = Some(issue.clone());
                return Err(io::Error::other(issue));
            }
        };
        self.stores[index] = None; // Started transaction now owns the duplicate pins.
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
                            // The game can reset its caption after a desktop/workspace transition.
                            let hwnd = usize::from_str_radix(&previous.window_handle[2..], 16).ok()?;
                            // Do not mutate game or dialog titles once full exit begins.
                            if !self.exiting {
                                let _ = label_slot_window(hwnd as _, child.pid, index as u32 + 1);
                            }
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
                    window(child, !self.exiting)
                });
                self.windows[index] = process.clone();
                let mut issue=self.issues[index].clone();
                #[cfg(feature="lazy-tester-package")]
                if self.running[index].is_none() {
                    if let Some(store)=&self.stores[index] {
                        match product_profiles::clear::pending(store) {
                            Ok(false)=>{},
                            Ok(true)=>issue=Some("Full data clear is pending. Open Data Storage to finish this launcher; other launchers are unchanged.".into()),
                            Err(error)=>issue=Some(format!("Launcher cleanup needs attention: {error}")),
                        }
                    }
                }
                let state = if self.running[index].is_some() {
                    match process.as_ref() {
                        Some(p) if p.responding => "RUNNING",
                        Some(_) => "NOT_RESPONDING",
                        // This owned process is still alive but has no usable game HWND.
                        // NOT_RESPONDING prevents an unsafe second Launch in the UI;
                        // ATTENTION without a process is reserved for stopped profiles.
                        None if issue.is_some() => "NOT_RESPONDING",
                        None => "STARTING",
                    }
                } else if issue.is_some() || self.stores[index].is_none() {
                    "ATTENTION"
                } else {
                    "STOPPED"
                };
                ProfileSession {
                    slot: index as u32 + 1,
                    state: state.into(),
                    process,
                    issue,
                }
            })
            .collect();
        SessionSnapshot {
            schema: 1,
            profiles,
            storage: None,
            settings: None,
        }
    }

    fn close_all(&mut self) -> io::Result<()> {
        self.exiting=true;
        self.reap();
        let snapshot=self.snapshot();
        let mut issues=Vec::new();
        let mut confirmed = [false; 5];
        let mut progress: [NormalCloseProgress; 5] = std::array::from_fn(|_| NormalCloseProgress::default());
        for profile in snapshot.profiles {
            let index=profile.slot as usize-1;
            let Some(started)=&self.running[index] else {continue;};
            let child=&started.children[0];
            let outcome=(||{
                child.identity()?;
                let Some(target)=profile.process else { if confirm_owned_exit(child)? {confirmed[index]=true;} return Ok(()); };
                if target.pid!=child.pid || target.created_file_time!=child.created.to_string(){return Err(invalid("Game identity changed"));}
                if confirm_education_exit(&target)? {confirmed[index]=true;return Ok(());}
                progress[index].request_window(&target)?;
                Ok(())
            })();
            if let Err(error)=outcome {
                // A normal exit can race the identity/window checks; reap it on the next pass.
                if child.exit_code_after_exit().is_err() {
                    issues.push(format!("Minecraft - {}: {error}",profile.slot));
                }
            }
        }
        let deadline=std::time::Instant::now()+Duration::from_secs(20);
        loop {
            self.reap();
            if self.running.iter().all(Option::is_none){return Ok(());}
            if !issues.is_empty(){self.exiting=false;return Err(io::Error::other(issues.join("; ")));}
            for index in 0..5 {
                if confirmed[index] { continue; }
                let Some(started)=&self.running[index] else {continue;};
                // The main HWND can disappear while its owned exit dialog remains.
                let child=&started.children[0];
                let result=confirm_owned_exit(child).and_then(|confirmed| {
                    if !confirmed {
                        if let Some(target)=window(child, false) {
                            progress[index].request_window(&target)?;
                        }
                    }
                    Ok(confirmed)
                });
                match result {
                    Ok(value)=>confirmed[index]=value,
                    Err(error)=>{
                        if child.exit_code_after_exit().is_err() {self.exiting=false;return Err(error);}
                    }
                }
            }
            if std::time::Instant::now()>=deadline {
                let pending = (0..5).filter_map(|index| {
                    let started = self.running[index].as_ref()?;
                    let evidence = progress[index].describe_pending(&started.children[0], confirmed[index]);
                    self.issues[index] = Some(format!("Normal exit pending: {evidence}"));
                    Some(format!("Minecraft - {}: {evidence}", index + 1))
                }).collect::<Vec<_>>();
                self.exiting=false;
                return Err(io::Error::new(io::ErrorKind::TimedOut,format!(
                    "Normal Minecraft exit is still pending. {}. Finish a remaining game prompt normally and retry; no game was force-terminated.",
                    pending.join("; ")
                )));
            }
            std::thread::sleep(Duration::from_millis(50));
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
        let observed = super::super::inspect_native_workspace()?.processes;
        (1..=5)
            .map(|slot| {
                let store = self.store(slot)?;
                let path = directory_path(&store.private)?;
                let mut result = ProfileStorage {
                    slot,
                    bytes: 0,
                    files: 0,
                    partial: false,
                    disk: profile_storage::disk(&store.private).ok(),
                    cache: {
                        let admission = self.data_admission(slot, &observed);
                        let mut cache = profile_storage::inspect(&store.private, admission.is_ok());
                        if !cache.partial {
                            if let Err(error) = admission { cache.issue = Some(error.to_string()); }
                        }
                        cache
                    },
                    game_settings: Some(game_preset::inspect_settings(&store.private, self.data_admission(slot, &observed))),
                    full_clear: Some(product_profiles::clear::inspect(store,self.data_admission(slot,&observed))),
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

    fn clear_cache(&mut self, slot: u32) -> io::Result<()> {
        self.reap();
        let observed = super::super::inspect_native_workspace()?.processes;
        self.data_admission(slot, &observed)?;
        // The sole session actor serializes admission, exact-directory deletion and launch.
        profile_storage::clear(&self.store(slot)?.private, true)
    }

    fn data_admission(&self, slot: u32, observed: &[OwnedProcess]) -> io::Result<()> {
        self.store(slot)?;
        if self.exiting { return Err(invalid("Minecraft sessions are closing; wait before changing launcher data")); }
        let owned = self.running.iter().flatten().map(|started| {
            let child = &started.children[0];
            child.identity()?;
            Ok((child.pid, child.created))
        }).collect::<io::Result<Vec<_>>>()?;
        require_data_idle(self.running[slot as usize - 1].is_none(), &owned, observed)
    }

    fn clear_data(&mut self,slot:u32,token:&str)->io::Result<()> {
        self.reap();
        let observed=super::super::inspect_native_workspace()?.processes;
        self.data_admission(slot,&observed)?;
        let index=slot as usize-1;
        let store=self.store(slot)?;
        if product_profiles::clear::confirmation(store)?!=token {
            return Err(invalid("Launcher data changed after confirmation; review it again"));
        }
        let logical=store.logical.try_clone()?;
        let root=product_profiles::clear::location(store)?;
        let store=self.stores[index].take().ok_or_else(||invalid("Inactive profile storage is unavailable"))?;
        let result=product_profiles::clear::run(store,token);
        match product_profiles::clear::reopen(logical,&root,slot) {
            Ok(store)=>self.stores[index]=Some(store),
            Err(error)=>{self.issues[index]=Some(format!("Full clear needs recovery: {error}"));return Err(error);}
        }
        let store=self.stores[index].as_ref().unwrap();
        let pending=product_profiles::clear::pending(store)?;
        if !pending && (result.is_ok() || product_profiles::clear::confirmation(store)?!=token) {
            self.issues[index]=None;self.windows[index]=None;
            return Ok(());
        }
        result.map_err(|error|io::Error::new(error.kind(),format!("Full clear did not finish. Refresh Data Storage to review its recovery status. {error}")))
    }

    fn settings(&mut self, apply: bool) -> io::Result<SettingsReport> {
        self.reap();
        let idle = self.running.iter().all(Option::is_none)
            && self.stores.iter().all(Option::is_some)
            && super::super::inspect_native_workspace()?.processes.is_empty();
        let mut report = SettingsReport::policy(idle, None)?;
        let changed = if apply {
            require_preset_idle(idle)?;
            // Retain the original directory/registry pins; never open or seed replacement profiles.
            let stores = self.stores.iter().map(|store| store.as_ref().unwrap().duplicate())
                .collect::<io::Result<Vec<_>>>()?;
            Some(game_preset::apply(&stores)? as u32)
        } else { None };
        report.changed_profiles = changed;
        Ok(report)
    }
}

fn require_data_idle(selected_idle: bool, owned: &[(u32,u64)], observed: &[OwnedProcess]) -> io::Result<()> {
    if !selected_idle { return Err(invalid("Close this Minecraft session normally before changing its launcher data.")); }
    if observed.iter().any(|process| !owned.iter().any(|(pid,birth)|
        process.pid == *pid && process.created_file_time.parse::<u64>().ok() == Some(*birth))) {
        return Err(invalid("An unmanaged Minecraft session is open. Close it normally before changing launcher data; managed sessions in other launchers may stay open."));
    }
    Ok(())
}

fn require_preset_idle(idle: bool) -> io::Result<()> {
    if !idle { return Err(invalid("Close all Minecraft sessions normally before applying the graphics preset")); }
    Ok(())
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

// Only the reviewed Minecraft Education main game surface (OGLES) is
// a normal-close target. A native exit dialog (#32770) can outlive that surface
// and must be handled exclusively by the exact-dialog verifier below.
fn game_window_class(hwnd: windows_sys::Win32::Foundation::HWND) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::GetClassNameW;
    let mut class = [0u16; 32];
    let count = unsafe { GetClassNameW(hwnd, class.as_mut_ptr(), class.len() as i32) };
    count == 5 && class[..5] == ['O' as u16, 'G' as u16, 'L' as u16, 'E' as u16, 'S' as u16]
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
            && game_window_class(hwnd)
            && GetWindowThreadProcessId(hwnd, &mut pid) != 0
            && pid == target.pid
            && GetAncestor(hwnd, GA_ROOT) == hwnd
            && GetWindow(hwnd, GW_OWNER).is_null()
    }
}

#[derive(Default)]
struct NormalCloseProgress {
    requested_windows: Vec<String>,
    original_window_thread: Option<OwnedHandle>,
    blocked_by_modal: bool,
}

impl NormalCloseProgress {
    // A posted WM_CLOSE is only a request, not proof that the game consumed it.
    // Permit another request only for a different, freshly verified game HWND.
    fn request_window(&mut self, target: &OwnedProcess) -> io::Result<()> {
        if self.requested_windows.contains(&target.window_handle) {
            return Ok(());
        }
        if self.requested_windows.len() >= 4 {
            return Err(invalid("Minecraft replaced its main window repeatedly during normal close"));
        }
        let thread = if self.requested_windows.is_empty() {
            pin_window_thread(target)
        } else {
            None
        };
        match close_verified_window(target) {
            // A native modal prompt disables its game owner. Wait for the
            // verified prompt or a re-enabled main window; do not abort early.
            Ok(()) => self.blocked_by_modal = false,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                self.blocked_by_modal = true;
                return Ok(());
            }
            Err(error) => return Err(error),
        }
        if self.requested_windows.is_empty() {
            self.original_window_thread = thread;
        }
        self.requested_windows.push(target.window_handle.clone());
        Ok(())
    }

    fn original_thread_state(&self) -> &'static str {
        let Some(thread) = self.original_window_thread.as_ref() else {
            return "not captured";
        };
        match unsafe { WaitForSingleObject(thread.as_raw_handle() as _, 0) } {
            WAIT_OBJECT_0 => "exited",
            WAIT_TIMEOUT => "alive",
            _ => "unavailable",
        }
    }

    fn describe_pending(&self, child: &PendingEducationBootstrap, dialog_confirmed: bool) -> String {
        // Never inspect account, world, dialog text or arbitrary process memory.
        let main_window = if window(child, false).is_some() { "present" } else { "absent or ambiguous" };
        format!(
            "game process still running; unique verified main HWND: {main_window}; original window thread: {}; distinct WM_CLOSE requests: {}; disabled main window encountered: {}; exact exit confirmation submitted: {dialog_confirmed}",
            self.original_thread_state(),
            self.requested_windows.len(),
            self.blocked_by_modal,
        )
    }
}

// Read-only, original-window-thread evidence for a pinned managed game. Failure to
// obtain a thread handle must not prevent its ordinary WM_CLOSE/normal shutdown.
fn pin_window_thread(target: &OwnedProcess) -> Option<OwnedHandle> {
    use windows_sys::Win32::{
        System::Threading::{
            GetProcessIdOfThread, OpenThread, SYNCHRONIZATION_SYNCHRONIZE,
            THREAD_QUERY_LIMITED_INFORMATION,
        },
        UI::WindowsAndMessaging::GetWindowThreadProcessId,
    };
    let hwnd = usize::from_str_radix(target.window_handle.strip_prefix("0x")?, 16).ok()?;
    let mut pid = 0;
    let thread_id = unsafe { GetWindowThreadProcessId(hwnd as _, &mut pid) };
    if thread_id == 0 || pid != target.pid {
        return None;
    }
    let raw = unsafe {
        OpenThread(
            SYNCHRONIZATION_SYNCHRONIZE | THREAD_QUERY_LIMITED_INFORMATION,
            0,
            thread_id,
        )
    };
    if raw.is_null() {
        return None;
    }
    let thread = unsafe { OwnedHandle::from_raw_handle(raw as _) };
    if unsafe { GetProcessIdOfThread(thread.as_raw_handle() as _) } != target.pid {
        return None;
    }
    Some(thread)
}

fn close_verified_window(target:&OwnedProcess)->io::Result<()> {
    use windows_sys::Win32::{Foundation::FILETIME,System::Threading::*,UI::WindowsAndMessaging::*};
    let handle=unsafe{OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION|SYNCHRONIZATION_SYNCHRONIZE,0,target.pid)};
    if handle.is_null(){return Err(io::Error::last_os_error());}
    let handle=unsafe{OwnedHandle::from_raw_handle(handle as _)};
    let mut birth:FILETIME=unsafe{std::mem::zeroed()};
    let mut exited=birth;let mut kernel=birth;let mut user=birth;
    if unsafe{GetProcessTimes(handle.as_raw_handle() as _,&mut birth,&mut exited,&mut kernel,&mut user)}==0 {
        return Err(io::Error::last_os_error());
    }
    let created=(((birth.dwHighDateTime as u64)<<32)|birth.dwLowDateTime as u64).to_string();
    if created!=target.created_file_time || interactive_session(target.pid)?!=target.session_id || !valid_retained_window(target) {
        return Err(invalid("Game window identity changed; close it normally"));
    }
    let window=usize::from_str_radix(&target.window_handle[2..],16).map_err(|_|invalid("Invalid game window"))? as windows_sys::Win32::Foundation::HWND;
    if unsafe{GetWindowLongPtrW(window,GWL_STYLE)} & WS_DISABLED as isize != 0 {
        return Err(io::Error::new(io::ErrorKind::WouldBlock, "Minecraft game window has a modal dialog"));
    }
    if unsafe{PostMessageW(window,WM_CLOSE,0,0)}==0 {return Err(io::Error::last_os_error());}
    Ok(())
}

// Minecraft Education's standard English exit prompt is an unowned native dialog.
// Unknown/localized prompts remain for the user; never click arbitrary dialogs.
fn confirm_education_exit(target:&OwnedProcess)->io::Result<bool> {
    if !valid_retained_window(target) { return Ok(false); }
    confirm_exit_dialog(target.pid, || valid_retained_window(target))
}

fn confirm_owned_exit(child:&PendingEducationBootstrap)->io::Result<bool> {
    let (pid, birth)=child.identity()?;
    confirm_exit_dialog(pid, || child.identity().ok()==Some((pid,birth)))
}

fn confirm_exit_dialog(owned_pid:u32, verify:impl Fn()->bool)->io::Result<bool> {
    use windows_sys::Win32::{Foundation::HWND,UI::{Input::KeyboardAndMouse::IsWindowEnabled,WindowsAndMessaging::*}};
    if !verify() { return Ok(false); }
    struct Search { pid:u32, matches:Vec<isize> }
    unsafe fn text(window:HWND)->Option<String> {
        let mut buffer=[0u16;256];let mut count=0usize;
        if SendMessageTimeoutW(window,WM_GETTEXT,buffer.len(),buffer.as_mut_ptr() as isize,
            SMTO_ABORTIFHUNG|SMTO_BLOCK|SMTO_ERRORONEXIT,100,&mut count)==0 {return None;}
        Some(String::from_utf16_lossy(&buffer[..count.min(buffer.len()-1)]))
    }
    unsafe fn matches_exact_prompt(window:HWND, expected_pid:u32)->bool {
        let mut pid=0;
        if GetWindowThreadProcessId(window,&mut pid)==0 || pid!=expected_pid
            || IsWindowVisible(window)==0 || IsWindowEnabled(window)==0 {return false;}
        let mut class=[0u16;32];let length=GetClassNameW(window,class.as_mut_ptr(),class.len() as i32);
        if length<=0 || String::from_utf16_lossy(&class[..length as usize])!="#32770" {return false;}
        let mut title=[0u16;64];let length=GetWindowTextW(window,title.as_mut_ptr(),title.len() as i32);
        if length<=0 || String::from_utf16_lossy(&title[..length as usize])!="Minecraft Education" {return false;}
        let message=GetDlgItem(window,65535);let ok=GetDlgItem(window,1);let cancel=GetDlgItem(window,2);
        if message.is_null() || ok.is_null() || cancel.is_null() {return false;}
        text(message).as_deref()==Some("Exit Minecraft Education?")
            && text(ok).as_deref()==Some("OK")
            && text(cancel).as_deref()==Some("Cancel")
    }
    unsafe extern "system" fn find(window:HWND,parameter:isize)->i32 {
        let search=&mut *(parameter as *mut Search);
        if matches_exact_prompt(window,search.pid) {search.matches.push(window as isize);}
        1
    }
    let mut search=Search{pid:owned_pid,matches:Vec::new()};
    if unsafe {EnumWindows(Some(find),&mut search as *mut Search as isize)}==0 {return Err(io::Error::last_os_error());}
    if search.matches.len()!=1 || !verify() {return Ok(false);}
    let dialog=search.matches[0] as HWND;
    // HWNDs can be destroyed/reused after EnumWindows. Revalidate ALL dialog
    // properties immediately before posting IDOK, not merely its owning PID.
    if !unsafe {matches_exact_prompt(dialog,owned_pid)} || !verify() {return Ok(false);}
    if unsafe {PostMessageW(dialog,WM_COMMAND,1,0)}==0 {return Err(io::Error::last_os_error());}
    Ok(true)
}

fn window(child: &PendingEducationBootstrap, label: bool) -> Option<OwnedProcess> {
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
        if pid == selection.pid
            && game_window_class(hwnd)
            && super::super::verify_live_client_02_window(hwnd, pid).is_ok() {
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
    if label {
        let _ = label_slot_window(selected.hwnd, child.pid, slot);
    }
    Some(OwnedProcess {
        pid: child.pid,
        created_file_time: child.created.to_string(),
        session_id: interactive_session(child.pid).ok()?,
        window_handle: format!("0x{:x}", selected.hwnd as usize),
        responding: true,
    })
}

fn sync_audio(sessions:&Sessions,audio:&mut Option<super::session_audio::Audio>,mode:crate::engine::manager_settings::BackgroundAudio)->io::Result<()> {
    use crate::engine::manager_settings::BackgroundAudio;
    if mode==BackgroundAudio::Keep {
        if let Some(current)=audio.as_mut(){current.restore()?;}
        *audio=None;return Ok(());
    }
    let targets=sessions.running.iter().flatten().filter_map(|s|s.children[0].identity().ok()).collect::<Vec<_>>();
    if targets.is_empty(){if let Some(current)=audio.as_mut(){current.restore()?;}*audio=None;return Ok(());}
    if audio.is_none(){*audio=Some(super::session_audio::Audio::new()?);}
    audio.as_mut().unwrap().sync(&targets,super::session_audio::foreground(),|pid,born|sessions.running.iter().flatten().any(|s|s.children[0].identity().ok()==Some((pid,born))))
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
    let mut audio=None;
    let mut audio_mode=crate::engine::manager_settings::load_audio()?;
    let mut audio_issue;
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
        audio_issue=sync_audio(&sessions,&mut audio,audio_mode).err().map(|e|e.to_string());
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
            let mut settings = None;
            match envelope.request {
                SessionRequest::Inspect => {}
                SessionRequest::InspectSettings => {
                    settings = Some(sessions.settings(false).map_err(|error| error.to_string())?);
                }
                SessionRequest::ApplyPreset => {
                    settings = Some(sessions.settings(true).map_err(|error| error.to_string())?);
                }
                SessionRequest::ApplyGraphicsPreset { slot, preset } => {
                    let report = sessions.settings(false).map_err(|error| error.to_string())?;
                    require_preset_idle(report.can_apply_preset).map_err(|error| error.to_string())?;
                    let mut values = crate::engine::manager_settings::load_graphics().map_err(|error| error.to_string())?;
                    let changed = game_preset::apply_selected(sessions.store(slot).map_err(|error| error.to_string())?, preset).map_err(|error| error.to_string())?;
                    values[slot as usize - 1] = preset;
                    crate::engine::manager_settings::save_graphics(values).map_err(|error| error.to_string())?;
                    let mut report = sessions.settings(false).map_err(|error| error.to_string())?;
                    report.changed_profiles = Some(changed as u32);
                    settings = Some(report);
                }
                SessionRequest::SavePerformance {active_fps,background_fps} => {
                    crate::engine::manager_settings::save_performance(crate::engine::manager_settings::PerformanceSettings {active_fps,background_fps})
                        .map_err(|error|error.to_string())?;
                    settings=Some(sessions.settings(false).map_err(|error|error.to_string())?);
                }
                SessionRequest::SaveAudio {mode}=>{
                    // Apply before committing the preference; failed restore stays retryable.
                    sync_audio(&sessions,&mut audio,mode).map_err(|e|e.to_string())?;
                    crate::engine::manager_settings::save_audio(mode).map_err(|e|e.to_string())?;
                    audio_mode=mode;audio_issue=None;
                    settings=Some(sessions.settings(false).map_err(|e|e.to_string())?);
                }
                SessionRequest::Launch { slot } => {
                    sessions.launch(slot).map_err(|error| error.to_string())?
                }
                SessionRequest::CloseAll => sessions.close_all().map_err(|error|error.to_string())?,
                SessionRequest::InspectStorage => {
                    storage = Some(sessions.storage().map_err(|error| error.to_string())?)
                }
                SessionRequest::ClearProfileCache { slot } => {
                    sessions
                        .clear_cache(slot)
                        .map_err(|error| error.to_string())?;
                    storage = Some(sessions.storage().map_err(|error| error.to_string())?);
                }
                SessionRequest::ResetGameSettings { slot } | SessionRequest::RestoreGameSettings { slot } => {
                    sessions.reap();
                    let observed=super::super::inspect_native_workspace().map_err(|error|error.to_string())?.processes;
                    sessions.data_admission(slot,&observed).map_err(|error|error.to_string())?;
                    let store=sessions.store(slot).map_err(|error|error.to_string())?;
                    if matches!(envelope.request,SessionRequest::ResetGameSettings {..}) {
                        game_preset::reset_settings(&store.private)
                    } else {
                        game_preset::restore_settings(&store.private)
                    }.map_err(|error|error.to_string())?;
                    storage=Some(sessions.storage().map_err(|error|error.to_string())?);
                }
                SessionRequest::ClearProfileData {slot,confirmation_token,confirmation_name}=>{
                    if confirmation_name!=format!("Minecraft - {slot}") {return Err("Launcher confirmation does not match".into());}
                    sessions.clear_data(slot,&confirmation_token).map_err(|error|error.to_string())?;
                    storage=Some(sessions.storage().map_err(|error|error.to_string())?);
                }
                SessionRequest::OpenProfileFolder { slot } => sessions
                    .open_folder(slot)
                    .map_err(|error| error.to_string())?,
            }
            let mut snapshot = sessions.snapshot();
            if let Some(report)=settings.as_mut(){report.audio_issue=audio_issue.clone();}
            snapshot.storage = storage;
            snapshot.settings = settings;
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
    // Private synthetic OGLES class, not Minecraft. Shared registration is
    // initialized once because Rust test cases can execute concurrently.
    fn fixture_game_window() -> windows_sys::Win32::Foundation::HWND {
        use windows_sys::Win32::{
            System::LibraryLoader::GetModuleHandleW,
            UI::WindowsAndMessaging::{CreateWindowExW, DefWindowProcW, RegisterClassW, WNDCLASSW, WS_OVERLAPPEDWINDOW},
        };
        static REGISTERED: std::sync::OnceLock<()> = std::sync::OnceLock::new();
        REGISTERED.get_or_init(|| unsafe {
            let mut class: WNDCLASSW = std::mem::zeroed();
            class.hInstance = GetModuleHandleW(std::ptr::null());
            class.lpszClassName = windows_sys::w!("OGLES");
            class.lpfnWndProc = Some(DefWindowProcW);
            assert_ne!(RegisterClassW(&class), 0);
        });
        unsafe {
            CreateWindowExW(0, windows_sys::w!("OGLES"), windows_sys::w!("Fixture"),
                WS_OVERLAPPEDWINDOW, 0, 0, 200, 150,
                std::ptr::null_mut(), std::ptr::null_mut(),
                GetModuleHandleW(std::ptr::null()), std::ptr::null())
        }
    }

    #[test]
    fn stopped_profile_cache_allows_other_owned_games_but_rejects_unknown_or_reused_processes() {
        let other = OwnedProcess { pid:42, created_file_time:"123456".into(), session_id:1,
            window_handle:"0x1234".into(), responding:true };
        assert!(require_data_idle(true, &[(42,123456)], std::slice::from_ref(&other)).is_ok());
        assert!(require_data_idle(false, &[(42,123456)], std::slice::from_ref(&other)).is_err());
        assert!(require_data_idle(true, &[], std::slice::from_ref(&other)).is_err());
        assert!(require_data_idle(true, &[(42,123455)], std::slice::from_ref(&other)).is_err());
        assert!(require_data_idle(true, &[(43,123456)], std::slice::from_ref(&other)).is_err());
        assert!(require_data_idle(true, &[], &[]).is_ok());
    }

    #[test]
    fn full_exit_of_idle_sessions_does_not_prepare_profiles_and_denies_new_launches() {
        let mut sessions=Sessions{stores:(0..5).map(|_|None).collect(),running:(0..5).map(|_|None).collect(),issues:vec![None;5],windows:vec![None;5],path:String::new(),image:File::open(std::env::current_exe().unwrap()).unwrap(),exiting:false};
        sessions.close_all().unwrap();
        assert!(sessions.exiting);
        assert!(sessions.stores.iter().all(Option::is_none));
        assert!(sessions.launch(1).unwrap_err().to_string().contains("closing"));
    }

    #[test]
    fn normal_close_checks_exact_window_identity_and_leaves_another_window_alone() {
        use windows_sys::Win32::{Foundation::FILETIME,System::Threading::*,UI::WindowsAndMessaging::*};
        unsafe {
            let own=fixture_game_window();let other=fixture_game_window();assert!(!own.is_null()&&!other.is_null());
            let result=std::panic::catch_unwind(||{
                let mut birth:FILETIME=std::mem::zeroed();let mut exited=birth;let mut kernel=birth;let mut user=birth;
                assert_ne!(GetProcessTimes(GetCurrentProcess(),&mut birth,&mut exited,&mut kernel,&mut user),0);
                let target=OwnedProcess{pid:std::process::id(),created_file_time:(((birth.dwHighDateTime as u64)<<32)|birth.dwLowDateTime as u64).to_string(),session_id:interactive_session(std::process::id()).unwrap(),window_handle:format!("0x{:x}",own as usize),responding:true};
                let mut forged=target.clone();forged.created_file_time="1".into();assert!(close_verified_window(&forged).is_err());
                assert_ne!(IsWindow(own),0);
                let style=GetWindowLongPtrW(own,GWL_STYLE);SetWindowLongPtrW(own,GWL_STYLE,style|WS_DISABLED as isize);
                assert!(close_verified_window(&target).is_err());SetWindowLongPtrW(own,GWL_STYLE,style);
                close_verified_window(&target).unwrap();
                let mut message:MSG=std::mem::zeroed();
                assert_ne!(PeekMessageW(&mut message,own,WM_CLOSE,WM_CLOSE,PM_REMOVE),0);
                DispatchMessageW(&message);
                assert_eq!(IsWindow(own),0);
                assert_ne!(IsWindow(other),0);
            });
            if IsWindow(own)!=0{DestroyWindow(own);}DestroyWindow(other);
            if let Err(error)=result{std::panic::resume_unwind(error);}
        }
    }


    #[test]
    fn normal_close_posts_once_per_window_and_tracks_original_window_thread() {
        use windows_sys::Win32::{Foundation::FILETIME, System::Threading::*, UI::WindowsAndMessaging::*};
        unsafe {
            let first=fixture_game_window();let successor=fixture_game_window();
            assert!(!first.is_null()&&!successor.is_null()&&first!=successor);
            let result=std::panic::catch_unwind(||{
                let mut birth:FILETIME=std::mem::zeroed();let mut exited=birth;let mut kernel=birth;let mut user=birth;
                assert_ne!(GetProcessTimes(GetCurrentProcess(),&mut birth,&mut exited,&mut kernel,&mut user),0);
                let target=OwnedProcess{
                    pid:std::process::id(),
                    created_file_time:(((birth.dwHighDateTime as u64)<<32)|birth.dwLowDateTime as u64).to_string(),
                    session_id:interactive_session(std::process::id()).unwrap(),
                    window_handle:format!("0x{:x}",first as usize),
                    responding:true,
                };
                let mut progress=NormalCloseProgress::default();
                let style=GetWindowLongPtrW(first,GWL_STYLE);
                SetWindowLongPtrW(first,GWL_STYLE,style|WS_DISABLED as isize);
                progress.request_window(&target).unwrap();
                assert!(progress.blocked_by_modal);
                assert!(progress.requested_windows.is_empty());
                let mut none:MSG=std::mem::zeroed();
                assert_eq!(PeekMessageW(&mut none,first,WM_CLOSE,WM_CLOSE,PM_NOREMOVE),0);
                SetWindowLongPtrW(first,GWL_STYLE,style);
                progress.request_window(&target).unwrap();
                progress.request_window(&target).unwrap();
                assert!(!progress.blocked_by_modal);
                assert_eq!(progress.requested_windows.len(),1);
                assert_eq!(progress.original_thread_state(),"alive");
                let mut message:MSG=std::mem::zeroed();
                assert_ne!(PeekMessageW(&mut message,first,WM_CLOSE,WM_CLOSE,PM_REMOVE),0);
                assert_eq!(PeekMessageW(&mut message,first,WM_CLOSE,WM_CLOSE,PM_REMOVE),0);

                // A new, uniquely verified top-level HWND receives its own request.
                let new_target=OwnedProcess{window_handle:format!("0x{:x}",successor as usize),..target};
                progress.request_window(&new_target).unwrap();
                progress.request_window(&new_target).unwrap();
                assert_eq!(progress.requested_windows.len(),2);
                assert_ne!(PeekMessageW(&mut message,successor,WM_CLOSE,WM_CLOSE,PM_REMOVE),0);
                assert_eq!(PeekMessageW(&mut message,successor,WM_CLOSE,WM_CLOSE,PM_REMOVE),0);
                assert_ne!(IsWindow(successor),0);
            });
            if IsWindow(first)!=0{DestroyWindow(first);}
            if IsWindow(successor)!=0{DestroyWindow(successor);}
            if let Err(error)=result{std::panic::resume_unwind(error);}
        }
    }


    #[test]
    fn exit_dialog_is_not_accepted_as_an_ogles_game_window() {
        use windows_sys::Win32::{Foundation::FILETIME,System::Threading::*,UI::WindowsAndMessaging::*};
        unsafe {
            let dialog=CreateWindowExW(0,windows_sys::w!("#32770"),windows_sys::w!("Minecraft Education"),
                WS_POPUP|WS_VISIBLE,0,0,250,140,ptr::null_mut(),ptr::null_mut(),ptr::null_mut(),ptr::null());
            assert!(!dialog.is_null());
            let outcome=std::panic::catch_unwind(||{
                let mut birth:FILETIME=std::mem::zeroed();let mut exit=birth;let mut kernel=birth;let mut user=birth;
                assert_ne!(GetProcessTimes(GetCurrentProcess(),&mut birth,&mut exit,&mut kernel,&mut user),0);
                let target=OwnedProcess{
                    pid:std::process::id(),
                    created_file_time:(((birth.dwHighDateTime as u64)<<32)|birth.dwLowDateTime as u64).to_string(),
                    session_id:interactive_session(std::process::id()).unwrap(),
                    window_handle:format!("0x{:x}",dialog as usize),
                    responding:true,
                };
                assert!(!game_window_class(dialog));
                assert!(!valid_retained_window(&target));
                assert!(close_verified_window(&target).is_err());
                let mut posted:MSG=std::mem::zeroed();
                assert_eq!(PeekMessageW(&mut posted,dialog,WM_CLOSE,WM_CLOSE,PM_NOREMOVE),0);
                assert_ne!(IsWindow(dialog),0);
            });
            DestroyWindow(dialog);
            if let Err(error)=outcome{std::panic::resume_unwind(error);}
        }
    }

    #[test]
    fn exit_dialog_refuses_unverified_process_without_enumerating_windows() {
        assert!(!confirm_exit_dialog(std::process::id(), || false).unwrap());
    }

    #[test]
    fn exit_confirmation_is_limited_to_the_exact_owned_prompt() {
        use windows_sys::Win32::{Foundation::FILETIME,System::Threading::*,UI::WindowsAndMessaging::*};
        unsafe {
            let make=|class,caption,style,parent,id|CreateWindowExW(WS_EX_NOACTIVATE,class,caption,style,
                -30000,-30000,200,100,parent,id as _,std::ptr::null_mut(),std::ptr::null());
            let game=make(windows_sys::w!("STATIC"),windows_sys::w!("Fixture"),WS_OVERLAPPEDWINDOW,
                std::ptr::null_mut(),0usize);
            let dialog=make(windows_sys::w!("#32770"),windows_sys::w!("Minecraft Education"),WS_POPUP|WS_VISIBLE,
                std::ptr::null_mut(),0usize);
            assert!(!game.is_null()&&!dialog.is_null());
            let result=std::panic::catch_unwind(||{
                let message=make(windows_sys::w!("STATIC"),windows_sys::w!("Unknown question"),WS_CHILD|WS_VISIBLE,dialog,65535);
                let ok=make(windows_sys::w!("BUTTON"),windows_sys::w!("OK"),WS_CHILD|WS_VISIBLE,dialog,1);
                let cancel=make(windows_sys::w!("BUTTON"),windows_sys::w!("Cancel"),WS_CHILD|WS_VISIBLE,dialog,2);
                assert!(!message.is_null()&&!ok.is_null()&&!cancel.is_null());
                let mut birth:FILETIME=std::mem::zeroed();let mut exited=birth;let mut kernel=birth;let mut user=birth;
                assert_ne!(GetProcessTimes(GetCurrentProcess(),&mut birth,&mut exited,&mut kernel,&mut user),0);
                let target=OwnedProcess{pid:std::process::id(),created_file_time:(((birth.dwHighDateTime as u64)<<32)|birth.dwLowDateTime as u64).to_string(),session_id:interactive_session(std::process::id()).unwrap(),window_handle:format!("0x{:x}",game as usize),responding:true};
                assert!(!confirm_education_exit(&target).unwrap());
                assert_ne!(SetWindowTextW(message,windows_sys::w!("Exit Minecraft Education?")),0);
                assert!(confirm_education_exit(&target).unwrap());
                let mut received:MSG=std::mem::zeroed();
                assert_ne!(PeekMessageW(&mut received,dialog,WM_COMMAND,WM_COMMAND,PM_REMOVE),0);
                assert_eq!(received.wParam,1);
                let mut forged=target.clone();forged.pid=forged.pid.wrapping_add(1);
                assert!(!confirm_education_exit(&forged).unwrap());
            });
            DestroyWindow(dialog);DestroyWindow(game);
            if let Err(error)=result{std::panic::resume_unwind(error);}
        }
    }

    #[test]
    fn preset_request_refuses_uncertain_storage_before_writing_or_starting_any_game() {
        let mut sessions=Sessions {
            stores:(0..5).map(|_|None).collect(), running:(0..5).map(|_|None).collect(),
            issues:vec![None;5], windows:vec![None;5], path:String::new(),
            image:File::open(std::env::current_exe().unwrap()).unwrap(), exiting:false,
        };
        assert!(sessions.settings(true).unwrap_err().to_string().contains("Close all Minecraft"));
        assert!(sessions.running.iter().all(Option::is_none));
        assert!(sessions.snapshot().profiles.iter().all(|profile|profile.state=="ATTENTION"));
    }

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
            disk: None,
            cache: Default::default(),
            game_settings: None,
            full_clear: None,
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
