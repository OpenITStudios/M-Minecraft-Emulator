//! Private presentation capability version and FPS limit decoding.
use super::frame_pacing::lazytester_frame_pacing_valid_ceilings;
fn decode(version:u32,reserved:u32)->Option<(u32,u32)> {
    if version==2 { return (reserved==0).then_some((144,10)); }
    if version!=3 && version!=4 { return None; }
    let active=reserved&0xffff;
    let background=reserved>>16;
    (lazytester_frame_pacing_valid_ceilings(active,background)!=0).then_some((active,background))
}
#[no_mangle]
pub unsafe extern "C" fn lazytester_presentation_limits(version:u32,reserved:u32,active:*mut u32,background:*mut u32)->u32 {
    if active.is_null() || background.is_null() || active==background {return 0;}
    let Some((a,b))=decode(version,reserved) else {return 0;};
    active.write(a);background.write(b);1
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn versions_and_limits() {
        assert_eq!(decode(2,0),Some((144,10)));
        assert_eq!(decode(2,60|(5<<16)),None);
        for version in [3,4] {
            assert_eq!(decode(version,60|(5<<16)),Some((60,5)));
            assert_eq!(decode(version,60|(61<<16)),None);
            assert_eq!(decode(version,0),None);
        }
        assert_eq!(decode(5,60|(5<<16)),None);
    }
}
