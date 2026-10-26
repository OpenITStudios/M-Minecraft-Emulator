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

fn render(original: &[u8]) -> io::Result<Vec<u8>> {
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
            output.push_str(PRESET[index].1);
            output.push_str(ending);
        } else {
            output.push_str(line); // Preserve all other settings byte-for-byte, including duplicates.
        }
    }
    for (index, (key, value)) in PRESET.iter().enumerate() {
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
    fn read(private: &File) -> io::Result<Self> {
        let mut path = directory_path(private)?;
        let mut directories = Vec::new();
        for part in [
            "Roaming",
            "Minecraft Education Edition",
            "games",
            "com.mojang",
            "minecraftpe",
        ] {
            path.push(part);
            match fs::create_dir(&path) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error),
            }
            directories.push(open_directory(
                &path,
                FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY,
            )?);
        }
        path.push("options.txt");
        let mut original = match options_file(&path) {
            Ok(file) => Some(file),
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => return Err(error),
        };
        let mut before = Vec::new();
        if let Some(file) = original.as_mut() {
            file.take((LIMIT + 1) as u64).read_to_end(&mut before)?;
        }
        let after = render(&before)?;
        Ok(Self {
            path,
            _directories: directories,
            original,
            before,
            after,
        })
    }

    fn commit(self) -> io::Result<bool> {
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
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .share_mode(0)
                .open(&backup)?;
            file.write_all(&self.before)?;
            file.sync_all()?;
        } else if self.path.try_exists()? {
            return Err(invalid(
                "Minecraft options appeared during preset preparation",
            ));
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
    let prepared: Vec<_> = stores
        .iter()
        .map(|store| Prepared::read(&store.private))
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
