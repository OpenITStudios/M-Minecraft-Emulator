//! Identity parsing and read-only Windows process-context evidence.
//! Context inspection proves neither isolation, an Education signature nor gameplay.
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProcessKey {
    pub pid: u32,
    pub created_file_time: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ContextComparison {
    pub same_windows_user: bool,
    pub same_profile_directory: bool,
    pub same_windows_session: bool,
}

// A comparison never grants launch/window authority or asserts Microsoft login isolation.
pub fn compare_contexts(
    left: &ProcessKey,
    right: &ProcessKey,
) -> std::io::Result<ContextComparison> {
    #[cfg(windows)]
    {
        windows::compare(left, right)
    }
    #[cfg(not(windows))]
    {
        let _ = (left, right);
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Windows process context is unavailable",
        ))
    }
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppContainerComparison {
    /// True only when both exact processes have actual AppContainer tokens.
    pub both_appcontainers: bool,
    /// A distinct AppContainer SID is NOT evidence of account isolation.
    pub distinct_container_identities: bool,
}

// Read-only and independent of any external sandbox driver. Only sanitized
// booleans leave this process; no SID, path, token or credential is serialized.
// This is evidence, never launch permission or proof of Minecraft sign-in.
pub fn compare_appcontainer_identities(
    left: &ProcessKey,
    right: &ProcessKey,
) -> std::io::Result<AppContainerComparison> {
    #[cfg(windows)]
    {
        windows::compare_appcontainers(left, right)
    }
    #[cfg(not(windows))]
    {
        let _ = (left, right);
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Windows AppContainer context is unavailable",
        ))
    }
}


// Actual desktop-native process observation for the independent engine's
// read-only port. This reuses the existing pinned Windows process handle
// check; it does NOT claim a signed Education image, visible game window,
// persistent storage, or isolated Microsoft credentials.
use virtual_clients_core::independent_engine::{
    inspect_existing, NativePortError, ProcessContainment, ProcessIdentity,
    ProcessObservation, ProcessObserverPort, SessionInspection, SessionSlot,
    StorageObservations, StorageObserverPort,
};

pub struct WindowsProcessObserver;
pub struct UnknownStorageObserver;

impl ProcessObserverPort for WindowsProcessObserver {
    fn observe_exact(
        &self,
        expected: ProcessIdentity,
    ) -> Result<ProcessObservation, NativePortError> {
        let key = ProcessKey {
            pid: expected.pid,
            created_file_time: expected.created_file_time.to_string(),
        };
        #[cfg(windows)]
        {
            let containment = windows::inspect_exact(&key).map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => NativePortError::StaleProcess,
                std::io::ErrorKind::InvalidInput | std::io::ErrorKind::InvalidData => {
                    NativePortError::InvalidIdentity
                }
                std::io::ErrorKind::PermissionDenied => NativePortError::AccessDenied,
                _ => NativePortError::OperationFailed,
            })?;
            Ok(ProcessObservation {
                identity: expected,
                containment,
                official_image_signature_observed: false,
                visible_window_observed: false,
            })
        }
        #[cfg(not(windows))]
        {
            let _ = key;
            Err(NativePortError::NotImplemented)
        }
    }
}

impl StorageObserverPort for UnknownStorageObserver {
    fn inspect_boundaries(
        &self,
        _slot: SessionSlot,
        _expected: ProcessIdentity,
    ) -> Result<StorageObservations, NativePortError> {
        // Never invent isolation from an AppContainer SID or private-looking path.
        // A real OS storage observer requires its own exact scope and evidence.
        Ok(StorageObservations::default())
    }
}

/// Source-level, read-only integration with the existing identity owner.
/// Not a Tauri command, not a new lifecycle controller or game launch route.
pub fn inspect_independent_source(
    slot: SessionSlot,
    expected: ProcessIdentity,
) -> Result<SessionInspection, NativePortError> {
    inspect_existing(&WindowsProcessObserver, &UnknownStorageObserver, slot, expected)
}


// Synthetic-only bounded native FILE-handle reader. The caller MUST have
// created/owned the fixture child and independently established which fresh
// AppContainer profile contains this ALREADY OPENED file handle. This routine
// cannot certify that source provenance: it returns UnboundFixturePaths only.
// Never accept a path from the UI, read account caches, or use this for a game.
#[cfg(windows)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FixtureRecordInspectionError {
    Process(NativePortError),
    NotSyntheticContainer,
    InvalidFileTypeOrLength,
    FileIo(std::io::ErrorKind),
    Record(virtual_clients_core::independent_engine::SyntheticFrameError),
}

