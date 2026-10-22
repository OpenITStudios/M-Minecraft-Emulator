//! Independent in-memory copy-on-write namespace for synthetic file and
//! registry-value resources. No filesystem, HKCU, Win32 hooks or game writes.
//!
//! A base snapshot is explicitly test-supplied and immutable. Each MC02-04
//! session owns a separate overlay. Read merges private entries over the
//! snapshot; deletion leaves tombstones that mask the snapshot; mutations
//! never change base bytes. This is executable COW *domain logic*, not a
//! Windows interception layer or Minecraft-compatible pathname adapter.
//!
//! Names are deliberately ASCII-only and bounded for synthetic fixtures.
//! Arbitrary real-world Minecraft Unicode filenames, registry key metadata,
//! directory operations, alternate streams and Windows link behavior require
//! a separate validated native implementation before any production use.

use std::collections::{BTreeMap, BTreeSet};

use super::{ProcessIdentity, SessionSlot};

pub const MAX_OVERLAY_ITEM_BYTES: usize = 64 * 1024;
pub const MAX_OVERLAY_ITEMS: usize = 1024;
pub const MAX_OVERLAY_PRIVATE_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum OverlaySurface {
    File,
    RegistryValue,
}

impl OverlaySurface {
    fn separator(self) -> char {
        match self {
            Self::File => '/',
            Self::RegistryValue => '\\',
        }
    }
}

/// A resource inside a *synthetic* namespace, never an OS-qualified path.
/// Case folding here covers ASCII fixture names only, not full Windows
/// Unicode case mappings. These keys MUST NOT be passed to native file APIs.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct OverlayKey {
    surface: OverlaySurface,
    name: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OverlayError {
    HostSlotForbidden,
    InvalidProcess,
    WrongOwnerOrProcess,
    InvalidName,
    EntryTooLarge,
    CapacityExceeded,
    AlreadyExists,
    MissingSource,
    AmbiguousPrivateProvenance,
    HostCopyIntoPrivateBoundary,
}

impl OverlayKey {
    pub fn parse(surface: OverlaySurface, input: &str) -> Result<Self, OverlayError> {
        if input.is_empty() || input.len() > 240 || !input.is_ascii() {
            return Err(OverlayError::InvalidName);
        }
        let separator = surface.separator();
        let parts: Vec<_> = input.split(separator).collect();
        if parts.len() > 32 || parts.iter().any(|component| !valid_component(component)) {
            return Err(OverlayError::InvalidName);
        }
        Ok(Self { surface, name: input.to_ascii_lowercase() })
    }

    pub fn surface(&self) -> OverlaySurface {
        self.surface
    }

    pub fn normalized_name(&self) -> &str {
        &self.name
    }

    fn at_or_below(&self, prefix: &Self) -> bool {
        if self.surface != prefix.surface {
            return false;
        }
        self.name == prefix.name
            || self.name.strip_prefix(&prefix.name).is_some_and(|rest| {
                rest.starts_with(self.surface.separator())
            })
    }
}

fn valid_component(value: &str) -> bool {
    if value.is_empty() || value.len() > 64 || value == "." || value == ".."
        || value.ends_with('.') || value.ends_with(' ')
        || value.as_bytes().iter().any(|c| !c.is_ascii_alphanumeric()
            && !matches!(*c, b'-' | b'_' | b'.' | b' ' | b'$'))
    {
        return false;
    }
    // Treat dangerous DOS device basenames as invalid even with extensions.
    // No code here opens Windows filenames, but accepting these would make
    // later adapters ambiguous.
    let stem = value.split('.').next().unwrap_or("");
    let stem = stem.to_ascii_uppercase();
    !matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL"
        | "CONIN$" | "CONOUT$")
        && !(stem.len() == 4 && (
            (stem.starts_with("COM") || stem.starts_with("LPT"))
                && stem.as_bytes()[3].is_ascii_digit()
        ))
}

/// Immutable test-supplied host view: there is no path or OS handle to
/// the actual desktop account, filesystem, registry or authentication broker.
#[derive(Clone, Debug, Default)]
pub struct SyntheticBaseView {
    entries: BTreeMap<OverlayKey, Vec<u8>>,
    bytes: usize,
}

impl SyntheticBaseView {
    pub fn insert_fixture(
        &mut self,
        key: OverlayKey,
        data: &[u8],
    ) -> Result<(), OverlayError> {
        if data.len() > MAX_OVERLAY_ITEM_BYTES {
            return Err(OverlayError::EntryTooLarge);
        }
        if self.entries.contains_key(&key) {
            return Err(OverlayError::AlreadyExists);
        }
        if self.entries.len() == MAX_OVERLAY_ITEMS
            || self.bytes + data.len() > MAX_OVERLAY_PRIVATE_BYTES
        {
            return Err(OverlayError::CapacityExceeded);
        }
        self.bytes += data.len();
        self.entries.insert(key, data.to_vec());
        Ok(())
    }

