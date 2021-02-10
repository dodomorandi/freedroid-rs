use crate::{
    b_font::BFontInfo,
    defs::{
        AlertNames, ALLBLASTTYPES, ALLSHIPS, MAXBLASTS, MAXBULLETS, MAX_ENEMYS_ON_SHIP, MAX_LEVELS,
        MAX_LIFT_ROWS,
    },
    highscore::HighscoreEntry,
    structs::{
        Blast, BlastSpec, Bullet, BulletSpec, Config, DruidSpec, Enemy, Influence, Level, Ship,
    },
};

use cstr::cstr;
use sdl::video::ll::SDL_Rect;
use std::{ffi::CStr, ptr::null_mut};

extern "C" {
    pub static mut ConfigDir: [i8; 255];
    pub static mut OrigBlock_Rect: SDL_Rect;
    pub static mut Block_Rect: SDL_Rect;
    pub static mut Screen_Rect: SDL_Rect;
    pub static mut User_Rect: SDL_Rect;
    pub static mut Classic_User_Rect: SDL_Rect;
    pub static mut Full_User_Rect: SDL_Rect;
    pub static mut Banner_Rect: SDL_Rect;
    pub static mut Portrait_Rect: SDL_Rect;
    pub static mut Cons_Droid_Rect: SDL_Rect;
    pub static mut Menu_Rect: SDL_Rect;
    pub static mut OptionsMenu_Rect: SDL_Rect;
    pub static mut OrigDigit_Rect: SDL_Rect;
    pub static mut Digit_Rect: SDL_Rect;
    pub static mut FirstDigit_Rect: SDL_Rect;
    pub static mut SecondDigit_Rect: SDL_Rect;
    pub static mut ThirdDigit_Rect: SDL_Rect;
    pub static mut Cons_Header_Rect: SDL_Rect;
    pub static mut Cons_Menu_Rect: SDL_Rect;
    pub static mut Cons_Text_Rect: SDL_Rect;
    pub static mut Cons_Menu_Rects: [SDL_Rect; 4];
    pub static mut LeftInfo_Rect: SDL_Rect;
    pub static mut RightInfo_Rect: SDL_Rect;
    pub static mut ConsMenuItem_Rect: SDL_Rect;
    pub static mut ProgressMeter_Rect: SDL_Rect;
    pub static mut ProgressBar_Rect: SDL_Rect;
    pub static mut ProgressText_Rect: SDL_Rect;
    pub static mut LastRefreshSound: f32;
    pub static mut LastGotIntoBlastSound: f32;
    pub static mut FPSover1: f32;
    pub static mut Alertcolor: [*mut i8; AlertNames::Last as usize];
    pub static mut Shipnames: [*mut i8; ALLSHIPS];
    pub static mut Classname: *mut *mut i8;
    pub static mut Classes: *mut *mut i8;
    pub static mut Weaponnames: *mut *mut i8;
    pub static mut Sensornames: *mut *mut i8;
    pub static mut Brainnames: *mut *mut i8;
    pub static mut Drivenames: *mut *mut i8;
    pub static mut ThisMessageTime: i32;
    pub static mut Me: Influence; /* the influence data */
    pub static mut Druidmap: *mut DruidSpec;
    pub static mut Bulletmap: *mut BulletSpec;
    pub static mut Blastmap: [BlastSpec; ALLBLASTTYPES];
    pub static mut Number_Of_Droid_Types: i32;
    pub static mut PreTakeEnergy: i32;
    pub static mut QuitProgram: i32;
    pub static mut GameOver: i32;
    pub static mut InvincibleMode: i32;
    pub static mut HideInvisibleMap: i32;
    pub static mut AlertLevel: i32;
    pub static mut AlertThreshold: i32; // threshold for FIRST Alert-color (yellow), the others are 2*, 3*..
    pub static mut AlertBonusPerSec: f32; // bonus/sec for FIRST Alert-color, the others are 2*, 3*,...
    pub static mut DeathCount: f32; // a cumulative/draining counter of kills->determines Alert!
    pub static mut DeathCountDrainSpeed: f32; // drain per second
    pub static mut RealScore: f32;
    pub static mut ShowScore: i64;
    pub static mut AllEnemys: [Enemy; MAX_ENEMYS_ON_SHIP];
    pub static mut NumEnemys: i32;
    pub static mut CurLevel: *mut Level; /* the current level data */
    pub static mut curShip: Ship; /* the current ship-data */
    pub static mut AllBullets: [Bullet; MAXBULLETS + 10];
    pub static mut AllBlasts: [Blast; MAXBLASTS + 10];
    pub static mut sound_on: i32; /* Toggle TRUE/FALSE for turning sounds on/off */
    pub static mut debug_level: i32; /* 0=no debug 1=some debug messages 2=...etc */
    /* (currently only 0 or !=0 is implemented) */
    pub static mut show_all_droids: i32; /* display enemys regardless of IsVisible() */
    pub static mut stop_influencer: i32; /* for bullet debugging: stop where u are */
    pub static mut level_rect: [SDL_Rect; MAX_LEVELS]; /* rect's of levels in side-view */
    pub static mut liftrow_rect: [SDL_Rect; MAX_LIFT_ROWS]; /* the lift-row rect's in side-view*/
    pub static mut Highscores: *mut *mut HighscoreEntry;
    pub static mut num_highscores: i32; /* total number of entries in our list (fixed) */
}

pub const INFLUENCE_MODE_NAMES: [&'static CStr; 17] = [
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
