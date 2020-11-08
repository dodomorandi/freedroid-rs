use crate::structs::{Finepoint, Level};

use std::os::raw::{c_float, c_int, c_uchar};

extern "C" {
    #[no_mangle]
    pub fn IsVisible(objpos: *mut Finepoint) -> c_int;

    #[no_mangle]
    pub fn GetMapBrick(deck: *mut Level, x: c_float, y: c_float) -> c_uchar;
}
