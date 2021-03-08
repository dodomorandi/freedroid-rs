#[cfg(feature = "gcw0")]
use crate::input::{KeyIsPressed, KeyIsPressedR};
use crate::{
    input::{cmd_is_active, cmd_is_activeR, KeyIsPressed, KeyIsPressedR, ModIsPressed},
    structs::Point,
    vars::User_Rect,
};

use bitflags::bitflags;
use cstr::cstr;
#[cfg(feature = "gcw0")]
use sdl::keysym::{
    SDLK_BACKSPACE, SDLK_ESCAPE, SDLK_LALT, SDLK_LCTRL, SDLK_LSHIFT, SDLK_RETURN, SDLK_SPACE,
    SDLK_TAB,
};
use sdl::{
    event::Mod,
    keysym::SDLK_RETURN,
    sdl::Rect,
    video::ll::{SDL_FreeSurface, SDL_Surface},
};
use std::{convert::TryFrom, ffi::CStr, os::raw::c_int};

pub const MAX_THEMES: usize = 100;

pub const JOY_MAX_VAL: usize = 32767; // maximal amplitude of joystick axis values

pub const RESET: c_int = 0x01;
pub const UPDATE: c_int = 0x02;
pub const INIT_ONLY: usize = 0x04;
pub const FREE_ONLY: usize = 0x08;

pub const DROID_ROTATION_TIME: f32 = 3.0;
pub const NUM_DECAL_PICS: usize = 2;

#[inline]
pub unsafe fn get_user_center() -> Rect {
    let Rect { x, y, w, h } = User_Rect;
    Rect {
        x: x + (w / 2) as i16,
        y: y + (h / 2) as i16,
        w,
        h,
    }
}

#[inline]
pub fn scale_rect(rect: &mut Rect, scale: f32) {
    rect.x = (f32::from(rect.x) * scale) as i16;
    rect.y = (f32::from(rect.y) * scale) as i16;
    rect.w = (f32::from(rect.w) * scale) as u16;
    rect.h = (f32::from(rect.h) * scale) as u16;
}

#[inline]
pub fn scale_point(point: &mut Point, scale: f32) {
    let scale: f64 = scale.into();
    point.x = (f64::from(point.x) * scale) as i32;
    point.y = (f64::from(point.y) * scale) as i32;
}

// #define Set_Rect(rect, xx, yy, ww, hh) do {\
// (rect).x = (xx); (rect).y = (yy); (rect).w = (ww); (rect).h = (hh); } while(0)

// #define Copy_Rect(src, dst) do {\
// (dst).x = (src).x; (dst).y = (src).y; (dst).w = (src).w; (dst).h = (src).h; } while(0)

#[inline]
pub unsafe fn free_if_unused(surface: *mut SDL_Surface) {
    if !surface.is_null() {
        SDL_FreeSurface(surface);
    }
}

// ----------------------------------------
// some input-related defines and macros

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum PointerStates {
    MouseUp = sdl::event::Key::Last as isize + 1,
    MouseRight,
    MouseDown,
    MouseLeft,
    MouseButton1,
    MouseButton2,
    MouseButton3,
    MouseWheelup,
    MouseWheeldown,

    JoyUp,
    JoyRight,
    JoyDown,
    JoyLeft,
    JoyButton1,
    JoyButton2,
    JoyButton3,
    JoyButton4,

    Last,
}

//--------------------------------------------------
// here come the actual game-"commands"
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum Cmds {
    Up = 0,
    Down,
    Left,
    Right,
    Fire,
    Activate,
    Takeover,
    Quit,
    Pause,
    Screenshot,
    Fullscreen,
    Menu,
    Back,
    Last,
}

//--------------------------------------------------

#[inline]
pub unsafe fn ReturnPressed() -> bool {
    KeyIsPressed(SDLK_RETURN as i32)
}

#[inline]
pub unsafe fn ReturnPressedR() -> bool {
    KeyIsPressedR(SDLK_RETURN as i32)
}

#[inline]
pub unsafe fn ShiftPressed() -> bool {
    ModIsPressed(Mod::LShift as u32 | Mod::RShift as u32)
}

#[inline]
pub unsafe fn AltPressed() -> bool {
    ModIsPressed(Mod::LAlt as u32 | Mod::RAlt as u32)
}

