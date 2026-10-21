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



/// Non-authorizing, sanitized comparison of two distinct Windows processes.
/// Not a broker, registry, singleton, signed-image or gameplay probe. A
/// distinct AppContainer SID by itself does NOT prove account separation.
#[derive(Debug, PartialEq, Eq)]
pub struct NativeProcessPairInspection {
    pub first_slot: SessionSlot,
    pub second_slot: SessionSlot,
    pub first: ProcessIdentity,
    pub second: ProcessIdentity,
    pub context: ContextComparison,
    pub appcontainer: AppContainerComparison,
}

#[cfg(windows)]
fn map_native_process_error(error: std::io::Error) -> NativePortError {
    match error.kind() {
        std::io::ErrorKind::NotFound => NativePortError::StaleProcess,
        std::io::ErrorKind::InvalidInput | std::io::ErrorKind::InvalidData => {
            NativePortError::InvalidIdentity
        }
        std::io::ErrorKind::PermissionDenied => NativePortError::AccessDenied,
        _ => NativePortError::OperationFailed,
    }
}

/// Source-only pairwise Windows inspection. Validates different logical slots
/// AND different PIDs before any platform calls. Existing process/token API
/// owners perform the actual pinned-handle checks; final observations reject
/// process exit or changes to the recorded AppContainer token category.
///
/// These separate queries are not an atomic snapshot and do not prove
/// per-process HKCU, singleton names, Microsoft WAM/MSAL or multiplayer.
/// The result is never registered as a Tauri command or admission action.
pub fn inspect_independent_process_pair_source(
    first_slot: SessionSlot,
    first: ProcessIdentity,
    second_slot: SessionSlot,
    second: ProcessIdentity,
) -> Result<NativeProcessPairInspection, NativePortError> {
    if first_slot == second_slot
        || first.pid == 0 || second.pid == 0
        || first.created_file_time == 0 || second.created_file_time == 0
        || first.pid == second.pid
    {
        return Err(NativePortError::InvalidIdentity);
    }
    let observer = WindowsProcessObserver;
    let first_before = observer.observe_exact(first)?;
    let second_before = observer.observe_exact(second)?;
    if first_before.identity != first || second_before.identity != second {
        return Err(NativePortError::StaleProcess);
    }
    let keys = |process: ProcessIdentity| ProcessKey {
        pid: process.pid,
        created_file_time: process.created_file_time.to_string(),
    };
    let first_key = keys(first);
    let second_key = keys(second);
    let context = compare_contexts(&first_key, &second_key)
        .map_err(|err| {
            #[cfg(windows)]
            { map_native_process_error(err) }
            #[cfg(not(windows))]
            { let _ = err; NativePortError::NotImplemented }
        })?;
    let appcontainer = compare_appcontainer_identities(&first_key, &second_key)
        .map_err(|err| {
            #[cfg(windows)]
            { map_native_process_error(err) }
            #[cfg(not(windows))]
            { let _ = err; NativePortError::NotImplemented }
        })?;
    if observer.observe_exact(first)? != first_before
        || observer.observe_exact(second)? != second_before
    {
        return Err(NativePortError::StaleProcess);
    }
    Ok(NativeProcessPairInspection {
        first_slot, second_slot, first, second, context, appcontainer,
    })
}


/// Default diagnostic: Windows identity does not reveal the unmodified
/// game's singleton or Microsoft broker account switching behavior.
pub struct UnknownPairBoundaryObserver;

impl virtual_clients_core::independent_engine::PairBoundaryObserverPort
    for UnknownPairBoundaryObserver
{
    fn observe_pair_boundaries(
        &self, first_slot: SessionSlot, first: ProcessIdentity,
        second_slot: SessionSlot, second: ProcessIdentity,
    ) -> Result<virtual_clients_core::independent_engine::PairBoundarySnapshot, NativePortError> {
        use virtual_clients_core::independent_engine::{
            PairBoundaryFact, PairBoundaryFinding, PairBoundarySnapshot,
            StorageEvidenceSource,
        };
        let unknown = PairBoundaryFact {
            source: StorageEvidenceSource::WindowsProcess,
            finding: PairBoundaryFinding::Unknown,
        };
        Ok(PairBoundarySnapshot {
            first_slot, second_slot, first, second,
            singleton: unknown, broker: unknown,
        })
    }
}

