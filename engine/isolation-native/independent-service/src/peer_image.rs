//! Win32 Authenticode observation of the exact native IPC PEER process.
//!
//! It is NOT executable-origin authorization. A signed file at the path
//! reported by QueryFullProcessImageNameW is NOT proof that the running
//! process's mapped image section came from that file. Neither a generic
//! Windows-trusted signer nor a matching leaf fingerprint grants privileged
//! IPC. No signing identity is guessed or embedded in this source.
//!
//! This original service-owned observer is compiled ONLY when the optional
//! readonly-ipc-pilot feature is enabled. Its sole caller holds a process
//! HANDLE pinned by OS PID + full creation FILETIME. The probe returns only
//! pending local evidence and never logs/serializes paths, SIDs or certs.

#![allow(dead_code)]

use std::{
    ffi::OsString, io, mem::size_of, os::windows::{
        ffi::OsStringExt,
        fs::OpenOptionsExt,
        io::AsRawHandle,
    },
    path::PathBuf, ptr,
};
use sha2::{Digest, Sha256};
use windows_sys::Win32::{
    Foundation::{HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT},
    Security::WinTrust::{
        WinVerifyTrust, WINTRUST_ACTION_GENERIC_VERIFY_V2,
        CRYPT_PROVIDER_CERT, CRYPT_PROVIDER_DATA, CRYPT_PROVIDER_SGNR,
        WINTRUST_DATA, WINTRUST_DATA_0, WINTRUST_FILE_INFO,
        WTD_CACHE_ONLY_URL_RETRIEVAL, WTD_CHOICE_FILE,
        WTD_REVOKE_WHOLECHAIN, WTD_STATEACTION_CLOSE,
        WTD_STATEACTION_VERIFY, WTD_UI_NONE,
    },
    Storage::FileSystem::{
        GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
        FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT, FILE_SHARE_READ,
    },
    System::{
        LibraryLoader::{GetProcAddress, LoadLibraryExW, LOAD_LIBRARY_SEARCH_SYSTEM32},
        Threading::{QueryFullProcessImageNameW, WaitForSingleObject},
    },
};

/// Reviewed signer DER leaf fingerprints are intentionally UNKNOWN.
/// A verified Authenticode chain is not a pin to the MIVUBI publisher.
/// Only an explicitly reviewed, signed installed-release policy may ever
/// fill this list; do not add a test/self-signed certificate here.
const REVIEWED_DESKTOP_LEAF_SHA256: &[[u8; 32]] = &[];

#[link(name = "kernel32")]
unsafe extern "system" {
    fn FreeLibrary(module: *mut std::ffi::c_void) -> i32;
}

fn refused(reason: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, reason)
}
fn invalid(reason: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, reason)
}

/// This is ONLY on-disk signature evidence, with no mapped-section binding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PendingPeerImageTrust {
    leaf_sha256: [u8; 32],
    on_disk_authenticode_verified: bool,
}
impl PendingPeerImageTrust {
    /// Check only a leaf fingerprint against an explicitly reviewed signer
    /// list. Even TRUE is not full application/process authentication.
    /// With the checked-in EMPTY list, this always rejects.
    pub(super) fn matches_reviewed_on_disk_signer(self) -> bool {
        signer_seen_against_pins(
            self.on_disk_authenticode_verified,
            Some(self.leaf_sha256),
            REVIEWED_DESKTOP_LEAF_SHA256,
        )
    }
}

fn signer_seen_against_pins(
    wintrust_ok: bool, leaf_sha256: Option<[u8; 32]>, pins: &[[u8; 32]],
) -> bool {
    if !wintrust_ok || pins.is_empty() {
        return false;
    }
    leaf_sha256.is_some_and(|leaf| leaf != [0; 32] && pins.contains(&leaf))
}

fn queried_image_path(process: HANDLE) -> io::Result<Vec<u16>> {
    if process.is_null() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "No retained process HANDLE"));
    }
    let mut path = vec![0u16; 32_768];
    let mut used = path.len() as u32;
    // SAFETY: The passed process HANDLE is already held and the buffer
    // has enough capacity for the documented maximum Win32 path length.
    if unsafe {
        QueryFullProcessImageNameW(process, 0, path.as_mut_ptr(), &mut used)
    } == 0 {
        return Err(io::Error::last_os_error());
    }
    if used == 0 || (used as usize) >= path.len() {
        return Err(invalid("Invalid or truncated native process image path"));
    }
    path.truncate(used as usize);
    if path.iter().any(|&character| character == 0) {
        return Err(invalid("Embedded NUL in native process image path"));
    }
    Ok(path)
}

