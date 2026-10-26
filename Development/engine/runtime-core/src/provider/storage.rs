use crate::{client::ClientId, paths::staging_root};
use std::{ffi::OsStr, fs, io, path::Path};

pub(crate) fn vm_container(vmx: &Path) -> io::Result<&Path> {
    vmx.parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "VMX has no parent directory"))
}

pub(crate) fn ensure_parent(path: &Path) -> io::Result<()> {
    fs::create_dir_all(vm_container(path)?)
}

pub(crate) fn remove_vm_container(path: &Path) -> io::Result<()> {
    let container = vm_container(path)?;
    if container.exists() {
        fs::remove_dir_all(container)?;
    }
    Ok(())
}

pub(crate) fn promote_staging_vm(staging_vmx: &Path, final_vmx: &Path) -> io::Result<()> {
    if !staging_vmx.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "provider reported clone success but staged VMX is missing",
        ));
    }
    let staging = vm_container(staging_vmx)?;
    let final_dir = vm_container(final_vmx)?;
    if final_dir.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("client destination already exists: {}", final_dir.display()),
        ));
    }
    let parent = final_dir
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "client path has no parent"))?;
    fs::create_dir_all(parent)?;
    fs::rename(staging, final_dir)
}

// Filesystem-only cutover primitive. The caller must stage a complete VM,
// including its client-profile.json, and verify VMware disk references first.
// Retain backup on success for explicit recovery; never delete it implicitly.
#[allow(dead_code)]
pub(crate) fn cutover_staged_vm(
    staged_vmx: &Path,
    final_vmx: &Path,
    backup_dir: &Path,
) -> io::Result<()> {
    cutover_staged_vm_with_rename(staged_vmx, final_vmx, backup_dir, |from, to| {
        fs::rename(from, to)
    })
}

// Rename injection belongs to the existing filesystem owner, not a new
// lifecycle manager. It proves recovery when promotion fails after backup.
fn cutover_staged_vm_with_rename(
    staged_vmx: &Path,
    final_vmx: &Path,
    backup_dir: &Path,
    mut rename: impl FnMut(&Path, &Path) -> io::Result<()>,
) -> io::Result<()> {
    let staged_dir = vm_container(staged_vmx)?;
    let final_dir = vm_container(final_vmx)?;
    if !staged_vmx.is_file() || !staged_dir.join("client-profile.json").is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "staged VM and its client profile must both exist before cutover",
        ));
    }
    if !final_vmx.is_file() || !final_dir.join("client-profile.json").is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "original VM and its client profile must both exist before cutover",
        ));
    }
    if backup_dir.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "recovery backup already exists; manual reconciliation required",
        ));
    }
    if staged_dir.parent() != final_dir.parent() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "staged and final VM directories must share a filesystem parent",
        ));
    }
    if backup_dir.parent() != final_dir.parent() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "recovery backup must be beside the original VM",
        ));
    }
    rename(final_dir, backup_dir)?;
    if let Err(error) = rename(staged_dir, final_dir) {
        // Do not hide recovery failure: backup is intentionally preserved.
        match rename(backup_dir, final_dir) {
            Ok(()) => return Err(io::Error::new(error.kind(),
                format!("replacement promotion failed; original restored: {error}"))),
            Err(restore_error) => return Err(io::Error::new(error.kind(),
                format!("replacement promotion failed ({error}); original remains at recovery location, automatic restoration failed ({restore_error})"))),
        }
    }
    Ok(())
}

pub(crate) fn has_suspend_state(vmx: &Path) -> bool {
    let Ok(container) = vm_container(vmx) else {
        return false;
    };
    let Ok(entries) = fs::read_dir(container) else {
        return false;
    };
    entries.flatten().any(|entry| {
        entry
            .path()
            .extension()
            .and_then(OsStr::to_str)
            .is_some_and(|extension| extension.eq_ignore_ascii_case("vmss"))
    })
}

pub(crate) fn staging_residue_count() -> io::Result<usize> {
    let staging = staging_root()?;
    if !staging.exists() {
        return Ok(0);
    }
    Ok(fs::read_dir(staging)?.filter_map(Result::ok).count())
}

pub(crate) fn cleanup_staging() -> io::Result<()> {
    cleanup_staging_at(&staging_root()?)
}