#[inline]
pub unsafe fn CtrlPressed() -> bool {
    ModIsPressed(Mod::LCtrl as u32 | Mod::RCtrl as u32)
}

// #define MouseLeftPressed() KeyIsPressed(MOUSE_BUTTON1)
// #define MouseLeftPressedR() KeyIsPressedR(MOUSE_BUTTON1)
// #define MouseRightPressed() KeyIsPressed(MOUSE_BUTTON2)
// #define MouseRightPressedR() KeyIsPressedR(MOUSE_BUTTON2)

// #define EscapePressed() KeyIsPressed(SDLK_ESCAPE)
// #define SpacePressed() KeyIsPressed(SDLK_SPACE)
// #define EscapePressedR() KeyIsPressedR (SDLK_ESCAPE)
// #define SpacePressedR() KeyIsPressedR (SDLK_SPACE)

#[cfg(feature = "gcw0")]
#[inline]
pub unsafe fn gcw0_a_pressed() -> bool {
    KeyIsPressed(SDLK_LCTRL as c_int)
}

#[cfg(feature = "gcw0")]
#[inline]
pub unsafe fn gcw0_b_pressed() -> bool {
    KeyIsPressed(SDLK_LALT as c_int)
}

#[cfg(feature = "gcw0")]
#[inline]
pub unsafe fn gcw0_x_pressed() -> bool {
    KeyIsPressed(SDLK_LSHIFT as c_int)
}

#[cfg(feature = "gcw0")]
#[inline]
pub unsafe fn gcw0_y_pressed() -> bool {
    KeyIsPressed(SDLK_SPACE as c_int)
}

#[cfg(feature = "gcw0")]
#[inline]
pub unsafe fn gcw0_rs_pressed() -> bool {
    KeyIsPressed(SDLK_BACKSPACE as c_int)
}

#[cfg(feature = "gcw0")]
#[inline]
pub unsafe fn gcw0_ls_pressed() -> bool {
    KeyIsPressed(SDLK_TAB as c_int)
}

#[cfg(feature = "gcw0")]
#[inline]
pub unsafe fn gcw0_start_pressed() -> bool {
    KeyIsPressed(SDLK_RETURN as c_int)
}

#[cfg(feature = "gcw0")]
#[inline]
pub unsafe fn gcw0_select_pressed() -> bool {
    KeyIsPressed(SDLK_ESCAPE as c_int)
}

#[cfg(feature = "gcw0")]
#[inline]
pub unsafe fn gcw0_any_button_pressed() -> bool {
    gcw0_a_pressed()
        || gcw0_b_pressed()
        || gcw0_x_pressed()
        || gcw0_y_pressed()
        || gcw0_ls_pressed()
        || gcw0_rs_pressed()
        || gcw0_start_pressed()
        || gcw0_select_pressed()
}

#[cfg(feature = "gcw0")]
#[inline]
pub unsafe fn gcw0_a_pressed_r() -> bool {
    KeyIsPressedR(SDLK_LCTRL as c_int)
}

#[cfg(feature = "gcw0")]
#[inline]
pub unsafe fn gcw0_b_pressed_r() -> bool {
    KeyIsPressedR(SDLK_LALT as c_int)
}

#[cfg(feature = "gcw0")]
#[inline]
pub unsafe fn gcw0_x_pressed_r() -> bool {
    KeyIsPressedR(SDLK_LSHIFT as c_int)
}

#[cfg(feature = "gcw0")]
#[inline]
pub unsafe fn gcw0_y_pressed_r() -> bool {
    KeyIsPressedR(SDLK_SPACE as c_int)
}

#[cfg(feature = "gcw0")]
#[inline]
pub unsafe fn gcw0_rs_pressed_r() -> bool {
    KeyIsPressed(SDLK_BACKSPACE as c_int)
}

#[cfg(feature = "gcw0")]
#[inline]
pub unsafe fn gcw0_ls_pressed_r() -> bool {
    KeyIsPressed(SDLK_TAB as c_int)
}

#[cfg(feature = "gcw0")]
#[inline]
pub unsafe fn gcw0_start_pressed_r() -> bool {
    KeyIsPressed(SDLK_RETURN as c_int)
}

#[cfg(feature = "gcw0")]
#[inline]
pub unsafe fn gcw0_select_pressed_r() -> bool {
    KeyIsPressed(SDLK_ESCAPE as c_int)
}

