use sdl::{
    sdl::Rect,
    video::ll::{SDL_Rect, SDL_Surface, SDL_UpperBlit},
};
use std::{
    convert::{TryFrom, TryInto},
    ffi::VaList,
    os::raw::{c_char, c_int},
};

extern "C" {
    pub static mut Highscore_BFont: *mut BFontInfo;
    pub static mut Para_BFont: *mut BFontInfo;
    pub static mut CurrentFont: *mut BFontInfo;
    fn vsprintf(str: *mut c_char, format: *const c_char, ap: VaList) -> c_int;
}

#[derive(Clone)]
#[repr(C)]
pub struct BFontInfo {
    /// font height
    pub h: c_int,

    /// font surface
    pub surface: *mut SDL_Surface,

    /// characters width
    pub chars: [SDL_Rect; 256],
}

#[no_mangle]
pub unsafe extern "C" fn GetCurrentFont() -> *mut BFontInfo {
    CurrentFont
}

#[no_mangle]
pub unsafe extern "C" fn SetCurrentFont(font: *mut BFontInfo) {
    CurrentFont = font;
}

#[no_mangle]
pub extern "C" fn FontHeight(font: &BFontInfo) -> c_int {
    font.h
}

#[no_mangle]
pub unsafe extern "C" fn PutStringFont(
    surface: *mut SDL_Surface,
    font: *mut BFontInfo,
    mut x: c_int,
    y: c_int,
    text: *const c_char,
) {
    let mut i = 0;
    while *text.offset(i) != b'\0' as i8 {
        x += PutCharFont(&mut *surface, &mut *font, x, y, (*text.offset(i)).into());
        i += 1;
    }
}

/// Put a single char on the surface with the specified font
#[no_mangle]
pub unsafe extern "C" fn PutCharFont(
    surface: &mut SDL_Surface,
    font: &mut BFontInfo,
    x: c_int,
    y: c_int,
    c: c_int,
) -> c_int {
    let mut dest = Rect::new(
        x.try_into().unwrap(),
        y.try_into().unwrap(),
        CharWidth(font, b' '.into()).try_into().unwrap(),
        FontHeight(font).try_into().unwrap(),
    );

    if c != b' '.into() {
        SDL_UpperBlit(
            font.surface,
            &mut font.chars[usize::try_from(c).unwrap()],
            surface,
            &mut dest,
        );
    }
    dest.w.into()
}

/// Return the width of the "c" character
#[no_mangle]
pub extern "C" fn CharWidth(font: &BFontInfo, c: c_int) -> c_int {
    font.chars[usize::try_from(c).unwrap()].w.into()
}

#[no_mangle]
pub unsafe extern "C" fn PutString(
    surface: *mut SDL_Surface,
    x: c_int,
    y: c_int,
    text: *const c_char,
) {
    PutStringFont(surface, CurrentFont, x, y, text);
}

/// Puts a single char on the surface
#[no_mangle]
pub unsafe extern "C" fn PutChar(surface: *mut SDL_Surface, x: c_int, y: c_int, c: c_int) -> c_int {
    PutCharFont(&mut *surface, &mut *CurrentFont, x, y, c)
}

#[no_mangle]
pub unsafe extern "C" fn PrintStringFont(
    surface: *mut SDL_Surface,
    font: *mut BFontInfo,
    x: c_int,
    y: c_int,
    fmt: *mut c_char,
    args: ...
) {
    let mut args = args.clone();

    let mut temp = [0; 1001];
    vsprintf(temp.as_mut_ptr(), fmt, args.as_va_list());
    PutStringFont(surface, font, x, y, temp.as_mut_ptr());
}