/// Source-only, not mounted as a Tauri command. Two exact process observers
/// are rechecked; singleton and broker findings cannot be invented from SIDs.
pub fn inspect_independent_pair_boundaries_source(
    first_slot: SessionSlot, first: ProcessIdentity,
    second_slot: SessionSlot, second: ProcessIdentity,
) -> Result<
    virtual_clients_core::independent_engine::PendingPairInspection,
    virtual_clients_core::independent_engine::PairBoundaryError,
> {
    virtual_clients_core::independent_engine::inspect_pair_boundaries(
        &WindowsProcessObserver, &UnknownPairBoundaryObserver,
        first_slot, first, second_slot, second,
    )
}


/// WinVerifyTrust's verdict for the file opened using a pinned process's
/// image-name query. This is NOT yet identification of its publisher or a
/// guarantee that the queried on-disk bytes equal the running PE section.
/// Interpretation of a WinTrust result after all provider handles have been
/// closed. Never publish signer bytes from a rejected verification, and never
/// silently accept a successful verification without a clean state lifetime.
/// This function does not identify the Minecraft publisher.
fn finish_file_signature_review(
    verified: bool,
    had_state: bool,
    close_succeeded: bool,
    signer: Result<Option<[u8; 32]>, ()>,
) -> Result<(bool, Option<[u8; 32]>), &'static str> {
    if had_state && !close_succeeded {
        return Err("WinTrust provider state could not be closed");
    }
    if !verified {
        return Ok((false, None));
    }
    if !had_state {
        return Err("WinTrust did not provide successful verification state");
    }
    match signer {
        Ok(Some(digest)) if digest != [0; 32] => Ok((true, Some(digest))),
        _ => Err("Verified WinTrust signer certificate is unavailable"),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageFileSignatureTrust {
    AuthenticodeTrustedOnDisk,
    NotVerified,
}

/// Deliberately no positive value until an actual certificate signer chain
/// and allowlisted official Minecraft Education publisher are established.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OfficialGamePublisher {
    NotAttested,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PinnedImageTrustCandidate {
    pub process: ProcessIdentity,
    pub image_file_signature: ImageFileSignatureTrust,
    // Audit-only DER leaf hash; NOT proof of Minecraft publisher identity.
    pub signer_leaf_sha256: Option<[u8; 32]>,
    pub publisher: OfficialGamePublisher,
}

fn unpromoted_image_trust(
    process: ProcessIdentity,
    signed_on_disk: bool,
    signer: Option<[u8; 32]>,
) -> PinnedImageTrustCandidate {
    PinnedImageTrustCandidate {
        process,
        image_file_signature: if signed_on_disk {
            ImageFileSignatureTrust::AuthenticodeTrustedOnDisk
        } else {
            ImageFileSignatureTrust::NotVerified
        },
        signer_leaf_sha256: if signed_on_disk { signer } else { None },
        publisher: OfficialGamePublisher::NotAttested,
    }
}

/// Source-only inspection. It NEVER returns a verified Minecraft publisher or
/// changes ProcessObservation::official_image_signature_observed. In particular
/// a trusted generic Windows signature cannot satisfy acceptance.rs' publisher
/// check and cannot activate Launch/Provision.
///
/// The exact process handle is pinned by the canonical desktop Windows owner.
/// Only its queried executable path is opened for non-mutating trust review;
/// no caller-supplied paths, user profile scanning, account data or WAM access.
pub fn inspect_process_image_trust_source(
    expected: ProcessIdentity,
) -> Result<PinnedImageTrustCandidate, NativePortError> {
    if expected.pid == 0 || expected.created_file_time == 0 {
        return Err(NativePortError::InvalidIdentity);
    }
    #[cfg(windows)]
    {
        let key = ProcessKey {
            pid: expected.pid,
            created_file_time: expected.created_file_time.to_string(),
        };
        let (signed, signer) = windows::inspect_file_authenticode(&key)
            .map_err(map_native_process_error)?;
        Ok(unpromoted_image_trust(expected, signed, signer))
    }
    #[cfg(not(windows))]
    {
        let _ = expected;
        Err(NativePortError::NotImplemented)
    }
}


/// Feed the actual Windows-source image trust observation to the canonical
/// non-authorizing provenance reviewer. No signer leaf fingerprint, mapped PE
/// section/file identity or package provenance is fabricated. The reviewed
/// official release signer policy is EMPTY pending source/rights verification.
/// A generic trusted Authenticode file must not claim a verified publisher.
pub fn inspect_process_image_provenance_source(
    expected: ProcessIdentity,
) -> Result<
    virtual_clients_core::independent_engine::PendingImageProvenance,
    NativePortError,
> {
    use virtual_clients_core::independent_engine::{
        review_image_provenance, ImageProvenanceError, PackageIdentityEvidence,
        PinnedImageProvenanceClaim, PublisherCertificateClaim,
        ReviewedPublisherPolicy, RunningImageBinding, StorageEvidenceSource,
    };
    let candidate = inspect_process_image_trust_source(expected)?;
    let source = PinnedImageProvenanceClaim {
        process: candidate.process,
        observed_process: Some(candidate.process),
        source: StorageEvidenceSource::WindowsProcess,
        authenticode_trusted_on_disk: candidate.image_file_signature
            == ImageFileSignatureTrust::AuthenticodeTrustedOnDisk,
        signer_certificate: candidate.signer_leaf_sha256.map(|digest| PublisherCertificateClaim {
            source: StorageEvidenceSource::WindowsProcess,
            signer_leaf_sha256: digest,
        }),
        image_binding: RunningImageBinding::QueriedExecutablePathOnly,
        package: PackageIdentityEvidence::Unknown,
    };
    review_image_provenance(source, ReviewedPublisherPolicy::default())
        .map_err(|problem| match problem {
            ImageProvenanceError::UnpinnedOrStaleProcess => NativePortError::StaleProcess,
            ImageProvenanceError::WrongEvidenceSource
            | ImageProvenanceError::InvalidCertificateDigest => NativePortError::OperationFailed,
        })
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

    // Require a stable fixed-length disk-file identity, not a named pipe,
    // directory or an opened reparse point. The caller MUST have opened this
    // file with FILE_FLAG_OPEN_REPARSE_POINT and exclusive sharing, and must
    // independently prove that its owned fixture child/profile produced it.
    // Metadata cannot establish source authenticity or prevent a compromised
    // writer changing eight bytes without a visible file metadata change.
    let before_file = windows::fixture_record_identity(opened_record)
        .map_err(|error| match error.kind() {
            std::io::ErrorKind::InvalidData => FixtureRecordInspectionError::InvalidFileTypeOrLength,
            _ => FixtureRecordInspectionError::FileIo(error.kind()),
        })?;
    opened_record
        .rewind()
        .map_err(|error| FixtureRecordInspectionError::FileIo(error.kind()))?;
    let mut bytes = [0u8; SYNTHETIC_FRAME_BYTES];
    opened_record
        .read_exact(&mut bytes)
        .map_err(|error| FixtureRecordInspectionError::FileIo(error.kind()))?;
    let after_file = windows::fixture_record_identity(opened_record)
        .map_err(|error| match error.kind() {
            std::io::ErrorKind::InvalidData => FixtureRecordInspectionError::InvalidFileTypeOrLength,
            _ => FixtureRecordInspectionError::FileIo(error.kind()),
        })?;
    if before_file != after_file {
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


/// Legacy Windows UAC virtualization flags for an exact native process token.
/// These flags concern a compatibility mechanism, NOT private HKCU hives,
/// per-AppContainer registry mapping, profile ownership or Minecraft sign-in.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyTokenVirtualizationObservation {
    pub identity: ProcessIdentity,
    pub allowed: bool,
    pub enabled: bool,
}

/// Source-only, unmounted Windows token metadata inspection. No registry
/// hives/values/paths or credentials are opened. This MUST NOT be converted
/// to StorageFinding::PrivateAccessObserved.
pub fn inspect_legacy_token_virtualization_source(
    expected: ProcessIdentity,
) -> Result<LegacyTokenVirtualizationObservation, NativePortError> {
    #[cfg(windows)]
    {
        let key = ProcessKey {
            pid: expected.pid,
            created_file_time: expected.created_file_time.to_string(),
        };
        let (allowed, enabled) = windows::legacy_token_virtualization(&key)
            .map_err(|error| match error.kind() {
                std::io::ErrorKind::NotFound => NativePortError::StaleProcess,
                std::io::ErrorKind::InvalidInput | std::io::ErrorKind::InvalidData => {
                    NativePortError::InvalidIdentity
                }
                std::io::ErrorKind::PermissionDenied => NativePortError::AccessDenied,
                _ => NativePortError::OperationFailed,
            })?;
        Ok(LegacyTokenVirtualizationObservation {
            identity: expected,
            allowed,
            enabled,
        })
    }
    #[cfg(not(windows))]
    {
        let _ = expected;
        Err(NativePortError::NotImplemented)
    }
}

/// Diagnostic integration only: the real Windows process observer and the
/// all-Unknown storage observer are consulted fresh, before/after the probe.
/// The return type does not express an approved route or mutation permission.
pub fn inspect_independent_storage_route_source(
    request: virtual_clients_core::independent_engine::StorageRouteRequest,
) -> Result<virtual_clients_core::independent_engine::RouteBoundaryInspection, NativePortError> {
    virtual_clients_core::independent_engine::inspect_storage_route(
        &WindowsProcessObserver,
        &UnknownStorageObserver,
        request,
    )
}

#[cfg(windows)]
mod windows {
    use super::{AppContainerComparison, ContextComparison, ProcessContainment, ProcessKey, finish_file_signature_review};
    use std::{io, mem::size_of, os::windows::io::AsRawHandle, ptr};
    use sha2::{Digest, Sha256};
    use std::os::windows::{ffi::OsStringExt, fs::OpenOptionsExt};
    use windows_sys::Win32::{
        Foundation::{
            CloseHandle, GetLastError, ERROR_INSUFFICIENT_BUFFER, FILETIME, HANDLE, WAIT_TIMEOUT,
        },
        Globalization::{CompareStringOrdinal, CSTR_EQUAL},
        Security::WinTrust::{
            WinVerifyTrust, WINTRUST_ACTION_GENERIC_VERIFY_V2,
            CRYPT_PROVIDER_CERT, CRYPT_PROVIDER_DATA, CRYPT_PROVIDER_SGNR,
            WINTRUST_DATA, WINTRUST_DATA_0, WINTRUST_FILE_INFO,
            WTD_CACHE_ONLY_URL_RETRIEVAL, WTD_CHOICE_FILE,
            WTD_REVOKE_WHOLECHAIN, WTD_STATEACTION_CLOSE,
            WTD_STATEACTION_VERIFY, WTD_UI_NONE,
        },
        Security::{
            GetLengthSid, GetTokenInformation, IsValidSid, TokenAppContainerSid,
            TokenIsAppContainer, TokenSessionId, TokenUser, TOKEN_APPCONTAINER_INFORMATION,
            TOKEN_QUERY, TOKEN_USER, TokenVirtualizationAllowed,
            TokenVirtualizationEnabled,
        },
        System::LibraryLoader::{
            FreeLibrary, GetProcAddress, LoadLibraryExW, LOAD_LIBRARY_SEARCH_SYSTEM32,
        },
        System::Threading::{
            GetProcessTimes, OpenProcess, OpenProcessToken, QueryFullProcessImageNameW,
            WaitForSingleObject, PROCESS_QUERY_LIMITED_INFORMATION,
            PROCESS_SYNCHRONIZE,
        },
        Storage::FileSystem::{
            GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
            FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT,
            FILE_SHARE_READ,
        },
        UI::Shell::GetUserProfileDirectoryW,
    };

    /// Exact identity and attributes of a fixture file HANDLE, not its path.
    /// This is an *intra-reader stability check*, not proof that a child wrote
    /// the file. The original fixture owner must enforce exclusive sharing
    /// and non-following reparse-point open options before passing a handle.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub(super) struct FixtureRecordIdentity {
        volume: u32,
        file_index: u64,
        length: u64,
        modified: u64,
        attributes: u32,
    }

    pub(super) fn fixture_record_identity(
        opened: &std::fs::File,
    ) -> io::Result<FixtureRecordIdentity> {
        use virtual_clients_core::independent_engine::SYNTHETIC_FRAME_BYTES;

        // Windows BY_HANDLE_FILE_INFORMATION has no uninitialized pointers;
        // zero initialization is the documented starting state for this FFI.
        let mut value: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
        if unsafe { GetFileInformationByHandle(opened.as_raw_handle(), &mut value) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let length = (u64::from(value.nFileSizeHigh) << 32)
            | u64::from(value.nFileSizeLow);
        if length != SYNTHETIC_FRAME_BYTES as u64
            || value.dwFileAttributes & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT) != 0
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Not a regular 8-byte fixture record",
            ));
        }
        Ok(FixtureRecordIdentity {
            volume: value.dwVolumeSerialNumber,
            file_index: (u64::from(value.nFileIndexHigh) << 32)
                | u64::from(value.nFileIndexLow),
            length,
            modified: (u64::from(value.ftLastWriteTime.dwHighDateTime) << 32)
                | u64::from(value.ftLastWriteTime.dwLowDateTime),
            attributes: value.dwFileAttributes,
        })
    }

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
    // Only legacy UAC compatibility metadata, never a registry hive query.
    // Pin the exact PID/FILTIME process handle so no reused PID is accepted.
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
            fn drop(&mut self) { unsafe { FreeLibrary(self.0); } }
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

    pub(super) fn inspect_file_authenticode(
        key: &ProcessKey,
    ) -> io::Result<(bool, Option<[u8; 32]>)> {
        use std::os::windows::io::AsRawHandle;

        fn image_name(process: &Handle) -> io::Result<Vec<u16>> {
            let mut buf = vec![0u16; 32768];
            let mut len = buf.len() as u32;
            if unsafe {
                QueryFullProcessImageNameW(process.0, 0, buf.as_mut_ptr(), &mut len)
            } == 0 {
                return Err(io::Error::last_os_error());
            }
            if len == 0 || len as usize >= buf.len() {
                return Err(invalid("Windows image path is invalid or oversized"));
            }
            buf.truncate(len as usize);
            if buf.iter().any(|&character| character == 0) {
                return Err(invalid("Invalid embedded null in image path"));
            }
            Ok(buf)
        }

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
        let path = image_name(&process)?;
        let pathname = std::path::PathBuf::from(std::ffi::OsString::from_wide(&path));
        let file = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .open(pathname)?;
        let before = file_identity(&file)?;
        let mut wide = path.clone();
        wide.push(0);
        let mut file_info: WINTRUST_FILE_INFO = unsafe { std::mem::zeroed() };
        file_info.cbStruct = size_of::<WINTRUST_FILE_INFO>() as u32;
        file_info.pcwszFilePath = wide.as_ptr();
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
        if before != after || image_name(&process)? != path {
            return Err(invalid("On-disk process image changed during inspection"));
        }
        live(&process)?;
        // WinVerifyTrust returns LONG; ONLY zero is a successful verification.
        Ok(reviewed)
    }

    pub(super) fn legacy_token_virtualization(
        key: &ProcessKey,
    ) -> io::Result<(bool, bool)> {
        let process = pin(key)?;
        let mut raw = ptr::null_mut();
        if unsafe { OpenProcessToken(process.0, TOKEN_QUERY, &mut raw) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let token = Handle(raw);
        let query = |kind| -> io::Result<bool> {
            let mut value = 0u32;
            let mut length = 0u32;
            if unsafe {
                GetTokenInformation(
                    token.0,
                    kind,
                    (&mut value as *mut u32).cast(),
                    size_of::<u32>() as u32,
                    &mut length,
                )
            } == 0 {
                return Err(io::Error::last_os_error());
            }
            if length != size_of::<u32>() as u32 || value > 1 {
                return Err(invalid("Invalid legacy virtualization token flag"));
            }
            Ok(value != 0)
        };
        let allowed = query(TokenVirtualizationAllowed)?;
        let enabled = query(TokenVirtualizationEnabled)?;
        live(&process)?;
        if enabled && !allowed {
            return Err(invalid("Inconsistent legacy virtualization token flags"));
        }
        Ok((allowed, enabled))
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
        fn fixture_record_identity_refuses_non_eight_byte_files() {
            // This is a normal test executable, never a synthetic observation.
            let executable = std::fs::File::open(std::env::current_exe().unwrap()).unwrap();
            let error = fixture_record_identity(&executable).unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        }

        #[test]
        fn stale_legacy_registry_metadata_is_rejected_and_never_reads_hkcu() {
            use virtual_clients_core::independent_engine::ProcessIdentity;
            let current = own_key();
            let stale = ProcessIdentity::checked(
                current.pid,
                current.created_file_time.parse::<u64>().unwrap() + 1,
            ).unwrap();
            assert_eq!(
                super::super::inspect_legacy_token_virtualization_source(stale).unwrap_err(),
                super::super::NativePortError::InvalidIdentity,
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
    fn pair_comparison_rejects_same_slot_or_pid_before_native_calls() {
        let first = ProcessIdentity::checked(71, 133400000000000071).unwrap();
        let second = ProcessIdentity::checked(72, 133400000000000072).unwrap();
        assert_eq!(
            inspect_independent_process_pair_source(
                SessionSlot::Mc02, first, SessionSlot::Mc02, second
            ).unwrap_err(),
            NativePortError::InvalidIdentity
        );
        assert_eq!(
            inspect_independent_process_pair_source(
                SessionSlot::Mc02, first, SessionSlot::Mc03, first
            ).unwrap_err(),
            NativePortError::InvalidIdentity
        );
        let recycled = ProcessIdentity::checked(71, first.created_file_time + 1).unwrap();
        assert_eq!(
            inspect_independent_process_pair_source(
                SessionSlot::Mc02, first, SessionSlot::Mc03, recycled
            ).unwrap_err(),
            NativePortError::InvalidIdentity
        );
        let invalid = ProcessIdentity { pid: 0, created_file_time: 1 };
        assert_eq!(
            inspect_independent_process_pair_source(
                SessionSlot::Mc01, invalid, SessionSlot::Mc02, second
            ).unwrap_err(),
            NativePortError::InvalidIdentity
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn pair_comparison_is_unavailable_without_windows_process_handles() {
        let first = ProcessIdentity::checked(71, 133400000000000071).unwrap();
        let second = ProcessIdentity::checked(72, 133400000000000072).unwrap();
        assert_eq!(
            inspect_independent_process_pair_source(
                SessionSlot::Mc01, first, SessionSlot::Mc02, second
            ).unwrap_err(),
            NativePortError::NotImplemented
        );
    }



    #[test]
    fn provenance_policy_remains_pending_even_with_mock_trusted_file_signature() {
        use virtual_clients_core::independent_engine::{
            review_image_provenance, ImageProvenanceGap,
            PackageIdentityEvidence, PinnedImageProvenanceClaim,
            ReviewedPublisherPolicy, RunningImageBinding, StorageEvidenceSource,
        };
        let id = ProcessIdentity::checked(51, 133400000000000051).unwrap();
        let candidate = unpromoted_image_trust(id, true, None);
        let claim = PinnedImageProvenanceClaim {
            process: id,
            observed_process: Some(id),
            source: StorageEvidenceSource::WindowsProcess,
            authenticode_trusted_on_disk: candidate.image_file_signature
                == ImageFileSignatureTrust::AuthenticodeTrustedOnDisk,
            signer_certificate: None,
            image_binding: RunningImageBinding::QueriedExecutablePathOnly,
            package: PackageIdentityEvidence::Unknown,
        };
        let reviewed = review_image_provenance(claim, ReviewedPublisherPolicy::default()).unwrap();
        assert!(reviewed.gaps.contains(&ImageProvenanceGap::LeafCertificateNotObserved));
        assert!(reviewed.gaps.contains(&ImageProvenanceGap::OfficialSignerPolicyNotReviewed));
        assert!(reviewed.gaps.contains(&ImageProvenanceGap::RunningImageNotBoundToVerifiedFile));
    }

    #[test]
    fn invalid_image_provenance_identity_refuses_any_native_inspection() {
        let invalid = ProcessIdentity { pid: 11, created_file_time: 0 };
        assert_eq!(inspect_process_image_provenance_source(invalid),
            Err(NativePortError::InvalidIdentity));
    }

    #[cfg(not(windows))]
    #[test]
    fn image_provenance_is_unavailable_without_windows_native_observation() {
        let key = ProcessIdentity::checked(52, 133400000000000052).unwrap();
        assert_eq!(inspect_process_image_provenance_source(key),
            Err(NativePortError::NotImplemented));
    }

    #[test]
    fn trusted_generic_signature_never_identifies_official_publisher() {
        let id = ProcessIdentity::checked(501, 133400000000000501).unwrap();
        for trusted in [false, true] {
            let observation = unpromoted_image_trust(id, trusted, None);
            assert_eq!(observation.process, id);
            assert_eq!(observation.publisher, OfficialGamePublisher::NotAttested);
            assert_eq!(observation.signer_leaf_sha256, None);
            assert_eq!(observation.image_file_signature, if trusted {
                ImageFileSignatureTrust::AuthenticodeTrustedOnDisk
            } else {
                ImageFileSignatureTrust::NotVerified
            });
        }
    }


    #[test]
    fn wintrust_signer_review_fails_closed_on_missing_or_unreleased_state() {
        let hash = [0x51; 32];
        let positive = finish_file_signature_review(true, true, true, Ok(Some(hash)));
        assert_eq!(positive, Ok((true, Some(hash))));
        assert_eq!(
            finish_file_signature_review(true, false, true, Ok(Some(hash))),
            Err("WinTrust did not provide successful verification state"),
        );
        for verified in [true, false] {
            assert_eq!(
                finish_file_signature_review(verified, true, false, Ok(Some(hash))),
                Err("WinTrust provider state could not be closed"),
            );
        }
        for invalid in [Ok(None), Ok(Some([0; 32])), Err(())] {
            assert_eq!(
                finish_file_signature_review(true, true, true, invalid),
                Err("Verified WinTrust signer certificate is unavailable"),
            );
        }
        // Rejected signatures cannot expose an alleged certificate, even if
        // a collector supplied a nonzero hash.
        assert_eq!(
            finish_file_signature_review(false, true, true, Ok(Some(hash))),
            Ok((false, None)),
        );
        assert_eq!(
            finish_file_signature_review(false, false, true, Err(())),
            Ok((false, None)),
        );
    }
    #[test]
    fn leaf_digest_remains_audit_only_with_publisher_and_mapping_unattested() {
        use virtual_clients_core::independent_engine::{
            review_image_provenance, ImageProvenanceGap, PackageIdentityEvidence,
            PinnedImageProvenanceClaim, PublisherCertificateClaim, ReviewedPublisherPolicy,
            RunningImageBinding, StorageEvidenceSource,
        };
        let id = ProcessIdentity::checked(703, 133400000000000703).unwrap();
        let digest = [0x42; 32];
        let candidate = unpromoted_image_trust(id, true, Some(digest));
        assert_eq!(candidate.signer_leaf_sha256, Some(digest));
        assert_eq!(candidate.publisher, OfficialGamePublisher::NotAttested);
        let report = review_image_provenance(PinnedImageProvenanceClaim {
            process: id, observed_process: Some(id),
            source: StorageEvidenceSource::WindowsProcess,
            authenticode_trusted_on_disk: true,
            signer_certificate: Some(PublisherCertificateClaim {
                source: StorageEvidenceSource::WindowsProcess,
                signer_leaf_sha256: digest,
            }),
            image_binding: RunningImageBinding::QueriedExecutablePathOnly,
            package: PackageIdentityEvidence::Unknown,
        }, ReviewedPublisherPolicy::default()).unwrap();
        assert!(!report.gaps.contains(&ImageProvenanceGap::LeafCertificateNotObserved));
        assert!(report.gaps.contains(&ImageProvenanceGap::OfficialSignerPolicyNotReviewed));
        assert!(report.gaps.contains(&ImageProvenanceGap::RunningImageNotBoundToVerifiedFile));
        assert!(report.gaps.contains(&ImageProvenanceGap::PackageOriginNotReviewed));
        assert_eq!(unpromoted_image_trust(id, false, Some(digest)).signer_leaf_sha256, None);
    }

    #[test]
    fn zero_process_identity_refuses_image_trust_inspection() {
        for key in [
            ProcessIdentity { pid: 0, created_file_time: 1 },
            ProcessIdentity { pid: 3, created_file_time: 0 },
        ] {
            assert_eq!(inspect_process_image_trust_source(key),
                Err(NativePortError::InvalidIdentity));
        }
    }

    #[cfg(not(windows))]
    #[test]
    fn image_trust_inspection_fails_closed_on_non_windows() {
        let id = ProcessIdentity::checked(11, 133400000000000011).unwrap();
        assert_eq!(inspect_process_image_trust_source(id),
            Err(NativePortError::NotImplemented));
    }

    #[test]
    fn default_pair_boundary_collector_keeps_both_unknown() {
        use virtual_clients_core::independent_engine::{
            PairBoundaryFinding, PairBoundaryObserverPort,
        };
        let first = ProcessIdentity::checked(101, 133400000000000101).unwrap();
        let second = ProcessIdentity::checked(102, 133400000000000102).unwrap();
        let observation = UnknownPairBoundaryObserver
            .observe_pair_boundaries(SessionSlot::Mc02, first, SessionSlot::Mc03, second)
            .unwrap();
        assert_eq!(observation.singleton.finding, PairBoundaryFinding::Unknown);
        assert_eq!(observation.broker.finding, PairBoundaryFinding::Unknown);
    }

    #[test]
    fn invalid_pair_is_rejected_without_native_process_io() {
        use virtual_clients_core::independent_engine::PairBoundaryError;
        let identity = ProcessIdentity::checked(101, 133400000000000101).unwrap();
        assert_eq!(
            inspect_independent_pair_boundaries_source(
                SessionSlot::Mc02, identity, SessionSlot::Mc03, identity,
            ),
            Err(PairBoundaryError::InvalidPair),
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn valid_pair_fails_closed_without_native_windows() {
        use virtual_clients_core::independent_engine::PairBoundaryError;
        let first = ProcessIdentity::checked(101, 133400000000000101).unwrap();
        let second = ProcessIdentity::checked(102, 133400000000000102).unwrap();
        assert_eq!(
            inspect_independent_pair_boundaries_source(
                SessionSlot::Mc02, first, SessionSlot::Mc03, second,
            ),
            Err(PairBoundaryError::Native(NativePortError::NotImplemented)),
        );
    }

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
    fn non_windows_legacy_registry_query_is_unavailable() {
        let process = ProcessIdentity::checked(2, 1234567).unwrap();
        assert_eq!(
            inspect_legacy_token_virtualization_source(process).unwrap_err(),
            NativePortError::NotImplemented,
        );
    }

    #[test]
    fn static_route_denial_has_no_windows_side_effects() {
        use virtual_clients_core::independent_engine::{
            RouteBoundaryInspection, StorageRouteAction, StorageRouteBlocker,
            StorageRouteDomain, StorageRouteRequest, StorageRouteTarget,
        };
        let identity = ProcessIdentity::checked(2, 133400000000000000).unwrap();
        let request = StorageRouteRequest {
            slot: SessionSlot::Mc02,
            expected_process: identity,
            observed_process: None,
            domain: StorageRouteDomain::CurrentUserRegistry,
            target: StorageRouteTarget::HostAccountData,
            action: StorageRouteAction::Read,
        };
        assert_eq!(
            inspect_independent_storage_route_source(request).unwrap(),
            RouteBoundaryInspection::Denied(StorageRouteBlocker::HostAccountBoundary),
        );
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
