//! Strict decoder for an EXISTING synthetic fixture observation record.
//!
//! This is NOT a new Windows transport, not authenticated IPC, not a
//! ProcessPathSnapshot and never launch authority. The Windows C++ fixture
//! writes its OWN 8-byte Observation{DWORD version, DWORD flags} into its
//! newly created AppContainer profile. The owning parent pins each child
//! process and checks its token separately. Decoding file bytes alone does
//! not establish which process wrote them or bind them to PID/FILETIME.

use super::PathLocation;

pub const SYNTHETIC_FRAME_BYTES: usize = 8;
const FORMAT_VERSION: u32 = 1;

// These bits mirror the independent C++ synthetic fixture. Do not import
// Sandboxie driver flags, proprietary ABIs or upstream sandbox sources.
const LOCAL_KNOWN_FOLDER: u32 = 1 << 5;
const ROAMING_ENV_PRIVATE: u32 = 1 << 6;
const ROAMING_KNOWN_AVAILABLE: u32 = 1 << 7;
const ROAMING_KNOWN_PRIVATE: u32 = 1 << 8;
const CUSTOM_ROAMING_CREATED: u32 = 1 << 11;
const CUSTOM_ROAMING_REOPENED: u32 = 1 << 12;
const ROAMING_ENV_AVAILABLE: u32 = 1 << 13;
const KNOWN_FLAGS: u32 = (1 << 14) - 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SyntheticFrameError {
    InvalidLength,
    UnsupportedVersion,
    UnsupportedFlags,
    ContradictoryFlags,
    InconsistentPhase,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SyntheticFixturePhase {
    FirstProcess,
    ReopenedProcess,
}

/// Unbound and self-reported PATH OBSERVATIONS, not live process identity,
/// file-access validation, registry, MSAL/WAM or Education account evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnboundFixturePaths {
    pub local_known_folder: PathLocation,
    pub roaming_environment: PathLocation,
    pub roaming_known_folder: PathLocation,
    pub phase: SyntheticFixturePhase,
}

fn location(available: bool, under_expected: bool) -> PathLocation {
    if !available {
        PathLocation::Unavailable
    } else if under_expected {
        PathLocation::UnderExpectedProfile
    } else {
        PathLocation::OutsideExpectedProfile
    }
}

/// Parse exactly one native Windows fixture observation. A strict size
/// protects readers from oversized or truncated untrusted fixture input.
/// The caller must enforce bounded I/O and independently verify process
/// creation identity and record provenance; this decoder never does either.
pub fn decode_synthetic_fixture_paths(
    bytes: &[u8],
) -> Result<UnboundFixturePaths, SyntheticFrameError> {
    if bytes.len() != SYNTHETIC_FRAME_BYTES {
        return Err(SyntheticFrameError::InvalidLength);
    }
    // DWORD in supported Windows x64 fixture builds is unsigned 32-bit LE.
    let version = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let flags = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    if version != FORMAT_VERSION {
        return Err(SyntheticFrameError::UnsupportedVersion);
    }
    if flags & !KNOWN_FLAGS != 0 {
        return Err(SyntheticFrameError::UnsupportedFlags);
    }
    let has = |flag| flags & flag != 0;
    if has(ROAMING_ENV_PRIVATE) && !has(ROAMING_ENV_AVAILABLE)
        || has(ROAMING_KNOWN_PRIVATE) && !has(ROAMING_KNOWN_AVAILABLE)
        || has(CUSTOM_ROAMING_CREATED) == has(CUSTOM_ROAMING_REOPENED)
    {
        return Err(SyntheticFrameError::ContradictoryFlags);
    }
    let phase = if has(CUSTOM_ROAMING_CREATED) {
        SyntheticFixturePhase::FirstProcess
    } else {
        SyntheticFixturePhase::ReopenedProcess
    };
    Ok(UnboundFixturePaths {
        // The current native fixture fails before writing its record when
        // SHGetKnownFolderPath(LocalAppData) fails, so this query is available.
        local_known_folder: location(true, has(LOCAL_KNOWN_FOLDER)),
        roaming_environment: location(
            has(ROAMING_ENV_AVAILABLE),
            has(ROAMING_ENV_PRIVATE),
        ),
        roaming_known_folder: location(
            has(ROAMING_KNOWN_AVAILABLE),
            has(ROAMING_KNOWN_PRIVATE),
        ),
        phase,
    })
}

/// Decode both independent fixture children, refusing a phase mismatch.
/// The identities A/B remain external *labels*, never token/PID attestation.
pub fn decode_synthetic_fixture_pair(
    first: &[u8],
    second: &[u8],
) -> Result<[UnboundFixturePaths; 2], SyntheticFrameError> {
    let a = decode_synthetic_fixture_paths(first)?;
    let b = decode_synthetic_fixture_paths(second)?;
    if a.phase != b.phase {
        return Err(SyntheticFrameError::InconsistentPhase);
    }
    Ok([a, b])
}

#[cfg(test)]
mod tests {
    use super::*;

    const OWN_FILE: u32 = 1;
    const PEER_DENIED: u32 = 2;
    const HOST_DENIED: u32 = 4;
    const MUTEX_PRIVATE: u32 = 8;
    const LOCAL_ENV: u32 = 16;
    const SYNTHETIC_OWN: u32 = 512;
    const SYNTHETIC_PEER_DENIED: u32 = 1024;
    const FIRST: u32 = CUSTOM_ROAMING_CREATED;
    const SECOND: u32 = CUSTOM_ROAMING_REOPENED;

