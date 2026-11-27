//! HUD geometry and worker-owned GDI raster. No extra worker or pixel copy.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Rect { pub left: i32, pub top: i32, pub right: i32, pub bottom: i32 }
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct Size { cx: i32, cy: i32 }

fn scale(value: i32, dpi: u32) -> i32 { (value * dpi as i32 + 48) / 96 }
fn extent(dpi: u32) -> Option<Size> {
    if !(48..=480).contains(&dpi) { return None; }
    Some(Size { cx: scale(128,dpi), cy: scale(32,dpi) })
}
fn placement(client: Rect, dpi: u32) -> Rect {
    let Some(size) = extent(dpi) else { return Rect::default(); };
    let margin = scale(12,dpi);
    if i64::from(client.right)-i64::from(client.left) < i64::from(size.cx+2*margin)
        || i64::from(client.bottom)-i64::from(client.top) < i64::from(size.cy+2*margin) {
        return Rect::default();
    }
    Rect { left: client.right-margin-size.cx, top: client.top+margin,
        right: client.right-margin, bottom: client.top+margin+size.cy }
}
#[no_mangle]
pub unsafe extern "C" fn lazytester_hud_placement(client: *const Rect, dpi: u32, result: *mut Rect) {
    if let Some(result) = result.as_mut() {
        *result = client.as_ref().map_or(Rect::default(), |c| placement(*c,dpi));
    }
}

