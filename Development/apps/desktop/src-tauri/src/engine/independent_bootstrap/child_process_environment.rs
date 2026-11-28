//! Build the original child's private Windows environment block without logging inherited values.
use super::{directory_path, invalid, SlotStorage, OBSERVATION_ENV};
use super::child_configuration::CONFIG_ENV;
#[cfg(feature="mc02-native-input")]
use super::mc02_input;
use std::{ffi::{OsStr, OsString}, io, os::windows::ffi::OsStrExt};
use windows_sys::Win32::{Foundation::HANDLE, Globalization::CompareStringOrdinal, System::Environment::{FreeEnvironmentStringsW, GetEnvironmentStringsW}};

pub(super) fn environment_name(row: &[u16]) -> &[u16] {
    let start = usize::from(row.first() == Some(&61)); // Preserve hidden =C: drive entries.
    let end = row[start..]
        .iter()
        .position(|c| *c == 61)
        .map_or(row.len(), |i| start + i);
    &row[..end]
}

pub(super) fn ordinal_names(a: &[u16], b: &[u16]) -> io::Result<std::cmp::Ordering> {
    match unsafe { CompareStringOrdinal(a.as_ptr(), a.len() as i32, b.as_ptr(), b.len() as i32, 1) }
    {
        1 => Ok(std::cmp::Ordering::Less),
        2 => Ok(std::cmp::Ordering::Equal),
        3 => Ok(std::cmp::Ordering::Greater),
        _ => Err(io::Error::last_os_error()),
    }
}

pub(super) fn child_environment(
    storage: &SlotStorage,
    section: HANDLE,
    observation: Option<HANDLE>,
    presentation: Option<HANDLE>,
    #[cfg(feature="mc02-native-input")] mc02_input: Option<HANDLE>,
) -> io::Result<Vec<u16>> {
    let raw = unsafe { GetEnvironmentStringsW() };
    if raw.is_null() {
        return Err(io::Error::last_os_error());
    }
    let mut reserved: Vec<Vec<u16>> = [
        "APPDATA",
        "LOCALAPPDATA",
        "TMP",
        "TEMP",
        CONFIG_ENV,
        "MIVUBI_BOOTSTRAP_TEST_SECTION",
        OBSERVATION_ENV,
        "MIVUBI_EDUCATION_PRESENTATION",
        "MIVUBI_EDUCATION_PRESENTATION_SECTION",
    ]
    .iter()
    .map(|name| OsStr::new(name).encode_wide().collect())
    .collect();
    #[cfg(feature="mc02-native-input")]
    reserved.push(OsStr::new(mc02_input::INPUT_ENV).encode_wide().collect());
    let mut rows = Vec::<Vec<u16>>::new();
    let mut cursor = raw;
    let mut units = 0usize;
    let copied = (|| {
        // Bound the copy; never log or persist inherited environment values.
        while unsafe { *cursor } != 0 {
            let mut length = 0;
            while unsafe { *cursor.add(length) } != 0 {
                length += 1;
                if units + length > 32767 {
                    return Err(invalid("Inherited environment exceeds bootstrap bound"));
                }
            }
            let row = unsafe { std::slice::from_raw_parts(cursor, length) };
            let row_name = environment_name(row);
            let mut overridden = false;
            for name in &reserved {
                overridden |= ordinal_names(row_name, name)? == std::cmp::Ordering::Equal;
            }
            if !overridden {
                rows.push(row.to_vec());
            }
            units += length + 1;
            cursor = unsafe { cursor.add(length + 1) };
        }
        Ok(())
    })();
    unsafe {
        FreeEnvironmentStringsW(raw);
    }
    copied?;
    let temporary = directory_path(&storage.temporary[1])?.into_os_string();
    for (key, value) in [
        (
            "APPDATA",
            storage.private_path.join("Roaming").into_os_string(),
        ),
        (
            "LOCALAPPDATA",
            storage.private_path.join("Local").into_os_string(),
        ),
        ("TMP", temporary.clone()),
        ("TEMP", temporary),
        (CONFIG_ENV, OsString::from(format!("{}", section as usize))),
    ] {
        let mut entry: Vec<u16> = OsStr::new(key).encode_wide().collect();
        entry.push(61);
        entry.extend(value.encode_wide());
        rows.push(entry);
    }
    if let Some(section) = observation {
        rows.push(
            format!("{OBSERVATION_ENV}={}", section as usize)
                .encode_utf16()
                .collect(),
        );
    }
    if let Some(section) = presentation {
        rows.push(
            "MIVUBI_EDUCATION_PRESENTATION=pace"
                .encode_utf16()
                .collect(),
        );
        rows.push(
            format!("MIVUBI_EDUCATION_PRESENTATION_SECTION={}", section as usize)
                .encode_utf16()
                .collect(),
        );
    }
    #[cfg(feature="mc02-native-input")]
    if let Some(input)=mc02_input {
        rows.push(format!("{}={}",mc02_input::INPUT_ENV,input as usize)
            .encode_utf16().collect());
    }
    let mut compare_error = None;
    rows.sort_by(|a, b| {
        ordinal_names(environment_name(a), environment_name(b)).unwrap_or_else(|e| {
            compare_error = Some(e);
            std::cmp::Ordering::Equal
        })
    });
    if let Some(error) = compare_error {
        return Err(error);
    }
    encode_environment_block(rows)
}

// Size the final buffer once, including each row terminator and the final NUL.
fn encode_environment_block(rows: Vec<Vec<u16>>) -> io::Result<Vec<u16>> {
    let units = rows.iter().try_fold(1usize, |total, row| {
        total.checked_add(row.len())?.checked_add(1)
    }).filter(|total| *total <= 32767)
      .ok_or_else(|| invalid("Configured environment exceeds bootstrap bound"))?;
    let mut result = Vec::with_capacity(units);
    for row in rows {
        result.extend(row);
        result.push(0);
    }
    result.push(0);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn environment_block_preserves_order_and_exact_bound() {
        let a: Vec<u16> = "A=1".encode_utf16().collect();
        let b: Vec<u16> = "B=2".encode_utf16().collect();
        assert_eq!(encode_environment_block(vec![a,b]).unwrap(),
                   "A=1\0B=2\0\0".encode_utf16().collect::<Vec<_>>());
        assert_eq!(encode_environment_block(vec![vec![65;32765]]).unwrap().len(),32767);
        assert!(encode_environment_block(vec![vec![65;32766]]).is_err());
        assert!(encode_environment_block(vec![vec![65;16383],vec![66;16382]]).is_err());
    }
}
