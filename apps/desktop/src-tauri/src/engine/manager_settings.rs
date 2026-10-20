use serde::{Deserialize, Serialize};
use std::{fs, io, io::Read, path::PathBuf};

pub const TARGETS: &str =
    include_str!("../../../../../engine/isolation-native/owned-runtime/minecraft-targets.json");
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ManagerSettings {
    pub session_directory: String,
    pub minecraft_executable: String,
}
impl ManagerSettings {
    pub fn validate(&self) -> io::Result<()> {
        for value in [&self.session_directory, &self.minecraft_executable] {
            if value.len() > 2048
                || value.chars().any(char::is_control)
                || value.contains(['\'', '"'])
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Invalid settings path",
                ));
            }
            if !value.is_empty() && !std::path::Path::new(value).is_absolute() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "An absolute path is required",
                ));
            }
        }
        Ok(())
    }
}
fn path() -> io::Result<PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA")
        .ok_or_else(|| io::Error::other("Windows application data unavailable"))?;
    Ok(PathBuf::from(base).join("MinecraftInstanceManager/NativePilot/manager-settings.json"))
}
pub fn load() -> io::Result<ManagerSettings> {
    read(&path()?)
}
fn read(path: &std::path::Path) -> io::Result<ManagerSettings> {
    let bytes = fs::File::open(path).and_then(|file| {
        let mut bytes = Vec::new();
        file.take(16385).read_to_end(&mut bytes)?;
        Ok(bytes)
    });
    match bytes {
        Ok(bytes) if bytes.len() <= 16384 => {
            let value: ManagerSettings =
                serde_json::from_slice(&bytes).map_err(io::Error::other)?;
            value.validate()?;
            Ok(value)
        }
        Ok(_) => Err(io::Error::other("Settings file is too large")),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(ManagerSettings::default()),
        Err(e) => Err(e),
    }
}
pub fn save(value: &ManagerSettings) -> io::Result<()> {
    write(&path()?, value)
}
fn write(path: &std::path::Path, value: &ManagerSettings) -> io::Result<()> {
    value.validate()?;
    fs::create_dir_all(path.parent().unwrap())?;
    // Same manager lock serializes reads and writes; publish a complete file.
    let temporary = path.with_extension("tmp");
    fs::write(
        &temporary,
        serde_json::to_vec_pretty(value).map_err(io::Error::other)?,
    )?;
    fs::rename(temporary, path)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalog_has_four_unique_owned_routes() {
        let v: serde_json::Value = serde_json::from_str(TARGETS).unwrap();
        let targets = v["targets"].as_array().unwrap();
        assert_eq!(targets.len(), 4);
        for (i, target) in targets.iter().enumerate() {
            assert_eq!(target["id"], format!("{:02}", i + 1));
            if i > 0 {
                assert_eq!(target["provider"], "OWNED_V1186");
            }
        }
    }
    #[test]
    fn settings_roundtrip_and_changed_target_are_preferences_only() {
        let p = std::env::temp_dir().join(format!("manager-settings-{}.json", std::process::id()));
        let settings = ManagerSettings {
            session_directory: String::new(),
            minecraft_executable: std::env::current_exe()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
        };
        write(&p, &settings).unwrap();
        assert_eq!(
            read(&p).unwrap().minecraft_executable,
            settings.minecraft_executable
        );
        write(&p, &ManagerSettings::default()).unwrap();
        assert!(read(&p).unwrap().minecraft_executable.is_empty());
        fs::write(&p, b"{}").unwrap();
        assert!(read(&p).is_err());
        fs::remove_file(&p).unwrap();
        assert!(ManagerSettings {
            session_directory: "relative".into(),
            minecraft_executable: String::new()
        }
        .validate()
        .is_err());
    }
    #[test]
    #[ignore = "requires a real registered Minecraft Education Windows installation"]
    fn actual_registered_game_matches_native_launcher() {
        let path = super::super::native_clients::registered_education_target().unwrap();
        assert!(std::path::Path::new(&path).is_file());
        assert!(path.ends_with("Minecraft.Windows.exe"));
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
        let (code, message, action) =
            if details.contains("Recent launch") || details.contains("Previous private launcher") {
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
                    "Check the installed executable in Settings, then check the clients again.",
                )
            } else if ["payload", "driver", "service", "ABI", "signature", "hash"]
                .iter()
                .any(|word| details.contains(word))
            {
                (
                    "ENGINE_UNAVAILABLE",
                    "The selected engine could not be verified.",
                    "Check engine configuration in Settings. No fallback engine was used.",
                )
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
/// Preferences are saved only after the existing provider proves a detected engine.
pub fn prepare(requested: Option<ManagerSettings>) -> io::Result<serde_json::Value> {
    if let Some(value) = requested {
        save(&value)?;
    }
    let loaded = load();
    let mut settings = loaded
        .as_ref()
        .map(|s| ManagerSettings {
            session_directory: s.session_directory.clone(),
            minecraft_executable: s.minecraft_executable.clone(),
        })
        .unwrap_or_default();
    let detected = super::native_clients::registered_education_target();
    let target_error = loaded
        .err()
        .map(|e| format!("Saved preferences could not be read. {e}"))
        .or_else(|| detected.as_ref().err().map(|e| e.to_string()));
    let mut engine_error = None;
    if settings.session_directory.is_empty() {
        match super::owned_provider::discover_active_session() {
            Ok(path) => settings.session_directory = path.to_string_lossy().into_owned(),
            Err(error) => engine_error = Some(OperationFailure::from(error)),
        }
    }
    let mut observation = None;
    if !settings.session_directory.is_empty() && detected.is_ok() {
        let request = super::owned_provider::OwnedRequest {
            session_directory: settings.session_directory.clone(),
            client_id: "02".into(),
            operation: super::owned_provider::OwnedOperation::Inspect,
            licensed: false,
            compatibility_confirmed: false,
            target: None,
        };
        match (super::owned_provider::OwnedTestBackend { request: &request }).execute() {
            Ok(value) => {
                observation = Some(value);
                if settings.minecraft_executable.is_empty() {
                    settings.minecraft_executable = detected.as_ref().unwrap().clone();
                }
                save(&settings)?;
            }
            Err(error) => engine_error = Some(OperationFailure::from(error)),
        }
    }
    Ok(
        serde_json::json!({"settings":settings,"catalog":serde_json::from_str::<serde_json::Value>(TARGETS).map_err(io::Error::other)?,"detectedMinecraft":detected.ok(),"targetError":target_error,"engineError":engine_error,"observation":observation}),
    )
}