#[cfg(feature = "gcw0")]
#[inline]
pub unsafe fn gcw0_any_button_pressed_r() -> bool {
    gcw0_a_pressed_r()
        || gcw0_b_pressed_r()
        || gcw0_x_pressed_r()
        || gcw0_y_pressed_r()
        || gcw0_ls_pressed_r()
        || gcw0_rs_pressed_r()
        || gcw0_start_pressed_r()
        || gcw0_select_pressed_r()
}

#[inline]
pub unsafe fn UpPressed() -> bool {
    cmd_is_active(Cmds::Up)
}

#[inline]
pub unsafe fn DownPressed() -> bool {
    cmd_is_active(Cmds::Down)
}

#[inline]
pub unsafe fn LeftPressed() -> bool {
    cmd_is_active(Cmds::Left)
}

#[inline]
pub unsafe fn RightPressed() -> bool {
    cmd_is_active(Cmds::Right)
}

#[inline]
pub unsafe fn FirePressed() -> bool {
    cmd_is_active(Cmds::Fire)
}

#[inline]
pub unsafe fn FirePressedR() -> bool {
    cmd_is_activeR(Cmds::Fire)
}

#[inline]
pub unsafe fn UpPressedR() -> bool {
    cmd_is_activeR(Cmds::Up)
}

#[inline]
pub unsafe fn DownPressedR() -> bool {
    cmd_is_activeR(Cmds::Down)
}

#[inline]
pub unsafe fn LeftPressedR() -> bool {
    cmd_is_activeR(Cmds::Left)
}

#[inline]
pub unsafe fn RightPressedR() -> bool {
    cmd_is_activeR(Cmds::Right)
}

// #define AnyCmdActive() (cmd_is_active(CMD_FIRE) || cmd_is_active(CMD_ACTIVATE) || cmd_is_active(CMD_TAKEOVER) )
// #define AnyCmdActiveR() (cmd_is_activeR(CMD_FIRE) || cmd_is_activeR(CMD_ACTIVATE) || cmd_is_activeR(CMD_TAKEOVER) )

// ----------------------------------------

bitflags! {
    #[repr(C)]
    pub struct MenuAction: i32 {
        const INFO = 0b0000_0000_0001;
        const BACK = 0b0000_0000_0010;
        const CLICK = 0b0000_0000_0100;
        const LEFT = 0b0000_0000_1000;
        const RIGHT = 0b0000_0001_0000;
        const UP = 0b0000_0010_0000;
        const DOWN = 0b0000_0100_0000;
        const DELETE = 0b0000_1000_0000;
        const UP_WHEEL = 0b0001_0000_0000;
        const DOWN_WHEEL = 0b0010_0000_0000;
        const LAST = 0b0100_0000_0000;
    }
}

pub const COLLISION_STEPSIZE: f64 = 0.1;

/* ************************************************************
 * Highscore related defines
 *************************************************************/
pub const HS_BACKGROUND_FILE: &str = "transfer.jpg";
pub const HS_BACKGROUND_FILE_C: &CStr = cstr!("transfer.jpg");
pub const HS_EMPTY_ENTRY: &str = "--- empty ---";
pub const MAX_NAME_LEN: usize = 15; /* max len of highscore name entry */
pub const MAX_HIGHSCORES: usize = 10; /* only keep Top10 */
pub const DATE_LEN: usize = 10; /* reserved for the date-string */
//***************************************************************

// find_file(): use current-theme subdir in search or not
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum Themed {
    NoTheme = 0,
    UseTheme,
}
// find_file(): how important is the file in question:
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum Criticality {
    Ignore = 0, // ignore if not found and return NULL
    WarnOnly,   // warn if not found and return NULL
    Critical,   // Error-message and Terminate
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InvalidCriticality;

impl TryFrom<i32> for Criticality {
    type Error = InvalidCriticality;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        use Criticality::*;
        match value {
            0 => Ok(Ignore),
            1 => Ok(WarnOnly),
            2 => Ok(Critical),
            _ => Err(InvalidCriticality),
        }
    }
}

// The flags for DisplayBanner are:
bitflags! {
    pub struct DisplayBannerFlags: u8 {
        const FORCE_UPDATE=1;
        const DONT_TOUCH_TEXT=2;
        const NO_SDL_UPDATE=4;
    }
}

