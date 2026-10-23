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
    use windows_sys::Win32::System::LibraryLoader::{
        FreeLibrary, GetProcAddress, LoadLibraryExW, LOAD_LIBRARY_SEARCH_SYSTEM32,
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
mod windows {
    use super::{
        AppContainerComparison, ContextComparison, ProcessContainment, ProcessKey,
        ObservedProcessPackageForm, ObservedPackageFamilyRelation,
        classify_package_probe, classify_package_family_pair, finish_file_signature_review,
    };
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
            GetProcAddress, LoadLibraryExW, LOAD_LIBRARY_SEARCH_SYSTEM32,
        },
        System::Threading::{
            GetCurrentProcess, GetCurrentThread, GetProcessTimes, OpenProcess,
            OpenProcessToken, QueryFullProcessImageNameW, WaitForSingleObject,
            PROCESS_QUERY_LIMITED_INFORMATION,
            PROCESS_SYNCHRONIZE,
        },
        Storage::FileSystem::{
            GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
            FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT,
            FILE_SHARE_READ,
        },
        UI::Shell::GetUserProfileDirectoryW,
    };

    // Windows-sys 0.61 does not project FreeLibrary in LibraryLoader.
    // Keep this one exact kernel32 ABI local to the loaded-system-DLL owner.
    // No calls from DllMain, no borrowed GetModuleHandle handles.
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn FreeLibrary(h_lib_module: *mut std::ffi::c_void) -> i32;
        fn LocalFree(h_mem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
        fn GetFinalPathNameByHandleW(
            handle: HANDLE, buffer: *mut u16, capacity: u32, flags: u32,
        ) -> u32;
        fn lstrlenW(text: *const u16) -> i32;
        fn GetPackageFullName(
            process: HANDLE, name_length: *mut u32, name: *mut u16,
        ) -> i32;
        fn GetPackageFamilyName(
            process: HANDLE, name_length: *mut u32, name: *mut u16,
        ) -> i32;
    }

    // GetSecurityInfo returns a DWORD status (not GetLastError), with
    // owner/DACL pointers borrowed from the LocalFree-owned descriptor.
    // SE_FILE_OBJECT=1, OWNER_SECURITY_INFORMATION=1, DACL_SECURITY_INFORMATION=4.
    // The handle must already carry READ_CONTROL; never enable privileges.
    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn GetSecurityInfo(
            handle: HANDLE,
            object_type: i32,
            security_information: u32,
            owner_sid: *mut *mut std::ffi::c_void,
            group_sid: *mut *mut std::ffi::c_void,
            dacl: *mut *mut std::ffi::c_void,
            sacl: *mut *mut std::ffi::c_void,
            security_descriptor: *mut *mut std::ffi::c_void,
        ) -> u32;
        fn IsValidAcl(acl: *mut std::ffi::c_void) -> i32;
        fn GetAclInformation(
            acl: *mut std::ffi::c_void,
            information: *mut std::ffi::c_void,
            size: u32,
            information_class: i32,
        ) -> i32;
        fn GetAce(
            acl: *mut std::ffi::c_void,
            index: u32,
            ace: *mut *mut std::ffi::c_void,
        ) -> i32;
        fn OpenThreadToken(
            thread: HANDLE,
            desired_access: u32,
            open_as_self: i32,
            token: *mut HANDLE,
        ) -> i32;
    }

    /// Only a minimal negative check. A non-null DACL is not a private ACL:
    /// broad Everyone/Users ACEs, inherited rights, ancestor junctions and
    /// the target game's actual file view remain unresolved.
    fn reject_invalid_directory_acl(owner_valid: bool, dacl_present: bool) -> io::Result<()> {
        if !owner_valid || !dacl_present {
            return Err(invalid("Private root owner/DACL is absent or invalid"));
        }
        Ok(())
    }



    #[link(name = "userenv")]
    unsafe extern "system" {
        fn GetAppContainerFolderPath(
            appcontainer_sid: *const u16,
            folder_path: *mut *mut u16,
        ) -> i32;
    }
    #[link(name = "ole32")]
    unsafe extern "system" {
        fn CoTaskMemFree(memory: *mut std::ffi::c_void);
    }

    // Convert *only* an authenticated TokenAppContainerSid from its canonical
    // SID binary representation to S-1-15-2-... . No caller-supplied or
    // account SID string is permitted. The OS provides the actual folder path.
    fn container_sid_text(sid: &[u8]) -> io::Result<String> {
        if sid.len() < 16 || sid[0] != 1 || !(2..=15).contains(&sid[1])
            || &sid[2..8] != [0, 0, 0, 0, 0, 15].as_slice()
            || sid.len() != 8 + 4 * usize::from(sid[1])
            || u32::from_le_bytes(sid[8..12].try_into().unwrap_or([0;4])) != 2
        {
            return Err(invalid("Not a canonical Windows AppContainer SID"));
        }
        let mut text = String::from("S-1-15");
        for chunk in sid[8..].chunks_exact(4) {
            let value = u32::from_le_bytes(chunk.try_into().unwrap_or([0; 4]));
            text.push('-');
            text.push_str(&value.to_string());
        }
        Ok(text)
    }

    // GetAppContainerFolderPath is CALLER-user-sensitive, not an operation
    // performed on a process token. Refuse a thread impersonation token and
    // a different Windows user before using it. Never report a false match
    // when another user owns the running AppContainer.
    fn no_thread_impersonation() -> io::Result<()> {
        const ERROR_NO_TOKEN: u32 = 1008;
        let mut raw: HANDLE = ptr::null_mut();
        let result = unsafe {
            OpenThreadToken(GetCurrentThread(), TOKEN_QUERY, 1, &mut raw)
        };
        if result != 0 || !raw.is_null() {
            if !raw.is_null() {
                let _owned = Handle(raw);
            }
            return Err(invalid("AppContainer folder lookup forbidden under impersonation"));
        }
        if unsafe { GetLastError() } != ERROR_NO_TOKEN {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    fn same_manager_windows_user(target_user: &[u8]) -> io::Result<bool> {
        let mut raw: HANDLE = ptr::null_mut();
        if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut raw) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let token = Handle(raw);
        let manager = user_sid_from_token(&token)?;
        Ok(manager == target_user)
    }

    fn appcontainer_local_app_data_path(sid: &[u8]) -> io::Result<std::path::PathBuf> {
        use std::ffi::OsString;
        let utf16: Vec<u16> = container_sid_text(sid)?
            .encode_utf16().chain(Some(0)).collect();
        let mut raw: *mut u16 = ptr::null_mut();
        let status = unsafe { GetAppContainerFolderPath(utf16.as_ptr(), &mut raw) };
        struct TaskPath(*mut u16);
        impl Drop for TaskPath {
            fn drop(&mut self) {
                if !self.0.is_null() {
                    unsafe { CoTaskMemFree(self.0.cast()) };
                }
            }
        }
        let owned = TaskPath(raw);
        if status < 0 || owned.0.is_null() {
            return Err(invalid("Windows AppContainer LocalAppData path unavailable"));
        }
        // The OS API returns an allocated null-terminated UTF-16 path.
        // lstrlenW is used only on OS-owned, validated successful output.
        let count = unsafe { lstrlenW(owned.0) };
        if !(4..=32760).contains(&count) {
            return Err(invalid("Invalid OS-owned AppContainer folder path size"));
        }
        let path = unsafe { std::slice::from_raw_parts(owned.0, count as usize) };
        Ok(std::path::PathBuf::from(OsString::from_wide(path)))
    }

    fn os_container_local_app_data_match(
        process_user: &[u8],
        container: &[u8],
        opened_root: super::PrivateRootObjectIdentity,
    ) -> io::Result<Option<bool>> {
        no_thread_impersonation()?;
        if !same_manager_windows_user(process_user)? {
            return Ok(None);
        }
        let path = appcontainer_local_app_data_path(container)?;
        use windows_sys::Win32::Storage::FileSystem::{
            FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
            FILE_SHARE_WRITE,
        };
        // Existing OS-returned location only. No Create, no data reads and
        // no accounts/worlds; do not follow the final reparse point.
        let folder = std::fs::OpenOptions::new()
            .access_mode(0x80 | 0x20 | 0x0002_0000) // FILE_READ_ATTRIBUTES, FILE_TRAVERSE, READ_CONTROL
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)?;
        let expected = ancestor_directory_identity(&folder)?;
        if expected != opened_root {
            return Ok(Some(false));
        }
        inspect_opened_ancestor_chain(&folder, expected)?;
        Ok(Some(true))
    }

    // These well-known SID byte encodings are fixed Windows SID formats:
    // S-1-1-0 (Everyone), S-1-5-11 (Authenticated Users),
    // S-1-5-7 (Anonymous), S-1-5-32-545 (Builtin Users) and
    // S-1-5-32-546 (Builtin Guests). Domain-specific groups are UNKNOWN.
    const SID_EVERYONE: &[u8] = &[1,1,0,0,0,0,0,1,0,0,0,0];
    const SID_AUTHENTICATED: &[u8] = &[1,1,0,0,0,0,0,5,11,0,0,0];
    const SID_ANONYMOUS: &[u8] = &[1,1,0,0,0,0,0,5,7,0,0,0];
    const SID_USERS: &[u8] = &[1,2,0,0,0,0,0,5,32,0,0,0,33,2,0,0];
    const SID_GUESTS: &[u8] = &[1,2,0,0,0,0,0,5,32,0,0,0,34,2,0,0];

    /// Source- and input-bounded basic ACE classifier. Never grants an
    /// effective-access conclusion: deny ACE ordering, owner privileges,
    /// group membership, inherited access and callback/object ACEs require
    /// a separately authenticated Windows authorization evaluation.
    fn reject_broad_directory_ace(ace: &[u8]) -> io::Result<()> {
        // Simple ACCESS_ALLOWED_ACE_TYPE=0 and ACCESS_DENIED_ACE_TYPE=1.
        // Callback, conditional and object ACE layouts differ: fail closed.
        if ace.len() < 20 || !matches!(ace[0], 0 | 1) {
            return Err(invalid("Unsupported or undersized private root ACE"));
        }
        let declared = u16::from_le_bytes([ace[2], ace[3]]) as usize;
        let subauthorities = usize::from(ace[9]);
        // The SID header is 8 bytes after the eight-byte ACE header/mask.
        // A basic SID is revision 1 with no more than 15 subauthorities.
        if declared != ace.len() || ace[8] != 1 || subauthorities > 15
            || 16 + 4 * subauthorities != ace.len()
        {
            return Err(invalid("Malformed basic private root ACE SID"));
        }
        let sid = &ace[8..];
        let broad = [SID_EVERYONE, SID_AUTHENTICATED, SID_ANONYMOUS, SID_USERS, SID_GUESTS]
            .contains(&sid);
        let mask = u32::from_le_bytes([ace[4], ace[5], ace[6], ace[7]]);
        // Conservative policy: any nonzero allow ACE for these wide
        // principals is unsuitable for a private manager-owned root,
        // including read-only and inheritable ACEs for child content.
        if ace[0] == 0 && broad && mask != 0 {
            return Err(invalid("Private root DACL grants access to a broad group"));
        }
        Ok(())
    }

    #[repr(C)]
    struct AclSizeInformation {
        ace_count: u32,
        acl_bytes_in_use: u32,
        acl_bytes_free: u32,
    }

    /// The returned ACL belongs to the still-live GetSecurityInfo descriptor.
    /// ACL size and iteration are capped to avoid unbounded native work.
    // Syntactic ACE reference only; token group state and ACE ordering
    // can make a matching entry irrelevant to effective access.
    fn basic_allow_for_sid(ace: &[u8], sid: &[u8]) -> bool {
        ace[0] == 0
            && u32::from_le_bytes([ace[4], ace[5], ace[6], ace[7]]) != 0
            && &ace[8..] == sid
    }

    fn inspect_directory_acl(
        acl: *mut std::ffi::c_void,
        expected_container: Option<&[u8]>,
    ) -> io::Result<bool> {
        if acl.is_null() || unsafe { IsValidAcl(acl) } == 0 {
            return Err(invalid("Private root DACL is missing or invalid"));
        }
        let mut info = AclSizeInformation {
            ace_count: 0,
            acl_bytes_in_use: 0,
            acl_bytes_free: 0,
        };
        // ACL_INFORMATION_CLASS::AclSizeInformation = 2.
        if unsafe {
            GetAclInformation(
                acl,
                (&mut info as *mut AclSizeInformation).cast(),
                size_of::<AclSizeInformation>() as u32,
                2,
            )
        } == 0 {
            return Err(io::Error::last_os_error());
        }
        // Empty DACL grants nobody access and is unusable as an ordinary
        // session profile; an oversized/complex DACL must be manually reviewed.
        if !(1..=64).contains(&info.ace_count)
            || !(8..=8192).contains(&info.acl_bytes_in_use)
        {
            return Err(invalid("Private root ACL count or size unsupported"));
        }
        let mut container_allow = false;
        for index in 0..info.ace_count {
            let mut raw: *mut std::ffi::c_void = ptr::null_mut();
            if unsafe { GetAce(acl, index, &mut raw) } == 0 || raw.is_null() {
                return Err(invalid("Private root ACL entry unavailable"));
            }
            // GetAce supplies a pointer into a validated ACL allocation.
            let base = raw.cast::<u8>();
            let len = unsafe {
                u16::from_le_bytes([*base.add(2), *base.add(3)]) as usize
            };
            if !(20..=1024).contains(&len) || len > info.acl_bytes_in_use as usize {
                return Err(invalid("Private root ACL entry length invalid"));
            }
            let entry = unsafe { std::slice::from_raw_parts(base, len) };
            reject_broad_directory_ace(entry)?;
            if expected_container.is_some_and(|sid| basic_allow_for_sid(entry, sid)) {
                container_allow = true;
            }
        }
        Ok(container_allow)
    }

    fn inspect_directory_security(
        opened: &std::fs::File,
        expected_user: Option<&[u8]>,
        expected_container: Option<&[u8]>,
    ) -> io::Result<super::PendingRootSidRelations> {
        struct Descriptor(*mut std::ffi::c_void);
        impl Drop for Descriptor {
            fn drop(&mut self) {
                if !self.0.is_null() {
                    // GetSecurityInfo memory is always freed with LocalFree,
                    // never CloseHandle/FreeLibrary or Rust's allocator.
                    let _ = unsafe { LocalFree(self.0) };
                }
            }
        }
        let mut owner: *mut std::ffi::c_void = ptr::null_mut();
        let mut dacl: *mut std::ffi::c_void = ptr::null_mut();
        let mut raw_descriptor: *mut std::ffi::c_void = ptr::null_mut();
        let status = unsafe {
            GetSecurityInfo(
                opened.as_raw_handle(),
                1, // SE_FILE_OBJECT
                1 | 4, // OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION
                &mut owner,
                ptr::null_mut(),
                &mut dacl,
                ptr::null_mut(),
                &mut raw_descriptor,
            )
        };
        let _descriptor = Descriptor(raw_descriptor);
        if status != 0 {
            return Err(io::Error::from_raw_os_error(status as i32));
        }
        if raw_descriptor.is_null() {
            return Err(invalid("Windows returned no private root security descriptor"));
        }
        let owner_valid = !owner.is_null() && unsafe { IsValidSid(owner) } != 0;
        reject_invalid_directory_acl(owner_valid, !dacl.is_null())?;
        let len = unsafe { GetLengthSid(owner) } as usize;
        if !(8..=68).contains(&len) {
            return Err(invalid("Private root owner SID length invalid"));
        }
        let owner_sid = unsafe { std::slice::from_raw_parts(owner.cast::<u8>(), len) };
        let allow = inspect_directory_acl(dacl, expected_container)?;
        Ok(super::PendingRootSidRelations {
            owner_matches_process_user: expected_user.is_some_and(|sid| sid == owner_sid),
            owner_matches_container: expected_container.is_some_and(|sid| sid == owner_sid),
            container_allow_ace_observed: allow,
            os_container_local_app_data_match: None,
        })
    }

    /// Inspect one already-open directory object and its bounded ACL metadata.
    /// Refuse known broad grants and unsupported ACE structures, but this is
    /// NOT an effective Windows permission check or proof of account ownership.

    // Strict read-only inspection of the prefixes of a previously opened
    // directory. Windows returned the path, not a user/Tauri input.
    // A path-only check is never a race-free root access capability.
    fn opened_dos_prefixes(path: &str) -> io::Result<Vec<String>> {
        let value = path.strip_prefix(r"\\?\")
            .ok_or_else(|| invalid("Unsupported root path prefix"))?;
        let bytes = value.as_bytes();
        if bytes.len() < 4 || !bytes[0].is_ascii_alphabetic()
            || bytes[1] != b':' || bytes[2] != b'\\'
            || value.contains('/') || value.contains('\0')
        {
            return Err(invalid("Root is not an absolute local DOS drive path"));
        }
        let rest = &value[3..];
        if rest.is_empty() || rest.ends_with('\\') {
            return Err(invalid("Root has no supported directory component"));
        }
        let parts: Vec<&str> = rest.split('\\').collect();
        if parts.len() > 32 {
            return Err(invalid("Root ancestor depth exceeds safety limit"));
        }
        let mut current = format!("\\\\?\\{}:\\", &value[..1]);
        let mut prefixes = vec![current.clone()];
        for part in parts {
            if part.is_empty() || matches!(part, "." | "..")
                || part.ends_with('.') || part.ends_with(' ')
                || part.chars().any(|ch| ch.is_control()
                    || matches!(ch, ':' | '*' | '?' | '"' | '<' | '>' | '|'))
            {
                return Err(invalid("Ambiguous root ancestor component"));
            }
            current.push_str(part);
            prefixes.push(current.clone());
            current.push('\\');
        }
        Ok(prefixes)
    }

    fn opened_root_name(file: &std::fs::File) -> io::Result<String> {
        const FILE_NAME_OPENED: u32 = 8;
        let handle = file.as_raw_handle();
        let needed = unsafe {
            GetFinalPathNameByHandleW(handle, ptr::null_mut(), 0, FILE_NAME_OPENED)
        };
        if !(5..=32760).contains(&needed) {
            return Err(invalid("Opened root path unavailable or oversized"));
        }
        let mut name = vec![0u16; needed as usize];
        let actual = unsafe {
            GetFinalPathNameByHandleW(handle, name.as_mut_ptr(), needed, FILE_NAME_OPENED)
        };
        if actual == 0 || actual >= needed || name[..actual as usize].contains(&0) {
            return Err(invalid("Opened root name could not be pinned"));
        }
        String::from_utf16(&name[..actual as usize])
            .map_err(|_| invalid("Invalid opened root UTF-16 name"))
    }

    fn ancestor_directory_identity(file: &std::fs::File)
        -> io::Result<super::PrivateRootObjectIdentity>
    {
        let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
        if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) } == 0 {
            return Err(io::Error::last_os_error());
        }
        if info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0
            || info.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
        {
            return Err(invalid("Ancestor is a file or reparse point"));
        }
        let file_index = (u64::from(info.nFileIndexHigh) << 32)
            | u64::from(info.nFileIndexLow);
        if file_index == 0 {
            return Err(invalid("Ancestor has no usable file index"));
        }
        Ok(super::PrivateRootObjectIdentity {
            volume: info.dwVolumeSerialNumber,
            file_index,
        })
    }


    // User-mode ntdll NtOpenFile, exactly the winternl.h six-argument ABI.
    // RootDirectory points to an OPENED directory HANDLE, not a DOS path.
    // These plain C-layout types are deliberately kept adjacent to the FFI;
    // IO_STATUS_BLOCK's first union occupies one pointer-sized slot.
    #[repr(C)]
    struct NtUnicodeString {
        length: u16,
        maximum_length: u16,
        buffer: *mut u16,
    }

    #[repr(C)]
    struct NtObjectAttributes {
        length: u32,
        root_directory: HANDLE,
        object_name: *mut NtUnicodeString,
        attributes: u32,
        security_descriptor: *mut std::ffi::c_void,
        security_quality_of_service: *mut std::ffi::c_void,
    }

    #[repr(C)]
    struct NtIoStatusBlock {
        status_or_pointer: usize,
        information: usize,
    }

    #[link(name = "ntdll")]
    unsafe extern "system" {
        fn NtOpenFile(
            file_handle: *mut HANDLE,
            desired_access: u32,
            object_attributes: *mut NtObjectAttributes,
            io_status_block: *mut NtIoStatusBlock,
            share_access: u32,
            open_options: u32,
        ) -> i32;
    }

    /// Open exactly ONE existing child component relative to an OS directory
    /// handle. This neither creates a file nor permits read/write data I/O.
    /// The parent stays open at least until the child has been inspected.
    /// Final-component reparse targets are opened AS reparse objects, never
    /// followed; a later metadata check rejects them.
    fn open_relative_directory_metadata(
        parent: &std::fs::File,
        component: &str,
    ) -> io::Result<std::fs::File> {
        use std::os::windows::io::FromRawHandle;
        const OBJ_CASE_INSENSITIVE: u32 = 0x40;
        const FILE_READ_ATTRIBUTES: u32 = 0x80;
        const FILE_TRAVERSE: u32 = 0x20;
        const SYNCHRONIZE: u32 = 0x0010_0000;
        const FILE_SYNCHRONOUS_IO_NONALERT: u32 = 0x20;
        const FILE_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
        const FILE_SHARE_READ: u32 = 1;
        const FILE_SHARE_WRITE: u32 = 2;

        // Require single, unambiguous components even if a future caller
        // bypasses the already validated opened-name parser.
        if component.is_empty() || component == "." || component == ".."
            || component.ends_with('.')
            || component.ends_with(' ') || component.chars().any(|ch|
                ch.is_control() || matches!(
                    ch, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|'
                ))
        {
            return Err(invalid("Invalid relative directory component"));
        }
        let mut wide: Vec<u16> = component.encode_utf16().collect();
        if wide.is_empty() || wide.len() > 255 {
            return Err(invalid("Relative directory name too long"));
        }
        let bytes = wide.len() * size_of::<u16>();
        let length = u16::try_from(bytes)
            .map_err(|_| invalid("Directory name exceeds NT length"))?;
        wide.push(0);
        let mut unicode = NtUnicodeString {
            length,
            maximum_length: length + 2,
            buffer: wide.as_mut_ptr(),
        };
        let mut attributes = NtObjectAttributes {
            length: size_of::<NtObjectAttributes>() as u32,
            root_directory: parent.as_raw_handle(),
            object_name: &mut unicode,
            attributes: OBJ_CASE_INSENSITIVE,
            security_descriptor: ptr::null_mut(),
            security_quality_of_service: ptr::null_mut(),
        };
        let mut iosb = NtIoStatusBlock {
            status_or_pointer: 0,
            information: 0,
        };
        let mut raw: HANDLE = ptr::null_mut();
        let status = unsafe {
            NtOpenFile(
                &mut raw,
                FILE_READ_ATTRIBUTES | FILE_TRAVERSE | SYNCHRONIZE,
                &mut attributes,
                &mut iosb,
                FILE_SHARE_READ | FILE_SHARE_WRITE, // no FILE_SHARE_DELETE
                FILE_SYNCHRONOUS_IO_NONALERT | FILE_OPEN_REPARSE_POINT,
            )
        };
        if status != 0 || raw.is_null() {
            if !raw.is_null() {
                let _ = unsafe { CloseHandle(raw) };
            }
            return Err(invalid("Failed to open child relative to a directory HANDLE"));
        }
        // Owned NT file handles are ordinary Windows file HANDLEs. File::Drop
        // uses CloseHandle; never close parent handles while children are in use.
        Ok(unsafe { std::fs::File::from_raw_handle(raw) })
    }

    // Open only the volume root by DOS name. Then open each descendant by
    // ONE component relative to a HELD parent HANDLE (NtOpenFile.RootDirectory).
    // Holding all parent handles without FILE_SHARE_DELETE reduces rename/swap
    // races. This cannot prove that the initial drive/root belongs to a
    // trusted profile owner; the result remains a pending diagnostic.
    fn inspect_opened_ancestor_chain(
        opened: &std::fs::File,
        expected: super::PrivateRootObjectIdentity,
    ) -> io::Result<()> {
        use windows_sys::Win32::Storage::FileSystem::{
            FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
        };
        let original_name = opened_root_name(opened)?;
        let prefixes = opened_dos_prefixes(&original_name)?;
        // Only the volume root uses a pathname. The remaining components
        // are opened relative to the preceding validated directory HANDLE.
        // Excluding FILE_SHARE_DELETE holds each ancestor against rename.
        let drive = std::fs::OpenOptions::new()
            .access_mode(0x80 | 0x20 | 0x0010_0000)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(std::path::Path::new(&prefixes[0]))?;
        let start = ancestor_directory_identity(&drive)?;
        if start.volume != expected.volume {
            return Err(invalid("Root drive volume mismatches opened directory"));
        }
        let mut held = vec![drive];
        for prefix in prefixes.iter().skip(1) {
            let component = prefix.rsplit('\\').next()
                .ok_or_else(|| invalid("Missing root ancestor component"))?;
            let child = open_relative_directory_metadata(
                held.last().expect("root directory is always held"), component,
            )?;
            let identity = ancestor_directory_identity(&child)?;
            if identity.volume != expected.volume {
                return Err(invalid("Ancestor crossed a volume boundary"));
            }
            held.push(child); // all ancestors stay open until final checks
        }
        if ancestor_directory_identity(
            held.last().expect("final root directory must be held"),
        )? != expected
        {
            return Err(invalid("Final ancestor is not original root object"));
        }
        if ancestor_directory_identity(opened)? != expected
            || opened_root_name(opened)? != original_name
        {
            return Err(invalid("Root changed during handle-relative inspection"));
        }
        Ok(())
    }

    fn inspect_directory_with_sids(
        opened: &std::fs::File,
        user: Option<&[u8]>,
        container: Option<&[u8]>,
    ) -> io::Result<(super::PrivateRootObjectIdentity, super::PendingRootSidRelations)> {
        fn snapshot(file: &std::fs::File) -> io::Result<super::PrivateRootObjectIdentity> {
            let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
            if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) } == 0 {
                return Err(io::Error::last_os_error());
            }
            // Must have been opened with FILE_FLAG_OPEN_REPARSE_POINT.
            // Checking a followed target alone cannot prove a safe ancestor.
            if info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0
                || info.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
            {
                return Err(invalid("Private root handle is not an ordinary directory"));
            }
            let index = (u64::from(info.nFileIndexHigh) << 32)
                | u64::from(info.nFileIndexLow);
            if index == 0 {
                return Err(invalid("Private root directory identity is unavailable"));
            }
            Ok(super::PrivateRootObjectIdentity {
                volume: info.dwVolumeSerialNumber,
                file_index: index,
            })
        }

        let before = snapshot(opened)?;
        // READ_CONTROL on the already-opened object, no path traversal and
        // no read of user account files, world data, or registry keys.
        let relations = inspect_directory_security(opened, user, container)?;
        if snapshot(opened)? != before {
            return Err(invalid("Directory changed during ACL observation"));
        }
        Ok((before, relations))
    }

    pub(super) fn inspect_directory_handle(
        opened: &std::fs::File,
    ) -> io::Result<super::PrivateRootObjectIdentity> {
        inspect_directory_with_sids(opened, None, None).map(|(identity, _)| identity)
    }

    /// Pin the exact process and its read-only token while inspecting one
    /// already-opened directory; never opens a profile by a caller path.
    pub(super) fn inspect_directory_for_process(
        opened: &std::fs::File,
        key: &ProcessKey,
    ) -> io::Result<(super::PrivateRootObjectIdentity, super::PendingRootSidRelations)> {
        let process = pin(key)?;
        let token = query_token(&process)?;
        // Only TokenUser and TokenAppContainerSid are needed here.
        // Avoid the unrelated GetUserProfileDirectoryW/session probes.
        let user = user_sid_from_token(&token)?;
        let container = appcontainer_sid_from_token(&token)?
            .ok_or_else(|| invalid("No AppContainer SID for scoped directory check"))?;
        let (directory, mut relations) =
            inspect_directory_with_sids(opened, Some(&user), Some(&container))?;
        inspect_opened_ancestor_chain(opened, directory)?;
        relations.os_container_local_app_data_match =
            os_container_local_app_data_match(&user, &container, directory)?;
        live(&process)?;
        Ok((directory, relations))
    }

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
    /// Read package identity without returning its name. Calling the first
    /// probe with a null buffer distinguishes MSIX package identity from an
    /// ordinary unpackaged executable on the SAME pinned process HANDLE.
    /// Metadata does not prove package origin, publisher or registry routing.
    pub(super) fn inspect_package_form(key: &ProcessKey) -> io::Result<ObservedProcessPackageForm> {
        let process = pin(key)?;
        let mut size = 0u32;
        let first = unsafe {
            GetPackageFullName(process.0, &mut size, ptr::null_mut())
        };
        let needed = classify_package_probe(first, size)
            .map_err(|_| invalid("Windows process package query failed"))?;
        let form = if let Some(count) = needed {
            let mut name = vec![0u16; count];
            let mut written = count as u32;
            let result = unsafe {
                GetPackageFullName(process.0, &mut written, name.as_mut_ptr())
            };
            if result != 0 || written < 2 || written as usize > count
                || name[written as usize - 1] != 0
                || name[..written as usize - 1].contains(&0)
            {
                return Err(invalid("Windows package identity changed or is invalid"));
            }
            // Deliberately discard package full name: no logging, hashing,
            // exporting or promotion to a trusted publisher is required.
            ObservedProcessPackageForm::Packaged
        } else {
            ObservedProcessPackageForm::Unpackaged
        };
        live(&process)?;
        Ok(form)
    }

    /// One read-only OS package-family query against an already pinned
    /// process HANDLE. Cap the UTF-16 buffer and never log or return names.
    fn package_family_name(process: &Handle) -> io::Result<Option<String>> {
        let mut required = 0u32;
        let first = unsafe {
            GetPackageFamilyName(process.0, &mut required, ptr::null_mut())
        };
        let Some(capacity) = classify_package_probe(first, required)
            .map_err(|_| invalid("Windows package-family query failed"))?
        else {
            return Ok(None);
        };
        let mut name = vec![0u16; capacity];
        let mut written = capacity as u32;
        let status = unsafe {
            GetPackageFamilyName(process.0, &mut written, name.as_mut_ptr())
        };
        if status != 0 || written < 2 || written as usize > capacity
            || name[written as usize - 1] != 0
            || name[..written as usize - 1].contains(&0)
        {
            return Err(invalid("Windows package-family identity changed or is invalid"));
        }
        String::from_utf16(&name[..written as usize - 1])
            .map(Some)
            .map_err(|_| invalid("Windows returned an invalid package-family string"))
    }

    /// Package names stay within this native call. Same family means only
    /// same OS-reported package family, not a shared user, a verified official
    /// Minecraft identity, or observed filesystem isolation.
    pub(super) fn compare_package_families(
        left: &ProcessKey,
        right: &ProcessKey,
    ) -> io::Result<ObservedPackageFamilyRelation> {
        let first = pin(left)?;
        let second = pin(right)?;
        let first_family = package_family_name(&first)?;
        let second_family = package_family_name(&second)?;
        live(&first)?;
        live(&second)?;
        Ok(classify_package_family_pair(
            first_family.as_deref(), second_family.as_deref(),
        ))
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
            fn drop(&mut self) {
                if !self.0.is_null() {
                    // A LoadLibraryExW-owned reference must be released once.
                    let _ = unsafe { FreeLibrary(self.0) };
                }
            }
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
    /// One read-only token handle per process observation. The same token
    /// is reused for user/profile/session and AppContainer SID in compare_pair.
    fn query_token(process: &Handle) -> io::Result<Handle> {
        let mut raw = ptr::null_mut();
        if unsafe { OpenProcessToken(process.0, TOKEN_QUERY, &mut raw) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Handle(raw))
    }

    // Read the user SID only. A private-root comparison does not need a
    // profile path or a Windows session query.
    fn user_sid_from_token(token: &Handle) -> io::Result<Vec<u8>> {
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
        Ok(unsafe { std::slice::from_raw_parts(sid.cast::<u8>(), length) }.to_vec())
    }

    fn context_from_token(token: &Handle) -> io::Result<Context> {
        let user = user_sid_from_token(token)?;
        let mut needed = 0u32;
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
    fn context(process: &Handle) -> io::Result<Context> {
        let token = query_token(process)?;
        context_from_token(&token)
    }

    // An independent, bounded token inspection. Reuses the existing pinned
    // process handles; never reads folders, broker stores or account data.
    fn appcontainer_sid_from_token(token: &Handle) -> io::Result<Option<Vec<u8>>> {
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

    fn appcontainer_sid(process: &Handle) -> io::Result<Option<Vec<u8>>> {
        let token = query_token(process)?;
        appcontainer_sid_from_token(&token)
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

    // Inspecting the token and profile on already-pinned handles does not
    // authenticate Microsoft/Minecraft accounts. It only returns booleans.
    fn compare_appcontainer_values(a: Option<&[u8]>, b: Option<&[u8]>) -> AppContainerComparison {
        let both = a.is_some() && b.is_some();
        AppContainerComparison {
            both_appcontainers: both,
            distinct_container_identities: both && a != b,
        }
    }

    fn compare_context_values(a: &Context, b: &Context) -> io::Result<ContextComparison> {
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
        Ok(ContextComparison {
            same_windows_user: a.user == b.user,
            same_profile_directory: profile_equal == CSTR_EQUAL,
            same_windows_session: a.session == b.session,
        })
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
        Ok(compare_appcontainer_values(a.as_deref(), b.as_deref()))
    }

    pub(super) fn compare(left: &ProcessKey, right: &ProcessKey) -> io::Result<ContextComparison> {
        let left_process = pin(left)?;
        let right_process = pin(right)?;
        let a = context(&left_process)?;
        let b = context(&right_process)?;
        let comparison = compare_context_values(&a, &b)?;
        live(&left_process)?;
        live(&right_process)?;
        Ok(comparison)
    }

    /// One coherent pairwise read: two pinned process HANDLEs and two
    /// read-only token HANDLEs (one for each process). Profile/session/SIDs
    /// are read from those exact token handles, without reopening a token
    /// for the AppContainer query. Final process liveness is checked.
    pub(super) fn compare_pair(
        left: &ProcessKey,
        right: &ProcessKey,
    ) -> io::Result<(ContextComparison, AppContainerComparison)> {
        let left_process = pin(left)?;
        let right_process = pin(right)?;
        let first_token = query_token(&left_process)?;
        let second_token = query_token(&right_process)?;
        let first = context_from_token(&first_token)?;
        let second = context_from_token(&second_token)?;
        let first_container = appcontainer_sid_from_token(&first_token)?;
        let second_container = appcontainer_sid_from_token(&second_token)?;
        let context_result = compare_context_values(&first, &second)?;
        let container_result =
            compare_appcontainer_values(first_container.as_deref(), second_container.as_deref());
        live(&left_process)?;
        live(&right_process)?;
        Ok((context_result, container_result))
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
        fn appcontainer_sid_text_accepts_only_canonical_binary_package_sids() {
            let value = [
                1u8,2,0,0,0,0,0,15,2,0,0,0,17,0,0,0,
            ];
            assert_eq!(container_sid_text(&value).unwrap(), "S-1-15-2-17");
            for bad in [
                &value[..15],
                &[2,2,0,0,0,0,0,15,2,0,0,0,17,0,0,0][..],
                &[1,2,0,0,0,0,0,5,2,0,0,0,17,0,0,0][..],
                &[1,2,0,0,0,0,0,15,3,0,0,0,17,0,0,0][..],
                &[1,16,0,0,0,0,0,15,2,0,0,0,17,0,0,0][..],
            ] {
                assert!(container_sid_text(bad).is_err());
            }
        }

        #[test]
        fn current_process_package_form_is_sanitized_and_stale_key_fails() {
            let own = own_key();
            let valid = inspect_package_form(&own).unwrap();
            assert!(matches!(
                valid,
                ObservedProcessPackageForm::Packaged
                    | ObservedProcessPackageForm::Unpackaged
            ));
            let stale = ProcessKey {
                pid: own.pid,
                created_file_time: "1".into(),
            };
            assert!(inspect_package_form(&stale).is_err());
        }

        #[test]
        fn current_process_family_query_agrees_with_package_form_without_exposing_names() {
            let key = own_key();
            let process = pin(&key).unwrap();
            let observed = package_family_name(&process).unwrap();
            live(&process).unwrap();
            assert_eq!(
                observed.is_some(),
                inspect_package_form(&key).unwrap() == ObservedProcessPackageForm::Packaged,
            );
            let recycled = ProcessKey {
                pid: key.pid, created_file_time: "1".to_owned(),
            };
            assert!(pin(&recycled).is_err());
        }

        #[test]
        fn combined_native_pair_matches_separate_read_only_inspections() {
            let key = own_key();
            let (context, container) = compare_pair(&key, &key).unwrap();
            assert_eq!(context, compare(&key, &key).unwrap());
            assert_eq!(container, compare_appcontainers(&key, &key).unwrap());
            assert!(!container.distinct_container_identities);

            let stale = ProcessKey {
                pid: key.pid,
                created_file_time: "1".into(),
            };
            assert!(compare_pair(&key, &stale).is_err());
            assert!(compare_pair(&stale, &key).is_err());
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
        fn token_user_only_reader_agrees_with_full_context_without_export() {
            let key = own_key();
            let process = pin(&key).unwrap();
            let token = query_token(&process).unwrap();
            let direct = user_sid_from_token(&token).unwrap();
            let context = context_from_token(&token).unwrap();
            assert_eq!(direct, context.user);
            assert!(direct.len() >= 8 && direct.len() <= 68);
            live(&process).unwrap();
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
        fn relative_directory_open_does_not_follow_arbitrary_path_segments() {
            let root = std::env::temp_dir().join(format!(
                "mivubi-relative-root-{}-{}",
                std::process::id(),
                std::time::SystemTime::now().duration_since(
                    std::time::UNIX_EPOCH,
                ).unwrap().as_nanos(),
            ));
            std::fs::create_dir(&root).unwrap();
            struct Cleanup(std::path::PathBuf);
            impl Drop for Cleanup {
                fn drop(&mut self) {
                    let _ = std::fs::remove_dir_all(&self.0);
                }
            }
            let _cleanup = Cleanup(root.clone());
            std::fs::create_dir(root.join("only-child")).unwrap();
            std::fs::create_dir(root.join(".private")).unwrap();
            std::fs::write(root.join("regular-file.txt"), b"synthetic").unwrap();
            use std::os::windows::fs::OpenOptionsExt;
            use windows_sys::Win32::Storage::FileSystem::{
                FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
            };
            let parent = std::fs::OpenOptions::new()
                .read(true)
                .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
                .open(&root).unwrap();
            let child = open_relative_directory_metadata(&parent, "only-child").unwrap();
            assert!(ancestor_directory_identity(&child).is_ok());
            // Dot-prefixed names are legitimate directories; only "."/".."
            // and ambiguous trailing/stream/path syntax are refused.
            let hidden = open_relative_directory_metadata(&parent, ".private").unwrap();
            assert!(ancestor_directory_identity(&hidden).is_ok());
            // NtOpenFile may return a regular file handle when not asking for
            // FILE_DIRECTORY_FILE: type-checking the resulting handle is mandatory.
            if let Ok(file) = open_relative_directory_metadata(&parent, "regular-file.txt") {
                assert!(ancestor_directory_identity(&file).is_err());
            }
            for bad in ["", ".", "..", r"..\peer", r"\absolute",
                "other/child", "file:stream", "trailing.", "trailing ",
            ] {
                assert!(open_relative_directory_metadata(&parent, bad).is_err());
            }
        }

        #[test]
        fn opened_dos_chain_rejects_unc_and_traversal_without_any_os_call() {
            let good = opened_dos_prefixes(r"\\?\C:\data\mc02").unwrap();
            assert!(opened_dos_prefixes(r"\\?\C:\data\.private").is_ok());
            assert_eq!(good, vec![
                r"\\?\C:\".to_owned(),
                r"\\?\C:\data".to_owned(),
                r"\\?\C:\data\mc02".to_owned(),
            ]);
            for bad in [
                "", r"C:\data\mc02", r"\\server\share\mc02",
                r"\\?\UNC\server\share\mc02", r"\\?\Volume{demo}\mc02",
                r"\\?\C:\", r"\\?\C:\data\", r"\\?\C:\data\\mc02",
                r"\\?\C:\data\..\peer", r"\\?\C:\data\.\mc02",
                r"\\?\C:\data\bad.", r"\\?\C:\data\bad ",
                r"\\?\C:\data\file:stream", r"\\?\C:\data/bad",
                "\\\\?\\C:\\data\\nul\0component",
            ] {
                assert!(opened_dos_prefixes(bad).is_err(),
                    "Ambiguous ancestor passed: {bad:?}");
            }
            let deep = format!("\\\\?\\C:\\{}", vec!["name"; 33].join("\\"));
            assert!(opened_dos_prefixes(&deep).is_err());
        }

        #[test]
        fn regular_file_handle_is_not_a_private_directory_root() {
            // Read-only validation; never probes a host account folder.
            let executable = std::fs::File::open(std::env::current_exe().unwrap()).unwrap();
            let result = inspect_directory_handle(&executable);
            assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidData);
        }

        #[test]
        fn incomplete_directory_acl_never_counts_as_a_checked_root() {
            for (owner, dacl) in [(false, false), (false, true), (true, false)] {
                assert_eq!(
                    reject_invalid_directory_acl(owner, dacl).unwrap_err().kind(),
                    io::ErrorKind::InvalidData,
                );
            }
            assert!(reject_invalid_directory_acl(true, true).is_ok());
            // Even two true inputs are NOT an ACL effectiveness or
            // account-isolation attestation, just a structural check.
        }

        #[test]
        fn basic_dacl_ace_rejects_broad_grants_and_unsupported_types() {
            // ACL examples are synthetic only; no OS/account/registry access.
            fn ace(kind: u8, mask: u32, sid: &[u8]) -> Vec<u8> {
                let mut data = vec![kind, 0, 0, 0];
                data[2..4].copy_from_slice(&((8 + sid.len()) as u16).to_le_bytes());
                data.extend_from_slice(&mask.to_le_bytes());
                data.extend_from_slice(sid);
                data
            }
            let groups = [
                SID_EVERYONE, SID_AUTHENTICATED, SID_ANONYMOUS,
                SID_USERS, SID_GUESTS,
            ];
            for group in groups {
                assert_eq!(
                    reject_broad_directory_ace(&ace(0, 1, group))
                        .unwrap_err().kind(),
                    io::ErrorKind::InvalidData,
                );
                // Deny does not grant access; it cannot override a separate
                // broad allow ACE elsewhere, which is independently rejected.
                assert!(reject_broad_directory_ace(&ace(1, 1, group)).is_ok());
            }
            let custom = &[1,1,0,0,0,0,0,5,21,0,0,0];
            assert!(reject_broad_directory_ace(&ace(0, 1, custom)).is_ok());
            for bad in [
                ace(5, 1, custom), // object ACE needs a different layout
                ace(9, 1, custom), // callback ACE
                ace(255, 1, custom),
            ] {
                assert_eq!(
                    reject_broad_directory_ace(&bad).unwrap_err().kind(),
                    io::ErrorKind::InvalidData,
                );
            }
        }

        #[test]
        fn malformed_basic_ace_is_rejected_before_sid_interpretation() {
            let world = [1,1,0,0,0,0,0,1,0,0,0,0];
            let mut basic = vec![0,0,20,0,1,0,0,0];
            basic.extend_from_slice(&world);
            assert!(reject_broad_directory_ace(&basic).is_err());
            basic[0] = 1;
            assert!(reject_broad_directory_ace(&basic).is_ok());
            for corrupted in [
                { let mut x=basic.clone(); x[2]=19; x },
                { let mut x=basic.clone(); x[8]=2; x },
                { let mut x=basic.clone(); x[9]=16; x },
                { let mut x=basic.clone(); x.truncate(15); x },
            ] {
                assert_eq!(
                    reject_broad_directory_ace(&corrupted)
                        .unwrap_err().kind(),
                    io::ErrorKind::InvalidData,
                );
            }
        }

        #[test]
        fn ace_sid_correlation_never_treats_deny_or_foreign_sid_as_allow() {
            fn ace(kind: u8, mask: u32, sid: &[u8]) -> Vec<u8> {
                let mut data = vec![kind, 0, 0, 0];
                data[2..4].copy_from_slice(&((sid.len() + 8) as u16).to_le_bytes());
                data.extend_from_slice(&mask.to_le_bytes());
                data.extend_from_slice(sid);
                data
            }
            let first = &[1,2,0,0,0,0,0,15,2,0,0,0,3,0,0,0];
            let second = &[1,2,0,0,0,0,0,15,2,0,0,0,4,0,0,0];
            assert!(basic_allow_for_sid(&ace(0, 1, first), first));
            assert!(!basic_allow_for_sid(&ace(0, 1, first), second));
            assert!(!basic_allow_for_sid(&ace(1, 1, first), first));
            assert!(!basic_allow_for_sid(&ace(0, 0, first), first));
            // All of these are only syntactic claims; no effective access.
        }

        #[test]
        fn owned_disposable_directory_handle_has_stable_file_identity() {
            use std::os::windows::fs::OpenOptionsExt;
            use windows_sys::Win32::Storage::FileSystem::{
                FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
            };
            // This is a future opt-in synthetic-only test fixture; the path
            // is always new and never a Minecraft profile or preexisting box.
            let root = std::env::temp_dir().join(format!(
                "mivubi-disposable-root-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
            ));
            std::fs::create_dir(&root).unwrap();
            struct OwnsDisposable(std::path::PathBuf);
            impl Drop for OwnsDisposable {
                fn drop(&mut self) { let _ = std::fs::remove_dir(&self.0); }
            }
            let _cleanup = OwnsDisposable(root.clone());
            let handle = std::fs::OpenOptions::new()
                .read(true)
                .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
                .open(&root).unwrap();
            let first = inspect_directory_handle(&handle).unwrap();
            let second = inspect_directory_handle(&handle).unwrap();
            assert_eq!(first, second);
            assert_ne!(first.file_index, 0);
            // The optional fixture is new/disposable; ancestor inspection
            // is expected only on supported local DOS paths.
            if opened_root_name(&handle).ok().is_some_and(|name|
                opened_dos_prefixes(&name).is_ok()
            ) {
                inspect_opened_ancestor_chain(&handle, first).unwrap();
                assert!(inspect_opened_ancestor_chain(&handle,
                    super::super::PrivateRootObjectIdentity {
                        volume: first.volume,
                        file_index: first.file_index.wrapping_add(1),
                    }).is_err());
            }
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

    #[cfg(windows)]
    #[test]
    fn pending_root_cannot_bind_host_zero_or_stale_process() {
        let valid = ProcessIdentity::checked(700, 133400000000000700).unwrap();
        let observed = ProcessObservation {
            identity: valid,
            containment: ProcessContainment::AppContainerTokenObserved,
            official_image_signature_observed: false,
            visible_window_observed: false,
        };
        for slot in [SessionSlot::Mc01, SessionSlot::Mc02] {
            for invalid in [
                ProcessIdentity { pid: 0, created_file_time: valid.created_file_time },
                ProcessIdentity { pid: valid.pid, created_file_time: 0 },
            ] {
                assert_eq!(
                    validate_private_root_observation(slot, invalid, observed, observed),
                    Err(PrivateRootObservationError::InvalidSlotOrProcess),
                );
            }
        }
        assert_eq!(
            validate_private_root_observation(SessionSlot::Mc01, valid, observed, observed),
            Err(PrivateRootObservationError::InvalidSlotOrProcess),
        );
        let stale = ProcessObservation {
            identity: ProcessIdentity::checked(valid.pid, valid.created_file_time + 1).unwrap(),
            ..observed
        };
        assert_eq!(
            validate_private_root_observation(SessionSlot::Mc02, valid, observed, stale),
            Err(PrivateRootObservationError::StaleProcess),
        );
    }

    #[cfg(windows)]
    #[test]
    fn pending_root_requires_appcontainer_and_stable_observation() {
        let valid = ProcessIdentity::checked(704, 133400000000000704).unwrap();
        let isolated = ProcessObservation {
            identity: valid,
            containment: ProcessContainment::AppContainerTokenObserved,
            official_image_signature_observed: false,
            visible_window_observed: false,
        };
        let normal = ProcessObservation {
            containment: ProcessContainment::NoAppContainerToken,
            ..isolated
        };
        assert_eq!(
            validate_private_root_observation(SessionSlot::Mc02, valid, normal, normal),
            Err(PrivateRootObservationError::NotAppContainer),
        );
        assert_eq!(
            validate_private_root_observation(SessionSlot::Mc03, valid, isolated, normal),
            Err(PrivateRootObservationError::StaleProcess),
        );
        assert!(validate_private_root_observation(
            SessionSlot::Mc04, valid, isolated, isolated
        ).is_ok());
        // A successful predicate is only one data-point, never ACL/owner proof.
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
    fn package_family_relation_is_diagnostic_not_a_process_count() {
        use ObservedPackageFamilyRelation as Relation;
        assert_eq!(
            classify_package_family_pair(None, None), Relation::BothUnpackaged
        );
        assert_eq!(
            classify_package_family_pair(Some("Fixture_abcd"), None),
            Relation::OnePackaged
        );
        assert_eq!(
            classify_package_family_pair(None, Some("Fixture_abcd")),
            Relation::OnePackaged
        );
        assert_eq!(
            classify_package_family_pair(
                Some("MinecraftFixture_abcd"), Some("minecraftfixture_ABCD"),
            ),
            Relation::SamePackageFamily
        );
        assert_eq!(
            classify_package_family_pair(
                Some("MinecraftFixture_abcd"), Some("DifferentFixture_abcd"),
            ),
            Relation::DifferentPackageFamilies
        );
    }

    #[test]
    fn package_family_pair_refuses_zero_duplicate_and_reused_process_ids() {
        let first = ProcessIdentity::checked(271, 133400000000000271).unwrap();
        let second = ProcessIdentity::checked(272, 133400000000000272).unwrap();
        for (left, right) in [
            (ProcessIdentity { pid: 0, ..first }, second),
            (first, ProcessIdentity { created_file_time: 0, ..second }),
            (first, first),
            (first, ProcessIdentity { pid: first.pid, ..second }),
        ] {
            assert_eq!(
                inspect_process_package_family_pair_source(left, right),
                Err(NativePortError::InvalidIdentity),
            );
        }
    }

    #[cfg(not(windows))]
    #[test]
    fn package_family_pair_requires_windows_native_process_handles() {
        let first = ProcessIdentity::checked(271, 133400000000000271).unwrap();
        let second = ProcessIdentity::checked(272, 133400000000000272).unwrap();
        assert_eq!(
            inspect_process_package_family_pair_source(first, second),
            Err(NativePortError::NotImplemented),
        );
    }

    #[test]
    fn package_form_probe_does_not_promote_unknown_errors_to_unpacked() {
        assert_eq!(classify_package_probe(15700, 0), Ok(None));
        assert_eq!(classify_package_probe(122, 2), Ok(Some(2)));
        assert_eq!(classify_package_probe(122, 1024), Ok(Some(1024)));
        for (status, chars) in [
            (0, 0), (5, 0), (122, 0), (122, 1), (122, 1025),
            (15701, 0),
        ] {
            assert_eq!(
                classify_package_probe(status, chars),
                Err(NativePortError::OperationFailed),
            );
        }
    }

    #[test]
    fn package_form_query_rejects_invalid_pid_filetime_before_windows_io() {
        for identity in [
            ProcessIdentity { pid: 0, created_file_time: 123 },
            ProcessIdentity { pid: 42, created_file_time: 0 },
        ] {
            assert_eq!(
                inspect_process_package_form_source(identity),
                Err(NativePortError::InvalidIdentity),
            );
        }
    }

    #[cfg(not(windows))]
    #[test]
    fn package_form_source_unavailable_outside_windows() {
        let identity = ProcessIdentity::checked(42, 123).unwrap();
        assert_eq!(
            inspect_process_package_form_source(identity),
            Err(NativePortError::NotImplemented),
        );
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
