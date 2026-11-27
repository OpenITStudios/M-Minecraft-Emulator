//! Read-only Windows metadata from an already-open file.
//! Callers retain the file pin and enforce their own admission policy.
use std::{fs::File, io, mem, os::windows::io::AsRawHandle};
use windows_sys::Win32::Storage::FileSystem::{
    GetFileInformationByHandleEx, FileAttributeTagInfo, FileIdInfo, FileStandardInfo,
    FILE_ATTRIBUTE_TAG_INFO, FILE_ID_INFO, FILE_STANDARD_INFO,
};

pub(crate) fn attribute_tag(file: &File) -> io::Result<FILE_ATTRIBUTE_TAG_INFO> {
    let mut tag: FILE_ATTRIBUTE_TAG_INFO = unsafe { mem::zeroed() };
    if unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle() as _, FileAttributeTagInfo,
            (&mut tag as *mut FILE_ATTRIBUTE_TAG_INFO).cast(),
            mem::size_of_val(&tag) as u32,
        )
    } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(tag)
}

pub(crate) fn attributes(file: &File) -> io::Result<u32> {
    Ok(attribute_tag(file)?.FileAttributes)
}

// Raw handle metadata only. Link count, directory, size and disposition
// admission belong to the caller; this function never modifies a file.
pub(crate) fn standard(file: &File) -> io::Result<FILE_STANDARD_INFO> {
    let mut info: FILE_STANDARD_INFO = unsafe { mem::zeroed() };
    if unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle() as _, FileStandardInfo,
            (&mut info as *mut FILE_STANDARD_INFO).cast(),
            mem::size_of_val(&info) as u32,
        )
    } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(info)
}

pub(crate) fn file_id(file: &File) -> io::Result<FILE_ID_INFO> {
    let mut id: FILE_ID_INFO = unsafe { mem::zeroed() };
    if unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle() as _, FileIdInfo,
            (&mut id as *mut FILE_ID_INFO).cast(),
            mem::size_of_val(&id) as u32,
        )
    } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_metadata_reads_preserve_pinned_file_identity() {
        let file = File::open(std::env::current_exe().unwrap()).unwrap();
        let first = file_id(&file).unwrap();
        let second = file_id(&file).unwrap();
        assert_eq!(first.VolumeSerialNumber, second.VolumeSerialNumber);
        assert_eq!(first.FileId.Identifier, second.FileId.Identifier);
        assert_eq!(attributes(&file).unwrap(), attribute_tag(&file).unwrap().FileAttributes);
        let info = standard(&file).unwrap();
        assert!(!info.Directory && !info.DeletePending);
        assert_eq!(info.EndOfFile as u64, file.metadata().unwrap().len());
    }
}
