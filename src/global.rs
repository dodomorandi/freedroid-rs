use crate::{
    b_font::BFontInfo,
    defs::{ALLBLASTTYPES, MAX_LEVELS, MAX_LIFT_ROWS},
    highscore::HighscoreEntry,
    structs::{BlastSpec, BulletSpec, Config, DruidSpec},
};

use cstr::cstr;
use sdl::video::ll::SDL_Rect;
use std::{ffi::CStr, ptr::null_mut};

extern "C" {
    pub static mut Druidmap: *mut DruidSpec;
    pub static mut Bulletmap: *mut BulletSpec;
    pub static mut Blastmap: [BlastSpec; ALLBLASTTYPES];
    pub static mut HideInvisibleMap: i32;
    /* (currently only 0 or !=0 is implemented) */
    pub static mut level_rect: [SDL_Rect; MAX_LEVELS]; /* rect's of levels in side-view */
    pub static mut liftrow_rect: [SDL_Rect; MAX_LIFT_ROWS]; /* the lift-row rect's in side-view*/
}

pub const INFLUENCE_MODE_NAMES: [&CStr; 17] = [
    cstr!("Mobile"),
    cstr!("Transfer"),
    cstr!("Weapon"),
    cstr!("Captured"),
    cstr!("Complete"),
    cstr!("Rejected"),
    cstr!("Logged In"),
    cstr!("Debriefing"),
    cstr!("Terminated"),
    cstr!("Pause"),
    cstr!("Cheese"),
    cstr!("Elevator"),
    cstr!("Briefing"),
    cstr!("Menu"),
    cstr!("Victory"),
    cstr!("Activate"),
    cstr!("-- OUT --"),
];

#[no_mangle]
pub static mut GameConfig: Config = Config {
    WantedTextVisibleTime: 0.,
    Draw_Framerate: 0,
    Draw_Energy: 0,
    Draw_Position: 0,
    Draw_DeathCount: 0,
    Droid_Talk: 0,
    Current_BG_Music_Volume: 0.,
    Current_Sound_FX_Volume: 0.,
    Current_Gamma_Correction: 0.,
    Theme_Name: [0; 100],
    FullUserRect: 0,
    UseFullscreen: 0,
    TakeoverActivates: 0,
    FireHoldTakeover: 0,
    ShowDecals: 0,
    AllMapVisible: 0,
    scale: 0.,
    HogCPU: 0,
    emptyLevelSpeedup: 0.,
};

#[no_mangle]
pub static mut Menu_BFont: *mut BFontInfo = null_mut();

#[no_mangle]
pub static mut Para_BFont: *mut BFontInfo = null_mut();

#[no_mangle]
pub static mut Highscore_BFont: *mut BFontInfo = null_mut();

#[no_mangle]
pub static mut Font0_BFont: *mut BFontInfo = null_mut();

#[no_mangle]
pub static mut Font1_BFont: *mut BFontInfo = null_mut();

#[no_mangle]
pub static mut Font2_BFont: *mut BFontInfo = null_mut();

#[no_mangle]
pub static mut SkipAFewFrames: i32 = 0;

#[no_mangle]
pub static mut LevelDoorsNotMovedTime: f32 = 0.;

#[no_mangle]
pub static mut Droid_Radius: f32 = 0.;

#[no_mangle]
pub static mut Time_For_Each_Phase_Of_Door_Movement: f32 = 0.;

#[no_mangle]
pub static mut Blast_Radius: f32 = 0.;

#[no_mangle]
pub static mut Blast_Damage_Per_Second: f32 = 0.;

#[no_mangle]
pub static mut CurrentCombatScaleFactor: f32 = 0.;

#[no_mangle]
pub static mut collision_lose_energy_calibrator: f32 = 0.;
