//! Registered Education installation admission and reviewed image compatibility.
use super::{file_digest, image_id, invalid, pin_image, same_image};
use std::{fs::File, io, path::Path};

pub(super) fn registered_image(observe: bool) -> io::Result<(String, File, u32)> {
    registered_image_with(observe, || crate::engine::native_clients::resolve_registered_education_target("Resolve-ManagerEducationTarget -RegisteredOnly"))
}

fn registered_image_with(observe: bool, mut resolve: impl FnMut() -> io::Result<String>) -> io::Result<(String, File, u32)> {
    let target = resolve()?;
    let path = Path::new(&target);
    let image = pin_image(path)?;
    let singleton_caller_rva = validate_registered_image_with(&target,&image,observe,&mut resolve)?;
    Ok((target,image,singleton_caller_rva))
}

fn validate_registered_image(target:&str,image:&File,observe:bool)->io::Result<u32> {
    validate_registered_image_with(target,image,observe,||crate::engine::native_clients::resolve_registered_education_target("Resolve-ManagerEducationTarget -RegisteredOnly"))
}
fn validate_registered_image_with(target:&str,image:&File,observe:bool,resolve:impl FnOnce()->io::Result<String>)->io::Result<u32> {
    // The original pin predates this full signature/registration check. Reopen the
    // current name only to prove it still denotes those same retained bytes.
    if resolve()? != target {
        return Err(invalid("Education registration changed during preparation"));
    }
    if !same_image(&image_id(image)?,&image_id(&pin_image(Path::new(target))?)?) {
        return Err(invalid("Registered Education image identity changed"));
    }
    if observe {
        return reviewed_singleton_recipe(file_digest(image, 256 * 1024 * 1024)?);
    }
    Ok(0)
}

pub(super) fn reviewed_singleton_recipe(digest: [u8; 32]) -> io::Result<u32> {
    // Publisher identity alone cannot authorize a version-specific callsite.
    let previous_image = [
        0x04, 0x16, 0x4c, 0x1c, 0x64, 0x45, 0x8a, 0x5d, 0x98, 0x4d, 0xbc, 0x16, 0x87, 0xd0, 0xe8,
        0x02, 0x3d, 0x18, 0x08, 0xc4, 0x4e, 0xe7, 0xbe, 0x03, 0x51, 0x9c, 0xdf, 0x83, 0x55, 0x23,
        0x3e, 0xaa,
    ];
    if digest == previous_image {
        return Ok(0x2cdc30);
    }
    // Education 1.26.33.0: transparent native startup observed this caller's
    // initially owned GUID mutex returning 0, then 183 in the rejected second process.
    let education_12633 = [
        0x59,0xfa,0xc4,0x8c,0xfb,0x6b,0x2c,0xf8,0xd0,0x82,0x1e,0x9a,0x0a,0xe6,0xd5,0x86,
        0x48,0x1a,0xb1,0xe7,0x3c,0xee,0xf4,0x6c,0xea,0x53,0xec,0xee,0x3a,0xc4,0x8b,0x43,
    ];
    if digest == education_12633 {
        return Ok(0x2cf0f0);
    }
    Err(io::Error::new(io::ErrorKind::Unsupported,
        "This Minecraft Education version is not yet supported for managed launch. Update Lazy Tester compatibility before launching. Saved launcher data is preserved."))
}

