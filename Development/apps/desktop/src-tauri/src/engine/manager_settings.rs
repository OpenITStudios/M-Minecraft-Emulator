use serde::{Deserialize, Serialize};
use std::{fs, io, io::Read, path::PathBuf};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum BackgroundAudio { #[default] Keep, Mute }
pub fn load_audio()->io::Result<BackgroundAudio> {read_audio(&product_settings()?.join("audio.json"))}
fn read_audio(path:&std::path::Path)->io::Result<BackgroundAudio> {
    match fs::symlink_metadata(path) {
        Err(e) if e.kind()==io::ErrorKind::NotFound=>return Ok(BackgroundAudio::Keep),
        Err(e)=>return Err(e),
        Ok(m) if !m.is_file()||m.file_type().is_symlink()=>return Err(io::Error::other("Audio settings must be an ordinary file")),
        Ok(_)=>{}
    }
    let mut bytes=Vec::new();fs::File::open(path)?.take(65).read_to_end(&mut bytes)?;
    if bytes.len()>64{return Err(io::Error::other("Audio settings are too large"));}
    serde_json::from_slice(&bytes).map_err(io::Error::other)
}
pub fn save_audio(value:BackgroundAudio)->io::Result<()> {write_audio(&product_settings()?.join("audio.json"),value)}
fn write_audio(path:&std::path::Path,value:BackgroundAudio)->io::Result<()> {
    use std::io::Write;
    read_audio(path)?;
    fs::create_dir_all(path.parent().ok_or_else(||io::Error::other("Audio settings parent unavailable"))?)?;
    let temp=path.with_extension(format!("{}-{}.tmp",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(io::Error::other)?.as_nanos()));
    let result=(||{let mut f=fs::OpenOptions::new().write(true).create_new(true).open(&temp)?;f.write_all(&serde_json::to_vec(&value).map_err(io::Error::other)?)?;f.sync_all()?;drop(f);fs::rename(&temp,path)})();
    if result.is_err(){let _=fs::remove_file(temp);}result
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum GraphicsPreset {
    // Newly unconfigured profiles retain every Minecraft graphics option.
    // Previously saved Low and High values keep their original meaning.
    #[default]
    Preserve,
    High,
    Low,
}

pub fn load_graphics() -> io::Result<[GraphicsPreset; 5]> {
    read_graphics(&performance_path()?.with_file_name("graphics.json"))
}
fn read_graphics(path: &std::path::Path) -> io::Result<[GraphicsPreset; 5]> {
    match fs::symlink_metadata(&path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok([GraphicsPreset::Preserve; 5]),
        Err(error) => return Err(error),
        Ok(metadata) if !metadata.is_file() || metadata.file_type().is_symlink() => return Err(io::Error::other("Graphics settings must be an ordinary file")),
        Ok(_) => {}
    }
    let mut bytes = Vec::new();
    fs::File::open(path)?.take(4097).read_to_end(&mut bytes)?;
    if bytes.len() > 4096 { return Err(io::Error::other("Graphics settings file is too large")); }
    serde_json::from_slice(&bytes).map_err(io::Error::other)
}

pub fn save_graphics(values: [GraphicsPreset; 5]) -> io::Result<()> {
    write_graphics(&performance_path()?.with_file_name("graphics.json"), values)
}
fn write_graphics(path: &std::path::Path, values: [GraphicsPreset; 5]) -> io::Result<()> {
    use std::io::Write;
    read_graphics(path)?;
    let temp = path.with_extension(format!("{}-{}.tmp", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(io::Error::other)?.as_nanos()));
    let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&temp)?;
    let result = (|| { file.write_all(&serde_json::to_vec(&values).map_err(io::Error::other)?)?; file.sync_all()?; drop(file); fs::rename(&temp, &path) })();
    if result.is_err() { let _ = fs::remove_file(temp); }
    result
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PerformanceSettings {
    pub active_fps: u32,
    pub background_fps: u32,
}
impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            active_fps: 120,
            background_fps: 5,
        }
    }
}
impl PerformanceSettings {
    pub fn validate(&self) -> io::Result<()> {
        if !(5..=1000).contains(&self.active_fps)
            || !(5..=self.active_fps).contains(&self.background_fps)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Use whole FPS limits from 5 to 1000. Background FPS must not exceed active FPS.",
            ));
        }
        Ok(())
    }
}
pub(crate) fn application_root(executable: &std::path::Path) -> io::Result<PathBuf> {
    let folder=executable.parent().ok_or_else(||io::Error::other("Application folder unavailable"))?;
    if !folder.file_name().is_some_and(|name|name.eq_ignore_ascii_case("App")) {
        for ancestor in folder.ancestors(){if ancestor.join("Development/package.json").is_file()&&ancestor.join("App").is_dir(){return Ok(ancestor.to_path_buf());}}
    }
    Ok(if folder.file_name().is_some_and(|name|name.eq_ignore_ascii_case("App")) {
        folder.parent().ok_or_else(||io::Error::other("Project folder unavailable"))?
    } else {folder}.to_path_buf())
}
pub(crate) fn product_settings() -> io::Result<PathBuf> {
    product_directory("Settings")
}
pub(crate) fn product_cache() -> io::Result<PathBuf> {
    product_directory("Cache")
}
fn product_directory(name:&str)->io::Result<PathBuf> {
    let directory=application_root(&std::env::current_exe()?)?.join("Data").join(name);
    for path in directory.ancestors() {
        if !path.try_exists()? {continue;}
        let metadata=fs::symlink_metadata(path)?;
        #[cfg(windows)] {use std::os::windows::fs::MetadataExt;if metadata.file_attributes()&0x400!=0{return Err(io::Error::other("Settings path must not contain links"));}}
        if !metadata.is_dir()||metadata.file_type().is_symlink(){return Err(io::Error::other("Settings path must contain ordinary directories"));}
    }
    Ok(directory)
}
fn performance_path() -> io::Result<PathBuf> {
    let directory=product_settings()?;
    fs::create_dir_all(&directory)?;
    Ok(directory.join("performance.json"))
}
pub fn load_performance() -> io::Result<PerformanceSettings> {
    read_performance(&performance_path()?)
}
pub fn save_performance(value: PerformanceSettings) -> io::Result<()> {
    write_performance(&performance_path()?, value)
}
fn read_performance(path: &std::path::Path) -> io::Result<PerformanceSettings> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(PerformanceSettings::default())
        }
        Err(error) => return Err(error),
        Ok(metadata) if !metadata.is_file() || metadata.file_type().is_symlink() => {
            return Err(io::Error::other(
                "Performance settings must be an ordinary file",
            ))
        }
        Ok(_) => {}
    }
    let mut bytes = Vec::new();
    fs::File::open(path)?.take(4097).read_to_end(&mut bytes)?;
    if bytes.len() > 4096 {
        return Err(io::Error::other("Performance settings file is too large"));
    }
    let value: PerformanceSettings = serde_json::from_slice(&bytes).map_err(|_| {
        io::Error::other("Performance settings are invalid; the saved file was preserved")
    })?;
    // Retain older explicit 1..4 FPS preferences; only newly saved settings require 5 FPS.
    if !(1..=1000).contains(&value.active_fps) || !(1..=value.active_fps).contains(&value.background_fps) {
        return Err(io::Error::other("Saved FPS limits are invalid; the saved file was preserved"));
    }
    Ok(value)
}
fn write_performance(path: &std::path::Path, value: PerformanceSettings) -> io::Result<()> {
    use std::io::Write;
    value.validate()?;
    read_performance(path)?; // Never silently replace corrupt settings or a link.
    let temp = path.with_extension(format!(
        "{}-{}.tmp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_nanos()
    ));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)?;
    let result = (|| {
        file.write_all(&serde_json::to_vec_pretty(&value).map_err(io::Error::other)?)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temp, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

#[cfg(test)]
mod tests {
    #[test]
    fn audio_roundtrip_rejects_unknown_and_preserves_bad_preferences() {
        use super::*;
        let root=std::env::temp_dir().join(format!("lazy-audio-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir(&root).unwrap();let path=root.join("audio.json");
        assert_eq!(read_audio(&path).unwrap(),BackgroundAudio::Keep);
        write_audio(&path,BackgroundAudio::Mute).unwrap();assert_eq!(read_audio(&path).unwrap(),BackgroundAudio::Mute);
        write_audio(&path,BackgroundAudio::Keep).unwrap();assert_eq!(read_audio(&path).unwrap(),BackgroundAudio::Keep);
        fs::write(&path,b"\"Other\"").unwrap();assert!(write_audio(&path,BackgroundAudio::Keep).is_err());
        assert_eq!(fs::read(&path).unwrap(),b"\"Other\"");
        assert!(root.starts_with(std::env::temp_dir()));fs::remove_dir_all(root).unwrap();
    }
    use super::*;
    #[test]
    fn graphics_preferences_are_bounded_atomic_and_preserve_bad_inputs() {
        let root=std::env::temp_dir().join(format!("lazy-graphics-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir(&root).unwrap();let path=root.join("graphics.json");
        assert_eq!(read_graphics(&path).unwrap(),[GraphicsPreset::Preserve;5]);
        assert!(!path.exists(), "Unconfigured profiles must not write graphics.json");
        let values=[GraphicsPreset::High,GraphicsPreset::Low,GraphicsPreset::Preserve,GraphicsPreset::Low,GraphicsPreset::High];
        write_graphics(&path,values).unwrap();assert_eq!(read_graphics(&path).unwrap(),values);
        // Existing five-choice files from older LazyTester remain valid and unchanged.
        fs::write(&path,b"[\"Low\",\"High\",\"Low\",\"Low\",\"High\"]").unwrap();
        assert_eq!(read_graphics(&path).unwrap(),[GraphicsPreset::Low,GraphicsPreset::High,GraphicsPreset::Low,GraphicsPreset::Low,GraphicsPreset::High]);
        for bytes in [b"[]".as_slice(),b"[\"Auto\",\"Low\",\"Low\",\"Low\",\"Low\"]",b"invalid"] {
            fs::write(&path,bytes).unwrap();assert!(write_graphics(&path,values).is_err());assert_eq!(fs::read(&path).unwrap(),bytes);
        }
        fs::remove_file(&path).unwrap();fs::remove_dir(&root).unwrap();
    }
    #[test]
    fn performance_defaults_persist_validate_and_preserve_invalid_files() {
        let root=std::env::temp_dir().join(format!("lazy-performance-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir(&root).unwrap();let path=root.join("performance.json");
        assert_eq!(read_performance(&path).unwrap(),PerformanceSettings::default());
        assert_eq!(PerformanceSettings::default(),PerformanceSettings {active_fps:120,background_fps:5});
        assert!(!path.exists());
        let custom=PerformanceSettings {active_fps:60,background_fps:5};
        write_performance(&path,custom).unwrap();assert_eq!(read_performance(&path).unwrap(),custom);
        for value in [PerformanceSettings {active_fps:4,background_fps:4}, PerformanceSettings {active_fps:120,background_fps:4}, PerformanceSettings {active_fps:0,background_fps:1},PerformanceSettings {active_fps:1001,background_fps:10},PerformanceSettings {active_fps:60,background_fps:61}] {
            assert!(write_performance(&path,value).is_err());assert_eq!(read_performance(&path).unwrap(),custom);
        }
        fs::write(&path,br#"{"activeFps":144,"backgroundFps":10}"#).unwrap();
        assert_eq!(read_performance(&path).unwrap(),PerformanceSettings {active_fps:144,background_fps:10});
        let legacy=br#"{"activeFps":60,"backgroundFps":1}"#;
        fs::write(&path,legacy).unwrap();
        assert_eq!(read_performance(&path).unwrap().background_fps,1);
        assert_eq!(fs::read(&path).unwrap(),legacy);
        write_performance(&path,PerformanceSettings {active_fps:5,background_fps:5}).unwrap();
        assert_eq!(read_performance(&path).unwrap().active_fps,5);
        fs::write(&path,b"invalid").unwrap();
        assert!(read_performance(&path).is_err());assert!(write_performance(&path,PerformanceSettings::default()).is_err());assert_eq!(fs::read(&path).unwrap(),b"invalid");
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn workspace_errors_remain_structured_without_masking_engine_failures() {
        let geometry = OperationFailure::from(io::Error::other(
            "Workspace geometry mismatch: requested vs observed",
        ));
        assert_eq!(geometry.code, "WORKSPACE_UNAVAILABLE");
        assert!(!geometry.message.contains("requested"));
        assert!(geometry.details.contains("requested"));
        let engine = OperationFailure::from(io::Error::other(
            "payload hash mismatch in workspace/owned-tests/example",
        ));
        assert_eq!(engine.code, "ENGINE_UNAVAILABLE");
        let timeout = OperationFailure::from(io::Error::other(
            "Surface helper timed out after 3000 ms at place/position",
        ));
        assert_eq!(timeout.message, "Minecraft display stopped responding.");
        assert!(timeout.action.contains("Retry display"));
        let covered = OperationFailure::from(io::Error::other(
            "Workspace game is covered; presentation was not confirmed",
        ));
        assert_eq!(covered.message, "Minecraft is hidden behind the manager.");
    }
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationFailure {
    pub code: &'static str,
    pub message: &'static str,
    pub action: &'static str,
    pub details: String,
}
impl From<std::io::Error> for OperationFailure {
    fn from(error: std::io::Error) -> Self {
        let details = error.to_string();
        let (code, message, action) = if details.contains("Recent launch")
            || details.contains("Previous private launcher")
        {
            (
                "LAUNCH_PENDING",
                "A previous launch is still pending.",
                "Check again before retrying; at least 30 seconds must pass.",
            )
        } else if details.contains("Minecraft")
            || details.contains("executable")
            || details.contains("installation")
        {
            (
                "GAME_TARGET_INVALID",
                "Minecraft could not be opened or verified.",
                "Make sure Minecraft Education Desktop is installed, then check again.",
            )
        } else if ["payload", "driver", "service", "ABI", "signature", "hash"]
            .iter()
            .any(|word| details.contains(word))
        {
            (
                "ENGINE_UNAVAILABLE",
                "The engine could not be verified.",
                "Check again. If this continues, open Settings Diagnostics for the reason.",
            )
        } else if details.to_ascii_lowercase().contains("surface helper")
            && details.contains("timed out")
        {
            ("WORKSPACE_UNAVAILABLE", "Minecraft display stopped responding.", "Choose Retry display. If it still fails, use Restore window. Diagnostics includes the last helper stage.")
        } else if details.contains("Workspace game is covered") {
            (
                "WORKSPACE_UNAVAILABLE",
                "Minecraft is hidden behind the manager.",
                "Choose Retry display to show it again, or Restore window to return it to Windows.",
            )
        } else if ["workspace", "surface", "game area", "window recovery"]
            .iter()
            .any(|word| details.to_ascii_lowercase().contains(word))
        {
            ("WORKSPACE_UNAVAILABLE","The workspace needs attention.","Restore the game window or enlarge the manager, then try again. See Diagnostics for details.")
        } else {
            (
                "OPERATION_REJECTED",
                "The operation could not be completed.",
                "Check again for a fresh status. See Diagnostics for details.",
            )
        };
        Self {
            code,
            message,
            action,
            details,
        }
    }
}
