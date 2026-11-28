//! Original-child window observation and bounded normal-close requests.
//! Process lifetime and exit acknowledgement remain with Sessions.
use super::super::{interactive_session, invalid, window_caption, PendingEducationBootstrap};
use crate::engine::process_identity::OwnedProcess;
use std::{io, os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle}, ptr};
use windows_sys::Win32::{Foundation::{WAIT_OBJECT_0, WAIT_TIMEOUT}, System::Threading::WaitForSingleObject};

// Only the reviewed Minecraft Education main game surface (OGLES) is
// a normal-close target. A native exit dialog (#32770) can outlive that surface
// and must be handled exclusively by the exact-dialog verifier below.
pub(super) fn game_window_class(hwnd: windows_sys::Win32::Foundation::HWND) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::GetClassNameW;
    let mut class = [0u16; 32];
    let count = unsafe { GetClassNameW(hwnd, class.as_mut_ptr(), class.len() as i32) };
    count == 5 && class[..5] == ['O' as u16, 'G' as u16, 'L' as u16, 'E' as u16, 'S' as u16]
}

// Return the same HWND whose native ownership was verified. This is a
// per-call observation, not a retained authorization or cached OS handle.
pub(super) fn verified_retained_hwnd(target: &OwnedProcess) -> Option<windows_sys::Win32::Foundation::HWND> {
    use windows_sys::Win32::UI::WindowsAndMessaging::*;
    let number = usize::from_str_radix(target.window_handle.strip_prefix("0x")?, 16).ok()?;
    let hwnd = number as windows_sys::Win32::Foundation::HWND;
    let mut pid = 0;
    let valid = unsafe {
        IsWindow(hwnd) != 0
            && game_window_class(hwnd)
            && GetWindowThreadProcessId(hwnd, &mut pid) != 0
            && pid == target.pid
            && GetAncestor(hwnd, GA_ROOT) == hwnd
            && GetWindow(hwnd, GW_OWNER).is_null()
    };
    valid.then_some(hwnd)
}

pub(super) fn valid_retained_window(target: &OwnedProcess) -> bool {
    verified_retained_hwnd(target).is_some()
}

pub(super) struct RequestedWindow {
    pub(super) handle: String,
    pub(super) thread_id: u32,
    // Retain thread identity while this bounded close attempt runs. If that
    // thread exits, Windows cannot recycle its thread ID until the retained
    // handle is released. No game process is owned or terminated by this pin.
    pub(super) _thread_pin: Option<OwnedHandle>,
}

#[derive(Default)]
pub(super) struct NormalCloseProgress {
    pub(super) requested_windows: Vec<RequestedWindow>,
    // Bounded once-per-exact-dialog (HWND, owning thread). Posting IDOK is not
    // evidence that the dialog was processed or the game process exited.
    pub(super) submitted_dialogs: Vec<(usize, u32)>,
    pub(super) blocked_by_modal: bool,
}

impl NormalCloseProgress {
    pub(super) fn already_requested(&self, handle:&str, thread_id:u32) -> bool {
        self.requested_windows.iter().any(|request|
            request.handle==handle && request.thread_id==thread_id)
    }

