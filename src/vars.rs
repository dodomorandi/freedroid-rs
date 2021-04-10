use crate::{
    defs::{Droid, Status, ALLBLASTTYPES, MAX_INFLU_POSITION_HISTORY},
    structs::{BlastSpec, BulletSpec, DruidSpec, Finepoint, Gps, Influence},
};

use cstr::cstr;
use sdl::Rect;
use std::{ffi::CStr, os::raw::c_int, ptr::null_mut};

pub static mut ORIG_BLOCK_RECT: Rect = rect! {0, 0, 64, 64}; // not to be rescaled ever!!
pub static mut BLOCK_RECT: Rect = rect! {0, 0, 64, 64};
pub static mut SCREEN_RECT: Rect = rect! {0, 0, 640, 480};
pub static mut USER_RECT: Rect = rect! {0, 0, 0, 0};
pub static mut CLASSIC_USER_RECT: Rect = rect! {32, 150, 9*64, 4*64};
pub static mut FULL_USER_RECT: Rect = rect! {0, 64, 640, 480 - 64};
pub static mut BANNER_RECT: Rect = rect! {0, 0, 640, 64 };
pub static mut PORTRAIT_RECT: Rect = rect! {0, 0, 132, 180}; // for droid-pic display in console
pub static mut CONS_DROID_RECT: Rect = rect! {30, 190, 132, 180};
pub static mut MENU_RECT: Rect = rect! {2*64, 150, 640 - 3*64, 480 - 64};
pub static mut OPTIONS_MENU_RECT: Rect = rect! {232, 0, 0, 0};
pub static mut ORIG_DIGIT_RECT: Rect = rect! {0, 0, 16, 18}; // not to be rescaled!
pub static mut DIGIT_RECT: Rect = rect! {0, 0, 16, 18};
pub static mut CONS_HEADER_RECT: Rect = rect! {75, 64+40, 640 - 80, 135 - 64};
pub static mut CONS_MENU_RECT: Rect = rect! {60, 180, 100, 256};
pub static mut CONS_TEXT_RECT: Rect = rect! {180, 180, 640-185, 480 - 185};
pub static mut CONS_MENU_RECTS: [Rect; 4] = [
    rect! {60, 180, 100, 62},
    rect! {60, 181 + 64, 100, 62},
    rect! {60, 181 + 2*64, 100, 62},
    rect! {60, 181 + 3*64, 100, 62},
];

// Startpos + dimensions of Banner-Texts

pub static mut LEFT_INFO_RECT: Rect = rect! { 26, 44, 0, 0 };
pub static mut RIGHT_INFO_RECT: Rect = rect! {484, 44, 0, 0 };
pub static mut PROGRESS_METER_RECT: Rect = rect! {0, 0, 640, 480};
pub static mut PROGRESS_BAR_RECT: Rect = rect! {446, 155, 22, 111};
pub static mut PROGRESS_TEXT_RECT: Rect = rect! {213, 390, 157, 30};
pub static mut SHIP_EMPTY_COUNTER: c_int = 0; /* counter to Message: you have won(this ship */
pub static mut CONS_MENU_ITEM_RECT: Rect = rect! {0, 0, 0, 0};
pub static mut ME: Influence = Influence {
    ty: Droid::Droid001 as i32,
    status: Status::Transfermode as i32,
    speed: Finepoint { x: 0., y: 0. },
    pos: Finepoint { x: 120., y: 48. },
    health: 100.,
    energy: 100.,
    firewait: 0.,
    phase: 0.,
    timer: 0.,
    last_crysound_time: 0.,
    last_transfer_sound_time: 0.,
    text_visible_time: 0.,
    text_to_be_displayed: null_mut(),
    position_history_ring_buffer: [Gps { x: 0., y: 0., z: 0 }; MAX_INFLU_POSITION_HISTORY],
};

pub static mut DRUIDMAP: *mut DruidSpec = null_mut();
pub static mut BULLETMAP: *mut BulletSpec = null_mut();
pub static mut BLASTMAP: [BlastSpec; ALLBLASTTYPES] = [BlastSpec::default_const(); ALLBLASTTYPES];
pub const CLASS_NAMES: [&CStr; 10] = [
    cstr!("Influence device"),
    cstr!("Disposal robot"),
    cstr!("Servant robot"),
    cstr!("Messenger robot"),
    cstr!("Maintenance robot"),
    cstr!("Crew droid"),
    cstr!("Sentinel droid"),
    cstr!("Battle droid"),
    cstr!("Security droid"),
    cstr!("Command Cyborg"),
];

pub const CLASSES: [&CStr; 11] = [
    cstr!("influence"),
    cstr!("disposal"),
    cstr!("servant"),
    cstr!("messenger"),
    cstr!("maintenance"),
    cstr!("crew"),
    cstr!("sentinel"),
    cstr!("battle"),
    cstr!("security"),
    cstr!("command"),
    cstr!("error"),
];

pub const DRIVE_NAMES: [&CStr; 7] = [
    cstr!("none"),
    cstr!("tracks"),
    cstr!("anti-grav"),
    cstr!("tripedal"),
    cstr!("wheels"),
    cstr!("bipedal"),
    cstr!("error"),
];

pub const SENSOR_NAMES: [&CStr; 7] = [
    cstr!(" - "),
    cstr!("spectral"),
    cstr!("infra-red"),
    cstr!("subsonic"),
    cstr!("ultra-sonic"),
    cstr!("radar"),
    cstr!("error"),
];

pub const BRAIN_NAMES: [&CStr; 4] = [
    cstr!("none"),
    cstr!("neutronic"),
    cstr!("primode"),
    cstr!("error"),
];

// Bullet-names:
pub const WEAPON_NAMES: [&CStr; 7] = [
    cstr!("none"),         // pulse
    cstr!("lasers"),       // single
    cstr!("lasers"),       // Military
    cstr!("disruptor"),    // flash
    cstr!("exterminator"), // exterminator
    cstr!("laser rifle"),  // laser-rifle
    cstr!("error"),
];
