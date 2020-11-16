use crate::{
    defs::{Direction, UnknownVariant, MAX_REFRESHES_ON_LEVEL},
    global::{curShip, CurLevel, Me},
    misc::Frame_Time,
    structs::{Finepoint, Level},
};

use log::trace;
use std::{
    convert::TryFrom,
    os::raw::{c_char, c_float, c_int, c_uchar, c_void},
};

extern "C" {
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

    let mut testpos = *objpos;

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

#[no_mangle]
pub unsafe extern "C" fn GetMapBrick(deck: &Level, x: c_float, y: c_float) -> c_uchar {
    let xx = x.round() as c_int;
    let yy = y.round() as c_int;

    if yy >= deck.ylen || yy < 0 || xx >= deck.xlen || xx < 0 {
        UnknownVariant::Void as c_uchar
    } else {
        *deck.map[usize::try_from(yy).unwrap()].offset(isize::try_from(xx).unwrap()) as c_uchar
    }
}

#[no_mangle]
pub unsafe extern "C" fn FreeShipMemory() {
    curShip
        .AllLevels
        .iter_mut()
        .take(usize::try_from(curShip.num_levels).unwrap())
        .map(|&mut level| level as *mut Level)
        .for_each(|level| {
            FreeLevelMemory(level);
            libc::free(level as *mut c_void);
        });
}

#[no_mangle]
pub unsafe extern "C" fn FreeLevelMemory(level: *mut Level) {
    if level.is_null() {
        return;
    }

    let level = &mut *level;
    libc::free(level.Levelname as *mut c_void);
    libc::free(level.Background_Song_Name as *mut c_void);
    libc::free(level.Level_Enter_Comment as *mut c_void);

    level
        .map
        .iter_mut()
        .take(level.ylen as usize)
        .map(|&mut map| map as *mut c_void)
        .for_each(|map| libc::free(map));
}

#[no_mangle]
pub unsafe extern "C" fn AnimateRefresh() {
    static mut INNER_WAIT_COUNTER: f32 = 0.;

    trace!("AnimateRefresh():  real function call confirmed.");

    INNER_WAIT_COUNTER += Frame_Time() * 10.;

    let cur_level = &*CurLevel;
    cur_level
        .refreshes
        .iter()
        .take(MAX_REFRESHES_ON_LEVEL)
        .take_while(|refresh| refresh.x != -1 && refresh.y != -1)
        .for_each(|refresh| {
            let x = isize::try_from(refresh.x).unwrap();
            let y = usize::try_from(refresh.y).unwrap();

            *cur_level.map[y].offset(x) = (((INNER_WAIT_COUNTER.round() as c_int) % 4)
                + UnknownVariant::Refresh1 as c_int)
                as c_char;
        });

    trace!("AnimateRefresh():  end of function reached.");
}
