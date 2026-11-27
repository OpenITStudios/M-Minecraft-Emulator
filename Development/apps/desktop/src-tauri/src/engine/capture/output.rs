//! Bounded worker receipt and no-clobber publication. No image transcoding.
use std::io;
pub(super) const MAX_WEBP_BYTES:usize=80*1024*1024;
#[derive(Debug,PartialEq,Eq)]
pub(super) struct Dimensions {pub width:u32,pub height:u32}
fn invalid()->io::Error {io::Error::new(io::ErrorKind::InvalidData,"Incomplete or unsupported WebP screenshot")}

// Only the simple VP8L RIFF produced by our approved worker is accepted.
// This is structural admission, not a substitute for native decode proof.
pub(super) fn inspect_webp(bytes:&[u8])->io::Result<Dimensions>{
    if !(26..=MAX_WEBP_BYTES).contains(&bytes.len())||&bytes[..4]!=b"RIFF"||
        &bytes[8..16]!=b"WEBPVP8L"||bytes[20]!=0x2f{return Err(invalid());}
    let riff=u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
    let chunk=u32::from_le_bytes(bytes[16..20].try_into().unwrap()) as usize;
    if riff.checked_add(8)!=Some(bytes.len())||chunk<6||
        20usize.checked_add(chunk).and_then(|n|n.checked_add(chunk%2))!=Some(bytes.len())||
        (chunk%2!=0&&bytes.last()!=Some(&0)) {return Err(invalid());}
    let header=u32::from_le_bytes(bytes[21..25].try_into().unwrap());
    let width=(header&0x3fff)+1;
    let height=((header>>14)&0x3fff)+1;
    if header>>29!=0||width>4096||height>4096{return Err(invalid());}
    Ok(Dimensions{width,height})
}

#[cfg(windows)]
pub(super) fn publish(path:&std::path::Path,bytes:&[u8])->io::Result<()>{
    use std::io::Write;
    if !path.extension().is_some_and(|ext|ext.eq_ignore_ascii_case("webp")) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,"Save screenshot with a .webp extension"));
    }
    let parent=path.parent().ok_or_else(||io::Error::other("Screenshot save directory unavailable"))?;
    // Exclusive temp file in the chosen folder; no clobber even if another
    // process creates the final file after the native dialog returns.
    let mut temp=tempfile::Builder::new().prefix(".lazy-tester-").suffix(".part").tempfile_in(parent)?;
    temp.write_all(bytes)?;
    temp.as_file().sync_all()?;
    temp.persist_noclobber(path).map_err(|e|e.error)?;
    Ok(())
}

#[cfg(test)]
mod tests{
    use super::*;
    fn fixture()->Vec<u8>{
        let mut b=b"RIFF".to_vec();b.extend_from_slice(&18u32.to_le_bytes());
        b.extend_from_slice(b"WEBPVP8L");b.extend_from_slice(&6u32.to_le_bytes());
        b.push(0x2f);b.extend_from_slice(&(1u32|(2u32<<14)).to_le_bytes());b.push(0);
        b
    }
    #[test]
    fn requires_complete_single_lossless_payload(){
        let b=fixture();
        assert_eq!(inspect_webp(&b).unwrap(),Dimensions{width:2,height:3});
        assert!(inspect_webp(&b[..25]).is_err());
        let mut extra=b.clone();extra.push(0);assert!(inspect_webp(&extra).is_err());
        let mut wrong=b.clone();wrong[15]=b' ';assert!(inspect_webp(&wrong).is_err());
        let mut huge=b;huge[21..25].copy_from_slice(&4096u32.to_le_bytes());assert!(inspect_webp(&huge).is_err());
    }
    #[cfg(windows)]
    #[test]
    fn publication_does_not_replace_existing_file(){
        let root=tempfile::tempdir().unwrap();
        let path=root.path().join("shot.webp");
        publish(&path,b"first").unwrap();
        assert!(publish(&path,b"second").is_err());
        assert_eq!(std::fs::read(path).unwrap(),b"first");
    }
}
