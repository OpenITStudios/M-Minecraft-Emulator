//! The approved, image-bound preset. Minecraft options.txt remains the only settings store.
use super::{directory_path, image_id, invalid, open_directory, same_image, SlotStorage};
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::windows::fs::OpenOptionsExt,
    path::PathBuf,
};
use windows_sys::Win32::Storage::FileSystem::{
    FILE_FLAG_OPEN_REPARSE_POINT, FILE_LIST_DIRECTORY, FILE_READ_ATTRIBUTES, FILE_SHARE_DELETE,
    FILE_SHARE_READ,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SettingsBackup {
    schema: u32,
    file: String,
    sha256: String,
}
const BACKUP_INDEX: &str = "options.lazy-tester-backup.json";
fn settings_hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|byte|format!("{byte:02x}")).collect()
}

fn read_bounded(path: &std::path::Path, limit: usize) -> io::Result<Vec<u8>> {
    let file = options_file(path)?;
    let mut bytes = Vec::new();
    file.take((limit + 1) as u64).read_to_end(&mut bytes)?;
    if bytes.len() > limit { return Err(invalid("Settings file exceeds its size limit")); }
    Ok(bytes)
}

fn backup_bytes(path: &std::path::Path) -> io::Result<Option<Vec<u8>>> {
    let bytes = match read_bounded(&path.with_file_name(BACKUP_INDEX), 1024) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let backup: SettingsBackup = serde_json::from_slice(&bytes).map_err(|_| invalid("Settings backup receipt is invalid"))?;
    let id = backup.file.strip_prefix("options.mivubi-").and_then(|name|name.strip_suffix(".bak"));
    if backup.schema != 1 || id.is_none_or(|id|id.len()!=32 || !id.bytes().all(|b|b.is_ascii_hexdigit())) {
        return Err(invalid("Settings backup receipt contains an unsupported name"));
    }
    let bytes = read_bounded(&path.with_file_name(&backup.file), LIMIT)?;
    if settings_hash(&bytes) != backup.sha256 {
        return Err(invalid("Settings backup verification failed; no settings changed"));
    }
    render_preset(&bytes, crate::engine::manager_settings::GraphicsPreset::Low)?;
    Ok(Some(bytes))
}

fn options_path(private: &File, create: bool) -> io::Result<(PathBuf, Vec<File>)> {
    let mut path = directory_path(private)?;
    let mut directories = vec![private.try_clone()?];
    for part in ["Roaming", "Minecraft Education Edition", "games", "com.mojang", "minecraftpe"] {
        path.push(part);
        if create {
            match fs::create_dir(&path) {
                Ok(()) => {},
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {},
                Err(error) => return Err(error),
            }
        }
        directories.push(open_directory(&path, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY)?);
    }
    path.push("options.txt");
    Ok((path,directories))
}

pub(super) fn inspect_settings(private: &File, admission: io::Result<()>) -> crate::engine::native_clients_session::GameSettingsStorage {
    use crate::engine::native_clients_session::GameSettingsStorage;
    let mut status = GameSettingsStorage {has_settings:false, backup_available:false, can_reset:false, can_restore:false, improved_input_response_enabled:None, texture_streaming_enabled:None, issue:None};
    // An unadmitted slot's options and backups cannot be acted upon.
    // Refuse before any reads to avoid redundant work on active/foreign games.
    if let Err(error) = admission {
        status.issue = Some(error.to_string());
        return status;
    }
    let mut resettable=false;
    let mut input_response=None;
    let mut texture_streaming=None;
    let inspected = (|| -> io::Result<()> {
        let (path,_directories) = match options_path(private,false) {
            Ok(value) => value,
            Err(error) if error.kind()==io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error),
        };
        match read_bounded(&path,LIMIT) {
            Ok(bytes) => {
                render_preset(&bytes, crate::engine::manager_settings::GraphicsPreset::Low)?;
                // A historical Education leak report makes this one optional
                // video key worth observing; inspection never changes it.
                input_response = observed_input_response(&bytes);
                texture_streaming = observed_texture_streaming(&bytes);
                status.has_settings = true;
                resettable=!bytes.is_empty();
            }
            Err(error) if error.kind()==io::ErrorKind::NotFound => {},
            Err(error) => return Err(error),
        }
        status.backup_available = backup_bytes(&path)?.is_some();
        Ok(())
    })();
    match inspected {
        Ok(()) => {status.can_reset=resettable;status.can_restore=status.backup_available;status.improved_input_response_enabled=input_response;status.texture_streaming_enabled=texture_streaming;}
        Err(error) => status.issue=Some(error.to_string()),
    }
    status
}

pub(super) fn reset_settings(private: &File) -> io::Result<bool> {
    let mut prepared=Prepared::read_preset(private, crate::engine::manager_settings::GraphicsPreset::Low)?;
    prepared.after=Vec::new();
    prepared.commit_with_backup(true)
}

pub(super) fn restore_settings(private: &File) -> io::Result<bool> {
    let mut prepared=Prepared::read_preset(private, crate::engine::manager_settings::GraphicsPreset::Low)?;
    prepared.after=backup_bytes(&prepared.path)?.ok_or_else(||invalid("No verified settings backup is available"))?;
    prepared.commit_with_backup(true)
}

