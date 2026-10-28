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
    let mut status = GameSettingsStorage {has_settings:false, backup_available:false, can_reset:false, can_restore:false, improved_input_response_enabled:None, issue:None};
    let mut resettable=false;
    let mut input_response=None;
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
                status.has_settings = true;
                resettable=!bytes.is_empty();
            }
            Err(error) if error.kind()==io::ErrorKind::NotFound => {},
            Err(error) => return Err(error),
        }
        status.backup_available = backup_bytes(&path)?.is_some();
        Ok(())
    })();
    match inspected.and(admission) {
        Ok(()) => {status.can_reset=resettable;status.can_restore=status.backup_available;status.improved_input_response_enabled=input_response;}
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

// The corresponding Bedrock option name is publicly documented, but
// current Education 26.32 semantics are NOT yet verified. Report only an
// explicit, unambiguous saved 0/1. Missing/malformed/conflicting values
// are unknown; never infer a default, leak, or mitigation from this flag.
fn observed_input_response(bytes: &[u8]) -> Option<bool> {
    let text=std::str::from_utf8(bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(bytes)).ok()?;
    let mut observed=None;
    for raw in text.split('\n') {
        let line=raw.strip_suffix('\r').unwrap_or(raw);
        let Some((key,value))=line.split_once(':') else { continue; };
        if key != "ctrl_improvedInputResponse" { continue; }
        let enabled=match value { "0"=>false, "1"=>true, _=>return None };
        if observed.is_some_and(|previous| previous != enabled) { return None; }
        observed=Some(enabled);
    }
    observed
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
    let prepared = stores.iter().map(|store| Prepared::read_preset(&store.private, values[store.slot as usize - 1])).collect::<io::Result<Vec<_>>>()?;
    let mut changed = 0;
    for preset in prepared { changed += usize::from(preset.commit()?); }
    Ok(changed)
}

pub(super) fn apply_selected(store: &SlotStorage, preset: crate::engine::manager_settings::GraphicsPreset) -> io::Result<usize> {
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
    fn input_response_receipt_never_assumes_disabled_or_edits_saved_options() {
        assert_eq!(observed_input_response(b"ctrl_improvedInputResponse:1
"),Some(true));
        assert_eq!(observed_input_response(b"ï»¿ctrl_improvedInputResponse:0
"),Some(false));
        assert_eq!(observed_input_response(b"ctrl_improvedInputResponse:1
ctrl_improvedInputResponse:1
"),Some(true));
        for unknown in [
            &b""[..], &b"ctrl_improvedInputResponse:"[..],
            &b"ctrl_improvedInputResponse:2"[..], &b"ctrl_improvedInputResponse:off"[..],
            &b"ctrl_improvedInputResponse:1
ctrl_improvedInputResponse:0"[..],
            &b"CTRL_improvedInputResponse:1"[..],
            &b"ÿctrl_improvedInputResponse:1"[..],
        ] { assert_eq!(observed_input_response(unknown), None); }
        let original=b"ctrl_improvedInputResponse:1
volume:0.7
";
        assert!(render(original).unwrap().starts_with(original));
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
