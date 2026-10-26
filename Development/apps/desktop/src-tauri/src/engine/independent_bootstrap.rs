//! Private first stage of the Education bootstrap experiment.
//! Explicit same-manager startup observation only; no provider admission.
//! Exact development payload approved for one startup experiment by the operator.
//! Tests keep the game policy empty; direct-start game compatibility remains UNKNOWN.
#![allow(dead_code)]

use crate::engine::instance_focus_events;
use std::{
    ffi::OsStr,
    fs::{File, OpenOptions},
    io, mem,
    os::windows::{
        ffi::OsStrExt,
        fs::OpenOptionsExt,
        io::{AsHandle, AsRawHandle, FromRawHandle, OwnedHandle},
    },
    path::Path,
    ptr,
};
use windows_sys::Win32::{
    Foundation::{FILETIME, HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT},
    Storage::FileSystem::{
        FileAttributeTagInfo, FileIdInfo, GetFileInformationByHandleEx, FILE_ATTRIBUTE_DIRECTORY,
        FILE_ATTRIBUTE_REPARSE_POINT, FILE_ATTRIBUTE_TAG_INFO, FILE_ID_INFO, FILE_SHARE_READ,
    },
    System::RemoteDesktop::ProcessIdToSessionId,
    System::Threading::{
        CreateProcessW, GetCurrentProcessId, GetProcessId, GetProcessIdOfThread, GetProcessTimes,
        IsWow64Process2, QueryFullProcessImageNameW, TerminateProcess, WaitForSingleObject,
        CREATE_SUSPENDED, CREATE_UNICODE_ENVIRONMENT, PROCESS_INFORMATION, STARTUPINFOW,
    },
};

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn wide(value: &OsStr) -> io::Result<Vec<u16>> {
    let mut result: Vec<u16> = value.encode_wide().collect();
    if result.is_empty()
        || result.len() > 30_000
        || result.contains(&0)
        || result.contains(&(b'"' as u16))
    {
        return Err(invalid("Invalid bootstrap executable path"));
    }
    result.push(0);
    Ok(result)
}