// Observe only unique, explicit 0/1 settings. A repeated same-value key
// is still ambiguous for any editor and must not appear actionable in UI.
fn observed_binary_option(bytes: &[u8], wanted: &str) -> Option<bool> {
    let text = std::str::from_utf8(bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(bytes)).ok()?;
    let mut observed = None;
    for raw in text.split('\n') {
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        let Some((name, value)) = line.split_once(':') else { continue; };
        if name != wanted { continue; }
        if observed.is_some() { return None; }
        observed = Some(match value {
            "0" => false,
            "1" => true,
            _ => return None,
        });
    }
    observed
}
fn observed_input_response(bytes: &[u8]) -> Option<bool> {
    observed_binary_option(bytes, "ctrl_improvedInputResponse")
}
fn observed_texture_streaming(bytes: &[u8]) -> Option<bool> {
    observed_binary_option(bytes, "gfx_texture_streaming")
}

// Operate on a user-confirmed existing setting, never seed a new key or
// alter another profile. Preserve BOM, mixed line endings and all other
// lines byte-for-byte; reject duplicates, invalid bytes and stray CR.
fn render_existing_binary_option(
    original: &[u8], wanted: &str, from: u8, to: u8,
) -> io::Result<Vec<u8>> {
    if original.len() > LIMIT || original.contains(&0) {
        return Err(invalid("Unsupported Minecraft options file"));
    }
    let bom = original.starts_with(b"\xef\xbb\xbf");
    let content = &original[if bom { 3 } else { 0 }..];
    std::str::from_utf8(content).map_err(|_| invalid("Minecraft options must be UTF-8"))?;
    let mut prefix = wanted.as_bytes().to_vec();
    prefix.push(b':');
    let mut output = Vec::with_capacity(original.len());
    if bom { output.extend_from_slice(&original[..3]); }
    let mut found = false;
    for raw in content.split_inclusive(|byte| *byte == b'\n') {
        let (line, ending): (&[u8], &[u8]) = if raw.ends_with(b"\r\n") {
            (&raw[..raw.len() - 2], b"\r\n")
        } else if raw.ends_with(b"\n") {
            (&raw[..raw.len() - 1], b"\n")
        } else { (raw, b"") };
        if line.contains(&b'\r') {
            return Err(invalid("Unsupported Minecraft options newline"));
        }
        if line.starts_with(&prefix) {
            if found || line.get(prefix.len()..) != Some(&[from][..]) {
                return Err(invalid("Minecraft option is not uniquely in its expected state"));
            }
            found = true;
            output.extend_from_slice(&prefix);
            output.push(to);
            output.extend_from_slice(ending);
        } else {
            output.extend_from_slice(raw);
        }
    }
    if !found {
        return Err(invalid("Minecraft option is absent or not explicitly in its expected state"));
    }
    Ok(output)
}
fn render_input_response_disabled(original: &[u8]) -> io::Result<Vec<u8>> {
    render_existing_binary_option(original, "ctrl_improvedInputResponse", b'1', b'0')
}
fn render_texture_streaming_enabled(original: &[u8]) -> io::Result<Vec<u8>> {
    render_existing_binary_option(original, "gfx_texture_streaming", b'0', b'1')
}
fn render_texture_streaming_disabled(original: &[u8]) -> io::Result<Vec<u8>> {
    render_existing_binary_option(original, "gfx_texture_streaming", b'1', b'0')
}
pub(super) fn disable_input_response(private: &File) -> io::Result<bool> {
    Prepared::read_existing_option(private, render_input_response_disabled)?.commit_with_backup(true)
}
pub(super) fn enable_texture_streaming(private: &File) -> io::Result<bool> {
    Prepared::read_existing_option(private, render_texture_streaming_enabled)?.commit_with_backup(true)
}
pub(super) fn disable_texture_streaming(private: &File) -> io::Result<bool> {
    Prepared::read_existing_option(private, render_texture_streaming_disabled)?.commit_with_backup(true)
}

const LIMIT: usize = 1024 * 1024;
// Observed Education version: Simple, 12 chunks, MSAA1, five decorative effects off.
// The caller pins the reviewed game image before using these option names/units.
const PRESET: [(&str, &str); 8] = [
    ("graphics_mode", "0"),
    ("gfx_viewdistance", "192"),
    ("gfx_msaa", "1"),
    ("gfx_transparentleaves", "0"),
    ("gfx_smoothlighting", "0"),
    ("gfx_fancyskies", "0"),
    ("gfx_toggleclouds", "0"),
    ("gfx_bubble_particles", "0"),
];

#[cfg(test)]
fn render(original: &[u8]) -> io::Result<Vec<u8>> {
    render_preset(original, crate::engine::manager_settings::GraphicsPreset::Low)
}

