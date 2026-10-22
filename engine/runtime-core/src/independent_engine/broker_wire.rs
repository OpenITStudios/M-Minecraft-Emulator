//! Versioned, fixed-size read-only diagnostic request framing for a future
//! independently owned Windows service. This is NOT service authentication.
//!
//! Every decoded PID, FILETIME, slot and correlation ID is attacker-controlled
//! wire data until the server checks the actual connected OS pipe endpoint,
//! process HANDLE, caller token, session and root permissions.
//!
//! There is deliberately NO launch, provision, file mutation, registry write,
//! credentials, process termination or "isolation ready" opcode. Merely
//! decoding a packet must never grant any authority or select a provider.

use super::{ProcessIdentity, SessionSlot};

pub const BROKER_PROBE_BYTES: usize = 32;
const MAGIC: [u8; 4] = *b"MIVB";
const SCHEMA: u16 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BrokerProbeKind {
    ObserveExactProcess,
    InspectPrivateRootAcl,
}

impl BrokerProbeKind {
    const fn code(self) -> u8 {
        match self {
            Self::ObserveExactProcess => 1,
            Self::InspectPrivateRootAcl => 2,
        }
    }

    fn from_code(code: u8) -> Result<Self, BrokerWireError> {
        match code {
            1 => Ok(Self::ObserveExactProcess),
            2 => Ok(Self::InspectPrivateRootAcl),
            _ => Err(BrokerWireError::UnknownOperation),
        }
    }
}

/// This structure describes a request from an UNTRUSTED caller. It contains
/// no verified process identity or security capability. A service must never
/// use this value directly as permission to open data or run a child process.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UntrustedBrokerProbe {
    pub kind: BrokerProbeKind,
    pub slot: SessionSlot,
    pub claimed_process: ProcessIdentity,
    /// Response correlation only. This is NOT an authentication nonce,
    /// anti-replay secret, session key, token or evidence of uniqueness.
    pub correlation_id: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BrokerWireError {
    IncorrectLength,
    IncorrectMagic,
    UnsupportedVersion,
    UnknownOperation,
    InvalidSlot,
    InvalidProcess,
    InvalidCorrelation,
    NonzeroReserved,
}

fn validated(probe: UntrustedBrokerProbe) -> Result<UntrustedBrokerProbe, BrokerWireError> {
    if probe.slot.is_host_native() {
        return Err(BrokerWireError::InvalidSlot);
    }
    if ProcessIdentity::checked(
        probe.claimed_process.pid, probe.claimed_process.created_file_time,
    ) != Some(probe.claimed_process) {
        return Err(BrokerWireError::InvalidProcess);
    }
    if probe.correlation_id == 0 {
        return Err(BrokerWireError::InvalidCorrelation);
    }
    Ok(probe)
}

fn slot_code(slot: SessionSlot) -> Result<u8, BrokerWireError> {
    match slot {
        SessionSlot::Mc01 => Err(BrokerWireError::InvalidSlot),
        SessionSlot::Mc02 => Ok(2),
        SessionSlot::Mc03 => Ok(3),
        SessionSlot::Mc04 => Ok(4),
    }
}

fn slot_from_code(code: u8) -> Result<SessionSlot, BrokerWireError> {
    match code {
        2 => Ok(SessionSlot::Mc02),
        3 => Ok(SessionSlot::Mc03),
        4 => Ok(SessionSlot::Mc04),
        _ => Err(BrokerWireError::InvalidSlot),
    }
}

/// Request ABI: magic[0..4], schema u16[4..6], probe u8[6], slot u8[7],
/// claimed PID u32[8..12], claimed creation FILETIME u64[12..20],
/// correlation u64[20..28], reserved zero[28..32], all little endian.
///
/// This encoder does NOT sign or authenticate anything. Future consumers
/// must pin peer identity through native Windows APIs after decoding.
pub fn encode_broker_probe(
    probe: UntrustedBrokerProbe,
) -> Result<[u8; BROKER_PROBE_BYTES], BrokerWireError> {
    let probe = validated(probe)?;
    let mut frame = [0u8; BROKER_PROBE_BYTES];
    frame[0..4].copy_from_slice(&MAGIC);
    frame[4..6].copy_from_slice(&SCHEMA.to_le_bytes());
    frame[6] = probe.kind.code();
    frame[7] = slot_code(probe.slot)?;
    frame[8..12].copy_from_slice(&probe.claimed_process.pid.to_le_bytes());
    frame[12..20].copy_from_slice(&probe.claimed_process.created_file_time.to_le_bytes());
    frame[20..28].copy_from_slice(&probe.correlation_id.to_le_bytes());
    Ok(frame)
}