    // A posted WM_CLOSE is only a request, not proof that the game consumed it.
    // A new verified HWND on a different owner thread is not the same lifetime
    // merely because the numeric HWND was reused.
    pub(super) fn request_window(&mut self, target: &OwnedProcess) -> io::Result<()> {
        use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
        let window=target.window_handle.strip_prefix("0x")
            .and_then(|value|usize::from_str_radix(value,16).ok())
            .ok_or_else(|| invalid("Invalid game window"))? as windows_sys::Win32::Foundation::HWND;
        let mut pid=0;
        let thread_id=unsafe {GetWindowThreadProcessId(window,&mut pid)};
        if thread_id==0 || pid!=target.pid || !valid_retained_window(target) {
            // A changing HWND does not authorize targeting a replacement or
            // failing the whole close. The same pinned child is reobserved.
            return Ok(());
        }
        if self.already_requested(&target.window_handle,thread_id) {
            return Ok(());
        }
        if self.requested_windows.len() >= 4 {
            return Err(invalid("Minecraft replaced its main window repeatedly during normal close"));
        }
        let thread_pin=pin_window_thread(target).filter(|handle|unsafe {
            windows_sys::Win32::System::Threading::GetThreadId(handle.as_raw_handle() as _)==thread_id
        });
        match close_verified_window(target) {
            // A native modal prompt disables its game owner. Wait for the
            // verified prompt or a re-enabled main window; do not abort early.
            Ok(()) => self.blocked_by_modal = false,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                self.blocked_by_modal = true;
                return Ok(());
            }
            // A destroyed/replaced HWND between enumeration and PostMessageW
            // is an observation race. The pinned child, not this HWND, remains
            // the process authority. A fresh lookup can find its successor.
            Err(_) if !valid_retained_window(target) => return Ok(()),
            Err(error) => return Err(error),
        }
        self.requested_windows.push(RequestedWindow {
            handle:target.window_handle.clone(),
            thread_id,
            _thread_pin:thread_pin,
        });
        Ok(())
    }

    pub(super) fn submit_exit_dialog(&mut self, child: &PendingEducationBootstrap) -> io::Result<bool> {
        confirm_owned_exit_once(child, &mut self.submitted_dialogs)
    }

    pub(super) fn phase(&self, game_window_present: bool) -> &'static str {
        if !self.submitted_dialogs.is_empty() { "DIALOG_CONFIRMATION_SENT" }
        else if !self.requested_windows.is_empty() { "CLOSE_REQUESTED" }
        else if self.blocked_by_modal { "WAITING_FOR_MODAL" }
        else if game_window_present { "WINDOW_AVAILABLE" }
        else { "WAITING_FOR_WINDOW" }
    }

    pub(super) fn original_thread_state(&self) -> &'static str {
        let Some(thread) = self.requested_windows.first().and_then(|value|value._thread_pin.as_ref()) else {
            return "not captured";
        };
        match unsafe { WaitForSingleObject(thread.as_raw_handle() as _, 0) } {
            WAIT_OBJECT_0 => "exited",
            WAIT_TIMEOUT => "alive",
            _ => "unavailable",
        }
    }

    pub(super) fn describe_pending(&self, child: &PendingEducationBootstrap) -> String {
        // Never inspect account, world, dialog text or arbitrary process memory.
        let visible=window(child, false).is_some();
        let hidden=!visible && hidden_window_for_close(child).is_some();
        let game_window_present=visible||hidden;
        let main_window=if visible { "visible" } else if hidden { "hidden but owned" }
            else { "absent or ambiguous" };
        format!(
            "game process still running; close phase: {}; unique verified main HWND: {main_window}; original window thread: {}; distinct WM_CLOSE requests: {}; disabled main window encountered: {}; distinct exact exit-dialog commands posted: {}",
            self.phase(game_window_present),
            self.original_thread_state(),
            self.requested_windows.len(),
            self.blocked_by_modal,
            self.submitted_dialogs.len(),
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

pub(super) fn close_verified_window(target:&OwnedProcess)->io::Result<()> {
    use windows_sys::Win32::{System::Threading::*,UI::WindowsAndMessaging::*};
    let handle=unsafe{OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION|SYNCHRONIZATION_SYNCHRONIZE,0,target.pid)};
    if handle.is_null(){return Err(io::Error::last_os_error());}
    let handle=unsafe{OwnedHandle::from_raw_handle(handle as _)};
    let created = crate::engine::windows_process_time::creation_filetime(handle.as_raw_handle() as _)?.to_string();
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
#[cfg(test)]
pub(super) fn confirm_education_exit(target:&OwnedProcess)->io::Result<bool> {
    if !valid_retained_window(target) { return Ok(false); }
    confirm_exit_dialog(target.pid, || valid_retained_window(target))
}

fn confirm_owned_exit_once(child:&PendingEducationBootstrap, sent:&mut Vec<(usize,u32)>)->io::Result<bool> {
    let (pid, birth)=child.identity()?;
    confirm_exit_dialog_once(pid, || child.identity().ok()==Some((pid,birth)), sent)
}

// Wrapper retained for the existing exact-prompt fixture. Production CloseAll
// owns one bounded 'sent' set for each managed child throughout its attempt.
#[cfg(test)]
pub(super) fn confirm_exit_dialog(owned_pid:u32, verify:impl Fn()->bool)->io::Result<bool> {
    confirm_exit_dialog_once(owned_pid,verify,&mut Vec::new())
}

pub(super) fn confirm_exit_dialog_once(owned_pid:u32, verify:impl Fn()->bool, sent:&mut Vec<(usize,u32)>)->io::Result<bool> {
    use windows_sys::Win32::{Foundation::HWND,UI::{Input::KeyboardAndMouse::IsWindowEnabled,WindowsAndMessaging::*}};
    if !verify() { return Ok(false); }
    struct Search { pid:u32, first:HWND, count:u8 }
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
        if matches_exact_prompt(window,search.pid) {
            if search.count==0 {search.first=window;}
            // Only uniqueness matters; two or more matches are ambiguous.
            search.count=search.count.saturating_add(1).min(2);
        }
        1
    }
    let mut search=Search{pid:owned_pid,first:ptr::null_mut(),count:0};
    if unsafe {EnumWindows(Some(find),&mut search as *mut Search as isize)}==0 {return Err(io::Error::last_os_error());}
    if search.count!=1 || !verify() {return Ok(false);}
    let dialog=search.first;
    // HWNDs can be destroyed/reused after EnumWindows. Revalidate ALL dialog
    // properties immediately before posting IDOK, not merely its owning PID.
    if !unsafe {matches_exact_prompt(dialog,owned_pid)} || !verify() {return Ok(false);}
    let mut actual_pid=0;
    let thread=unsafe {GetWindowThreadProcessId(dialog,&mut actual_pid)};
    if thread==0 || actual_pid!=owned_pid {return Ok(false);}
    let identity=(dialog as usize,thread);
    if sent.contains(&identity) { return Ok(true); }
    if sent.len()>=4 {
        return Err(invalid("Minecraft opened too many distinct exit prompts during normal close"));
    }
    if unsafe {PostMessageW(dialog,WM_COMMAND,1,0)}==0 {return Err(io::Error::last_os_error());}
    sent.push(identity);
    Ok(true)
}

// Exactly the confirmed game's original top-level OGLES root can receive
// WM_CLOSE while hidden. Ordinary launcher selection is deliberately
// visibility-only. A modal #32770 prompt must NEVER be eligible here.
pub(super) fn hidden_close_candidate(hwnd:windows_sys::Win32::Foundation::HWND,expected_pid:u32)->bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::*;
    let mut actual_pid=0;
    unsafe {
        IsWindow(hwnd)!=0
            && GetWindowThreadProcessId(hwnd,&mut actual_pid)!=0
            && actual_pid==expected_pid
            && IsWindowVisible(hwnd)==0
            && game_window_class(hwnd)
            && GetAncestor(hwnd,GA_ROOT)==hwnd
            && GetWindow(hwnd,GW_OWNER).is_null()
    }
}

