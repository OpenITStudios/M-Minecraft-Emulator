//! Same-logon, same-image authenticated endpoint and original peer lifetime.
//! The parent pipe retains transport and worker ownership; no alternate endpoint.
use super::{own, raw, refused};
use sha2::{Digest, Sha256};
use std::{
    fs::{File, OpenOptions}, io, mem,
    os::windows::{ffi::OsStringExt, fs::OpenOptionsExt,
        io::{AsHandle, BorrowedHandle, FromRawHandle, OwnedHandle}},
    ptr, time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::{LocalFree, HANDLE, INVALID_HANDLE_VALUE, WAIT_OBJECT_0, WAIT_TIMEOUT},
    Security::{Authorization::{ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW}, *},
    Storage::FileSystem::*,
    System::{Diagnostics::ToolHelp::*, Pipes::*, Threading::*},
};

pub(super) fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}
// Win32 pipe queries require read attributes as well as data. Writes grant data only,
// not GENERIC_WRITE (which would also grant FILE_CREATE_PIPE_INSTANCE).
pub(super) const CLIENT_RIGHTS: u32 = FILE_GENERIC_READ | FILE_WRITE_DATA;

pub(super) struct Allocation(pub(super) *mut std::ffi::c_void);
impl Drop for Allocation {
    fn drop(&mut self) {
        unsafe {
            LocalFree(self.0);
        }
    }
}

fn token_data(token: HANDLE, class: TOKEN_INFORMATION_CLASS) -> io::Result<Vec<usize>> {
    let mut size = 0;
    unsafe { GetTokenInformation(token, class, ptr::null_mut(), 0, &mut size) };
    if size == 0 || size > 65536 {
        return Err(refused("Unsupported process token"));
    }
    let mut buffer = vec![0usize; (size as usize).div_ceil(mem::size_of::<usize>())];
    if unsafe { GetTokenInformation(token, class, buffer.as_mut_ptr().cast(), size, &mut size) }
        == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(buffer)
}
unsafe fn sid_bytes(sid: PSID) -> io::Result<Vec<u8>> {
    if sid.is_null() || IsValidSid(sid) == 0 {
        return Err(refused("Invalid process token SID"));
    }
    let size = GetLengthSid(sid) as usize;
    if !(8..=68).contains(&size) {
        return Err(refused("Unsupported process token SID"));
    }
    Ok(std::slice::from_raw_parts(sid.cast::<u8>(), size).to_vec())
}

