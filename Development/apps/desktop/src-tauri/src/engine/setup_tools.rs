use std::{fs, io, path::Path, process::Command};
use virtual_clients_core::inspect_base_preparation;

pub fn open_base_location() -> io::Result<()> {
    let report = inspect_base_preparation()?;
    let vmx = report.base_expected_path.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Base location is unavailable until Minecraft Education is detected on this PC",
        )
    })?;
    let vmx = std::path::PathBuf::from(vmx);
    let folder = vmx.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "Base VM path does not have a parent directory",
        )
    })?;
    fs::create_dir_all(folder)?;
    open_folder(folder)
}

#[cfg(target_os = "windows")]
pub fn open_folder(path: &Path) -> io::Result<()> {
    Command::new("explorer.exe").arg(path).spawn()?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn open_folder(path: &Path) -> io::Result<()> {
    Command::new("open").arg(path).spawn()?;
    Ok(())
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn open_folder(_path: &Path) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "opening the Base location is unsupported on this platform",
    ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn helper_does_not_define_setup_state() {
        let source = include_str!("setup_tools.rs").split("#[cfg(test)]").next().unwrap();
        assert!(!source.contains("nextSetupAction"));
        assert!(!source.contains("FINALIZED ="));
    }
}
