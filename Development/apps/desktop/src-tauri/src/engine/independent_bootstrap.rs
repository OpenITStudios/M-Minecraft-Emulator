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

use self::legacy_startup_observation::{StartupObservation, OBSERVATION_ENV};
#[cfg(test)]
use self::legacy_startup_observation::{prepare_registered_enrolled_for, OBSERVATION_BYTES};

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


#[path = "independent_bootstrap/profile_storage_handles.rs"]
mod profile_storage_handles;
use self::profile_storage_handles::{
    RegistryKey, SlotStorage, directory_id, open_directory, directory_path,
    fresh_root, registry_leaf, open_existing_registry_store, user_registry_path,
    open_private_temporary, PrivateTemporaryMode, create_private_slot, known_folder,
};
#[cfg(test)]
use self::profile_storage_handles::fresh_slots;


#[path = "independent_bootstrap/reviewed_profile_replay.rs"]
mod reviewed_profile_replay;
use self::reviewed_profile_replay::{
    registered_replay_slots, replay_roots, reopen_slots_for, REVIEWED_FIVE_REPLAY,
};
#[cfg(test)]
use self::reviewed_profile_replay::{five_replay_bytes, reopen_slots};

fn registered_slots<const N: usize>() -> io::Result<[SlotStorage; N]> {
    if N == 5 && cfg!(feature = "lazy-tester-package") {
        if let Some(stores) = product_profiles::open()? {
            return Vec::from(stores).try_into().map_err(|_| invalid("Invalid product profile count"));
        }
    }
    registered_replay_slots::<N>()
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

#[cfg(test)]
#[path = "independent_bootstrap/tests.rs"]
mod tests;
