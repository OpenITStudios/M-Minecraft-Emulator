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
    // Read both sanitized comparisons using the SAME pair of pinned
    // process handles instead of opening two fresh handle pairs.
    let (context, appcontainer) = {
        #[cfg(windows)]
        {
            windows::compare_pair(&first_key, &second_key)
                .map_err(map_native_process_error)?
        }
        #[cfg(not(windows))]
        {
            return Err(NativePortError::NotImplemented);
        }
    };
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

/// Exact-process Windows package form, not Microsoft/Minecraft publisher
/// identity. Package presence does not prove MSIX registry virtualization,
/// WAM compatibility, a supported launcher or entitlement to any file.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObservedProcessPackageForm {
    Packaged,
    Unpackaged,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PendingProcessPackageObservation {
    pub process: ProcessIdentity,
    pub package_form: ObservedProcessPackageForm,
}

/// Interpret the documented first GetPackageFullName probe. A process with no
/// package reports APPMODEL_ERROR_NO_PACKAGE. A process with a package
/// reports ERROR_INSUFFICIENT_BUFFER and the required UTF-16 length.
/// Any other return value or excessive name is a failed observation, never
/// implicitly "unpackaged".
fn classify_package_probe(status: i32, required_chars: u32) -> Result<Option<usize>, NativePortError> {
    match status {
        15700 => Ok(None), // APPMODEL_ERROR_NO_PACKAGE
        122 if (2..=1024).contains(&required_chars) => Ok(Some(required_chars as usize)),
        _ => Err(NativePortError::OperationFailed),
    }
}

/// Non-authorizing, exact-process package-family comparison. MSIX app data
/// can be scoped per package AND Windows user; two process PIDs are NOT two
/// separate package storage identities. This is a diagnostic relation only,
/// not filesystem/HKCU/WAM proof, official publisher attestation or a route
/// to launch another Minecraft instance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObservedPackageFamilyRelation {
    BothUnpackaged,
    OnePackaged,
    SamePackageFamily,
    DifferentPackageFamilies,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PendingPackageFamilyPairObservation {
    pub left: ProcessIdentity,
    pub right: ProcessIdentity,
    pub relation: ObservedPackageFamilyRelation,
}

fn classify_package_family_pair(
    left: Option<&str>,
    right: Option<&str>,
) -> ObservedPackageFamilyRelation {
    use ObservedPackageFamilyRelation as Relation;
    match (left, right) {
        (None, None) => Relation::BothUnpackaged,
        (Some(_), None) | (None, Some(_)) => Relation::OnePackaged,
        (Some(first), Some(second)) if first.eq_ignore_ascii_case(second) => {
            Relation::SamePackageFamily
        }
        (Some(_), Some(_)) => Relation::DifferentPackageFamilies,
    }
}

/// The native observer retains TWO distinct read-only process handles,
/// checking each full PID/creation FILETIME before querying its Windows
/// Package Family Name and rechecking both process lifetimes afterwards.
/// Raw package names remain confined to the native stack and are discarded.
/// Query errors are errors, NEVER interpreted as "unpackaged".
pub fn inspect_process_package_family_pair_source(
    left: ProcessIdentity,
    right: ProcessIdentity,
) -> Result<PendingPackageFamilyPairObservation, NativePortError> {
    if left.pid == 0 || left.created_file_time == 0 ||
        right.pid == 0 || right.created_file_time == 0 ||
        left.pid == right.pid
    {
        return Err(NativePortError::InvalidIdentity);
    }
    #[cfg(windows)]
    {
        let first = ProcessKey {
            pid: left.pid, created_file_time: left.created_file_time.to_string(),
        };
        let second = ProcessKey {
            pid: right.pid, created_file_time: right.created_file_time.to_string(),
        };
        let relation = windows::compare_package_families(&first, &second)
            .map_err(map_native_process_error)?;
        Ok(PendingPackageFamilyPairObservation { left, right, relation })
    }
    #[cfg(not(windows))]
    {
        let _ = (left, right);
        Err(NativePortError::NotImplemented)
    }
}

/// Unmounted read-only package-form diagnostic. No package name is returned,
/// logged, stored or compared to an unreviewed Minecraft publisher allowlist.
pub fn inspect_process_package_form_source(
    expected: ProcessIdentity,
) -> Result<PendingProcessPackageObservation, NativePortError> {
    if expected.pid == 0 || expected.created_file_time == 0 {
        return Err(NativePortError::InvalidIdentity);
    }
    #[cfg(windows)]
    {
        let key = ProcessKey {
            pid: expected.pid,
            created_file_time: expected.created_file_time.to_string(),
        };
        let form = windows::inspect_package_form(&key).map_err(map_native_process_error)?;
        Ok(PendingProcessPackageObservation {
            process: expected, package_form: form,
        })
    }
    #[cfg(not(windows))]
    {
        let _ = expected;
        Err(NativePortError::NotImplemented)
    }
}