fn image_id(file: &File) -> io::Result<FILE_ID_INFO> {
    let handle = file.as_raw_handle() as HANDLE;
    let mut tag: FILE_ATTRIBUTE_TAG_INFO = unsafe { mem::zeroed() };
    let mut id: FILE_ID_INFO = unsafe { mem::zeroed() };
    // SAFETY: File owns the handle and both output buffers have their exact ABI sizes.
    if unsafe {
        GetFileInformationByHandleEx(
            handle,
            FileAttributeTagInfo,
            &mut tag as *mut _ as _,
            mem::size_of_val(&tag) as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    if tag.FileAttributes & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT) != 0 {
        return Err(invalid("Bootstrap image must be an ordinary file"));
    }
    if unsafe {
        GetFileInformationByHandleEx(
            handle,
            FileIdInfo,
            &mut id as *mut _ as _,
            mem::size_of_val(&id) as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(id)
}

fn same_image(a: &FILE_ID_INFO, b: &FILE_ID_INFO) -> bool {
    a.VolumeSerialNumber == b.VolumeSerialNumber && a.FileId.Identifier == b.FileId.Identifier
}

fn pin_image(path: &Path) -> io::Result<File> {
    // Deny write/delete sharing while verification and process creation use this image.
    let file = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .open(path)?;
    image_id(&file)?;
    Ok(file)
}

/// Owns original CreateProcessW handles, or verified duplicates in the fixed cfg(test) handoff.
// Per-attempt cleanup receipt, shared only with the original unstarted child.
// It never grants permission after a resume attempt or unconfirmed cancellation.
#[derive(Clone)]
struct UnstartedCleanup(std::sync::Arc<std::sync::atomic::AtomicBool>);
impl UnstartedCleanup {
    fn new() -> Self {
        Self(std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true)))
    }
    fn retry_safe(&self) -> bool {
        self.0.load(std::sync::atomic::Ordering::Acquire)
    }
    fn pending(&self) {
        self.0.store(false, std::sync::atomic::Ordering::Release);
    }
    fn confirmed(&self) {
        self.0.store(true, std::sync::atomic::Ordering::Release);
    }
}

/// Cancellation is allowed only before the first resume attempt; no PID-reopen API.
pub(super) struct PendingEducationBootstrap {
    process: OwnedHandle,
    _thread: OwnedHandle,
    _image: File,
    pid: u32,
    created: u64,
    storage: Option<SlotStorage>,
    configuration: Option<OwnedHandle>,
    observation: Option<StartupObservation>,
    policy: Option<instance_focus_events::PolicyChannel>,
    resume_attempted: bool,
    cleanup: Option<UnstartedCleanup>,
}

impl PendingEducationBootstrap {
    pub(super) fn identity(&self) -> io::Result<(u32, u64)> {
        let process = self.process.as_raw_handle() as HANDLE;
        let mut created: FILETIME = unsafe { mem::zeroed() };
        let mut exited: FILETIME = unsafe { mem::zeroed() };
        let mut kernel: FILETIME = unsafe { mem::zeroed() };
        let mut user: FILETIME = unsafe { mem::zeroed() };
        // SAFETY: Query only the retained newly created process, never an existing PID.
        if unsafe { GetProcessTimes(process, &mut created, &mut exited, &mut kernel, &mut user) }
            == 0
        {
            return Err(io::Error::last_os_error());
        }
        let birth = ((created.dwHighDateTime as u64) << 32) | created.dwLowDateTime as u64;
        if unsafe { GetProcessId(process) } != self.pid
            || birth == 0
            || (self.created != 0 && self.created != birth)
        {
            return Err(invalid("Bootstrap process identity mismatch"));
        }
        match unsafe { WaitForSingleObject(process, 0) } {
            WAIT_TIMEOUT => Ok((self.pid, birth)),
            WAIT_OBJECT_0 => Err(invalid("Bootstrap process exited before enrollment")),
            _ => Err(io::Error::last_os_error()),
        }
    }

    pub(super) fn cancel(&mut self) -> io::Result<()> {
        if self.resume_attempted {
            return Err(invalid("A resumed or uncertain child must be preserved"));
        }
        let process = self.process.as_raw_handle() as HANDLE;
        // SAFETY: Only an engine-created child with no resume attempt is cancellable.
        match unsafe { WaitForSingleObject(process, 0) } {
            WAIT_OBJECT_0 => return Ok(()),
            WAIT_TIMEOUT => {}
            _ => return Err(io::Error::last_os_error()),
        }
        if unsafe { TerminateProcess(process, 90) } == 0
            && unsafe { WaitForSingleObject(process, 0) } != WAIT_OBJECT_0
        {
            return Err(io::Error::last_os_error());
        }
        if unsafe { WaitForSingleObject(process, 5_000) } != WAIT_OBJECT_0 {
            return Err(invalid("Unstarted bootstrap child cleanup is unconfirmed"));
        }
        Ok(())
    }
}

impl Drop for PendingEducationBootstrap {
    fn drop(&mut self) {
        if !self.resume_attempted {
            match self.cancel() {
                Ok(()) => {
                    if let Some(receipt) = &self.cleanup {
                        receipt.confirmed();
                    }
                }
                Err(_) => eprintln!("Unstarted Education bootstrap child cleanup is unconfirmed"),
            }
        }
    }
}

fn interactive_session(pid: u32) -> io::Result<u32> {
    let mut session = 0;
    // SAFETY: Windows writes one u32; no process/token/account contents are read.
    if unsafe { ProcessIdToSessionId(pid, &mut session) } == 0 {
        return Err(io::Error::last_os_error());
    }
    if session == 0 {
        return Err(invalid("Interactive bootstrap session required"));
    }
    Ok(session)
}

// Private to the registered-target entry point and this module's own-executable tests.
fn create_unstarted(
    path: &Path,
    image: File,
    storage: Option<SlotStorage>,
) -> io::Result<PendingEducationBootstrap> {
    create_unstarted_impl(
        path,
        image,
        storage,
        false,
        #[cfg(test)]
        None,
    )
}

fn create_unstarted_impl(
    path: &Path,
    image: File,
    storage: Option<SlotStorage>,
    observe: bool,
    #[cfg(test)] proof: Option<&tests::FixtureReceipt>,
) -> io::Result<PendingEducationBootstrap> {
    create_unstarted_configured(
        path,
        image,
        storage,
        observe,
        None,
        #[cfg(test)]
        proof,
    )
}

// A private caller supplies the already-pinned payload to opt into event pacing.
// Only the reviewed registered replay opts in; plain preparation stays unpaced.
fn create_unstarted_configured(
    path: &Path,
    image: File,
    storage: Option<SlotStorage>,
    observe: bool,
    presentation_library: Option<&BootstrapLibrary>,
    #[cfg(test)] proof: Option<&tests::FixtureReceipt>,
) -> io::Result<PendingEducationBootstrap> {
    create_unstarted_tracked(path, image, storage, observe, presentation_library, None,
        #[cfg(test)] proof)
}

fn create_unstarted_tracked(
    path: &Path,
    image: File,
    storage: Option<SlotStorage>,
    observe: bool,
    presentation_library: Option<&BootstrapLibrary>,
    cleanup: Option<UnstartedCleanup>,
    #[cfg(test)] proof: Option<&tests::FixtureReceipt>,
) -> io::Result<PendingEducationBootstrap> {
    if presentation_library.is_some() && storage.is_none() {
        return Err(invalid("Presentation policy requires configured storage"));
    }
    if observe && storage.is_none() {
        return Err(invalid("Startup observation requires configured storage"));
    }
    #[cfg(test)]
    if proof.is_some() && storage.is_none() {
        return Err(invalid("Fixture receipt requires configured storage"));
    }
    if !path.is_absolute() {
        return Err(invalid("Absolute bootstrap image required"));
    }
    let expected = image_id(&image)?;
    let application = wide(path.as_os_str())?;
    let directory = wide(
        path.parent()
            .ok_or_else(|| invalid("Missing image directory"))?
            .as_os_str(),
    )?;
    let session = interactive_session(unsafe { GetCurrentProcessId() })?;
    let mut command = vec![b'"' as u16];
    command.extend_from_slice(&application[..application.len() - 1]);
    command.extend_from_slice(&[b'"' as u16, 0]);
    let configuration = storage
        .as_ref()
        .map(|_| Configuration::create())
        .transpose()?;
    let presentation = presentation_library
        .map(|_| Configuration::create_sized(120))
        .transpose()?;
    let observation = if observe {
        Some(StartupObservation::create()?)
    } else {
        None
    };
    let mut inherited: Vec<HANDLE> = configuration
        .as_ref()
        .map(|c| vec![c.readonly.as_raw_handle() as HANDLE])
        .unwrap_or_default();
    if let Some(observation) = &observation {
        inherited.push(observation.section.as_raw_handle() as HANDLE);
    }
    if let Some(config) = &presentation {
        inherited.push(config.readonly.as_raw_handle() as HANDLE);
    }
    #[cfg(test)]
    let inherited = {
        let mut handles = inherited;
        if let Some(proof) = proof {
            handles.push(proof.0.as_raw_handle() as HANDLE);
        }
        handles
    };
    let attributes = if inherited.is_empty() {
        None
    } else {
        Some(InheritList::new(&inherited)?)
    };
    let environment = match (&storage, &configuration) {
        (Some(store), Some(config)) => Some(child_environment(
            store,
            config.readonly.as_raw_handle() as HANDLE,
            observation
                .as_ref()
                .map(|o| o.section.as_raw_handle() as HANDLE),
            presentation
                .as_ref()
                .map(|c| c.readonly.as_raw_handle() as HANDLE),
        )?),
        _ => None,
    };
    #[cfg(test)]
    let environment = match (environment, proof) {
        (Some(environment), Some(proof)) => Some(proof.environment(environment)?),
        (environment, _) => environment,
    };
    let mut startup: STARTUPINFOEXW = unsafe { mem::zeroed() };
    startup.StartupInfo.cb = if configuration.is_some() {
        mem::size_of::<STARTUPINFOEXW>() as u32
    } else {
        mem::size_of::<STARTUPINFOW>() as u32
    };
    startup.lpAttributeList = attributes.as_ref().map_or(ptr::null_mut(), |a| a.list);
    let mut info: PROCESS_INFORMATION = unsafe { mem::zeroed() };
    if let Some(receipt) = &cleanup { receipt.pending(); }
    // SAFETY: Absolute lpApplicationName; writable quoted argv[0] only; no shell,
    // arbitrary arguments, ambient handle inheritance, custom token, elevation or breakaway fallback.
    if unsafe {
        CreateProcessW(
            application.as_ptr(),
            command.as_mut_ptr(),
            ptr::null(),
            ptr::null(),
            i32::from(configuration.is_some()),
            CREATE_SUSPENDED
                | CREATE_UNICODE_ENVIRONMENT
                | if configuration.is_some() {
                    EXTENDED_STARTUPINFO_PRESENT
                } else {
                    0
                },
            environment
                .as_ref()
                .map_or(ptr::null(), |e| e.as_ptr() as _),
            directory.as_ptr(),
            &startup.StartupInfo,
            &mut info,
        )
    } == 0
    {
        let error = io::Error::last_os_error();
        // Failed CreateProcess returns no owned child. Later failures need a
        // confirmed exit from the retained never-resumed process before retry.
        if let Some(receipt) = &cleanup { receipt.confirmed(); }
        return Err(error);
    }
    // SAFETY: A successful CreateProcessW returns two newly owned valid handles.
    let mut child = PendingEducationBootstrap {
        process: unsafe { OwnedHandle::from_raw_handle(info.hProcess as _) },
        _thread: unsafe { OwnedHandle::from_raw_handle(info.hThread as _) },
        _image: image,
        pid: info.dwProcessId,
        created: 0,
        storage: None,
        configuration: None,
        observation,
        policy: None,
        resume_attempted: false,
        cleanup,
    };
    #[cfg(test)]
    if let Some(proof) = proof {
        if unsafe {
            SetHandleInformation(proof.0.as_raw_handle() as HANDLE, HANDLE_FLAG_INHERIT, 0)
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
    }
    if let Some(observation) = &child.observation {
        if unsafe {
            SetHandleInformation(
                observation.section.as_raw_handle() as HANDLE,
                HANDLE_FLAG_INHERIT,
                0,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
    }
    child.created = child.identity()?.1;
    if unsafe { GetProcessIdOfThread(child._thread.as_raw_handle() as HANDLE) } != child.pid
        || interactive_session(child.pid)? != session
    {
        return Err(invalid("Bootstrap primary thread/session mismatch"));
    }
    let mut machine = 0;
    let mut native = 0;
    if unsafe {
        IsWow64Process2(
            child.process.as_raw_handle() as HANDLE,
            &mut machine,
            &mut native,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    if machine != 0 || native != 0x8664 {
        return Err(invalid("Only native AMD64 bootstrap is supported"));
    }
    let mut actual = [0u16; 32768];
    let mut count = actual.len() as u32;
    if unsafe {
        QueryFullProcessImageNameW(
            child.process.as_raw_handle() as HANDLE,
            0,
            actual.as_mut_ptr(),
            &mut count,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    use std::{ffi::OsString, os::windows::ffi::OsStringExt};
    let actual_path = std::path::PathBuf::from(OsString::from_wide(&actual[..count as usize]));
    if !same_image(&expected, &image_id(&pin_image(&actual_path)?)?) {
        return Err(invalid(
            "Created bootstrap process image does not match pinned target",
        ));
    }
    if let (Some(config), Some(store)) = (configuration, storage) {
        child.configuration = Some(config.publish(&child, &store)?);
        child.storage = Some(store);
    }
    if let (Some(config), Some(library)) = (&presentation, presentation_library) {
        if unsafe {
            SetHandleInformation(
                config.readonly.as_raw_handle() as HANDLE,
                HANDLE_FLAG_INHERIT,
                0,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        child.policy = Some(instance_focus_events::register_policy(
            child.process.as_handle(),
            child.pid,
            child.created,
        )?);
        library.prepare_presentation(&child, config)?;
    }
    child.identity()?;
    Ok(child)
}

const MAX_EDUCATION_SLOTS: u32 = 5;

#[path = "independent_bootstrap/game_preset.rs"]
mod game_preset;

#[path = "independent_bootstrap/session_controller.rs"]
mod session_controller;
#[path = "independent_bootstrap/profile_storage.rs"]
mod profile_storage;
#[path = "independent_bootstrap/profile_binding.rs"]
pub(super) mod profile_binding;
#[path = "independent_bootstrap/product_profiles.rs"]
pub(super) mod product_profiles;

/// Source-only two-slot preparation. No game/DLL code can be resumed here.
/// The existing registered-image resolver remains the only target selector.
pub(super) fn prepare_registered() -> io::Result<[PendingEducationBootstrap; 2]> {
    prepare_registered_mode(false, None)
}
fn prepare_registered_mode<const N: usize>(
    observe: bool,
    presentation_library: Option<&BootstrapLibrary>,
) -> io::Result<[PendingEducationBootstrap; N]> {
    if N != 2 && N != 5 {
        return Err(invalid("Unsupported Education slot count"));
    }
    if presentation_library.is_some() && !observe {
        return Err(invalid(
            "Registered presentation requires reviewed profile replay",
        ));
    }
    let (target, image) = registered_image(observe)?;
    let path = Path::new(&target);
    let stores = registered_slots::<N>(observe)?;
    if observe {
        let changed = game_preset::apply(&stores)?;
        use std::io::Write;
        writeln!(io::stdout(), "{{\"event\":\"graphics-preset\",\"preset\":\"Simple12\",\"changedProfiles\":{changed}}}")?;
    }
    let mut children = Vec::with_capacity(N);
    for mut store in stores {
        if observe {
            store.singleton_caller_rva = 0x2cdc30;
        }
        children.push(create_unstarted_configured(
            path,
            image.try_clone()?,
            Some(store),
            observe,
            presentation_library,
            #[cfg(test)]
            None,
        )?);
    }
    children.try_into().map_err(|_| invalid("Incomplete Education process set"))
}

fn registered_image(observe: bool) -> io::Result<(String, File)> {
    let selector = "Resolve-ManagerEducationTarget -RegisteredOnly";
    let target = super::resolve_registered_education_target(selector)?;
    let path = Path::new(&target);
    let image = pin_image(path)?;
    if super::resolve_registered_education_target(selector)? != target {
        return Err(invalid("Education registration changed during preparation"));
    }
    // The scoped experiment is bound to the actually observed official image bytes/callsite.
    // An update refuses before allocating profiles; publisher identity alone cannot authorize an RVA.
    let expected = [
        0x04, 0x16, 0x4c, 0x1c, 0x64, 0x45, 0x8a, 0x5d, 0x98, 0x4d, 0xbc, 0x16, 0x87, 0xd0, 0xe8,
        0x02, 0x3d, 0x18, 0x08, 0xc4, 0x4e, 0xe7, 0xbe, 0x03, 0x51, 0x9c, 0xdf, 0x83, 0x55, 0x23,
        0x3e, 0xaa,
    ];
    if observe && file_digest(&image, 256 * 1024 * 1024)? != expected {
        return Err(invalid(
            "Education image changed; singleton recipe requires review",
        ));
    }
    Ok((target, image))
}

const OBSERVATION_ENV: &str = "MIVUBI_EDUCATION_OBSERVATION_SECTION";
// Must match private startup-observation.h; no public wire schema.
const OBSERVATION_BYTES: usize = 31968;
struct StartupObservation {
    section: OwnedHandle,
}
impl StartupObservation {
    fn create() -> io::Result<Self> {
        let raw = unsafe {
            CreateFileMappingW(
                INVALID_HANDLE_VALUE,
                ptr::null(),
                PAGE_READWRITE,
                0,
                OBSERVATION_BYTES as u32,
                ptr::null(),
            )
        };
        if raw.is_null() {
            return Err(io::Error::last_os_error());
        }
        let section = unsafe { OwnedHandle::from_raw_handle(raw as _) };
        if unsafe { SetHandleInformation(raw, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self { section })
    }
}
impl PendingEducationBootstrap {
    // Raw diagnostic metadata remains private/local; copy only after the original child exits.
    fn observation_path(&self) -> io::Result<PathBuf> {
        if self.created == 0 {
            return Err(invalid("Missing observation process birth"));
        }
        let store = self
            .storage
            .as_ref()
            .ok_or_else(|| invalid("Missing observation storage"))?;
        Ok(store
            .private_path
            .join(format!("startup-observation-v3-{:016x}.bin", self.created)))
    }

    fn observation_bytes_after_exit(&self) -> io::Result<Vec<u8>> {
        match unsafe { WaitForSingleObject(self.process.as_raw_handle() as HANDLE, 0) } {
            WAIT_OBJECT_0 => {}
            WAIT_TIMEOUT => {
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "Observation writer is still alive",
                ))
            }
            _ => return Err(io::Error::last_os_error()),
        }
        let observation = self
            .observation
            .as_ref()
            .ok_or_else(|| invalid("No startup observation"))?;
        let view = unsafe {
            MapViewOfFile(
                observation.section.as_raw_handle() as HANDLE,
                FILE_MAP_READ,
                0,
                0,
                OBSERVATION_BYTES,
            )
        };
        if view.Value.is_null() {
            return Err(io::Error::last_os_error());
        }
        let bytes =
            unsafe { std::slice::from_raw_parts(view.Value as *const u8, OBSERVATION_BYTES) }
                .to_vec();
        if unsafe { UnmapViewOfFile(view) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let word =
            |offset: usize| u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        let birth = u64::from_le_bytes(bytes[16..24].try_into().unwrap());
        if word(0) != 0x4f425331
            || word(4) != 3
            || word(8) != OBSERVATION_BYTES as u32
            || word(12) != self.pid
            || birth != self.created
            || word(28) != 64
            || word(48) > 64
            || word(56) != 30000
            || word(60) != 1
        {
            return Err(invalid("Incomplete or mismatched startup observation"));
        }
        Ok(bytes)
    }
}

// Fixed development bytes authorized for the explicit startup mode only, not packaging/admission.
// Tests must never turn --include-ignored into game execution.
#[cfg(test)]
const REVIEWED_EDUCATION_BOOTSTRAP: Option<PayloadDigests> = None;
#[cfg(not(test))]
const REVIEWED_EDUCATION_BOOTSTRAP: Option<PayloadDigests> = Some(PayloadDigests {
    enroller: [
        0x6e, 0x7c, 0x00, 0x0b, 0xee, 0x2d, 0xcf, 0x04, 0x3d, 0x37, 0x23, 0x17, 0xc7, 0x7a, 0x04, 0x07, 0x8c, 0xa1, 0x24, 0x51, 0xc7, 0x27, 0x3a, 0x18, 0xbd, 0x54, 0xe8, 0xf8, 0x9e, 0x62, 0xaf, 0x2a,
    ],
    compatibility: [
        0x9d, 0x1c, 0xf6, 0x06, 0x73, 0xc1, 0xeb, 0xbc, 0x0d, 0x94, 0xc3, 0x0c, 0x1e, 0x28, 0xe2, 0x26, 0xaa, 0x44, 0x29, 0x65, 0x39, 0x01, 0xe5, 0xab, 0xd6, 0xe9, 0x54, 0x00, 0x45, 0x7e, 0xf8, 0xcb,
    ],
});
#[derive(Clone, Copy)]
struct PayloadDigests {
    enroller: [u8; 32],
    compatibility: [u8; 32],
}
type EnrollmentEntry =
    unsafe extern "system" fn(HANDLE, HANDLE, u32, u64, HANDLE, HANDLE, HANDLE) -> u32;

fn payload_digest(file: &File) -> io::Result<[u8; 32]> {
    file_digest(file, 32 * 1024 * 1024)
}

fn file_digest(file: &File, maximum: usize) -> io::Result<[u8; 32]> {
    use sha2::{Digest, Sha256};
    use std::io::{Read, Seek, SeekFrom};
    let mut reader = file.try_clone()?;
    reader.seek(SeekFrom::Start(0))?;
    let mut hash = Sha256::new();
    let mut total = 0usize;
    let mut buffer = [0u8; 16384];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        total += count;
        if total > maximum {
            return Err(invalid("Bootstrap file exceeds reviewed size bound"));
        }
        hash.update(&buffer[..count]);
    }
    if total < 64 {
        return Err(invalid("Bootstrap DLL is incomplete"));
    }
    Ok(hash.finalize().into())
}

struct BootstrapLibrary {
    module: windows_sys::Win32::Foundation::HMODULE,
    entry: EnrollmentEntry,
    _files: [File; 2],
    _directories: Vec<File>,
}
impl BootstrapLibrary {
    fn reviewed() -> io::Result<Self> {
        let digests = REVIEWED_EDUCATION_BOOTSTRAP.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::Unsupported,
                "No reviewed Education bootstrap payload",
            )
        })?;
        let executable = std::env::current_exe()?;
        let root = executable
            .parent()
            .ok_or_else(|| invalid("Missing bootstrap payload parent"))?
            .join("independent-bootstrap");
        Self::load(&root, digests)
    }

    // Only the fixed reviewed production location or cfg(test) build directory can call this.
    fn load(root: &Path, digests: PayloadDigests) -> io::Result<Self> {
        use windows_sys::Win32::System::LibraryLoader::{
            LoadLibraryExW, LOAD_LIBRARY_SEARCH_SYSTEM32,
        };
        if !root.is_absolute()
            || root.components().any(|component| {
                matches!(
                    component,
                    std::path::Component::ParentDir | std::path::Component::CurDir
                )
            })
        {
            return Err(invalid("Absolute ordinary bootstrap directory required"));
        }
        // Hold every ancestor without delete sharing: reject reparses before loading code.
        let mut directories = Vec::new();
        for ancestor in root.ancestors().collect::<Vec<_>>().into_iter().rev() {
            directories.push(open_directory(ancestor, FILE_READ_ATTRIBUTES)?);
        }
        let root = directory_path(
            directories
                .last()
                .ok_or_else(|| invalid("Missing payload root"))?,
        )?;
        let library = root.join("bootstrap-enroll.dll");
        let files = [
            pin_image(&library)?,
            pin_image(&root.join("bootstrap-compat.dll"))?,
        ];
        if payload_digest(&files[0])? != digests.enroller
            || payload_digest(&files[1])? != digests.compatibility
        {
            return Err(invalid("Bootstrap DLL bytes do not match reviewed payload"));
        }
        let name = wide(library.as_os_str())?;
        let module =
            unsafe { LoadLibraryExW(name.as_ptr(), ptr::null_mut(), LOAD_LIBRARY_SEARCH_SYSTEM32) };
        if module.is_null() {
            return Err(io::Error::last_os_error());
        }
        let checked = (|| {
            let mut loaded = [0u16; 512];
            let count = unsafe {
                windows_sys::Win32::System::LibraryLoader::GetModuleFileNameW(
                    module,
                    loaded.as_mut_ptr(),
                    loaded.len() as u32,
                )
            } as usize;
            if count == 0 || count >= loaded.len() {
                return Err(invalid("Cannot bind loaded bootstrap module"));
            }
            let loaded = PathBuf::from(OsString::from_wide(&loaded[..count]));
            if !same_image(&image_id(&files[0])?, &image_id(&pin_image(&loaded)?)?) {
                return Err(invalid("Loaded bootstrap module file identity mismatch"));
            }
            let entry =
                unsafe { GetProcAddress(module, c"EnrollPreparedEducationV2".as_ptr().cast()) }
                    .ok_or_else(|| invalid("Missing bootstrap enrollment ABI v2"))?;
            Ok(unsafe {
                mem::transmute::<unsafe extern "system" fn() -> isize, EnrollmentEntry>(entry)
            })
        })();
        match checked {
            Ok(entry) => Ok(Self {
                module,
                entry,
                _files: files,
                _directories: directories,
            }),
            Err(error) => {
                unsafe {
                    windows_sys::Win32::Foundation::FreeLibrary(module);
                }
                Err(error)
            }
        }
    }

    fn prepare_presentation(
        &self,
        child: &PendingEducationBootstrap,
        section: &Configuration,
    ) -> io::Result<()> {
        if child.resume_attempted {
            return Err(invalid("Presentation requires original unstarted child"));
        }
        let (pid, birth) = child.identity()?;
        let storage = child
            .configuration
            .as_ref()
            .ok_or_else(|| invalid("Presentation requires storage"))?;
        let inputs = child
            .policy
            .as_ref()
            .ok_or_else(|| invalid("Presentation lease missing"))?
            .reader_handles()?;
        let handles = inputs.each_ref().map(|h| h.as_raw_handle() as HANDLE);
        type Entry = unsafe extern "system" fn(
            HANDLE,
            HANDLE,
            u32,
            u64,
            HANDLE,
            HANDLE,
            HANDLE,
            *const HANDLE,
            u32,
            u32,
        ) -> u32;
        let limits = crate::engine::manager_settings::load_performance()?;
        let symbol = unsafe {
            GetProcAddress(
                self.module,
                c"PrepareEducationPresentationV3".as_ptr().cast(),
            )
        }
        .ok_or_else(|| invalid("Pinned payload lacks presentation capability preparation"))?;
        let entry: Entry = unsafe { mem::transmute(symbol) };
        let status = unsafe {
            entry(
                child.process.as_raw_handle() as HANDLE,
                child._thread.as_raw_handle() as HANDLE,
                pid,
                birth,
                storage.as_raw_handle() as HANDLE,
                child._image.as_raw_handle() as HANDLE,
                section.writable.as_raw_handle() as HANDLE,
                handles.as_ptr(),
                limits.active_fps,
                limits.background_fps,
            )
        };
        if status != 0 {
            return Err(io::Error::from_raw_os_error(status as i32));
        }
        child.identity()?;
        Ok(())
    }

    fn enroll(&self, child: &PendingEducationBootstrap) -> io::Result<()> {
        let (pid, birth) = child.identity()?;
        let config = child
            .configuration
            .as_ref()
            .ok_or_else(|| invalid("Enrollment requires published child configuration"))?;
        let status = unsafe {
            (self.entry)(
                child.process.as_raw_handle() as HANDLE,
                child._thread.as_raw_handle() as HANDLE,
                pid,
                birth,
                config.as_raw_handle() as HANDLE,
                child._image.as_raw_handle() as HANDLE,
                child
                    .observation
                    .as_ref()
                    .map_or(ptr::null_mut(), |o| o.section.as_raw_handle() as HANDLE),
            )
        };
        if status != 0 {
            return Err(io::Error::from_raw_os_error(status as i32));
        }
        child.identity()?;
        Ok(())
    }
}
impl Drop for BootstrapLibrary {
    fn drop(&mut self) {
        if unsafe { windows_sys::Win32::Foundation::FreeLibrary(self.module) } == 0 {
            eprintln!("Bootstrap module cleanup is unconfirmed");
        }
    }
}

/// Original children are cancelled before DLL/directory pins are released (field drop order).
/// Only the explicit observation transaction below can attempt resume; no public admission.
pub(super) struct PreparedEducationEnrollment<const N: usize = 2> {
    _children: [PendingEducationBootstrap; N],
    _payload: BootstrapLibrary,
}

/// Per-slot startup result, never game readiness. Unexpected suspend counts are not retried.
#[derive(Debug)]
pub(super) enum StartupAttempt {
    NotAttempted,
    Resumed,
    Unconfirmed(io::Error),
}

/// Retains the same process/config/storage/journal and payload pins while observed.
/// Drop closes handles and cancels only children whose resume was never attempted.
/// No job object, timeout termination, automatic helper enrollment or admission.
pub(super) struct StartedEducationObservation<const N: usize = 2> {
    children: [PendingEducationBootstrap; N],
    pub(super) attempts: [StartupAttempt; N],
    _payload: BootstrapLibrary,
}

impl PendingEducationBootstrap {
    fn exit_code_after_exit(&self) -> io::Result<u32> {
        match unsafe { WaitForSingleObject(self.process.as_raw_handle() as HANDLE, 0) } {
            WAIT_OBJECT_0 => {}
            WAIT_TIMEOUT => {
                return Err(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "Child still alive",
                ))
            }
            _ => return Err(io::Error::last_os_error()),
        }
        let mut code = 0;
        if unsafe {
            windows_sys::Win32::System::Threading::GetExitCodeProcess(
                self.process.as_raw_handle() as HANDLE,
                &mut code,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(code)
    }

    fn resume_once(&mut self) -> io::Result<()> {
        if self.resume_attempted {
            return Err(invalid("Bootstrap resume already attempted"));
        }
        self.identity()?;
        // Set before the call: even an ambiguous failure must never fall back to kill-on-drop.
        self.resume_attempted = true;
        let previous = unsafe {
            windows_sys::Win32::System::Threading::ResumeThread(
                self._thread.as_raw_handle() as HANDLE
            )
        };
        match previous {
            1 => Ok(()),
            u32::MAX => Err(io::Error::last_os_error()),
            _ => Err(invalid(
                "Unexpected suspend count; child preserved without retry",
            )),
        }
    }
}

impl<const N: usize> PreparedEducationEnrollment<N> {
    /// Unmounted, separately authorized experiment only. Empty reviewed policy prevents
    /// constructing this pair for Education. Preflight errors cancel only unstarted children;
    /// after any attempt return the owner and outcomes even on partial failure.
    pub(super) fn resume_for_authorized_observation(
        self,
    ) -> io::Result<StartedEducationObservation<N>> {
        for child in &self._children {
            child.identity()?;
        }
        let Self {
            _children: mut children,
            _payload,
        } = self;
        let mut attempts = std::array::from_fn(|_| StartupAttempt::NotAttempted);
        for (child, attempt) in children.iter_mut().zip(&mut attempts) {
            match child.resume_once() {
                Ok(()) => *attempt = StartupAttempt::Resumed,
                Err(error) => {
                    *attempt = StartupAttempt::Unconfirmed(error);
                    break;
                }
            }
        }
        Ok(StartedEducationObservation {
            children,
            attempts,
            _payload,
        })
    }
}

pub(super) struct SavedStartupObservation {
    pub(super) path: PathBuf,
    pub(super) exit_code: u32,
}
impl<const N: usize> StartedEducationObservation<N> {
    /// Caller polls after the operator closes the game normally. Does not wait, kill,
    /// infer normal exit from a code, or collect account/world/command-line content.
    /// Close the manager early and the journal is lost, while attempted children survive.
    pub(super) fn save_after_exit(&self, slot: u32) -> io::Result<SavedStartupObservation> {
        use std::io::Write;
        let child = self
            .children
            .get(slot.checked_sub(1).unwrap_or(u32::MAX) as usize)
            .ok_or_else(|| invalid("Slot is outside the prepared observation set"))?;
        if !child.resume_attempted {
            return Err(invalid("Observation startup was not attempted"));
        }
        let exit_code = child.exit_code_after_exit()?;
        let bytes = child.observation_bytes_after_exit()?;
        let store = child
            .storage
            .as_ref()
            .ok_or_else(|| invalid("Missing observation storage"))?;
        // Pin all ordinary ancestors until the create-new write completes; reject substituted paths.
        let mut pins = Vec::new();
        for ancestor in store
            .private_path
            .ancestors()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            pins.push(open_directory(ancestor, FILE_READ_ATTRIBUTES)?);
        }
        if !same_image(
            &directory_id(pins.last().unwrap())?,
            &directory_id(&store.private)?,
        ) {
            return Err(invalid("Observation output directory changed"));
        }
        let path = child.observation_path()?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .share_mode(FILE_SHARE_READ)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(&path)?;
        image_id(&file)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        // A failed write leaves an incomplete local artifact; never overwrite it on retry.
        Ok(SavedStartupObservation { path, exit_code })
    }
}

pub(super) fn prepare_registered_enrolled() -> io::Result<PreparedEducationEnrollment> {
    prepare_registered_enrolled_for::<2>()
}
fn prepare_registered_enrolled_for<const N: usize>() -> io::Result<PreparedEducationEnrollment<N>> {
    let payload = BootstrapLibrary::reviewed()?;
    let children = prepare_registered_mode::<N>(true, Some(&payload))?;
    for child in &children {
        payload.enroll(child)?;
    }
    Ok(PreparedEducationEnrollment {
        _children: children,
        _payload: payload,
    })
}

/// Fixed opt-in invocation in the existing manager, dispatched before creating its UI.
/// No path/PID/options or hash override. Standard output is count/status metadata only.
pub(super) fn maybe_run_startup_observation() -> bool {
    if session_controller::maybe_run() { return true; }
    let mut args = std::env::args_os().skip(1);
    let mode = args.next();
    let count = match mode.as_deref() {
        Some(v) if v == OsStr::new("--education-startup-observation-once") => 2,
        Some(v) if v == OsStr::new("--education-five-startup-observation-once") => 5,
        Some(v) if v == OsStr::new("--education-prepare-five-profiles-once") => 0,
        _ => return false,
    };
    let result = if args.next().is_some() {
        Err(invalid(
            "Startup observation accepts no additional arguments",
        ))
    } else {
        if count == 0 {
            prepare_five_profiles()
        } else {
            run_startup_observation(count)
        }
    };
    let code = match result {
        Ok(code) => code,
        Err(error) => {
            use std::io::Write;
            let _ = writeln!(
                io::stderr(),
                "{{\"event\":\"startup-refused\",\"kind\":\"{:?}\",\"osCode\":{}}}",
                error.kind(),
                error
                    .raw_os_error()
                    .map_or_else(|| "null".to_owned(), |code| code.to_string())
            );
            2
        }
    };
    std::process::exit(code);
}

fn run_startup_observation(count: usize) -> io::Result<i32> {
    use windows_sys::Win32::{
        Foundation::{GetLastError, ERROR_ALREADY_EXISTS},
        System::Threading::CreateMutexW,
    };
    // One experiment process per interactive session; no persisted readiness/account state.
    let name = wide(OsStr::new("Local\\MivubiEducationStartupObservation"))?;
    let raw = unsafe { CreateMutexW(ptr::null(), 0, name.as_ptr()) };
    if raw.is_null() {
        return Err(io::Error::last_os_error());
    }
    let existed = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
    let _gate = unsafe { OwnedHandle::from_raw_handle(raw as _) };
    if existed {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Startup experiment already active",
        ));
    }
    let mut launch_guard = super::acquire_native_launch()?;
    // Preserve existing games; no termination, sign-out or activation fallback.
    if !super::inspect_native_workspace()?.processes.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Close the existing game normally first",
        ));
    }
    if count == 5 {
        let started =
            prepare_registered_enrolled_for::<5>()?.resume_for_authorized_observation()?;
        *launch_guard = Some(std::time::Instant::now());
        observe_until_exit(&started)
    } else if count == 2 {
        let started = prepare_registered_enrolled()?.resume_for_authorized_observation()?;
        *launch_guard = Some(std::time::Instant::now());
        observe_until_exit(&started)
    } else {
        Err(invalid("Unsupported Education slot count"))
    }
}

fn slot_caption(slot: u32) -> io::Result<Vec<u16>> {
    if !(1..=MAX_EDUCATION_SLOTS).contains(&slot) {
        return Err(invalid("Invalid Minecraft window slot"));
    }
    wide(OsStr::new(&format!("Minecraft - {slot}")))
}

fn label_slot_window(
    hwnd: windows_sys::Win32::Foundation::HWND,
    pid: u32,
    slot: u32,
) -> io::Result<()> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetWindowTextW, SendMessageTimeoutW, SMTO_ABORTIFHUNG, SMTO_BLOCK, SMTO_ERRORONEXIT,
        WM_SETTEXT,
    };
    let caption = slot_caption(slot)?;
    super::verify_live_client_02_window(hwnd, pid)?;
    let mut current = [0u16; 32];
    let length = unsafe { GetWindowTextW(hwnd, current.as_mut_ptr(), current.len() as i32) };
    if length > 0 && current[..length as usize] == caption[..caption.len() - 1] {
        return Ok(());
    }
    // System-message marshalling, bounded wait; never inject another game hook for presentation.
    super::verify_live_client_02_window(hwnd, pid)?;
    let mut result = 0;
    if unsafe {
        SendMessageTimeoutW(
            hwnd,
            WM_SETTEXT,
            0,
            caption.as_ptr() as isize,
            SMTO_ABORTIFHUNG | SMTO_BLOCK | SMTO_ERRORONEXIT,
            200,
            &mut result,
        )
    } == 0
        || result == 0
    {
        return Err(io::Error::last_os_error());
    }
    super::verify_live_client_02_window(hwnd, pid)
}