    pub fn fixture_bytes(&self, key: &OverlayKey) -> Option<&[u8]> {
        self.entries.get(key).map(Vec::as_slice)
    }
}

/// A bound, in-memory overlay; there is no process-identity trust claim.
/// The caller must re-pin a real PID/FILETIME before using any future OS
/// adapter. A slot/proc mismatch fails before reading or altering state.
#[derive(Clone, Debug)]
pub struct SyntheticSlotOverlay {
    slot: SessionSlot,
    process: ProcessIdentity,
    private: BTreeMap<OverlayKey, Vec<u8>>,
    // An explicit host copy-up or partial update of host-derived bytes
    // cannot later be disguised as newly authored data through rename.
    host_derived: BTreeSet<OverlayKey>,
    tombstones: BTreeSet<OverlayKey>,
    // Explicit read boundary: fixture host values under these prefixes may
    // NEVER fall back, even when not deleted. Private values still override.
    private_only_prefixes: BTreeSet<OverlayKey>,
    private_bytes: usize,
}

impl SyntheticSlotOverlay {
    pub fn new(slot: SessionSlot, process: ProcessIdentity) -> Result<Self, OverlayError> {
        if slot.is_host_native() {
            return Err(OverlayError::HostSlotForbidden);
        }
        if process.pid == 0 || process.created_file_time == 0 {
            return Err(OverlayError::InvalidProcess);
        }
        Ok(Self {
            slot,
            process,
            private: BTreeMap::new(),
            host_derived: BTreeSet::new(),
            tombstones: BTreeSet::new(),
            private_only_prefixes: BTreeSet::new(),
            private_bytes: 0,
        })
    }

    fn check(&self, slot: SessionSlot, process: ProcessIdentity) -> Result<(), OverlayError> {
        if slot != self.slot || process != self.process {
            return Err(OverlayError::WrongOwnerOrProcess);
        }
        Ok(())
    }

    fn masked(&self, key: &OverlayKey) -> bool {
        self.tombstones.iter().any(|deleted| key.at_or_below(deleted))
    }

    fn host_fallback_denied(&self, key: &OverlayKey) -> bool {
        self.private_only_prefixes.iter().any(|prefix| key.at_or_below(prefix))
    }

    /// Require all reads under a synthetic directory/registry prefix to
    /// originate from this slot's PRIVATE overlay. This is a diagnostic
    /// policy method, never a filesystem ACL or Windows hook.
    pub fn make_private_only(
        &mut self,
        slot: SessionSlot,
        process: ProcessIdentity,
        prefix: OverlayKey,
    ) -> Result<(), OverlayError> {
        self.check(slot, process)?;
        if self.private_only_prefixes.contains(&prefix) {
            return Ok(());
        }
        // A rule installed after private bytes exist cannot establish
        // whether those bytes were originally copied from the base view.
        // Synthetic data has no provenance receipt; fail closed rather
        // than misrepresenting the resulting subtree as private-only.
        if self.private.keys().any(|key| key.at_or_below(&prefix)) {
            return Err(OverlayError::AmbiguousPrivateProvenance);
        }
        if self.private_only_prefixes.len() == MAX_OVERLAY_ITEMS {
            return Err(OverlayError::CapacityExceeded);
        }
        self.private_only_prefixes.insert(prefix);
        Ok(())
    }