// Read-only Windows directory HANDLE verifier for a future **trusted owner**
// of a fresh private session profile. A returned volume/file ID proves only
// that one opened regular directory object is stable, NOT that it is the
// session's root, has private ACLs, or redirects Minecraft's file view.
//
// The eventual caller must own the open capability and also validate all
// ancestor directories, final reparse points, ACLs, token and process
// identity. This helper cannot be used as a launch/read/write admission.
#[cfg(windows)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PrivateRootObjectIdentity {
    volume: u32,
    file_index: u64,
}

/// Diagnostic SID comparisons; never an access or root ownership grant.
#[cfg(windows)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct PendingRootSidRelations {
    owner_matches_process_user: bool,
    owner_matches_container: bool,
    container_allow_ace_observed: bool,
    // None = manager cannot safely resolve the target user's profile
    // (or the OS lookup was unavailable). Some only compares FILE identities.
    // Neither state is a per-game Roaming/HKCU or access attestation.
    os_container_local_app_data_match: Option<bool>,
}

#[cfg(windows)]
pub(crate) fn inspect_private_root_handle_source(
    opened: &std::fs::File,
) -> std::io::Result<PrivateRootObjectIdentity> {
    windows::inspect_directory_handle(opened)
}

// This is intentionally *not* an ownership or access capability. A trusted
// original profile owner must later prove the root's ACL/ancestor provenance
// and AppContainer SID membership; a token category alone is insufficient.
#[cfg(windows)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PendingPrivateRootInspection {
    slot: SessionSlot,
    process: ProcessIdentity,
    directory: PrivateRootObjectIdentity,
    sid_relations: PendingRootSidRelations,
}

#[cfg(windows)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PrivateRootObservationError {
    InvalidSlotOrProcess,
    StaleProcess,
    NotAppContainer,
    Native(NativePortError),
    Directory(std::io::ErrorKind),
}

#[cfg(windows)]
fn validate_private_root_observation(
    slot: SessionSlot,
    expected: ProcessIdentity,
    before: ProcessObservation,
    after: ProcessObservation,
) -> Result<(), PrivateRootObservationError> {
    if slot.is_host_native() || expected.pid == 0 || expected.created_file_time == 0 {
        return Err(PrivateRootObservationError::InvalidSlotOrProcess);
    }
    if before.identity != expected || after != before {
        return Err(PrivateRootObservationError::StaleProcess);
    }
    if before.containment != ProcessContainment::AppContainerTokenObserved {
        return Err(PrivateRootObservationError::NotAppContainer);
    }
    Ok(())
}

/// Reads one already-open root HANDLE between two exact process observations.
/// Even a successful report cannot prove the root belongs to this process,
/// that the AppContainer is unique, or that Minecraft resolves files here.
#[cfg(windows)]
pub(crate) fn inspect_private_root_for_process_source(
    slot: SessionSlot,
    expected: ProcessIdentity,
    opened: &std::fs::File,
) -> Result<PendingPrivateRootInspection, PrivateRootObservationError> {
    if slot.is_host_native() || expected.pid == 0 || expected.created_file_time == 0 {
        return Err(PrivateRootObservationError::InvalidSlotOrProcess);
    }
    let observer = WindowsProcessObserver;
    let before = observer.observe_exact(expected).map_err(PrivateRootObservationError::Native)?;
    // Avoid even reading a directory when the observed process is not
    // AppContainer-scoped. This does not assert owner SID equality.
    validate_private_root_observation(slot, expected, before, before)?;
    let key = ProcessKey {
        pid: expected.pid,
        created_file_time: expected.created_file_time.to_string(),
    };
    let (directory, sid_relations) = windows::inspect_directory_for_process(opened, &key)
        .map_err(|e| PrivateRootObservationError::Directory(e.kind()))?;
    let after = observer.observe_exact(expected).map_err(PrivateRootObservationError::Native)?;
    validate_private_root_observation(slot, expected, before, after)?;
    Ok(PendingPrivateRootInspection {
        slot, process: expected, directory, sid_relations,
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

/// Read-only detection of the *native Windows* Process Security Environment
/// API. It never creates a security environment, changes DACLs, launches a
/// process, adopts a Minecraft profile or grants MC02 Launch/Provision.
///
/// PSEC filesystem permissions are NOT filesystem redirection; even a fully
/// supported OS API cannot establish private Roaming AppData, HKCU or WAM.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NativePsecInspection {
    pub schema: u32,
    pub evidence_class: &'static str,
    pub api_present: bool,
    pub version_supported: bool,
    pub contract_minor: Option<u32>,
    pub fs_deny_supported: Option<bool>,
    pub network_proxy_supported: Option<bool>,
}

fn unavailable_psec() -> NativePsecInspection {
    NativePsecInspection {
        schema: 1,
        evidence_class: "WINDOWS_PSEC_CAPABILITY_ONLY_NO_GAME_ISOLATION",
        api_present: false,
        version_supported: false,
        contract_minor: None,
        fs_deny_supported: None,
        network_proxy_supported: None,
    }
}

pub fn inspect_native_psec_capabilities() -> std::io::Result<NativePsecInspection> {
    #[cfg(windows)]
    {
        psec_windows::inspect()
    }
    #[cfg(not(windows))]
    {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Native Process Security Environment requires Windows",
        ))
    }
}

