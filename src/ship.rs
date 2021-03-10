use crate::structs::Level;

use sdl::{
    video::ll::{SDL_FreeSurface, SDL_Surface},
    Rect,
};
use std::os::raw::{c_float, c_int};

extern "C" {
    pub fn show_droid_info(droid_type: c_int, page: c_int, flags: c_int);
    pub fn show_droid_portrait(dst: Rect, droid_type: c_int, cycle_time: c_float, flags: c_int);
    pub fn ShowDeckMap(deck: Level);
    pub fn AlertLevelWarning();
    pub fn EnterLift();
    pub fn EnterKonsole();

    pub static droid_background: *mut SDL_Surface;
    pub static droid_pics: *mut SDL_Surface;
}

#[no_mangle]
pub unsafe extern "C" fn FreeDroidPics() {
    SDL_FreeSurface(droid_pics);
    SDL_FreeSurface(droid_background);
}
