#![windows_subsystem = "windows"]

use std::{
    ffi::OsStr,
    io,
    os::windows::{ffi::OsStrExt, fs::MetadataExt, process::CommandExt},
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

#[link(name = "user32")]
extern "system" {
    fn MessageBoxW(
        window: *mut core::ffi::c_void,
        text: *const u16,
        title: *const u16,
        flags: u32,
    ) -> i32;
}

fn target(root: &Path) -> io::Result<PathBuf> {
    let app = root.join("App");
    let executable = app.join("lazy-tester.exe");
    for (path, directory) in [(&app, true), (&executable, false)] {
        let metadata = std::fs::symlink_metadata(path)?;
        if metadata.file_attributes() & 0x400 != 0
            || metadata.is_dir() != directory
            || (!directory && !metadata.is_file())
        {
            return Err(io::Error::other(
                "The application path must contain ordinary folders and files.",
            ));
        }
    }
    Ok(executable)
}

fn run() -> io::Result<i32> {
    let own = std::env::current_exe()?;
    let root = own
        .parent()
        .ok_or_else(|| io::Error::other("Project folder unavailable"))?;
    let target = target(root)?;
    let arguments = std::env::args_os().skip(1).collect::<Vec<_>>();
    if arguments == [OsStr::new("--check-target")] {
        return Ok(0);
    }
    if !arguments.is_empty() {
        return Err(io::Error::other(
            "Open Lazy Tester without command-line arguments.",
        ));
    }
    // Keep only a small waiting launcher; all application/session authority stays in App.
    let status = Command::new(&target)
        .current_dir(target.parent().unwrap())
        .creation_flags(0x08000000)
        .spawn()?
        .wait()?;
    Ok(status.code().unwrap_or(1))
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code.clamp(0, 255) as u8),
        Err(error) => {
            if std::env::args_os().len() == 1 {
                let text: Vec<u16> = format!("Lazy Tester could not start.\nKeep this executable beside the App folder.\n\n{error}").encode_utf16().chain([0]).collect();
                let title: Vec<u16> = OsStr::new("Lazy Tester").encode_wide().chain([0]).collect();
                unsafe {
                    MessageBoxW(std::ptr::null_mut(), text.as_ptr(), title.as_ptr(), 0x10);
                }
            }
            ExitCode::FAILURE
        }
    }
}