/// Stable, untrusted disk FILE identity, NOT the image-section identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FileIdentity {
    volume: u32,
    index: u64,
    size: u64,
    modified: u64,
}
fn file_identity(file: &std::fs::File) -> io::Result<FileIdentity> {
    // SAFETY: A live read-only file HANDLE and aligned native output.
    let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) } == 0 {
        return Err(io::Error::last_os_error());
    }
    if info.dwFileAttributes & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT) != 0 {
        return Err(refused("The image path is a directory or reparse object"));
    }
    let size = (u64::from(info.nFileSizeHigh) << 32) | u64::from(info.nFileSizeLow);
    if size == 0 {
        return Err(invalid("Empty executable file cannot be attested"));
    }
    Ok(FileIdentity {
        volume: info.dwVolumeSerialNumber,
        index: (u64::from(info.nFileIndexHigh) << 32) | u64::from(info.nFileIndexLow),
        size,
        modified: (u64::from(info.ftLastWriteTime.dwHighDateTime) << 32)
            | u64::from(info.ftLastWriteTime.dwLowDateTime),
    })
}

/// Get a DER *leaf signer certificate* fingerprint from a successful
/// provider state. These WTHelper exports require dynamic loading from
/// System32; their borrowed pointers are invalid after STATEACTION_CLOSE.
fn trusted_leaf_sha256(state: HANDLE) -> io::Result<[u8; 32]> {
    type GetProvider = unsafe extern "system" fn(HANDLE) -> *mut CRYPT_PROVIDER_DATA;
    type GetSigner = unsafe extern "system" fn(
        *mut CRYPT_PROVIDER_DATA, u32, i32, u32,
    ) -> *mut CRYPT_PROVIDER_SGNR;
    type GetCertificate = unsafe extern "system" fn(
        *mut CRYPT_PROVIDER_SGNR, u32,
    ) -> *mut CRYPT_PROVIDER_CERT;

    struct OwnedDll(*mut std::ffi::c_void);
    impl Drop for OwnedDll {
        fn drop(&mut self) {
            if !self.0.is_null() {
                // SAFETY: Owned LoadLibraryExW reference, never borrowed HMODULE.
                unsafe { FreeLibrary(self.0) };
            }
        }
    }
    if state.is_null() {
        return Err(invalid("WinTrust provider state missing"));
    }
    let dll_name: Vec<u16> = "wintrust.dll".encode_utf16().chain(Some(0)).collect();
    let dll = OwnedDll(unsafe {
        LoadLibraryExW(dll_name.as_ptr(), ptr::null_mut(), LOAD_LIBRARY_SEARCH_SYSTEM32)
    });
    if dll.0.is_null() {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: Windows-supplied Wintrust.dll exports; calls remain in scope
    // of both the WinTrust state and the loaded, System32-only module.
    let provider = unsafe {
        GetProcAddress(dll.0, b"WTHelperProvDataFromStateData\0".as_ptr())
    }.ok_or_else(|| invalid("WinTrust provider export unavailable"))?;
    let signer = unsafe {
        GetProcAddress(dll.0, b"WTHelperGetProvSignerFromChain\0".as_ptr())
    }.ok_or_else(|| invalid("WinTrust signer export unavailable"))?;
    let cert = unsafe {
        GetProcAddress(dll.0, b"WTHelperGetProvCertFromChain\0".as_ptr())
    }.ok_or_else(|| invalid("WinTrust signer-certificate export unavailable"))?;
    let get_provider: GetProvider = unsafe { std::mem::transmute(provider) };
    let get_signer: GetSigner = unsafe { std::mem::transmute(signer) };
    let get_cert: GetCertificate = unsafe { std::mem::transmute(cert) };
    let provider = unsafe { get_provider(state) };
    if provider.is_null() {
        return Err(invalid("WinTrust provider did not return data"));
    }
    let signer = unsafe { get_signer(provider, 0, 0, 0) };
    if signer.is_null() || unsafe { (*signer).dwError != 0 || (*signer).csCertChain == 0 } {
        return Err(invalid("Invalid primary signer chain"));
    }
    let leaf = unsafe { get_cert(signer, 0) };
    if leaf.is_null() || unsafe { (*leaf).dwError != 0 } {
        return Err(invalid("WinTrust leaf certificate not verified"));
    }
    let context = unsafe { (*leaf).pCert };
    if context.is_null() {
        return Err(invalid("WinTrust certificate context missing"));
    }
    let encoded = unsafe { &*context };
    let len = encoded.cbCertEncoded as usize;
    if encoded.pbCertEncoded.is_null() || !(1..=65_536).contains(&len) {
        return Err(invalid("Invalid WinTrust leaf certificate DER"));
    }
    let der = unsafe { std::slice::from_raw_parts(encoded.pbCertEncoded, len) };
    Ok(Sha256::digest(der).into())
}

/// Read-only, native trust observation for a live, already PID/FILETIME-pinned
/// OS process HANDLE. Never accept any caller-supplied pathname, signer pin,
/// or file bytes from a service request. This file has NOT been bound to the
/// running executable's mapped section; even a successful result is not
/// authorization to open a privileged IPC endpoint.
pub(super) fn inspect_os_peer_image(process: HANDLE) -> io::Result<PendingPeerImageTrust> {
    let original_path = queried_image_path(process)?;
    let path = PathBuf::from(OsString::from_wide(&original_path));
    // Windows file HANDLE is held across WinVerifyTrust to limit TOCTOU
    // on the disk file, with file sharing restricted to read-only peers.
    let file = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .open(path)?;
    let before = file_identity(&file)?;
    let mut wide = original_path.clone();
    wide.push(0);
    let mut file_info: WINTRUST_FILE_INFO = unsafe { std::mem::zeroed() };
    file_info.cbStruct = size_of::<WINTRUST_FILE_INFO>() as u32;
    file_info.pcwszFilePath = wide.as_ptr();
    file_info.hFile = file.as_raw_handle();

    let mut trust: WINTRUST_DATA = unsafe { std::mem::zeroed() };
    trust.cbStruct = size_of::<WINTRUST_DATA>() as u32;
    trust.dwUIChoice = WTD_UI_NONE;
    trust.fdwRevocationChecks = WTD_REVOKE_WHOLECHAIN;
    trust.dwUnionChoice = WTD_CHOICE_FILE;
    trust.Anonymous = WINTRUST_DATA_0 { pFile: &mut file_info };
    trust.dwStateAction = WTD_STATEACTION_VERIFY;
    // No interactive UI and NO network revocation retrieval by the service.
    // Missing cached revocation evidence is never promoted to success.
    trust.dwProvFlags = WTD_CACHE_ONLY_URL_RETRIEVAL;

    let mut action = WINTRUST_ACTION_GENERIC_VERIFY_V2;
    // SAFETY: WINTRUST_DATA/file_INFO and all backing buffers live until
    // the provider's state is finally closed below.
    let status = unsafe {
        WinVerifyTrust(ptr::null_mut(), &mut action, (&mut trust as *mut WINTRUST_DATA).cast())
    };
    let had_state = !trust.hWVTStateData.is_null();
    let signer = if status == 0 && had_state {
        trusted_leaf_sha256(trust.hWVTStateData)
    } else {
        Err(refused("Windows did not verify the image's on-disk signature"))
    };

    trust.dwStateAction = WTD_STATEACTION_CLOSE;
    // Even a failed WinVerifyTrust call can return provider-owned state.
    // Always CLOSE when a state handle exists, before returning any result.
    let closed = !had_state || unsafe {
        WinVerifyTrust(ptr::null_mut(), &mut action, (&mut trust as *mut WINTRUST_DATA).cast())
    } == 0;
    if !closed {
        return Err(invalid("Failed to close WinTrust provider state"));
    }
    if status != 0 || !had_state {
        return Err(refused("Authenticode verification is unavailable"));
    }
    let digest = signer?;
    if digest == [0; 32] {
        return Err(invalid("All-zero leaf fingerprint cannot be attested"));
    }
    if file_identity(&file)? != before || queried_image_path(process)? != original_path {
        return Err(invalid("Queried on-disk executable changed during trust review"));
    }
    // The existing caller also retains the process through its final PID/
    // FILETIME/liveness check; this extra observation rejects exit now.
    match unsafe { WaitForSingleObject(process, 0) } {
        WAIT_TIMEOUT => {}
        WAIT_OBJECT_0 => return Err(refused("Client exited during signed-file observation")),
        _ => return Err(io::Error::last_os_error()),
    }
    Ok(PendingPeerImageTrust {
        leaf_sha256: digest,
        on_disk_authenticode_verified: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_reviewed_desktop_leaf_identity_is_approved_by_default() {
        let leaf = [0x56; 32];
        assert!(REVIEWED_DESKTOP_LEAF_SHA256.is_empty());
        assert!(!signer_seen_against_pins(true, Some(leaf), &[]));
        assert!(!PendingPeerImageTrust {
            leaf_sha256: leaf,
            on_disk_authenticode_verified: true,
        }.matches_reviewed_on_disk_signer());
    }

    #[test]
    fn on_disk_leaf_comparison_rejects_unknown_untrusted_and_invalid_signers() {
        let leaf = [0x75; 32];
        let reviewed = [leaf];
        assert!(signer_seen_against_pins(true, Some(leaf), &reviewed));
        assert!(!signer_seen_against_pins(false, Some(leaf), &reviewed));
        assert!(!signer_seen_against_pins(true, None, &reviewed));
        assert!(!signer_seen_against_pins(true, Some([0; 32]), &[[0; 32]]));
        assert!(!signer_seen_against_pins(true, Some([0x76; 32]), &reviewed));
        // Even a pinned on-disk leaf does NOT establish the image section.
    }

    #[test]
    fn missing_native_process_capability_fails_before_file_or_trust_io() {
        assert_eq!(
            queried_image_path(ptr::null_mut()).unwrap_err().kind(),
            io::ErrorKind::InvalidInput,
        );
    }
}