// The flags for AssembleCombatWindow are:
bitflags! {
    pub struct AssembleCombatWindowFlags: u8 {
        const ONLY_SHOW_MAP = 0x01;
        const DO_SCREEN_UPDATE = 0x02;
        const SHOW_FULL_MAP = 0x04;
    }
}

// symbolic Alert-names
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum AlertNames {
    Green = 0,
    Yellow,
    Amber,
    Red,
    Last,
}

// **********************************************************************
// Constants for Paths and names of Data-files
// the root "FD_DATADIR" should be defined in the Makefile as $(pkgdatadir)
// if not, we set it here:
// #ifndef FD_DATADIR

#[cfg(target_os = "macosx")]
pub const FD_DATADIR: &str = "FreeDroid.app/Contents/Resources"; // our local fallback

#[cfg(not(target_os = "macosx"))]
pub const FD_DATADIR: &str = "."; // our local fallback

// #endif // !FD_DATADIR

// #ifndef LOCAL_DATADIR
pub const LOCAL_DATADIR: &str = ".."; // local fallback
                                      // #endif

pub const GRAPHICS_DIR: &str = "graphics/";
pub const GRAPHICS_DIR_C: &CStr = cstr!("graphics/");
pub const SOUND_DIR: &str = "sound/";
pub const SOUND_DIR_C: &CStr = cstr!("sound/");
pub const MAP_DIR: &str = "map/";
pub const MAP_DIR_C: &CStr = cstr!("map/");

pub const MAP_BLOCK_FILE: &str = "map_blocks.png";
pub const MAP_BLOCK_FILE_C: &CStr = cstr!("map_blocks.png");
pub const DROID_BLOCK_FILE: &str = "droids.png";
pub const DROID_BLOCK_FILE_C: &CStr = cstr!("droids.png");
pub const BULLET_BLOCK_FILE: &str = "bullet.png";
pub const BULLET_BLOCK_FILE_C: &CStr = cstr!("bullet.png");
pub const BLAST_BLOCK_FILE: &str = "blast.png";
pub const BLAST_BLOCK_FILE_C: &CStr = cstr!("blast.png");
pub const DIGIT_BLOCK_FILE: &str = "digits.png";
pub const DIGIT_BLOCK_FILE_C: &CStr = cstr!("digits.png");

pub const BANNER_BLOCK_FILE: &str = "banner.png";
pub const BANNER_BLOCK_FILE_C: &CStr = cstr!("banner.png");
pub const TITLE_PIC_FILE: &str = "title.jpg";
pub const TITLE_PIC_FILE_C: &CStr = cstr!("title.jpg");
pub const CONSOLE_PIC_FILE: &str = "console_fg.png";
pub const CONSOLE_PIC_FILE_C: &CStr = cstr!("console_fg.png");
pub const CONSOLE_BG_PIC1_FILE: &str = "console_bg1.jpg";
pub const CONSOLE_BG_PIC1_FILE_C: &CStr = cstr!("console_bg1.jpg");
pub const CONSOLE_BG_PIC2_FILE: &str = "console_bg2.jpg";
pub const CONSOLE_BG_PIC2_FILE_C: &CStr = cstr!("console_bg2.jpg");
pub const TAKEOVER_BG_PIC_FILE: &str = "takeover_bg.jpg";
pub const TAKEOVER_BG_PIC_FILE_C: &CStr = cstr!("takeover_bg.jpg");
pub const CREDITS_PIC_FILE: &str = "credits.jpg";
pub const CREDITS_PIC_FILE_C: &CStr = cstr!("credits.jpg");

pub const SHIP_ON_PIC_FILE: &str = "ship_on.png";
pub const SHIP_ON_PIC_FILE_C: &CStr = cstr!("ship_on.png");
pub const SHIP_OFF_PIC_FILE: &str = "ship_off.png";
pub const SHIP_OFF_PIC_FILE_C: &CStr = cstr!("ship_off.png");

pub const PROGRESS_METER_FILE: &str = "progress_meter.png";
pub const PROGRESS_METER_FILE_C: &CStr = cstr!("progress_meter.png");
pub const PROGRESS_FILLER_FILE: &str = "progress_filler.png";
pub const PROGRESS_FILLER_FILE_C: &CStr = cstr!("progress_filler.png");

