use sdl::video::ll::{SDL_Rect, SDL_Surface};
use std::os::raw::{c_char, c_int};

extern "C" {
    #[no_mangle]
    pub fn DisplayText(
        text: *const c_char,
        startx: c_int,
        starty: c_int,
        clip: *const SDL_Rect,
    ) -> c_int;

    #[no_mangle]
    pub fn printf_SDL(screen: *mut SDL_Surface, x: c_int, y: c_int, fmt: *mut c_char, ...);

}
