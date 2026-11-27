//! Reviewed standalone image only; never execute a UI-selected path or PATH entry.
use std::{fs::{File,OpenOptions},io::{self,Read},os::windows::fs::OpenOptionsExt,path::PathBuf,time::Duration};
use sha2::{Digest,Sha256};
use windows_sys::Win32::Storage::FileSystem::*;
use crate::engine::{process_identity::OwnedProcess,windows_file_identity};
use super::output::MAX_WEBP_BYTES;

// Deliberate package admission hold. Replace only after exact executable,
// dependency lock, native behavior and distribution review; not a user setting.
const REVIEWED_CAPTURE_WORKER_SHA256:Option<[u8;32]>=None;
const MAX_WORKER_BYTES:u64=128*1024*1024;

pub(super) fn available()->bool { REVIEWED_CAPTURE_WORKER_SHA256.is_some() }

struct Image { path:PathBuf, _file:File, _directory:File }
fn reviewed_image()->io::Result<Image>{
    let expected=REVIEWED_CAPTURE_WORKER_SHA256.ok_or_else(||io::Error::new(io::ErrorKind::Unsupported,"Capture worker approval pending"))?;
    let current=std::env::current_exe()?;
    let folder=current.parent().ok_or_else(||io::Error::other("Capture helper directory unavailable"))?;
    // Pin the existing executable directory and exact worker file. No reparse
    // points or file write/delete sharing during hash and child lifetime.
    let directory=OpenOptions::new().read(true).share_mode(FILE_SHARE_READ|FILE_SHARE_WRITE)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT|FILE_FLAG_BACKUP_SEMANTICS).open(folder)?;
    let attrs=windows_file_identity::attributes(&directory)?;
    if attrs&FILE_ATTRIBUTE_REPARSE_POINT!=0||attrs&FILE_ATTRIBUTE_DIRECTORY==0 {
        return Err(io::Error::other("Capture helper directory is not ordinary"));
    }
    let path=folder.join("lazy-tester-gpu-capture.exe");
    let mut file=OpenOptions::new().read(true).share_mode(FILE_SHARE_READ)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT).open(&path)?;
    if windows_file_identity::attributes(&file)?&(FILE_ATTRIBUTE_REPARSE_POINT|FILE_ATTRIBUTE_DIRECTORY)!=0 {
        return Err(io::Error::other("Capture helper is not an ordinary file"));
    }
    let length=file.metadata()?.len();
    if !(64..=MAX_WORKER_BYTES).contains(&length){return Err(io::Error::other("Capture helper size invalid"));}
    let mut hash=Sha256::new();
    let mut buffer=[0u8;16384];
    let mut total=0u64;
    loop {
        let count=file.read(&mut buffer)?;
        if count==0 {break;}
        total+=count as u64;
        if total>length {return Err(io::Error::other("Capture helper changed"));}
        hash.update(&buffer[..count]);
    }
    if total!=length||<[u8;32]>::from(hash.finalize())!=expected {
        return Err(io::Error::other("Capture helper digest not approved"));
    }
    Ok(Image{path,_file:file,_directory:directory})
}

pub(super) fn screenshot(target:&OwnedProcess)->io::Result<Vec<u8>>{
    let image=reviewed_image()?;
    let hwnd=target.window_handle.strip_prefix("0x")
        .and_then(|s|u64::from_str_radix(s,16).ok()).filter(|n|*n>0)
        .ok_or_else(||io::Error::other("Minecraft window unavailable"))?;
    let args=["--screenshot".to_owned(),"--pid".into(),target.pid.to_string(),
        "--window-id".into(),hwnd.to_string(),"--birth".into(),target.created_file_time.clone(),
        "--session".into(),target.session_id.to_string()];
    // Same bounded runner as existing manager helpers. Binary stdout never
    // becomes Base64, a log message, or a webview payload.
    let bytes=virtual_clients_core::bounded_capture_output(&image.path,args,Duration::from_secs(12))?;
    if bytes.len()>MAX_WEBP_BYTES {return Err(io::Error::other("Screenshot output exceeded budget"));}
    Ok(bytes)
}
