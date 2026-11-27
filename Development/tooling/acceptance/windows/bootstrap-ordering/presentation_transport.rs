//! Read-only presentation mapping transport and admission.
use std::{ffi::c_void,ptr};
use super::presentation_image::{lazytester_presentation_bytes,lazytester_presentation_entry,lazytester_presentation_image,ImageMetadata};
use super::presentation_policy::native::lazytester_presentation_validate;
type Handle=*mut c_void;
const MODE:&[u16]=&[77,73,86,85,66,73,95,69,68,85,67,65,84,73,79,78,95,80,82,69,83,69,78,84,65,84,73,79,78,0];
const SECTION:&[u16]=&[77,73,86,85,66,73,95,69,68,85,67,65,84,73,79,78,95,80,82,69,83,69,78,84,65,84,73,79,78,95,83,69,67,84,73,79,78,0];
const PACE:&[u16]=&[112,97,99,101];
const OBSERVE:&[u16]=&[111,98,115,101,114,118,101];
#[link(name="kernel32")]
extern "system" {
    fn GetEnvironmentVariableW(name:*const u16,out:*mut u16,size:u32)->u32;
    fn GetLastError()->u32;
    fn SetLastError(error:u32);
    fn SetHandleInformation(handle:Handle,mask:u32,flags:u32)->i32;
    fn MapViewOfFile(section:Handle,access:u32,high:u32,low:u32,size:usize)->Handle;
    fn UnmapViewOfFile(view:Handle)->i32;
    fn CloseHandle(handle:Handle)->i32;
    fn GetCurrentProcess()->Handle;
    fn ReadProcessMemory(process:Handle,address:*const c_void,out:*mut c_void,bytes:usize,read:*mut usize)->i32;
}
struct Section(Handle);
impl Drop for Section {fn drop(&mut self){unsafe{CloseHandle(self.0);}}}
struct View(Handle);
impl Drop for View {fn drop(&mut self){unsafe{UnmapViewOfFile(self.0);}}}
fn parse_handle(s:&[u16])->Option<usize>{
    if s.is_empty(){return None;}
    let mut value=0usize;
    for &c in s {
        if !(48..=57).contains(&c){return None;}
        value=value.checked_mul(10)?.checked_add((c-48) as usize)?;
    }
    (value!=0&&value<=isize::MAX as usize).then_some(value)
}
fn word(b:&[u8],at:usize)->Option<u32>{Some(u32::from_le_bytes(b.get(at..at+4)?.try_into().ok()?))}
fn handle(b:&[u8],at:usize)->Option<Handle>{Some(u64::from_le_bytes(b.get(at..at+8)?.try_into().ok()?) as usize as Handle)}
unsafe fn copy(section:Handle,bytes:usize,out:&mut [u8])->bool{
    if bytes>out.len(){return false;}
    let view=MapViewOfFile(section,4,0,0,bytes);
    if view.is_null(){return false;}
    let view=View(view);let mut read=0usize;
    ReadProcessMemory(GetCurrentProcess(),view.0,out.as_mut_ptr().cast(),bytes,&mut read)!=0 && read==bytes
}
#[repr(C)]
pub struct Admission {
    mode:u32,active:u32,background:u32,reserved:u32,
    old_entry:Handle,hwnd_entry:Handle,level:Handle,changed:Handle,stop:Handle,controller:Handle,selected:Handle,
}
const _:()={
    use std::mem::{align_of,offset_of,size_of};
    assert!(size_of::<Admission>()==72 && align_of::<Admission>()==8);
    assert!(offset_of!(Admission,old_entry)==16 && offset_of!(Admission,selected)==64);
};
impl Admission {
    fn empty()->Self{Self{mode:0,active:144,background:10,reserved:0,old_entry:ptr::null_mut(),hwnd_entry:ptr::null_mut(),
        level:ptr::null_mut(),changed:ptr::null_mut(),stop:ptr::null_mut(),controller:ptr::null_mut(),selected:ptr::null_mut()}}
}
unsafe fn admit(pid:u32,session:u32,birth_low:u32,birth_high:u32)->Option<Admission>{
    let mut setting=[0u16;32];
    SetLastError(0);
    let len=GetEnvironmentVariableW(MODE.as_ptr(),setting.as_mut_ptr(),32) as usize;
    if len==0&&GetLastError()==203 {
        SetLastError(0);
        return (GetEnvironmentVariableW(SECTION.as_ptr(),setting.as_mut_ptr(),32)==0&&GetLastError()==203).then_some(Admission::empty());
    }
    let mode=match setting.get(..len)?{s if s==PACE=>2,s if s==OBSERVE=>1,_=>return None};
    let len=GetEnvironmentVariableW(SECTION.as_ptr(),setting.as_mut_ptr(),32) as usize;
    if len==0||len>=32{return None;}
    let raw=parse_handle(&setting[..len])? as Handle;
    // Do not take ownership until the inherited handle admits flag changes.
    if SetHandleInformation(raw,1,0)==0{return None;}
    let section=Section(raw);
    let mut data=[0u8;128];
    if !copy(section.0,72,&mut data){return None;}
    let version=word(&data,4)?;
    let bytes=lazytester_presentation_bytes(u32::from(mode==2),version) as usize;
    if bytes==0||bytes>data.len(){return None;}
    // Writable capabilities are never accepted.
    let writable=MapViewOfFile(section.0,2,0,0,bytes);
    if !writable.is_null(){drop(View(writable));return None;}
    data.fill(0);
    if !copy(section.0,bytes,&mut data){return None;}
    drop(section);
    if word(&data,0)?!=0x50524431||word(&data,4)?!=version||word(&data,8)? as usize!=bytes
        ||word(&data,12)?!=pid||word(&data,16)?!=birth_low||word(&data,20)?!=birth_high
        ||word(&data,24)?!=session{return None;}
    let mut image=ImageMetadata::default();let mut module=ptr::null_mut();
    if lazytester_presentation_image(&mut image,&mut module)==0||word(&data,28)?!=image.timestamp
        ||word(&data,32)?!=image.bytes||data[48..72]!=image.id.as_bytes(){return None;}
    let old=lazytester_presentation_entry(module,image.bytes,word(&data,36)?);
    let hwnd=lazytester_presentation_entry(module,image.bytes,word(&data,40)?);
    if old.is_null()||hwnd.is_null()||old==hwnd{return None;}
    let mut result=Admission::empty();
    result.mode=mode;result.old_entry=old;result.hwnd_entry=hwnd;
    if mode==2{
        result.level=handle(&data,72)?;result.changed=handle(&data,80)?;
        result.stop=handle(&data,88)?;result.controller=handle(&data,96)?;
        if version==4{result.selected=handle(&data,120)?;}
        if lazytester_presentation_validate(version,word(&data,108)?,result.level,result.changed,result.stop,
            result.controller,result.selected,word(&data,104)?,word(&data,112)?,word(&data,116)?,
            pid,session,&mut result.active,&mut result.background)==0{return None;}
    }
    Some(result)
}
#[no_mangle]
pub unsafe extern "C" fn lazytester_presentation_admit(pid:u32,session:u32,birth_low:u32,birth_high:u32,out:*mut Admission)->u32{
    if out.is_null(){return 0;}
    let Some(result)=admit(pid,session,birth_low,birth_high) else{return 0;};
    out.write(result);1
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn bounded_handles(){
        assert_eq!(parse_handle(&[49,50,51]),Some(123));
        assert_eq!(parse_handle(&[]),None);assert_eq!(parse_handle(&[48]),None);
        assert_eq!(parse_handle(&[45,49]),None);assert_eq!(parse_handle(&[57;32]),None);
    }
}

 // Candidate classification is observation only: no retained HWND or COM reference.
#[link(name="user32")]
extern "system" {
    fn GetWindowThreadProcessId(window:Handle,pid:*mut u32)->u32;
    fn GetAncestor(window:Handle,flags:u32)->Handle;
    fn GetClassNameW(window:Handle,name:*mut u16,max:i32)->i32;
}
#[link(name="kernel32")]
extern "system" { fn GetCurrentProcessId()->u32; }
#[no_mangle]
pub unsafe extern "C" fn lazytester_presentation_candidate(window:Handle)->u32 {
    if window.is_null(){return 0;}
    let mut pid=0;
    if GetWindowThreadProcessId(window,&mut pid)==0 || pid!=GetCurrentProcessId()
        || GetAncestor(window,2)!=window {return 0;}
    let mut name=[0u16;128];
    let len=GetClassNameW(window,name.as_mut_ptr(),128);
    u32::from(len==5 && name[..5]==[79,71,76,69,83])
}
