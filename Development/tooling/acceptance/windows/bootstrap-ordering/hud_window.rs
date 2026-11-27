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
unsafe extern "system" fn location(_:Handle,_:u32,window:Handle,object:i32,child:i32,_:u32,_:u32) {
    if object==0 && child==0 { CALLBACK.with(|c| { if let Some((context,notify))=c.get() { notify(context,window); } }); }
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
pub(crate) unsafe fn lazytester_hud_window_host_create(context:*mut c_void,notify:Notify)->*mut WindowHost {
    WindowHost::new(context,notify).map_or(ptr::null_mut(),|h|Box::into_raw(Box::new(h)))
}
pub(crate) unsafe fn lazytester_hud_window_host_destroy(host:*mut WindowHost) {
    if !host.is_null() { drop(Box::from_raw(host)); }
}
pub(crate) unsafe fn lazytester_hud_window_clear(host:*mut WindowHost) {
    if let Some(h)=host.as_mut() { h.clear(); }
}
// 0 = continue, 1 = stop/quit, 2 = failure. Selection is only included
// while waiting for promotion. Keep a bounded reconciliation deadline for
// missed window/owner changes, and the existing 64-message pump bound.
pub(crate) unsafe fn lazytester_hud_window_wait(stop:Handle,changed:Handle,selection:Handle)->u32 {
    let handles=[stop,changed,selection];
    let (count,timeout)=if selection.is_null(){(2,250)}else{(3,1000)};
    match MsgWaitForMultipleObjectsEx(count,handles.as_ptr(),timeout,0x04ff,4) {
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

// Worker-local composition: retain one raster and redraw only on changed display state.
use super::hud_canvas::{Rect, Size};
use super::hud_canvas::windows::{Canvas, CanvasView, lazytester_hud_canvas_create,
    lazytester_hud_canvas_destroy, lazytester_hud_canvas_paint, lazytester_hud_physical_client};
use super::hud_canvas::lazytester_hud_placement;
#[repr(C)]
struct Blend { operation:u8, flags:u8, alpha:u8, format:u8 }
#[link(name="user32")]
extern "system" {
    fn GetDpiForWindow(window:Handle)->u32;
    fn IsWindowVisible(window:Handle)->i32;
    fn IsIconic(window:Handle)->i32;
    fn ShowWindow(window:Handle,command:i32)->i32;
    fn SetWindowPos(window:Handle,after:Handle,x:i32,y:i32,cx:i32,cy:i32,flags:u32)->i32;
    fn UpdateLayeredWindow(window:Handle,destination:Handle,position:*const Point,size:*const Size,
        source:Handle,origin:*const Point,key:u32,blend:*const Blend,flags:u32)->i32;
}
#[repr(C)]
pub struct Composition { pub window:Handle,pub bounds:Rect,pub dpi:u32,pub visible:u32,pub drawn:u32 }
pub struct Renderer { canvas:*mut Canvas, previous_dpi:u32, previous_rate:u32, previous_valid:bool, previous_bounds:Rect }
impl Drop for Renderer { fn drop(&mut self) { unsafe { lazytester_hud_canvas_destroy(self.canvas); } } }
pub(crate) unsafe fn lazytester_hud_renderer_create()->*mut Renderer {
    let canvas=lazytester_hud_canvas_create();
    if canvas.is_null() { return ptr::null_mut(); }
    Box::into_raw(Box::new(Renderer {canvas,previous_dpi:0,previous_rate:u32::MAX,previous_valid:false,previous_bounds:Rect::default()}))
}
pub(crate) unsafe fn lazytester_hud_renderer_destroy(renderer:*mut Renderer) {
    if !renderer.is_null() { drop(Box::from_raw(renderer)); }
}
pub(crate) unsafe fn lazytester_hud_renderer_reset(renderer:*mut Renderer) {
    if let Some(r)=renderer.as_mut() { r.previous_dpi=0;r.previous_bounds=Rect::default(); }
}
pub(crate) unsafe fn lazytester_hud_renderer_update(renderer:*mut Renderer,host:*mut WindowHost,
    target:Handle,bound:u32,admitted:u32,milli:u32,valid:u32,out:*mut Composition)->u32 {
    let (Some(r),Some(h),Some(out))=(renderer.as_mut(),host.as_mut(),out.as_mut()) else { return 0; };
    *out=Composition {window:ptr::null_mut(),bounds:Rect::default(),dpi:0,visible:0,drawn:0};
    let mut client=Rect::default();
    if bound==0 || lazytester_hud_physical_client(target,&mut client)==0 {
        if !h.window.is_null() && IsWindowVisible(h.window)!=0 { ShowWindow(h.window,0); }
        r.previous_dpi=0;
        return 1;
    }
    if h.window.is_null() {
        h.window=CreateWindowExW(0x080800a0,NAME.as_ptr(),[0u16].as_ptr(),0x80000000,
            client.left,client.top,0,0,target,ptr::null_mut(),h.instance,ptr::null_mut());
    }
    if h.window.is_null() { return 0; }
    let window=h.window;
    let dpi=GetDpiForWindow(window);
    let mut bounds=Rect::default();
    lazytester_hud_placement(&client,dpi,&mut bounds);
    let fits=bounds.right>bounds.left;
    let visible=fits && admitted!=0 && IsWindowVisible(target)!=0 && IsIconic(target)==0;
    // Hidden HUDs keep their HWND geometry current (including monitor DPI)
    // without drawing or uploading pixels. Invalidate the existing raster
    // checkpoint so the next visible frame is current.
    if !visible {
        if IsWindowVisible(window)!=0 { ShowWindow(window,0); }
        r.previous_dpi=0;
        if fits && bounds!=r.previous_bounds {
            if SetWindowPos(window,ptr::null_mut(),bounds.left,bounds.top,
                bounds.right-bounds.left,bounds.bottom-bounds.top,0x14)==0 { return 0; }
            r.previous_bounds=bounds;
        }
        out.window=window;out.bounds=bounds;out.dpi=dpi;
        return 1;
    }
    {
        let rate=((u64::from(milli)+50)/100) as u32;
        let redraw=dpi!=r.previous_dpi || (valid!=0)!=r.previous_valid || rate!=r.previous_rate;
        if redraw {
            let mut view:CanvasView=std::mem::zeroed();
            if lazytester_hud_canvas_paint(r.canvas,milli,valid,dpi,&mut view)==0 { return 0; }
            let position=Point{x:bounds.left,y:bounds.top};
            let origin=Point{x:0,y:0};
            let blend=Blend{operation:0,flags:0,alpha:240,format:1};
            if UpdateLayeredWindow(window,ptr::null_mut(),&position,&view.size,view.dc,&origin,0,&blend,2)==0 { return 0; }
            out.drawn=1;
        } else if bounds!=r.previous_bounds {
            if SetWindowPos(window,ptr::null_mut(),bounds.left,bounds.top,0,0,0x15)==0 { return 0; }
        }
        r.previous_dpi=dpi;r.previous_bounds=bounds;r.previous_rate=rate;r.previous_valid=valid!=0;
    }
    if IsWindowVisible(window)==0 { ShowWindow(window,4); }
    out.window=window;out.bounds=bounds;out.dpi=dpi;out.visible=1;1
}
