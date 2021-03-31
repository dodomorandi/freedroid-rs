use crate::{
    defs::{AlertNames, Droid, Status, ALLBLASTTYPES, MAX_INFLU_POSITION_HISTORY},
    structs::{BlastSpec, BulletSpec, DruidSpec, Finepoint, Gps, Influence},
};

use cstr::cstr;
use sdl::Rect;
use std::{ffi::CStr, os::raw::c_int, ptr::null_mut};

#[no_mangle]
pub static mut OrigBlock_Rect: Rect = rect! {0, 0, 64, 64}; // not to be rescaled ever!!

#[no_mangle]
pub static mut Block_Rect: Rect = rect! {0, 0, 64, 64};

#[no_mangle]
pub static mut Screen_Rect: Rect = rect! {0, 0, 640, 480};

#[no_mangle]
pub static mut User_Rect: Rect = rect! {0, 0, 0, 0};

#[no_mangle]
pub static mut Classic_User_Rect: Rect = rect! {32, 150, 9*64, 4*64};

#[no_mangle]
pub static mut Full_User_Rect: Rect = rect! {0, 64, 640, 480 - 64};

#[no_mangle]
pub static mut Banner_Rect: Rect = rect! {0, 0, 640, 64 };

#[no_mangle]
pub static mut Portrait_Rect: Rect = rect! {0, 0, 132, 180}; // for droid-pic display in console

#[no_mangle]
pub static mut Cons_Droid_Rect: Rect = rect! {30, 190, 132, 180};

#[no_mangle]
pub static mut Menu_Rect: Rect = rect! {2*64, 150, 640 - 3*64, 480 - 64};

#[no_mangle]
pub static mut OptionsMenu_Rect: Rect = rect! {232, 0, 0, 0};

#[no_mangle]
pub static mut OrigDigit_Rect: Rect = rect! {0, 0, 16, 18}; // not to be rescaled!

#[no_mangle]
pub static mut Digit_Rect: Rect = rect! {0, 0, 16, 18};

#[no_mangle]
pub static mut Cons_Header_Rect: Rect = rect! {75, 64+40, 640 - 80, 135 - 64};

#[no_mangle]
pub static mut Cons_Menu_Rect: Rect = rect! {60, 180, 100, 256};

#[no_mangle]
pub static mut Cons_Text_Rect: Rect = rect! {180, 180, 640-185, 480 - 185};

#[no_mangle]
pub static mut Cons_Menu_Rects: [Rect; 4] = [
    rect! {60, 180, 100, 62},
    rect! {60, 181 + 64, 100, 62},
    rect! {60, 181 + 2*64, 100, 62},
    rect! {60, 181 + 3*64, 100, 62},
];

// Startpos + dimensions of Banner-Texts
#[no_mangle]
pub static mut LeftInfo_Rect: Rect = rect! { 26, 44, 0, 0 };

#[no_mangle]
pub static mut RightInfo_Rect: Rect = rect! {484, 44, 0, 0 };

#[no_mangle]
pub static mut ProgressMeter_Rect: Rect = rect! {0, 0, 640, 480};

#[no_mangle]
pub static mut ProgressBar_Rect: Rect = rect! {446, 155, 22, 111};

#[no_mangle]
pub static mut ProgressText_Rect: Rect = rect! {213, 390, 157, 30};

#[no_mangle]
pub static mut ShipEmptyCounter: c_int = 0; /* counter to Message: you have won(this ship */

#[no_mangle]
pub static mut ConsMenuItem_Rect: Rect = rect! {0, 0, 0, 0};

#[no_mangle]
pub static mut Me: Influence = Influence {
    ty: Droid::Droid001 as i32,
    status: Status::Transfermode as i32,
    speed: Finepoint { x: 0., y: 0. },
    pos: Finepoint { x: 120., y: 48. },
    health: 100.,
    energy: 100.,
    firewait: 0.,
    phase: 0.,
    timer: 0.,
    LastCrysoundTime: 0.,
    LastTransferSoundTime: 0.,
    TextVisibleTime: 0.,
    TextToBeDisplayed: null_mut(),
    Position_History_Ring_Buffer: [Gps { x: 0., y: 0., z: 0 }; MAX_INFLU_POSITION_HISTORY],
};

#[no_mangle]
pub static mut Druidmap: *mut DruidSpec = null_mut();

#[no_mangle]
pub static mut Bulletmap: *mut BulletSpec = null_mut();

#[no_mangle]
pub static mut Blastmap: [BlastSpec; ALLBLASTTYPES] = [BlastSpec::default_const(); ALLBLASTTYPES];

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

pub const SHIP_NAMES: [&CStr; 3] = [cstr!("Paradroid"), cstr!("Metahawk"), cstr!("Graftgold")];

pub const ALERT_COLORS: [&CStr; AlertNames::Last as usize] = [
    cstr!("green"),
    cstr!("yellow"),
    cstr!("amber"),
    cstr!("red"),
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
