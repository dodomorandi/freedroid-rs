use sdl::video::ll::{SDL_Rect, SDL_Surface};
use std::os::raw::{c_char, c_int};

extern "C" {
    #[no_mangle]
    pub static mut Highscore_BFont: *mut BFontInfo;

    #[no_mangle]
    pub static mut Para_BFont: *mut BFontInfo;

    #[no_mangle]
    pub static mut CurrentFont: *mut BFontInfo;

    #[no_mangle]
    pub fn PrintStringFont(
        surface: *mut SDL_Surface,
        font: *mut BFontInfo,
        x: c_int,
        y: c_int,
        fmt: *mut c_char,
        ...
    );

}

#[derive(Clone)]
#[repr(C)]
pub struct BFontInfo {
    /// font height
    h: c_int,

    /// font surface
    surface: *mut SDL_Surface,

    /// characters width
    chars: [SDL_Rect; 256],
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
pub fn FontHeight(font: &BFontInfo) -> c_int {
    font.h
}
