//! Fixed own-descendant handoff control. Never compiled into product launch paths.
use super::*;
use std::sync::atomic::{AtomicI32, Ordering};
use windows_sys::Win32::System::Threading::{CreateEventW, SetEvent};

#[repr(C)]
pub(super) struct Handoff {
    magic: u32,
    version: u32,
    bytes: u32,
    reserved: u32,
    configuration: HANDLE,
    proof: HANDLE,
    ready: HANDLE,
    reply: HANDLE,
    pub(super) process: HANDLE,
    pub(super) thread: HANDLE,
    pub(super) pid: u32,
    error: u32,
    pub(super) created: FILETIME,
    decision: AtomicI32,
    reserved_end: u32,
}
const _: () = assert!(mem::size_of::<Handoff>() == 88);
const _: () = assert!(mem::offset_of!(Handoff, process) == 48);
const _: () = assert!(mem::offset_of!(Handoff, created) == 72);
const _: () = assert!(mem::offset_of!(Handoff, decision) == 80);

pub(super) struct Request {
    view: *mut Handoff,
    _section: OwnedHandle,
    ready: OwnedHandle,
    reply: OwnedHandle,
    config: Option<Configuration>,
    proof: OwnedHandle,
    parent: (u32, u64),
}
impl Request {
    pub(super) fn new(
        parent: &PendingEducationBootstrap,
        proof: &tests::FixtureReceipt,
        helper_proof: &tests::FixtureReceipt,
    ) -> io::Result<Self> {
        if parent.resume_attempted {
            return Err(invalid("Install fixture handoff before parent resume"));
        }
        let identity = parent.identity()?;
        let retained_proof = helper_proof.0.try_clone()?;
        let config = Configuration::create()?;
        let event = || -> io::Result<OwnedHandle> {
            let raw = unsafe { CreateEventW(ptr::null(), 1, 0, ptr::null()) };
            if raw.is_null() {
                return Err(io::Error::last_os_error());
            }
            Ok(unsafe { OwnedHandle::from_raw_handle(raw as _) })
        };
        let ready = event()?;
        let reply = event()?;
        let raw = unsafe {
            CreateFileMappingW(
                INVALID_HANDLE_VALUE,
                ptr::null(),
                PAGE_READWRITE,
                0,
                mem::size_of::<Handoff>() as u32,
                ptr::null(),
            )
        };
        if raw.is_null() {
            return Err(io::Error::last_os_error());
        }
        let section = unsafe { OwnedHandle::from_raw_handle(raw as _) };
        let mapped = unsafe { MapViewOfFile(raw, FILE_MAP_WRITE, 0, 0, mem::size_of::<Handoff>()) };
        if mapped.Value.is_null() {
            return Err(io::Error::last_os_error());
        }
        let mut request = Self {
            view: mapped.Value as _,
            _section: section,
            ready,
            reply,
            config: Some(config),
            proof: retained_proof,
            parent: identity,
        };
        unsafe {
            ptr::write(request.view, mem::zeroed());
        }
        let duplicate = |raw, inherit| -> io::Result<HANDLE> {
            let mut remote = ptr::null_mut();
            if unsafe {
                DuplicateHandle(
                    windows_sys::Win32::System::Threading::GetCurrentProcess(),
                    raw,
                    parent.process.as_raw_handle() as HANDLE,
                    &mut remote,
                    0,
                    inherit,
                    DUPLICATE_SAME_ACCESS,
                )
            } == 0
            {
                return Err(io::Error::last_os_error());
            }
            Ok(remote) // Original parent owns these bounded capabilities until normal process exit.
        };
        let configuration = duplicate(
            request.config.as_ref().unwrap().readonly.as_raw_handle() as HANDLE,
            1,
        )?;
        let helper_proof = duplicate(helper_proof.0.as_raw_handle() as HANDLE, 1)?;
        let ready = duplicate(request.ready.as_raw_handle() as HANDLE, 0)?;
        let reply = duplicate(request.reply.as_raw_handle() as HANDLE, 0)?;
        let wire = request.packet();
        wire.magic = 0x484c5031;
        wire.version = 1;
        wire.bytes = mem::size_of::<Handoff>() as u32;
        wire.configuration = configuration;
        wire.proof = helper_proof;
        wire.ready = ready;
        wire.reply = reply;
        let remote = duplicate(request._section.as_raw_handle() as HANDLE, 0)?;
        let receipt = unsafe {
            MapViewOfFile(
                proof.0.as_raw_handle() as HANDLE,
                FILE_MAP_WRITE,
                0,
                0,
                2064,
            )
        };
        if receipt.Value.is_null() {
            return Err(io::Error::last_os_error());
        }
        // Existing own-fixture receipt.go is unused by its Rust receiver outside this control.
        unsafe {
            ptr::write((receipt.Value as *mut u8).add(2040) as *mut HANDLE, remote);
        }
        if unsafe { UnmapViewOfFile(receipt) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(request)
    }
    pub(super) fn packet(&mut self) -> &mut Handoff {
        unsafe { &mut *self.view }
    }
    pub(super) fn wait(&self, parent: &PendingEducationBootstrap) -> io::Result<()> {
        let handles = [
            self.ready.as_raw_handle() as HANDLE,
            parent.process.as_raw_handle() as HANDLE,
        ];
        if unsafe {
            windows_sys::Win32::System::Threading::WaitForMultipleObjects(
                2,
                handles.as_ptr(),
                0,
                10000,
            )
        } != WAIT_OBJECT_0
        {
            return Err(invalid("Own-descendant handoff did not arrive"));
        }
        parent.identity()?;
        Ok(())
    }
    fn accept(&self) -> io::Result<()> {
        if unsafe { &*self.view }
            .decision
            .compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Err(invalid("Handoff already decided"));
        }
        if unsafe { SetEvent(self.reply.as_raw_handle() as HANDLE) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}
impl Drop for Request {
    fn drop(&mut self) {
        unsafe { &*self.view }
            .decision
            .compare_exchange(0, 2, Ordering::SeqCst, Ordering::SeqCst)
            .ok();
        unsafe {
            SetEvent(self.reply.as_raw_handle() as HANDLE);
            UnmapViewOfFile(
                windows_sys::Win32::System::Memory::MEMORY_MAPPED_VIEW_ADDRESS {
                    Value: self.view as _,
                },
            );
        }
        // Sender still owns original child handles and cancels on refusal/timeout. No receiver handle is published.
    }
}

fn original_parent(process: HANDLE) -> io::Result<u32> {
    #[repr(C)]
    struct Basic {
        exit: i32,
        peb: *mut c_void,
        affinity: usize,
        priority: i32,
        pid: usize,
        parent: usize,
    }
    const _: () = assert!(mem::size_of::<Basic>() == 48);
    const _: () = assert!(mem::offset_of!(Basic, parent) == 40);
    type Query = unsafe extern "system" fn(HANDLE, u32, *mut c_void, u32, *mut u32) -> i32;
    let module = unsafe { GetModuleHandleW(wide(OsStr::new("ntdll.dll"))?.as_ptr()) };
    let entry = unsafe { GetProcAddress(module, c"NtQueryInformationProcess".as_ptr().cast()) }
        .ok_or_else(|| invalid("Process ancestry query unavailable"))?;
    let query: Query = unsafe { mem::transmute(entry) };
    let mut basic: Basic = unsafe { mem::zeroed() };
    let mut bytes = 0;
    if unsafe {
        query(
            process,
            0,
            &mut basic as *mut _ as _,
            mem::size_of::<Basic>() as u32,
            &mut bytes,
        )
    } < 0
        || bytes as usize != mem::size_of::<Basic>()
        || basic.parent > u32::MAX as usize
        || basic.pid != unsafe { GetProcessId(process) } as usize
    {
        return Err(invalid("Original descendant ancestry unavailable"));
    }
    Ok(basic.parent as u32)
}

impl BootstrapLibrary {
    pub(super) fn accept_owned_descendant<'a>(
        &'a self,
        parent: &'a PendingEducationBootstrap,
        request: &mut Request,
        proof: &tests::FixtureReceipt,
    ) -> io::Result<OwnedHelperTransaction<'a>> {
        request.wait(parent)?;
        if unsafe {
            windows_sys::Win32::Foundation::CompareObjectHandles(
                request.proof.as_raw_handle() as HANDLE,
                proof.0.as_raw_handle() as HANDLE,
            )
        } == 0
        {
            return Err(invalid("Handoff receipt capability changed"));
        }
        if !parent.resume_attempted || parent.identity()? != request.parent {
            return Err(invalid("Handoff parent changed"));
        }
        let store = checked_slot_binding(parent)?;
        let wire = request.packet();
        if wire.magic != 0x484c5031
            || wire.version != 1
            || wire.bytes != mem::size_of::<Handoff>() as u32
            || wire.reserved != 0
            || wire.reserved_end != 0
            || wire.error != 0
            || wire.decision.load(Ordering::SeqCst) != 0
        {
            return Err(invalid("Invalid own-descendant handoff"));
        }
        let pid = wire.pid;
        let birth =
            (u64::from(wire.created.dwHighDateTime) << 32) | u64::from(wire.created.dwLowDateTime);
        let duplicate = |raw| -> io::Result<OwnedHandle> {
            let mut local = ptr::null_mut();
            if unsafe {
                DuplicateHandle(
                    parent.process.as_raw_handle() as HANDLE,
                    raw,
                    windows_sys::Win32::System::Threading::GetCurrentProcess(),
                    &mut local,
                    0,
                    0,
                    DUPLICATE_SAME_ACCESS,
                )
            } == 0
            {
                return Err(io::Error::last_os_error());
            }
            Ok(unsafe { OwnedHandle::from_raw_handle(local as _) })
        };
        let process = duplicate(wire.process)?;
        let thread = duplicate(wire.thread)?;
        let raw = process.as_raw_handle() as HANDLE;
        let mut actual: FILETIME = unsafe { mem::zeroed() };
        let mut exited = actual;
        let mut kernel = actual;
        let mut user = actual;
        if pid == parent.pid
            || birth == 0
            || unsafe { GetProcessId(raw) } != pid
            || unsafe { GetProcessIdOfThread(thread.as_raw_handle() as HANDLE) } != pid
            || unsafe { WaitForSingleObject(raw, 0) } != WAIT_TIMEOUT
            || unsafe { GetProcessTimes(raw, &mut actual, &mut exited, &mut kernel, &mut user) }
                == 0
            || actual.dwLowDateTime != birth as u32
            || actual.dwHighDateTime != (birth >> 32) as u32
            || original_parent(raw)? != parent.pid
            || interactive_session(pid)? != interactive_session(parent.pid)?
        {
            return Err(invalid("Original descendant identity/ancestry mismatch"));
        }
        let root = directory_path(
            self._directories
                .last()
                .ok_or_else(|| invalid("Missing payload root"))?,
        )?;
        let image = pin_image(&root.join("bootstrap-ordering.exe"))?;
        let mut path = [0u16; 32768];
        let mut count = path.len() as u32;
        if unsafe { QueryFullProcessImageNameW(raw, 0, path.as_mut_ptr(), &mut count) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let actual_path = PathBuf::from(OsString::from_wide(&path[..count as usize]));
        if !same_image(&image_id(&image)?, &image_id(&pin_image(&actual_path)?)?)
            || !same_image(&image_id(&image)?, &image_id(&parent._image)?)
        {
            return Err(invalid("Original descendant image mismatch"));
        }
        // Only after all provenance checks may existing Pending own cancellation of this fixture.
        let mut child = PendingEducationBootstrap {
            process,
            _thread: thread,
            _image: image,
            pid,
            created: birth,
            storage: None,
            configuration: None,
            observation: None,
            policy: None,
            resume_attempted: false,
        };
        let mut storage = store.duplicate()?;
        storage.singleton_caller_rva = 0;
        let config = request
            .config
            .take()
            .ok_or_else(|| invalid("Handoff already consumed"))?;
        child.configuration = Some(config.publish(&child, &storage)?);
        child.storage = Some(storage);
        let status = tests::prepare_fixture(self, &child, proof, true);
        if status != 0 {
            return Err(io::Error::from_raw_os_error(status as i32));
        }
        self.enroll(&child)?;
        let transaction = OwnedHelperTransaction {
            child,
            parent,
            _payload: self,
        };
        transaction.verify()?;
        request.accept()?;
        Ok(transaction)
    }
}