#[cfg(windows)]
pub(crate) mod windows {
    use super::{extent, scale, Rect, Size};
    use std::{ffi::c_void, ptr, slice};
    type Handle = *mut c_void;
    #[repr(C)]
    struct Point { x: i32, y: i32 }
    #[repr(C)]
    struct BitmapHeader {
        size: u32, width: i32, height: i32, planes: u16, bits: u16,
        compression: u32, image_size: u32, xppm: i32, yppm: i32, used: u32, important: u32,
    }
    #[repr(C)]
    struct BitmapInfo { header: BitmapHeader, colors: [u32;1] }
    const _: () = assert!(std::mem::size_of::<BitmapHeader>() == 40);
    #[link(name="gdi32")]
    extern "system" {
        fn CreateCompatibleDC(dc: Handle) -> Handle;
        fn DeleteDC(dc: Handle) -> i32;
        fn DeleteObject(object: Handle) -> i32;
        fn SelectObject(dc: Handle, object: Handle) -> Handle;
        fn CreateDIBSection(dc: Handle, info: *const BitmapInfo, usage: u32, bits: *mut Handle, section: Handle, offset: u32) -> Handle;
        fn CreateFontW(height: i32,width: i32,escapement: i32,orientation: i32,weight: i32,
            italic: u32,underline: u32,strike: u32,charset: u32,out_precision: u32,
            clip_precision: u32,quality: u32,pitch: u32,face: *const u16) -> Handle;
        fn SetBkMode(dc: Handle, mode: i32) -> i32;
        fn SetTextColor(dc: Handle, color: u32) -> u32;
        fn GdiFlush() -> i32;
    }
    #[link(name="user32")]
    extern "system" {
        fn DrawTextW(dc: Handle, text: *const u16, count: i32, bounds: *mut Rect, format: u32) -> i32;
        fn GetWindowDpiAwarenessContext(window: Handle) -> Handle;
        fn SetThreadDpiAwarenessContext(context: Handle) -> Handle;
        fn GetClientRect(window: Handle, rect: *mut Rect) -> i32;
        fn ClientToScreen(window: Handle, point: *mut Point) -> i32;
        fn LogicalToPhysicalPointForPerMonitorDPI(window: Handle, point: *mut Point) -> i32;
    }
    #[no_mangle]
    pub unsafe extern "C" fn lazytester_hud_physical_client(owner: Handle, result: *mut Rect) -> u32 {
        let Some(result) = result.as_mut() else { return 0; };
        let context = GetWindowDpiAwarenessContext(owner);
        if context.is_null() { return 0; }
        let previous = SetThreadDpiAwarenessContext(context);
        if previous.is_null() { return 0; }
        let mut client = Rect::default();
        let mut okay = GetClientRect(owner,&mut client) != 0;
        let mut a = Point { x: client.left, y: client.top };
        let mut b = Point { x: client.right, y: client.bottom };
        okay = okay && ClientToScreen(owner,&mut a)!=0 && ClientToScreen(owner,&mut b)!=0
            && LogicalToPhysicalPointForPerMonitorDPI(owner,&mut a)!=0
            && LogicalToPhysicalPointForPerMonitorDPI(owner,&mut b)!=0;
        let restored = !SetThreadDpiAwarenessContext(previous).is_null();
        *result = Rect { left:a.x,top:a.y,right:b.x,bottom:b.y };
        u32::from(okay && restored)
    }
    fn selected(h: Handle) -> bool { !h.is_null() && h as isize != -1 }
    pub struct Canvas {
        dc: Handle, bitmap: Handle, font: Handle, old_bitmap: Handle, old_font: Handle,
        pixels: *mut u32, size: Size, dpi: u32,
    }
    impl Canvas {
        unsafe fn new() -> Option<Self> {
            let dc = CreateCompatibleDC(ptr::null_mut());
            if dc.is_null() { return None; }
            Some(Self { dc, bitmap:ptr::null_mut(),font:ptr::null_mut(),old_bitmap:ptr::null_mut(),
                old_font:ptr::null_mut(),pixels:ptr::null_mut(),size:Size::default(),dpi:0 })
        }
        unsafe fn release(&mut self) {
            if !self.old_font.is_null() { SelectObject(self.dc,self.old_font); }
            if !self.old_bitmap.is_null() { SelectObject(self.dc,self.old_bitmap); }
            // If deselection failed, closing the DC first permits object deletion.
            let font_failed = !self.font.is_null() && DeleteObject(self.font)==0;
            let bitmap_failed = !self.bitmap.is_null() && DeleteObject(self.bitmap)==0;
            if font_failed || bitmap_failed {
                // Recreate on next allocation; never keep a failed selected resource.
                DeleteDC(self.dc); self.dc=ptr::null_mut();
                if font_failed { DeleteObject(self.font); }
                if bitmap_failed { DeleteObject(self.bitmap); }
            }
            self.font=ptr::null_mut(); self.bitmap=ptr::null_mut();
            self.old_font=ptr::null_mut(); self.old_bitmap=ptr::null_mut();
            self.pixels=ptr::null_mut(); self.size=Size::default(); self.dpi=0;
        }
        unsafe fn allocate(&mut self, size: Size, dpi: u32) -> bool {
            if !self.bitmap.is_null() && !self.font.is_null() && self.size==size && self.dpi==dpi { return true; }
            self.release();
            if self.dc.is_null() { self.dc=CreateCompatibleDC(ptr::null_mut()); }
            if self.dc.is_null() { return false; }
            let info=BitmapInfo { header:BitmapHeader { size:40,width:size.cx,height:-size.cy,
                planes:1,bits:32,compression:0,image_size:0,xppm:0,yppm:0,used:0,important:0 },colors:[0] };
            let mut mapped=ptr::null_mut();
            let bitmap=CreateDIBSection(self.dc,&info,0,&mut mapped,ptr::null_mut(),0);
            if bitmap.is_null() || mapped.is_null() {
                if !bitmap.is_null() { DeleteObject(bitmap); } return false;
            }
            let old=SelectObject(self.dc,bitmap);
            if !selected(old) { DeleteObject(bitmap); return false; }
            self.bitmap=bitmap; self.old_bitmap=old; self.pixels=mapped.cast();
            let face=[83u16,101,103,111,101,32,85,73,0];
            let font=CreateFontW(-scale(17,dpi),0,0,0,700,0,0,0,1,0,0,4,0,face.as_ptr());
            if font.is_null() { self.release(); return false; }
            let old=SelectObject(self.dc,font);
            if !selected(old) { DeleteObject(font); self.release(); return false; }
            self.font=font; self.old_font=old; self.size=size; self.dpi=dpi; true
        }
        unsafe fn paint(&mut self, milli:u32, valid:bool, dpi:u32) -> bool {
            let Some(size)=extent(dpi) else { return false; };
            if !self.allocate(size,dpi) { return false; }
            let count=(size.cx*size.cy) as usize;
            slice::from_raw_parts_mut(self.pixels,count).fill(0xff182026);
            // Fixed stack UTF-16 buffer; no per-redraw heap formatting.
            let mut text=[0u16;32];
            let mut len=0;
            if valid {
                let tenths=(u64::from(milli)+50)/100;
                let mut whole=tenths/10;
                let mut reversed=[0u16;16];
                let mut digits=0;
                loop { reversed[digits]=48+(whole%10) as u16; digits+=1; whole/=10; if whole==0 {break;} }
                for digit in reversed[..digits].iter().rev() { text[len]=*digit;len+=1; }
                text[len]=46;text[len+1]=48+(tenths%10) as u16;len+=2;
            } else { text[0]=45;text[1]=45;len=2; }
            for ch in [32u16,70,80,83] { text[len]=ch;len+=1; }
            SetBkMode(self.dc,1); SetTextColor(self.dc,0xffffff);
            let mut bounds=Rect { left:0,top:0,right:size.cx,bottom:size.cy };
            if DrawTextW(self.dc,text.as_ptr(),len as i32,&mut bounds,0x825)!=0 && GdiFlush()!=0 {
                for pixel in slice::from_raw_parts_mut(self.pixels,count) { *pixel|=0xff000000; }
                true
            } else { false }
        }
    }
    impl Drop for Canvas {
        fn drop(&mut self) { unsafe { self.release(); if !self.dc.is_null() { DeleteDC(self.dc); } } }
    }
    #[repr(C)]
    pub struct CanvasView { pub dc:Handle,pixels:*const u32,pub size:Size }
    // Borrowed view valid only until next paint or destruction on owner thread.
    #[no_mangle]
    pub unsafe extern "C" fn lazytester_hud_canvas_create() -> *mut Canvas {
        Canvas::new().map_or(ptr::null_mut(),|c|Box::into_raw(Box::new(c)))
    }
    #[no_mangle]
    pub unsafe extern "C" fn lazytester_hud_canvas_destroy(canvas:*mut Canvas) {
        if !canvas.is_null() { drop(Box::from_raw(canvas)); }
    }
    #[no_mangle]
    pub unsafe extern "C" fn lazytester_hud_canvas_paint(canvas:*mut Canvas,milli:u32,valid:u32,dpi:u32,out:*mut CanvasView) -> u32 {
        let (Some(canvas),Some(out))=(canvas.as_mut(),out.as_mut()) else { return 0; };
        if !canvas.paint(milli,valid!=0,dpi) { return 0; }
        *out=CanvasView {dc:canvas.dc,pixels:canvas.pixels,size:canvas.size};1
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounds_and_dpi() {
        assert_eq!(placement(Rect{left:-1000,top:20,right:1000,bottom:1220},144),
            Rect{left:790,top:38,right:982,bottom:86});
        assert_eq!(placement(Rect{left:0,top:0,right:16,bottom:16},96),Rect::default());
        assert!(extent(47).is_none() && extent(481).is_none());
    }
}
