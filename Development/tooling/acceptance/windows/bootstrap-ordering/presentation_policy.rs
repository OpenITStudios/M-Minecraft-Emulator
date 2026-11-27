//! Private presentation capability version and FPS limit decoding.
use super::frame_pacing::lazytester_frame_pacing_valid_ceilings;
fn decode(version:u32,reserved:u32)->Option<(u32,u32)> {
    if version==2 { return (reserved==0).then_some((144,10)); }
    if version!=3 && version!=4 { return None; }
    let active=reserved&0xffff;
    let background=reserved>>16;
    (lazytester_frame_pacing_valid_ceilings(active,background)!=0).then_some((active,background))
}
#[no_mangle]
pub unsafe extern "C" fn lazytester_presentation_limits(version:u32,reserved:u32,active:*mut u32,background:*mut u32)->u32 {
    if active.is_null() || background.is_null() || active==background {return 0;}
    let Some((a,b))=decode(version,reserved) else {return 0;};
    active.write(a);background.write(b);1
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn versions_and_limits() {
        assert_eq!(decode(2,0),Some((144,10)));
        assert_eq!(decode(2,60|(5<<16)),None);
        for version in [3,4] {
            assert_eq!(decode(version,60|(5<<16)),Some((60,5)));
            assert_eq!(decode(version,60|(61<<16)),None);
            assert_eq!(decode(version,0),None);
        }
        assert_eq!(decode(5,60|(5<<16)),None);
    }
}

#[cfg(windows)]
pub(crate) mod native {
    use super::decode;
    use std::{ffi::{c_void,c_char},mem,ptr};
    type Handle = *mut c_void;
    type Query = unsafe extern "system" fn(Handle,u32,*mut c_void,u32,*mut u32)->i32;
    type Compare = unsafe extern "system" fn(Handle,Handle)->i32;
    #[repr(C)]
    #[derive(Clone,Copy,Default,PartialEq,Eq)]
    struct FileTime { low:u32,high:u32 }
    #[repr(C)]
    struct ObjectBasic {
        attributes:u32, granted_access:u32, handle_count:u32,pointer_count:u32,reserved:[u32;10],
    }
    #[repr(C)]
    struct EventInfo { event_type:i32,state:i32 }
    const _:()=assert!(mem::size_of::<ObjectBasic>()==56);
    const _:()=assert!(mem::size_of::<EventInfo>()==8);
    #[link(name="kernel32")]
    extern "system" {
        fn GetModuleHandleW(name:*const u16)->Handle;
        fn GetProcAddress(module:Handle,name:*const c_char)->*mut c_void;
        fn GetProcessId(process:Handle)->u32;
        fn GetProcessTimes(process:Handle,birth:*mut FileTime,exit:*mut FileTime,kernel:*mut FileTime,user:*mut FileTime)->i32;
        fn ProcessIdToSessionId(pid:u32,session:*mut u32)->i32;
        fn WaitForSingleObject(handle:Handle,ms:u32)->u32;
        fn SetHandleInformation(handle:Handle,mask:u32,flags:u32)->i32;
    }
    const NTDLL:[u16;10]=[110,116,100,108,108,46,100,108,108,0];
    const KERNELBASE:[u16;15]=[107,101,114,110,101,108,98,97,115,101,46,100,108,108,0];
    const SYNC:u32=0x0010_0000;
    const PROCESS_QUERY_LIMITED_INFORMATION:u32=0x1000;
    const EVENT_QUERY_STATE:u32=1;
    const WAIT_TIMEOUT:u32=258;

    // The inherited capability is already copied from a bounded read-only mapping.
    // Resolve only loaded modules. NtQueryEvent remains undocumented: reject
    // missing exports, unexpected return lengths, access or event layouts.
    unsafe fn validate(version:u32,reserved:u32,handles:[Handle;5],controller_pid:u32,
        birth:FileTime,child_pid:u32,child_session:u32)->Option<(u32,u32)> {
        let limits=decode(version,reserved)?;
        if controller_pid==0 || controller_pid==child_pid {return None;}
        let ntdll=GetModuleHandleW(NTDLL.as_ptr());
        let kernelbase=GetModuleHandleW(KERNELBASE.as_ptr());
        if ntdll.is_null() || kernelbase.is_null() {return None;}
        let object=GetProcAddress(ntdll,b"NtQueryObject\0".as_ptr().cast());
        let event=GetProcAddress(ntdll,b"NtQueryEvent\0".as_ptr().cast());
        let compare=GetProcAddress(kernelbase,b"CompareObjectHandles\0".as_ptr().cast());
        if object.is_null()||event.is_null()||compare.is_null(){return None;}
        let object:Query=mem::transmute(object);
        let event:Query=mem::transmute(event);
        let compare:Compare=mem::transmute(compare);
        let count=if version==4 {5}else{4};
        for i in 0..count {
            let handle=handles[i];
            if handle as isize<=0{return None;}
            for &earlier in &handles[..i] {
                if handle==earlier || compare(handle,earlier)!=0{return None;}
            }
            let mut basic:ObjectBasic=mem::zeroed();
            let mut returned=0;
            let size=mem::size_of::<ObjectBasic>() as u32;
            let expected=SYNC|if i==3 {PROCESS_QUERY_LIMITED_INFORMATION}else{EVENT_QUERY_STATE};
            if object(handle,0,(&mut basic as *mut ObjectBasic).cast(),size,&mut returned)!=0
                || returned!=size || basic.granted_access!=expected {return None;}
            if i!=3 {
                let mut info:EventInfo=mem::zeroed();
                let size=mem::size_of::<EventInfo>() as u32;
                returned=0;
                if event(handle,0,(&mut info as *mut EventInfo).cast(),size,&mut returned)!=0
                    || returned!=size || info.event_type!=if i==1 {1}else{0}
                    || (i==2 && info.state!=0) {return None;}
            }
        }
        let controller=handles[3];
        let (mut actual,mut exit,mut kernel,mut user)=(FileTime::default(),FileTime::default(),FileTime::default(),FileTime::default());
        let mut session=0;
        if GetProcessId(controller)!=controller_pid
            || GetProcessTimes(controller,&mut actual,&mut exit,&mut kernel,&mut user)==0
            || actual!=birth || ProcessIdToSessionId(controller_pid,&mut session)==0
            || session!=child_session || WaitForSingleObject(controller,0)!=WAIT_TIMEOUT {return None;}
        for &handle in &handles[..count] {
            if SetHandleInformation(handle,1,0)==0{return None;}
        }
        Some(limits)
    }

    // Scalar ABI avoids sharing the full versioned C++ mapping layout.
    // Outputs are published only after all admission checks succeed.
    #[no_mangle]
    pub unsafe extern "C" fn lazytester_presentation_validate(version:u32,reserved:u32,
        level:Handle,changed:Handle,stop:Handle,controller:Handle,selected:Handle,
        controller_pid:u32,birth_low:u32,birth_high:u32,child_pid:u32,child_session:u32,
        active:*mut u32,background:*mut u32)->u32 {
        if active.is_null()||background.is_null()||active==background{return 0;}
        let Some((a,b))=validate(version,reserved,[level,changed,stop,controller,selected],
            controller_pid,FileTime{low:birth_low,high:birth_high},child_pid,child_session) else{return 0;};
        ptr::write(active,a);ptr::write(background,b);1
    }
}
