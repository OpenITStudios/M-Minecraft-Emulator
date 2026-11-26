//! Read-only Windows metadata from an already-open file.
//! Callers retain the file pin and enforce their own admission policy.
use std::{fs::File, io, mem, os::windows::io::AsRawHandle};
use windows_sys::Win32::Storage::FileSystem::{
    GetFileInformationByHandleEx, FileAttributeTagInfo, FileIdInfo,
    FILE_ATTRIBUTE_TAG_INFO, FILE_ID_INFO,
};

pub(crate) fn attributes(file: &File) -> io::Result<u32> {
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
    Ok(tag.FileAttributes)
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
        assert_eq!(attributes(&file).unwrap(), attributes(&file).unwrap());
    }
}
