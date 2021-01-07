use sdl::Rect;
use std::os::raw::{c_float, c_int};

extern "C" {
    pub fn FreeDroidPics();
    pub fn show_droid_info(droid_type: c_int, page: c_int, flags: c_int);
    pub fn show_droid_portrait(dst: Rect, droid_type: c_int, cycle_time: c_float, flags: c_int);
}
