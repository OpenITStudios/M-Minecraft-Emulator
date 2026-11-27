//! Recording-only OS pin for a controller-admitted game. Never changes focus.
use crate::engine::{process_identity::OwnedProcess,windows_process_time};
use super::video_encoder::VideoSize;
use std::{io,os::windows::io::{AsRawHandle,FromRawHandle,OwnedHandle}};
use windows_sys::Win32::{
    Foundation::{HWND,RECT,WAIT_TIMEOUT},
    System::{RemoteDesktop::ProcessIdToSessionId,Threading::*},
    UI::WindowsAndMessaging::*,
};
pub(super) struct Target{pid:u32,handle:OwnedHandle,hwnd:usize,thread:u32,pub size:VideoSize}
impl Target{
    pub fn pin(identity:OwnedProcess)->io::Result<Self>{
        let hwnd=identity.window_handle.strip_prefix("0x").and_then(|s|usize::from_str_radix(s,16).ok())
            .filter(|x|*x!=0).ok_or_else(changed)?;
        let raw=unsafe{OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION|SYNCHRONIZATION_SYNCHRONIZE,0,identity.pid)};
        if raw.is_null(){return Err(changed());}
        let handle=unsafe{OwnedHandle::from_raw_handle(raw as _)};
        // Validate immutable process identity once against the retained OS
        // handle. The handle cannot adopt a later process with a reused PID.
        let birth=identity.created_file_time.parse::<u64>().map_err(|_|changed())?;
        let mut game_session=0;
        let mut own_session=0;
        if birth==0||windows_process_time::creation_filetime(raw)?!=birth||
            unsafe{ProcessIdToSessionId(identity.pid,&mut game_session)}==0||
            unsafe{ProcessIdToSessionId(std::process::id(),&mut own_session)}==0||
            game_session!=identity.session_id||game_session!=own_session||
            unsafe{WaitForSingleObject(raw,0)}!=WAIT_TIMEOUT{return Err(changed());}
        let (thread,size)=window(hwnd as HWND,identity.pid)?;
        let t=Self{pid:identity.pid,handle,hwnd,thread,size:size.validate()?};
        t.verify()?;
        Ok(t)
    }
    pub fn hwnd(&self)->u64{self.hwnd as u64}
    pub fn verify(&self)->io::Result<()>{
        let process=self.handle.as_raw_handle() as _;
        // Recheck live facts, not immutable birth/session metadata. Check
        // the original handle on both sides of HWND observation to reject
        // exit/PID reuse during the window checks.
        if unsafe{WaitForSingleObject(process,0)}!=WAIT_TIMEOUT{return Err(changed());}
        let (thread,size)=window(self.hwnd as HWND,self.pid)?;
        if thread!=self.thread||size!=self.size||
            unsafe{WaitForSingleObject(process,0)}!=WAIT_TIMEOUT{return Err(changed());}
        Ok(())
    }
}
fn changed()->io::Error{io::Error::other("Recording target closed, changed, resized or became unavailable")}
fn window(hwnd:HWND,pid:u32)->io::Result<(u32,VideoSize)>{
    let mut actual=0;
    let mut rect=RECT{left:0,top:0,right:0,bottom:0};
    let thread=unsafe{GetWindowThreadProcessId(hwnd,&mut actual)};
    if thread==0||actual!=pid||unsafe{IsWindow(hwnd)}==0||
        unsafe{GetAncestor(hwnd,GA_ROOT)}!=hwnd||!unsafe{GetWindow(hwnd,GW_OWNER)}.is_null()||
        unsafe{IsWindowVisible(hwnd)}==0||unsafe{IsIconic(hwnd)}!=0||
        unsafe{IsHungAppWindow(hwnd)}!=0||unsafe{GetClientRect(hwnd,&mut rect)}==0{
        return Err(changed());
    }
    let width=u32::try_from(i64::from(rect.right)-i64::from(rect.left)).map_err(|_|changed())?;
    let height=u32::try_from(i64::from(rect.bottom)-i64::from(rect.top)).map_err(|_|changed())?;
    Ok((thread,VideoSize{width,height}))
}