pub const STANDARD_MISSION: &str = "Paradroid.mission";
pub const STANDARD_MISSION_C: &CStr = cstr!("Paradroid.mission");
pub const NEW_MISSION: &str = "CleanPrivateGoodsStorageCellar.mission";
pub const NEW_MISSION_C: &CStr = cstr!("CleanPrivateGoodsStorageCellar.mission");

pub const PARA_FONT_FILE: &str = "parafont.png";
pub const PARA_FONT_FILE_C: &CStr = cstr!("parafont.png");
pub const FONT0_FILE: &str = "font05.png";
pub const FONT0_FILE_C: &CStr = cstr!("font05.png");
pub const FONT1_FILE: &str = "font05_green.png";
pub const FONT1_FILE_C: &CStr = cstr!("font05_green.png");
pub const FONT2_FILE: &str = "font05_red.png";
pub const FONT2_FILE_C: &CStr = cstr!("font05_red.png");
pub const ICON_FILE: &str = "paraicon_48x48.png";
pub const ICON_FILE_C: &CStr = cstr!("paraicon_48x48.png");

// **********************************************************************

pub const DIGITNUMBER: usize = 10;

pub const TEXT_STRETCH: f64 = 1.2;
pub const LEFT_TEXT_LEN: usize = 10;
pub const RIGHT_TEXT_LEN: usize = 6;

pub const BULLET_BULLET_COLLISION_DIST: f64 = 10.0 / 64.0;
pub const BULLET_COLL_DIST2: f64 = 0.0244140625;
// **********************************************************************
//
//

// The following is the definition of the sound file names used in freedroid
// DO NOT EVER CHANGE THE ORDER OF APPEARENCE IN THIS LIST PLEASE!!!!!
// The order of appearance here should match the order of appearance
// in the SoundSampleFilenames definition located in sound.c!
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum Sound {
    Error = 0,
    Blast,
    Collision,
    CollisionGotDamaged,
    CollisionDamagedEnemy,
    GotIntoBlast,
    MoveElevator,
    Refresh,
    LeaveElevator,
    EnterElevator,
    ThouArtDefeated,
    GotHit,
    TakeoverSetCapsule,
    MenuItemSelected,
    MoveMenuPosition,
    TakeoverGameWon,
    TakeoverGameDeadlock,
    TakeoverGameLost,
    FireBulletPulse,
    FireBulletSinglePulse,
    FireBulletMilitary,
    FireBulletFlash,
    FireBulletExterminator,
    FireBulletLaserRifle,
    Cry,
    Transfer,
    Countdown,
    Endcountdown,
    Influexplosion,
    WhiteNoise,
    Alert,
    Screenshot,
    All, // marks the last entry always!
}

// choose background music by level-color:
// if filename_raw==BYCOLOR then chose bg_music[color]
pub const BYCOLOR: &CStr = cstr!("BYCOLOR");

// The sounds when the influencers energy is low or when he is in transfer mode
// occur periodically.  These constants specify which intervals are to be used
// for these periodic happenings...
pub const CRY_SOUND_INTERVAL: f32 = 2.;
pub const TRANSFER_SOUND_INTERVAL: f32 = 1.1;

// **********************************************************************

pub const ERR: i8 = -1;
pub const OK: i8 = 0;

/* Ship-Elevator Picture */

pub const DIRECTIONS: usize = 8;

pub const ALLSHIPS: usize = 4;
pub const ENEMYPHASES: usize = 8;
pub const DROID_PHASES: usize = ENEMYPHASES;

pub const WAIT_LEVELEMPTY: f64 = 0.5; /* warte bevor Graufaerben (in seconds)*/
pub const SLOWMO_FACTOR: f32 = 0.33; // slow-motion effect on last blast when level is going empty
pub const WAIT_AFTER_KILLED: u32 = 2000; // time (in ms) to wait and still display pictures after the destruction of
pub const SHOW_WAIT: u32 = 3500; // std amount of time to show something
                                 // the players droid.  This is now measured in seconds and can be a float
pub const WAIT_SHIPEMPTY: usize = 20;
pub const WAIT_TRANSFERMODE: f64 = 0.3; /* this is a "float" indicating the number of seconds the influence
                                        stand still with space pressed, before switching into transfermode
                                        This variable describes the amount in SECONDS */
pub const WAIT_COLLISION: usize = 1; // after a little collision with influ, enemys hold position for a while
                                     // this variable describes the amount of time in SECONDS
