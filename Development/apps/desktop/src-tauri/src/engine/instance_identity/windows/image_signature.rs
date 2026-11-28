//! Read-only Authenticode verification of the pinned on-disk image, not mapped-image proof.
use super::{finish_file_signature_review, invalid, live, pin, ProcessKey, FreeLibrary};
use sha2::{Digest, Sha256};
use std::{io, mem::size_of, os::windows::{ffi::OsStringExt, fs::OpenOptionsExt, io::AsRawHandle}, ptr};
use windows_sys::Win32::{Foundation::HANDLE, Security::WinTrust::{WinVerifyTrust, WINTRUST_ACTION_GENERIC_VERIFY_V2, CRYPT_PROVIDER_CERT, CRYPT_PROVIDER_DATA, CRYPT_PROVIDER_SGNR, WINTRUST_DATA, WINTRUST_DATA_0, WINTRUST_FILE_INFO, WTD_CACHE_ONLY_URL_RETRIEVAL, WTD_CHOICE_FILE, WTD_REVOKE_WHOLECHAIN, WTD_STATEACTION_CLOSE, WTD_STATEACTION_VERIFY, WTD_UI_NONE}, System::LibraryLoader::{GetProcAddress, LoadLibraryExW, LOAD_LIBRARY_SEARCH_SYSTEM32}, Storage::FileSystem::{GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION, FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT, FILE_SHARE_READ}};

/// Inspect the queryable IMAGE FILE ON DISK for the exact live process.
/// A separate future component must bind signer's certificate + publisher,
/// verify PE section identity and check target Minecraft applicability.
///
/// hFile pins the specific opened file object used by WinVerifyTrust.
/// The caller cannot choose the path; no profile or credential is read.
/// A process executable pathname is still not cryptographic proof of
/// the section already mapped into that process (replacement/moves).
/// The signer certificate is borrowed from a successful verification state.
/// WinTrust owns every pointer until WTD_STATEACTION_CLOSE.
fn verified_leaf_sha256(state: HANDLE) -> io::Result<[u8; 32]> {
    type GetProvider = unsafe extern "system" fn(HANDLE) -> *mut CRYPT_PROVIDER_DATA;
    type GetSigner = unsafe extern "system" fn(
        *mut CRYPT_PROVIDER_DATA, u32, i32, u32,
    ) -> *mut CRYPT_PROVIDER_SGNR;
    type GetCertificate =
        unsafe extern "system" fn(*mut CRYPT_PROVIDER_SGNR, u32) -> *mut CRYPT_PROVIDER_CERT;

    struct Dll(*mut std::ffi::c_void);
    impl Drop for Dll {
        fn drop(&mut self) {
            if !self.0.is_null() {
                // A LoadLibraryExW-owned reference must be released once.
                let _ = unsafe { FreeLibrary(self.0) };
            }
        }
    }
    if state.is_null() {
        return Err(invalid("Verified WinTrust provider state is missing"));
    }
    let name: Vec<u16> = "wintrust.dll".encode_utf16().chain(Some(0)).collect();
    let dll = Dll(unsafe {
        LoadLibraryExW(name.as_ptr(), ptr::null_mut(), LOAD_LIBRARY_SEARCH_SYSTEM32)
    });
    if dll.0.is_null() { return Err(io::Error::last_os_error()); }
    // Windows documents these WTHelper exports as dynamically loaded only.
    // Do not search the application directory for a replacement DLL.
    let provider = unsafe {
        GetProcAddress(dll.0, b"WTHelperProvDataFromStateData\0".as_ptr())
    }.ok_or_else(|| invalid("WinTrust provider export unavailable"))?;
    let signer = unsafe {
        GetProcAddress(dll.0, b"WTHelperGetProvSignerFromChain\0".as_ptr())
    }.ok_or_else(|| invalid("WinTrust signer export unavailable"))?;
    let certificate = unsafe {
        GetProcAddress(dll.0, b"WTHelperGetProvCertFromChain\0".as_ptr())
    }.ok_or_else(|| invalid("WinTrust certificate export unavailable"))?;
    // FARPROC and WINAPI function pointers are word-sized on Windows.
    let get_provider: GetProvider = unsafe { std::mem::transmute(provider) };
    let get_signer: GetSigner = unsafe { std::mem::transmute(signer) };
    let get_certificate: GetCertificate = unsafe { std::mem::transmute(certificate) };
    let provider_data = unsafe { get_provider(state) };
    if provider_data.is_null() { return Err(invalid("WinTrust provider unavailable")); }
    let primary_signer = unsafe { get_signer(provider_data, 0, 0, 0) };
    if primary_signer.is_null() { return Err(invalid("WinTrust signer unavailable")); }
    // The primary signer and its first chain certificate must have
    // succeeded; never hash a failed/empty signer as a trusted leaf.
    if unsafe { (*primary_signer).dwError != 0 || (*primary_signer).csCertChain == 0 } {
        return Err(invalid("WinTrust primary signer chain is invalid"));
    }
    let leaf = unsafe { get_certificate(primary_signer, 0) };
    if leaf.is_null() { return Err(invalid("WinTrust signer leaf unavailable")); }
    if unsafe { (*leaf).dwError != 0 } {
        return Err(invalid("WinTrust leaf certificate failed provider checks"));
    }
    let context = unsafe { (*leaf).pCert };
    if context.is_null() { return Err(invalid("WinTrust leaf context missing")); }
    let encoded = unsafe { &*context };
    let count = encoded.cbCertEncoded as usize;
    if encoded.pbCertEncoded.is_null() || !(1..=65536).contains(&count) {
        return Err(invalid("WinTrust signer DER unavailable or oversized"));
    }
    let der = unsafe { std::slice::from_raw_parts(encoded.pbCertEncoded, count) };
    Ok(Sha256::digest(der).into())
}