#[cfg(windows)]
mod psec_windows {
    use super::{unavailable_psec, NativePsecInspection};
    use std::{io, mem, ptr};
    use windows_sys::Win32::Foundation::FreeLibrary;
    use windows_sys::Win32::System::LibraryLoader::{
        GetProcAddress, LoadLibraryExW, LOAD_LIBRARY_SEARCH_SYSTEM32,
    };

    // The only loaded DLL is the actual Windows System32 component. Never
    // search PATH, an install directory, a user profile or an arbitrary DLL.
    pub(super) fn inspect() -> io::Result<NativePsecInspection> {
        let library: Vec<u16> = "processmodel.dll\0".encode_utf16().collect();
        let dll = unsafe {
            LoadLibraryExW(library.as_ptr(), ptr::null_mut(), LOAD_LIBRARY_SEARCH_SYSTEM32)
        };
        if dll.is_null() {
            let error = io::Error::last_os_error();
            if matches!(error.raw_os_error(), Some(2 | 126)) {
                return Ok(unavailable_psec());
            }
            return Err(error);
        }
        struct LoadedDll(windows_sys::Win32::Foundation::HMODULE);
        impl Drop for LoadedDll {
            fn drop(&mut self) { unsafe { FreeLibrary(self.0); } }
        }
        let dll = LoadedDll(dll);

        // Require the capability and release/creation symbols as a coherent
        // family; an export by itself is NOT a supported PSEC contract.
        let query = unsafe {
            GetProcAddress(dll.0, b"QueryProcessSecurityEnvironmentSupport\0".as_ptr())
        };
        let version = unsafe {
            GetProcAddress(dll.0, b"IsProcessSecurityEnvironmentVersionSupported\0".as_ptr())
        };
        let creator = unsafe {
            GetProcAddress(dll.0, b"CreateProcessSecurityEnvironment\0".as_ptr())
        };
        let closer = unsafe {
            GetProcAddress(dll.0, b"CloseProcessSecurityEnvironment\0".as_ptr())
        };
        if query.is_none() || version.is_none() || creator.is_none() || closer.is_none() {
            return Ok(unavailable_psec());
        }

        // Version check: DWORD, BOOLEAN*, DWORD*, returning HRESULT.
        // Current MXC PSEC V2 query uses a UINT64* output bitmask, whereas
        // the public processmodel.h page names an enum pointer. Reserve a
        // full eight-byte output: both a 32-bit enum write and a 64-bit V2
        // write stay in bounds. Keep this diagnostic non-authorizing.
        type VersionCheck = unsafe extern "system" fn(u32, *mut u8, *mut u32) -> i32;
        type SupportQuery = unsafe extern "system" fn(*mut u64) -> i32;
        let check_version: VersionCheck = unsafe { mem::transmute(version.unwrap()) };
        let query_support: SupportQuery = unsafe { mem::transmute(query.unwrap()) };
        let mut available: u8 = 0;
        let mut minor: u32 = 0;
        if unsafe { check_version(1, &mut available, &mut minor) } < 0 {
            return Err(io::Error::other("Native Windows PSEC version query failed"));
        }
        if available == 0 {
            return Ok(NativePsecInspection {
                api_present: true,
                ..unavailable_psec()
            });
        }
        if available != 1 {
            return Err(io::Error::other("Invalid Windows PSEC availability response"));
        }

        let mut support_flags: u64 = 0;
        if unsafe { query_support(&mut support_flags) } < 0 {
            return Err(io::Error::other("Native Windows PSEC policy query failed"));
        }
        Ok(NativePsecInspection {
            schema: 1,
            evidence_class: "WINDOWS_PSEC_CAPABILITY_ONLY_NO_GAME_ISOLATION",
            api_present: true,
            version_supported: true,
            contract_minor: Some(minor),
            // Current published PROCESS_SECURITY_ENVIRONMENT_SUPPORT_FLAGS:
            // PSE_SUPPORT_FS_DENY=1, PSE_SUPPORT_NETWORK_PROXY=2.
            fs_deny_supported: Some(support_flags & 1 != 0),
            network_proxy_supported: Some(support_flags & 2 != 0),
        })
    }
}

#[cfg(windows)]
mod windows;
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
mod tests;

// F3 TCP owner feasibility, test-only: no Windows IP Helper calls, listener,
// transport, world attribution, clipboard, session owner or Copy admission.
#[cfg(test)]
mod coordinate_tcp_pid_witness_fixture;