fn render_preset(original: &[u8], preset: crate::engine::manager_settings::GraphicsPreset) -> io::Result<Vec<u8>> {
    if preset == crate::engine::manager_settings::GraphicsPreset::Preserve {
        // Do not create/alter an options.txt, including an absent one.
        return Ok(original.to_vec());
    }
    let mut entries = PRESET;
    if preset == crate::engine::manager_settings::GraphicsPreset::High {
        // Keep the reviewed distance/MSAA units; enable the known visual-effect options.
        for (key, value) in &mut entries {
            if *key != "gfx_viewdistance" && *key != "gfx_msaa" { *value = "1"; }
        }
    }
    if original.len() > LIMIT || original.contains(&0) {
        return Err(invalid("Unsupported Minecraft options file"));
    }
    let bom = original.starts_with(b"\xef\xbb\xbf");
    let text = std::str::from_utf8(&original[if bom { 3 } else { 0 }..])
        .map_err(|_| invalid("Minecraft options must be UTF-8"))?;
    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut output = if bom {
        "\u{feff}".to_owned()
    } else {
        String::new()
    };
    let mut seen = [false; PRESET.len()];
    for line in text.split_inclusive('\n') {
        let ending = if line.ends_with("\r\n") {
            "\r\n"
        } else if line.ends_with('\n') {
            "\n"
        } else {
            ""
        };
        let body = &line[..line.len() - ending.len()];
        if body.contains('\r') {
            return Err(invalid("Unsupported Minecraft options newline"));
        }
        let entry = body
            .split_once(':')
            .and_then(|(key, _)| PRESET.iter().position(|(name, _)| *name == key));
        if let Some(index) = entry {
            seen[index] = true;
            output.push_str(PRESET[index].0);
            output.push(':');
            output.push_str(entries[index].1);
            output.push_str(ending);
        } else {
            output.push_str(line); // Preserve all other settings byte-for-byte, including duplicates.
        }
    }
    for (index, (key, value)) in entries.iter().enumerate() {
        if !seen[index] {
            if !output.is_empty() && output != "\u{feff}" && !output.ends_with('\n') {
                output.push_str(newline);
            }
            output.push_str(key);
            output.push(':');
            output.push_str(value);
            output.push_str(newline);
        }
    }
    if output.len() > LIMIT {
        return Err(invalid("Preset would exceed Minecraft options size limit"));
    }
    Ok(output.into_bytes())
}

struct Prepared {
    path: PathBuf,
    _directories: Vec<File>,
    original: Option<File>,
    before: Vec<u8>,
    after: Vec<u8>,
}

fn options_file(path: &std::path::Path) -> io::Result<File> {
    let file = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_DELETE)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    image_id(&file)?; // Refuse directories/reparse files, never follow an options-file link.
    Ok(file)
}

impl Prepared {
    #[cfg(test)]
    fn read(private: &File) -> io::Result<Self> {
        Self::read_preset(private, crate::engine::manager_settings::GraphicsPreset::Low)
    }

    fn read_preset(private: &File, preset: crate::engine::manager_settings::GraphicsPreset) -> io::Result<Self> {
        let (path, directories) = options_path(private,true)?;
        let mut original = match options_file(&path) {
            Ok(file) => Some(file),
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => return Err(error),
        };
        let mut before = Vec::new();
        if let Some(file) = original.as_mut() {
            file.take((LIMIT + 1) as u64).read_to_end(&mut before)?;
        }
        let after = render_preset(&before, preset)?;
        Ok(Self {
            path,
            _directories: directories,
            original,
            before,
            after,
        })
    }

    // Both opt-in choices require an existing reviewed options.txt and
    // pinned directories. No missing setting, game profile or file is created.
    fn read_existing_option(
        private: &File, render: fn(&[u8]) -> io::Result<Vec<u8>>,
    ) -> io::Result<Self> {
        let (path, directories) = options_path(private, false)?;
        let mut original = options_file(&path)?;
        let mut before = Vec::new();
        Read::by_ref(&mut original)
            .take((LIMIT + 1) as u64)
            .read_to_end(&mut before)?;
        let after = render(&before)?;
        Ok(Self {
            path,
            _directories: directories,
            original: Some(original),
            before,
            after,
        })
    }

    fn commit(self) -> io::Result<bool> {
        self.commit_with_backup(false)
    }

    fn commit_with_backup(self, publish_backup: bool) -> io::Result<bool> {
        if self.before == self.after {
            return Ok(false);
        }
        let mut id: windows_sys::core::GUID = unsafe { std::mem::zeroed() };
        if unsafe { windows_sys::Win32::System::Com::CoCreateGuid(&mut id) } < 0 {
            return Err(invalid("Cannot allocate preset update identity"));
        }
        let suffix: String = id.data4.iter().map(|b| format!("{b:02x}")).collect();
        let name = format!(
            "options.mivubi-{:08x}{:04x}{:04x}{suffix}",
            id.data1, id.data2, id.data3
        );
        let temporary = self.path.with_file_name(format!("{name}.tmp"));
        let backup = self.path.with_file_name(format!("{name}.bak"));
        if let Some(original) = self.original.as_ref() {
            let current = options_file(&self.path)?;
            if !same_image(&image_id(original)?, &image_id(&current)?) {
                return Err(invalid(
                    "Minecraft options changed during preset preparation",
                ));
            }
        } else if self.path.try_exists()? {
            return Err(invalid("Minecraft options appeared during preparation"));
        }
        if self.original.is_some() || publish_backup {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .share_mode(0)
                .open(&backup)?;
            file.write_all(&self.before)?;
            file.sync_all()?;
        }
        if publish_backup {
            if read_bounded(&backup,LIMIT)? != self.before {
                return Err(invalid("Settings backup verification failed; original settings were preserved"));
            }
            // Publish the verified recovery point before replacing settings. A failed update
            // leaves the original or a usable backup; interrupted temporary files are ignored.
            let receipt = SettingsBackup {schema:1,file:format!("{name}.bak"),sha256:settings_hash(&self.before)};
            let receipt_temp=self.path.with_file_name(format!("{name}.json.tmp"));
            let mut file=OpenOptions::new().write(true).create_new(true).share_mode(0).open(&receipt_temp)?;
            let result=(||{file.write_all(&serde_json::to_vec(&receipt).map_err(io::Error::other)?)?;file.sync_all()}) ();
            drop(file);
            let result=result.and_then(|()|fs::rename(&receipt_temp,self.path.with_file_name(BACKUP_INDEX)));
            if result.is_err(){let _=fs::remove_file(&receipt_temp);}
            result?;
        }
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .share_mode(0)
            .open(&temporary)?;
        let result = file.write_all(&self.after).and_then(|()| file.sync_all());
        drop(file);
        let result = result.and_then(|()| fs::rename(&temporary, &self.path));
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        } // Only our create-new temporary.
        result?;
        Ok(true)
    }
}