fn label_started_windows<const N: usize>(started: &StartedEducationObservation<N>) {
    use windows_sys::Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{EnumWindows, GetWindowThreadProcessId},
    };
    struct Selection {
        pid: u32,
        window: HWND,
        ambiguous: bool,
    }
    unsafe extern "system" fn select(hwnd: HWND, data: isize) -> i32 {
        let state = &mut *(data as *mut Selection);
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == state.pid && super::verify_live_client_02_window(hwnd, pid).is_ok() {
            if state.window.is_null() {
                state.window = hwnd;
            } else {
                state.ambiguous = true;
            }
        }
        1
    }
    for child in &started.children {
        if !child.resume_attempted || child.identity().is_err() {
            continue;
        }
        let Some(store) = &child.storage else {
            continue;
        };
        let mut selection = Selection {
            pid: child.pid,
            window: ptr::null_mut(),
            ambiguous: false,
        };
        if unsafe { EnumWindows(Some(select), &mut selection as *mut Selection as isize) } == 0
            || selection.ambiguous
            || selection.window.is_null()
            || child.identity().is_err()
        {
            continue;
        }
        // The immutable store slot, never enumeration order or a matching title, assigns the number.
        let _ = label_slot_window(selection.window, child.pid, store.slot);
    }
}

fn observe_until_exit<const N: usize>(started: &StartedEducationObservation<N>) -> io::Result<i32> {
    use std::io::Write;
    let mut completed = [false; N];
    let mut failed = false;
    let mut caption_check = std::time::Instant::now();
    for (index, attempt) in started.attempts.iter().enumerate() {
        let state = match attempt {
            StartupAttempt::NotAttempted => {
                completed[index] = true;
                failed = true;
                "not-attempted"
            }
            StartupAttempt::Resumed => "resume-call-succeeded",
            StartupAttempt::Unconfirmed(_) => {
                failed = true;
                "resume-unconfirmed"
            }
        };
        writeln!(io::stdout(), "{{\"event\":\"startup-attempt\",\"slot\":{},\"state\":\"{}\",\"providerReady\":false}}",
            index + 1, state)?;
    }
    // Recording ends at its existing bound. Keep original handles until operator closure;
    // dropping this runner at any time still preserves every attempted child.
    while completed.iter().any(|done| !done) {
        if caption_check.elapsed() >= std::time::Duration::from_secs(1) {
            label_started_windows(started);
            caption_check = std::time::Instant::now();
        }
        for (index, done) in completed.iter_mut().enumerate() {
            if *done {
                continue;
            }
            match started.save_after_exit(index as u32 + 1) {
                Ok(saved) => {
                    *done = true;
                    failed |= saved.exit_code != 0;
                    writeln!(io::stdout(),
                        "{{\"event\":\"journal-saved\",\"slot\":{},\"exitCode\":{},\"providerReady\":false}}",
                        index + 1, saved.exit_code)?;
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                Err(error) => {
                    *done = true;
                    failed = true;
                    // A rejected/incomplete journal must not erase the original child's exit outcome.
                    let code = started.children[index]
                        .exit_code_after_exit()
                        .ok()
                        .map_or_else(|| "null".to_owned(), |code| code.to_string());
                    writeln!(io::stdout(),
                        "{{\"event\":\"journal-failed\",\"slot\":{},\"kind\":\"{:?}\",\"exitCode\":{},\"providerReady\":false}}",
                        index + 1, error.kind(), code)?;
                }
            }
        }
        if completed.iter().any(|done| !done) {
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
    }
    Ok(i32::from(failed))
}

use std::{
    ffi::{c_void, OsString},
    os::windows::ffi::OsStringExt,
    path::PathBuf,
};
use windows_sys::Win32::{
    Foundation::{
        DuplicateHandle, SetHandleInformation, DUPLICATE_SAME_ACCESS, HANDLE_FLAG_INHERIT,
        INVALID_HANDLE_VALUE,
    },
    Globalization::CompareStringOrdinal,
    Storage::FileSystem::{
        FileCaseSensitiveInfo, GetFinalPathNameByHandleW, FILE_CASE_SENSITIVE_INFO,
        FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_LIST_DIRECTORY,
        FILE_READ_ATTRIBUTES, FILE_SHARE_WRITE, VOLUME_NAME_DOS,
    },
    System::{
        Com::{CoCreateGuid, CoTaskMemFree},
        Environment::{FreeEnvironmentStringsW, GetEnvironmentStringsW},
        LibraryLoader::{GetModuleHandleW, GetProcAddress},
        Memory::{
            CreateFileMappingW, MapViewOfFile, UnmapViewOfFile, FILE_MAP_READ, FILE_MAP_WRITE,
            PAGE_READWRITE,
        },
        Registry::{
            RegCloseKey, RegCreateKeyExW, RegOpenCurrentUser, RegOpenKeyExW, HKEY,
            HKEY_CURRENT_USER, KEY_ALL_ACCESS, KEY_READ, REG_CREATED_NEW_KEY,
            REG_OPTION_NON_VOLATILE,
        },
        Threading::{
            DeleteProcThreadAttributeList, GetCurrentProcess, InitializeProcThreadAttributeList,
            UpdateProcThreadAttribute, EXTENDED_STARTUPINFO_PRESENT, LPPROC_THREAD_ATTRIBUTE_LIST,
            PROC_THREAD_ATTRIBUTE_HANDLE_LIST, STARTUPINFOEXW,
        },
    },
    UI::Shell::{
        FOLDERID_LocalAppData, FOLDERID_Profile, FOLDERID_RoamingAppData, SHGetKnownFolderPath,
    },
};

const CONFIG_ENV: &str = "MIVUBI_EDUCATION_BOOTSTRAP_SECTION";

// Must match the private education-config.h, not a public product wire type.
#[repr(C)]
struct EducationConfig {
    magic: u32,
    version: u32,
    bytes: u32,
    slot: u32,
    pid: u32,
    session: u32,
    scope: u32,
    singleton_caller_rva: u32,
    created: FILETIME,
    logical_root: HANDLE,
    private_root: HANDLE,
    registry_root: HKEY,
    logical_id: FILE_ID_INFO,
    private_id: FILE_ID_INFO,
    user_key_path: [u16; 512],
}
const _: () = assert!(mem::size_of::<EducationConfig>() == 1136);
const _: () = assert!(mem::offset_of!(EducationConfig, logical_id) == 64);
const _: () = assert!(mem::offset_of!(EducationConfig, user_key_path) == 112);

struct RegistryKey(HKEY);
impl Drop for RegistryKey {
    fn drop(&mut self) {
        unsafe {
            RegCloseKey(self.0);
        }
    }
}

struct SlotStorage {
    slot: u32,
    logical: File,
    private: File,
    temporary: [File; 2], // Retain ordinary Local and Local/Temp without delete sharing.
    registry: RegistryKey,
    user_key_path: [u16; 512],
    // Fresh data persists on drop; this is not another profile/readiness database.
    private_path: PathBuf,
    singleton_caller_rva: u32,
}

fn directory_id(file: &File) -> io::Result<FILE_ID_INFO> {
    let raw = file.as_raw_handle() as HANDLE;
    let mut tag: FILE_ATTRIBUTE_TAG_INFO = unsafe { mem::zeroed() };
    let mut id: FILE_ID_INFO = unsafe { mem::zeroed() };
    let mut case: FILE_CASE_SENSITIVE_INFO = unsafe { mem::zeroed() };
    // SAFETY: Query fixed-size metadata on the retained directory handle.
    if unsafe {
        GetFileInformationByHandleEx(
            raw,
            FileAttributeTagInfo,
            &mut tag as *mut _ as _,
            mem::size_of_val(&tag) as u32,
        )
    } == 0
        || unsafe {
            GetFileInformationByHandleEx(
                raw,
                FileIdInfo,
                &mut id as *mut _ as _,
                mem::size_of_val(&id) as u32,
            )
        } == 0
        || unsafe {
            GetFileInformationByHandleEx(
                raw,
                FileCaseSensitiveInfo,
                &mut case as *mut _ as _,
                mem::size_of_val(&case) as u32,
            )
        } == 0
    {
        return Err(io::Error::last_os_error());
    }
    if tag.FileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0
        || tag.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
        || case.Flags != 0
    {
        return Err(invalid("Unsupported bootstrap directory"));
    }
    Ok(id)
}

fn open_directory(path: &Path, access: u32) -> io::Result<File> {
    let file = OpenOptions::new()
        .read(true)
        .access_mode(access)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    directory_id(&file)?;
    directory_path(&file)?;
    Ok(file)
}

fn directory_path(file: &File) -> io::Result<PathBuf> {
    let mut path = [0u16; 512];
    let count = unsafe {
        GetFinalPathNameByHandleW(
            file.as_raw_handle() as HANDLE,
            path.as_mut_ptr(),
            512,
            VOLUME_NAME_DOS,
        )
    };
    if count < 7 || count >= 480 {
        return Err(invalid("Unsupported bootstrap directory path length"));
    }
    let value = &path[..count as usize];
    if !value.starts_with(&[92, 92, 63, 92])
        || value[5] != 58
        || value[6] != 92
        || !((65..=90).contains(&value[4]) || (97..=122).contains(&value[4]))
        || value.iter().any(|c| *c < 32 || *c > 126)
    {
        return Err(invalid(
            "Bootstrap DLL currently requires bounded ASCII directory paths",
        ));
    }
    Ok(PathBuf::from(OsString::from_wide(&value[4..])))
}

fn fresh_root(parent: &Path) -> io::Result<PathBuf> {
    let mut guid: windows_sys::core::GUID = unsafe { mem::zeroed() };
    if unsafe { CoCreateGuid(&mut guid) } < 0 {
        return Err(invalid("Cannot allocate fresh pilot identity"));
    }
    let suffix: String = guid.data4.iter().map(|b| format!("{b:02x}")).collect();
    let path = parent.join(format!(
        "MivubiEducationPilot_{:08x}{:04x}{:04x}{suffix}",
        guid.data1, guid.data2, guid.data3
    ));
    std::fs::create_dir(&path)?; // Never reopen, migrate, recursively clear or reseed old data.
    Ok(path)
}

fn registry_leaf(id: &FILE_ID_INFO, slot: u32) -> io::Result<String> {
    if !(1..=MAX_EDUCATION_SLOTS).contains(&slot) {
        return Err(invalid("Invalid Education profile slot"));
    }
    let bytes: String = id
        .FileId
        .Identifier
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    Ok(format!(
        "{:016x}_{bytes}_{}",
        id.VolumeSerialNumber,
        char::from(b'A' + (slot - 1) as u8)
    ))
}

fn user_registry_path() -> io::Result<[u16; 512]> {
    let mut key = ptr::null_mut();
    let status = unsafe { RegOpenCurrentUser(KEY_READ, &mut key) };
    if status != 0 {
        return Err(io::Error::from_raw_os_error(status as i32));
    }
    let key = RegistryKey(key);
    type Query = unsafe extern "system" fn(HANDLE, u32, *mut c_void, u32, *mut u32) -> i32;
    let name = wide(OsStr::new("ntdll.dll"))?;
    let module = unsafe { GetModuleHandleW(name.as_ptr()) };
    if module.is_null() {
        return Err(io::Error::last_os_error());
    }
    let address = unsafe { GetProcAddress(module, c"NtQueryKey".as_ptr() as _) }
        .ok_or_else(|| invalid("Native registry metadata query unavailable"))?;
    let query: Query = unsafe { mem::transmute(address) };
    let mut buffer = [0u32; 257];
    let mut returned = 0;
    // SAFETY: Fixed loaded ntdll export, KeyNameInformation, bounded aligned output.
    if unsafe {
        query(
            key.0,
            3,
            buffer.as_mut_ptr() as _,
            mem::size_of_val(&buffer) as u32,
            &mut returned,
        )
    } < 0
        || returned < 4
        || returned as usize > mem::size_of_val(&buffer)
    {
        return Err(invalid("Current-user registry metadata unavailable"));
    }
    let bytes = buffer[0] as usize;
    if bytes % 2 != 0 || bytes >= 1024 || bytes > returned as usize - 4 {
        return Err(invalid("Invalid current-user registry path"));
    }
    let mut result = [0u16; 512];
    unsafe {
        ptr::copy_nonoverlapping(
            buffer.as_ptr().add(1) as *const u16,
            result.as_mut_ptr(),
            bytes / 2,
        );
    }
    Ok(result)
}

fn private_temporary(private: &File) -> io::Result<[File; 2]> {
    let access = FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY;
    let local = open_directory(&directory_path(private)?.join("Local"), access)?;
    let path = directory_path(&local)?.join("Temp");
    match std::fs::create_dir(&path) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error),
    }
    let temporary = open_directory(&path, access)?;
    Ok([local, temporary]) // Never follow a reparse, clear a cache or copy a profile.
}

fn existing_private_temporary(private: &File) -> io::Result<[File; 2]> {
    let access = FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY;
    let local = open_directory(&directory_path(private)?.join("Local"), access)?;
    let temporary = open_directory(&directory_path(&local)?.join("Temp"), access)?;
    Ok([local, temporary])
}

fn create_private_slot(logical: &File, root: &Path, slot: u32) -> io::Result<SlotStorage> {
    create_private_slot_at(logical, &root.join(format!("MC{slot:02}")), slot)
}

fn create_private_slot_at(logical: &File, private_path: &Path, slot: u32) -> io::Result<SlotStorage> {
    if !(1..=MAX_EDUCATION_SLOTS).contains(&slot) {
        return Err(invalid("Invalid Education slot"));
    }
    let user_key_path = user_registry_path()?;
    std::fs::create_dir(&private_path)?;
    let private = open_directory(&private_path, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY)?;
    // Match the DLL's canonical spelling, including expansion of Windows 8.3 aliases.
    let private_path = directory_path(&private)?;
    for leaf in ["Roaming", "Local"] {
        std::fs::create_dir(private_path.join(leaf))?;
    }
    let id = directory_id(&private)?;
    let path = wide(OsStr::new(&format!(
        "Software\\MivubiEducationPilot\\{}",
        registry_leaf(&id, slot)?
    )))?;
    let mut key = ptr::null_mut();
    let mut disposition = 0;
    let status = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            path.as_ptr(),
            0,
            ptr::null(),
            REG_OPTION_NON_VOLATILE,
            KEY_ALL_ACCESS,
            ptr::null(),
            &mut key,
            &mut disposition,
        )
    };
    if status != 0 {
        return Err(io::Error::from_raw_os_error(status as i32));
    }
    let registry = RegistryKey(key);
    if disposition != REG_CREATED_NEW_KEY {
        return Err(invalid("Refusing an existing pilot registry store"));
    }
    Ok(SlotStorage {
        slot,
        logical: logical.try_clone()?,
        temporary: private_temporary(&private)?,
        private,
        registry,
        user_key_path,
        private_path,
        singleton_caller_rva: 0,
    })
}

fn fresh_slots(logical: File, parent: &Path) -> io::Result<[SlotStorage; 2]> {
    let logical_path = directory_path(&logical)?;
    let root = fresh_root(parent)?;
    let root_handle = open_directory(&root, FILE_READ_ATTRIBUTES)?;
    let root_path = directory_path(&root_handle)?;
    let a = root_path.to_string_lossy().to_ascii_lowercase();
    let b = logical_path.to_string_lossy().to_ascii_lowercase();
    if Path::new(&a).starts_with(&b) || Path::new(&b).starts_with(&a) {
        return Err(invalid(
            "Private backing must be outside the logical AppData tree",
        ));
    }
    let mut slots = Vec::with_capacity(2);
    for slot in 1..=2 {
        slots.push(create_private_slot(&logical, &root, slot)?);
    }
    slots
        .try_into()
        .map_err(|_| invalid("Missing private slot"))
}

fn known_folder(id: &windows_sys::core::GUID) -> io::Result<PathBuf> {
    let mut raw = ptr::null_mut();
    let status = unsafe { SHGetKnownFolderPath(id, 0, ptr::null_mut(), &mut raw) };
    if status < 0 || raw.is_null() {
        if !raw.is_null() {
            unsafe {
                CoTaskMemFree(raw as _);
            }
        }
        return Err(invalid("Current-user Known Folder unavailable"));
    }
    let mut count = 0;
    while count < 32768 && unsafe { *raw.add(count) } != 0 {
        count += 1;
    }
    let path = if count < 32768 {
        Ok(PathBuf::from(OsString::from_wide(unsafe {
            std::slice::from_raw_parts(raw, count)
        })))
    } else {
        Err(invalid("Invalid Known Folder pathname"))
    };
    unsafe {
        CoTaskMemFree(raw as _);
    }
    path
}