fn cleanup_staging_at(staging: &Path) -> io::Result<()> {
    if !staging.exists() {
        return Ok(());
    }
    // Clone failures may leave disks in an indeterminate state. Never
    // recursively delete those artifacts just because a new operation started.
    if fs::read_dir(&staging)?.next().transpose()?.is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "staging contains unresolved VM artifacts; inspect and clear them manually before provisioning",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::cleanup_staging_at;
    use std::{
        fs, io,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn cutover_keeps_original_backup_with_profile() {
        let root = std::env::temp_dir().join(format!(
            "vc-cutover-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let old = root.join("Virtual-01");
        let stage = root.join("Virtual-01-new");
        let backup = root.join("Virtual-01-backup");
        fs::create_dir_all(&old).unwrap();
        fs::create_dir_all(&stage).unwrap();
        for (dir, value) in [(&old, "original"), (&stage, "replacement")] {
            fs::write(dir.join("Virtual-01.vmx"), value).unwrap();
            fs::write(dir.join("client-profile.json"), value).unwrap();
        }
        super::cutover_staged_vm(
            &stage.join("Virtual-01.vmx"),
            &old.join("Virtual-01.vmx"),
            &backup,
        )
        .unwrap();
        assert_eq!(
            fs::read_to_string(old.join("client-profile.json")).unwrap(),
            "replacement"
        );
        assert_eq!(
            fs::read_to_string(backup.join("client-profile.json")).unwrap(),
            "original"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn failed_promotion_restores_original_vm_and_profile() {
        let root = std::env::temp_dir().join(format!(
            "vc-cutover-fault-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let old = root.join("Virtual-01");
        let stage = root.join("Virtual-01-new");
        let backup = root.join("Virtual-01-backup");
        fs::create_dir_all(&old).unwrap();
        fs::create_dir_all(&stage).unwrap();
        for (dir, value) in [(&old, "old"), (&stage, "new")] {
            fs::write(dir.join("Virtual-01.vmx"), value).unwrap();
            fs::write(dir.join("client-profile.json"), value).unwrap();
        }
        let mut calls = 0;
        let error = super::cutover_staged_vm_with_rename(
            &stage.join("Virtual-01.vmx"),
            &old.join("Virtual-01.vmx"),
            &backup,
            |from, to| {
                calls += 1;
                if calls == 2 {
                    Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "injected promotion failure",
                    ))
                } else {
                    fs::rename(from, to)
                }
            },
        )
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(calls, 3);
        assert!(error.to_string().contains("original restored"));
        assert_eq!(
            fs::read_to_string(old.join("Virtual-01.vmx")).unwrap(),
            "old"
        );
        assert_eq!(
            fs::read_to_string(old.join("client-profile.json")).unwrap(),
            "old"
        );
        assert_eq!(
            fs::read_to_string(stage.join("client-profile.json")).unwrap(),
            "new"
        );
        assert!(!backup.exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn cutover_rejects_invalid_staging_without_touching_original() {
        let root = std::env::temp_dir().join(format!(
            "vc-cutover-invalid-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let old = root.join("Virtual-01");
        let stage = root.join("candidate");
        let backup = root.join("backup");
        fs::create_dir_all(&old).unwrap();
        fs::create_dir_all(&stage).unwrap();
        fs::write(old.join("Virtual-01.vmx"), "old").unwrap();
        fs::write(old.join("client-profile.json"), "old-profile").unwrap();
        fs::write(stage.join("Virtual-01.vmx"), "new").unwrap();
        assert!(super::cutover_staged_vm(
            &stage.join("Virtual-01.vmx"),
            &old.join("Virtual-01.vmx"),
            &backup
        )
        .is_err());
        assert_eq!(
            fs::read_to_string(old.join("client-profile.json")).unwrap(),
            "old-profile"
        );
        assert!(!backup.exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unresolved_staging_is_preserved() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "virtual-clients-staging-{}-{suffix}",
            std::process::id()
        ));
        let artifact = root.join("Virtual-01").join("Virtual-01.vmx");
        fs::create_dir_all(artifact.parent().unwrap()).unwrap();
        fs::write(&artifact, b"partial clone").unwrap();
        let error = cleanup_staging_at(&root).expect_err("residue must block cleanup");
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert_eq!(fs::read(&artifact).unwrap(), b"partial clone");
        fs::remove_dir_all(root).unwrap();
    }
}