pub(super) fn apply(stores: &[SlotStorage]) -> io::Result<usize> {
    // The launch owner already holds its session gate. Also protect direct/internal callers.
    if !super::super::inspect_native_workspace()?
        .processes
        .is_empty()
    {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Close Minecraft normally before applying its preset",
        ));
    }
    apply_inactive(stores)
}

// The retained session owner proves these particular profiles have no resumed child.
// Other profiles may be running; never pass their stores here.
pub(super) fn apply_inactive(stores: &[SlotStorage]) -> io::Result<usize> {
    let values = crate::engine::manager_settings::load_graphics()?;
    // Preserve never resolves, creates, or rewrites a game's options directory.
    // Existing explicit High/Low selections retain their legacy behavior.
    let prepared = stores.iter().filter(|store| values[store.slot as usize - 1]
        != crate::engine::manager_settings::GraphicsPreset::Preserve)
        .map(|store| Prepared::read_preset(&store.private, values[store.slot as usize - 1]))
        .collect::<io::Result<Vec<_>>>()?;
    let mut changed = 0;
    for preset in prepared { changed += usize::from(preset.commit()?); }
    Ok(changed)
}

pub(super) fn apply_selected(store: &SlotStorage, preset: crate::engine::manager_settings::GraphicsPreset) -> io::Result<usize> {
    if preset == crate::engine::manager_settings::GraphicsPreset::Preserve {
        return Ok(0); // Only save the preference; never rewrite Minecraft graphics options.
    }
    Ok(usize::from(Prepared::read_preset(&store.private, preset)?.commit()?))
}

