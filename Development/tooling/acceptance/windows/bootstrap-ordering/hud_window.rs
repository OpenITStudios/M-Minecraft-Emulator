//! Thread-local HUD window, DPI context, location hook and bounded message pump.
use std::{ffi::c_void,ptr,cell::Cell};
type Handle=*mut c_void;
type Proc=unsafe extern "system" fn(Handle,u32,usize,isize)->isize;
type Hook=unsafe extern "system" fn(Handle,u32,Handle,i32,i32,u32,u32);
type Notify=unsafe extern "C" fn(*mut c_void,Handle);
#[repr(C)]
struct Class { style:u32,proc:Option<Proc>,class_extra:i32,window_extra:i32,instance:Handle,
    icon:Handle,cursor:Handle,background:Handle,menu:*const u16,name:*const u16 }
#[repr(C)]
struct Point { x:i32,y:i32 }
#[repr(C)]
struct Message { window:Handle,message:u32,w:usize,l:isize,time:u32,point:Point,private:u32 }
#[link(name="user32")]
extern "system" {
    fn SetThreadDpiAwarenessContext(context:Handle)->Handle;
    fn RegisterClassW(class:*const Class)->u16;
    fn GetClassInfoW(instance:Handle,name:*const u16,class:*mut Class)->i32;
    fn DefWindowProcW(window:Handle,message:u32,w:usize,l:isize)->isize;
    fn SetWinEventHook(min:u32,max:u32,module:Handle,callback:Option<Hook>,process:u32,thread:u32,flags:u32)->Handle;
    fn UnhookWinEvent(hook:Handle)->i32;
    fn CreateWindowExW(ex:u32,class:*const u16,title:*const u16,style:u32,x:i32,y:i32,w:i32,h:i32,
        owner:Handle,menu:Handle,instance:Handle,param:Handle)->Handle;
    fn DestroyWindow(window:Handle)->i32;
    fn IsWindow(window:Handle)->i32;
    fn MsgWaitForMultipleObjectsEx(count:u32,handles:*const Handle,ms:u32,mask:u32,flags:u32)->u32;
    fn PeekMessageW(message:*mut Message,window:Handle,min:u32,max:u32,remove:u32)->i32;
    fn TranslateMessage(message:*const Message)->i32;
    fn DispatchMessageW(message:*const Message)->isize;
}
#[link(name="kernel32")]
extern "system" {
    fn GetModuleHandleW(name:*const u16)->Handle;
    fn GetCurrentProcessId()->u32;
    fn GetLastError()->u32;
}
const NAME:[u16;21]=[77,105,118,117,98,105,77,101,97,115,117,114,101,100,70,112,115,72,117,100,0];
thread_local! { static CALLBACK:Cell<Option<(*mut c_void,Notify)>>=const { Cell::new(None) }; }
unsafe extern "system" fn procedure(window:Handle,message:u32,w:usize,l:isize)->isize {
    match message { 0x84=>-1,0x21=>3,0x14=>1,_=>DefWindowProcW(window,message,w,l) }
}
unsafe extern "system" fn location(_:Handle,_:u32,window:Handle,object:i32,_:i32,_:u32,_:u32) {
    if object==0 { CALLBACK.with(|c| { if let Some((context,notify))=c.get() { notify(context,window); } }); }
}
pub struct WindowHost { previous:Handle,hook:Handle,instance:Handle,window:Handle }
impl Drop for WindowHost {
    fn drop(&mut self) { unsafe {
        self.clear();
        if !self.hook.is_null() { UnhookWinEvent(self.hook); }
        CALLBACK.with(|c|c.set(None));
        if !self.previous.is_null() { SetThreadDpiAwarenessContext(self.previous); }
    } }
}
impl WindowHost {
    unsafe fn new(context:*mut c_void,notify:Notify)->Option<Self> {
        let previous=SetThreadDpiAwarenessContext(-4isize as Handle);
        if previous.is_null() { return None; }
        let mut host=Self { previous,hook:ptr::null_mut(),instance:GetModuleHandleW(ptr::null()),window:ptr::null_mut() };
        if host.instance.is_null() { return None; }
        let mut class:Class=std::mem::zeroed();
        class.proc=Some(procedure); class.instance=host.instance; class.name=NAME.as_ptr();
        if RegisterClassW(&class)==0 {
            let mut existing:Class=std::mem::zeroed();
            if GetLastError()!=1410 || GetClassInfoW(host.instance,NAME.as_ptr(),&mut existing)==0
                || existing.proc.map(|p|p as usize)!=Some(procedure as Proc as usize) { return None; }
        }
        CALLBACK.with(|c|c.set(Some((context,notify))));
        host.hook=SetWinEventHook(0x800b,0x800b,ptr::null_mut(),Some(location),GetCurrentProcessId(),0,0);
        if host.hook.is_null() { return None; }
        Some(host)
    }
    unsafe fn clear(&mut self) {
        if !self.window.is_null() && IsWindow(self.window)!=0 { DestroyWindow(self.window); }
        self.window=ptr::null_mut();
    }
}
#[no_mangle]
pub unsafe extern "C" fn lazytester_hud_window_host_create(context:*mut c_void,notify:Notify)->*mut WindowHost {
    WindowHost::new(context,notify).map_or(ptr::null_mut(),|h|Box::into_raw(Box::new(h)))
}
#[no_mangle]
pub unsafe extern "C" fn lazytester_hud_window_host_destroy(host:*mut WindowHost) {
    if !host.is_null() { drop(Box::from_raw(host)); }
}
#[no_mangle]
pub unsafe extern "C" fn lazytester_hud_window_clear(host:*mut WindowHost) {
    if let Some(h)=host.as_mut() { h.clear(); }
}
#[no_mangle]
pub unsafe extern "C" fn lazytester_hud_window_get(host:*mut WindowHost,owner:Handle,x:i32,y:i32)->Handle {
    let Some(h)=host.as_mut() else { return ptr::null_mut(); };
    if h.window.is_null() {
        h.window=CreateWindowExW(0x080800a0,NAME.as_ptr(),[0u16].as_ptr(),0x80000000,x,y,0,0,
            owner,ptr::null_mut(),h.instance,ptr::null_mut());
    }
    h.window
}
// 0 = continue, 1 = stop/quit, 2 = failure. Keep 250ms wake and 64-message bound.
#[no_mangle]
pub unsafe extern "C" fn lazytester_hud_window_wait(stop:Handle,changed:Handle)->u32 {
    let handles=[stop,changed];
    match MsgWaitForMultipleObjectsEx(2,handles.as_ptr(),250,0x04ff,4) {
        0=>return 1,u32::MAX=>return 2,_=>{}
    }
    let mut message:Message=std::mem::zeroed();
    for _ in 0..64 {
        if PeekMessageW(&mut message,ptr::null_mut(),0,0,1)==0 { break; }
        if message.message==0x12 { return 1; }
        TranslateMessage(&message); DispatchMessageW(&message);
    }
    0
}