#[derive(Clone, PartialEq, Eq)]
pub(super) struct Identity {
    pub(super) user: Vec<u8>,
    pub(super) logon: String,
    integrity: Vec<u8>,
    authentication: (u32, i32),
    pub(super) session: u32,
}
impl Identity {
    pub(super) fn process(process: HANDLE) -> io::Result<Self> {
        let mut token = ptr::null_mut();
        if unsafe { OpenProcessToken(process, TOKEN_QUERY, &mut token) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let token = own(token)?;
        let user = token_data(raw(&token), TokenUser)?;
        let groups = token_data(raw(&token), TokenLogonSid)?;
        let integrity = token_data(raw(&token), TokenIntegrityLevel)?;
        let statistics = token_data(raw(&token), TokenStatistics)?;
        let session = token_data(raw(&token), TokenSessionId)?;
        unsafe {
            let groups = &*groups.as_ptr().cast::<TOKEN_GROUPS>();
            if groups.GroupCount != 1 || groups.Groups[0].Attributes & 0xc0000000 != 0xc0000000 {
                return Err(refused("Process has no unique Windows logon SID"));
            }
            let logon = sid_bytes(groups.Groups[0].Sid)?;
            let mut text = ptr::null_mut();
            if ConvertSidToStringSidW(logon.as_ptr() as _, &mut text) == 0 {
                return Err(io::Error::last_os_error());
            }
            let _allocation = Allocation(text.cast());
            let mut len = 0;
            while len < 256 && *text.add(len) != 0 {
                len += 1;
            }
            if len == 256 {
                return Err(refused("Invalid logon SID text"));
            }
            let logon = String::from_utf16(std::slice::from_raw_parts(text, len))
                .map_err(|_| refused("Invalid logon SID text"))?;
            let statistics = &*statistics.as_ptr().cast::<TOKEN_STATISTICS>();
            Ok(Self {
                user: sid_bytes((*(user.as_ptr().cast::<TOKEN_USER>())).User.Sid)?,
                logon,
                integrity: sid_bytes(
                    (*(integrity.as_ptr().cast::<TOKEN_MANDATORY_LABEL>()))
                        .Label
                        .Sid,
                )?,
                authentication: (
                    statistics.AuthenticationId.LowPart,
                    statistics.AuthenticationId.HighPart,
                ),
                session: *(session.as_ptr().cast::<u32>()),
            })
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(super) struct ImageId {
    volume: u64,
    pub(super) file: [u8; 16],
}
fn image(path: &std::path::Path) -> io::Result<(File, ImageId)> {
    let file = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    if !crate::engine::windows_file_identity::attributes(&file).is_ok_and(|attributes|
        attributes & (FILE_ATTRIBUTE_REPARSE_POINT | FILE_ATTRIBUTE_DIRECTORY) == 0)
    {
        return Err(refused("Controller executable is not an ordinary file"));
    }
    let id = crate::engine::windows_file_identity::file_id(&file)?;
    Ok((
        file,
        ImageId {
            volume: id.VolumeSerialNumber,
            file: id.FileId.Identifier,
        },
    ))
}

pub(crate) struct Peer {
    pub(super) process: OwnedHandle,
    _image: File,
    pid:u32,
    pub(super) birth: u64,
}
impl Peer {
    // Original named-pipe endpoint already verified and pinned the CLI
    // process. No extra per-pulse process query or caller-provided token.
    #[cfg(feature="mc02-native-input")]
    pub(crate) fn input_sender(&self)->(u32,u64) {
        (self.pid,self.birth)
    }
    pub(crate) fn wait_handle(&self) -> BorrowedHandle<'_> {
        self.process.as_handle()
    }
    pub(crate) fn alive(&self) -> bool {
        // Original handle remains pinned; PID reuse cannot turn it into another peer.
        (unsafe { WaitForSingleObject(raw(&self.process), 0) == WAIT_TIMEOUT })
            && crate::engine::windows_process_time::creation_filetime(raw(&self.process)).is_ok_and(|birth| birth == self.birth)
    }
}

// Bind the Windows-observed direct Node parent of the verified one-shot CLI.
// This is a process handle/birth check, not MCP-script attestation or an
// authorization to take over an abandoned Testing run.
#[derive(Debug)]
pub(crate) struct McpOwnerLifetime {
    pub(super) process: OwnedHandle,
    pub(super) pid: u32,
    pub(super) birth: u64,
}
impl McpOwnerLifetime {
    pub(crate) fn same(&self, other: &Self) -> bool {
        self.pid == other.pid && self.birth == other.birth
    }
    pub(crate) fn status(&self) -> &'static str {
        let wait=unsafe {WaitForSingleObject(raw(&self.process),0)};
        if wait==WAIT_OBJECT_0 {return "BOUND_PARENT_EXIT_CONFIRMED";}
        if wait!=WAIT_TIMEOUT {return "BOUND_PARENT_STATUS_UNKNOWN";}
        if !crate::engine::windows_process_time::creation_filetime(raw(&self.process)).is_ok_and(|birth| birth == self.birth) {
            return "BOUND_PARENT_STATUS_UNKNOWN";
        }
        "BOUND_PARENT_RUNNING_AT_INSPECTION"
    }
}
impl Peer {
    pub(crate) fn mcp_parent(&self)->io::Result<McpOwnerLifetime> {
        if !self.alive() {return Err(refused("Testing CLI peer is no longer live"));}
        let child_pid=unsafe {GetProcessId(raw(&self.process))};
        if child_pid==0 {return Err(refused("Testing CLI identity unavailable"));}
        // ToolHelp parent attribution can be spoofed by a privileged
        // same-user caller; never treat it as a signed MCP identity.
        let snapshot=own(unsafe {CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS,0)})?;
        let mut entry:PROCESSENTRY32W=unsafe {mem::zeroed()};
        entry.dwSize=mem::size_of::<PROCESSENTRY32W>() as u32;
        if unsafe {Process32FirstW(raw(&snapshot),&mut entry)}==0 {
            return Err(io::Error::last_os_error());
        }
        let mut parent_pid=0;
        loop {
            if entry.th32ProcessID==child_pid {
                parent_pid=entry.th32ParentProcessID;
                break;
            }
            if unsafe {Process32NextW(raw(&snapshot),&mut entry)}==0 {break;}
        }
        if parent_pid==0||parent_pid==child_pid {
            return Err(refused("Testing MCP direct parent unavailable"));
        }
        let process=own(unsafe {OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZATION_SYNCHRONIZE,
            0,parent_pid)})?;
        if Identity::process(raw(&process))? !=
           Identity::process(unsafe {GetCurrentProcess()})? {
            return Err(refused("Testing MCP parent logon/session differs"));
        }
        let image_path=crate::engine::windows_process_image::query(raw(&process))?;
        let parent_path=std::ffi::OsString::from_wide(&image_path);
        if !std::path::Path::new(&parent_path).file_name()
           .and_then(|name|name.to_str())
           .is_some_and(|name|name.eq_ignore_ascii_case("node.exe")) {
            return Err(refused("Testing MCP direct parent is not Node"));
        }
        let birth = crate::engine::windows_process_time::creation_filetime(raw(&process))?;
        if birth==0||birth>self.birth {
            return Err(refused("Testing MCP parent creation is inconsistent"));
        }
        let owner=McpOwnerLifetime{process,pid:parent_pid,birth};
        if owner.status()!="BOUND_PARENT_RUNNING_AT_INSPECTION"||!self.alive() {
            return Err(refused("Testing MCP parent or CLI lifetime uncertain"));
        }
        Ok(owner)
    }
}

