//! Reviewed native DLL admission, retained payload pins, and original-child enrollment.
//! No launch authority or alternate payload route; the original controller owns the transaction.
use super::{directory_path, file_digest, image_id, invalid, open_directory, pin_image, same_image, wide, Configuration, PendingEducationBootstrap};
use crate::engine::manager_settings::PerformanceSettings;
use std::{ffi::OsString, fs::File, io, mem, os::windows::{ffi::OsStringExt, io::AsRawHandle}, path::{Path, PathBuf}, ptr};
use windows_sys::Win32::{Foundation::HANDLE, Storage::FileSystem::FILE_READ_ATTRIBUTES, System::LibraryLoader::GetProcAddress};

// Fixed development bytes authorized for the explicit startup mode only, not packaging/admission.
// Tests must never turn --include-ignored into game execution.
// Presentation ABI v4 requires a newly reviewed matching DLL pair. The
// previous hashes are historical at 937e7f3; do not admit them for this ABI.
const REVIEWED_EDUCATION_BOOTSTRAP: Option<PayloadDigests> = None;
// A feature-enabled AI controller must NOT adopt the ordinary reviewed
// bootstrap-compat.dll: that production image intentionally omits optional
// Win32 virtual input hooks. A standalone experimental C++ DLL changes its
// hash and must be separately reviewed/pinned for this WHOLE controller.
// No hash override or game DLL fallback. OFF until separately approved.
#[cfg(feature="mc02-native-input")]
const REVIEWED_EDUCATION_AI_BOOTSTRAP: Option<PayloadDigests> = None;

#[derive(Clone, Copy)]
pub(super) struct PayloadDigests {
    pub(super) enroller: [u8; 32],
    pub(super) compatibility: [u8; 32],
}
pub(super) type EnrollmentEntry =
    unsafe extern "system" fn(HANDLE, HANDLE, u32, u64, HANDLE, HANDLE, HANDLE) -> u32;

pub(super) fn payload_digest(file: &File) -> io::Result<[u8; 32]> {
    file_digest(file, 32 * 1024 * 1024)
}

pub(super) struct BootstrapLibrary {
    pub(super) module: windows_sys::Win32::Foundation::HMODULE,
    pub(super) entry: EnrollmentEntry,
    _files: [File; 2],
    pub(super) _directories: Vec<File>,
}
impl BootstrapLibrary {
    pub(super) fn reviewed() -> io::Result<Self> {
        // Select one reviewed compatibility DLL pair PER feature-enabled
        // original session controller, not per slot or per late MCP packet.
        // A hybrid human MC01 has no AI mapping, so experimental hooks stay
        // inert for that process while sharing one consistent DLL image.
        #[cfg(feature="mc02-native-input")]
        let (expected, missing) = (
            REVIEWED_EDUCATION_AI_BOOTSTRAP,
            "No separately reviewed experimental AI bootstrap payload",
        );
        #[cfg(not(feature="mc02-native-input"))]
        let (expected, missing) = (
            REVIEWED_EDUCATION_BOOTSTRAP,
            "No reviewed Education bootstrap payload for presentation ABI v4",
        );
        let digests = expected.ok_or_else(|| {
            io::Error::new(io::ErrorKind::Unsupported, missing)
        })?;
        let executable = std::env::current_exe()?;
        let root = executable
            .parent()
            .ok_or_else(|| invalid("Missing bootstrap payload parent"))?
            .join("independent-bootstrap");
        Self::load(&root, digests)
    }

    // Only the fixed reviewed production location or cfg(test) build directory can call this.
    pub(super) fn load(root: &Path, digests: PayloadDigests) -> io::Result<Self> {
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

    pub(super) fn prepare_presentation(
        &self,
        child: &PendingEducationBootstrap,
        section: &Configuration,
        limits: PerformanceSettings,
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
        // The transaction already validated the saved limits before creating
        // the child. Never reopen a mutable preference during enrollment.
        let symbol = unsafe {
            GetProcAddress(
                self.module,
                c"PrepareEducationPresentationV4".as_ptr().cast(),
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

    pub(super) fn enroll(&self, child: &PendingEducationBootstrap) -> io::Result<()> {
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
// Negative source fixture: no native payload, game launch or profile write
// can be authorized for an unreviewed AI-input feature build.
#[cfg(all(test, feature="mc02-native-input"))]
#[test]
fn experimental_native_input_cannot_adopt_the_standard_dll() {
    assert!(REVIEWED_EDUCATION_AI_BOOTSTRAP.is_none());
    let refusal = BootstrapLibrary::reviewed()
        .err()
        .expect("Unapproved experimental DLL must be refused");
    assert_eq!(refusal.kind(), io::ErrorKind::Unsupported);
}

impl Drop for BootstrapLibrary {
    fn drop(&mut self) {
        if unsafe { windows_sys::Win32::Foundation::FreeLibrary(self.module) } == 0 {
            eprintln!("Bootstrap module cleanup is unconfirmed");
        }
    }
}