#[cfg(test)]
mod launch_preflight_measurement {
    use super::*;
    use std::fs::OpenOptions;
    #[test]
    fn each_launch_resolves_and_pins_the_current_installation_only_for_that_attempt() {
        let root=std::env::temp_dir().join(format!("lazy-registration-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir(&root).unwrap();
        let first=root.join("first.exe");let second=root.join("second.exe");
        std::fs::write(&first,[7u8;128]).unwrap();std::fs::write(&second,[9u8;128]).unwrap();
        for path in [&first,&second] {
            let mut calls=0;
            let (target,pin,rva)=registered_image_with(false,||{calls+=1;Ok(path.to_str().unwrap().into())}).unwrap();
            assert_eq!(rva,0);
            assert_eq!(calls,2,"Registration must be checked after the image is pinned");
            assert_eq!(target,path.to_str().unwrap());
            assert!(OpenOptions::new().write(true).open(path).is_err());
            drop(pin);
            assert!(OpenOptions::new().write(true).open(path).is_ok(),"No idle controller pin may block an update");
        }
        let mut calls=0;
        assert!(registered_image_with(false,||{calls+=1;Ok(if calls==1{&first}else{&second}.to_str().unwrap().into())}).is_err(),"Registration races must still fail");
        assert_eq!(registered_image_with(true,||Ok(second.to_str().unwrap().into())).unwrap_err().kind(),io::ErrorKind::Unsupported);
        assert!(OpenOptions::new().write(true).open(&second).is_ok(),"Refused attempts must release their image pins");
        std::fs::remove_file(first).unwrap();std::fs::remove_file(second).unwrap();std::fs::remove_dir(root).unwrap();
    }

    #[test]
    fn a_recipe_binds_exact_image_bytes_to_its_reviewed_callsite() {
        let digest=[0x04,0x16,0x4c,0x1c,0x64,0x45,0x8a,0x5d,0x98,0x4d,0xbc,0x16,0x87,0xd0,0xe8,0x02,0x3d,0x18,0x08,0xc4,0x4e,0xe7,0xbe,0x03,0x51,0x9c,0xdf,0x83,0x55,0x23,0x3e,0xaa];
        assert_eq!(reviewed_singleton_recipe(digest).unwrap(),0x2cdc30);
        let updated=[0x59,0xfa,0xc4,0x8c,0xfb,0x6b,0x2c,0xf8,0xd0,0x82,0x1e,0x9a,0x0a,0xe6,0xd5,0x86,0x48,0x1a,0xb1,0xe7,0x3c,0xee,0xf4,0x6c,0xea,0x53,0xec,0xee,0x3a,0xc4,0x8b,0x43];
        assert_eq!(reviewed_singleton_recipe(updated).unwrap(),0x2cf0f0);
        let mut changed=digest;changed[31]^=1;
        assert_eq!(reviewed_singleton_recipe(changed).unwrap_err().kind(),io::ErrorKind::Unsupported);
    }
    #[test]
    fn retained_launch_validation_checks_fresh_registration_identity_and_bytes() {
        let root=std::env::temp_dir().join(format!("lazy-preflight-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir(&root).unwrap();let path=root.join("fixture.exe");let other=root.join("other.exe");
        std::fs::write(&path,[7u8;128]).unwrap();std::fs::write(&other,[9u8;128]).unwrap();
        let target=path.to_str().unwrap();let image=pin_image(&path).unwrap();
        let mut calls=0;
        validate_registered_image_with(target,&image,false,||{calls+=1;Ok(target.into())}).unwrap();assert_eq!(calls,1);
        assert!(OpenOptions::new().write(true).open(&path).is_err(),"Original pin must deny concurrent writers");
        assert!(validate_registered_image_with(target,&image,false,||Ok(other.to_str().unwrap().into())).is_err());
        let wrong=pin_image(&other).unwrap();
        assert!(validate_registered_image_with(target,&wrong,false,||Ok(target.into())).is_err());
        assert!(validate_registered_image_with(target,&image,true,||Ok(target.into())).is_err(),"Unreviewed bytes must still be refused");
        assert_eq!(validate_registered_image_with(target,&image,false,||Err(io::Error::from(io::ErrorKind::PermissionDenied))).unwrap_err().kind(),io::ErrorKind::PermissionDenied);
        drop(wrong);drop(image);assert!(root.starts_with(std::env::temp_dir()));std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    #[ignore="read-only timing of the official registered installation; never opens profiles or starts Minecraft"]
    fn registered_launch_validation_cost() {
        for sample in 0..3 {
            let start=std::time::Instant::now();
            let target=crate::engine::native_clients::resolve_registered_education_target("Resolve-ManagerEducationTarget -RegisteredOnly").unwrap();
            let registration_ms=start.elapsed().as_millis();
            let start=std::time::Instant::now();let image=pin_image(Path::new(&target)).unwrap();
            let _=file_digest(&image,256*1024*1024).unwrap();
            let image_ms=start.elapsed().as_millis();
            let start=std::time::Instant::now();drop(registered_image(true).unwrap());let full_ms=start.elapsed().as_millis();
            let start=std::time::Instant::now();validate_registered_image(&target,&image,true).unwrap();let retained_ms=start.elapsed().as_millis();
            eprintln!("LAZY_PREFLIGHT sample={sample} registration_signature_ms={registration_ms} image_pin_digest_ms={image_ms} full_validation_ms={full_ms} retained_validation_ms={retained_ms}");
        }
    }
}