pub(super) fn hidden_window_for_close(child:&PendingEducationBootstrap)->Option<OwnedProcess> {
    use windows_sys::Win32::{Foundation::HWND,UI::WindowsAndMessaging::*};
    struct Found { pid:u32, total:u32, hidden:HWND }
    unsafe extern "system" fn select(hwnd:HWND,context:isize)->i32 {
        let found=&mut *(context as *mut Found);
        let mut pid=0;
        if GetWindowThreadProcessId(hwnd,&mut pid)!=0 && pid==found.pid
            && IsWindow(hwnd)!=0 && game_window_class(hwnd)
            && GetAncestor(hwnd,GA_ROOT)==hwnd && GetWindow(hwnd,GW_OWNER).is_null() {
            // Refuse any ambiguity, including an additional visible OGLES.
            found.total+=1;
            if IsWindowVisible(hwnd)==0 {found.hidden=hwnd;}
        }
        1
    }
    let (pid,birth)=child.identity().ok()?;
    let mut found=Found{pid,total:0,hidden:ptr::null_mut()};
    if unsafe{EnumWindows(Some(select),&mut found as *mut Found as isize)}==0
        || found.total!=1 || found.hidden.is_null() {return None;}
    if child.identity().ok()!=Some((pid,birth)) || !hidden_close_candidate(found.hidden,pid){
        return None;
    }
    let candidate=OwnedProcess{
        pid,
        created_file_time:birth.to_string(),
        session_id:interactive_session(pid).ok()?,
        window_handle:format!("0x{:x}",found.hidden as usize),
        responding:false, // Hidden is not equivalent to playable or visible.
    };
    valid_retained_window(&candidate).then_some(candidate)
}

pub(super) fn window(child: &PendingEducationBootstrap, label: bool) -> Option<OwnedProcess> {
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
            && super::super::super::verify_live_client_02_window(hwnd, pid).is_ok() {
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
        let _ = window_caption::label_slot_window(selected.hwnd, child.pid, slot);
    }
    let process=OwnedProcess {
        pid: child.pid,
        created_file_time: child.created.to_string(),
        session_id: interactive_session(child.pid).ok()?,
        window_handle: format!("0x{:x}", selected.hwnd as usize),
        // Enumeration only verifies a visible root. It does not establish
        // message-loop responsiveness; match the retained-HWND snapshot path.
        responding: unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::IsHungAppWindow(selected.hwnd)==0
        },
    };
    // Recheck immediately after observing/optionally labeling. A recycled
    // HWND or destroyed game surface is never published as a playable game.
    valid_retained_window(&process).then_some(process)
}
