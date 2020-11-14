use sdl::video::ll::{SDL_Rect, SDL_Surface};
use std::os::raw::{c_float, c_int};

extern "C" {
    pub fn MakeGridOnScreen(grid_rectangle: *mut SDL_Rect);
    pub fn ApplyFilter(
        surface: *mut SDL_Surface,
        fred: c_float,
        fgreen: c_float,
        fblue: c_float,
    ) -> c_int;
    pub static mut vid_bpp: c_int;
}
