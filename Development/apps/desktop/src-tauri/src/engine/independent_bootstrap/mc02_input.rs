//! MC02-only source implementation: controller-owned, pagefile-backed input
//! state consumed inside the already enrolled bootstrap-compat DLL.
//! No global SendInput, driver, new named pipe, second game controller, or
//! profile storage write. This module is absent unless its Cargo feature is
//! explicitly enabled. Minecraft Education compatibility is NOT_EXECUTED.
use super::*;
use std::sync::atomic::{AtomicU64, Ordering};
use windows_sys::Win32::System::SystemInformation::GetTickCount64;

pub(super) const INPUT_ENV: &str = "MIVUBI_EDUCATION_MC02_INPUT_SECTION";
pub(super) const INPUT_OPTIN: &str = "LAZY_TESTER_MC02_VIRTUAL_INPUT";
pub(super) const INPUT_CONSENT: &str = "I_ACCEPT_EXPERIMENTAL_MC02_INPUT";
const MAGIC: u32 = 0x4c544932; // LTI2 (MC02 only)
const VERSION: u32 = 1;

// C++ counterpart: mc02-virtual-input.cpp. Atomic 64-bit on Win x64.
// High 32 bits: GetTickCount() expiration. Bits 0..7: W,S,A,D,
// jump,sneak,attack,use. Bits 16..23 / 24..31: signed dx/dy.
// Expiration is bounded to 500 ms; no sticky input after an agent crash.
#[repr(C, align(8))]
struct SharedInput {
    magic:u32, version:u32, bytes:u32, slot:u32, pid:u32, session:u32,
    birth:u64,
    packet:AtomicU64,
}
const _: () = assert!(std::mem::size_of::<SharedInput>() == 40);
const _: () = assert!(std::mem::align_of::<SharedInput>() == 8);

pub(super) struct Channel {
    section: Configuration,
    target: Option<(u32,u64,u32)>,
}
impl Channel {
    pub(super) fn create(slot:u32) -> io::Result<Option<Self>> {
        if slot!=2 || std::env::var(INPUT_OPTIN).ok().as_deref()!=Some(INPUT_CONSENT) {
            return Ok(None);
        }
        Ok(Some(Self {
            section:Configuration::create_sized(std::mem::size_of::<SharedInput>() as u32)?,
            target:None,
        }))
    }
    pub(super) fn reader(&self)->HANDLE {
        self.section.readonly.as_raw_handle() as HANDLE
    }
    fn mapped(&self)->io::Result<windows_sys::Win32::System::Memory::MEMORY_MAPPED_VIEW_ADDRESS> {
        let view=unsafe {MapViewOfFile(self.section.writable.as_raw_handle() as HANDLE,
            FILE_MAP_WRITE,0,0,std::mem::size_of::<SharedInput>())};
        if view.Value.is_null() {Err(io::Error::last_os_error())} else {Ok(view)}
    }
    pub(super) fn bind(&mut self, pid:u32, birth:u64, session:u32)->io::Result<()> {
        if pid==0 || birth==0 || session==0 || self.target.is_some() {
            return Err(invalid("MC02 input binding is not an unstarted owned client"));
        }
        let view=self.mapped()?;
        unsafe {
            std::ptr::write(view.Value.cast::<SharedInput>(), SharedInput {
                magic:MAGIC,version:VERSION,
                bytes:std::mem::size_of::<SharedInput>() as u32,
                slot:2,pid,session,birth,packet:AtomicU64::new(0),
            });
        }
        if unsafe{UnmapViewOfFile(view)}==0 {return Err(io::Error::last_os_error());}
        // Inherited by the ONE already-created MC02 child. Never leak it to
        // unrelated children spawned later by the controller.
        if unsafe{SetHandleInformation(self.reader(),HANDLE_FLAG_INHERIT,0)}==0 {
            return Err(io::Error::last_os_error());
        }
        self.target=Some((pid,birth,session));
        Ok(())
    }
    pub(super) fn write(&self,pid:u32,birth:u64,session:u32,
                        keys:u32,dx:i32,dy:i32,ttl_ms:u32)->io::Result<()> {
        if self.target!=Some((pid,birth,session)) || keys>255 ||
            !(-127..=127).contains(&dx) || !(-127..=127).contains(&dy) ||
            !(1..=500).contains(&ttl_ms) {
            return Err(invalid("Unbound, expired or invalid MC02 input command"));
        }
        let expires=(unsafe{GetTickCount64()} as u32).wrapping_add(ttl_ms);
        let word=((expires as u64)<<32) | ((dy as i8 as u8 as u64)<<24)
             | ((dx as i8 as u8 as u64)<<16) | keys as u64;
        let view=self.mapped()?;
        // The existing authenticated native controller is the sole writer.
        unsafe{(*view.Value.cast::<SharedInput>()).packet.store(word,Ordering::Release);}
        if unsafe{UnmapViewOfFile(view)}==0 {return Err(io::Error::last_os_error());}
        Ok(())
    }
}