#[cfg(test)]
fn apply_directories<'a>(directories: impl Iterator<Item = &'a File>) -> io::Result<usize> {
    let prepared: Vec<_> = directories
        .map(Prepared::read)
        .collect::<io::Result<_>>()?;
    // Validate every profile before replacing any options file; retries are idempotent.
    let mut changed = 0;
    for preset in prepared {
        changed += usize::from(preset.commit()?);
    }
    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn denied_settings_inspection_returns_unknown_before_opening_options() {
        let root = super::super::fresh_root(&std::env::temp_dir()).unwrap();
        let private = open_directory(&root, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY).unwrap();
        // An old inspection would report this invalid file instead of admission refusal.
        let path = Prepared::read(&private).unwrap().path;
        let original = b"gfx_texture_streaming:1\nctrl_improvedInputResponse:1\ninvalid:\xff\n";
        fs::write(&path, original).unwrap();
        let blocked = inspect_settings(&private, Err(invalid("Profile is running")));
        assert_eq!(blocked.issue.as_deref(), Some("Profile is running"));
        assert!(!blocked.has_settings && !blocked.backup_available);
        assert!(!blocked.can_reset && !blocked.can_restore);
        assert_eq!(blocked.texture_streaming_enabled, None);
        assert_eq!(blocked.improved_input_response_enabled, None);
        assert_eq!(fs::read(&path).unwrap(), original);
        assert!(inspect_settings(&private, Ok(())).issue.is_some(),
            "Admitted inspection must still reject malformed options");
        drop(private);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn input_response_receipt_never_assumes_disabled_or_edits_saved_options() {
        assert_eq!(observed_input_response(b"ctrl_improvedInputResponse:1\n"),Some(true));
        assert_eq!(observed_input_response(b"\xef\xbb\xbfctrl_improvedInputResponse:0\r\n"),Some(false));
        assert_eq!(observed_input_response(b"ctrl_improvedInputResponse:1\nctrl_improvedInputResponse:1\n"),None);
        for unknown in [
            &b""[..], &b"ctrl_improvedInputResponse:"[..],
            &b"ctrl_improvedInputResponse:2"[..], &b"ctrl_improvedInputResponse:off"[..],
            &b"ctrl_improvedInputResponse:1\nctrl_improvedInputResponse:0"[..],
            &b"CTRL_improvedInputResponse:1"[..],
            &b"\xffctrl_improvedInputResponse:1"[..],
        ] { assert_eq!(observed_input_response(unknown), None); }
        let original=b"ctrl_improvedInputResponse:1\nvolume:0.7\n";
        assert!(render(original).unwrap().starts_with(original));
    }

    #[test]
    fn input_response_workaround_changes_one_existing_key_and_restores_backup() {
        let root = super::super::fresh_root(&std::env::temp_dir()).unwrap();
        let private = open_directory(&root, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY).unwrap();
        // Missing directories/options are never created by the workaround.
        assert!(disable_input_response(&private).is_err());
        assert!(!root.join("Roaming").exists());
        let path = Prepared::read(&private).unwrap().path;
        let before = b"\xef\xbb\xbfvolume:0.75\r\nctrl_improvedInputResponse:1\r\nkey_jump:32\r\n";
        let after = b"\xef\xbb\xbfvolume:0.75\r\nctrl_improvedInputResponse:0\r\nkey_jump:32\r\n";
        fs::write(&path, before).unwrap();
        fs::write(root.join("world-fixture"), b"untouched").unwrap();
        assert_eq!(render_input_response_disabled(before).unwrap(), after);
        assert!(disable_input_response(&private).unwrap());
        assert_eq!(fs::read(&path).unwrap(), after);
        assert_eq!(backup_bytes(&path).unwrap().unwrap(), before);
        assert_eq!(inspect_settings(&private, Ok(())).improved_input_response_enabled, Some(false));
        assert!(restore_settings(&private).unwrap());
        assert_eq!(fs::read(&path).unwrap(), before);
        assert_eq!(fs::read(root.join("world-fixture")).unwrap(), b"untouched");
        drop(private);
        assert_eq!(root.parent(), Some(std::env::temp_dir().as_path()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn input_response_workaround_rejects_missing_disabled_ambiguous_or_invalid_settings() {
        for value in [
            &b""[..],
            &b"account:kept\n"[..],
            &b"ctrl_improvedInputResponse:0\n"[..],
            &b"ctrl_improvedInputResponse:2\n"[..],
            &b"ctrl_improvedInputResponse:off\n"[..],
            &b"ctrl_improvedInputResponse:1\nctrl_improvedInputResponse:1\n"[..],
            &b"ctrl_improvedInputResponse:1\nctrl_improvedInputResponse:0\n"[..],
            &b"ctrl_improvedInputResponse:1\nother:bad\rvalue"[..],
            &b"ctrl_improvedInputResponse:1\n\0"[..],
            &b"ctrl_improvedInputResponse:1\n\xff"[..],
        ] {
            assert!(render_input_response_disabled(value).is_err());
        }
        assert_eq!(render_input_response_disabled(b"ctrl_improvedInputResponse:1").unwrap(),
            b"ctrl_improvedInputResponse:0");
    }

    #[test]
    fn texture_streaming_opt_in_requires_unique_off_flag_and_preserves_other_options() {
        let before = b"\xef\xbb\xbfgfx_texture_streaming:0\r\nvolume:0.75\nctrl_improvedInputResponse:1\r\n";
        let after = b"\xef\xbb\xbfgfx_texture_streaming:1\r\nvolume:0.75\nctrl_improvedInputResponse:1\r\n";
        assert_eq!(observed_texture_streaming(before), Some(false));
        assert_eq!(render_texture_streaming_enabled(before).unwrap(), after);
        assert_eq!(observed_texture_streaming(after), Some(true));
        assert_eq!(render_texture_streaming_disabled(after).unwrap(), before);
        assert_eq!(render_input_response_disabled(before).unwrap(),
            b"\xef\xbb\xbfgfx_texture_streaming:0\r\nvolume:0.75\nctrl_improvedInputResponse:0\r\n");
        for ambiguous in [
            &b""[..], &b"other:0\n"[..], &b"gfx_texture_streaming:1\n"[..],
            &b"gfx_texture_streaming:2\n"[..],
            &b"gfx_texture_streaming:0\ngfx_texture_streaming:0\n"[..],
            &b"gfx_texture_streaming:0\ngfx_texture_streaming:1\n"[..],
            &b"gfx_texture_streaming:0\0"[..],
            &b"gfx_texture_streaming:0\nother:bad\rvalue"[..],
            &b"gfx_texture_streaming:0\n\xff"[..],
        ] {
            assert!(render_texture_streaming_enabled(ambiguous).is_err());
        }
        assert_eq!(observed_texture_streaming(b"gfx_texture_streaming:0\ngfx_texture_streaming:0\n"), None);
        assert_eq!(observed_texture_streaming(b"unknown:1\n"), None);
    }

    #[test]
    fn preserve_visuals_keeps_exact_options_bytes_and_creates_no_graphics_override() {
        use crate::engine::manager_settings::GraphicsPreset::{Preserve, High, Low};
        let original=b"gfx_viewdistance:720\r\ngfx_msaa:4\r\ngfx_smoothlighting:1\r\nvolume:0.8\r\n";
        assert_eq!(render_preset(original, Preserve).unwrap(), original);
        assert!(render_preset(&[], Preserve).unwrap().is_empty());
        assert_ne!(render_preset(original, Low).unwrap(), original);
        assert_ne!(render_preset(original, High).unwrap(), original);
        let root=super::super::fresh_root(&std::env::temp_dir()).unwrap();
        let private=open_directory(&root,FILE_READ_ATTRIBUTES|FILE_LIST_DIRECTORY).unwrap();
        assert!(!root.join("Roaming").exists());
        // A Preserve selection does not create any Minecraft options directories.
        let unchanged = render_preset(original, Preserve).unwrap();
        assert_eq!(unchanged, original);
        drop(private);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn reversible_texture_streaming_changes_only_existing_unique_binary_key() {
        let initial=b"\xef\xbb\xbfgfx_texture_streaming:0\r\nvolume:0.75\nkey_jump:32\r\n";
        let enabled=b"\xef\xbb\xbfgfx_texture_streaming:1\r\nvolume:0.75\nkey_jump:32\r\n";
        assert_eq!(render_texture_streaming_enabled(initial).unwrap(), enabled);
        assert_eq!(render_texture_streaming_disabled(enabled).unwrap(), initial);
        for invalid in [
            &b""[..], &b"gfx_texture_streaming:0\n"[..],
            &b"gfx_texture_streaming:1\ngfx_texture_streaming:1\n"[..],
            &b"gfx_texture_streaming:1\ngfx_texture_streaming:0\n"[..],
            &b"gfx_texture_streaming:2\n"[..], &b"gfx_texture_streaming:1\0"[..],
            &b"gfx_texture_streaming:1\rstray"[..], &b"gfx_texture_streaming:1\n\xff"[..],
        ] { assert!(render_texture_streaming_disabled(invalid).is_err()); }
    }

    #[test]
    fn texture_streaming_disable_is_reversible_and_keeps_world_unchanged() {
        let root=super::super::fresh_root(&std::env::temp_dir()).unwrap();
        let private=open_directory(&root,FILE_READ_ATTRIBUTES|FILE_LIST_DIRECTORY).unwrap();
        assert!(disable_texture_streaming(&private).is_err());
        assert!(!root.join("Roaming").exists());
        let path=Prepared::read(&private).unwrap().path;
        let before=b"gfx_texture_streaming:1\r\naccount:kept\n";
        let after=b"gfx_texture_streaming:0\r\naccount:kept\n";
        fs::write(&path,before).unwrap();
        fs::write(root.join("world-fixture"),b"untouched").unwrap();
        assert!(disable_texture_streaming(&private).unwrap());
        assert_eq!(fs::read(&path).unwrap(),after);
        assert_eq!(backup_bytes(&path).unwrap().unwrap(),before);
        assert_eq!(inspect_settings(&private,Ok(())).texture_streaming_enabled,Some(false));
        assert!(disable_texture_streaming(&private).is_err());
        assert!(restore_settings(&private).unwrap());
        assert_eq!(fs::read(&path).unwrap(),before);
        assert_eq!(fs::read(root.join("world-fixture")).unwrap(),b"untouched");
        drop(private);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn texture_streaming_opt_in_preserves_world_and_can_restore_old_setting() {
        let root = super::super::fresh_root(&std::env::temp_dir()).unwrap();
        let private = open_directory(&root, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY).unwrap();
        // This explicit choice must not populate missing Minecraft folders.
        assert!(enable_texture_streaming(&private).is_err());
        assert!(!root.join("Roaming").exists());
        let path = Prepared::read(&private).unwrap().path;
        let before = b"gfx_texture_streaming:0\naccount:kept\n";
        fs::write(&path, before).unwrap();
        fs::write(root.join("world-fixture"), b"untouched").unwrap();
        let inspected = inspect_settings(&private, Ok(()));
        assert_eq!(inspected.texture_streaming_enabled, Some(false));
        assert!(inspected.can_reset);
        assert!(enable_texture_streaming(&private).unwrap());
        assert_eq!(fs::read(&path).unwrap(), b"gfx_texture_streaming:1\naccount:kept\n");
        assert_eq!(backup_bytes(&path).unwrap().unwrap(), before);
        assert_eq!(inspect_settings(&private, Ok(())).texture_streaming_enabled, Some(true));
        assert!(enable_texture_streaming(&private).is_err());
        assert!(restore_settings(&private).unwrap());
        assert_eq!(fs::read(&path).unwrap(), before);
        assert_eq!(fs::read(root.join("world-fixture")).unwrap(), b"untouched");
        drop(private);
        assert_eq!(root.parent(), Some(std::env::temp_dir().as_path()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn settings_reset_restore_roundtrip_and_undo_preserve_other_profile_data() {
        let root=super::super::fresh_root(&std::env::temp_dir()).unwrap();
        let other=super::super::fresh_root(&std::env::temp_dir()).unwrap();
        let private=open_directory(&root,FILE_READ_ATTRIBUTES|FILE_LIST_DIRECTORY).unwrap();
        let other_pin=open_directory(&other,FILE_READ_ATTRIBUTES|FILE_LIST_DIRECTORY).unwrap();
        let other_path=Prepared::read(&other_pin).unwrap().path;
        fs::write(&other_path,b"other_launcher:unchanged\n").unwrap();
        let fresh=inspect_settings(&private,Ok(()));
        assert!(!fresh.has_settings && !fresh.can_reset && !fresh.backup_available);
        assert!(!root.join("Roaming").exists(),"Inspection created settings directories");
        let path=Prepared::read(&private).unwrap().path;
        let original=b"\xef\xbb\xbfgraphics_mode:1\r\nkey_jump:32\r\nvolume:0.75\r\n";
        fs::write(&path,original).unwrap();
        fs::write(root.join("synthetic-world"),b"world unchanged").unwrap();
        fs::write(root.join("synthetic-account"),b"account unchanged").unwrap();
        assert!(!inspect_settings(&private,Err(invalid("Profile is running"))).can_reset);
        assert!(reset_settings(&private).unwrap());
        assert_eq!(fs::read(&path).unwrap(),b"");
        assert_eq!(backup_bytes(&path).unwrap().unwrap(),original);
        assert!(inspect_settings(&private,Ok(())).can_restore);
        assert!(restore_settings(&private).unwrap());
        assert_eq!(fs::read(&path).unwrap(),original);
        assert_eq!(backup_bytes(&path).unwrap().unwrap(),b"", "Restore did not back up the current settings");
        assert!(restore_settings(&private).unwrap());
        assert_eq!(fs::read(&path).unwrap(),b"");
        assert_eq!(fs::read(root.join("synthetic-world")).unwrap(),b"world unchanged");
        assert_eq!(fs::read(root.join("synthetic-account")).unwrap(),b"account unchanged");
        assert_eq!(fs::read(other_path).unwrap(),b"other_launcher:unchanged\n");
        drop(private);
        drop(other_pin);
        for path in [root,other] {
            assert_eq!(path.parent(),Some(std::env::temp_dir().as_path()));
            fs::remove_dir_all(path).unwrap();
        }
    }

    #[test]
    fn settings_restore_rejects_corruption_traversal_and_interrupted_publication() {
        let root=super::super::fresh_root(&std::env::temp_dir()).unwrap();
        let private=open_directory(&root,FILE_READ_ATTRIBUTES|FILE_LIST_DIRECTORY).unwrap();
        let path=Prepared::read(&private).unwrap().path;
        fs::write(&path,b"key_jump:32\n").unwrap();
        assert!(restore_settings(&private).is_err());
        let index=path.with_file_name(BACKUP_INDEX);
        fs::create_dir(&index).unwrap();
        assert!(reset_settings(&private).is_err(),"Reset proceeded without publishing its backup");
        assert_eq!(fs::read(&path).unwrap(),b"key_jump:32\n");
        fs::remove_dir(&index).unwrap();
        reset_settings(&private).unwrap();
        let receipt=fs::read(&index).unwrap();
        let record:SettingsBackup=serde_json::from_slice(&receipt).unwrap();
        let backup=path.with_file_name(&record.file);
        fs::write(&backup,b"corrupted:backup\n").unwrap();
        assert!(restore_settings(&private).is_err());
        assert_eq!(fs::read(&path).unwrap(),b"");
        assert!(!inspect_settings(&private,Ok(())).can_restore);
        let forged=SettingsBackup {schema:1,file:"../outside.bak".into(),sha256:record.sha256};
        fs::write(&index,serde_json::to_vec(&forged).unwrap()).unwrap();
        assert!(restore_settings(&private).is_err());
        assert_eq!(fs::read(&path).unwrap(),b"");
        fs::write(&index,b"invalid receipt").unwrap();
        assert!(restore_settings(&private).is_err());
        drop(private);
        assert_eq!(root.parent(),Some(std::env::temp_dir().as_path()));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn settings_reset_refuses_a_junction_ancestor_without_touching_its_target() {
        let root=super::super::fresh_root(&std::env::temp_dir()).unwrap();
        let outside=super::super::fresh_root(&std::env::temp_dir()).unwrap();
        fs::write(outside.join("keep"),b"untouched").unwrap();
        let output=std::process::Command::new("cmd.exe").args(["/c","mklink","/J"])
            .arg(root.join("Roaming")).arg(&outside).output().unwrap();
        assert!(output.status.success());
        let private=open_directory(&root,FILE_READ_ATTRIBUTES|FILE_LIST_DIRECTORY).unwrap();
        assert!(reset_settings(&private).is_err());
        assert!(!inspect_settings(&private,Ok(())).can_reset);
        assert_eq!(fs::read(outside.join("keep")).unwrap(),b"untouched");
        drop(private);
        fs::remove_dir(root.join("Roaming")).unwrap();
        for path in [root,outside] {
            assert_eq!(path.parent(),Some(std::env::temp_dir().as_path()));
            fs::remove_dir_all(path).unwrap();
        }
    }

    #[test]
    fn high_low_preserve_world_settings_and_switch_idempotently() {
        use crate::engine::manager_settings::GraphicsPreset::{High, Low};
        let original = b"gfx_simulationdistance:8\naccount:kept\ngfx_viewdistance:720\n";
        let high = render_preset(original, High).unwrap();
        assert!(String::from_utf8_lossy(&high).contains("graphics_mode:1\n"));
        assert!(String::from_utf8_lossy(&high).contains("gfx_toggleclouds:1\n"));
        assert!(String::from_utf8_lossy(&high).contains("account:kept\n"));
        assert_eq!(render_preset(&high, High).unwrap(), high);
        assert_eq!(render_preset(&high, Low).unwrap(), render(original).unwrap());
    }

    #[test]
    fn five_profile_preset_validates_all_inputs_before_writing_and_retries_idempotently() {
        let roots: Vec<_> = (0..5).map(|_| super::super::fresh_root(&std::env::temp_dir()).unwrap()).collect();
        let pins: Vec<_> = roots.iter().map(|root| open_directory(root, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY).unwrap()).collect();
        let paths: Vec<_> = pins.iter().map(|pin| Prepared::read(pin).unwrap().path).collect();
        for path in &paths { fs::write(path, b"graphics_mode:1\nsimulation:7\nother:kept\n").unwrap(); }
        fs::write(&paths[4], b"invalid:\xff").unwrap();
        assert!(apply_directories(pins.iter()).is_err());
        for path in &paths[..4] { assert_eq!(fs::read(path).unwrap(), b"graphics_mode:1\nsimulation:7\nother:kept\n"); }
        fs::write(&paths[4], b"simulation:7\nother:kept\n").unwrap();
        assert_eq!(apply_directories(pins.iter()).unwrap(), 5);
        assert_eq!(apply_directories(pins.iter()).unwrap(), 0);
        for path in &paths {
            let text = fs::read_to_string(path).unwrap();
            assert!(text.contains("simulation:7\nother:kept\n"));
            assert!(text.contains("gfx_viewdistance:192\n"));
        }
        drop(pins);
        for root in roots { assert_eq!(root.parent(), Some(std::env::temp_dir().as_path())); fs::remove_dir_all(root).unwrap(); }
    }

    #[test]
    fn preset_preserves_unowned_settings_and_is_idempotent() {
        let input = b"\xef\xbb\xbfgraphics_mode:1\r\ngfx_viewdistance:720\r\ngfx_simulationdistance:8\r\ngfx_fullscreen:1\r\nother:value:with:colons\r\ngfx_msaa:2\r\ngraphics_mode:1\r\n";
        let output = render(input).unwrap();
        let text = std::str::from_utf8(&output).unwrap();
        assert!(output.starts_with(b"\xef\xbb\xbf"));
        for line in [
            "gfx_simulationdistance:8\r\n",
            "gfx_fullscreen:1\r\n",
            "other:value:with:colons\r\n",
        ] {
            assert!(text.contains(line));
        }
        assert_eq!(text.matches("graphics_mode:0\r\n").count(), 2);
        assert!(!text.contains("gfx_viewdistance:720"));
        for (key, value) in PRESET {
            assert!(text.contains(&format!("{key}:{value}\r\n")));
        }
        assert_eq!(render(&output).unwrap(), output);
        let without_final_newline = render(b"unknown:kept").unwrap();
        assert!(without_final_newline.starts_with(b"unknown:kept\n"));
        assert_eq!(
            render(&without_final_newline).unwrap(),
            without_final_newline
        );
    }

    #[test]
    fn preset_rejects_unsupported_bytes_without_repairing_them() {
        assert!(render(&vec![b'x'; LIMIT]).is_err());
        for bytes in [
            &b"\xff\xfe"[..],
            &b"x:\0"[..],
            &b"x:y\rz"[..],
            &vec![b'x'; LIMIT + 1][..],
        ] {
            assert!(render(bytes).is_err());
        }
    }

    #[test]
    fn preset_replaces_only_options_with_backup_and_seeds_new_profiles() {
        let root = super::super::fresh_root(&std::env::temp_dir()).unwrap();
        let private = open_directory(&root, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY).unwrap();
        let fresh = Prepared::read(&private).unwrap();
        let path = fresh.path.clone();
        assert!(fresh.commit().unwrap());
        assert_eq!(fs::read(&path).unwrap(), render(b"").unwrap());
        assert!(!Prepared::read(&private).unwrap().commit().unwrap());
        let original = b"graphics_mode:1\nsimulation:unchanged\nresolution:unchanged\n";
        fs::write(&path, original).unwrap();
        let prepared = Prepared::read(&private).unwrap();
        assert!(OpenOptions::new().write(true).open(&path).is_err());
        assert!(prepared.commit().unwrap());
        let backups: Vec<_> = fs::read_dir(path.parent().unwrap())
            .unwrap()
            .map(|x| x.unwrap().path())
            .filter(|p| p.extension().is_some_and(|x| x == "bak"))
            .collect();
        assert_eq!(backups.len(), 1);
        assert_eq!(fs::read(&backups[0]).unwrap(), original);
        assert_eq!(fs::read(&path).unwrap(), render(original).unwrap());
        fs::write(&path, original).unwrap();
        let stale = Prepared::read(&private).unwrap();
        let moved = path.with_extension("moved");
        fs::rename(&path, &moved).unwrap();
        fs::write(&path, b"replacement:must-survive\n").unwrap();
        assert!(stale.commit().is_err());
        assert_eq!(fs::read(&path).unwrap(), b"replacement:must-survive\n");
        assert_eq!(fs::read(&moved).unwrap(), original);
        fs::write(&path, b"invalid:\xff").unwrap();
        assert!(Prepared::read(&private).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"invalid:\xff");
    }
}