// Existing pair remains immutable. Five-slot replay requires its own reviewed identity capability.
#[cfg(test)]
const REVIEWED_FIVE_REPLAY: Option<[u8; 32]> = None;
#[cfg(not(test))]
const REVIEWED_FIVE_REPLAY: Option<[u8; 32]> = Some([
    0xaa, 0x04, 0xa0, 0x71, 0x52, 0xfb, 0x1b, 0x5f, 0x43, 0xdd, 0xcc, 0xe4, 0x49, 0x8b, 0x3c, 0xa9,
    0xb7, 0x84, 0x9d, 0x7e, 0x79, 0xb4, 0x0d, 0x7a, 0x42, 0x8c, 0x18, 0xbd, 0x1d, 0xd0, 0xfa, 0x0b,
]);
// One reviewed acceptance pair only; no profile discovery, migration or token/world copying.
#[cfg(test)]
const REVIEWED_REPLAY: Option<[u8; 32]> = None;
#[cfg(not(test))]
const REVIEWED_REPLAY: Option<[u8; 32]> = Some([
    0x41, 0xdb, 0xc7, 0xdb, 0x44, 0xd2, 0x91, 0xcd, 0x5e, 0xb9, 0xc7, 0xfc, 0x60, 0x90, 0x54, 0xa8,
    0xbb, 0x0c, 0x76, 0x37, 0xc1, 0x04, 0xfa, 0x85, 0xe8, 0xdc, 0x42, 0x70, 0xa9, 0xa4, 0xa8, 0xaf,
]);

fn reopen_slots(
    logical: File,
    parent: &Path,
    bytes: &[u8],
    expected: [u8; 32],
) -> io::Result<[SlotStorage; 2]> {
    reopen_slots_for::<2>(logical, parent, bytes, expected)
}
fn reopen_slots_for<const N: usize>(
    logical: File,
    parent: &Path,
    bytes: &[u8],
    expected: [u8; 32],
) -> io::Result<[SlotStorage; N]> {
    use sha2::{Digest, Sha256};
    if N != 2 && N != 5 {
        return Err(invalid("Unsupported replay slot count"));
    }
    let size = 168 + N * 24;
    if bytes.len() != size || <[u8; 32]>::from(Sha256::digest(bytes)) != expected {
        return Err(invalid("Unreviewed replay profile capability"));
    }
    let word = |at| u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
    if word(0) != 0x52504c31
        || word(4) != (if N == 2 { 1 } else { 2 })
        || word(8) != size as u32
        || word(12) != 0
    {
        return Err(invalid("Invalid replay profile capability"));
    }
    let units: Vec<u16> = bytes[16..144]
        .chunks_exact(2)
        .map(|b| u16::from_le_bytes(b.try_into().unwrap()))
        .collect();
    let end = units
        .iter()
        .position(|u| *u == 0)
        .ok_or_else(|| invalid("Unterminated replay name"))?;
    let name = String::from_utf16(&units[..end]).map_err(|_| invalid("Invalid replay name"))?;
    let suffix = name
        .strip_prefix("MivubiEducationPilot_")
        .ok_or_else(|| invalid("Invalid replay root"))?;
    if suffix.len() != 32
        || !suffix.bytes().all(|c| c.is_ascii_hexdigit())
        || units[end..].iter().any(|u| *u != 0)
    {
        return Err(invalid("Invalid replay root"));
    }
    let root = parent.join(name);
    let mut pins = Vec::new();
    for path in root.ancestors().collect::<Vec<_>>().into_iter().rev() {
        pins.push(open_directory(path, FILE_READ_ATTRIBUTES)?);
    }
    let matches = |file: &File, at: usize| -> io::Result<bool> {
        let id = directory_id(file)?;
        Ok(
            id.VolumeSerialNumber == u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
                && id.FileId.Identifier == bytes[at + 8..at + 24],
        )
    };
    if !matches(pins.last().unwrap(), 144)? {
        return Err(invalid("Replay root was replaced"));
    }
    let logical_path = directory_path(&logical)?
        .to_string_lossy()
        .to_ascii_lowercase();
    let root_path = directory_path(pins.last().unwrap())?
        .to_string_lossy()
        .to_ascii_lowercase();
    if Path::new(&logical_path).starts_with(&root_path)
        || Path::new(&root_path).starts_with(&logical_path)
    {
        return Err(invalid("Overlapping replay AppData roots"));
    }
    let user_key_path = user_registry_path()?;
    let mut slots = Vec::with_capacity(N);
    for slot in 1..=N as u32 {
        let private = open_directory(
            &root.join(format!("MC{slot:02}")),
            FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY,
        )?;
        if !matches(&private, 168 + (slot as usize - 1) * 24)? {
            return Err(invalid("Replay slot was replaced"));
        }
        let private_path = directory_path(&private)?;
        // Open existing directories and registry only. A missing store never silently resets login.
        for leaf in ["Roaming", "Local"] {
            pins.push(open_directory(
                &private_path.join(leaf),
                FILE_READ_ATTRIBUTES,
            )?);
        }
        let path = wide(OsStr::new(&format!(
            "Software\\MivubiEducationPilot\\{}",
            registry_leaf(&directory_id(&private)?, slot)?
        )))?;
        let mut key = ptr::null_mut();
        let status = unsafe {
            RegOpenKeyExW(
                HKEY_CURRENT_USER,
                path.as_ptr(),
                0,
                KEY_ALL_ACCESS,
                &mut key,
            )
        };
        if status != 0 {
            return Err(io::Error::from_raw_os_error(status as i32));
        }
        slots.push(SlotStorage {
            slot,
            logical: logical.try_clone()?,
            temporary: existing_private_temporary(&private)?,
            private,
            registry: RegistryKey(key),
            user_key_path,
            private_path,
            singleton_caller_rva: 0,
        });
    }
    slots.try_into().map_err(|_| invalid("Missing replay slot"))
}

fn five_replay_bytes(pair: &[u8], stores: &[SlotStorage]) -> io::Result<Vec<u8>> {
    if pair.len() != 216 || stores.len() != 5 {
        return Err(invalid("Incomplete five-slot identity set"));
    }
    let mut bytes = vec![0u8; 288];
    bytes[..216].copy_from_slice(pair);
    bytes[4..8].copy_from_slice(&2u32.to_le_bytes());
    bytes[8..12].copy_from_slice(&288u32.to_le_bytes());
    for (index, store) in stores.iter().enumerate() {
        if store.slot != index as u32 + 1 {
            return Err(invalid("Unordered five-slot identity set"));
        }
        let id = directory_id(&store.private)?;
        let at = 168 + index * 24;
        if index < 2 {
            if pair[at..at + 8] != id.VolumeSerialNumber.to_le_bytes()
                || pair[at + 8..at + 24] != id.FileId.Identifier
            {
                return Err(invalid("Original pair identity changed"));
            }
        }
        bytes[at..at + 8].copy_from_slice(&id.VolumeSerialNumber.to_le_bytes());
        bytes[at + 8..at + 24].copy_from_slice(&id.FileId.Identifier);
    }
    Ok(bytes)
}

// Explicit one-shot preparation only. Never launch a game or read account/world contents.
fn prepare_five_profiles() -> io::Result<i32> {
    use sha2::{Digest, Sha256};
    use std::io::{Read, Write};
    use windows_sys::Win32::{
        Foundation::{GetLastError, ERROR_ALREADY_EXISTS},
        System::Threading::CreateMutexW,
    };
    let name = wide(OsStr::new("Local\\MivubiEducationFiveProfilePreparation"))?;
    let raw = unsafe { CreateMutexW(ptr::null(), 0, name.as_ptr()) };
    if raw.is_null() {
        return Err(io::Error::last_os_error());
    }
    let existed = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
    let _gate = unsafe { OwnedHandle::from_raw_handle(raw as _) };
    if existed {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Five-profile preparation already active",
        ));
    }
    // Existing pair replay is the trust anchor; no root path or profile override is accepted.
    let pair = registered_slots::<2>(true)?;
    let payload_root = std::env::current_exe()?
        .parent()
        .ok_or_else(|| invalid("Missing replay payload root"))?
        .join("independent-bootstrap");
    let _payload_pin = open_directory(&payload_root, FILE_READ_ATTRIBUTES)?;
    let mut original = pin_image(&payload_root.join("replay-profile-pair.bin"))?;
    let mut pair_bytes = [0u8; 216];
    original.read_exact(&mut pair_bytes)?;
    if Some(<[u8; 32]>::from(Sha256::digest(pair_bytes))) != REVIEWED_REPLAY {
        return Err(invalid("Original replay changed"));
    }
    let output = payload_root.join("replay-profile-five.bin");
    let parent = known_folder(&FOLDERID_Profile)?;
    let bytes = if output.try_exists()? {
        let mut file = pin_image(&output)?;
        if file.metadata()?.len() != 288 {
            return Err(invalid(
                "Incomplete five-slot capability; preserve data and review",
            ));
        }
        let mut bytes = vec![0u8; 288];
        file.read_exact(&mut bytes)?;
        if bytes[16..216] != pair_bytes[16..216] {
            return Err(invalid("Five-slot capability changed the original pair"));
        }
        // Preparation revalidation is not launch authorization: launch still requires its reviewed digest.
        let stores = reopen_slots_for::<5>(
            pair[0].logical.try_clone()?,
            &parent,
            &bytes,
            Sha256::digest(&bytes).into(),
        )?;
        if five_replay_bytes(&pair_bytes, &stores)? != bytes {
            return Err(invalid("Five-slot capability mismatch"));
        }
        bytes
    } else {
        let root = pair[0]
            .private_path
            .parent()
            .ok_or_else(|| invalid("Missing original profile root"))?
            .to_path_buf();
        let _root_pin = open_directory(&root, FILE_READ_ATTRIBUTES)?;
        // Refuse existing extra directories before writing anything; never adopt/reset a partial profile.
        for slot in 3..=5 {
            if root.join(format!("MC{slot:02}")).try_exists()? {
                return Err(invalid(
                    "Extra slot already exists; preserve data and review partial preparation",
                ));
            }
        }
        let mut stores = Vec::from(pair);
        let root = root.to_path_buf();
        for slot in 3..=5 {
            stores.push(create_private_slot(&stores[0].logical, &root, slot)?);
        }
        let bytes = five_replay_bytes(&pair_bytes, &stores)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .share_mode(FILE_SHARE_READ)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(&output)?;
        image_id(&file)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        bytes
    };
    let digest: String = Sha256::digest(&bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    writeln!(io::stdout(), "{{\"event\":\"five-profiles-prepared\",\"slots\":5,\"capabilitySha256\":\"{digest}\",\"gameLaunched\":false}}")?;
    Ok(0)
}

fn replay_roots() -> io::Result<(File, PathBuf)> {
    let roaming = known_folder(&FOLDERID_RoamingAppData)?;
    let local = known_folder(&FOLDERID_LocalAppData)?;
    if roaming.file_name() != Some(OsStr::new("Roaming"))
        || local.file_name() != Some(OsStr::new("Local"))
    {
        return Err(invalid(
            "Redirected AppData layouts require explicit mediation",
        ));
    }
    let a = open_directory(
        roaming
            .parent()
            .ok_or_else(|| invalid("Missing AppData parent"))?,
        FILE_READ_ATTRIBUTES,
    )?;
    let b = open_directory(
        local
            .parent()
            .ok_or_else(|| invalid("Missing AppData parent"))?,
        FILE_READ_ATTRIBUTES,
    )?;
    if !same_image(&directory_id(&a)?, &directory_id(&b)?) {
        return Err(invalid(
            "Split Roaming/Local roots are not supported by this DLL configuration",
        ));
    }
    let parent = known_folder(&FOLDERID_Profile)?;
    Ok((a, parent))
}

fn registered_slots<const N: usize>(observe: bool) -> io::Result<[SlotStorage; N]> {
    if N != 2 && N != 5 {
        return Err(invalid("Unsupported replay slot count"));
    }
    if N == 5 && observe && cfg!(feature = "lazy-tester-package") {
        if let Some(stores) = product_profiles::open()? {
            return Vec::from(stores).try_into().map_err(|_| invalid("Invalid product profile count"));
        }
    }
    // Authorized observation reuses an existing pair or refuses. An unset
    // review policy must never silently become fresh account/world storage.
    let replay = if observe {
        Some(
            (if N == 2 {
                REVIEWED_REPLAY
            } else {
                REVIEWED_FIVE_REPLAY
            })
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Unsupported,
                    "No reviewed existing Education profile pair",
                )
            })?,
        )
    } else {
        None
    };
    let (a, parent) = replay_roots()?;
    if let Some(expected) = replay {
        use std::io::Read;
        let binding = if N == 5 && cfg!(feature = "lazy-tester-package") {
            Some(profile_binding::binding_path(false)?)
        } else { None };
        let path = if let Some((path, _pins)) = &binding {
            path.clone()
        } else { std::env::current_exe()?
            .parent()
            .ok_or_else(|| invalid("Missing replay payload root"))?
            .join(if N == 2 {
                "independent-bootstrap/replay-profile-pair.bin"
            } else {
                "independent-bootstrap/replay-profile-five.bin"
            }) };
        let mut file = pin_image(&path)?;
        let size = 168 + N * 24;
        if file.metadata()?.len() != size as u64 {
            return Err(invalid("Invalid replay capability size"));
        }
        let mut bytes = vec![0u8; size];
        file.read_exact(&mut bytes)?;
        return reopen_slots_for::<N>(a, &parent, &bytes, expected);
    }
    if N != 2 {
        return Err(invalid(
            "Five-slot replay requires an explicit reviewed capability",
        ));
    }
    Vec::from(fresh_slots(a, &parent)?)
        .try_into()
        .map_err(|_| invalid("Invalid fresh slot count"))
}

