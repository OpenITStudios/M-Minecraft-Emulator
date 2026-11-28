//! Registered Education installation admission and reviewed image compatibility.
use super::{file_digest, image_id, invalid, pin_image, same_image};
use std::{fs::File, io, path::Path};

pub(super) fn registered_image(observe: bool) -> io::Result<(String, File, u32)> {
    registered_image_for_generation(observe, std::iter::empty::<(&File, u32)>())
}

// Only live controller-owned children with continuously retained image pins
// may supply a reviewed recipe. No path/version cache survives the last exit.
pub(super) fn registered_image_for_generation<'a>(
    observe: bool, active: impl IntoIterator<Item = (&'a File, u32)>,
) -> io::Result<(String, File, u32)> {
    use crate::engine::native_clients::resolve_registered_education_target;
    // Refuse a known busy updater BEFORE taking a write/delete-denying pin.
    // Recheck after pinning to narrow the race; neither check is an installer lock.
    // Both checks reuse existing resolver processes; no passive polling.
    let selector = if observe {
        "Resolve-ManagerEducationTarget -RegisteredOnly -RequireUpdaterIdle"
    } else {
        "Resolve-ManagerEducationTarget -RegisteredOnly"
    };
    let target = resolve_registered_education_target(selector)?;
    let (target, image, _) = pin_registered_image_with(target, false, || resolve_registered_education_target(selector))?;
    let recipe = if observe {
        generation_recipe(&image, active, || reviewed_singleton_recipe(file_digest(&image, 256 * 1024 * 1024)?))?
    } else { 0 };
    Ok((target, image, recipe))
}

#[cfg(test)]
fn registered_image_with(observe: bool, mut resolve: impl FnMut() -> io::Result<String>) -> io::Result<(String, File, u32)> {
    let target = resolve()?;
    pin_registered_image_with(target, observe, resolve)
}

// Preparation already resolved this exact path for the initial workspace
// observation. Reuse it only within that transaction; the post-pin resolver
// still verifies the current signed registration and the same retained bytes.
pub(super) fn prepare_registered_image(target: String) -> io::Result<(String, File, u32)> {
    pin_registered_image_with(target, false, || crate::engine::native_clients::resolve_registered_education_target("Resolve-ManagerEducationTarget -RegisteredOnly"))
}

fn pin_registered_image_with(target: String, observe: bool, resolve: impl FnOnce() -> io::Result<String>) -> io::Result<(String, File, u32)> {
    let image = pin_image(Path::new(&target))?;
    let singleton_caller_rva = validate_registered_image_with(&target,&image,observe,resolve)?;
    Ok((target,image,singleton_caller_rva))
}

// Explicit informational observation. Reuse the post-pin registration and
// file identity gate; do not acquire an idle/retained controller image pin.
pub(super) fn installation_report() -> io::Result<crate::engine::native_clients::EducationInstallationReport> {
    use crate::engine::native_clients::{registered_education_metadata, EducationInstallationReport, EducationUpdaterState};
    let initial = registered_education_metadata()?;
    if matches!(initial.updater, EducationUpdaterState::Busy) {
        // A known pending/running update needs no game-image pin or hash.
        return Ok(EducationInstallationReport { schema: 1, registered_version: initial.registered_version,
            updater: initial.updater, reviewed_image: None });
    }
    let mut metadata = None;
    let (_, image, _) = pin_registered_image_with(initial.path, false, || {
        let receipt = registered_education_metadata()?;
        let path = receipt.path.clone();
        metadata = Some(receipt);
        Ok(path)
    })?;
    let metadata = metadata.ok_or_else(|| invalid("Education installation metadata unavailable"))?;
    let reviewed_image = if matches!(metadata.updater, EducationUpdaterState::Busy) {
        None // Updater started during the observation. Release the pin without hashing.
    } else {
        Some(reviewed_singleton_recipe(file_digest(&image, 256 * 1024 * 1024)?).is_ok())
    };
    Ok(EducationInstallationReport { schema: 1, registered_version: metadata.registered_version,
        updater: metadata.updater, reviewed_image })
}

// Retained-image comparison is used only by the opt-in timing fixture.
#[cfg(test)]
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

pub(super) const UNREVIEWED_IMAGE_MESSAGE: &str =
    "This Minecraft Education version is not yet supported for managed launch. Update Lazy Tester compatibility before launching. Saved launcher data is preserved.";

// Reviewed data only. A Microsoft signature or version label cannot admit a
// version-specific recipe. New entries require exact-image native review.
const REVIEWED_IMAGE_RECIPES: [([u8; 32], u32); 2] = [
    ([
        0x04, 0x16, 0x4c, 0x1c, 0x64, 0x45, 0x8a, 0x5d, 0x98, 0x4d, 0xbc, 0x16, 0x87, 0xd0, 0xe8,
        0x02, 0x3d, 0x18, 0x08, 0xc4, 0x4e, 0xe7, 0xbe, 0x03, 0x51, 0x9c, 0xdf, 0x83, 0x55, 0x23,
        0x3e, 0xaa,
    ], 0x2cdc30),
    // Education 1.26.33.0: previously observed native startup recipe.
    ([
        0x59,0xfa,0xc4,0x8c,0xfb,0x6b,0x2c,0xf8,0xd0,0x82,0x1e,0x9a,0x0a,0xe6,0xd5,0x86,
        0x48,0x1a,0xb1,0xe7,0x3c,0xee,0xf4,0x6c,0xea,0x53,0xec,0xee,0x3a,0xc4,0x8b,0x43,
    ], 0x2cf0f0),
];

