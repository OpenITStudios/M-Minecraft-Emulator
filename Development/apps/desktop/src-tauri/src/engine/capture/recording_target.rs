//! Recording-only OS pin for a controller-admitted game. Never changes focus.
use crate::engine::{process_identity::OwnedProcess,windows_process_time};
use super::video_encoder::VideoSize;
use std::{io,os::windows::io::{AsRawHandle,FromRawHandle,OwnedHandle}};
use windows_sys::Win32::{
    Foundation::{HWND,RECT,WAIT_TIMEOUT},
    System::{RemoteDesktop::ProcessIdToSessionId,Threading::*},
    UI::WindowsAndMessaging::*,
};
pub(super) struct Target{identity:OwnedProcess,handle:OwnedHandle,hwnd:usize,thread:u32,pub size:VideoSize}
impl Target{
    pub fn pin(identity:OwnedProcess)->io::Result<Self>{
        let hwnd=identity.window_handle.strip_prefix("0x").and_then(|s|usize::from_str_radix(s,16).ok())
            .filter(|x|*x!=0).ok_or_else(changed)?;
        let raw=unsafe{OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION|SYNCHRONIZATION_SYNCHRONIZE,0,identity.pid)};
        if raw.is_null(){return Err(changed());}
        let handle=unsafe{OwnedHandle::from_raw_handle(raw as _)};
        let (thread,size)=window(hwnd as HWND,identity.pid)?;
        let t=Self{identity,handle,hwnd,thread,size:size.validate()?};
        t.verify()?;
        Ok(t)
    }
    pub fn hwnd(&self)->u64{self.hwnd as u64}
    pub fn verify(&self)->io::Result<()>{
        let process=self.handle.as_raw_handle() as _;
        let birth=self.identity.created_file_time.parse::<u64>().map_err(|_|changed())?;
        let mut game=0;
        let mut own=0;
        if unsafe{WaitForSingleObject(process,0)}!=WAIT_TIMEOUT||
            windows_process_time::creation_filetime(process)?!=birth||
            unsafe{ProcessIdToSessionId(self.identity.pid,&mut game)}==0||
            unsafe{ProcessIdToSessionId(std::process::id(),&mut own)}==0||
            game!=self.identity.session_id||game!=own{return Err(changed());}
        let (thread,size)=window(self.hwnd as HWND,self.identity.pid)?;
        if thread!=self.thread||size!=self.size{return Err(changed());}
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