pub fn decode_broker_probe(
    bytes: &[u8],
) -> Result<UntrustedBrokerProbe, BrokerWireError> {
    if bytes.len() != BROKER_PROBE_BYTES {
        return Err(BrokerWireError::IncorrectLength);
    }
    if &bytes[0..4] != MAGIC.as_slice() {
        return Err(BrokerWireError::IncorrectMagic);
    }
    if u16::from_le_bytes(bytes[4..6].try_into().expect("fixed-size wire")) != SCHEMA {
        return Err(BrokerWireError::UnsupportedVersion);
    }
    if bytes[28..32].iter().any(|value| *value != 0) {
        return Err(BrokerWireError::NonzeroReserved);
    }
    let kind = BrokerProbeKind::from_code(bytes[6])?;
    let slot = slot_from_code(bytes[7])?;
    let pid = u32::from_le_bytes(bytes[8..12].try_into().expect("fixed-size wire"));
    let created = u64::from_le_bytes(bytes[12..20].try_into().expect("fixed-size wire"));
    let correlation_id = u64::from_le_bytes(
        bytes[20..28].try_into().expect("fixed-size wire"),
    );
    validated(UntrustedBrokerProbe {
        kind,
        slot,
        claimed_process: ProcessIdentity { pid, created_file_time: created },
        correlation_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example() -> UntrustedBrokerProbe {
        UntrustedBrokerProbe {
            kind: BrokerProbeKind::InspectPrivateRootAcl,
            slot: SessionSlot::Mc03,
            claimed_process: ProcessIdentity::checked(
                0x1234_5678, 0x1122_3344_5566_7788,
            ).unwrap(),
            correlation_id: 0x1020_3040_5060_7080,
        }
    }

    #[test]
    fn fixed_wire_encoding_round_trips_without_truncating_filetime() {
        let request = example();
        let frame = encode_broker_probe(request).unwrap();
        assert_eq!(&frame[0..4], b"MIVB");
        assert_eq!(&frame[4..8], &[1, 0, 2, 3]);
        assert_eq!(&frame[8..12], &0x1234_5678u32.to_le_bytes());
        assert_eq!(&frame[12..20], &0x1122_3344_5566_7788u64.to_le_bytes());
        assert_eq!(&frame[20..28], &0x1020_3040_5060_7080u64.to_le_bytes());
        assert!(frame[28..].iter().all(|value| *value == 0));
        assert_eq!(decode_broker_probe(&frame), Ok(request));

        let process_probe = UntrustedBrokerProbe {
            kind: BrokerProbeKind::ObserveExactProcess,
            slot: SessionSlot::Mc02,
            correlation_id: 1,
            ..request
        };
        assert_eq!(
            decode_broker_probe(&encode_broker_probe(process_probe).unwrap()),
            Ok(process_probe),
        );
    }

    #[test]
    fn malformed_frames_and_authority_like_opcodes_fail_closed() {
        let good = encode_broker_probe(example()).unwrap();
        assert_eq!(decode_broker_probe(&good[..31]), Err(BrokerWireError::IncorrectLength));
        assert_eq!(decode_broker_probe(&[0u8; 64]), Err(BrokerWireError::IncorrectLength));

        for (offset, value, expected) in [
            (0, 0, BrokerWireError::IncorrectMagic),
            (4, 2, BrokerWireError::UnsupportedVersion),
            (6, 3, BrokerWireError::UnknownOperation),
            (6, 255, BrokerWireError::UnknownOperation),
            (7, 1, BrokerWireError::InvalidSlot),
            (7, 5, BrokerWireError::InvalidSlot),
            (28, 1, BrokerWireError::NonzeroReserved),
            (31, 255, BrokerWireError::NonzeroReserved),
        ] {
            let mut corrupted = good;
            corrupted[offset] = value;
            assert_eq!(decode_broker_probe(&corrupted), Err(expected));
        }
        for range in [8..12, 12..20, 20..28] {
            let mut corrupted = good;
            corrupted[range].fill(0);
            let error = decode_broker_probe(&corrupted).unwrap_err();
            assert!(matches!(
                error,
                BrokerWireError::InvalidProcess | BrokerWireError::InvalidCorrelation
            ));
        }
        let mut host = example();
        host.slot = SessionSlot::Mc01;
        assert_eq!(encode_broker_probe(host), Err(BrokerWireError::InvalidSlot));
        host = example();
        host.claimed_process.pid = 0;
        assert_eq!(encode_broker_probe(host), Err(BrokerWireError::InvalidProcess));
    }
}
