use serde::{Deserialize, Serialize};
use std::{fs, io, io::Read, path::PathBuf};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum BackgroundAudio { #[default] Keep, Mute }
pub fn load_audio()->io::Result<BackgroundAudio> {read_audio(&product_settings()?.join("audio.json"))}
fn read_audio(path:&std::path::Path)->io::Result<BackgroundAudio> {
    Ok(read_saved_audio(path)?.unwrap_or_default())
}
fn read_saved_audio(path: &std::path::Path) -> io::Result<Option<BackgroundAudio>> {
    match fs::symlink_metadata(path) {
        Err(e) if e.kind()==io::ErrorKind::NotFound=>return Ok(None),
        Err(e)=>return Err(e),
        Ok(m) if !m.is_file()||m.file_type().is_symlink()=>return Err(io::Error::other("Audio settings must be an ordinary file")),
        Ok(_)=>{}
    }
    let mut bytes=Vec::new();fs::File::open(path)?.take(65).read_to_end(&mut bytes)?;
    if bytes.len()>64{return Err(io::Error::other("Audio settings are too large"));}
    serde_json::from_slice(&bytes).map(Some).map_err(io::Error::other)
}
pub fn save_audio(value:BackgroundAudio)->io::Result<()> {write_audio(&product_settings()?.join("audio.json"),value)}
fn write_audio(path:&std::path::Path,value:BackgroundAudio)->io::Result<()> {
    use std::io::Write;
    // A missing file still permits an explicit first save of the default.
    if read_saved_audio(path)? == Some(value) { return Ok(()); }
    fs::create_dir_all(path.parent().ok_or_else(||io::Error::other("Audio settings parent unavailable"))?)?;
    let temp=path.with_extension(format!("{}-{}.tmp",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(io::Error::other)?.as_nanos()));
    let result=(||{let mut f=fs::OpenOptions::new().write(true).create_new(true).open(&temp)?;f.write_all(&serde_json::to_vec(&value).map_err(io::Error::other)?)?;f.sync_all()?;drop(f);fs::rename(&temp,path)})();
    if result.is_err(){let _=fs::remove_file(temp);}result
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum GraphicsPreset {
    // Retained wire vocabulary for legacy requests/reports. Active policy is Low.
    #[default]
    Preserve,
    High,
    Low,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PerformanceSettings {
    pub active_fps: u32,
    pub background_fps: u32,
}
impl Default for PerformanceSettings {
    fn default() -> Self {
        // First-run only: match the reviewed native frame pacer. Saved values win.
        Self {
            active_fps: 144,
            background_fps: 10,
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
    let folder = executable.parent().ok_or_else(|| io::Error::other("Application folder unavailable"))?;
    // Installed and prepared applications retain their original App/../Data
    // contract. Never infer the root from a process working directory.
    if folder.file_name().is_some_and(|name| name.eq_ignore_ascii_case("App")) {
        return Ok(folder.parent().ok_or_else(|| io::Error::other("Project folder unavailable"))?.to_path_buf());
    }
    for ancestor in folder.ancestors() {
        if !ancestor.join("Development/package.json").is_file() { continue; }
        // Check the official dev output BEFORE the staged App/ shortcut.
        // Once App/ exists, an older build-local Data must still block a
        // silent switch of profile/account/world roots.
        // Build output is never a second project Data/ authority.
        let source_checkout = ancestor.join("DEV.cmd").is_file()
            && ancestor.join("Development/apps/desktop/src-tauri/tauri.lazy-tester.conf.json").is_file();
        let developer_owned_target =
            folder.starts_with(ancestor.join("workspace/builds/desktop/target"))
            || folder.starts_with(ancestor.join("Development/apps/desktop/src-tauri/target"));
        if source_checkout && developer_owned_target {
            // Older dev runs might have placed private data beside the
            // executable. Refuse a silent root switch; no copy/deletion.
            match fs::symlink_metadata(folder.join("Data")) {
                Ok(_) => return Err(io::Error::new(io::ErrorKind::AlreadyExists,
                    "Legacy developer data exists under the build output. Preserve it and review the root before continuing.")),
                Err(error) if error.kind()==io::ErrorKind::NotFound => {},
                Err(error) => return Err(error),
            }
            return Ok(ancestor.to_path_buf());
        }
        // Existing staged source layouts that were not built at the official
        // dev target retain their previous checkout-root resolution.
        if ancestor.join("App").is_dir() { return Ok(ancestor.to_path_buf()); }
    }
    // Preserve the non-project / legacy flat-install fallback as before.
    Ok(folder.to_path_buf())
}
pub(crate) fn product_settings() -> io::Result<PathBuf> {
    product_directory("Settings")
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StoragePreferences {
    pub auto_clear_web_cache: bool,
    #[serde(default)]
    pub auto_clear_download_cache: bool,
    #[serde(default)]
    pub auto_clear_old_logs: bool,
}

// Explicit native acceptance gates; a preference is never producer evidence.
pub(crate) const DOWNLOAD_CACHE_REVIEWED: bool = false;
pub(crate) const OLD_LOGS_REVIEWED: bool = false;
impl StoragePreferences {
    fn validate(&self) -> io::Result<()> {
        if self.auto_clear_download_cache && !DOWNLOAD_CACHE_REVIEWED || self.auto_clear_old_logs && !OLD_LOGS_REVIEWED {
            return Err(io::Error::new(io::ErrorKind::Unsupported,"Download cache or old log cleanup is not native-verified"));
        }
        Ok(())
    }
    pub(super) fn has_maintenance(&self) -> bool {
        self.auto_clear_web_cache || (DOWNLOAD_CACHE_REVIEWED && self.auto_clear_download_cache) || (OLD_LOGS_REVIEWED && self.auto_clear_old_logs)
    }
}
impl Default for StoragePreferences {
    fn default() -> Self { Self { auto_clear_web_cache: true, auto_clear_download_cache: false, auto_clear_old_logs: false } }
}
pub fn load_storage_preferences() -> io::Result<StoragePreferences> {
    read_storage_preferences(&product_settings()?.join("storage.json"))
}
pub fn save_storage_preferences(value: StoragePreferences) -> io::Result<()> {
    let directory=product_settings()?;
    fs::create_dir_all(&directory)?;
    write_storage_preferences(&product_settings()?.join("storage.json"),value)
}
fn read_storage_preferences(path:&std::path::Path)->io::Result<StoragePreferences> {
    Ok(read_saved_storage_preferences(path)?.unwrap_or_default())
}
fn read_saved_storage_preferences(path: &std::path::Path) -> io::Result<Option<StoragePreferences>> {
    match fs::symlink_metadata(path) {
        Err(e) if e.kind()==io::ErrorKind::NotFound=>return Ok(None),
        Err(e)=>return Err(e),
        Ok(m)=>{
            if !m.is_file()||m.file_type().is_symlink(){return Err(io::Error::other("Storage settings must be an ordinary file"));}
            #[cfg(windows)] { use std::os::windows::fs::MetadataExt; if m.file_attributes()&0x400!=0{return Err(io::Error::other("Linked storage settings"));} }
        }
    }
    let mut bytes=Vec::new();
    let mut options=fs::OpenOptions::new();
    options.read(true);
    #[cfg(windows)] { use std::os::windows::fs::{OpenOptionsExt,MetadataExt}; options.custom_flags(0x00200000); }
    let file=options.open(path)?;
    let meta=file.metadata()?;
    if !meta.is_file(){return Err(io::Error::other("Storage settings must be an ordinary file"));}
    #[cfg(windows)] {use std::os::windows::fs::MetadataExt; if meta.file_attributes()&0x400!=0{return Err(io::Error::other("Linked storage settings"));}}
    file.take(4097).read_to_end(&mut bytes)?;
    if bytes.len()>4096{return Err(io::Error::other("Storage settings are too large"));}
    let value: StoragePreferences=serde_json::from_slice(&bytes).map_err(|_|io::Error::other("Storage settings invalid; saved file preserved"))?;
    value.validate()?;
    Ok(Some(value))
}
fn write_storage_preferences(path:&std::path::Path,value:StoragePreferences)->io::Result<()> {
    use std::io::Write;
    value.validate()?;
    if read_saved_storage_preferences(path)? == Some(value) { return Ok(()); }
    let temp=path.with_extension(format!("{}-{}.tmp",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(io::Error::other)?.as_nanos()));
    let mut file=fs::OpenOptions::new().write(true).create_new(true).open(&temp)?;
    let result=(||{file.write_all(&serde_json::to_vec(&value).map_err(io::Error::other)?)?;file.sync_all()?;drop(file);read_storage_preferences(path)?;fs::rename(&temp,path)})();
    if result.is_err(){let _=fs::remove_file(&temp);}
    result
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
// One freshly checked directory per report, not a persistent settings cache.
// Preserve the existing directory creation and individual file validation.
pub(super) fn load_report_preferences() -> io::Result<(PerformanceSettings, BackgroundAudio, [GraphicsPreset; 5], Option<StoragePreferences>)> {
    let directory = product_settings()?;
    fs::create_dir_all(&directory)?;
    read_report_preferences(&directory)
}
fn read_report_preferences(directory: &std::path::Path) -> io::Result<(PerformanceSettings, BackgroundAudio, [GraphicsPreset; 5], Option<StoragePreferences>)> {
    let performance = read_performance(&directory.join("performance.json"))?;
    let audio = read_audio(&directory.join("audio.json"))?;
    // Same report-local checked directory; storage failure stays optional.
    let storage = read_storage_preferences(&directory.join("storage.json")).ok();
    // Compatibility projection of the global policy, not saved per-slot choices.
    let graphics = [GraphicsPreset::Low; 5];
    Ok((performance, audio, graphics, storage))
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
    Ok(read_saved_performance(path)?.unwrap_or_default())
}
fn read_saved_performance(path: &std::path::Path) -> io::Result<Option<PerformanceSettings>> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
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
    Ok(Some(value))
}
fn write_performance(path: &std::path::Path, value: PerformanceSettings) -> io::Result<()> {
    use std::io::Write;
    value.validate()?;
    // Validate even an equal request; corrupt settings and links remain refused.
    if read_saved_performance(path)? == Some(value) { return Ok(()); }
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
    fn developer_root_without_app_stays_at_checkout_and_preserves_legacy_data() {
        use super::*;
        let root=std::env::temp_dir().join(format!("lazy-root-fixture-{}-{}",
            std::process::id(),std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let source=root.join("Development/apps/desktop/src-tauri");
        let target=root.join("workspace/builds/desktop/target/x86_64-pc-windows-msvc/debug");
        fs::create_dir_all(&target).unwrap();
        fs::create_dir_all(&source).unwrap();
        fs::write(root.join("Development/package.json"),b"{}").unwrap();
        fs::write(root.join("DEV.cmd"),b"").unwrap();
        fs::write(source.join("tauri.lazy-tester.conf.json"),b"{}").unwrap();
        let exe=target.join("lazy-tester.exe");
        assert!(!root.join("App").exists());
        assert_eq!(application_root(&exe).unwrap(),root);
        // Existing build-local worlds must never be silently hidden or moved.
        let old_data=target.join("Data");
        fs::create_dir(&old_data).unwrap();
        fs::write(old_data.join("preserved-world"),b"keep").unwrap();
        assert_eq!(application_root(&exe).unwrap_err().kind(),io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read(old_data.join("preserved-world")).unwrap(),b"keep");
        fs::remove_dir_all(&old_data).unwrap();
        let default_target=source.join("target/debug");
        fs::create_dir_all(&default_target).unwrap();
        assert_eq!(application_root(&default_target.join("lazy-tester.exe")).unwrap(),root);
        let staged=root.join("App");
        fs::create_dir(&staged).unwrap();
        assert_eq!(application_root(&staged.join("lazy-tester.exe")).unwrap(),root);
        assert_eq!(application_root(&exe).unwrap(),root);
        // Recreating legacy dev-local data AFTER App staging still blocks
        // accidental root switching and preserves the existing bytes.
        fs::create_dir(&old_data).unwrap();
        fs::write(old_data.join("preserved-world"),b"keep-again").unwrap();
        assert_eq!(application_root(&exe).unwrap_err().kind(),io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read(old_data.join("preserved-world")).unwrap(),b"keep-again");
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn audio_roundtrip_rejects_unknown_and_preserves_bad_preferences() {
        use super::*;
        let root=std::env::temp_dir().join(format!("lazy-audio-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir(&root).unwrap();let path=root.join("audio.json");
        assert_eq!(read_audio(&path).unwrap(),BackgroundAudio::Keep);
        assert!(!path.exists());
        write_audio(&path,BackgroundAudio::Keep).unwrap();
        assert!(path.is_file());
        let same=b" \"Keep\"\n";
        fs::write(&path,same).unwrap();
        write_audio(&path,BackgroundAudio::Keep).unwrap();
        assert_eq!(fs::read(&path).unwrap(),same);
        write_audio(&path,BackgroundAudio::Mute).unwrap();assert_eq!(read_audio(&path).unwrap(),BackgroundAudio::Mute);
        write_audio(&path,BackgroundAudio::Keep).unwrap();assert_eq!(read_audio(&path).unwrap(),BackgroundAudio::Keep);
        fs::write(&path,b"\"Other\"").unwrap();assert!(write_audio(&path,BackgroundAudio::Keep).is_err());
        assert_eq!(fs::read(&path).unwrap(),b"\"Other\"");
        assert!(root.starts_with(std::env::temp_dir()));fs::remove_dir_all(root).unwrap();
    }
    use super::*;
    #[test]
    fn storage_preferences_default_roundtrip_and_preserve_invalid_file() {
        let root=std::env::temp_dir().join(format!("lazy-storage-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir(&root).unwrap();let path=root.join("storage.json");
        assert!(read_storage_preferences(&path).unwrap().auto_clear_web_cache);
        assert!(!path.exists());
        write_storage_preferences(&path,StoragePreferences::default()).unwrap();
        assert!(path.is_file());
        let disabled=StoragePreferences{auto_clear_web_cache:false,..StoragePreferences::default()};
        assert!(!disabled.has_maintenance());
        fs::write(&path,br#"{"autoClearWebCache":false}"#).unwrap();
        assert_eq!(read_storage_preferences(&path).unwrap(),disabled);
        assert!(write_storage_preferences(&path,StoragePreferences{auto_clear_old_logs:true,..disabled}).is_err());
        assert!(write_storage_preferences(&path,StoragePreferences{auto_clear_download_cache:true,..disabled}).is_err());
        write_storage_preferences(&path,disabled).unwrap();
        assert_eq!(fs::read(&path).unwrap(),br#"{"autoClearWebCache":false}"#);
        assert_eq!(read_storage_preferences(&path).unwrap(),disabled);
        write_storage_preferences(&path,StoragePreferences::default()).unwrap();
        assert!(read_storage_preferences(&path).unwrap().auto_clear_web_cache);
        let invalid=br#"{"autoClearWebCache":true,"autoClearDownloadCache":true}"#;
        fs::write(&path,invalid).unwrap();
        assert!(write_storage_preferences(&path,disabled).is_err());
        assert_eq!(fs::read(&path).unwrap(),invalid);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn performance_defaults_persist_validate_and_preserve_invalid_files() {
        let root=std::env::temp_dir().join(format!("lazy-performance-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir(&root).unwrap();let path=root.join("performance.json");
        assert_eq!(read_performance(&path).unwrap(),PerformanceSettings::default());
        assert_eq!(PerformanceSettings::default(),PerformanceSettings {active_fps:144,background_fps:10});
        assert!(!path.exists());
        write_performance(&path,PerformanceSettings::default()).unwrap();
        assert!(path.is_file());
        let custom=PerformanceSettings {active_fps:60,background_fps:5};
        write_performance(&path,custom).unwrap();assert_eq!(read_performance(&path).unwrap(),custom);
        for value in [PerformanceSettings {active_fps:4,background_fps:4}, PerformanceSettings {active_fps:120,background_fps:4}, PerformanceSettings {active_fps:0,background_fps:1},PerformanceSettings {active_fps:1001,background_fps:10},PerformanceSettings {active_fps:60,background_fps:61}] {
            assert!(write_performance(&path,value).is_err());assert_eq!(read_performance(&path).unwrap(),custom);
        }
        let same=br#"{"activeFps":144,"backgroundFps":10}"#;
        fs::write(&path,same).unwrap();
        write_performance(&path,PerformanceSettings::default()).unwrap();
        assert_eq!(fs::read(&path).unwrap(),same);
        assert_eq!(read_performance(&path).unwrap(),PerformanceSettings {active_fps:144,background_fps:10});
        let legacy=br#"{"activeFps":60,"backgroundFps":1}"#;
        fs::write(&path,legacy).unwrap();
        assert_eq!(read_performance(&path).unwrap().background_fps,1);
        assert!(write_performance(&path,PerformanceSettings {active_fps:60,background_fps:1}).is_err());
        assert_eq!(fs::read(&path).unwrap(),legacy);
        write_performance(&path,PerformanceSettings {active_fps:5,background_fps:5}).unwrap();
        assert_eq!(read_performance(&path).unwrap().active_fps,5);
        fs::write(&path,b"invalid").unwrap();
        assert!(read_performance(&path).is_err());assert!(write_performance(&path,PerformanceSettings::default()).is_err());assert_eq!(fs::read(&path).unwrap(),b"invalid");
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn report_preferences_read_current_files_and_preserve_invalid_inputs() {
        let root = std::env::temp_dir().join(format!("lazy-report-preferences-{}-{}",
            std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir(&root).unwrap();
        assert_eq!(read_report_preferences(&root).unwrap(),
            (PerformanceSettings::default(), BackgroundAudio::Keep, [GraphicsPreset::Low; 5], Some(StoragePreferences::default())));
        let limits = PerformanceSettings {active_fps: 60, background_fps: 5};
        let graphics = [GraphicsPreset::Low; 5];
        write_performance(&root.join("performance.json"), limits).unwrap();
        write_audio(&root.join("audio.json"), BackgroundAudio::Mute).unwrap();
        // Historical mixed preferences are inert, not migrated or overwritten.
        fs::write(root.join("graphics.json"), br#"["High","Preserve","Low","High","Preserve"]"#).unwrap();
        let old_bytes = fs::read(root.join("graphics.json")).unwrap();
        assert_eq!(read_report_preferences(&root).unwrap(), (limits, BackgroundAudio::Mute, graphics, Some(StoragePreferences::default())));
        assert_eq!(fs::read(root.join("graphics.json")).unwrap(), old_bytes);
        fs::write(root.join("graphics.json"), b"invalid").unwrap();
        assert_eq!(read_report_preferences(&root).unwrap(), (limits, BackgroundAudio::Mute, graphics, Some(StoragePreferences::default())));
        assert_eq!(fs::read(root.join("graphics.json")).unwrap(), b"invalid");
        let storage=root.join("storage.json");
        let disabled=StoragePreferences {auto_clear_web_cache:false,..StoragePreferences::default()};
        write_storage_preferences(&storage,disabled).unwrap();
        assert_eq!(read_report_preferences(&root).unwrap(), (limits,BackgroundAudio::Mute,graphics,Some(disabled)));
        fs::write(&storage,b"invalid").unwrap();
        assert_eq!(read_report_preferences(&root).unwrap(), (limits,BackgroundAudio::Mute,graphics,None));
        assert_eq!(fs::read(&storage).unwrap(),b"invalid");
        for name in ["performance.json", "audio.json"] {
            let path = root.join(name);
            let original = fs::read(&path).unwrap();
            fs::write(&path, b"invalid").unwrap();
            assert!(read_report_preferences(&root).is_err());
            assert_eq!(fs::read(&path).unwrap(), b"invalid");
            fs::write(&path, original).unwrap();
        }
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
