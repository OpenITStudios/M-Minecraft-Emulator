//! Standalone distribution-only Windows SCM parent provisioner.
//! Ordinary builds are inert. No Minecraft launch, profile or account I/O.
use std::ffi::OsString;

#[cfg(all(windows, feature = "installer-parent-bootstrap-source"))]
#[path = "../../../isolation-native/independent-service/src/service_identity.rs"]
mod service_identity;

const ACTION: &str = "--create-fixed-parent";

fn approved(args: &[OsString]) -> bool {
    args.len() == 2 && args[1] == OsString::from(ACTION)
}

fn main() {
    #[cfg(all(windows, feature = "installer-parent-bootstrap-source"))]
    {
        let args: Vec<OsString> = std::env::args_os().collect();
        if !approved(&args) {
            eprintln!("Exactly --create-fixed-parent is required.");
            std::process::exit(2);
        }
        if let Err(error) = service_identity::provision_installer_parent_source() {
            eprintln!("Independent SCM installer-parent provisioning denied: {error}");
            // Do not delete or repair an object after postflight failure.
            std::process::exit(1);
        }
        println!("Fixed installer parent created and reviewed; Minecraft not enabled.");
    }
    #[cfg(not(all(windows, feature = "installer-parent-bootstrap-source")))]
    {
        eprintln!("Privileged installer-parent bootstrap is disabled.");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_one_explicit_action_without_any_path_is_accepted() {
        assert!(approved(&[
            OsString::from("bootstrap.exe"),
            OsString::from("--create-fixed-parent"),
        ]));
        for args in [
            vec![OsString::from("bootstrap.exe")],
            vec![OsString::from("bootstrap.exe"), OsString::from("--help")],
            vec![OsString::from("bootstrap.exe"), OsString::from("--delete")],
            vec![OsString::from("bootstrap.exe"), OsString::from("--create-parent")],
            vec![OsString::from("bootstrap.exe"),
                OsString::from("--create-fixed-parent"), OsString::from("C:\\Users")],
        ] {
            assert!(!approved(&args));
        }
    }
}
