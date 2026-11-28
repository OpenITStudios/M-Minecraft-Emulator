//! Shared native Education bootstrap for the retained session controller and
//! explicit historical startup observation. Reviewed image, payload and
//! profile admission remain mandatory; test fixtures do not authorize gameplay.
#![allow(dead_code)]

use crate::engine::instance_focus_events;
use crate::engine::manager_settings::PerformanceSettings;
#[cfg(test)]
use windows_sys::Win32::System::Registry::HKEY_CURRENT_USER;
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
        GetFileInformationByHandleEx, FILE_ATTRIBUTE_DIRECTORY,
        FILE_ATTRIBUTE_REPARSE_POINT, FILE_ID_INFO, FILE_SHARE_READ,
    },
    System::RemoteDesktop::ProcessIdToSessionId,
    System::Threading::{
        CreateProcessW, GetCurrentProcessId, GetProcessId, GetProcessIdOfThread,
        IsWow64Process2, TerminateProcess, WaitForSingleObject,
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
    let attributes = crate::engine::windows_file_identity::attributes(file)?;
    if attributes & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT) != 0 {
        return Err(invalid("Bootstrap image must be an ordinary file"));
    }
    crate::engine::windows_file_identity::file_id(file)
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
    #[cfg(feature="mc02-native-input")]
    mc02_input: Option<mc02_input::Channel>,
    resume_attempted: bool,
    cleanup: Option<UnstartedCleanup>,
}

