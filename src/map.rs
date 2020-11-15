use crate::{
    defs::Direction,
    global::Me,
    structs::{Finepoint, Level},
};

use std::os::raw::{c_float, c_int, c_uchar};

extern "C" {
    pub fn GetMapBrick(deck: *mut Level, x: c_float, y: c_float) -> c_uchar;
    pub fn FreeShipMemory();
    pub fn AnimateRefresh();
    pub fn IsPassable(x: c_float, y: c_float, check_pos: c_int) -> c_int;
}

/// Determines wether object on x/y is visible to the 001 or not
#[no_mangle]
pub unsafe extern "C" fn IsVisible(objpos: &Finepoint) -> c_int {
    let influ_x = Me.pos.x;
    let influ_y = Me.pos.y;

    let a_x = influ_x - objpos.x;
    let a_y = influ_y - objpos.y;

    let a_len = (a_x * a_x + a_y * a_y).sqrt();
    let mut step_num = a_len * 4.0;

    if step_num == 0. {
        step_num = 1.;
    }
    let step = Finepoint {
        x: a_x / step_num,
        y: a_y / step_num,
    };

    let mut testpos = objpos.clone();

    let step_num = step_num as i32;
    for _ in 1..step_num {
        testpos.x += step.x;
        testpos.y += step.y;

        if IsPassable(testpos.x, testpos.y, Direction::Light as i32) != Direction::Center as i32 {
            return false.into();
        }
    }

    true.into()
}
