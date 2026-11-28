//! Reviewed legacy profile replay capability; historical profile preparation is retired.
//! Existing identity checks remain for preserved stores and migration.
use super::{
    directory_id, directory_path, invalid, known_folder, open_directory,
    open_existing_registry_store, open_private_temporary, pin_image, profile_binding, same_image,
    user_registry_path, PrivateTemporaryMode, SlotStorage,
};
use std::{ffi::OsStr, fs::File, io, path::{Path, PathBuf}};
use windows_sys::Win32::{
    Storage::FileSystem::{FILE_LIST_DIRECTORY, FILE_READ_ATTRIBUTES},
    UI::Shell::{FOLDERID_LocalAppData, FOLDERID_Profile, FOLDERID_RoamingAppData},
};

// Existing pair remains immutable. Five-slot replay requires its own reviewed identity capability.
#[cfg(test)]
pub(super) const REVIEWED_FIVE_REPLAY: Option<[u8; 32]> = None;
#[cfg(not(test))]
pub(super) const REVIEWED_FIVE_REPLAY: Option<[u8; 32]> = Some([
    0xaa, 0x04, 0xa0, 0x71, 0x52, 0xfb, 0x1b, 0x5f, 0x43, 0xdd, 0xcc, 0xe4, 0x49, 0x8b, 0x3c, 0xa9,
    0xb7, 0x84, 0x9d, 0x7e, 0x79, 0xb4, 0x0d, 0x7a, 0x42, 0x8c, 0x18, 0xbd, 0x1d, 0xd0, 0xfa, 0x0b,
]);
// One reviewed acceptance pair only; no profile discovery, migration or token/world copying.
#[cfg(test)]
pub(super) const REVIEWED_REPLAY: Option<[u8; 32]> = None;
#[cfg(not(test))]
pub(super) const REVIEWED_REPLAY: Option<[u8; 32]> = Some([
    0x41, 0xdb, 0xc7, 0xdb, 0x44, 0xd2, 0x91, 0xcd, 0x5e, 0xb9, 0xc7, 0xfc, 0x60, 0x90, 0x54, 0xa8,
    0xbb, 0x0c, 0x76, 0x37, 0xc1, 0x04, 0xfa, 0x85, 0xe8, 0xdc, 0x42, 0x70, 0xa9, 0xa4, 0xa8, 0xaf,
]);

#[cfg(test)]
pub(super) fn reopen_slots(
    logical: File,
    parent: &Path,
    bytes: &[u8],
    expected: [u8; 32],
) -> io::Result<[SlotStorage; 2]> {
    reopen_slots_for::<2>(logical, parent, bytes, expected)
}
pub(super) fn reopen_slots_for<const N: usize>(
    logical: File,
    parent: &Path,
    bytes: &[u8],
    expected: [u8; 32],
) -> io::Result<[SlotStorage; N]> {
    use sha2::{Digest, Sha256};
    if N != 2 && N != 5 {
        return Err(invalid("Unsupported replay slot count"));
    }
    let size = 168 + N * 24;
    if bytes.len() != size || <[u8; 32]>::from(Sha256::digest(bytes)) != expected {
        return Err(invalid("Unreviewed replay profile capability"));
    }
    let word = |at| u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
    if word(0) != 0x52504c31
        || word(4) != (if N == 2 { 1 } else { 2 })
        || word(8) != size as u32
        || word(12) != 0
    {
        return Err(invalid("Invalid replay profile capability"));
    }
    // The reviewed wire format has exactly 64 UTF-16 units.
    let units: [u16; 64] = std::array::from_fn(|index| {
        let at = 16 + index * 2;
        u16::from_le_bytes([bytes[at], bytes[at + 1]])
    });
    let end = units
        .iter()
        .position(|u| *u == 0)
        .ok_or_else(|| invalid("Unterminated replay name"))?;
    let name = String::from_utf16(&units[..end]).map_err(|_| invalid("Invalid replay name"))?;
    let suffix = name
        .strip_prefix("MivubiEducationPilot_")
        .ok_or_else(|| invalid("Invalid replay root"))?;
    if suffix.len() != 32
        || !suffix.bytes().all(|c| c.is_ascii_hexdigit())
        || units[end..].iter().any(|u| *u != 0)
    {
        return Err(invalid("Invalid replay root"));
    }
    let root = parent.join(name);
    let mut pins = Vec::new();
    for path in root.ancestors().collect::<Vec<_>>().into_iter().rev() {
        pins.push(open_directory(path, FILE_READ_ATTRIBUTES).map_err(|error|io::Error::new(error.kind(),format!("Saved launcher root is unavailable: {error}")))?);
    }
    let matches = |file: &File, at: usize| -> io::Result<bool> {
        let id = directory_id(file)?;
        Ok(
            id.VolumeSerialNumber == u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap())
                && id.FileId.Identifier == bytes[at + 8..at + 24],
        )
    };
    if !matches(pins.last().unwrap(), 144)? {
        return Err(invalid("Replay root was replaced"));
    }
    let logical_path = directory_path(&logical)?
        .to_string_lossy()
        .to_ascii_lowercase();
    let root_path = directory_path(pins.last().unwrap())?
        .to_string_lossy()
        .to_ascii_lowercase();
    if Path::new(&logical_path).starts_with(&root_path)
        || Path::new(&root_path).starts_with(&logical_path)
    {
        return Err(invalid("Overlapping replay AppData roots"));
    }
    let user_key_path = user_registry_path()?;
    let mut slots = Vec::with_capacity(N);
    for slot in 1..=N as u32 {
        let private = open_directory(
            &root.join(format!("MC{slot:02}")),
            FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY,
        ).map_err(|error|io::Error::new(error.kind(),format!("Minecraft - {slot}: profile folder is unavailable: {error}")))?;
        if !matches(&private, 168 + (slot as usize - 1) * 24)? {
            return Err(invalid("Replay slot was replaced"));
        }
        let private_path = directory_path(&private)?;
        // Open existing directories and registry only. A missing store never silently resets login.
        for leaf in ["Roaming", "Local"] {
            pins.push(open_directory(
                &private_path.join(leaf),
                FILE_READ_ATTRIBUTES,
            )?);
        }
        let registry = open_existing_registry_store(&directory_id(&private)?,slot)?;
        slots.push(SlotStorage {
            slot,
            logical: logical.try_clone()?,
            temporary: open_private_temporary(&private, PrivateTemporaryMode::ExistingOnly).map_err(|error|io::Error::new(error.kind(),format!("Minecraft - {slot}: temporary folder is unavailable: {error}")))?,
            private,
            registry,
            user_key_path,
            private_path,
            singleton_caller_rva: 0,
        });
    }
    slots.try_into().map_err(|_| invalid("Missing replay slot"))
}