struct Configuration {
    writable: OwnedHandle,
    readonly: OwnedHandle,
}
impl Configuration {
    fn create() -> io::Result<Self> {
        Self::create_sized(mem::size_of::<EducationConfig>() as u32)
    }
    fn create_sized(bytes: u32) -> io::Result<Self> {
        let raw = unsafe {
            CreateFileMappingW(
                INVALID_HANDLE_VALUE,
                ptr::null(),
                PAGE_READWRITE,
                0,
                bytes,
                ptr::null(),
            )
        };
        if raw.is_null() {
            return Err(io::Error::last_os_error());
        }
        let writable = unsafe { OwnedHandle::from_raw_handle(raw as _) };
        let mut reader = ptr::null_mut();
        if unsafe {
            DuplicateHandle(
                GetCurrentProcess(),
                raw,
                GetCurrentProcess(),
                &mut reader,
                FILE_MAP_READ,
                1,
                0,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(Self {
            writable,
            readonly: unsafe { OwnedHandle::from_raw_handle(reader as _) },
        })
    }

    fn publish(
        self,
        child: &PendingEducationBootstrap,
        storage: &SlotStorage,
    ) -> io::Result<OwnedHandle> {
        let identity = child.identity()?;
        let target = child.process.as_raw_handle() as HANDLE;
        let duplicate = |source: HANDLE| -> io::Result<HANDLE> {
            let mut remote = ptr::null_mut();
            if unsafe {
                DuplicateHandle(
                    GetCurrentProcess(),
                    source,
                    target,
                    &mut remote,
                    0,
                    0,
                    DUPLICATE_SAME_ACCESS,
                )
            } == 0
            {
                return Err(io::Error::last_os_error());
            }
            // Valid in child only; child cancellation closes these on any later error.
            Ok(remote)
        };
        let mut config: EducationConfig = unsafe { mem::zeroed() };
        config.magic = 0x45444231;
        config.version = 2;
        config.bytes = mem::size_of_val(&config) as u32;
        config.slot = storage.slot;
        config.pid = identity.0;
        config.session = interactive_session(identity.0)?;
        config.scope = if storage.singleton_caller_rva == 0 {
            1
        } else {
            2
        };
        config.singleton_caller_rva = storage.singleton_caller_rva;
        config.created = FILETIME {
            dwLowDateTime: identity.1 as u32,
            dwHighDateTime: (identity.1 >> 32) as u32,
        };
        config.logical_root = duplicate(storage.logical.as_raw_handle() as HANDLE)?;
        config.private_root = duplicate(storage.private.as_raw_handle() as HANDLE)?;
        config.registry_root = duplicate(storage.registry.0)?;
        config.logical_id = directory_id(&storage.logical)?;
        config.private_id = directory_id(&storage.private)?;
        config.user_key_path = storage.user_key_path;
        let mapped = unsafe {
            MapViewOfFile(
                self.writable.as_raw_handle() as HANDLE,
                FILE_MAP_WRITE,
                0,
                0,
                mem::size_of::<EducationConfig>(),
            )
        };
        if mapped.Value.is_null() {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: Full pagefile-backed writable view of the checked private ABI size.
        unsafe {
            ptr::write(mapped.Value as *mut EducationConfig, config);
        }
        if unsafe { UnmapViewOfFile(mapped) } == 0 {
            return Err(io::Error::last_os_error());
        }
        if unsafe {
            SetHandleInformation(
                self.readonly.as_raw_handle() as HANDLE,
                HANDLE_FLAG_INHERIT,
                0,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        child.identity()?;
        // No parent writable view/handle survives publication.
        Ok(self.readonly)
    }
}

struct InheritList<'a> {
    _memory: Vec<usize>,
    list: LPPROC_THREAD_ATTRIBUTE_LIST,
    _handles: &'a [HANDLE],
}
impl<'a> InheritList<'a> {
    fn new(handles: &'a [HANDLE]) -> io::Result<Self> {
        let mut bytes = 0;
        unsafe {
            InitializeProcThreadAttributeList(ptr::null_mut(), 1, 0, &mut bytes);
        }
        if bytes == 0 {
            return Err(io::Error::last_os_error());
        }
        let mut memory = vec![0usize; bytes.div_ceil(mem::size_of::<usize>())];
        let list = memory.as_mut_ptr() as LPPROC_THREAD_ATTRIBUTE_LIST;
        if unsafe { InitializeProcThreadAttributeList(list, 1, 0, &mut bytes) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let result = Self {
            _memory: memory,
            list,
            _handles: handles,
        };
        if unsafe {
            UpdateProcThreadAttribute(
                list,
                0,
                PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
                handles.as_ptr() as _,
                mem::size_of_val(handles),
                ptr::null_mut(),
                ptr::null(),
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(result)
    }
}

impl Drop for InheritList<'_> {
    fn drop(&mut self) {
        unsafe {
            DeleteProcThreadAttributeList(self.list);
        }
    }
}

fn environment_name(row: &[u16]) -> &[u16] {
    let start = usize::from(row.first() == Some(&61)); // Preserve hidden =C: drive entries.
    let end = row[start..]
        .iter()
        .position(|c| *c == 61)
        .map_or(row.len(), |i| start + i);
    &row[..end]
}

fn ordinal_names(a: &[u16], b: &[u16]) -> io::Result<std::cmp::Ordering> {
    match unsafe { CompareStringOrdinal(a.as_ptr(), a.len() as i32, b.as_ptr(), b.len() as i32, 1) }
    {
        1 => Ok(std::cmp::Ordering::Less),
        2 => Ok(std::cmp::Ordering::Equal),
        3 => Ok(std::cmp::Ordering::Greater),
        _ => Err(io::Error::last_os_error()),
    }
}

fn child_environment(
    storage: &SlotStorage,
    section: HANDLE,
    observation: Option<HANDLE>,
    presentation: Option<HANDLE>,
) -> io::Result<Vec<u16>> {
    let raw = unsafe { GetEnvironmentStringsW() };
    if raw.is_null() {
        return Err(io::Error::last_os_error());
    }
    let reserved: Vec<Vec<u16>> = [
        "APPDATA",
        "LOCALAPPDATA",
        "TMP",
        "TEMP",
        CONFIG_ENV,
        "MIVUBI_BOOTSTRAP_TEST_SECTION",
        OBSERVATION_ENV,
        "MIVUBI_EDUCATION_PRESENTATION",
        "MIVUBI_EDUCATION_PRESENTATION_SECTION",
    ]
    .iter()
    .map(|name| OsStr::new(name).encode_wide().collect())
    .collect();
    let mut rows = Vec::<Vec<u16>>::new();
    let mut cursor = raw;
    let mut units = 0usize;
    let copied = (|| {
        // Bound the copy; never log or persist inherited environment values.
        while unsafe { *cursor } != 0 {
            let mut length = 0;
            while unsafe { *cursor.add(length) } != 0 {
                length += 1;
                if units + length > 32767 {
                    return Err(invalid("Inherited environment exceeds bootstrap bound"));
                }
            }
            let row = unsafe { std::slice::from_raw_parts(cursor, length) };
            let mut overridden = false;
            for name in &reserved {
                overridden |=
                    ordinal_names(environment_name(row), name)? == std::cmp::Ordering::Equal;
            }
            if !overridden {
                rows.push(row.to_vec());
            }
            units += length + 1;
            cursor = unsafe { cursor.add(length + 1) };
        }
        Ok(())
    })();
    unsafe {
        FreeEnvironmentStringsW(raw);
    }
    copied?;
    let temporary = directory_path(&storage.temporary[1])?.into_os_string();
    for (key, value) in [
        (
            "APPDATA",
            storage.private_path.join("Roaming").into_os_string(),
        ),
        (
            "LOCALAPPDATA",
            storage.private_path.join("Local").into_os_string(),
        ),
        ("TMP", temporary.clone()),
        ("TEMP", temporary),
        (CONFIG_ENV, OsString::from(format!("{}", section as usize))),
    ] {
        let mut entry: Vec<u16> = OsStr::new(key).encode_wide().collect();
        entry.push(61);
        entry.extend(value.encode_wide());
        rows.push(entry);
    }
    if let Some(section) = observation {
        rows.push(
            format!("{OBSERVATION_ENV}={}", section as usize)
                .encode_utf16()
                .collect(),
        );
    }
    if let Some(section) = presentation {
        rows.push(
            "MIVUBI_EDUCATION_PRESENTATION=pace"
                .encode_utf16()
                .collect(),
        );
        rows.push(
            format!("MIVUBI_EDUCATION_PRESENTATION_SECTION={}", section as usize)
                .encode_utf16()
                .collect(),
        );
    }
    let mut compare_error = None;
    rows.sort_by(|a, b| {
        ordinal_names(environment_name(a), environment_name(b)).unwrap_or_else(|e| {
            compare_error = Some(e);
            std::cmp::Ordering::Equal
        })
    });
    if let Some(error) = compare_error {
        return Err(error);
    }
    let mut result = Vec::new();
    for row in rows {
        result.extend(row);
        result.push(0);
    }
    result.push(0);
    if result.len() > 32767 {
        return Err(invalid("Configured environment exceeds bootstrap bound"));
    }
    Ok(result)
}

#[cfg(test)]
#[path = "independent_bootstrap/owned_handoff.rs"]
mod owned_handoff;

// Fixed own-image transaction only. No descendant discovery or game-helper admission.
#[cfg(test)]
struct OwnedHelperTransaction<'a> {
    child: PendingEducationBootstrap,
    parent: &'a PendingEducationBootstrap,
    _payload: &'a BootstrapLibrary,
}

#[cfg(test)]
fn checked_slot_binding(child: &PendingEducationBootstrap) -> io::Result<&SlotStorage> {
    let (pid, birth) = child.identity()?;
    let store = child
        .storage
        .as_ref()
        .ok_or_else(|| invalid("Missing parent slot"))?;
    let section = child
        .configuration
        .as_ref()
        .ok_or_else(|| invalid("Missing parent config"))?;
    let view = unsafe {
        MapViewOfFile(
            section.as_raw_handle() as HANDLE,
            FILE_MAP_READ,
            0,
            0,
            mem::size_of::<EducationConfig>(),
        )
    };
    if view.Value.is_null() {
        return Err(io::Error::last_os_error());
    }
    let config = unsafe { ptr::read(view.Value as *const EducationConfig) };
    if unsafe { UnmapViewOfFile(view) } == 0 {
        return Err(io::Error::last_os_error());
    }
    if config.magic != 0x45444231
        || config.version != 2
        || config.bytes != mem::size_of::<EducationConfig>() as u32
        || config.pid != pid
        || config.slot != store.slot
        || !(1..=MAX_EDUCATION_SLOTS).contains(&store.slot)
        || config.created.dwLowDateTime != birth as u32
        || config.created.dwHighDateTime != (birth >> 32) as u32
        || config.session != interactive_session(pid)?
        || config.user_key_path != store.user_key_path
        || config.singleton_caller_rva != store.singleton_caller_rva
        || config.scope
            != if store.singleton_caller_rva == 0 {
                1
            } else {
                2
            }
        || !same_image(&config.logical_id, &directory_id(&store.logical)?)
        || !same_image(&config.private_id, &directory_id(&store.private)?)
        || !same_image(
            &config.private_id,
            &directory_id(&open_directory(&store.private_path, FILE_READ_ATTRIBUTES)?)?,
        )
    {
        return Err(invalid("Parent slot binding changed"));
    }
    let mut key = ptr::null_mut();
    if unsafe {
        DuplicateHandle(
            child.process.as_raw_handle() as HANDLE,
            config.registry_root,
            windows_sys::Win32::System::Threading::GetCurrentProcess(),
            &mut key,
            0,
            0,
            DUPLICATE_SAME_ACCESS,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let key = RegistryKey(key);
    if unsafe { windows_sys::Win32::Foundation::CompareObjectHandles(key.0, store.registry.0) } == 0
    {
        return Err(invalid("Parent registry binding changed"));
    }
    Ok(store)
}

#[cfg(test)]
impl BootstrapLibrary {
    fn prepare_owned_helper<'a>(
        &'a self,
        parent: &'a PendingEducationBootstrap,
        proof: &tests::FixtureReceipt,
    ) -> io::Result<OwnedHelperTransaction<'a>> {
        if !parent.resume_attempted {
            return Err(invalid("Helper requires an attempted parent"));
        }
        let store = checked_slot_binding(parent)?;
        let path = directory_path(
            self._directories
                .last()
                .ok_or_else(|| invalid("Missing payload root"))?,
        )?
        .join("bootstrap-ordering.exe");
        let image = pin_image(&path)?;
        if !same_image(&image_id(&image)?, &image_id(&parent._image)?) {
            return Err(invalid("Only the fixed original own image may be a helper"));
        }
        let mut copied = store.duplicate()?;
        copied.singleton_caller_rva = 0;
        let child = create_unstarted_impl(&path, image, Some(copied), false, Some(proof))?;
        let status = tests::prepare_fixture(self, &child, proof, true);
        if status != 0 {
            return Err(io::Error::from_raw_os_error(status as i32));
        }
        self.enroll(&child)?;
        parent.identity()?;
        let transaction = OwnedHelperTransaction {
            child,
            parent,
            _payload: self,
        };
        transaction.verify()?;
        Ok(transaction)
    }
}

#[cfg(test)]
impl OwnedHelperTransaction<'_> {
    fn verify(&self) -> io::Result<()> {
        let parent = checked_slot_binding(self.parent)?;
        let helper = checked_slot_binding(&self.child)?;
        if parent.slot != helper.slot
            || helper.singleton_caller_rva != 0
            || self.child.observation.is_some()
            || !same_image(
                &directory_id(&parent.private)?,
                &directory_id(&helper.private)?,
            )
            || !same_image(
                &directory_id(&parent.logical)?,
                &directory_id(&helper.logical)?,
            )
            || !same_image(
                &image_id(&self.parent._image)?,
                &image_id(&self.child._image)?,
            )
        {
            return Err(invalid("Helper slot/image binding changed"));
        }
        Ok(())
    }
    fn resume(&mut self) -> io::Result<()> {
        self.verify()?;
        self.child.resume_once()
    }
}

impl SlotStorage {
    fn duplicate(&self) -> io::Result<Self> {
        let mut key = ptr::null_mut();
        if unsafe {
            DuplicateHandle(
                windows_sys::Win32::System::Threading::GetCurrentProcess(),
                self.registry.0,
                windows_sys::Win32::System::Threading::GetCurrentProcess(),
                &mut key,
                0,
                0,
                DUPLICATE_SAME_ACCESS,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        let registry = RegistryKey(key);
        Ok(Self {
            slot: self.slot,
            logical: self.logical.try_clone()?,
            private: self.private.try_clone()?,
            temporary: [
                self.temporary[0].try_clone()?,
                self.temporary[1].try_clone()?,
            ],
            registry,
            user_key_path: self.user_key_path,
            private_path: self.private_path.clone(),
            singleton_caller_rva: self.singleton_caller_rva,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, GetExitCodeProcess};

    // Writable receipt exists only for our cooperative C++ control, never the game.
    pub(super) struct FixtureReceipt(pub(super) OwnedHandle);
    impl FixtureReceipt {
        fn new() -> Self {
            let raw = unsafe {
                CreateFileMappingW(
                    INVALID_HANDLE_VALUE,
                    ptr::null(),
                    PAGE_READWRITE,
                    0,
                    2064,
                    ptr::null(),
                )
            };
            assert!(!raw.is_null());
            let owned = unsafe { OwnedHandle::from_raw_handle(raw as _) };
            assert_ne!(
                unsafe { SetHandleInformation(raw, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT) },
                0
            );
            Self(owned)
        }
        pub(super) fn environment(&self, environment: Vec<u16>) -> io::Result<Vec<u16>> {
            let mut rows: Vec<Vec<u16>> = environment
                .split(|&unit| unit == 0)
                .filter(|row| !row.is_empty())
                .map(|row| row.to_vec())
                .collect();
            rows.push(
                format!(
                    "MIVUBI_BOOTSTRAP_TEST_SECTION={}",
                    self.0.as_raw_handle() as usize
                )
                .encode_utf16()
                .collect(),
            );
            rows.sort_by(|a, b| ordinal_names(environment_name(a), environment_name(b)).unwrap());
            let mut result = Vec::new();
            for row in rows {
                result.extend(row);
                result.push(0);
            }
            result.push(0);
            if result.len() > 32767 {
                return Err(invalid("Fixture environment too large"));
            }
            Ok(result)
        }
    }

    type FixtureEntry = unsafe extern "system" fn(HANDLE, HANDLE, u32, u64, HANDLE, HANDLE) -> u32;
    fn fixture_digests(path: &Path) -> PayloadDigests {
        PayloadDigests {
            enroller: payload_digest(&pin_image(&path.join("bootstrap-enroll.dll")).unwrap())
                .unwrap(),
            compatibility: payload_digest(&pin_image(&path.join("bootstrap-compat.dll")).unwrap())
                .unwrap(),
        }
    }
    pub(super) fn prepare_fixture(
        library: &BootstrapLibrary,
        child: &PendingEducationBootstrap,
        proof: &FixtureReceipt,
        expect_enrolled: bool,
    ) -> u32 {
        let symbol = if expect_enrolled {
            c"PrepareOwnedFixtureForEnrollment"
        } else {
            c"PrepareOwnedFixture"
        };
        let entry = unsafe { GetProcAddress(library.module, symbol.as_ptr().cast()) }.unwrap();
        let entry: FixtureEntry = unsafe { mem::transmute(entry) };
        unsafe {
            entry(
                child.process.as_raw_handle() as HANDLE,
                child._thread.as_raw_handle() as HANDLE,
                child.pid,
                child.created,
                child.configuration.as_ref().unwrap().as_raw_handle() as HANDLE,
                proof.0.as_raw_handle() as HANDLE,
            )
        }
    }
    fn rejected_enrollment(
        library: &BootstrapLibrary,
        child: &PendingEducationBootstrap,
        birth: u64,
        image: &File,
    ) -> u32 {
        unsafe {
            (library.entry)(
                child.process.as_raw_handle() as HANDLE,
                child._thread.as_raw_handle() as HANDLE,
                child.pid,
                birth,
                child.configuration.as_ref().unwrap().as_raw_handle() as HANDLE,
                image.as_raw_handle() as HANDLE,
                ptr::null_mut(),
            )
        }
    }

    #[test]
    fn registered_enrollment_refuses_unreviewed_payload_before_game_lookup() {
        assert!(matches!(prepare_registered_enrolled(), Err(error)
            if error.kind() == io::ErrorKind::Unsupported));
    }

    #[test]
    fn five_slot_policy_rejects_unreviewed_or_out_of_range_sets() {
        assert!(
            matches!(registered_slots::<5>(true), Err(error) if error.kind() == io::ErrorKind::Unsupported)
        );
        assert!(registered_slots::<0>(true).is_err());
        assert!(registered_slots::<6>(true).is_err());
        let id = FILE_ID_INFO {
            VolumeSerialNumber: 7,
            FileId: windows_sys::Win32::Storage::FileSystem::FILE_ID_128 {
                Identifier: [3; 16],
            },
        };
        let leaves: Vec<_> = (1..=5)
            .map(|slot| registry_leaf(&id, slot).unwrap())
            .collect();
        for (index, leaf) in leaves.iter().enumerate() {
            assert!(leaf.ends_with(&format!("_{}", char::from(b'A' + index as u8))));
        }
        assert_eq!(
            leaves
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            5
        );
        assert!(registry_leaf(&id, 0).is_err());
        assert!(registry_leaf(&id, 6).is_err());
    }

    #[test]
    fn registered_observation_requires_replay_policy_before_storage_lookup() {
        // Test policy is deliberately absent. Refuse before Known Folders,
        // registry access or fresh profile allocation; never involve the game.
        assert!(matches!(registered_slots::<2>(true), Err(error)
            if error.kind() == io::ErrorKind::Unsupported));
    }

    #[test]
    #[ignore = "Requires the explicitly built own C++ payload"]
    fn bootstrap_payload_rejects_mismatched_hashes_and_pins_loaded_files() {
        let exe = PathBuf::from(std::env::var_os("MIVUBI_BOOTSTRAP_CPP_READER").unwrap())
            .canonicalize()
            .unwrap();
        let root = exe.parent().unwrap();
        let reviewed = fixture_digests(root);
        let mut wrong = reviewed;
        wrong.enroller[0] ^= 1;
        assert!(BootstrapLibrary::load(root, wrong).is_err());
        let mut wrong = reviewed;
        wrong.compatibility[0] ^= 1;
        assert!(BootstrapLibrary::load(root, wrong).is_err());
        let library = BootstrapLibrary::load(root, reviewed).unwrap();
        for leaf in ["bootstrap-enroll.dll", "bootstrap-compat.dll"] {
            assert!(OpenOptions::new()
                .write(true)
                .open(root.join(leaf))
                .is_err());
        }
        drop(library);
    }

    #[test]
    fn startup_observation_requires_config_and_never_reads_a_live_writer() {
        let path = std::env::current_exe().unwrap().canonicalize().unwrap();
        assert!(create_unstarted_impl(&path, pin_image(&path).unwrap(), None, true, None).is_err());
        let [a, _b] = test_slots();
        let mut child =
            create_unstarted_impl(&path, pin_image(&path).unwrap(), Some(a), true, None).unwrap();
        assert_eq!(
            child.observation_bytes_after_exit().unwrap_err().kind(),
            io::ErrorKind::WouldBlock
        );
        child.cancel().unwrap();
        assert_eq!(
            child.observation_bytes_after_exit().unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }

    fn clone_store(store: &SlotStorage) -> SlotStorage {
        store.duplicate().unwrap()
    }

    fn finish_fixture(child: &PendingEducationBootstrap) {
        // Only this fixed owned fixture can be resumed in the test module.
        assert_eq!(
            unsafe {
                windows_sys::Win32::System::Threading::ResumeThread(
                    child._thread.as_raw_handle() as HANDLE
                )
            },
            1
        );
        check_finished_fixture(child);
    }

    fn check_finished_fixture(child: &PendingEducationBootstrap) {
        assert_eq!(
            unsafe { WaitForSingleObject(child.process.as_raw_handle() as HANDLE, 10000) },
            WAIT_OBJECT_0
        );
        let mut code = 1;
        assert_ne!(
            unsafe { GetExitCodeProcess(child.process.as_raw_handle() as HANDLE, &mut code) },
            0
        );
        assert_eq!(code, 0);
        // Query in the unhooked parent, never through the child's translated cached key.
        use windows_sys::Win32::System::Registry::{RegOpenKeyExW, RegQueryValueExW, REG_DWORD};
        let name = wide(OsStr::new(&format!(
            "Software\\EducationEntry_{}_{}",
            child.pid, child.created
        )))
        .unwrap();
        let mut host = ptr::null_mut();
        assert_eq!(
            unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, name.as_ptr(), 0, KEY_READ, &mut host) },
            0
        );
        let host = RegistryKey(host);
        let mut marker = 0u32;
        let mut kind = 0;
        let mut bytes = mem::size_of_val(&marker) as u32;
        assert_eq!(
            unsafe {
                RegQueryValueExW(
                    host.0,
                    [77u16, 97, 114, 107, 101, 114, 0].as_ptr(),
                    ptr::null(),
                    &mut kind,
                    &mut marker as *mut _ as _,
                    &mut bytes,
                )
            },
            0
        );
        assert_eq!((kind, bytes, marker), (REG_DWORD, 4, 0x484f5354));
    }

    #[test]
    #[ignore = "Requires the explicitly built own C++ fixture and test-only enroller DLL"]
    fn rust_enrolls_own_fixture_before_dependency_tls_and_main() {
        let path = PathBuf::from(
            std::env::var_os("MIVUBI_BOOTSTRAP_CPP_READER")
                .expect("Set the trusted built C++ fixture path"),
        )
        .canonicalize()
        .unwrap();
        assert_eq!(path.file_name(), Some(OsStr::new("bootstrap-ordering.exe")));
        let root = path.parent().unwrap();
        let enroller = BootstrapLibrary::load(root, fixture_digests(root)).unwrap();
        let [a, b] = test_slots();
        for store in [&a, &b] {
            std::fs::write(
                directory_path(&store.logical)
                    .unwrap()
                    .join("Roaming/rust-enrolled.txt"),
                b"HOST",
            )
            .unwrap();
        }
        // A mismatched birth is rejected before mutation; original child remains cancellable.
        let proof = FixtureReceipt::new();
        let mut rejected = create_unstarted_impl(
            &path,
            pin_image(&path).unwrap(),
            Some(clone_store(&a)),
            false,
            Some(&proof),
        )
        .unwrap();
        assert_ne!(
            rejected_enrollment(&enroller, &rejected, rejected.created + 1, &rejected._image),
            0
        );
        rejected.identity().unwrap();
        rejected.cancel().unwrap();
        // The same live handles for a different executable are rejected before any canary creation.
        let self_path = std::env::current_exe().unwrap().canonicalize().unwrap();
        let proof = FixtureReceipt::new();
        let mut wrong_image = create_unstarted_impl(
            &self_path,
            pin_image(&self_path).unwrap(),
            Some(clone_store(&a)),
            false,
            Some(&proof),
        )
        .unwrap();
        assert_ne!(
            rejected_enrollment(
                &enroller,
                &wrong_image,
                wrong_image.created,
                &pin_image(&path).unwrap()
            ),
            0
        );
        wrong_image.identity().unwrap();
        wrong_image.cancel().unwrap();
        // Positive unhooked control must observe all three host registry canaries.
        let proof = FixtureReceipt::new();
        let baseline = create_unstarted_impl(
            &path,
            pin_image(&path).unwrap(),
            Some(clone_store(&a)),
            false,
            Some(&proof),
        )
        .unwrap();
        assert_eq!(prepare_fixture(&enroller, &baseline, &proof, false), 0);
        finish_fixture(&baseline);
        let mut known_callsite = 0;
        let mut saved_journals = Vec::new();
        for round in 1..=2 {
            let mut first_store = clone_store(&a);
            let mut second_store = clone_store(&b);
            let host_mutex = if round == 2 {
                first_store.singleton_caller_rva = known_callsite;
                second_store.singleton_caller_rva = known_callsite;
                assert_ne!(known_callsite, 0);
                let id = directory_id(&a.logical).unwrap().FileId.Identifier;
                let part = u16::from_le_bytes([id[0], id[1]]);
                let mut bytes = [0u8; 8];
                bytes[..6].copy_from_slice(&id[2..8]);
                let tail = u64::from_le_bytes(bytes);
                let name = wide(OsStr::new(&format!(
                    "feedface-{part:04x}-4000-8000-{tail:012x}"
                )))
                .unwrap();
                let raw = unsafe {
                    windows_sys::Win32::System::Threading::CreateMutexW(
                        ptr::null(),
                        0,
                        name.as_ptr(),
                    )
                };
                assert!(!raw.is_null());
                Some(unsafe { OwnedHandle::from_raw_handle(raw as _) })
            } else {
                None
            };
            let first_proof = FixtureReceipt::new();
            let second_proof = FixtureReceipt::new();
            let first = create_unstarted_impl(
                &path,
                pin_image(&path).unwrap(),
                Some(first_store),
                true,
                Some(&first_proof),
            )
            .unwrap();
            let second = create_unstarted_impl(
                &path,
                pin_image(&path).unwrap(),
                Some(second_store),
                true,
                Some(&second_proof),
            )
            .unwrap();
            assert_eq!(prepare_fixture(&enroller, &first, &first_proof, true), 0);
            enroller.enroll(&first).unwrap();
            assert_eq!(prepare_fixture(&enroller, &second, &second_proof, true), 0);
            enroller.enroll(&second).unwrap();
            let transaction = PreparedEducationEnrollment {
                _children: [first, second],
                _payload: BootstrapLibrary::load(root, fixture_digests(root)).unwrap(),
            }
            .resume_for_authorized_observation()
            .unwrap();
            assert!(transaction
                .attempts
                .iter()
                .all(|a| matches!(a, StartupAttempt::Resumed)));
            assert!(transaction.save_after_exit(0).is_err());
            assert!(transaction.save_after_exit(3).is_err());
            for child in &transaction.children {
                check_finished_fixture(child);
            }
            // Each process lifetime saves a fresh journal; repeated saves of one lifetime still refuse.
            assert_eq!(observe_until_exit(&transaction).unwrap(), 0);
            for child in &transaction.children {
                let snapshot = child.observation_bytes_after_exit().unwrap();
                let count = u32::from_le_bytes(snapshot[48..52].try_into().unwrap()) as usize;
                let callsite = (0..count)
                    .find_map(|i| {
                        let at = 64 + i * 496;
                        let word = |offset| {
                            u32::from_le_bytes(
                                snapshot[at + offset..at + offset + 4].try_into().unwrap(),
                            )
                        };
                        (word(4) == 6 && word(480) == 1).then(|| word(484))
                    })
                    .expect("Own executable CreateMutexA callsite");
                if known_callsite == 0 {
                    known_callsite = callsite;
                }
                assert_eq!(callsite, known_callsite);
                assert_eq!(u32::from_le_bytes(snapshot[48..52].try_into().unwrap()), 64);
                assert_eq!(u32::from_le_bytes(snapshot[52..56].try_into().unwrap()), 1);
                let slot = child.storage.as_ref().unwrap().slot;
                let path = child.observation_path().unwrap();
                assert_eq!(std::fs::read(&path).unwrap(), snapshot);
                saved_journals.push((path, snapshot));
                assert_eq!(
                    transaction.save_after_exit(slot).err().unwrap().kind(),
                    io::ErrorKind::AlreadyExists
                );
                assert_eq!(
                    std::fs::metadata(child.observation_path().unwrap())
                        .unwrap()
                        .len(),
                    OBSERVATION_BYTES as u64
                );
            }
            if let Some(host) = &host_mutex {
                assert_eq!(
                    unsafe { WaitForSingleObject(host.as_raw_handle() as HANDLE, 0) },
                    WAIT_OBJECT_0
                );
                assert_ne!(
                    unsafe {
                        windows_sys::Win32::System::Threading::ReleaseMutex(
                            host.as_raw_handle() as HANDLE
                        )
                    },
                    0
                );
            }
            for store in [&a, &b] {
                assert_eq!(
                    std::fs::read_to_string(store.private_path.join("Roaming/rust-enrolled.txt"))
                        .unwrap(),
                    format!("{}:{round}", store.slot)
                );
                assert_eq!(
                    std::fs::read(
                        directory_path(&store.logical)
                            .unwrap()
                            .join("Roaming/rust-enrolled.txt")
                    )
                    .unwrap(),
                    b"HOST"
                );
            }
        }
        assert_eq!(saved_journals.len(), 4);
        for (index, (path, bytes)) in saved_journals.iter().enumerate() {
            assert!(saved_journals[..index]
                .iter()
                .all(|(previous, _)| previous != path));
            assert_eq!(&std::fs::read(path).unwrap(), bytes);
        }
    }

    // Test-only duplicates of the original owned fixture handles; bounded panic cleanup
    // can terminate only this control, never a game or a PID-opened process.
    struct FixtureLifetime {
        process: OwnedHandle,
        thread: OwnedHandle,
    }
    impl FixtureLifetime {
        fn new(child: &PendingEducationBootstrap) -> Self {
            fn copy(raw: HANDLE) -> OwnedHandle {
                let mut result = ptr::null_mut();
                assert_ne!(
                    unsafe {
                        DuplicateHandle(
                            GetCurrentProcess(),
                            raw,
                            GetCurrentProcess(),
                            &mut result,
                            0,
                            0,
                            DUPLICATE_SAME_ACCESS,
                        )
                    },
                    0
                );
                unsafe { OwnedHandle::from_raw_handle(result as _) }
            }
            Self {
                process: copy(child.process.as_raw_handle() as HANDLE),
                thread: copy(child._thread.as_raw_handle() as HANDLE),
            }
        }
        fn raw(&self) -> HANDLE {
            self.process.as_raw_handle() as HANDLE
        }
    }
    impl Drop for FixtureLifetime {
        fn drop(&mut self) {
            if unsafe { WaitForSingleObject(self.raw(), 0) } != WAIT_OBJECT_0 {
                unsafe {
                    TerminateProcess(self.raw(), 91);
                    WaitForSingleObject(self.raw(), 5000);
                }
            }
        }
    }

    fn fixture_barrier(
        child: &PendingEducationBootstrap,
        proof: &FixtureReceipt,
    ) -> (OwnedHandle, OwnedHandle) {
        use windows_sys::Win32::System::Threading::CreateEventW;
        let view = unsafe {
            MapViewOfFile(
                proof.0.as_raw_handle() as HANDLE,
                FILE_MAP_WRITE,
                0,
                0,
                2064,
            )
        };
        assert!(!view.Value.is_null());
        let events = [2032usize, 2056usize].map(|offset| {
            let raw = unsafe { CreateEventW(ptr::null(), 1, 0, ptr::null()) };
            assert!(!raw.is_null());
            let own = unsafe { OwnedHandle::from_raw_handle(raw as _) };
            let mut remote = ptr::null_mut();
            assert_ne!(
                unsafe {
                    DuplicateHandle(
                        GetCurrentProcess(),
                        raw,
                        child.process.as_raw_handle() as HANDLE,
                        &mut remote,
                        0,
                        0,
                        DUPLICATE_SAME_ACCESS,
                    )
                },
                0
            );
            unsafe {
                ptr::write((view.Value as *mut u8).add(offset) as *mut HANDLE, remote);
            }
            own
        });
        assert_ne!(unsafe { UnmapViewOfFile(view) }, 0);
        let [ready, finish] = events;
        (ready, finish)
    }

    #[test]
    #[ignore = "Requires the explicitly built own C++ payload"]
    fn owned_descendant_handoff_preserves_provenance_and_reuses_slot() {
        use owned_handoff::Request;
        use windows_sys::Win32::System::Threading::SetEvent;
        let path = PathBuf::from(std::env::var_os("MIVUBI_BOOTSTRAP_CPP_READER").unwrap())
            .canonicalize()
            .unwrap();
        let library = BootstrapLibrary::load(
            path.parent().unwrap(),
            fixture_digests(path.parent().unwrap()),
        )
        .unwrap();
        let helper_proofs = [FixtureReceipt::new(), FixtureReceipt::new()];
        let mut parents = Vec::new();
        let mut requests = Vec::new();
        let mut held = Vec::new();
        let mut barriers = Vec::new();
        for (index, store) in test_slots().into_iter().enumerate() {
            std::fs::write(
                directory_path(&store.logical)
                    .unwrap()
                    .join("Roaming/rust-enrolled.txt"),
                b"HOST",
            )
            .unwrap();
            let proof = FixtureReceipt::new();
            let mut parent = create_unstarted_impl(
                &path,
                pin_image(&path).unwrap(),
                Some(store),
                false,
                Some(&proof),
            )
            .unwrap();
            assert_eq!(prepare_fixture(&library, &parent, &proof, true), 0);
            library.enroll(&parent).unwrap();
            barriers.push(fixture_barrier(&parent, &proof));
            let request = Request::new(&parent, &proof, &helper_proofs[index]).unwrap();
            held.push(FixtureLifetime::new(&parent));
            parent.resume_once().unwrap();
            request.wait(&parent).unwrap();
            parents.push(parent);
            requests.push(request);
        }
        assert!(library
            .accept_owned_descendant(&parents[0], &mut requests[0], &helper_proofs[1])
            .is_err());
        requests[0].packet().created.dwLowDateTime ^= 1;
        assert!(library
            .accept_owned_descendant(&parents[0], &mut requests[0], &helper_proofs[0])
            .is_err());
        requests[0].packet().created.dwLowDateTime ^= 1;
        // Substitute a complete live peer-descendant tuple. PID/thread/birth/image all match;
        // native ancestry must still reject it without killing the peer or either sender.
        let peer = requests[1].packet();
        let peer_tuple = (peer.process, peer.thread, peer.pid, peer.created);
        let mut imported = Vec::new();
        let mut remote = Vec::new();
        for original in [peer_tuple.0, peer_tuple.1] {
            let mut local = ptr::null_mut();
            assert_ne!(
                unsafe {
                    DuplicateHandle(
                        parents[1].process.as_raw_handle() as HANDLE,
                        original,
                        GetCurrentProcess(),
                        &mut local,
                        0,
                        0,
                        DUPLICATE_SAME_ACCESS,
                    )
                },
                0
            );
            imported.push(unsafe { OwnedHandle::from_raw_handle(local as _) });
            let mut in_sender = ptr::null_mut();
            assert_ne!(
                unsafe {
                    DuplicateHandle(
                        GetCurrentProcess(),
                        local,
                        parents[0].process.as_raw_handle() as HANDLE,
                        &mut in_sender,
                        0,
                        0,
                        DUPLICATE_SAME_ACCESS,
                    )
                },
                0
            );
            remote.push(in_sender);
        }
        let first = requests[0].packet();
        let original = (first.process, first.thread, first.pid, first.created);
        first.process = remote[0];
        first.thread = remote[1];
        first.pid = peer_tuple.2;
        first.created = peer_tuple.3;
        assert!(library
            .accept_owned_descendant(&parents[0], &mut requests[0], &helper_proofs[0])
            .is_err());
        assert!(parents.iter().all(|parent| parent.identity().is_ok()));
        assert_eq!(
            unsafe { WaitForSingleObject(imported[0].as_raw_handle() as HANDLE, 0) },
            WAIT_TIMEOUT
        );
        let first = requests[0].packet();
        first.process = original.0;
        first.thread = original.1;
        first.pid = original.2;
        first.created = original.3;
        for index in 0..2 {
            let mut helper = library
                .accept_owned_descendant(
                    &parents[index],
                    &mut requests[index],
                    &helper_proofs[index],
                )
                .unwrap();
            let lifetime = FixtureLifetime::new(&helper.child);
            assert!(library
                .accept_owned_descendant(
                    &parents[index],
                    &mut requests[index],
                    &helper_proofs[index]
                )
                .is_err());
            assert_eq!(
                unsafe { WaitForSingleObject(barriers[index].0.as_raw_handle() as HANDLE, 10000) },
                WAIT_OBJECT_0
            );
            assert_eq!(
                helper.child.storage.as_ref().unwrap().slot,
                index as u32 + 1
            );
            helper.resume().unwrap();
            check_finished_fixture(&helper.child);
            assert!(parents.iter().all(|parent| parent.identity().is_ok()));
            assert_eq!(
                std::fs::read_to_string(
                    parents[index]
                        .storage
                        .as_ref()
                        .unwrap()
                        .private_path
                        .join("Roaming/rust-enrolled.txt")
                )
                .unwrap(),
                format!("{}:2", index + 1)
            );
            drop(helper);
            drop(lifetime);
        }
        for (parent, barrier) in parents.iter().zip(&barriers) {
            assert_ne!(unsafe { SetEvent(barrier.1.as_raw_handle() as HANDLE) }, 0);
            check_finished_fixture(parent);
        }
        drop(held);
    }

    #[test]
    #[ignore = "Requires the explicitly built own C++ payload"]
    fn owned_descendant_refusal_before_and_after_publication_preserves_data() {
        let path = PathBuf::from(std::env::var_os("MIVUBI_BOOTSTRAP_CPP_READER").unwrap())
            .canonicalize()
            .unwrap();
        let library = BootstrapLibrary::load(
            path.parent().unwrap(),
            fixture_digests(path.parent().unwrap()),
        )
        .unwrap();
        for (index, store) in test_slots().into_iter().enumerate() {
            std::fs::write(
                directory_path(&store.logical)
                    .unwrap()
                    .join("Roaming/rust-enrolled.txt"),
                b"HOST",
            )
            .unwrap();
            let private = store.private_path.clone();
            let proof = FixtureReceipt::new();
            let helper_proof = FixtureReceipt::new();
            let mut parent = create_unstarted_impl(
                &path,
                pin_image(&path).unwrap(),
                Some(store),
                false,
                Some(&proof),
            )
            .unwrap();
            assert_eq!(prepare_fixture(&library, &parent, &proof, true), 0);
            library.enroll(&parent).unwrap();
            let mut request = owned_handoff::Request::new(&parent, &proof, &helper_proof).unwrap();
            let lifetime = FixtureLifetime::new(&parent);
            if index == 0 {
                drop(request); // Sender sees refusal before it may create a descendant.
                parent.resume_once().unwrap();
            } else {
                parent.resume_once().unwrap();
                request.wait(&parent).unwrap();
                let mut raw = ptr::null_mut();
                assert_ne!(
                    unsafe {
                        DuplicateHandle(
                            parent.process.as_raw_handle() as HANDLE,
                            request.packet().process,
                            GetCurrentProcess(),
                            &mut raw,
                            0,
                            0,
                            DUPLICATE_SAME_ACCESS,
                        )
                    },
                    0
                );
                let descendant = unsafe { OwnedHandle::from_raw_handle(raw as _) };
                drop(request); // Sender still owns and cancels this original suspended descendant.
                assert_eq!(unsafe { WaitForSingleObject(raw, 10000) }, WAIT_OBJECT_0);
                let mut code = 0;
                assert_ne!(unsafe { GetExitCodeProcess(raw, &mut code) }, 0);
                assert_eq!(code, 90);
                drop(descendant);
            }
            assert_eq!(
                unsafe { WaitForSingleObject(parent.process.as_raw_handle() as HANDLE, 10000) },
                WAIT_OBJECT_0
            );
            assert_eq!(parent.exit_code_after_exit().unwrap(), 86); // Fixed fixture reports a refused handoff.
            assert_eq!(
                std::fs::read_to_string(private.join("Roaming/rust-enrolled.txt")).unwrap(),
                format!("{}:1", index + 1)
            );
            assert_eq!(
                std::fs::read(
                    directory_path(&parent.storage.as_ref().unwrap().logical)
                        .unwrap()
                        .join("Roaming/rust-enrolled.txt")
                )
                .unwrap(),
                b"HOST"
            );
            drop(lifetime);
        }
    }

    #[test]
    #[ignore = "Requires the explicitly built own C++ payload"]
    fn manager_owned_helper_transaction_keeps_slot_and_rejects_changed_bindings() {
        use windows_sys::Win32::System::Threading::SetEvent;
        let path = PathBuf::from(std::env::var_os("MIVUBI_BOOTSTRAP_CPP_READER").unwrap())
            .canonicalize()
            .unwrap();
        let root = path.parent().unwrap();
        let library = BootstrapLibrary::load(root, fixture_digests(root)).unwrap();
        let stores = test_slots();
        let mut parents = Vec::new();
        let mut held = Vec::new();
        let mut barriers = Vec::new();
        for store in stores {
            std::fs::write(
                directory_path(&store.logical)
                    .unwrap()
                    .join("Roaming/rust-enrolled.txt"),
                b"HOST",
            )
            .unwrap();
            let proof = FixtureReceipt::new();
            let mut parent = create_unstarted_impl(
                &path,
                pin_image(&path).unwrap(),
                Some(store),
                false,
                Some(&proof),
            )
            .unwrap();
            assert_eq!(prepare_fixture(&library, &parent, &proof, true), 0);
            library.enroll(&parent).unwrap();
            assert!(library
                .prepare_owned_helper(&parent, &FixtureReceipt::new())
                .is_err());
            let barrier = fixture_barrier(&parent, &proof);
            held.push(FixtureLifetime::new(&parent));
            parent.resume_once().unwrap();
            assert_eq!(
                unsafe { WaitForSingleObject(barrier.0.as_raw_handle() as HANDLE, 10000) },
                WAIT_OBJECT_0
            );
            parents.push(parent);
            barriers.push(barrier);
        }
        for index in 0..2 {
            parents[index].created ^= 1;
            assert!(library
                .prepare_owned_helper(&parents[index], &FixtureReceipt::new())
                .is_err());
            parents[index].created ^= 1;
            let original_image = mem::replace(
                &mut parents[index]._image,
                pin_image(&std::env::current_exe().unwrap()).unwrap(),
            );
            assert!(library
                .prepare_owned_helper(&parents[index], &FixtureReceipt::new())
                .is_err());
            parents[index]._image = original_image;
            let mut foreign = parents[1 - index]
                .storage
                .as_ref()
                .unwrap()
                .duplicate()
                .unwrap();
            mem::swap(
                &mut parents[index].storage.as_mut().unwrap().registry,
                &mut foreign.registry,
            );
            assert!(library
                .prepare_owned_helper(&parents[index], &FixtureReceipt::new())
                .is_err());
            mem::swap(
                &mut parents[index].storage.as_mut().unwrap().registry,
                &mut foreign.registry,
            );
            let original_path = mem::replace(
                &mut parents[index].storage.as_mut().unwrap().private_path,
                foreign.private_path.clone(),
            );
            assert!(library
                .prepare_owned_helper(&parents[index], &FixtureReceipt::new())
                .is_err());
            parents[index].storage.as_mut().unwrap().private_path = original_path;
            let abandoned_proof = FixtureReceipt::new();
            let abandoned = library
                .prepare_owned_helper(&parents[index], &abandoned_proof)
                .unwrap();
            let abandoned_lifetime = FixtureLifetime::new(&abandoned.child);
            drop(abandoned);
            assert_eq!(
                unsafe { WaitForSingleObject(abandoned_lifetime.raw(), 10000) },
                WAIT_OBJECT_0
            );
            let mut abandoned_code = 0;
            assert_ne!(
                unsafe { GetExitCodeProcess(abandoned_lifetime.raw(), &mut abandoned_code) },
                0
            );
            assert_eq!(abandoned_code, 90);
            assert!(parents.iter().all(|parent| parent.identity().is_ok()));
            let proof = FixtureReceipt::new();
            let mut helper = library
                .prepare_owned_helper(&parents[index], &proof)
                .unwrap();
            let helper_lifetime = FixtureLifetime::new(&helper.child);
            let config = read_config(&helper.child);
            assert_ne!(
                helper.child.identity().unwrap(),
                parents[index].identity().unwrap()
            );
            assert_eq!(
                (config.slot, config.scope, config.singleton_caller_rva),
                (index as u32 + 1, 1, 0)
            );
            assert!(helper.child.observation.is_none());
            helper.child.created ^= 1;
            assert!(helper.resume().is_err());
            assert!(!helper.child.resume_attempted);
            helper.child.created ^= 1;
            let slot = helper.child.storage.as_ref().unwrap().slot;
            helper.child.storage.as_mut().unwrap().slot = 3 - slot;
            assert!(helper.resume().is_err());
            assert!(!helper.child.resume_attempted);
            helper.child.storage.as_mut().unwrap().slot = slot;
            helper.resume().unwrap();
            check_finished_fixture(&helper.child);
            assert!(helper.resume().is_err());
            assert!(parents.iter().all(|parent| parent.identity().is_ok()));
            let store = parents[index].storage.as_ref().unwrap();
            assert_eq!(
                std::fs::read_to_string(store.private_path.join("Roaming/rust-enrolled.txt"))
                    .unwrap(),
                format!("{}:2", store.slot)
            );
            assert_eq!(
                std::fs::read(
                    directory_path(&store.logical)
                        .unwrap()
                        .join("Roaming/rust-enrolled.txt")
                )
                .unwrap(),
                b"HOST"
            );
            drop(helper);
            drop(helper_lifetime);
        }
        for (parent, barrier) in parents.iter().zip(&barriers) {
            let proof = FixtureReceipt::new();
            let mut helper = library.prepare_owned_helper(parent, &proof).unwrap();
            let lifetime = FixtureLifetime::new(&helper.child);
            assert_ne!(unsafe { SetEvent(barrier.1.as_raw_handle() as HANDLE) }, 0);
            check_finished_fixture(parent);
            assert!(helper.resume().is_err());
            assert!(!helper.child.resume_attempted);
            drop(helper);
            assert_eq!(
                unsafe { WaitForSingleObject(lifetime.raw(), 10000) },
                WAIT_OBJECT_0
            );
            let mut code = 0;
            assert_ne!(unsafe { GetExitCodeProcess(lifetime.raw(), &mut code) }, 0);
            assert_eq!(code, 90);
        }
        drop(held);
    }

    #[test]
    #[ignore = "Requires the explicitly built own C++ payload"]
    fn observation_transaction_preserves_attempted_children_on_partial_resume_and_drop() {
        use windows_sys::Win32::System::Threading::{ResumeThread, SetEvent, SuspendThread};
        let path = PathBuf::from(std::env::var_os("MIVUBI_BOOTSTRAP_CPP_READER").unwrap())
            .canonicalize()
            .unwrap();
        assert_eq!(path.file_name(), Some(OsStr::new("bootstrap-ordering.exe")));
        let root = path.parent().unwrap();
        let library = BootstrapLibrary::load(root, fixture_digests(root)).unwrap();
        let [a, b] = test_slots();
        let proofs = [FixtureReceipt::new(), FixtureReceipt::new()];
        let children = [a, b]
            .into_iter()
            .zip(&proofs)
            .map(|(store, proof)| {
                std::fs::write(
                    directory_path(&store.logical)
                        .unwrap()
                        .join("Roaming/rust-enrolled.txt"),
                    b"HOST",
                )
                .unwrap();
                let child = create_unstarted_impl(
                    &path,
                    pin_image(&path).unwrap(),
                    Some(store),
                    true,
                    Some(proof),
                )
                .unwrap();
                assert_eq!(prepare_fixture(&library, &child, proof, true), 0);
                library.enroll(&child).unwrap();
                child
            })
            .collect::<Vec<_>>();
        let children: [PendingEducationBootstrap; 2] = children.try_into().ok().unwrap();
        let held = children.each_ref().map(FixtureLifetime::new);
        let barriers = [
            fixture_barrier(&children[0], &proofs[0]),
            fixture_barrier(&children[1], &proofs[1]),
        ];
        // Force the second call to report "still suspended", after the first has started.
        assert_eq!(
            unsafe { SuspendThread(children[1]._thread.as_raw_handle() as HANDLE) },
            1
        );
        let mut transaction = PreparedEducationEnrollment {
            _children: children,
            _payload: library,
        }
        .resume_for_authorized_observation()
        .unwrap();
        assert!(matches!(transaction.attempts[0], StartupAttempt::Resumed));
        assert!(matches!(
            transaction.attempts[1],
            StartupAttempt::Unconfirmed(_)
        ));
        assert_eq!(
            unsafe { WaitForSingleObject(barriers[0].0.as_raw_handle() as HANDLE, 10000) },
            WAIT_OBJECT_0
        );
        assert_eq!(
            transaction.save_after_exit(1).err().unwrap().kind(),
            io::ErrorKind::WouldBlock
        );
        for child in &mut transaction.children {
            assert!(child.cancel().is_err());
            assert!(child.resume_once().is_err());
        }
        drop(transaction);
        for lifetime in &held {
            assert_eq!(
                unsafe { WaitForSingleObject(lifetime.raw(), 0) },
                WAIT_TIMEOUT
            );
        }
        // Explicit recovery/normal finish only for our controlled executable.
        assert_eq!(
            unsafe { ResumeThread(held[1].thread.as_raw_handle() as HANDLE) },
            1
        );
        assert_eq!(
            unsafe { WaitForSingleObject(barriers[1].0.as_raw_handle() as HANDLE, 10000) },
            WAIT_OBJECT_0
        );
        for (_, finish) in &barriers {
            assert_ne!(unsafe { SetEvent(finish.as_raw_handle() as HANDLE) }, 0);
        }
        for lifetime in &held {
            assert_eq!(
                unsafe { WaitForSingleObject(lifetime.raw(), 10000) },
                WAIT_OBJECT_0
            );
            let mut exit = 1;
            assert_ne!(unsafe { GetExitCodeProcess(lifetime.raw(), &mut exit) }, 0);
            assert_eq!(exit, 0);
        }
    }

    #[test]
    #[ignore = "Requires the explicitly built own C++ payload"]
    fn transaction_preflight_and_not_attempted_children_keep_unstarted_cleanup() {
        use windows_sys::Win32::System::Threading::SuspendThread;
        let path = PathBuf::from(std::env::var_os("MIVUBI_BOOTSTRAP_CPP_READER").unwrap())
            .canonicalize()
            .unwrap();
        let root = path.parent().unwrap();
        for preflight_failure in [true, false] {
            // No execution or enrollment in these ownership failure branches.
            let mut children =
                [0, 1].map(|_| create_unstarted(&path, pin_image(&path).unwrap(), None).unwrap());
            let held = children.each_ref().map(FixtureLifetime::new);
            if preflight_failure {
                children[1].cancel().unwrap();
            } else {
                assert_eq!(
                    unsafe { SuspendThread(children[0]._thread.as_raw_handle() as HANDLE) },
                    1
                );
            }
            let prepared = PreparedEducationEnrollment {
                _children: children,
                _payload: BootstrapLibrary::load(root, fixture_digests(root)).unwrap(),
            };
            let started = prepared.resume_for_authorized_observation();
            if preflight_failure {
                assert!(started.is_err());
            } else {
                let started = started.unwrap();
                assert!(matches!(
                    started.attempts[0],
                    StartupAttempt::Unconfirmed(_)
                ));
                assert!(matches!(started.attempts[1], StartupAttempt::NotAttempted));
                drop(started);
            }
            assert_eq!(
                unsafe { WaitForSingleObject(held[1].raw(), 5000) },
                WAIT_OBJECT_0
            );
            assert_eq!(
                unsafe { WaitForSingleObject(held[0].raw(), 0) },
                if preflight_failure {
                    WAIT_OBJECT_0
                } else {
                    WAIT_TIMEOUT
                }
            );
        }
    }

    #[test]
    fn tracked_startup_restores_retry_only_before_creation_or_after_confirmed_exit() {
        let path = std::env::current_exe().unwrap().canonicalize().unwrap();
        let before_creation = UnstartedCleanup::new();
        assert!(create_unstarted_tracked(Path::new("relative.exe"), pin_image(&path).unwrap(),
            None, false, None, Some(before_creation.clone()), None).is_err());
        assert!(before_creation.retry_safe());

        let missing = UnstartedCleanup::new();
        let absent = path.parent().unwrap().join(format!("absent-startup-{}.exe", std::process::id()));
        assert!(!absent.exists());
        assert!(create_unstarted_tracked(&absent, pin_image(&path).unwrap(),
            None, false, None, Some(missing.clone()), None).is_err());
        assert!(missing.retry_safe());

        let cleanup = UnstartedCleanup::new();
        let child = create_unstarted_tracked(&path, pin_image(&path).unwrap(),
            None, false, None, Some(cleanup.clone()), None).unwrap();
        let process = child.process.try_clone().unwrap();
        assert!(!cleanup.retry_safe());
        drop(child); // Own test executable remains suspended throughout.
        assert!(cleanup.retry_safe());
        assert_eq!(unsafe { WaitForSingleObject(process.as_raw_handle() as _, 0) }, WAIT_OBJECT_0);
    }

    #[test]
    fn unconfirmed_cancellation_and_resume_boundary_never_restore_retry() {
        let path = std::env::current_exe().unwrap().canonicalize().unwrap();
        for attempted in [false, true] {
            let cleanup = UnstartedCleanup::new();
            let mut child = create_unstarted_tracked(&path, pin_image(&path).unwrap(),
                None, false, None, Some(cleanup.clone()), None).unwrap();
            let full = child.process.try_clone().unwrap();
            if attempted {
                // Simulate only the irreversible boundary; never resume this fixture.
                child.resume_attempted = true;
            } else {
                let mut limited = ptr::null_mut();
                assert_ne!(unsafe { DuplicateHandle(GetCurrentProcess(), child.process.as_raw_handle() as _,
                    GetCurrentProcess(), &mut limited,
                    windows_sys::Win32::System::Threading::SYNCHRONIZATION_SYNCHRONIZE,
                    0, 0) }, 0);
                child.process = unsafe { OwnedHandle::from_raw_handle(limited as _) };
            }
            drop(child);
            assert!(!cleanup.retry_safe());
            assert_eq!(unsafe { WaitForSingleObject(full.as_raw_handle() as _, 0) }, WAIT_TIMEOUT);
            // Only the retained full-access handle to the own, never-resumed fixture.
            assert_ne!(unsafe { TerminateProcess(full.as_raw_handle() as _, 91) }, 0);
            assert_eq!(unsafe { WaitForSingleObject(full.as_raw_handle() as _, 5000) }, WAIT_OBJECT_0);
            assert!(!cleanup.retry_safe()); // External teardown is not a receipt.
        }
    }

    #[test]
    fn duplicated_storage_pins_keep_the_same_profile_after_failed_creation() {
        let parent = fresh_root(&std::env::temp_dir()).unwrap();
        let logical_path = parent.join("logical");
        std::fs::create_dir(&logical_path).unwrap();
        for name in ["Roaming", "Local"] { std::fs::create_dir(logical_path.join(name)).unwrap(); }
        let logical = open_directory(&logical_path, FILE_READ_ATTRIBUTES).unwrap();
        let store = create_private_slot(&logical, &parent, 1).unwrap();
        let leaf = registry_leaf(&directory_id(&store.private).unwrap(), 1).unwrap();
        let sentinel = directory_path(&store.private).unwrap().join("Roaming/world-fixture.bin");
        std::fs::write(&sentinel, b"preserve").unwrap();
        let duplicate = store.duplicate().unwrap();
        assert!(same_image(&directory_id(&store.private).unwrap(), &directory_id(&duplicate.private).unwrap()));
        assert!(same_image(&directory_id(&store.logical).unwrap(), &directory_id(&duplicate.logical).unwrap()));
        assert_eq!(store.private_path, duplicate.private_path);
        assert_eq!(store.user_key_path, duplicate.user_key_path);
        for index in 0..2 {
            assert!(same_image(&directory_id(&store.temporary[index]).unwrap(), &directory_id(&duplicate.temporary[index]).unwrap()));
        }
        let path = std::env::current_exe().unwrap().canonicalize().unwrap();
        let cleanup = UnstartedCleanup::new();
        assert!(create_unstarted_tracked(Path::new("relative.exe"), pin_image(&path).unwrap(),
            Some(duplicate), false, None, Some(cleanup.clone()), None).is_err());
        assert!(cleanup.retry_safe());
        let retry = store.duplicate().unwrap();
        assert_eq!(std::fs::read(&sentinel).unwrap(), b"preserve");
        assert!(same_image(&directory_id(&store.private).unwrap(), &directory_id(&retry.private).unwrap()));
        drop(retry); drop(store); drop(logical);
        let key = wide(OsStr::new(&format!("Software\\MivubiEducationPilot\\{leaf}"))).unwrap();
        assert_eq!(unsafe { windows_sys::Win32::System::Registry::RegDeleteTreeW(HKEY_CURRENT_USER, key.as_ptr()) }, 0);
        assert_eq!(parent.parent(), Some(std::env::temp_dir().as_path()));
        std::fs::remove_dir_all(parent).unwrap();
    }

    fn test_slots() -> [SlotStorage; 2] {
        let parent = fresh_root(&std::env::temp_dir()).unwrap();
        let logical = parent.join("host-view");
        std::fs::create_dir(&logical).unwrap();
        for leaf in ["Roaming", "Local"] {
            std::fs::create_dir(logical.join(leaf)).unwrap();
        }
        fresh_slots(
            open_directory(&logical, FILE_READ_ATTRIBUTES).unwrap(),
            &parent,
        )
        .unwrap()
    }

    #[test]
    fn slot_captions_use_stable_numbers_and_reject_foreign_windows() {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            CreateWindowExW, DestroyWindow, GetWindowTextW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
        };
        for slot in 1..=MAX_EDUCATION_SLOTS {
            assert_eq!(
                String::from_utf16_lossy(&slot_caption(slot).unwrap()),
                format!("Minecraft - {slot}\0")
            );
        }
        assert!(slot_caption(0).is_err());
        assert!(slot_caption(6).is_err());
        let class = wide(OsStr::new("STATIC")).unwrap();
        let title = wide(OsStr::new("Owned caption control")).unwrap();
        let hwnd = unsafe {
            CreateWindowExW(
                0,
                class.as_ptr(),
                title.as_ptr(),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                0,
                0,
                240,
                80,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null(),
            )
        };
        assert!(!hwnd.is_null());
        let outcome = std::panic::catch_unwind(|| {
            assert!(
                label_slot_window(hwnd, unsafe { GetCurrentProcessId() }.wrapping_add(1), 1)
                    .is_err()
            );
            label_slot_window(hwnd, unsafe { GetCurrentProcessId() }, 2).unwrap();
            let mut current = [0u16; 32];
            let length = unsafe { GetWindowTextW(hwnd, current.as_mut_ptr(), 32) };
            assert_eq!(
                String::from_utf16_lossy(&current[..length as usize]),
                "Minecraft - 2"
            );
            label_slot_window(hwnd, unsafe { GetCurrentProcessId() }, 2).unwrap();
        });
        assert_ne!(unsafe { DestroyWindow(hwnd) }, 0);
        assert!(label_slot_window(hwnd, unsafe { GetCurrentProcessId() }, 1).is_err());
        if let Err(error) = outcome {
            std::panic::resume_unwind(error);
        }
    }

    #[test]
    #[ignore = "Requires the explicitly built own C++ payload"]
    fn observation_reports_nonzero_child_exits_even_when_journals_save() {
        let path = PathBuf::from(std::env::var_os("MIVUBI_BOOTSTRAP_CPP_READER").unwrap())
            .canonicalize()
            .unwrap();
        let root = path.parent().unwrap();
        let enroller = BootstrapLibrary::load(root, fixture_digests(root)).unwrap();
        let children = test_slots().map(|store| {
            // Force an existing receiver validation error after successful attach, not a game run.
            let name = wide(OsStr::new("RustEnrolledRound")).unwrap();
            let value = 2u32;
            assert_eq!(
                unsafe {
                    windows_sys::Win32::System::Registry::RegSetValueExW(
                        store.registry.0,
                        name.as_ptr(),
                        0,
                        windows_sys::Win32::System::Registry::REG_DWORD,
                        &value as *const u32 as _,
                        4,
                    )
                },
                0
            );
            let proof = FixtureReceipt::new();
            let child = create_unstarted_impl(
                &path,
                pin_image(&path).unwrap(),
                Some(store),
                true,
                Some(&proof),
            )
            .unwrap();
            assert_eq!(prepare_fixture(&enroller, &child, &proof, true), 0);
            enroller.enroll(&child).unwrap();
            child
        });
        let transaction = PreparedEducationEnrollment {
            _children: children,
            _payload: enroller,
        }
        .resume_for_authorized_observation()
        .unwrap();
        for child in &transaction.children {
            assert_eq!(
                unsafe { WaitForSingleObject(child.process.as_raw_handle() as HANDLE, 10000) },
                WAIT_OBJECT_0
            );
            assert_eq!(child.exit_code_after_exit().unwrap(), 86);
        }
        assert_eq!(observe_until_exit(&transaction).unwrap(), 1);
        for child in &transaction.children {
            assert_eq!(
                std::fs::metadata(child.observation_path().unwrap())
                    .unwrap()
                    .len(),
                OBSERVATION_BYTES as u64
            );
        }
    }

    #[test]
    fn five_slot_replay_preserves_pair_and_rejects_replaced_extra_slot() {
        use sha2::{Digest, Sha256};
        let pair = test_slots();
        let root = pair[0].private_path.parent().unwrap().to_path_buf();
        let parent = root.parent().unwrap();
        let mut pair_bytes = [0u8; 216];
        for (at, value) in [(0, 0x52504c31u32), (4, 1), (8, 216)] {
            pair_bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
        }
        for (i, unit) in root.file_name().unwrap().encode_wide().enumerate() {
            pair_bytes[16 + i * 2..18 + i * 2].copy_from_slice(&unit.to_le_bytes());
        }
        let root_file = open_directory(&root, FILE_READ_ATTRIBUTES).unwrap();
        for (i, file) in [&root_file, &pair[0].private, &pair[1].private]
            .iter()
            .enumerate()
        {
            let id = directory_id(file).unwrap();
            let at = 144 + i * 24;
            pair_bytes[at..at + 8].copy_from_slice(&id.VolumeSerialNumber.to_le_bytes());
            pair_bytes[at + 8..at + 24].copy_from_slice(&id.FileId.Identifier);
        }
        let original_paths = pair.each_ref().map(|store| store.private_path.clone());
        let mut stores = Vec::from(pair);
        for slot in 3..=5 {
            stores.push(create_private_slot(&stores[0].logical, &root, slot).unwrap());
        }
        for store in &stores {
            std::fs::write(
                store.private_path.join("Roaming/five-slot-sentinel.txt"),
                format!("slot-{}", store.slot),
            )
            .unwrap();
        }
        let bytes = five_replay_bytes(&pair_bytes, &stores).unwrap();
        assert_eq!(&bytes[16..216], &pair_bytes[16..216]);
        let reopened = reopen_slots_for::<5>(
            stores[0].logical.try_clone().unwrap(),
            parent,
            &bytes,
            Sha256::digest(&bytes).into(),
        )
        .unwrap();
        for (index, store) in reopened.iter().enumerate() {
            assert_eq!(store.slot, index as u32 + 1);
            assert_eq!(
                std::fs::read(store.private_path.join("Roaming/five-slot-sentinel.txt")).unwrap(),
                format!("slot-{}", index + 1).as_bytes()
            );
            if index < 2 {
                assert_eq!(store.private_path, original_paths[index]);
            }
        }
        assert!(create_private_slot(&stores[0].logical, &root, 3).is_err());
        let legacy = reopen_slots(
            stores[0].logical.try_clone().unwrap(),
            parent,
            &pair_bytes,
            Sha256::digest(pair_bytes).into(),
        )
        .unwrap();
        assert_eq!(legacy[0].private_path, original_paths[0]);
        assert_eq!(legacy[1].private_path, original_paths[1]);
        let mut replaced = bytes.clone();
        replaced[216] ^= 1;
        assert!(reopen_slots_for::<5>(
            stores[0].logical.try_clone().unwrap(),
            parent,
            &replaced,
            Sha256::digest(&replaced).into()
        )
        .is_err());
        assert!(reopen_slots_for::<5>(
            stores[0].logical.try_clone().unwrap(),
            parent,
            &pair_bytes,
            Sha256::digest(pair_bytes).into()
        )
        .is_err());
    }

    #[test]
    fn replay_reopens_only_the_reviewed_existing_pair() {
        use sha2::{Digest, Sha256};
        let stores = test_slots();
        let root = stores[0].private_path.parent().unwrap().to_path_buf();
        let parent = root.parent().unwrap();
        let mut bytes = [0u8; 216];
        for (at, value) in [(0, 0x52504c31u32), (4, 1), (8, 216)] {
            bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
        }
        for (i, unit) in root.file_name().unwrap().encode_wide().enumerate() {
            bytes[16 + i * 2..18 + i * 2].copy_from_slice(&unit.to_le_bytes());
        }
        let root_file = open_directory(&root, FILE_READ_ATTRIBUTES).unwrap();
        for (i, file) in [&root_file, &stores[0].private, &stores[1].private]
            .iter()
            .enumerate()
        {
            let id = directory_id(file).unwrap();
            let at = 144 + i * 24;
            bytes[at..at + 8].copy_from_slice(&id.VolumeSerialNumber.to_le_bytes());
            bytes[at + 8..at + 24].copy_from_slice(&id.FileId.Identifier);
        }
        let expected: [u8; 32] = Sha256::digest(bytes).into();
        for store in &stores {
            std::fs::write(
                store.private_path.join("Local/Temp/replay-temp-canary.bin"),
                format!("slot-{}", store.slot),
            )
            .unwrap();
        }
        assert!(reopen_slots(
            stores[0].logical.try_clone().unwrap(),
            parent,
            &bytes,
            [0; 32]
        )
        .is_err());
        let reopened = reopen_slots(
            stores[0].logical.try_clone().unwrap(),
            parent,
            &bytes,
            expected,
        )
        .unwrap();
        for i in 0..2 {
            assert!(same_image(
                &directory_id(&stores[i].private).unwrap(),
                &directory_id(&reopened[i].private).unwrap()
            ));
            assert_eq!(stores[i].private_path, reopened[i].private_path);
            assert!(same_image(
                &directory_id(&stores[i].temporary[1]).unwrap(),
                &directory_id(&reopened[i].temporary[1]).unwrap()
            ));
            assert_eq!(
                std::fs::read(
                    reopened[i]
                        .private_path
                        .join("Local/Temp/replay-temp-canary.bin")
                )
                .unwrap(),
                format!("slot-{}", i + 1).as_bytes()
            );
        }
        let mut replaced = bytes;
        replaced[168] ^= 1;
        let expected = Sha256::digest(replaced).into();
        assert!(reopen_slots(
            stores[0].logical.try_clone().unwrap(),
            parent,
            &replaced,
            expected
        )
        .is_err());
        // A missing directory must fail, not recreate an empty profile.
        drop(reopened);
        let local = stores[0].private_path.join("Local");
        let logical = stores[0].logical.try_clone().unwrap();
        std::fs::remove_file(local.join("Temp/replay-temp-canary.bin")).unwrap();
        assert!(std::fs::remove_dir(local.join("Temp")).is_err()); // Live storage pins protect it.
        drop(stores);
        std::fs::remove_dir(local.join("Temp")).unwrap();
        std::fs::remove_dir(&local).unwrap();
        assert!(reopen_slots(logical, parent, &bytes, Sha256::digest(bytes).into()).is_err());
        assert!(!local.exists());
    }

    fn read_config(child: &PendingEducationBootstrap) -> EducationConfig {
        let section = child.configuration.as_ref().unwrap().as_raw_handle() as HANDLE;
        let writable = unsafe {
            MapViewOfFile(
                section,
                FILE_MAP_WRITE,
                0,
                0,
                mem::size_of::<EducationConfig>(),
            )
        };
        assert!(writable.Value.is_null());
        let view = unsafe {
            MapViewOfFile(
                section,
                FILE_MAP_READ,
                0,
                0,
                mem::size_of::<EducationConfig>(),
            )
        };
        assert!(!view.Value.is_null());
        let config = unsafe { ptr::read(view.Value as *const EducationConfig) };
        assert_ne!(unsafe { UnmapViewOfFile(view) }, 0);
        config
    }

    #[test]
    fn configured_slots_publish_exact_child_bindings_and_exclude_an_ambient_handle() {
        use windows_sys::Win32::{
            Foundation::CompareObjectHandles, Security::SECURITY_ATTRIBUTES,
            System::Threading::CreateEventW,
        };
        let attributes = SECURITY_ATTRIBUTES {
            nLength: mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: ptr::null_mut(),
            bInheritHandle: 1,
        };
        let ambient = unsafe { CreateEventW(&attributes, 1, 0, ptr::null()) };
        assert!(!ambient.is_null());
        let ambient = unsafe { OwnedHandle::from_raw_handle(ambient as _) };
        let [a, b] = test_slots();
        assert!(!same_image(
            &directory_id(&a.private).unwrap(),
            &directory_id(&b.private).unwrap()
        ));
        let path = std::env::current_exe().unwrap().canonicalize().unwrap();
        let mut first = create_unstarted(&path, pin_image(&path).unwrap(), Some(a)).unwrap();
        let mut second = create_unstarted(&path, pin_image(&path).unwrap(), Some(b)).unwrap();
        for (slot, child) in [(1, &first), (2, &second)] {
            let c = read_config(child);
            assert_eq!(
                (
                    c.pid,
                    ((c.created.dwHighDateTime as u64) << 32) | c.created.dwLowDateTime as u64
                ),
                child.identity().unwrap()
            );
            assert_eq!(c.slot, slot);
            let storage = child.storage.as_ref().unwrap();
            assert!(same_image(
                &c.private_id,
                &directory_id(&storage.private).unwrap()
            ));
            // An inherited handle keeps its value. A coincidental reused child value
            // is accepted only if it names a different kernel object.
            let mut copy = ptr::null_mut();
            if unsafe {
                DuplicateHandle(
                    child.process.as_raw_handle() as HANDLE,
                    ambient.as_raw_handle() as HANDLE,
                    GetCurrentProcess(),
                    &mut copy,
                    0,
                    0,
                    DUPLICATE_SAME_ACCESS,
                )
            } != 0
            {
                let copy = unsafe { OwnedHandle::from_raw_handle(copy as _) };
                assert_eq!(
                    unsafe {
                        CompareObjectHandles(
                            ambient.as_raw_handle() as HANDLE,
                            copy.as_raw_handle() as HANDLE,
                        )
                    },
                    0
                );
            }
        }
        first.cancel().unwrap();
        second.cancel().unwrap();
    }

    #[test]
    fn nested_backing_is_refused_without_reusing_or_deleting_the_logical_view() {
        let parent = fresh_root(&std::env::temp_dir()).unwrap();
        let marker = parent.join("host-marker");
        std::fs::write(&marker, b"HOST").unwrap();
        let logical = open_directory(&parent, FILE_READ_ATTRIBUTES).unwrap();
        assert!(fresh_slots(logical, &parent).is_err());
        assert_eq!(std::fs::read(marker).unwrap(), b"HOST");
    }

    #[test]
    #[ignore = "Requires the explicitly built bootstrap-ordering.exe C++ reader"]
    fn cpp_receiver_consumes_rust_configuration_for_both_slots_across_relaunch() {
        use windows_sys::Win32::System::Threading::ResumeThread;
        let path = PathBuf::from(
            std::env::var_os("MIVUBI_BOOTSTRAP_CPP_READER")
                .expect("Set the trusted built C++ fixture path"),
        )
        .canonicalize()
        .unwrap();
        assert_eq!(path.file_name(), Some(OsStr::new("bootstrap-ordering.exe")));
        let [a, b] = test_slots();
        for round in 1..=2 {
            let first =
                create_unstarted(&path, pin_image(&path).unwrap(), Some(clone_store(&a))).unwrap();
            let second =
                create_unstarted(&path, pin_image(&path).unwrap(), Some(clone_store(&b))).unwrap();
            // Test-only resume: fixed trusted C++ fixture, no game target or injection.
            for child in [&first, &second] {
                assert_eq!(
                    unsafe { ResumeThread(child._thread.as_raw_handle() as HANDLE) },
                    1
                );
            }
            for child in [&first, &second] {
                assert_eq!(
                    unsafe { WaitForSingleObject(child.process.as_raw_handle() as HANDLE, 10000) },
                    WAIT_OBJECT_0
                );
                let mut code = 1;
                assert_ne!(
                    unsafe {
                        GetExitCodeProcess(child.process.as_raw_handle() as HANDLE, &mut code)
                    },
                    0
                );
                assert_eq!(code, 0);
            }
            for store in [&a, &b] {
                assert_eq!(
                    std::fs::read_to_string(store.private_path.join("Roaming/rust-bridge.txt"))
                        .unwrap(),
                    format!("{}:{round}", store.slot)
                );
            }
        }
    }

    #[test]
    #[ignore = "Requires the explicitly built own C++ payload; never activates windows"]
    fn focus_lease_reaches_cpp_creation_and_drop_stops_without_killing_children() {
        use windows_sys::Win32::System::Threading::SetEvent;
        let path = PathBuf::from(std::env::var_os("MIVUBI_BOOTSTRAP_CPP_READER").unwrap())
            .canonicalize()
            .unwrap();
        assert_eq!(path.file_name(), Some(OsStr::new("bootstrap-ordering.exe")));
        let root = path.parent().unwrap();
        let library = BootstrapLibrary::load(root, fixture_digests(root)).unwrap();
        // Registered pacing must not allocate fresh account-bearing slots.
        assert!(
            matches!(prepare_registered_mode::<2>(false, Some(&library)), Err(error)
            if error.kind() == io::ErrorKind::InvalidData)
        );
        // A policy request without storage refuses before process creation.
        assert!(create_unstarted_configured(
            &path,
            pin_image(&path).unwrap(),
            None,
            false,
            Some(&library),
            None
        )
        .is_err());
        let proofs = [FixtureReceipt::new(), FixtureReceipt::new()];
        let mut children = Vec::new();
        let mut barriers = Vec::new();
        let mut lifetimes = Vec::new();
        for (store, proof) in test_slots().into_iter().zip(&proofs) {
            let mut child = create_unstarted_configured(
                &path,
                pin_image(&path).unwrap(),
                Some(store),
                false,
                Some(&library),
                Some(proof),
            )
            .unwrap();
            assert!(child.policy.is_some());
            assert_eq!(prepare_fixture(&library, &child, proof, true), 0);
            barriers.push(fixture_barrier(&child, proof));
            lifetimes.push(FixtureLifetime::new(&child));
            library.enroll(&child).unwrap();
            child.resume_once().unwrap();
            children.push(child);
        }
        // Both hidden renderers must complete actual child-side cap validation,
        // wrapping and background pacing before the producer lease is released.
        for (child, (ready, _)) in children.iter().zip(&barriers) {
            let waits = [
                ready.as_raw_handle() as HANDLE,
                child.process.as_raw_handle() as HANDLE,
            ];
            assert_eq!(
                unsafe {
                    windows_sys::Win32::System::Threading::WaitForMultipleObjects(
                        2,
                        waits.as_ptr(),
                        0,
                        10000,
                    )
                },
                WAIT_OBJECT_0
            );
        }
        for child in &mut children {
            let lease = child.policy.take().unwrap();
            let readers = lease.reader_handles().unwrap();
            drop(lease);
            assert_eq!(
                unsafe { WaitForSingleObject(readers[2].as_raw_handle() as HANDLE, 0) },
                WAIT_OBJECT_0
            );
            assert!(child.identity().is_ok());
        }
        for (_, finish) in &barriers {
            assert_ne!(unsafe { SetEvent(finish.as_raw_handle() as HANDLE) }, 0);
        }
        for child in &children {
            assert_eq!(
                unsafe { WaitForSingleObject(child.process.as_raw_handle() as HANDLE, 10000) },
                WAIT_OBJECT_0
            );
            assert_eq!(child.exit_code_after_exit().unwrap(), 0);
        }
        drop(children);
        drop(lifetimes);
        // Last private lease stops the sole worker. Join before other owner tests.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            match instance_focus_events::stop_observer() {
                Ok(report) => {
                    assert!(!report.enabled);
                    break;
                }
                Err(error)
                    if error.kind() == io::ErrorKind::WouldBlock
                        && std::time::Instant::now() < deadline =>
                {
                    std::thread::sleep(std::time::Duration::from_millis(5))
                }
                other => panic!("Focus worker cleanup failed: {other:?}"),
            }
        }
    }

    #[test]
    fn original_child_is_pinned_unstarted_and_cancellable() {
        let path = std::env::current_exe().unwrap().canonicalize().unwrap();
        let image = pin_image(&path).unwrap();
        let id = image_id(&image).unwrap();
        let mut child = create_unstarted(&path, image, None).unwrap();
        let identity = child.identity().unwrap();
        assert_ne!(identity.0, unsafe { GetCurrentProcessId() });
        assert!(identity.1 > 0);
        assert!(same_image(&id, &image_id(&child._image).unwrap()));
        child.cancel().unwrap();
        assert!(child.identity().is_err());
        let mut code = 0;
        assert_ne!(
            unsafe { GetExitCodeProcess(child.process.as_raw_handle() as HANDLE, &mut code) },
            0
        );
        assert_eq!(code, 90); // The test runner never ran in the suspended child.
        assert_eq!(
            unsafe { WaitForSingleObject(GetCurrentProcess(), 0) },
            WAIT_TIMEOUT
        );
    }

    #[test]
    fn dropped_preparation_cancels_only_its_original_child() {
        use windows_sys::Win32::Foundation::{DuplicateHandle, DUPLICATE_SAME_ACCESS};
        let path = std::env::current_exe().unwrap().canonicalize().unwrap();
        let child = create_unstarted(&path, pin_image(&path).unwrap(), None).unwrap();
        let current = unsafe { GetCurrentProcess() };
        let mut duplicate = ptr::null_mut();
        assert_ne!(
            unsafe {
                DuplicateHandle(
                    current,
                    child.process.as_raw_handle() as HANDLE,
                    current,
                    &mut duplicate,
                    0,
                    0,
                    DUPLICATE_SAME_ACCESS,
                )
            },
            0
        );
        let retained = unsafe { OwnedHandle::from_raw_handle(duplicate as _) };
        drop(child);
        assert_eq!(
            unsafe { WaitForSingleObject(retained.as_raw_handle() as HANDLE, 0) },
            WAIT_OBJECT_0
        );
    }

    #[test]
    fn malformed_executable_strings_are_refused() {
        for value in ["", "x\0y", "x\"y"] {
            assert!(wide(OsStr::new(value)).is_err());
        }
    }
}