    /// Private value first, then optionally visible fixture base; never
    /// reads the real Windows host OS, regardless of the configured prefix.
    pub fn read<'a>(
        &'a self,
        host: &'a SyntheticBaseView,
        slot: SessionSlot,
        process: ProcessIdentity,
        key: &OverlayKey,
    ) -> Result<Option<&'a [u8]>, OverlayError> {
        self.check(slot, process)?;
        if let Some(value) = self.private.get(key) {
            return Ok(Some(value.as_slice()));
        }
        if self.masked(key) || self.host_fallback_denied(key) {
            return Ok(None);
        }
        Ok(host.fixture_bytes(key))
    }

    /// A full replacement is always made in private memory, even when the
    /// immutable host fixture contains the same name. No host write exists.
    pub fn write(
        &mut self,
        slot: SessionSlot,
        process: ProcessIdentity,
        key: OverlayKey,
        data: &[u8],
    ) -> Result<(), OverlayError> {
        self.check(slot, process)?;
        if data.len() > MAX_OVERLAY_ITEM_BYTES {
            return Err(OverlayError::EntryTooLarge);
        }
        let previous = self.private.get(&key).map_or(0, Vec::len);
        if !self.private.contains_key(&key) && self.private.len() == MAX_OVERLAY_ITEMS {
            return Err(OverlayError::CapacityExceeded);
        }
        let total = self.private_bytes - previous + data.len();
        if total > MAX_OVERLAY_PRIVATE_BYTES {
            return Err(OverlayError::CapacityExceeded);
        }
        self.private_bytes = total;
        // A full replacement has newly authored content, not a copy-up.
        self.host_derived.remove(&key);
        // A write after a subtree delete must NOT remove its prefix
        // tombstone: that would resurrect unrelated host descendants.
        // Reads always prefer private bytes over matching tombstones.
        self.private.insert(key, data.to_vec());
        Ok(())
    }

    /// First private read of a host value can be copied up explicitly before
    /// a partial write. A tombstone deliberately suppresses copy-up.
    pub fn copy_up(
        &mut self,
        host: &SyntheticBaseView,
        slot: SessionSlot,
        process: ProcessIdentity,
        key: &OverlayKey,
    ) -> Result<(), OverlayError> {
        self.check(slot, process)?;
        if self.private.contains_key(key) {
            return Ok(());
        }
        let bytes = self.read(host, slot, process, key)?
            .ok_or(OverlayError::MissingSource)?
            .to_vec();
        self.write(slot, process, key.clone(), &bytes)?;
        self.host_derived.insert(key.clone());
        Ok(())
    }

    /// A bounded partial file write first materializes the fixture base
    /// into private memory. Offsets are byte offsets; holes are zero-filled
    /// in the PRIVATE copy only. Registry values have no offset operation.
    pub fn write_at(
        &mut self,
        host: &SyntheticBaseView,
        slot: SessionSlot,
        process: ProcessIdentity,
        key: OverlayKey,
        offset: usize,
        data: &[u8],
    ) -> Result<(), OverlayError> {
        self.check(slot, process)?;
        if key.surface != OverlaySurface::File {
            return Err(OverlayError::InvalidName);
        }
        let length = offset.checked_add(data.len()).ok_or(OverlayError::EntryTooLarge)?;
        if length > MAX_OVERLAY_ITEM_BYTES {
            return Err(OverlayError::EntryTooLarge);
        }
        let derived = self.host_derived.contains(&key)
            || (!self.private.contains_key(&key)
                && self.read(host, slot, process, &key)?.is_some());
        let mut owned = self.read(host, slot, process, &key)?
            .map_or_else(Vec::new, |bytes| bytes.to_vec());
        owned.resize(owned.len().max(length), 0);
        owned[offset..length].copy_from_slice(data);
        self.write(slot, process, key.clone(), &owned)?;
        if derived {
            self.host_derived.insert(key);
        }
        Ok(())
    }

    /// Truncation cannot silently modify a synthetic host resource.
    /// A host fallback is copied to a private buffer before resizing.
    pub fn truncate_file(
        &mut self,
        host: &SyntheticBaseView,
        slot: SessionSlot,
        process: ProcessIdentity,
        key: OverlayKey,
        length: usize,
    ) -> Result<(), OverlayError> {
        self.check(slot, process)?;
        if key.surface != OverlaySurface::File {
            return Err(OverlayError::InvalidName);
        }
        if length > MAX_OVERLAY_ITEM_BYTES {
            return Err(OverlayError::EntryTooLarge);
        }
        let derived = self.host_derived.contains(&key) || !self.private.contains_key(&key);
        let mut owned = self.read(host, slot, process, &key)?
            .ok_or(OverlayError::MissingSource)?
            .to_vec();
        owned.resize(length, 0);
        self.write(slot, process, key.clone(), &owned)?;
        if derived {
            self.host_derived.insert(key);
        }
        Ok(())
    }

    /// A leaf delete always masks the host fixture, including its fallback.
    pub fn delete(
        &mut self,
        slot: SessionSlot,
        process: ProcessIdentity,
        key: OverlayKey,
    ) -> Result<(), OverlayError> {
        self.check(slot, process)?;
        if !self.tombstones.contains(&key)
            && self.tombstones.len() == MAX_OVERLAY_ITEMS
        {
            return Err(OverlayError::CapacityExceeded);
        }
        if let Some(value) = self.private.remove(&key) {
            self.private_bytes -= value.len();
        }
        self.host_derived.remove(&key);
        self.tombstones.insert(key);
        Ok(())
    }

    /// Hide a directory/subkey prefix in the fixture view. Later private
    /// entries beneath it remain visible, while old host descendants do not.
    pub fn delete_tree(
        &mut self,
        slot: SessionSlot,
        process: ProcessIdentity,
        prefix: OverlayKey,
    ) -> Result<(), OverlayError> {
        self.check(slot, process)?;
        if !self.tombstones.contains(&prefix)
            && self.tombstones.len() == MAX_OVERLAY_ITEMS
        {
            return Err(OverlayError::CapacityExceeded);
        }
        let old: Vec<_> = self.private.keys()
            .filter(|key| key.at_or_below(&prefix))
            .cloned()
            .collect();
        for key in old {
            if let Some(bytes) = self.private.remove(&key) {
                self.private_bytes -= bytes.len();
            }
            self.host_derived.remove(&key);
        }
        self.tombstones.insert(prefix);
        Ok(())
    }

    /// Transactional synthetic rename with checked quotas, no whole-state
    /// clone, and no OS host mutation. The old key is tombstoned in this slot.
    pub fn rename(
        &mut self,
        host: &SyntheticBaseView,
        slot: SessionSlot,
        process: ProcessIdentity,
        old: &OverlayKey,
        new: OverlayKey,
    ) -> Result<(), OverlayError> {
        self.check(slot, process)?;
        if old == &new {
            return if self.read(host, slot, process, old)?.is_some() {
                Ok(())
            } else {
                Err(OverlayError::MissingSource)
            };
        }
        if old.surface != new.surface {
            return Err(OverlayError::InvalidName);
        }
        // A host-backed item must not cross into a private-only region
        // via rename without explicit, independently authorized import.
        // There is no import capability in this synthetic model.
        if (!self.private.contains_key(old) || self.host_derived.contains(old))
            && self.host_fallback_denied(&new)
        {
            return Err(OverlayError::HostCopyIntoPrivateBoundary);
        }
        // Preflight every fallible constraint BEFORE changing this overlay.
        // Renaming one item must not clone up to 8 MiB of unrelated data.
        let size = self.read(host, slot, process, old)?
            .ok_or(OverlayError::MissingSource)?.len();
        if self.read(host, slot, process, &new)?.is_some() {
            return Err(OverlayError::AlreadyExists);
        }
        let private_source = self.private.contains_key(old);
        if size > MAX_OVERLAY_ITEM_BYTES {
            return Err(OverlayError::EntryTooLarge);
        }
        // An existing tombstone needs no extra slot; otherwise deletion
        // must have room to suppress the original host fallback.
        if !self.tombstones.contains(old) && self.tombstones.len() >= MAX_OVERLAY_ITEMS {
            return Err(OverlayError::CapacityExceeded);
        }
        let future_items = self.private.len() - usize::from(private_source) + 1;
        if future_items > MAX_OVERLAY_ITEMS {
            return Err(OverlayError::CapacityExceeded);
        }
        let previous = if private_source { size } else { 0 };
        let future_bytes = self.private_bytes.checked_sub(previous)
            .and_then(|remaining| remaining.checked_add(size))
            .ok_or(OverlayError::CapacityExceeded)?;
        if future_bytes > MAX_OVERLAY_PRIVATE_BYTES {
            return Err(OverlayError::CapacityExceeded);
        }
        let derived = self.host_derived.contains(old) || !private_source;
        // A host fallback may copy ONE bounded fixture value; failures
        // must happen before any private map or tombstone is changed.
        let host_copy = if private_source {
            None
        } else {
            Some(host.fixture_bytes(old).ok_or(OverlayError::MissingSource)?.to_vec())
        };
        // No logical Result/error path remains after the first mutation.
        // An owned private buffer is transferred without allocating a copy.
        let moved = if private_source {
            self.private.remove(old).expect("preflighted private COW source")
        } else {
            host_copy.expect("preflighted synthetic host source")
        };
        self.private_bytes = future_bytes;
        self.host_derived.remove(old);
        self.tombstones.insert(old.clone());
        self.private.insert(new.clone(), moved);
        if derived {
            self.host_derived.insert(new);
        }
        Ok(())
    }

    /// Deterministic merged enumeration, delimited by directory/key-prefix
    /// boundaries. Never combines different surfaces or another slot's state.
    pub fn list(
        &self,
        host: &SyntheticBaseView,
        slot: SessionSlot,
        process: ProcessIdentity,
        prefix: &OverlayKey,
    ) -> Result<Vec<OverlayKey>, OverlayError> {
        self.check(slot, process)?;
        let mut seen = BTreeSet::new();
        for key in host.entries.keys() {
            if key.at_or_below(prefix)
                && !self.masked(key) && !self.host_fallback_denied(key)
            {
                seen.insert(key.clone());
            }
        }
        for key in self.private.keys() {
            if key.at_or_below(prefix) {
                seen.insert(key.clone());
            }
        }
        Ok(seen.into_iter().collect())
    }

    pub fn private_byte_count(&self) -> usize {
        self.private_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(pid: u32) -> ProcessIdentity {
        ProcessIdentity::checked(pid, 133400000000000000 + u64::from(pid)).unwrap()
    }
    fn file(value: &str) -> OverlayKey {
        OverlayKey::parse(OverlaySurface::File, value).unwrap()
    }
    fn reg(value: &str) -> OverlayKey {
        OverlayKey::parse(OverlaySurface::RegistryValue, value).unwrap()
    }
    fn setup() -> (SyntheticBaseView, SyntheticSlotOverlay, SyntheticSlotOverlay) {
        let mut base = SyntheticBaseView::default();
        base.insert_fixture(file("world/data.bin"), b"original").unwrap();
        base.insert_fixture(file("world/other.bin"), b"second").unwrap();
        base.insert_fixture(reg("Software\\Game\\Setting"), b"base-setting").unwrap();
        (base,
         SyntheticSlotOverlay::new(SessionSlot::Mc02, id(22)).unwrap(),
         SyntheticSlotOverlay::new(SessionSlot::Mc03, id(33)).unwrap())
    }

    #[test]
    fn fixture_paths_are_case_insensitive_but_reject_os_path_escape() {
        assert_eq!(file("World/DATA.bin"), file("world/data.BIN"));
        assert_eq!(reg("Software\\GAME\\Setting"), reg("software\\game\\setting"));
        for (kind, bad) in [
            (OverlaySurface::File, "../host"),
            (OverlaySurface::File, "/absolute"),
            (OverlaySurface::File, "world//file"),
            (OverlaySurface::File, "world\\backslash"),
            (OverlaySurface::File, "world/NUL.txt"),
            (OverlaySurface::File, "world/com1.log"),
            (OverlaySurface::File, "world/bad:stream"),
            (OverlaySurface::RegistryValue, "HKCU\\..\\outside"),
            (OverlaySurface::RegistryValue, "Software\\A/B"),
            (OverlaySurface::RegistryValue, "\\HKCU"),
            (OverlaySurface::RegistryValue, "Software\\\\Value"),
            (OverlaySurface::File, "world/é"),
        ] {
            assert_eq!(OverlayKey::parse(kind, bad), Err(OverlayError::InvalidName));
        }
        assert!(OverlayKey::parse(OverlaySurface::File, "world/.cache").is_ok());
    }

    #[test]
    fn private_copy_on_write_masks_host_without_mutating_base() {
        let (base, mut box_a, box_b) = setup();
        let key = file("world/data.bin");
        assert_eq!(box_a.read(&base, SessionSlot::Mc02, id(22), &key).unwrap(), Some(b"original".as_slice()));
        box_a.copy_up(&base, SessionSlot::Mc02, id(22), &key).unwrap();
        assert_eq!(box_a.private_byte_count(), b"original".len());
        box_a.write(SessionSlot::Mc02, id(22), key.clone(), b"changed").unwrap();
        assert_eq!(box_a.read(&base, SessionSlot::Mc02, id(22), &key).unwrap(), Some(b"changed".as_slice()));
        assert_eq!(base.fixture_bytes(&key), Some(b"original".as_slice()));
        assert_eq!(box_b.read(&base, SessionSlot::Mc03, id(33), &key).unwrap(), Some(b"original".as_slice()));
        box_a.delete(SessionSlot::Mc02, id(22), key.clone()).unwrap();
        assert_eq!(box_a.read(&base, SessionSlot::Mc02, id(22), &key).unwrap(), None);
        assert_eq!(base.fixture_bytes(&key), Some(b"original".as_slice()));
    }

    #[test]
    fn registry_values_are_independent_of_file_names_and_peer_boxes() {
        let (base, mut a, b) = setup();
        let registry = reg("SOFTWARE\\Game\\Setting");
        a.write(SessionSlot::Mc02, id(22), registry.clone(), b"a-private").unwrap();
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22), &registry).unwrap(), Some(b"a-private".as_slice()));
        assert_eq!(b.read(&base, SessionSlot::Mc03, id(33), &registry).unwrap(), Some(b"base-setting".as_slice()));
        a.delete(SessionSlot::Mc02, id(22), registry.clone()).unwrap();
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22), &registry).unwrap(), None);
        assert_eq!(b.read(&base, SessionSlot::Mc03, id(33), &registry).unwrap(), Some(b"base-setting".as_slice()));
    }

    #[test]
    fn directory_tombstones_hide_host_descendants_but_allow_new_private_children() {
        let (base, mut box_a, box_b) = setup();
        box_a.delete_tree(SessionSlot::Mc02, id(22), file("world")).unwrap();
        assert_eq!(box_a.read(&base, SessionSlot::Mc02, id(22), &file("world/data.bin")).unwrap(), None);
        box_a.write(SessionSlot::Mc02, id(22), file("world/new.bin"), b"new").unwrap();
        let names = box_a.list(&base, SessionSlot::Mc02, id(22), &file("world")).unwrap();
        assert_eq!(names, vec![file("world/new.bin")]);
        assert_eq!(box_b.list(&base, SessionSlot::Mc03, id(33), &file("world")).unwrap().len(), 2);
        assert!(box_a.list(&base, SessionSlot::Mc02, id(22), &file("worl")).unwrap().is_empty());
    }

    #[test]
    fn pid_reuse_cross_slot_and_mc01_must_never_touch_a_box() {
        let (base, mut box_a, _) = setup();
        let key = file("world/data.bin");
        for (slot, process) in [
            (SessionSlot::Mc03, id(22)),
            (SessionSlot::Mc02, id(33)),
            (SessionSlot::Mc02, id(23)),
        ] {
            assert_eq!(box_a.read(&base, slot, process, &key), Err(OverlayError::WrongOwnerOrProcess));
            assert_eq!(box_a.write(slot, process, key.clone(), b"poison"), Err(OverlayError::WrongOwnerOrProcess));
        }
        assert_eq!(SyntheticSlotOverlay::new(SessionSlot::Mc01, id(1)).unwrap_err(), OverlayError::HostSlotForbidden);
        assert_eq!(SyntheticSlotOverlay::new(SessionSlot::Mc02,
            ProcessIdentity { pid: 0, created_file_time: 1 }).unwrap_err(), OverlayError::InvalidProcess);
        assert_eq!(box_a.read(&base, SessionSlot::Mc02, id(22), &key).unwrap(), Some(b"original".as_slice()));
    }

    #[test]
    fn rename_is_atomic_and_never_changes_original_host_entries() {
        let (base, mut a, _) = setup();
        let old = file("world/data.bin");
        a.rename(&base, SessionSlot::Mc02, id(22), &old, file("world/renamed.bin")).unwrap();
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22), &old).unwrap(), None);
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22), &file("world/renamed.bin")).unwrap(), Some(b"original".as_slice()));
        assert_eq!(base.fixture_bytes(&old), Some(b"original".as_slice()));
        assert_eq!(a.rename(&base, SessionSlot::Mc02, id(22), &file("world/renamed.bin"), file("world/other.bin")), Err(OverlayError::AlreadyExists));
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22), &file("world/renamed.bin")).unwrap(), Some(b"original".as_slice()));
        assert_eq!(a.rename(&base, SessionSlot::Mc02, id(22), &file("world/renamed.bin"), reg("Software\\Game\\New")), Err(OverlayError::InvalidName));
    }

    #[test]
    fn rename_at_item_and_tombstone_capacity_moves_bytes_atomically() {
        let (base, mut a, _) = setup();
        for n in 0..MAX_OVERLAY_ITEMS {
            a.write(SessionSlot::Mc02, id(22),
                file(&format!("quota/item_{n:04}.bin")), &[(n & 0xff) as u8]).unwrap();
        }
        let old = file("quota/item_0000.bin");
        let destination = file("quota/moved.bin");
        let original_buffer = a.private.get(&old).unwrap().as_ptr();
        let count_before = a.private_byte_count();
        a.rename(&base, SessionSlot::Mc02, id(22), &old, destination.clone()).unwrap();
        assert_eq!(a.private.len(), MAX_OVERLAY_ITEMS);
        assert_eq!(a.private_byte_count(), count_before);
        assert_eq!(a.private.get(&destination).unwrap().as_ptr(), original_buffer);
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22), &old).unwrap(), None);
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22), &destination).unwrap(),
            Some(&[0][..]));

        // Host source may not be copied when the destination's quota is full.
        let host_source = file("world/data.bin");
        let host_destination = file("quota/from_host.bin");
        assert_eq!(a.rename(&base, SessionSlot::Mc02, id(22),
            &host_source, host_destination.clone()), Err(OverlayError::CapacityExceeded));
        assert_eq!(a.private_byte_count(), count_before);
        assert_eq!(a.private.len(), MAX_OVERLAY_ITEMS);
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22), &host_source).unwrap(),
            Some(b"original".as_slice()));
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22), &host_destination).unwrap(), None);

        // Full tombstones must reject even a small rename without changing
        // its still-visible host source or creating a partial destination.
        let mut tombstoned = SyntheticSlotOverlay::new(SessionSlot::Mc04, id(44)).unwrap();
        for n in 0..MAX_OVERLAY_ITEMS {
            tombstoned.delete(SessionSlot::Mc04, id(44),
                file(&format!("removed/item_{n:04}.bin"))).unwrap();
        }
        let failed_destination = file("world/attempted.bin");
        assert_eq!(tombstoned.rename(&base, SessionSlot::Mc04, id(44),
            &host_source, failed_destination.clone()), Err(OverlayError::CapacityExceeded));
        assert_eq!(tombstoned.private.len(), 0);
        assert_eq!(tombstoned.tombstones.len(), MAX_OVERLAY_ITEMS);
        assert_eq!(tombstoned.private_byte_count(), 0);
        assert_eq!(tombstoned.read(&base, SessionSlot::Mc04, id(44), &host_source).unwrap(),
            Some(b"original".as_slice()));
        assert_eq!(tombstoned.read(&base, SessionSlot::Mc04, id(44),
            &failed_destination).unwrap(), None);
    }

    #[test]
    fn subtree_delete_then_private_root_replacement_does_not_resurrect_host() {
        let (base, mut box_a, _) = setup();
        box_a.delete_tree(SessionSlot::Mc02, id(22), file("world")).unwrap();
        box_a.write(SessionSlot::Mc02, id(22), file("world"), b"private").unwrap();
        assert_eq!(
            box_a.read(&base, SessionSlot::Mc02, id(22), &file("world")).unwrap(),
            Some(b"private".as_slice()),
        );
        assert_eq!(
            box_a.read(&base, SessionSlot::Mc02, id(22), &file("world/data.bin")).unwrap(),
            None,
        );
    }

    #[test]
    fn partial_writes_and_truncation_copy_host_bytes_into_private_view() {
        let (base, mut box_a, box_b) = setup();
        let key = file("world/data.bin");
        box_a.write_at(&base, SessionSlot::Mc02, id(22), key.clone(), 3, b"XY").unwrap();
        assert_eq!(
            box_a.read(&base, SessionSlot::Mc02, id(22), &key).unwrap(),
            Some(b"oriXYnal".as_slice()),
        );
        box_a.truncate_file(&base, SessionSlot::Mc02, id(22), key.clone(), 3).unwrap();
        assert_eq!(
            box_a.read(&base, SessionSlot::Mc02, id(22), &key).unwrap(),
            Some(b"ori".as_slice()),
        );
        assert_eq!(
            box_b.read(&base, SessionSlot::Mc03, id(33), &key).unwrap(),
            Some(b"original".as_slice()),
        );
        box_a.write_at(&base, SessionSlot::Mc02, id(22), file("world/sparse.bin"), 3, b"x").unwrap();
        assert_eq!(
            box_a.read(&base, SessionSlot::Mc02, id(22), &file("world/sparse.bin")).unwrap(),
            Some(&[0, 0, 0, b'x'][..]),
        );
        assert_eq!(box_a.write_at(
            &base, SessionSlot::Mc02, id(22), key.clone(), usize::MAX, b"x",
        ), Err(OverlayError::EntryTooLarge));
        assert_eq!(box_a.truncate_file(
            &base, SessionSlot::Mc02, id(22), reg("Software\\Game\\Setting"), 1,
        ), Err(OverlayError::InvalidName));
        assert_eq!(base.fixture_bytes(&key), Some(b"original".as_slice()));
    }

    #[test]
    fn private_only_policies_block_host_fallback_but_allow_own_private_values() {
        let (base, mut a, b) = setup();
        a.make_private_only(SessionSlot::Mc02, id(22), file("world")).unwrap();
        let key = file("WORLD/data.bin");
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22), &key).unwrap(), None);
        assert_eq!(a.copy_up(&base, SessionSlot::Mc02, id(22), &key),
            Err(OverlayError::MissingSource));
        assert_eq!(a.list(&base, SessionSlot::Mc02, id(22), &file("world")).unwrap().len(), 0);
        a.write(SessionSlot::Mc02, id(22), key.clone(), b"private").unwrap();
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22), &key).unwrap(),
            Some(b"private".as_slice()));
        assert_eq!(a.list(&base, SessionSlot::Mc02, id(22), &file("world")).unwrap(),
            vec![file("world/data.bin")]);
        assert_eq!(b.read(&base, SessionSlot::Mc03, id(33), &key).unwrap(),
            Some(b"original".as_slice()));
        assert_eq!(base.fixture_bytes(&key), Some(b"original".as_slice()));
    }

    #[test]
    fn private_only_file_policy_does_not_change_registry_namespace() {
        let (base, mut a, _) = setup();
        a.make_private_only(SessionSlot::Mc02, id(22), file("world")).unwrap();
        assert_eq!(
            a.read(&base, SessionSlot::Mc02, id(22), &reg("Software\\Game\\Setting")).unwrap(),
            Some(b"base-setting".as_slice()),
        );
        a.make_private_only(SessionSlot::Mc02, id(22), reg("Software\\Game")).unwrap();
        assert_eq!(
            a.read(&base, SessionSlot::Mc02, id(22), &reg("Software\\Game\\Setting")).unwrap(),
            None,
        );
        assert_eq!(
            a.make_private_only(SessionSlot::Mc03, id(33), reg("Software\\Other")),
            Err(OverlayError::WrongOwnerOrProcess),
        );
    }

    #[test]
    fn late_private_only_rule_cannot_relabel_existing_base_copies() {
        let (base, mut a, _) = setup();
        let key = file("world/data.bin");
        a.copy_up(&base, SessionSlot::Mc02, id(22), &key).unwrap();
        assert_eq!(
            a.make_private_only(SessionSlot::Mc02, id(22), file("world")),
            Err(OverlayError::AmbiguousPrivateProvenance),
        );
        assert_eq!(
            a.read(&base, SessionSlot::Mc02, id(22), &key).unwrap(),
            Some(b"original".as_slice()),
        );
    }

    #[test]
    fn host_rename_cannot_smuggle_bytes_across_private_only_boundary() {
        let (base, mut a, _) = setup();
        a.make_private_only(SessionSlot::Mc02, id(22), file("isolated")).unwrap();
        assert_eq!(
            a.rename(&base, SessionSlot::Mc02, id(22),
                &file("world/data.bin"), file("isolated/copied.bin")),
            Err(OverlayError::HostCopyIntoPrivateBoundary),
        );
        assert_eq!(
            a.read(&base, SessionSlot::Mc02, id(22),
                &file("isolated/copied.bin")).unwrap(),
            None,
        );
        assert_eq!(base.fixture_bytes(&file("world/data.bin")),
            Some(b"original".as_slice()));
        a.write(SessionSlot::Mc02, id(22), file("world/private.bin"), b"mine").unwrap();
        a.rename(&base, SessionSlot::Mc02, id(22),
            &file("world/private.bin"), file("isolated/renamed.bin")).unwrap();
        assert_eq!(
            a.read(&base, SessionSlot::Mc02, id(22),
                &file("isolated/renamed.bin")).unwrap(),
            Some(b"mine".as_slice()),
        );
    }

    #[test]
    fn copied_host_bytes_cannot_be_renamed_into_private_only_prefix() {
        let (base, mut a, _) = setup();
        let key = file("world/data.bin");
        a.copy_up(&base, SessionSlot::Mc02, id(22), &key).unwrap();
        a.make_private_only(SessionSlot::Mc02, id(22), file("isolated")).unwrap();
        assert_eq!(a.rename(&base, SessionSlot::Mc02, id(22),
            &key, file("isolated/copied.bin")),
            Err(OverlayError::HostCopyIntoPrivateBoundary));
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22), &key).unwrap(),
            Some(b"original".as_slice()));
        a.rename(&base, SessionSlot::Mc02, id(22),
            &key, file("world/moved.bin")).unwrap();
        assert_eq!(a.rename(&base, SessionSlot::Mc02, id(22),
            &file("world/moved.bin"), file("isolated/copied.bin")),
            Err(OverlayError::HostCopyIntoPrivateBoundary));
        // An actual newly authored full overwrite clears synthetic provenance.
        a.write(SessionSlot::Mc02, id(22), file("world/moved.bin"), b"new").unwrap();
        a.rename(&base, SessionSlot::Mc02, id(22),
            &file("world/moved.bin"), file("isolated/new.bin")).unwrap();
        assert_eq!(base.fixture_bytes(&key), Some(b"original".as_slice()));
    }

    #[test]
    fn host_derived_partial_write_remains_tagged_until_full_replacement() {
        let (base, mut a, _) = setup();
        a.make_private_only(SessionSlot::Mc02, id(22), file("isolated")).unwrap();
        let key = file("world/data.bin");
        a.write_at(&base, SessionSlot::Mc02, id(22), key.clone(), 1, b"x").unwrap();
        assert_eq!(a.rename(&base, SessionSlot::Mc02, id(22),
            &key, file("isolated/no.bin")),
            Err(OverlayError::HostCopyIntoPrivateBoundary));
        a.truncate_file(&base, SessionSlot::Mc02, id(22), key.clone(), 2).unwrap();
        assert_eq!(a.rename(&base, SessionSlot::Mc02, id(22),
            &key, file("isolated/no.bin")),
            Err(OverlayError::HostCopyIntoPrivateBoundary));
    }

    #[test]
    fn quotas_and_repeated_replacements_preserve_private_state_on_failure() {
        let (base, mut a, _) = setup();
        let key = file("world/new.bin");
        a.write(SessionSlot::Mc02, id(22), key.clone(), b"small").unwrap();
        assert_eq!(a.write(SessionSlot::Mc02, id(22), key.clone(), &vec![1; MAX_OVERLAY_ITEM_BYTES + 1]), Err(OverlayError::EntryTooLarge));
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22), &key).unwrap(), Some(b"small".as_slice()));
        assert_eq!(a.private_byte_count(), 5);
    }
}