pub(super) fn reviewed_singleton_recipe(digest: [u8; 32]) -> io::Result<u32> {
    REVIEWED_IMAGE_RECIPES.iter().find_map(|(known, recipe)| (*known == digest).then_some(*recipe))
        .ok_or_else(|| io::Error::new(io::ErrorKind::Unsupported, UNREVIEWED_IMAGE_MESSAGE))
}

// Existing child image pins deny write/delete sharing. Do not mix installations
// if official registration moves while an original managed child remains alive.
pub(super) fn require_same_generation<'a>(
    image: &File, active: impl IntoIterator<Item = &'a File>,
) -> io::Result<()> {
    let candidate = image_id(image)?;
    for original in active {
        if !same_image(&candidate, &image_id(original)?) {
            return Err(io::Error::new(io::ErrorKind::WouldBlock,
                "Minecraft Education installation changed. Close existing games normally before launching the new installation."));
        }
    }
    Ok(())
}

// Reuse the previously admitted recipe only when every original pin still
// names the exact same file object and all retained recipes agree. No cached
// metadata or caller-supplied version is used. The first launch after idle
// must read the new bytes again, even when the path or file ID is unchanged.
fn generation_recipe<'a>(
    image: &File, active: impl IntoIterator<Item = (&'a File, u32)>,
    review: impl FnOnce() -> io::Result<u32>,
) -> io::Result<u32> {
    let mut known = None;
    for (original, recipe) in active {
        require_same_generation(image, [original])?;
        if recipe == 0 || known.is_some_and(|previous| previous != recipe) {
            return Err(invalid("Retained Education compatibility recipe is inconsistent"));
        }
        known = Some(recipe);
    }
    match known { Some(recipe) => Ok(recipe), None => review() }
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
    fn preparation_reuses_observed_path_but_rejects_changed_registration_after_pin() {
        let root=tempfile::tempdir().unwrap();
        let first=root.path().join("first.exe");
        let second=root.path().join("second.exe");
        std::fs::write(&first,[7u8;128]).unwrap();
        std::fs::write(&second,[9u8;128]).unwrap();
        let target=first.to_str().unwrap().to_owned();
        let mut calls=0;
        let (observed,pin,rva)=pin_registered_image_with(target.clone(),false,||{
            calls+=1;
            assert!(OpenOptions::new().write(true).open(&first).is_err(),"Resolve again while the original image is pinned");
            Ok(target.clone())
        }).unwrap();
        assert_eq!(calls,1);
        assert_eq!(observed,target);
        assert_eq!(rva,0);
        drop(pin);
        assert!(pin_registered_image_with(target.clone(),false,||Ok(second.to_str().unwrap().into())).is_err());
        assert_eq!(pin_registered_image_with(target,false,||Err(io::Error::from(io::ErrorKind::PermissionDenied))).unwrap_err().kind(),io::ErrorKind::PermissionDenied);
        assert!(OpenOptions::new().write(true).open(&first).is_ok(),"Failed preparation must release its pin");
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
    fn changed_image_is_refused_until_original_generation_is_closed() {
        let root = tempfile::tempdir().unwrap();
        let first = root.path().join("first.exe");
        let second = root.path().join("second.exe");
        std::fs::write(&first, [7u8; 128]).unwrap();
        std::fs::write(&second, [7u8; 128]).unwrap();
        let original = pin_image(&first).unwrap();
        let same = pin_image(&first).unwrap();
        let replacement = pin_image(&second).unwrap();
        assert!(require_same_generation(&same, [&original]).is_ok());
        assert_eq!(require_same_generation(&replacement, [&original]).unwrap_err().kind(), io::ErrorKind::WouldBlock);
        assert!(require_same_generation(&replacement, std::iter::empty::<&File>()).is_ok());
        assert!(OpenOptions::new().write(true).open(&first).is_err());
        drop(same);
        drop(original);
        assert!(OpenOptions::new().write(true).open(&first).is_ok());
    }
    #[test]
    fn only_continuously_pinned_matching_generation_reuses_review() {
        let root = tempfile::tempdir().unwrap();
        let first = root.path().join("first.exe");
        let second = root.path().join("second.exe");
        std::fs::write(&first, [7u8; 128]).unwrap();
        std::fs::write(&second, [7u8; 128]).unwrap();
        let original = pin_image(&first).unwrap();
        let same = pin_image(&first).unwrap();
        let replacement = pin_image(&second).unwrap();
        let mut reviews = 0;
        assert_eq!(generation_recipe(&same, [(&original, 7)], || {reviews+=1; Ok(8)}).unwrap(), 7);
        assert_eq!(reviews, 0);
        assert!(generation_recipe(&replacement, [(&original, 7)], || panic!("must refuse replacement before review")).is_err());
        assert!(generation_recipe(&same, [(&original, 7), (&same, 8)], || panic!("must refuse conflicting recipes")).is_err());
        assert!(generation_recipe(&same, [(&original, 0)], || panic!("must refuse missing recipe")).is_err());
        drop(same);
        drop(original);
        std::fs::write(&first, [9u8;128]).unwrap();
        let updated = pin_image(&first).unwrap();
        assert_eq!(generation_recipe(&updated, std::iter::empty::<(&File,u32)>(), || {reviews+=1; Ok(8)}).unwrap(), 8);
        assert_eq!(reviews, 1);
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
