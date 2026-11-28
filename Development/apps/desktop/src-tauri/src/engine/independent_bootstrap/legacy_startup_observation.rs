//! Shared startup journal for the retained controller; historical multi-child
//! observation helpers are compiled only as disposable Rust fixtures.
use super::{directory_id, image_id, invalid, open_directory, same_image,
    PendingEducationBootstrap, SavedStartupObservation, StartedEducationObservation};
#[cfg(test)]
use super::{create_suspended_process, game_preset, registered_image, registered_slots,
    window_caption, BootstrapLibrary, PreparedEducationEnrollment, StartupAttempt};
use std::{fs::OpenOptions, io, os::windows::{fs::OpenOptionsExt, io::{AsRawHandle, FromRawHandle, OwnedHandle}}, path::PathBuf, ptr};
#[cfg(test)]
use std::path::Path;
use windows_sys::Win32::{
    Foundation::{HANDLE, HANDLE_FLAG_INHERIT, INVALID_HANDLE_VALUE, SetHandleInformation, WAIT_OBJECT_0, WAIT_TIMEOUT},
    Storage::FileSystem::{FILE_FLAG_OPEN_REPARSE_POINT, FILE_READ_ATTRIBUTES, FILE_SHARE_READ},
    System::{Memory::{CreateFileMappingW, MapViewOfFile, UnmapViewOfFile, FILE_MAP_READ, PAGE_READWRITE}, Threading::WaitForSingleObject},
};

#[cfg(test)]
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
        if pid == state.pid && super::super::verify_live_client_02_window(hwnd, pid).is_ok() {
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
        let _ = window_caption::label_slot_window(selection.window, child.pid, store.slot);
    }
}

#[cfg(test)]
pub(super) fn observe_until_exit<const N: usize>(started: &StartedEducationObservation<N>) -> io::Result<i32> {
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

// Retained only for the unreviewed-payload refusal fixture; no product entrypoint.
#[cfg(test)]
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

#[cfg(test)]
pub(super) fn prepare_registered_enrolled_for<const N: usize>() -> io::Result<PreparedEducationEnrollment<N>> {
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

pub(super) const OBSERVATION_ENV: &str = "MIVUBI_EDUCATION_OBSERVATION_SECTION";
// Must match private startup-observation.h; no public wire schema.
pub(super) const OBSERVATION_BYTES: usize = 31968;
pub(super) struct StartupObservation {
    pub(super) section: OwnedHandle,
}
impl StartupObservation {
    pub(super) fn create() -> io::Result<Self> {
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
    pub(super) fn observation_path(&self) -> io::Result<PathBuf> {
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

    pub(super) fn observation_bytes_after_exit(&self) -> io::Result<Vec<u8>> {
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