impl PendingEducationBootstrap {
    pub(super) fn identity(&self) -> io::Result<(u32, u64)> {
        let process = self.process.as_raw_handle() as HANDLE;
        // Query only the retained newly created process, never an existing PID.
        let birth = crate::engine::windows_process_time::creation_filetime(process)?;
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

// Test-only conveniences retain the fixture's original argument defaults.
// Production callers use the single suspended-process creation owner directly.
#[cfg(test)]
fn create_fixture_unstarted(
    path: &Path,
    image: File,
    storage: Option<SlotStorage>,
) -> io::Result<PendingEducationBootstrap> {
    create_fixture_observed(path, image, storage, false, None)
}

#[cfg(test)]
fn create_fixture_observed(
    path: &Path,
    image: File,
    storage: Option<SlotStorage>,
    observe: bool,
    proof: Option<&tests::FixtureReceipt>,
) -> io::Result<PendingEducationBootstrap> {
    create_suspended_process(path, image, storage, observe, None, None,
        #[cfg(feature="mc02-native-input")] None,
        proof)
}

#[cfg(test)]
fn create_fixture_configured(
    path: &Path,
    image: File,
    storage: Option<SlotStorage>,
    observe: bool,
    presentation_library: Option<&BootstrapLibrary>,
    proof: Option<&tests::FixtureReceipt>,
) -> io::Result<PendingEducationBootstrap> {
    create_fixture_tracked(path, image, storage, observe, presentation_library, None, proof)
}

#[cfg(test)]
fn create_fixture_tracked(
    path: &Path,
    image: File,
    storage: Option<SlotStorage>,
    observe: bool,
    presentation_library: Option<&BootstrapLibrary>,
    cleanup: Option<UnstartedCleanup>,
    proof: Option<&tests::FixtureReceipt>,
) -> io::Result<PendingEducationBootstrap> {
    let presentation_policy = presentation_library
        .map(|library| crate::engine::manager_settings::load_performance().map(|limits|(library,limits)))
        .transpose()?;
    create_suspended_process(path, image, storage, observe, presentation_policy, cleanup,
        #[cfg(feature="mc02-native-input")] None,
        proof)
}

// Only explicit launch intent from the authenticated controller may enable
// experimental input; all registered observation and fixture callers pass None.
fn create_suspended_process(
    path: &Path,
    image: File,
    storage: Option<SlotStorage>,
    observe: bool,
    presentation_policy: Option<(&BootstrapLibrary, PerformanceSettings)>,
    cleanup: Option<UnstartedCleanup>,
    #[cfg(feature="mc02-native-input")] experimental_input: Option<crate::engine::native_clients_session::AgentControlMode>,
    #[cfg(test)] proof: Option<&tests::FixtureReceipt>,
) -> io::Result<PendingEducationBootstrap> {
    if presentation_policy.is_some() && storage.is_none() {
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
    let presentation = presentation_policy
        .map(|_| Configuration::create_sized(128))
        .transpose()?;
    let observation = if observe {
        Some(StartupObservation::create()?)
    } else {
        None
    };
    #[cfg(feature="mc02-native-input")]
    let mc02_input=if let Some(store)=&storage {
        mc02_input::Channel::create(store.slot,experimental_input)?
    } else {None};
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
    #[cfg(feature="mc02-native-input")]
    if let Some(input)=&mc02_input { inherited.push(input.reader()); }
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
            #[cfg(feature="mc02-native-input")]
            mc02_input.as_ref().map(|i|i.reader()),
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
        #[cfg(feature="mc02-native-input")]
        mc02_input,
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
    #[cfg(feature="mc02-native-input")]
    if let Some(input)=child.mc02_input.as_mut() {
        input.bind(child.pid,child.created,session)?;
    }
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
    let actual = crate::engine::windows_process_image::query(
        child.process.as_raw_handle() as HANDLE,
    )?;
    use std::{ffi::OsString, os::windows::ffi::OsStringExt};
    let actual_path = std::path::PathBuf::from(OsString::from_wide(&actual));
    if !same_image(&expected, &image_id(&pin_image(&actual_path)?)?) {
        return Err(invalid(
            "Created bootstrap process image does not match pinned target",
        ));
    }
    if let (Some(config), Some(store)) = (configuration, storage) {
        child.configuration = Some(config.publish(&child, &store)?);
        child.storage = Some(store);
    }
    if let (Some(config), Some((library, limits))) = (&presentation, presentation_policy) {
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
        library.prepare_presentation(&child, config, limits)?;
    }
    child.identity()?;
    Ok(child)
}

const MAX_EDUCATION_SLOTS: u32 = 5;

#[path = "independent_bootstrap/game_preset.rs"]
mod game_preset;

#[path = "independent_bootstrap/legacy_startup_observation.rs"]
mod legacy_startup_observation;

#[path = "independent_bootstrap/session_controller.rs"]
mod session_controller;
#[cfg(feature="mc02-native-input")]
#[path = "independent_bootstrap/mc02_input.rs"]
mod mc02_input;
#[path = "independent_bootstrap/session_audio.rs"]
mod session_audio;
#[path = "independent_bootstrap/window_caption.rs"]
mod window_caption;
#[path = "independent_bootstrap/registered_image.rs"]
mod registered_image;

pub(super) fn education_installation_report() -> io::Result<super::EducationInstallationReport> {
    registered_image::installation_report()
}
#[path = "independent_bootstrap/profile_storage.rs"]
mod profile_storage;
#[path = "independent_bootstrap/resource_pack_inventory.rs"]
mod resource_pack_inventory;
#[path = "independent_bootstrap/profile_binding.rs"]
pub(super) mod profile_binding;
#[path = "independent_bootstrap/product_profiles.rs"]
pub(super) mod product_profiles;

// Explicit legacy observation uses reviewed profiles and the reviewed payload;
// there is no registered fresh-profile preparation route.
fn prepare_registered_observation<const N: usize>(
    payload: &BootstrapLibrary,
) -> io::Result<[PendingEducationBootstrap; N]> {
    if N != 2 && N != 5 {
        return Err(invalid("Unsupported Education slot count"));
    }
    let (target, image, singleton_caller_rva) = registered_image::registered_image(true)?;
    let path = Path::new(&target);
    let limits = crate::engine::manager_settings::load_performance()?;
    let stores = registered_slots::<N>()?;
    let changed = game_preset::apply(&stores)?;
    use std::io::Write;
    writeln!(io::stdout(), "{{\"event\":\"graphics-preset\",\"preset\":\"Simple12\",\"changedProfiles\":{changed}}}")?;
    let mut children = Vec::with_capacity(N);
    for mut store in stores {
        store.singleton_caller_rva = singleton_caller_rva;
        children.push(create_suspended_process(
            path,
            image.try_clone()?,
            Some(store),
            true,
            Some((payload, limits)),
            None,
            #[cfg(feature="mc02-native-input")]
            None,
            #[cfg(test)]
            None,
        )?);
    }
    children.try_into().map_err(|_| invalid("Incomplete Education process set"))
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

#[path = "independent_bootstrap/reviewed_bootstrap_payload.rs"]
mod reviewed_bootstrap_payload;
use self::reviewed_bootstrap_payload::BootstrapLibrary;
#[cfg(test)]
use self::reviewed_bootstrap_payload::{payload_digest, PayloadDigests};

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
    let children = prepare_registered_observation::<N>(&payload)?;
    for child in &children {
        payload.enroll(child)?;
    }
    Ok(PreparedEducationEnrollment {
        _children: children,
        _payload: payload,
    })
}

/// Dispatch internal product modes before UI startup; historical diagnostics
/// have their own explicitly named CLI owner.
pub(super) fn maybe_run_native_entrypoint() -> bool {
    #[cfg(feature = "lazy-tester-package")]
    if std::env::args_os().skip(1).eq([OsString::from("--lazy-tester-migrate-data")]) {
        use std::io::Write;
        let result=std::thread::spawn(product_profiles::migrate_reviewed).join().unwrap_or_else(|_|Err(io::Error::other("Data migration task failed")));
        let ready=result.is_ok();
        let value=match result {Ok(())=>serde_json::json!({"schema":1,"configured":true,"migratedProfiles":5,"sourcePreserved":true,"gameLaunched":false}),Err(error)=>serde_json::json!({"error":error.to_string(),"sourcePreserved":true,"gameLaunched":false})};
        let _=writeln!(io::stdout(),"{value}");std::process::exit(if ready{0}else{2});
    }
    #[cfg(feature = "lazy-tester-package")]
    if std::env::args_os().skip(1).eq([OsString::from("--lazy-tester-initialize")]) {
        use std::io::Write;
        let report = std::thread::spawn(|| super::profile_binding(false)).join()
            .unwrap_or_else(|_| Err(io::Error::other("Launcher initialization task failed")));
        let ready = matches!(&report, Ok(binding) if binding.configured);
        let value = match report {
            Ok(binding) => serde_json::to_value(binding).unwrap(),
            Err(error) => serde_json::json!({"error": error.to_string()}),
        };
        let _ = writeln!(io::stdout(), "{value}");
        std::process::exit(if ready { 0 } else { 2 });
    }
    if std::env::args_os().skip(1).eq([OsString::from("--lazy-tester-inspect-setup")]) {
        use std::io::Write;
        // Read only existing binding/directory/registry metadata; never provision or launch.
        let report=profile_binding::inspect_existing();
        let _=writeln!(io::stdout(),"{}",serde_json::to_string(&report).unwrap());
        std::process::exit(if report.configured {0}else{2});
    }
    if session_controller::maybe_run() { return true; }
    legacy_startup_observation::maybe_run()
}

// Retain a named existence lease, not mutex ownership. Do not wait or
// release the mutex: the original handle keeps the gate alive for its caller.
// Preserve the OS last-error immediately after creation.
fn acquire_startup_gate(name: &str, already_exists: io::Error) -> io::Result<OwnedHandle> {
    crate::engine::native_clients_session::acquire_existence_gate(name, already_exists)
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
    Storage::FileSystem::{
        FileCaseSensitiveInfo, GetFinalPathNameByHandleW, FILE_CASE_SENSITIVE_INFO,
        FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_LIST_DIRECTORY,
        FILE_READ_ATTRIBUTES, FILE_SHARE_WRITE, VOLUME_NAME_DOS,
    },
    System::{
        Com::{CoCreateGuid, CoTaskMemFree},
        LibraryLoader::{GetModuleHandleW, GetProcAddress},
        Memory::{
            CreateFileMappingW, MapViewOfFile, UnmapViewOfFile, FILE_MAP_READ, FILE_MAP_WRITE,
            PAGE_READWRITE,
        },
        Registry::{
            RegCloseKey, RegCreateKeyExW, RegOpenCurrentUser, RegOpenKeyExW, HKEY,
            KEY_ALL_ACCESS, KEY_READ, REG_CREATED_NEW_KEY,
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

#[path = "independent_bootstrap/child_configuration.rs"]
mod child_configuration;
use self::child_configuration::Configuration;
#[cfg(test)]
use self::child_configuration::EducationConfig;
#[path = "independent_bootstrap/child_process_environment.rs"]
mod child_process_environment;
use self::child_process_environment::child_environment;
#[cfg(test)]
use self::child_process_environment::{environment_name, ordinal_names};


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
    // Reuse the existing raw metadata owner; keep the bootstrap-specific
    // case-sensitivity and ordinary-directory policy here.
    let attributes = crate::engine::windows_file_identity::attributes(file)?;
    let id = crate::engine::windows_file_identity::file_id(file)?;
    let mut case: FILE_CASE_SENSITIVE_INFO = unsafe { mem::zeroed() };
    if unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle() as _, FileCaseSensitiveInfo,
            &mut case as *mut _ as _, mem::size_of_val(&case) as u32,
        )
    } == 0 {
        return Err(io::Error::last_os_error());
    }
    if attributes & FILE_ATTRIBUTE_DIRECTORY == 0
        || attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
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

#[cfg(test)]
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

fn open_existing_registry_store(id: &FILE_ID_INFO, slot: u32) -> io::Result<RegistryKey> {
    // Match user_registry_path's explicit user hive, not a process-wide cached HKCU mapping.
    let root = current_user_registry(KEY_READ)?;
    let path = wide(OsStr::new(&format!("Software\\MivubiEducationPilot\\{}", registry_leaf(id,slot)?)))?;
    let mut key = ptr::null_mut();
    let status = unsafe { RegOpenKeyExW(root.0,path.as_ptr(),0,KEY_ALL_ACCESS,&mut key) };
    if status != 0 {
        return Err(io::Error::new(io::Error::from_raw_os_error(status as i32).kind(),format!("Minecraft - {slot}: saved registry store cannot be opened (Windows error {status})")));
    }
    Ok(RegistryKey(key))
}

fn current_user_registry(access: u32) -> io::Result<RegistryKey> {
    let mut key = ptr::null_mut();
    let status = unsafe { RegOpenCurrentUser(access, &mut key) };
    if status != 0 {
        return Err(io::Error::from_raw_os_error(status as i32));
    }
    Ok(RegistryKey(key))
}

fn user_registry_path() -> io::Result<[u16; 512]> {
    let key = current_user_registry(KEY_READ)?;
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
    let root = current_user_registry(KEY_ALL_ACCESS)?;
    let path = wide(OsStr::new(&format!(
        "Software\\MivubiEducationPilot\\{}",
        registry_leaf(&id, slot)?
    )))?;
    let mut key = ptr::null_mut();
    let mut disposition = 0;
    let status = unsafe {
        RegCreateKeyExW(
            root.0,
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

#[cfg(test)]
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

#[cfg(test)]
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
        pins.push(open_directory(path, FILE_READ_ATTRIBUTES).map_err(|error|io::Error::new(error.kind(),format!("Saved launcher root is unavailable: {error}")))?);
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
        ).map_err(|error|io::Error::new(error.kind(),format!("Minecraft - {slot}: profile folder is unavailable: {error}")))?;
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
        let registry = open_existing_registry_store(&directory_id(&private)?,slot)?;
        slots.push(SlotStorage {
            slot,
            logical: logical.try_clone()?,
            temporary: existing_private_temporary(&private).map_err(|error|io::Error::new(error.kind(),format!("Minecraft - {slot}: temporary folder is unavailable: {error}")))?,
            private,
            registry,
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
    let _gate = acquire_startup_gate(
        "Local\\MivubiEducationFiveProfilePreparation",
        io::Error::new(io::ErrorKind::AlreadyExists, "Five-profile preparation already active"),
    )?;
    // Existing pair replay is the trust anchor; no root path or profile override is accepted.
    let pair = registered_slots::<2>()?;
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

fn registered_slots<const N: usize>() -> io::Result<[SlotStorage; N]> {
    if N == 5 && cfg!(feature = "lazy-tester-package") {
        if let Some(stores) = product_profiles::open()? {
            return Vec::from(stores).try_into().map_err(|_| invalid("Invalid product profile count"));
        }
    }
    registered_replay_slots::<N>()
}

// The caller may already have established product catalog absence in the
// same observation. Do not reopen that catalog on the legacy fallback.
fn registered_replay_slots<const N: usize>() -> io::Result<[SlotStorage; N]> {
    if N != 2 && N != 5 {
        return Err(invalid("Unsupported replay slot count"));
    }
    // Require the reviewed replay before touching Known Folders or storage.
    let expected = (if N == 2 { REVIEWED_REPLAY } else { REVIEWED_FIVE_REPLAY })
        .ok_or_else(|| io::Error::new(
            io::ErrorKind::Unsupported,
            "No reviewed existing Education profile pair",
        ))?;
    let (a, parent) = replay_roots()?;
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
    reopen_slots_for::<N>(a, &parent, &bytes, expected)
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
        let child = create_fixture_observed(&path, image, Some(copied), false, Some(proof))?;
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
#[path = "independent_bootstrap/tests.rs"]
mod tests;