#[cfg(windows)]
pub(crate) fn inspect_unbound_synthetic_fixture_record(
    expected: ProcessIdentity,
    opened_record: &mut std::fs::File,
) -> Result<
    virtual_clients_core::independent_engine::UnboundFixturePaths,
    FixtureRecordInspectionError,
> {
    use std::io::{Read, Seek};
    use virtual_clients_core::independent_engine::{
        decode_synthetic_fixture_paths, ProcessContainment, SYNTHETIC_FRAME_BYTES,
    };

    let before = WindowsProcessObserver
        .observe_exact(expected)
        .map_err(FixtureRecordInspectionError::Process)?;
    if before.identity != expected {
        return Err(FixtureRecordInspectionError::Process(NativePortError::StaleProcess));
    }
    if before.containment != ProcessContainment::AppContainerTokenObserved {
        return Err(FixtureRecordInspectionError::NotSyntheticContainer);
    }

    // A fixed 8-byte regular file handle is the only accepted input. No path
    // traversal, opening, profile discovery, file writes, or unbounded reads.
    // The caller remains responsible for exclusive/open-handle provenance;
    // length checks cannot defeat a malicious concurrent writer.
    let metadata = opened_record
        .metadata()
        .map_err(|error| FixtureRecordInspectionError::FileIo(error.kind()))?;
    if !metadata.is_file() || metadata.len() != SYNTHETIC_FRAME_BYTES as u64 {
        return Err(FixtureRecordInspectionError::InvalidFileTypeOrLength);
    }
    opened_record
        .rewind()
        .map_err(|error| FixtureRecordInspectionError::FileIo(error.kind()))?;
    let mut bytes = [0u8; SYNTHETIC_FRAME_BYTES];
    opened_record
        .read_exact(&mut bytes)
        .map_err(|error| FixtureRecordInspectionError::FileIo(error.kind()))?;
    let after_metadata = opened_record
        .metadata()
        .map_err(|error| FixtureRecordInspectionError::FileIo(error.kind()))?;
    if !after_metadata.is_file() || after_metadata.len() != SYNTHETIC_FRAME_BYTES as u64 {
        return Err(FixtureRecordInspectionError::InvalidFileTypeOrLength);
    }
    let paths = decode_synthetic_fixture_paths(&bytes)
        .map_err(FixtureRecordInspectionError::Record)?;

    let after = WindowsProcessObserver
        .observe_exact(expected)
        .map_err(FixtureRecordInspectionError::Process)?;
    if after.identity != expected {
        return Err(FixtureRecordInspectionError::Process(NativePortError::StaleProcess));
    }
    if after.containment != ProcessContainment::AppContainerTokenObserved {
        return Err(FixtureRecordInspectionError::NotSyntheticContainer);
    }
    Ok(paths)
}

#[cfg(windows)]
mod windows {
    use super::{AppContainerComparison, ContextComparison, ProcessContainment, ProcessKey};
    use std::{io, mem::size_of, ptr};
    use windows_sys::Win32::{
        Foundation::{
            CloseHandle, GetLastError, ERROR_INSUFFICIENT_BUFFER, FILETIME, HANDLE, WAIT_TIMEOUT,
        },
        Globalization::{CompareStringOrdinal, CSTR_EQUAL},
        Security::{
            GetLengthSid, GetTokenInformation, IsValidSid, TokenAppContainerSid,
            TokenIsAppContainer, TokenSessionId, TokenUser, TOKEN_APPCONTAINER_INFORMATION,
            TOKEN_QUERY, TOKEN_USER,
        },
        System::Threading::{
            GetProcessTimes, OpenProcess, OpenProcessToken, WaitForSingleObject,
            PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE,
        },
        UI::Shell::GetUserProfileDirectoryW,
    };

