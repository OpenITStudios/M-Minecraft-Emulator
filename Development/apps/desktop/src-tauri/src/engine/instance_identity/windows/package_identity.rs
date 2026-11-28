//! OS-reported package form and family comparison; no package names leave the observer.
use super::{classify_package_family_pair, classify_package_probe, invalid, live, pin, ProcessKey, ObservedProcessPackageForm, ObservedPackageFamilyRelation, GetPackageFullName, GetPackageFamilyName};
use std::{io, ptr, os::windows::io::{AsRawHandle, OwnedHandle}};

/// Read package identity without returning its name. Calling the first
/// probe with a null buffer distinguishes MSIX package identity from an
/// ordinary unpackaged executable on the SAME pinned process HANDLE.
/// Metadata does not prove package origin, publisher or registry routing.
pub(in crate::engine::instance_identity) fn inspect_package_form(key: &ProcessKey) -> io::Result<ObservedProcessPackageForm> {
    let process = pin(key)?;
    let mut size = 0u32;
    let first = unsafe {
        GetPackageFullName(process.as_raw_handle() as _, &mut size, ptr::null_mut())
    };
    let needed = classify_package_probe(first, size)
        .map_err(|_| invalid("Windows process package query failed"))?;
    let form = if let Some(count) = needed {
        let mut name = vec![0u16; count];
        let mut written = count as u32;
        let result = unsafe {
            GetPackageFullName(process.as_raw_handle() as _, &mut written, name.as_mut_ptr())
        };
        if result != 0 || written < 2 || written as usize > count
            || name[written as usize - 1] != 0
            || name[..written as usize - 1].contains(&0)
        {
            return Err(invalid("Windows package identity changed or is invalid"));
        }
        // Deliberately discard package full name: no logging, hashing,
        // exporting or promotion to a trusted publisher is required.
        ObservedProcessPackageForm::Packaged
    } else {
        ObservedProcessPackageForm::Unpackaged
    };
    live(&process)?;
    Ok(form)
}

/// One read-only OS package-family query against an already pinned
/// process HANDLE. Cap the UTF-16 buffer and never log or return names.
pub(super) fn package_family_name(process: &OwnedHandle) -> io::Result<Option<String>> {
    let mut required = 0u32;
    let first = unsafe {
        GetPackageFamilyName(process.as_raw_handle() as _, &mut required, ptr::null_mut())
    };
    let Some(capacity) = classify_package_probe(first, required)
        .map_err(|_| invalid("Windows package-family query failed"))?
    else {
        return Ok(None);
    };
    let mut name = vec![0u16; capacity];
    let mut written = capacity as u32;
    let status = unsafe {
        GetPackageFamilyName(process.as_raw_handle() as _, &mut written, name.as_mut_ptr())
    };
    if status != 0 || written < 2 || written as usize > capacity
        || name[written as usize - 1] != 0
        || name[..written as usize - 1].contains(&0)
    {
        return Err(invalid("Windows package-family identity changed or is invalid"));
    }
    String::from_utf16(&name[..written as usize - 1])
        .map(Some)
        .map_err(|_| invalid("Windows returned an invalid package-family string"))
}

/// Package names stay within this native call. Same family means only
/// same OS-reported package family, not a shared user, a verified official
/// Minecraft identity, or observed filesystem isolation.
pub(in crate::engine::instance_identity) fn compare_package_families(
    left: &ProcessKey,
    right: &ProcessKey,
) -> io::Result<ObservedPackageFamilyRelation> {
    let first = pin(left)?;
    let second = pin(right)?;
    let first_family = package_family_name(&first)?;
    let second_family = package_family_name(&second)?;
    live(&first)?;
    live(&second)?;
    Ok(classify_package_family_pair(
        first_family.as_deref(), second_family.as_deref(),
    ))
}