    fn frame(flags: u32) -> [u8; 8] {
        let mut bytes = [0u8; SYNTHETIC_FRAME_BYTES];
        bytes[..4].copy_from_slice(&FORMAT_VERSION.to_le_bytes());
        bytes[4..].copy_from_slice(&flags.to_le_bytes());
        bytes
    }

    fn required(phase: u32) -> u32 {
        OWN_FILE | PEER_DENIED | HOST_DENIED | MUTEX_PRIVATE
            | SYNTHETIC_OWN | SYNTHETIC_PEER_DENIED | phase
    }

    #[test]
    fn native_fixture_record_maps_real_flags_to_three_path_only_states() {
        let initial = frame(required(FIRST) | LOCAL_ENV | LOCAL_KNOWN_FOLDER
            | ROAMING_ENV_AVAILABLE | ROAMING_KNOWN_AVAILABLE);
        let result = decode_synthetic_fixture_paths(&initial).unwrap();
        assert_eq!(result.local_known_folder, PathLocation::UnderExpectedProfile);
        assert_eq!(result.roaming_environment, PathLocation::OutsideExpectedProfile);
        assert_eq!(result.roaming_known_folder, PathLocation::OutsideExpectedProfile);
        assert_eq!(result.phase, SyntheticFixturePhase::FirstProcess);
    }

    #[test]
    fn all_private_looking_paths_remain_anonymous_not_process_attestation() {
        let first = frame(required(FIRST) | LOCAL_KNOWN_FOLDER
            | ROAMING_ENV_AVAILABLE | ROAMING_ENV_PRIVATE
            | ROAMING_KNOWN_AVAILABLE | ROAMING_KNOWN_PRIVATE);
        let result = decode_synthetic_fixture_paths(&first).unwrap();
        assert_eq!(result.local_known_folder, PathLocation::UnderExpectedProfile);
        assert_eq!(result.roaming_environment, PathLocation::UnderExpectedProfile);
        assert_eq!(result.roaming_known_folder, PathLocation::UnderExpectedProfile);
        // Deliberately no process identity, signed-game, account or launch fields.
    }

    #[test]
    fn missing_optional_queries_report_unknown_without_hiding_other_paths() {
        let raw = frame(required(FIRST));
        let result = decode_synthetic_fixture_paths(&raw).unwrap();
        assert_eq!(result.local_known_folder, PathLocation::OutsideExpectedProfile);
        assert_eq!(result.roaming_environment, PathLocation::Unavailable);
        assert_eq!(result.roaming_known_folder, PathLocation::Unavailable);
    }

    #[test]
    fn exact_eight_byte_record_is_required() {
        let raw = frame(required(FIRST));
        assert_eq!(
            decode_synthetic_fixture_paths(&raw[..7]),
            Err(SyntheticFrameError::InvalidLength)
        );
        let mut extended = raw.to_vec();
        extended.push(0);
        assert_eq!(
            decode_synthetic_fixture_paths(&extended),
            Err(SyntheticFrameError::InvalidLength)
        );
        assert_eq!(
            decode_synthetic_fixture_paths(&[]),
            Err(SyntheticFrameError::InvalidLength)
        );
    }

    #[test]
    fn unknown_native_version_and_reserved_flags_fail_closed() {
        let mut raw = frame(required(FIRST));
        raw[0] = 2;
        assert_eq!(
            decode_synthetic_fixture_paths(&raw),
            Err(SyntheticFrameError::UnsupportedVersion)
        );
        assert_eq!(
            decode_synthetic_fixture_paths(&frame(required(FIRST) | (1 << 14))),
            Err(SyntheticFrameError::UnsupportedFlags)
        );
    }

    #[test]
    fn mutually_inconsistent_flags_are_rejected() {
        for flags in [
            required(FIRST) | ROAMING_ENV_PRIVATE,
            required(FIRST) | ROAMING_KNOWN_PRIVATE,
            required(FIRST) | SECOND,
            required(0),
        ] {
            assert_eq!(
                decode_synthetic_fixture_paths(&frame(flags)),
                Err(SyntheticFrameError::ContradictoryFlags)
            );
        }
    }

    #[test]
    fn independent_children_must_come_from_the_same_fixture_phase() {
        let a = frame(required(FIRST) | LOCAL_KNOWN_FOLDER);
        let b = frame(required(FIRST) | ROAMING_ENV_AVAILABLE);
        let pair = decode_synthetic_fixture_pair(&a, &b).unwrap();
        assert_eq!(pair[0].local_known_folder, PathLocation::UnderExpectedProfile);
        assert_eq!(pair[1].local_known_folder, PathLocation::OutsideExpectedProfile);
        assert_eq!(pair[1].roaming_environment, PathLocation::OutsideExpectedProfile);

        let reopened = frame(required(SECOND));
        assert_eq!(
            decode_synthetic_fixture_pair(&a, &reopened),
            Err(SyntheticFrameError::InconsistentPhase)
        );
        assert_eq!(
            decode_synthetic_fixture_paths(&reopened).unwrap().phase,
            SyntheticFixturePhase::ReopenedProcess
        );
    }
}