    struct Handle(HANDLE);
    impl Drop for Handle {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
    fn invalid(message: &'static str) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, message)
    }
    fn live(process: &Handle) -> io::Result<()> {
        match unsafe { WaitForSingleObject(process.0, 0) } {
            WAIT_TIMEOUT => Ok(()),
            0 => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Observed process has exited",
            )),
            _ => Err(io::Error::last_os_error()),
        }
    }
    fn created(process: &Handle) -> io::Result<u64> {
        let mut times = [FILETIME {
            dwLowDateTime: 0,
            dwHighDateTime: 0,
        }; 4];
        let [a, b, c, d] = &mut times;
        if unsafe { GetProcessTimes(process.0, a, b, c, d) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok((u64::from(a.dwHighDateTime) << 32) | u64::from(a.dwLowDateTime))
    }
    fn pin(key: &ProcessKey) -> io::Result<Handle> {
        let expected = key
            .created_file_time
            .parse::<u64>()
            .map_err(|_| invalid("Invalid process creation time"))?;
        if key.pid == 0 || expected == 0 {
            return Err(invalid("Invalid process identity"));
        }
        let raw = unsafe {
            OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
                0,
                key.pid,
            )
        };
        if raw.is_null() {
            return Err(io::Error::last_os_error());
        }
        let process = Handle(raw);
        if created(&process)? != expected {
            return Err(invalid("Process creation time changed"));
        }
        live(&process)?;
        Ok(process)
    }
    struct Context {
        user: Vec<u8>,
        profile: Vec<u16>,
        session: u32,
    }
    fn context(process: &Handle) -> io::Result<Context> {
        let mut raw = ptr::null_mut();
        if unsafe { OpenProcessToken(process.0, TOKEN_QUERY, &mut raw) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let token = Handle(raw);
        let mut needed = 0;
        let first =
            unsafe { GetTokenInformation(token.0, TokenUser, ptr::null_mut(), 0, &mut needed) };
        if first != 0
            || unsafe { GetLastError() } != ERROR_INSUFFICIENT_BUFFER
            || needed < size_of::<TOKEN_USER>() as u32
            || needed > 65536
        {
            return Err(invalid("Windows token user size is unavailable"));
        }
        // Token structures contain pointers and require native alignment.
        let mut buffer = vec![0usize; (needed as usize).div_ceil(size_of::<usize>())];
        if unsafe {
            GetTokenInformation(
                token.0,
                TokenUser,
                buffer.as_mut_ptr().cast(),
                needed,
                &mut needed,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        let sid = unsafe { (*(buffer.as_ptr().cast::<TOKEN_USER>())).User.Sid };
        if unsafe { IsValidSid(sid) } == 0 {
            return Err(invalid("Invalid Windows token user"));
        }
        let length = unsafe { GetLengthSid(sid) } as usize;
        if !(8..=68).contains(&length) {
            return Err(invalid("Invalid Windows token user length"));
        }
        let user = unsafe { std::slice::from_raw_parts(sid.cast::<u8>(), length) }.to_vec();
        let mut session = 0u32;
        if unsafe {
            GetTokenInformation(
                token.0,
                TokenSessionId,
                (&mut session as *mut u32).cast(),
                size_of::<u32>() as u32,
                &mut needed,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        let mut count = 0;
        let first = unsafe { GetUserProfileDirectoryW(token.0, ptr::null_mut(), &mut count) };
        if first != 0
            || unsafe { GetLastError() } != ERROR_INSUFFICIENT_BUFFER
            || !(2..=32768).contains(&count)
        {
            return Err(invalid("Windows profile directory is unavailable"));
        }
        let mut profile = vec![0u16; count as usize];
        if unsafe { GetUserProfileDirectoryW(token.0, profile.as_mut_ptr(), &mut count) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let end = profile
            .iter()
            .position(|&c| c == 0)
            .ok_or_else(|| invalid("Invalid Windows profile directory"))?;
        if end == 0 {
            return Err(invalid("Empty Windows profile directory"));
        }
        profile.truncate(end);
        Ok(Context {
            user,
            profile,
            session,
        })
    }
    // An independent, bounded token inspection. Reuses the existing pinned
    // process handles; never reads folders, broker stores or account data.
    fn appcontainer_sid(process: &Handle) -> io::Result<Option<Vec<u8>>> {
        let mut raw = ptr::null_mut();
        if unsafe { OpenProcessToken(process.0, TOKEN_QUERY, &mut raw) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let token = Handle(raw);
        let mut flag = 0u32;
        let mut needed = 0u32;
        if unsafe {
            GetTokenInformation(
                token.0,
                TokenIsAppContainer,
                (&mut flag as *mut u32).cast(),
                size_of::<u32>() as u32,
                &mut needed,
            )
        } == 0 {
            return Err(io::Error::last_os_error());
        }
        if flag == 0 {
            return Ok(None);
        }
        let first = unsafe {
            GetTokenInformation(
                token.0,
                TokenAppContainerSid,
                ptr::null_mut(),
                0,
                &mut needed,
            )
        };
        if first != 0
            || unsafe { GetLastError() } != ERROR_INSUFFICIENT_BUFFER
            || needed < size_of::<TOKEN_APPCONTAINER_INFORMATION>() as u32
            || needed > 65536
        {
            return Err(invalid("AppContainer token identity size is unavailable"));
        }
        // Use word-aligned storage because the returned structure has a pointer.
        let mut buffer = vec![0usize; (needed as usize).div_ceil(size_of::<usize>())];
        if unsafe {
            GetTokenInformation(
                token.0,
                TokenAppContainerSid,
                buffer.as_mut_ptr().cast(),
                needed,
                &mut needed,
            )
        } == 0 {
            return Err(io::Error::last_os_error());
        }
        let sid = unsafe {
            (*(buffer.as_ptr().cast::<TOKEN_APPCONTAINER_INFORMATION>())).TokenAppContainer
        };
        if sid.is_null() || unsafe { IsValidSid(sid) } == 0 {
            return Err(invalid("AppContainer token SID is invalid"));
        }
        let length = unsafe { GetLengthSid(sid) } as usize;
        if !(8..=68).contains(&length) {
            return Err(invalid("AppContainer token SID length is invalid"));
        }
        Ok(Some(unsafe {
            std::slice::from_raw_parts(sid.cast::<u8>(), length)
        }.to_vec()))
    }

    /// Reuse the same pinned handle/FILETIME check as the existing desktop
    /// observer. The handle is retained until the final liveness check.
    pub(super) fn inspect_exact(expected: &ProcessKey) -> io::Result<ProcessContainment> {
        let process = pin(expected)?;
        let containment = if appcontainer_sid(&process)?.is_some() {
            ProcessContainment::AppContainerTokenObserved
        } else {
            ProcessContainment::NoAppContainerToken
        };
        live(&process)?;
        Ok(containment)
    }

    pub(super) fn compare_appcontainers(
        left: &ProcessKey,
        right: &ProcessKey,
    ) -> io::Result<AppContainerComparison> {
        let left_process = pin(left)?;
        let right_process = pin(right)?;
        let a = appcontainer_sid(&left_process)?;
        let b = appcontainer_sid(&right_process)?;
        live(&left_process)?;
        live(&right_process)?;
        let both = a.is_some() && b.is_some();
        Ok(AppContainerComparison {
            both_appcontainers: both,
            distinct_container_identities: both && a != b,
        })
    }

    pub(super) fn compare(left: &ProcessKey, right: &ProcessKey) -> io::Result<ContextComparison> {
        let left_process = pin(left)?;
        let right_process = pin(right)?;
        let a = context(&left_process)?;
        let b = context(&right_process)?;
        let profile_equal = unsafe {
            CompareStringOrdinal(
                a.profile.as_ptr(),
                a.profile.len() as i32,
                b.profile.as_ptr(),
                b.profile.len() as i32,
                1,
            )
        };
        if profile_equal == 0 {
            return Err(io::Error::last_os_error());
        }
        live(&left_process)?;
        live(&right_process)?;
        Ok(ContextComparison {
            same_windows_user: a.user == b.user,
            same_profile_directory: profile_equal == CSTR_EQUAL,
            same_windows_session: a.session == b.session,
        })
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        fn own_key() -> ProcessKey {
            let process = Handle(unsafe {
                OpenProcess(
                    PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
                    0,
                    std::process::id(),
                )
            });
            assert!(!process.0.is_null());
            ProcessKey {
                pid: std::process::id(),
                created_file_time: created(&process).unwrap().to_string(),
            }
        }
        #[test]
        fn actual_windows_context_is_private_and_rejects_stale_identity() {
            let key = own_key();
            let comparison = compare(&key, &key).unwrap();
            assert_eq!(
                comparison,
                ContextComparison {
                    same_windows_user: true,
                    same_profile_directory: true,
                    same_windows_session: true
                }
            );
            let json = serde_json::to_value(comparison).unwrap();
            assert_eq!(json.as_object().unwrap().len(), 3);
            assert!(json
                .as_object()
                .unwrap()
                .values()
                .all(|value| value.is_boolean()));
            assert!(compare(
                &ProcessKey {
                    pid: key.pid,
                    created_file_time: "1".into()
                },
                &key
            )
            .is_err());
            assert!(compare(
                &ProcessKey {
                    pid: 0,
                    created_file_time: key.created_file_time.clone()
                },
                &key
            )
            .is_err());
        }

        #[test]
        fn appcontainer_evidence_is_bounded_and_stale_processes_are_rejected() {
            let key = own_key();
            let comparison = compare_appcontainers(&key, &key).unwrap();
            assert!(!comparison.distinct_container_identities);
            let json = serde_json::to_value(comparison).unwrap();
            assert_eq!(json.as_object().unwrap().len(), 2);
            assert!(json.as_object().unwrap().values().all(|v| v.is_boolean()));
            assert!(compare_appcontainers(
                &key,
                &ProcessKey {
                    pid: key.pid,
                    created_file_time: "1".into(),
                },
            ).is_err());
        }

        #[test]
        fn independent_source_observation_keeps_game_and_storage_unknown() {
            use virtual_clients_core::independent_engine::{
                NativePortError, ProcessIdentity, SessionSlot, StorageSurface,
            };
            let key = own_key();
            let expected = ProcessIdentity::checked(
                key.pid,
                key.created_file_time.parse::<u64>().unwrap(),
            ).unwrap();
            let result = super::super::inspect_independent_source(
                SessionSlot::Mc02,
                expected,
            ).unwrap();
            assert_eq!(result.process.identity, expected);
            assert!(matches!(
                result.process.containment,
                ProcessContainment::NoAppContainerToken
                    | ProcessContainment::AppContainerTokenObserved
            ));
            assert!(!result.process.official_image_signature_observed);
            assert!(!result.process.visible_window_observed);
            assert!(result.storage.unresolved().contains(&StorageSurface::RoamingKnownFolder));
            assert!(result.storage.unresolved().contains(&StorageSurface::AuthenticationBroker));
            assert!(result.storage.unresolved().contains(&StorageSurface::NamedObjects));

            let stale = ProcessIdentity::checked(expected.pid, expected.created_file_time + 1)
                .unwrap();
            assert_eq!(
                super::super::inspect_independent_source(SessionSlot::Mc02, stale)
                    .unwrap_err(),
                NativePortError::InvalidIdentity
            );
        }

        #[test]
        fn unbound_fixture_reader_checks_process_before_open_file_bytes() {
            use virtual_clients_core::independent_engine::ProcessIdentity;
            let key = own_key();
            let process = Handle(unsafe {
                OpenProcess(
                    PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
                    0,
                    key.pid,
                )
            });
            assert!(!process.0.is_null());
            // A normal desktop test process is not a synthetic AppContainer;
            // do not attempt to interpret arbitrary files as fixture evidence.
            if appcontainer_sid(&process).unwrap().is_none() {
                let mut executable = std::fs::File::open(std::env::current_exe().unwrap()).unwrap();
                let expected = ProcessIdentity::checked(
                    key.pid, key.created_file_time.parse::<u64>().unwrap()
                ).unwrap();
                assert_eq!(
                    super::super::inspect_unbound_synthetic_fixture_record(
                        expected, &mut executable
                    ).unwrap_err(),
                    super::super::FixtureRecordInspectionError::NotSyntheticContainer
                );

                let stale = ProcessIdentity::checked(
                    expected.pid, expected.created_file_time + 1
                ).unwrap();
                assert_eq!(
                    super::super::inspect_unbound_synthetic_fixture_record(
                        stale, &mut executable
                    ).unwrap_err(),
                    super::super::FixtureRecordInspectionError::Process(
                        super::super::NativePortError::InvalidIdentity
                    )
                );
            }
        }

        #[test]
        fn actual_child_identity_is_pinned_without_controlling_its_lifetime() {
            use std::{
                os::windows::process::CommandExt,
                process::{Child, Command, Stdio},
            };
            struct Fixture(Child);
            impl Drop for Fixture {
                fn drop(&mut self) {
                    let _ = self.0.kill();
                    let _ = self.0.wait();
                }
            }
            let mut fixture = Fixture(
                Command::new(crate::engine::native_clients::system_powershell().unwrap())
                    .args([
                        "-NoProfile",
                        "-NonInteractive",
                        "-Command",
                        "Start-Sleep -Seconds 60",
                    ])
                    .creation_flags(0x08000000)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .unwrap(),
            );
            let process = Handle(unsafe {
                OpenProcess(
                    PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
                    0,
                    fixture.0.id(),
                )
            });
            assert!(!process.0.is_null());
            let child_key = ProcessKey {
                pid: fixture.0.id(),
                created_file_time: created(&process).unwrap().to_string(),
            };
            let parent_key = own_key();
            assert_eq!(
                compare(&parent_key, &child_key).unwrap(),
                ContextComparison {
                    same_windows_user: true,
                    same_profile_directory: true,
                    same_windows_session: true,
                }
            );
            drop(pin(&child_key).unwrap());
            assert!(
                fixture.0.try_wait().unwrap().is_none(),
                "Releasing observation must not terminate a process"
            );
            fixture.0.kill().unwrap();
            fixture.0.wait().unwrap();
            // Keep the original handle alive: a dead identity must not be replaced by PID reuse.
            assert_eq!(
                compare(&parent_key, &child_key).unwrap_err().kind(),
                io::ErrorKind::NotFound
            );
            assert_eq!(live(&process).unwrap_err().kind(), io::ErrorKind::NotFound);
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowIdentity {
    pub pid: u32,
    pub created_file_time: u64,
    pub hwnd: u64,
}

pub fn parse_window_identity(
    pid: u32,
    created_file_time: Option<&str>,
    window_handle: Option<&str>,
) -> Option<WindowIdentity> {
    if pid == 0 {
        return None;
    }
    let created = created_file_time?.parse::<u64>().ok()?;
    let hwnd = u64::from_str_radix(window_handle?.strip_prefix("0x")?, 16).ok()?;
    if created == 0 || hwnd == 0 {
        return None;
    }
    Some(WindowIdentity {
        pid,
        created_file_time: created,
        hwnd,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent_storage_adapter_cannot_invent_verified_broker_or_roaming() {
        use virtual_clients_core::independent_engine::{
            StorageFinding, StorageSurface,
        };
        let expected = ProcessIdentity::checked(42, 133400000000000000).unwrap();
        let report = UnknownStorageObserver
            .inspect_boundaries(SessionSlot::Mc02, expected)
            .unwrap();
        for surface in StorageSurface::ALL {
            assert_eq!(report.finding(surface), StorageFinding::Unknown);
        }
        assert_eq!(report.unresolved().len(), StorageSurface::ALL.len());
    }

    #[cfg(not(windows))]
    #[test]
    fn non_windows_source_observation_fails_closed() {
        let process = ProcessIdentity::checked(1, 12345).unwrap();
        assert!(matches!(
            inspect_independent_source(SessionSlot::Mc02, process),
            Err(NativePortError::NotImplemented)
        ));
    }

    #[test]
    fn accepts_full_width_windows_filetime_without_lossy_float_conversion() {
        assert_eq!(
            parse_window_identity(42, Some("133400000000000000"), Some("0x100")),
            Some(WindowIdentity {
                pid: 42,
                created_file_time: 133400000000000000,
                hwnd: 256
            })
        );
        assert_eq!(
            parse_window_identity(42, Some("18446744073709551615"), Some("0xffffffffffffffff"))
                .map(|key| key.created_file_time),
            Some(u64::MAX)
        );
    }
    #[test]
    fn refuses_missing_zero_reused_or_malformed_identity_pieces() {
        for (pid, created, handle) in [
            (0, Some("10"), Some("0x2")),
            (1, None, Some("0x2")),
            (1, Some("0"), Some("0x2")),
            (1, Some("-10"), Some("0x2")),
            (1, Some("10.5"), Some("0x2")),
            (1, Some("18446744073709551616"), Some("0x2")),
            (1, Some("10"), None),
            (1, Some("10"), Some("0x0")),
            (1, Some("10"), Some("2")),
            (1, Some("10"), Some("0x")),
            (1, Some("10"), Some("0xzz")),
        ] {
            assert_eq!(parse_window_identity(pid, created, handle), None);
        }
    }
}
