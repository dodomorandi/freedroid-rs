use sdl::video::ll::{SDL_Rect, SDL_Surface};
use std::os::raw::c_int;

extern "C" {
    #[no_mangle]
    pub fn GetCurrentFont() -> *mut BFontInfo;

    #[no_mangle]
    pub fn SetCurrentFont(font: *mut BFontInfo);

    #[no_mangle]
    pub fn FontHeight(font: *mut BFontInfo) -> c_int;

    #[no_mangle]
    pub static mut Highscore_BFont: *mut BFontInfo;

    #[no_mangle]
    pub static mut Para_BFont: *mut BFontInfo;
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
