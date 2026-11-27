//! Shared enrollment image metadata and bounded capability-header validation.
//! No device creation, loader calls, allocation, worker or game changes.
use std::{ffi::c_void,mem,ptr};
type Handle=*mut c_void;
#[repr(C)]
#[derive(Clone,Copy,Default,PartialEq,Eq)]
pub struct FileId { volume:u64, id:[u8;16] }
impl FileId { pub(crate) fn as_bytes(&self)->[u8;24] { let mut b=[0;24];b[..8].copy_from_slice(&self.volume.to_le_bytes());b[8..].copy_from_slice(&self.id);b } }
#[repr(C)]
#[derive(Clone,Copy,Default)]
pub struct ImageMetadata { pub timestamp:u32, pub bytes:u32, pub id:FileId }
const _:()=assert!(mem::size_of::<ImageMetadata>()==32);
#[repr(C)]
struct MemoryInfo {
    base:Handle,allocation_base:Handle,allocation_protect:u32,partition:u16,
    region_size:usize,state:u32,protect:u32,kind:u32,
}
const _:()=assert!(mem::size_of::<MemoryInfo>()==48);
#[link(name="kernel32")]
extern "system" {
    fn GetModuleHandleW(name:*const u16)->Handle;
    fn GetModuleFileNameW(module:Handle,name:*mut u16,capacity:u32)->u32;
    fn GetSystemDirectoryW(name:*mut u16,capacity:u32)->u32;
    fn CompareStringOrdinal(a:*const u16,an:i32,b:*const u16,bn:i32,ignore_case:i32)->i32;
    fn CreateFileW(name:*const u16,access:u32,share:u32,attributes:Handle,creation:u32,flags:u32,template:Handle)->Handle;
    fn GetFileInformationByHandleEx(file:Handle,kind:i32,out:*mut c_void,bytes:u32)->i32;
    fn CloseHandle(handle:Handle)->i32;
    fn GetCurrentProcess()->Handle;
    fn ReadProcessMemory(process:Handle,address:*const c_void,out:*mut c_void,bytes:usize,read:*mut usize)->i32;
    fn VirtualQuery(address:*const c_void,out:*mut MemoryInfo,bytes:usize)->usize;
}
const DXGI:[u16;9]=[100,120,103,105,46,100,108,108,0];
const SUFFIX:[u16;10]=[92,100,120,103,105,46,100,108,108,0];
const FILE_READ_ATTRIBUTES:u32=0x80;
const FILE_FLAG_OPEN_REPARSE_POINT:u32=0x0020_0000;
struct File(Handle);
impl Drop for File {fn drop(&mut self){unsafe{CloseHandle(self.0);}}}
fn u16_at(b:&[u8],offset:usize)->Option<u16>{Some(u16::from_le_bytes(b.get(offset..offset+2)?.try_into().ok()?))}
fn u32_at(b:&[u8],offset:usize)->Option<u32>{Some(u32::from_le_bytes(b.get(offset..offset+4)?.try_into().ok()?))}
unsafe fn read<const N:usize>(module:Handle,offset:usize)->Option<[u8;N]> {
    let address=(module as usize).checked_add(offset)? as *const c_void;
    let mut bytes=[0;N];let mut count=0;
    if ReadProcessMemory(GetCurrentProcess(),address,bytes.as_mut_ptr().cast(),N,&mut count)==0 || count!=N {None}
    else{Some(bytes)}
}
fn pe_offset(dos:&[u8])->Option<usize>{
    if u16_at(dos,0)?!=0x5a4d{return None;}
    let offset=u32_at(dos,60)? as usize;
    (offset>0&&offset<=4096).then_some(offset)
}
fn pe_metadata(nt:&[u8])->Option<(u32,u32)>{
    if u32_at(nt,0)?!=0x4550||u16_at(nt,4)?!=0x8664||u16_at(nt,24)?!=0x20b{return None;}
    // SizeOfImage lies at offset 56 in the PE32+ optional header.
    if u16_at(nt,20)?<60{return None;}
    let bytes=u32_at(nt,80)?;
    (bytes>4096).then_some((u32_at(nt,8)?,bytes))
}
unsafe fn metadata()->Option<(Handle,ImageMetadata)> {
    let module=GetModuleHandleW(DXGI.as_ptr());
    if module.is_null(){return None;}
    let (mut actual,mut system)=([0u16;1024],[0u16;1024]);
    let actual_len=GetModuleFileNameW(module,actual.as_mut_ptr(),1024) as usize;
    let system_len=GetSystemDirectoryW(system.as_mut_ptr(),1024) as usize;
    if actual_len==0||actual_len>=1024||system_len==0||system_len+10>=1024{return None;}
    system[system_len..system_len+10].copy_from_slice(&SUFFIX);
    if CompareStringOrdinal(actual.as_ptr(),actual_len as i32,system.as_ptr(),(system_len+9) as i32,1)!=2{return None;}
    let file=CreateFileW(system.as_ptr(),FILE_READ_ATTRIBUTES,7,ptr::null_mut(),3,FILE_FLAG_OPEN_REPARSE_POINT,ptr::null_mut());
    if file as isize == -1{return None;}
    let file=File(file);
    let mut attributes=[0u32;2];let mut id=FileId::default();
    // FILE_ATTRIBUTE_TAG_INFO = 9; FILE_ID_INFO = 18.
    if GetFileInformationByHandleEx(file.0,9,attributes.as_mut_ptr().cast(),8)==0
        ||attributes[0]&(0x10|0x400)!=0
        ||GetFileInformationByHandleEx(file.0,18,(&mut id as *mut FileId).cast(),24)==0{return None;}
    let dos=read::<64>(module,0)?;
    let nt=read::<84>(module,pe_offset(&dos)?)?;
    let (timestamp,bytes)=pe_metadata(&nt)?;
    Some((module,ImageMetadata{timestamp,bytes,id}))
}
#[no_mangle]
pub unsafe extern "C" fn lazytester_presentation_image(out:*mut ImageMetadata,module:*mut Handle)->u32 {
    if out.is_null()||module.is_null(){return 0;}
    let Some((h,m))=metadata() else{return 0;};
    out.write(m);module.write(h);1
}
fn expected_bytes(pacing:bool,version:u32)->Option<u32>{
    match (pacing,version){(false,1)=>Some(72),(true,2|3)=>Some(120),(true,4)=>Some(128),_=>None}
}
#[no_mangle]
pub extern "C" fn lazytester_presentation_bytes(pacing:u32,version:u32)->u32{
    expected_bytes(pacing!=0,version).unwrap_or(0)
}
#[no_mangle]
pub unsafe extern "C" fn lazytester_presentation_entry(module:Handle,image_bytes:u32,rva:u32)->Handle{
    if module.is_null()||rva<4096||rva>=image_bytes{return ptr::null_mut();}
    let Some(address)=(module as usize).checked_add(rva as usize) else{return ptr::null_mut();};
    let mut info:MemoryInfo=mem::zeroed();
    if VirtualQuery(address as Handle,&mut info,mem::size_of::<MemoryInfo>())!=mem::size_of::<MemoryInfo>()
        ||info.allocation_base!=module||info.state!=0x1000||info.kind!=0x0100_0000
        ||info.protect&0x100!=0||info.protect&0xf0==0{return ptr::null_mut();}
    address as Handle
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn sizes(){assert_eq!(expected_bytes(false,1),Some(72));assert_eq!(expected_bytes(true,2),Some(120));assert_eq!(expected_bytes(true,3),Some(120));assert_eq!(expected_bytes(true,4),Some(128));assert_eq!(expected_bytes(false,4),None);assert_eq!(expected_bytes(true,5),None);}
    #[test]fn malformed_headers(){
        let mut dos=[0u8;64];assert_eq!(pe_offset(&dos),None);
        dos[..2].copy_from_slice(&0x5a4du16.to_le_bytes());dos[60..64].copy_from_slice(&64u32.to_le_bytes());
        assert_eq!(pe_offset(&dos),Some(64));
        let mut nt=[0u8;84];assert_eq!(pe_metadata(&nt),None);
        nt[..4].copy_from_slice(&0x4550u32.to_le_bytes());nt[4..6].copy_from_slice(&0x8664u16.to_le_bytes());
        nt[20..22].copy_from_slice(&240u16.to_le_bytes());nt[24..26].copy_from_slice(&0x20bu16.to_le_bytes());
        nt[8..12].copy_from_slice(&7u32.to_le_bytes());nt[80..84].copy_from_slice(&8192u32.to_le_bytes());
        assert_eq!(pe_metadata(&nt),Some((7,8192)));assert_eq!(pe_metadata(&nt[..80]),None);
    }
}