pub const ENEMYMAXWAIT: f64 = 2.0; // after each robot has reached its current destination waypoint is waits a
                                   // while.  This variable describes the amount of time in SECONDS.  However,
                                   // the final wait time is a random number within [0,ENEMYMAXWAIT].
pub const FLASH_DURATION: f32 = 0.1; // in seconds

/* direction definitions (fireing bullets and testing blockedness of positions) */
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum Direction {
    Oben = 0,
    Rechtsoben,
    Rechts,
    Rechtsunten,
    Unten,
    Linksunten,
    Links,
    Linksoben,
    Center,
    Light, /* special: checking passability for light, not for a checkpos */
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidDirection;

macro_rules! direction_try_from {
    () => {};

    ($ty:ty $(, $( $rest:ty ),* )? $(,)* ) => {
        impl TryFrom<$ty> for Direction {
            type Error = InvalidDirection;

            fn try_from(value: $ty) -> Result<Self, Self::Error> {
                use Direction::*;
                Ok(match value {
                    0 => Oben,
                    1 => Rechtsoben,
                    2 => Rechts,
                    3 => Rechtsunten,
                    4 => Unten,
                    5 => Linksunten,
                    6 => Links,
                    7 => Linksoben,
                    8 => Center,
                    9 => Light,
                    _ => return Err(InvalidDirection),
                })
            }
        }

        $(
            direction_try_from!($($rest),*);
        )?
    };
}
direction_try_from!(i8, u8, i16, u16, i32, u32);

/* Maximal number of ... */

pub const NUM_MAP_BLOCKS: usize = 51; // total number of map-blocks
pub const NUM_COLORS: usize = 7; // how many different level colorings?/different tilesets?

// const #define: usize = ALLBULLETTYPES;		4	/* number of bullet-types */
pub const ALLBLASTTYPES: usize = 2; /* number of different exposions */

pub const MAXBULLETS: usize = 100; /* maximum possible Bullets in the air */
pub const MAXBLASTS: usize = 100; /* max. possible Blasts visible */
pub const AGGRESSIONMAX: usize = 100;
pub const ROBOT_MAX_WAIT_BETWEEN_SHOTS: usize = 5; // how long shoud each droid wait at most until
                                                   // is considers fireing again?

/* Map-related defines:
    WARNING leave them here, they are required in struct.h
*/
pub const MAX_WP_CONNECTIONS: usize = 12;
pub const MAX_MAP_ROWS: usize = 255;
pub const MAX_MAP_COLS: usize = 255;
pub const MAX_ENEMYS_ON_SHIP: usize = 300;
pub const MAX_CHAT_KEYWORDS_PER_DROID: usize = 30;
pub const MAX_INFLU_POSITION_HISTORY: usize = 100;

pub const MAX_LIFTS: usize = 50; /* actually the entries to the lifts */
pub const MAX_LEVELS: usize = 29; /* don't change this easily */
/* corresponds to a reserved palette range ! */
pub const MAX_LIFT_ROWS: usize = 15; /* the different lift "rows" */
/* don't change this easily */
/* corresponds to a reserved palette range !*/
pub const MAX_LEVEL_RECTS: usize = 20; // how many rects compose a level
pub const MAX_EVENT_TRIGGERS: usize = 20; // how many event triggers at most to allow
pub const MAX_TRIGGERED_ACTIONS: usize = 20; // how many triggerable actions to allow at most

pub const MAXWAYPOINTS: usize = 100;
pub const MAX_DOORS_ON_LEVEL: usize = 60;
pub const MAX_REFRESHES_ON_LEVEL: usize = 40;
pub const MAX_ALERTS_ON_LEVEL: usize = 40;
pub const MAX_TELEPORTERS_ON_LEVEL: usize = 10;

pub const MAX_PHASES_IN_A_BULLET: usize = 12;
pub const MAX_STEPS_IN_GIVEN_COURSE: usize = 1000;

pub const BREMSDREHUNG: usize = 3; /* warte 3*, bevor Influencer weitergedreht wird */

/* Wegstossgeschw. von Tueren u.ae. */
// NORMALISATION #define PUSHSPEED 2
pub const PUSHSPEED: usize = 2;

/* Schusstypen */
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum BulletKind {
    Pulse = 0,
    SinglePulse,
    Military,
    Flash,
    Exterminator,
    LaserRifle,
}

/* Explosionstypen */
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum Explosion {
    Bulletblast = 0,
    Druidblast,
    Rejectblast,
}

pub const BLINKENERGY: f32 = 25.;

/* Druidtypen */
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum Droid {
    Droid001 = 0, /* You will know why are the numbers there, when you */
    Droid123 = 1, /* enter the crew of a level !! */
    Droid139 = 2,
    Droid247 = 3,
    Droid249 = 4,
    Droid296 = 5,
    Droid302 = 6,
    Droid329 = 7,
    Droid420 = 8,
    Droid476 = 9,
    Droid493 = 10,
    Droid516 = 11,
    Droid571 = 12,
    Droid598 = 13,
    Droid614 = 14,
    Droid615 = 15,
    Droid629 = 16,
    Droid711 = 17,
    Droid742 = 18,
    Droid751 = 19,
    Droid821 = 20,
    Droid834 = 21,
    Droid883 = 22,
    Droid999 = 23,
    NumDroids,
}

/* Status- Werte der Druids */
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum Status {
    Mobile,
    Transfermode,
    Weapon,
    Captured,
    Complete,
    Rejected,
    Console,
    Debriefing,
    Terminated,
    Pause,
    Cheese,
    Elevator,
    Briefing,
    Menu,
    Victory,
    Activate,
    Out,
}

pub const DECKCOMPLETEBONUS: usize = 500;

/* Konstanten die die Kartenwerte anschaulich machen */
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub enum MapTile {
    Floor,
    EckLu,
    TU,
    EckRu,
    TL,
    Kreuz,
    TR,
    EckLo,
    TO,
    EckRo,
    HWall,
    VWall,
    Invisible,
    Block1,
    Block2,
    Block3,
    Block4,
    Block5,
    HZutuere,
    HHalbtuere1,
    HHalbtuere2,
    HHalbtuere3,
    HGanztuere,
    KonsoleL,
    KonsoleR,
    KonsoleO,
    KonsoleU,
    VZutuere,
    VHalbtuere1,
    VHalbtuere2,
    VHalbtuere3,
    VGanztuere,
    Lift,
    Void,
    Refresh1,
    Refresh2,
    Refresh3,
    Refresh4,
    AlertGreen,
    AlertYellow,
    AlertAmber,
    AlertRed,
    Unused2,
    FineGrid,
    NumMapTiles,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidMapTile;

macro_rules! impl_try_from_map_tile {
    () => {};

    ($ty:ty $(, $rest:ty)* $(,)*) => {
        impl TryFrom<$ty> for MapTile {
            type Error = InvalidMapTile;

            fn try_from(value: $ty) -> Result<Self, Self::Error> {
                use MapTile::*;
                Ok(match value {
                    0 => Floor,
                    1 => EckLu,
                    2 => TU,
                    3 => EckRu,
                    4 => TL,
                    5 => Kreuz,
                    6 => TR,
                    7 => EckLo,
                    8 => TO,
                    9 => EckRo,
                    10 => HWall,
                    11 => VWall,
                    12 => Invisible,
                    13 => Block1,
                    14 => Block2,
                    15 => Block3,
                    16 => Block4,
                    17 => Block5,
                    18 => HZutuere,
                    19 => HHalbtuere1,
                    20 => HHalbtuere2,
                    21 => HHalbtuere3,
                    22 => HGanztuere,
                    23 => KonsoleL,
                    24 => KonsoleR,
                    25 => KonsoleO,
                    26 => KonsoleU,
                    27 => VZutuere,
                    28 => VHalbtuere1,
                    29 => VHalbtuere2,
                    30 => VHalbtuere3,
                    31 => VGanztuere,
                    32 => Lift,
                    33 => Void,
                    34 => Refresh1,
                    35 => Refresh2,
                    36 => Refresh3,
                    37 => Refresh4,
                    38 => AlertGreen,
                    39 => AlertYellow,
                    40 => AlertAmber,
                    41 => AlertRed,
                    42 => Unused2,
                    43 => FineGrid,
                    44 => NumMapTiles,
                    _ => return Err(InvalidMapTile),
                })
            }
        }

        $(
            impl_try_from_map_tile!($rest);
        )*
    };
}

impl_try_from_map_tile!(i8, u8, c_int);