#[cfg(test)]
pub(super) fn five_replay_bytes(pair: &[u8], stores: &[SlotStorage]) -> io::Result<Vec<u8>> {
    if pair.len() != 216 || stores.len() != 5 {
        return Err(invalid("Incomplete five-slot identity set"));
    }
    let mut bytes = vec![0u8; 288];
    bytes[..216].copy_from_slice(pair);
    bytes[4..8].copy_from_slice(&2u32.to_le_bytes());
    bytes[8..12].copy_from_slice(&288u32.to_le_bytes());
    for (index, store) in stores.iter().enumerate() {
        if store.slot != index as u32 + 1 {
            return Err(invalid("Unordered five-slot identity set"));
        }
        let id = directory_id(&store.private)?;
        let at = 168 + index * 24;
        if index < 2 {
            if pair[at..at + 8] != id.VolumeSerialNumber.to_le_bytes()
                || pair[at + 8..at + 24] != id.FileId.Identifier
            {
                return Err(invalid("Original pair identity changed"));
            }
        }
        bytes[at..at + 8].copy_from_slice(&id.VolumeSerialNumber.to_le_bytes());
        bytes[at + 8..at + 24].copy_from_slice(&id.FileId.Identifier);
    }
    Ok(bytes)
}

pub(super) fn replay_roots() -> io::Result<(File, PathBuf)> {
    let roaming = known_folder(&FOLDERID_RoamingAppData)?;
    let local = known_folder(&FOLDERID_LocalAppData)?;
    if roaming.file_name() != Some(OsStr::new("Roaming"))
        || local.file_name() != Some(OsStr::new("Local"))
    {
        return Err(invalid(
            "Redirected AppData layouts require explicit mediation",
        ));
    }
    let a = open_directory(
        roaming
            .parent()
            .ok_or_else(|| invalid("Missing AppData parent"))?,
        FILE_READ_ATTRIBUTES,
    )?;
    let b = open_directory(
        local
            .parent()
            .ok_or_else(|| invalid("Missing AppData parent"))?,
        FILE_READ_ATTRIBUTES,
    )?;
    if !same_image(&directory_id(&a)?, &directory_id(&b)?) {
        return Err(invalid(
            "Split Roaming/Local roots are not supported by this DLL configuration",
        ));
    }
    let parent = known_folder(&FOLDERID_Profile)?;
    Ok((a, parent))
}

// The caller may already have established product catalog absence in the
// same observation. Do not reopen that catalog on the legacy fallback.
pub(super) fn registered_replay_slots<const N: usize>() -> io::Result<[SlotStorage; N]> {
    if N != 2 && N != 5 {
        return Err(invalid("Unsupported replay slot count"));
    }
    // Require the reviewed replay before touching Known Folders or storage.
    let expected = (if N == 2 { REVIEWED_REPLAY } else { REVIEWED_FIVE_REPLAY })
        .ok_or_else(|| io::Error::new(
            io::ErrorKind::Unsupported,
            "No reviewed existing Education profile pair",
        ))?;
    let (a, parent) = replay_roots()?;
    use std::io::Read;
    let binding = if N == 5 && cfg!(feature = "lazy-tester-package") {
        Some(profile_binding::binding_path(false)?)
    } else { None };
    let path = if let Some((path, _pins)) = &binding {
        path.clone()
    } else { std::env::current_exe()?
        .parent()
        .ok_or_else(|| invalid("Missing replay payload root"))?
        .join(if N == 2 {
            "independent-bootstrap/replay-profile-pair.bin"
        } else {
            "independent-bootstrap/replay-profile-five.bin"
        }) };
    let mut file = pin_image(&path)?;
    let size = 168 + N * 24;
    if file.metadata()?.len() != size as u64 {
        return Err(invalid("Invalid replay capability size"));
    }
    let mut bytes = vec![0u8; size];
    file.read_exact(&mut bytes)?;
    reopen_slots_for::<N>(a, &parent, &bytes, expected)
}