pub(in crate::engine::instance_identity) fn inspect_file_authenticode(
    key: &ProcessKey,
) -> io::Result<(bool, Option<[u8; 32]>)> {
    use std::os::windows::io::AsRawHandle;

    fn file_identity(file: &std::fs::File) -> io::Result<(u32, u64, u64, u64)> {
        let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
        if unsafe {
            GetFileInformationByHandle(file.as_raw_handle(), &mut info)
        } == 0 {
            return Err(io::Error::last_os_error());
        }
        if info.dwFileAttributes & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT) != 0 {
            return Err(invalid("Image file is a directory or reparse object"));
        }
        let length = (u64::from(info.nFileSizeHigh) << 32) | u64::from(info.nFileSizeLow);
        if length == 0 {
            return Err(invalid("Empty executable image file"));
        }
        let index = (u64::from(info.nFileIndexHigh) << 32) | u64::from(info.nFileIndexLow);
        let write_time = (u64::from(info.ftLastWriteTime.dwHighDateTime) << 32)
            | u64::from(info.ftLastWriteTime.dwLowDateTime);
        Ok((info.dwVolumeSerialNumber, index, length, write_time))
    }

    let process = pin(key)?;
    let mut path = crate::engine::windows_process_image::query(process.as_raw_handle() as _)?;
    let pathname = std::path::PathBuf::from(std::ffi::OsString::from_wide(&path));
    let file = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .open(pathname)?;
    let before = file_identity(&file)?;
    // Reuse the queried UTF-16 buffer; retain the original non-NUL path for comparison.
    path.push(0);
    let mut file_info: WINTRUST_FILE_INFO = unsafe { std::mem::zeroed() };
    file_info.cbStruct = size_of::<WINTRUST_FILE_INFO>() as u32;
    file_info.pcwszFilePath = path.as_ptr();
    file_info.hFile = file.as_raw_handle();
    let mut data: WINTRUST_DATA = unsafe { std::mem::zeroed() };
    data.cbStruct = size_of::<WINTRUST_DATA>() as u32;
    data.dwUIChoice = WTD_UI_NONE;
    data.fdwRevocationChecks = WTD_REVOKE_WHOLECHAIN;
    data.dwUnionChoice = WTD_CHOICE_FILE;
    data.Anonymous = WINTRUST_DATA_0 { pFile: &mut file_info };
    data.dwStateAction = WTD_STATEACTION_VERIFY;
    // No HTTP/network revocation fetch; missing cached revocation may
    // make trust unavailable and must not be upgraded to positive.
    data.dwProvFlags = WTD_CACHE_ONLY_URL_RETRIEVAL;
    let mut policy = WINTRUST_ACTION_GENERIC_VERIFY_V2;
    let status = unsafe {
        WinVerifyTrust(ptr::null_mut(), &mut policy,
            (&mut data as *mut WINTRUST_DATA).cast())
    };
    let state_created = !data.hWVTStateData.is_null();
    // Extract only on success and always CLOSE, even if extraction fails.
    let signer_result = if status == 0 && state_created {
        verified_leaf_sha256(data.hWVTStateData).map(Some)
    } else {
        Ok(None)
    };
    data.dwStateAction = WTD_STATEACTION_CLOSE;
    // A failed signature may still allocate provider state. Close whenever
    // there is state to release, and reject a failed close independently
    // of the verification result.
    let close_succeeded = !state_created || unsafe {
        WinVerifyTrust(ptr::null_mut(), &mut policy,
            (&mut data as *mut WINTRUST_DATA).cast())
    } == 0;
    let reviewed = finish_file_signature_review(
        status == 0,
        state_created,
        close_succeeded,
        signer_result.map_err(|_| ()),
    ).map_err(invalid)?;
    let after = file_identity(&file)?;
    if before != after || crate::engine::windows_process_image::query(process.as_raw_handle() as _)?.as_slice() != &path[..path.len()-1] {
        return Err(invalid("On-disk process image changed during inspection"));
    }
    live(&process)?;
    // WinVerifyTrust returns LONG; ONLY zero is a successful verification.
    Ok(reviewed)
}
