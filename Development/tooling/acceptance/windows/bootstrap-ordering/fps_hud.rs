//! One Rust-owned HUD session, one worker, one generation gate.
use std::{ffi::c_void,ptr,sync::{Arc,Mutex,MutexGuard},thread::{self,JoinHandle}};
use super::{fps_meter::FpsMeterState,hud_canvas::Rect,hud_window::*};
type Handle=*mut c_void;
const TIMEOUT:u32=258;
const SYNC:u32=0x00100000;
#[link(name="kernel32")]
extern "system" {
    fn CreateEventW(attributes:Handle,manual:i32,initial:i32,name:*const u16)->Handle;
    fn SetEvent(event:Handle)->i32;
    fn CloseHandle(handle:Handle)->i32;
    fn WaitForSingleObject(handle:Handle,ms:u32)->u32;
    fn GetCurrentProcess()->Handle;
    fn GetCurrentProcessId()->u32;
    fn DuplicateHandle(source_process:Handle,source:Handle,target_process:Handle,target:*mut Handle,access:u32,inherit:i32,options:u32)->i32;
    fn OpenThread(access:u32,inherit:i32,thread:u32)->Handle;
    fn GetProcessIdOfThread(thread:Handle)->u32;
    fn QueryPerformanceFrequency(value:*mut i64)->i32;
    fn QueryPerformanceCounter(value:*mut i64)->i32;
    fn GetTickCount64()->u64;
    fn GetModuleHandleExW(flags:u32,name:*const u16,module:*mut Handle)->i32;
}
#[link(name="user32")]
extern "system" {
    fn GetWindowThreadProcessId(window:Handle,pid:*mut u32)->u32;
    fn GetAncestor(window:Handle,flags:u32)->Handle;
    fn IsWindow(window:Handle)->i32;
}
// Owned kernel handles are stored as integer tokens, never dereferenced.
struct Owned(usize);
impl Owned {
    fn raw(&self)->Handle { self.0 as Handle }
    unsafe fn event(manual:bool)->Option<Self> {
        let h=CreateEventW(ptr::null_mut(),i32::from(manual),0,ptr::null());
        (!h.is_null()).then_some(Self(h as usize))
    }
    unsafe fn duplicate(h:Handle)->Option<Self> {
        let mut copy=ptr::null_mut();
        let process=GetCurrentProcess();
        if h.is_null() || DuplicateHandle(process,h,process,&mut copy,SYNC,0,0)==0 {None}
        else {Some(Self(copy as usize))}
    }
}
impl Drop for Owned { fn drop(&mut self) { if self.0!=0 { unsafe {CloseHandle(self.raw());} } } }
fn lock<T>(m:&Mutex<T>)->MutexGuard<'_,T> { m.lock().unwrap_or_else(|e|e.into_inner()) }
fn now()->i64 { let mut n=0; unsafe {if QueryPerformanceCounter(&mut n)==0 {0}else{n}} }
#[no_mangle]
pub unsafe extern "C" fn lazytester_hud_selection(managed:u32,selected:Handle,stop:Handle,controller:Handle)->u32 {
    u32::from(managed==0 || (!selected.is_null() && !stop.is_null() && !controller.is_null()
        && WaitForSingleObject(selected,0)==0 && WaitForSingleObject(stop,0)==TIMEOUT
        && WaitForSingleObject(controller,0)==TIMEOUT))
}
#[repr(C)]
#[derive(Clone,Copy,Default)]
pub struct HudSnapshot {
    accepted:u64,milli:u32,valid:u32,window:usize,bounds:Rect,dpi:u32,
    draws:u32,failures:u32,visible:u32,active:u32,
}
const _:()={
    use std::mem::{align_of,offset_of,size_of};
    assert!(size_of::<HudSnapshot>()==64 && align_of::<HudSnapshot>()==8);
    assert!(offset_of!(HudSnapshot,window)==16 && offset_of!(HudSnapshot,bounds)==24);
    assert!(offset_of!(HudSnapshot,dpi)==40 && offset_of!(HudSnapshot,active)==56);
};
struct Binding {
    owner:usize,thread:u32,life:Option<Owned>,idle:u64,generation:u64,
    finished:bool,display:HudSnapshot,meter:FpsMeterState,
}
struct Shared {
    state:Mutex<Binding>,stop:Owned,changed:Owned,policy:Option<[Owned;3]>,
}
pub struct Hud { shared:Arc<Shared>,worker:Mutex<Option<JoinHandle<()>>> }
impl Shared {
    fn admitted(&self)->bool {
        self.policy.as_ref().is_none_or(|p| unsafe {
            lazytester_hud_selection(1,p[0].raw(),p[1].raw(),p[2].raw())!=0
        })
    }
    fn signal(&self)->bool {unsafe {SetEvent(self.changed.raw())!=0}}
    fn unbind(&self) {
        let mut s=lock(&self.state);
        s.owner=0;s.thread=0;s.life=None;s.idle=unsafe{GetTickCount64()};
        s.generation=s.generation.wrapping_add(1);s.meter.reset();
        drop(s);self.signal();
    }
    fn fail(&self) {let mut s=lock(&self.state);s.display.failures=s.display.failures.saturating_add(1);}
}
unsafe extern "C" fn location(context:*mut c_void,window:Handle) {
    let s=&*(context as *const Shared);
    if lock(&s.state).owner==window as usize {s.signal();}
}
fn module_anchor(){}
unsafe fn valid_owner(owner:Handle,thread:u32)->bool {
    let mut pid=0;
    !owner.is_null() && thread!=0 && GetWindowThreadProcessId(owner,&mut pid)==thread && pid==GetCurrentProcessId()
}
struct Host(*mut WindowHost);
impl Drop for Host {fn drop(&mut self){unsafe{lazytester_hud_window_host_destroy(self.0);}}}
struct Draw(*mut Renderer);
impl Drop for Draw {fn drop(&mut self){unsafe{lazytester_hud_renderer_destroy(self.0);}}}
fn run(shared:Arc<Shared>) {
    {let mut s=lock(&shared.state);s.display.active=1;}
    let ok=unsafe{run_inner(&shared)};
    let mut s=lock(&shared.state);
    if !ok {s.display.failures=s.display.failures.saturating_add(1);}
    s.display.window=0;s.display.visible=0;s.display.active=0;s.finished=true;
}
unsafe fn run_inner(shared:&Shared)->bool {
    let host=Host(lazytester_hud_window_host_create((shared as *const Shared).cast_mut().cast(),location));
    if host.0.is_null(){return false;}
    let draw=Draw(lazytester_hud_renderer_create());
    if draw.0.is_null(){return false;}
    let mut previous_owner=0;
    let mut previous_generation=0;
    let mut window:Handle=ptr::null_mut();
    loop {
        let (owner,thread,generation,idle)={
            let mut s=lock(&shared.state);
            if s.life.as_ref().is_some_and(|h|WaitForSingleObject(h.raw(),0)==0) {
                s.finished=true;return true;
            }
            (s.owner,s.thread,s.generation,s.idle)
        };
        let target=owner as Handle;
        let bound=valid_owner(target,thread);
        let expired=owner==0 && idle!=0 && GetTickCount64().wrapping_sub(idle)>=1000;
        if (owner!=0 && !bound)||expired {
            let mut s=lock(&shared.state);
            if s.owner==owner && s.generation==generation && (!expired||s.idle==idle) {
                s.finished=true;return true;
            }
            continue;
        }
        if !bound||owner!=previous_owner||generation!=previous_generation||(!window.is_null()&&IsWindow(window)==0) {
            lazytester_hud_window_clear(host.0);window=ptr::null_mut();
            previous_owner=owner;previous_generation=generation;lazytester_hud_renderer_reset(draw.0);
        }
        let reading=lock(&shared.state).meter.reading(now());
        let mut composed:Composition=std::mem::zeroed();
        if lazytester_hud_renderer_update(draw.0,host.0,target,u32::from(bound),u32::from(shared.admitted()),
            reading.milli_fps,reading.valid,&mut composed)==0 {return false;}
        if !composed.window.is_null(){window=composed.window;}
        {
            let mut s=lock(&shared.state);
            if s.generation==generation {
                s.display.window=composed.window as usize;s.display.visible=composed.visible;
                if !composed.window.is_null(){s.display.bounds=composed.bounds;s.display.dpi=composed.dpi;}
                s.display.draws=s.display.draws.saturating_add(composed.drawn);
            }
        }
        match lazytester_hud_window_wait(shared.stop.raw(),shared.changed.raw()) {0=>{},1=>return true,_=>return false}
    }
}
impl Hud {
    unsafe fn new(selected:Handle,stop:Handle,controller:Handle)->Option<Self> {
        let mut frequency=0;
        if QueryPerformanceFrequency(&mut frequency)==0 || !(1..=1_000_000_000_000).contains(&frequency){return None;}
        // Legacy v2/v3 has stop/controller but no selection event: HUD stays hidden.
        let policy=if selected.is_null()&&stop.is_null()&&controller.is_null(){None}
            else if selected.is_null() && !stop.is_null() && !controller.is_null() {
                Some([Owned::event(true)?,Owned::duplicate(stop)?,Owned::duplicate(controller)?])
            }
            else {Some([Owned::duplicate(selected)?,Owned::duplicate(stop)?,Owned::duplicate(controller)?])};
        Some(Self{shared:Arc::new(Shared{state:Mutex::new(Binding{owner:0,thread:0,life:None,idle:0,generation:0,
            finished:false,display:HudSnapshot::default(),meter:FpsMeterState::new(frequency)}),
            stop:Owned::event(true)?,changed:Owned::event(false)?,policy}),worker:Mutex::new(None)})
    }
    unsafe fn bind(&self,owner:Handle,thread:u32)->bool {
        if !valid_owner(owner,thread)||GetAncestor(owner,2)!=owner{return false;}
        let life=Owned(OpenThread(SYNC|0x0800,0,thread) as usize);
        if life.0==0||GetProcessIdOfThread(life.raw())!=GetCurrentProcessId()
            ||!valid_owner(owner,thread)||WaitForSingleObject(life.raw(),0)!=TIMEOUT{return false;}
        let mut worker=lock(&self.worker);
        let finished={
            let mut s=lock(&self.shared.state);
            s.owner=owner as usize;s.thread=thread;s.life=Some(life);s.idle=0;
            s.generation=s.generation.wrapping_add(1);s.meter.reset();s.finished
        };
        if finished {if let Some(w)=worker.take(){let _=w.join();}}
        if worker.is_none(){
            let mut pinned=ptr::null_mut();
            if GetModuleHandleExW(5,module_anchor as *const () as *const u16,&mut pinned)==0 {
                self.shared.unbind();return false;
            }
            lock(&self.shared.state).finished=false;
            let shared=Arc::clone(&self.shared);
            match thread::Builder::new().spawn(move||run(shared)){
                Ok(w)=>*worker=Some(w),
                Err(_)=>{self.shared.unbind();self.shared.fail();return false;}
            }
        }
        self.shared.signal()
    }
    fn snapshot(&self)->HudSnapshot {
        let s=lock(&self.shared.state);
        let mut out=s.display;let reading=s.meter.reading(now());
        out.accepted=reading.accepted;out.milli=reading.milli_fps;
        out.valid=if s.owner!=0&&s.thread!=0&&out.active!=0{reading.valid}else{0};out
    }
}
impl Drop for Hud {
    fn drop(&mut self){unsafe{SetEvent(self.shared.stop.raw());}
        if let Some(w)=lock(&self.worker).take(){let _=w.join();}
    }
}
#[no_mangle]
pub unsafe extern "C" fn lazytester_hud_create(selected:Handle,stop:Handle,controller:Handle)->*mut Hud {
    Hud::new(selected,stop,controller).map_or(ptr::null_mut(),|h|Box::into_raw(Box::new(h)))
}
#[no_mangle]
pub unsafe extern "C" fn lazytester_hud_destroy(h:*mut Hud){if !h.is_null(){drop(Box::from_raw(h));}}
#[no_mangle]
pub unsafe extern "C" fn lazytester_hud_bind(h:*const Hud,owner:Handle,thread:u32)->u32 {
    u32::from(h.as_ref().is_some_and(|h|h.bind(owner,thread)))
}
#[no_mangle]
pub unsafe extern "C" fn lazytester_hud_unbind(h:*const Hud){if let Some(h)=h.as_ref(){h.shared.unbind();}}
#[no_mangle]
pub unsafe extern "C" fn lazytester_hud_present(h:*const Hud,result:i32,flags:u32){
    if result==0&&flags&1==0 {if let Some(h)=h.as_ref(){lock(&h.shared.state).meter.record(now());}}
}
#[no_mangle]
pub unsafe extern "C" fn lazytester_hud_snapshot(h:*const Hud,out:*mut HudSnapshot){
    if let Some(out)=out.as_mut(){*out=h.as_ref().map_or(HudSnapshot::default(),Hud::snapshot);}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hud_constructor_preserves_legacy_hidden_and_selected_policy() {
        unsafe {
            let selected = Owned::event(true).unwrap();
            let stop = Owned::event(true).unwrap();
            let controller = Owned::event(true).unwrap();
            let null = ptr::null_mut();
            let unmanaged = Hud::new(null, null, null).unwrap();
            assert!(unmanaged.shared.admitted());
            // Legacy policy has no selection handle; it must remain hidden.
            let legacy = Hud::new(null, stop.raw(), controller.raw()).unwrap();
            assert!(!legacy.shared.admitted());
            for (a, b, c) in [(selected.raw(), null, controller.raw()),
                (selected.raw(), stop.raw(), null), (null, stop.raw(), null),
                (null, null, controller.raw())] {
                assert!(Hud::new(a, b, c).is_none());
            }
            let managed = Hud::new(selected.raw(), stop.raw(), controller.raw()).unwrap();
            assert!(!managed.shared.admitted());
            assert_ne!(SetEvent(selected.raw()), 0);
            assert!(managed.shared.admitted());
            assert!(!legacy.shared.admitted());
            assert_ne!(SetEvent(controller.raw()), 0);
            assert!(!managed.shared.admitted());
            // Synthetic events only; no bind, window, thread or game launch.
            assert_eq!(managed.snapshot().active, 0);
        }
    }
}