pub(crate) struct Endpoint {
    pub(super) name: Vec<u16>,
    pub(super) identity: Identity,
    pub(super) image: ImageId,
    _image: File,
}
impl Endpoint {
    pub(crate) fn current() -> io::Result<Self> {
        let identity = Identity::process(unsafe { GetCurrentProcess() })?;
        let (pinned, image) = image(&std::env::current_exe()?)?;
        let mut hash = Sha256::new();
        hash.update(identity.logon.as_bytes());
        hash.update(identity.session.to_le_bytes());
        hash.update(image.volume.to_le_bytes());
        hash.update(image.file);
        let digest: [u8; 32] = hash.finalize().into();
        let suffix = digest[..16]
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        Ok(Self {
            name: wide(&format!(r"\\.\pipe\LazyTester-v1-{suffix}")),
            identity,
            image,
            _image: pinned,
        })
    }
    pub(super) fn verify(&self, pipe: HANDLE, server_side: bool) -> io::Result<Peer> {
        let mut pid = 0;
        let get_peer = if server_side {
            GetNamedPipeClientProcessId
        } else {
            GetNamedPipeServerProcessId
        };
        if unsafe { get_peer(pipe, &mut pid) } == 0 || pid == 0 {
            return Err(refused("No Windows pipe peer"));
        }
        let process = own(unsafe {
            OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZATION_SYNCHRONIZE,
                0,
                pid,
            )
        })?;
        if Identity::process(raw(&process))? != self.identity {
            return Err(refused(
                "Controller peer user/logon/session/integrity differs",
            ));
        }
        let path = crate::engine::windows_process_image::query(raw(&process))?;
        let (pinned, identity) = image(std::path::Path::new(
            &std::ffi::OsString::from_wide(&path),
        ))?;
        if identity != self.image {
            return Err(refused("Controller peer executable differs"));
        }
        let birth = crate::engine::windows_process_time::creation_filetime(raw(&process))?;
        let mut repeated = 0;
        if birth == 0 || unsafe { get_peer(pipe, &mut repeated) } == 0 || pid != repeated {
            return Err(refused("Controller peer identity changed"));
        }
        let peer = Peer {
            process,
            _image: pinned,
            pid,
            birth,
        };
        if !peer.alive() {
            return Err(refused("Controller peer already exited"));
        }
        Ok(peer)
    }
    pub(crate) fn connect(&self) -> io::Result<File> {
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            // Exact pipe rights; generic file-open helpers may add unrelated filesystem rights.
            let handle = unsafe {
                CreateFileW(
                    self.name.as_ptr(),
                    windows_sys::Win32::Foundation::GENERIC_READ | FILE_WRITE_DATA,
                    0,
                    ptr::null(),
                    OPEN_EXISTING,
                    SECURITY_SQOS_PRESENT | SECURITY_IDENTIFICATION,
                    ptr::null_mut(),
                )
            };
            let result = if handle == INVALID_HANDLE_VALUE {
                Err(io::Error::last_os_error())
            } else {
                Ok(unsafe { File::from_raw_handle(handle as _) })
            };
            match result {
                Err(error) if error.raw_os_error() == Some(231) && Instant::now() < deadline => unsafe {
                    WaitNamedPipeW(self.name.as_ptr(), 100);
                },
                other => return other,
            }
        }
    }
    pub(super) fn bind(&self) -> io::Result<File> {
        let sddl = wide(&format!(
            "D:P(A;;0x{CLIENT_RIGHTS:08x};;;{})",
            self.identity.logon
        ));
        let mut descriptor = ptr::null_mut();
        if unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                sddl.as_ptr(),
                1,
                &mut descriptor,
                ptr::null_mut(),
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        let _descriptor = Allocation(descriptor);
        let attributes = SECURITY_ATTRIBUTES {
            nLength: mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: descriptor,
            bInheritHandle: 0,
        };
        let handle = unsafe {
            CreateNamedPipeW(
                self.name.as_ptr(),
                PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE | FILE_FLAG_OVERLAPPED,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_REJECT_REMOTE_CLIENTS,
                1,
                65536,
                65536,
                1000,
                &attributes,
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        Ok(unsafe { File::from_raw_handle(handle as _) })
    }
}
