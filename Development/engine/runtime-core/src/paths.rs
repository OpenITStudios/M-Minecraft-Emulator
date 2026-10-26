use std::{env, io, path::PathBuf};

pub fn runtime_root() -> io::Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let root = env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotFound, "LOCALAPPDATA is unavailable")
            })?;
        return Ok(root.join("M-Bedrock").join("VirtualClients"));
    }

    #[cfg(target_os = "macos")]
    {
        let home = env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME is unavailable"))?;
        return Ok(home
            .join("Library")
            .join("Application Support")
            .join("M-Bedrock")
            .join("VirtualClients"));
    }

    #[allow(unreachable_code)]
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Virtual Clients supports Windows and macOS only",
    ))
}

pub fn base_version_root(version: &str) -> io::Result<PathBuf> {
    validate_version_segment(version)?;
    Ok(runtime_root()?.join("bases").join(version))
}

pub fn base_vmx_path_for_version(version: &str) -> io::Result<PathBuf> {
    let root = base_version_root(version)?;

    #[cfg(target_os = "windows")]
    {
        return Ok(root.join("Base").join("Base.vmx"));
    }

    #[cfg(target_os = "macos")]
    {
        return Ok(root.join("Base.vmwarevm").join("Base.vmx"));
    }

    #[allow(unreachable_code)]
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "unsupported platform",
    ))
}

pub fn base_profile_path_for_version(version: &str) -> io::Result<PathBuf> {
    Ok(base_version_root(version)?.join("base-profile.json"))
}

pub(crate) fn validate_version_segment(version: &str) -> io::Result<()> {
    if version.is_empty()
        || version.split('.').any(|segment| {
            segment.is_empty() || !segment.chars().all(|character| character.is_ascii_digit())
        })
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Minecraft version is not safe for a Base path",
        ));
    }
    Ok(())
}

pub fn client_root() -> io::Result<PathBuf> {
    Ok(runtime_root()?.join("clients"))
}

pub fn client_profile_path(client_name: &str) -> io::Result<PathBuf> {
    Ok(client_root()?.join(client_name).join("client-profile.json"))
}

pub fn staging_root() -> io::Result<PathBuf> {
    Ok(runtime_root()?.join("staging"))
}

pub fn update_staging_root() -> io::Result<PathBuf> {
    Ok(runtime_root()?.join("updates"))
}

#[cfg(test)]
mod tests {
    use super::validate_version_segment;

    #[test]
    fn version_path_segment_is_strict() {
        assert!(validate_version_segment("1.21.120.0").is_ok());
        assert!(validate_version_segment("1").is_ok());

        for invalid in ["", ".", "..", ".1", "1.", "1..2", "../bad", "1.21 preview"] {
            assert!(
                validate_version_segment(invalid).is_err(),
                "{invalid:?} must not be a Base path version"
            );
        }
    }
}
