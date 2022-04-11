#[cfg(feature = "gcw0")]
use crate::input::{key_is_pressed, key_is_pressed_r};
use crate::{input::Input, structs::Point, vars::Vars, Data, Sdl};

use bitflags::bitflags;
use cstr::cstr;
#[cfg(feature = "gcw0")]
use sdl::keysym::{
    SDLKey_SDLK_BACKSPACE, SDLKey_SDLK_LALT, SDLKey_SDLK_LCTRL, SDLKey_SDLK_LSHIFT,
    SDLKey_SDLK_RETURN, SDLKey_SDLK_TAB,
};
use sdl_sys::{
    SDLKey_SDLK_ESCAPE, SDLKey_SDLK_LAST, SDLKey_SDLK_RETURN, SDLKey_SDLK_SPACE, SDLMod_KMOD_LALT,
    SDLMod_KMOD_LCTRL, SDLMod_KMOD_LSHIFT, SDLMod_KMOD_RALT, SDLMod_KMOD_RCTRL, SDLMod_KMOD_RSHIFT,
};
use std::{cell::Cell, ffi::CStr, fmt, os::raw::c_int};

pub const MAX_THEMES: usize = 100;

pub const RESET: c_int = 0x01;
pub const UPDATE: c_int = 0x02;
pub const INIT_ONLY: usize = 0x04;
pub const FREE_ONLY: usize = 0x08;

pub const DROID_ROTATION_TIME: f32 = 3.0;
pub const NUM_DECAL_PICS: usize = 2;

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

// ----------------------------------------
// some input-related defines and macros

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[allow(dead_code)]
pub enum PointerStates {
    MouseUp = SDLKey_SDLK_LAST as isize + 1,
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

#[cfg(feature = "gcw0")]
#[inline]
pub fn gcw0_a_pressed() -> bool {
    KeyIsPressed(SDLKey_SDLK_LCTRL as c_int)
}

#[cfg(feature = "gcw0")]
#[inline]
pub fn gcw0_b_pressed() -> bool {
    KeyIsPressed(SDLKey_SDLK_LALT as c_int)
}

#[cfg(feature = "gcw0")]
#[inline]
pub fn gcw0_x_pressed() -> bool {
    KeyIsPressed(SDLKey_SDLK_LSHIFT as c_int)
}

#[cfg(feature = "gcw0")]
#[inline]
pub fn gcw0_y_pressed() -> bool {
    KeyIsPressed(SDLKey_SDLK_SPACE as c_int)
}

#[cfg(feature = "gcw0")]
#[inline]
pub fn gcw0_rs_pressed() -> bool {
    KeyIsPressed(SDLKey_SDLK_BACKSPACE as c_int)
}

#[cfg(feature = "gcw0")]
#[inline]
pub fn gcw0_ls_pressed() -> bool {
    KeyIsPressed(SDLKey_SDLK_TAB as c_int)
}

#[cfg(feature = "gcw0")]
#[inline]
pub fn gcw0_start_pressed() -> bool {
    KeyIsPressed(SDLKey_SDLK_RETURN as c_int)
}

#[cfg(feature = "gcw0")]
#[inline]
pub fn gcw0_select_pressed() -> bool {
    KeyIsPressed(SDLKey_SDLK_ESCAPE as c_int)
}

#[cfg(feature = "gcw0")]
#[inline]
pub fn gcw0_any_button_pressed() -> bool {
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
pub fn gcw0_a_pressed_r() -> bool {
    KeyIsPressedR(SDLKey_SDLK_LCTRL as c_int)
}

#[cfg(feature = "gcw0")]
#[inline]
pub fn gcw0_b_pressed_r() -> bool {
    KeyIsPressedR(SDLKey_SDLK_LALT as c_int)
}

#[cfg(feature = "gcw0")]
#[inline]
pub fn gcw0_x_pressed_r() -> bool {
    KeyIsPressedR(SDLKey_SDLK_LSHIFT as c_int)
}

#[cfg(feature = "gcw0")]
#[inline]
pub fn gcw0_y_pressed_r() -> bool {
    KeyIsPressedR(SDLKey_SDLK_SPACE as c_int)
}

#[cfg(feature = "gcw0")]
#[inline]
pub fn gcw0_rs_pressed_r() -> bool {
    KeyIsPressed(SDLKey_SDLK_BACKSPACE as c_int)
}

#[cfg(feature = "gcw0")]
#[inline]
pub fn gcw0_ls_pressed_r() -> bool {
    KeyIsPressed(SDLKey_SDLK_TAB as c_int)
}

#[cfg(feature = "gcw0")]
#[inline]
pub fn gcw0_start_pressed_r() -> bool {
    KeyIsPressed(SDLKey_SDLK_RETURN as c_int)
}

#[cfg(feature = "gcw0")]
#[inline]
pub fn gcw0_select_pressed_r() -> bool {
    KeyIsPressed(SDLKey_SDLK_ESCAPE as c_int)
}

#[cfg(feature = "gcw0")]
#[inline]
pub fn gcw0_any_button_pressed_r() -> bool {
    gcw0_a_pressed_r()
        || gcw0_b_pressed_r()
        || gcw0_x_pressed_r()
        || gcw0_y_pressed_r()
        || gcw0_ls_pressed_r()
        || gcw0_rs_pressed_r()
        || gcw0_start_pressed_r()
        || gcw0_select_pressed_r()
}

impl Data<'_> {
    #[inline]
    pub fn return_pressed_r(&mut self) -> bool {
        self.key_is_pressed_r(SDLKey_SDLK_RETURN as i32)
    }

    #[inline]
    pub fn shift_pressed(&mut self) -> bool {
        self.mod_is_pressed(SDLMod_KMOD_LSHIFT as u32 | SDLMod_KMOD_RSHIFT as u32)
    }

    #[inline]
    pub fn alt_pressed(&mut self) -> bool {
        self.mod_is_pressed(SDLMod_KMOD_LALT as u32 | SDLMod_KMOD_RALT as u32)
    }

    #[inline]
    pub fn ctrl_pressed(&mut self) -> bool {
        self.mod_is_pressed(SDLMod_KMOD_LCTRL as u32 | SDLMod_KMOD_RCTRL as u32)
    }

    #[inline]
    pub fn mouse_left_pressed(&mut self) -> bool {
        self.key_is_pressed(PointerStates::MouseButton1 as c_int)
    }

    #[inline]
    pub fn mouse_left_pressed_r(&mut self) -> bool {
        self.key_is_pressed_r(PointerStates::MouseButton1 as c_int)
    }

    #[inline]
    pub fn space_pressed(&mut self) -> bool {
        self.key_is_pressed(SDLKey_SDLK_SPACE as c_int)
    }

    #[inline]
    pub fn escape_pressed_r(&mut self) -> bool {
        self.key_is_pressed_r(SDLKey_SDLK_ESCAPE as c_int)
    }

    #[inline]
    pub fn up_pressed(&mut self) -> bool {
        self.cmd_is_active(Cmds::Up)
    }

    #[inline]
    pub fn up_pressed_static(
        sdl: &Sdl,
        input: &mut Input,
        vars: &Vars,
        quit: &Cell<bool>,
    ) -> bool {
        Self::cmd_is_active_static(sdl, input, vars, quit, Cmds::Up)
    }

    #[inline]
    pub fn down_pressed(&mut self) -> bool {
        self.cmd_is_active(Cmds::Down)
    }

    #[inline]
    pub fn down_pressed_static(
        sdl: &Sdl,
        input: &mut Input,
        vars: &Vars,
        quit: &Cell<bool>,
    ) -> bool {
        Self::cmd_is_active_static(sdl, input, vars, quit, Cmds::Down)
    }

    #[inline]
    pub fn left_pressed(&mut self) -> bool {
        self.cmd_is_active(Cmds::Left)
    }

    #[inline]
    pub fn right_pressed(&mut self) -> bool {
        self.cmd_is_active(Cmds::Right)
    }

    #[inline]
    pub fn fire_pressed(&mut self) -> bool {
        self.cmd_is_active(Cmds::Fire)
    }

    #[inline]
    pub fn fire_pressed_r(&mut self) -> bool {
        self.cmd_is_active_r(Cmds::Fire)
    }

    #[inline]
    pub fn fire_pressed_r_static(
        sdl: &Sdl,
        input: &mut Input,
        vars: &Vars,
        quit: &Cell<bool>,
    ) -> bool {
        Self::cmd_is_active_r_static(sdl, input, vars, quit, Cmds::Fire)
    }

    #[inline]
    pub fn up_pressed_r(&mut self) -> bool {
        self.cmd_is_active_r(Cmds::Up)
    }

    #[inline]
    pub fn down_pressed_r(&mut self) -> bool {
        self.cmd_is_active_r(Cmds::Down)
    }

    #[inline]
    pub fn left_pressed_r(&mut self) -> bool {
        self.cmd_is_active_r(Cmds::Left)
    }

    #[inline]
    pub fn right_pressed_r(&mut self) -> bool {
        self.cmd_is_active_r(Cmds::Right)
    }

    #[inline]
    pub fn any_cmd_active(&mut self) -> bool {
        self.cmd_is_active(Cmds::Fire)
            || self.cmd_is_active(Cmds::Activate)
            || self.cmd_is_active(Cmds::Takeover)
    }
}

// ----------------------------------------

bitflags! {
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

pub const COLLISION_STEPSIZE: f32 = 0.1;

/* ************************************************************
 * Highscore related defines
 *************************************************************/
pub const HS_BACKGROUND_FILE: &[u8] = b"transfer.jpg";
pub const HS_EMPTY_ENTRY: &str = "--- empty ---";
pub const MAX_NAME_LEN: usize = 15; /* max len of highscore name entry */
pub const MAX_HIGHSCORES: usize = 10; /* only keep Top10 */
pub const DATE_LEN: usize = 10; /* reserved for the date-string */
//***************************************************************

// find_file(): use current-theme subdir in search or not
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Themed {
    NoTheme = 0,
    UseTheme,
}
// find_file(): how important is the file in question:
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
pub enum AlertNames {
    Green = 0,
    Yellow,
    Amber,
    Red,
    Last,
}

impl AlertNames {
    pub fn to_str(self) -> &'static str {
        self.into()
    }
}

impl From<AlertNames> for &'static str {
    fn from(alert_name: AlertNames) -> Self {
        use AlertNames::*;
        match alert_name {
            Green => "green",
            Yellow => "yellow",
            Amber => "amber",
            Red => "red",
            Last => panic!("invalid alert name"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct InvalidAlertName;

impl TryFrom<i32> for AlertNames {
    type Error = InvalidAlertName;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        use AlertNames::*;
        Ok(match value {
            0 => Green,
            1 => Yellow,
            2 => Amber,
            3 => Red,
            _ => return Err(InvalidAlertName),
        })
    }
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

pub const GRAPHICS_DIR_C: &CStr = cstr!("graphics/");
pub const SOUND_DIR_C: &CStr = cstr!("sound/");
pub const MAP_DIR_C: &CStr = cstr!("map/");

pub const MAP_BLOCK_FILE: &[u8] = b"map_blocks.png";
pub const DROID_BLOCK_FILE: &[u8] = b"droids.png";
pub const BULLET_BLOCK_FILE: &[u8] = b"bullet.png";
pub const BLAST_BLOCK_FILE: &[u8] = b"blast.png";
pub const DIGIT_BLOCK_FILE: &[u8] = b"digits.png";

pub const BANNER_BLOCK_FILE: &[u8] = b"banner.png";
pub const TITLE_PIC_FILE: &[u8] = b"title.jpg";
pub const CONSOLE_PIC_FILE: &[u8] = b"console_fg.png";
pub const CONSOLE_BG_PIC1_FILE: &[u8] = b"console_bg1.jpg";
pub const CONSOLE_BG_PIC2_FILE: &[u8] = b"console_bg2.jpg";
pub const TAKEOVER_BG_PIC_FILE: &[u8] = b"takeover_bg.jpg";
pub const CREDITS_PIC_FILE: &[u8] = b"credits.jpg";

pub const SHIP_ON_PIC_FILE: &[u8] = b"ship_on.png";
pub const SHIP_OFF_PIC_FILE: &[u8] = b"ship_off.png";

pub const PROGRESS_METER_FILE: &[u8] = b"progress_meter.png";
pub const PROGRESS_FILLER_FILE: &[u8] = b"progress_filler.png";

pub const STANDARD_MISSION: &str = "Paradroid.mission";

pub const PARA_FONT_FILE: &str = "parafont.png";
pub const FONT0_FILE: &str = "font05.png";
pub const FONT1_FILE: &str = "font05_green.png";
pub const FONT2_FILE: &str = "font05_red.png";
pub const ICON_FILE: &str = "paraicon_48x48.png";

// **********************************************************************

pub const DIGITNUMBER: usize = 10;

pub const TEXT_STRETCH: f64 = 1.2;
pub const LEFT_TEXT_LEN: usize = 10;
pub const RIGHT_TEXT_LEN: usize = 6;

pub const BULLET_COLL_DIST2: f32 = 0.024_414_063;
// **********************************************************************
//
//

// The following is the definition of the sound file names used in freedroid
// DO NOT EVER CHANGE THE ORDER OF APPEARENCE IN THIS LIST PLEASE!!!!!
// The order of appearance here should match the order of appearance
// in the SoundSampleFilenames definition located in sound.c!
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(dead_code)]
pub enum SoundType {
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
pub const BYCOLOR: &[u8] = b"BYCOLOR";

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

pub const ENEMYPHASES: u8 = 8;

pub const WAIT_LEVELEMPTY: f32 = 0.5; /* warte bevor Graufaerben (in seconds)*/
pub const SLOWMO_FACTOR: f32 = 0.33; // slow-motion effect on last blast when level is going empty
pub const WAIT_AFTER_KILLED: u32 = 2000; // time (in ms) to wait and still display pictures after the destruction of
pub const SHOW_WAIT: u32 = 3500; // std amount of time to show something
                                 // the players droid.  This is now measured in seconds and can be a float
pub const WAIT_TRANSFERMODE: f32 = 0.3; /* this is a "float" indicating the number of seconds the influence
                                        stand still with space pressed, before switching into transfermode
                                        This variable describes the amount in SECONDS */
pub const WAIT_COLLISION: c_int = 1; // after a little collision with influ, enemys hold position for a while
                                     // this variable describes the amount of time in SECONDS
pub const ENEMYMAXWAIT: c_int = 2; // after each robot has reached its current destination waypoint is waits a
                                   // while.  This variable describes the amount of time in SECONDS.  However,
                                   // the final wait time is a random number within [0,ENEMYMAXWAIT].
pub const FLASH_DURATION: f32 = 0.1; // in seconds

/* direction definitions (fireing bullets and testing blockedness of positions) */
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
pub const AGGRESSIONMAX: c_int = 100;
pub const ROBOT_MAX_WAIT_BETWEEN_SHOTS: f32 = 5.; // how long shoud each droid wait at most until
                                                  // is considers fireing again?

/* Map-related defines:
    WARNING leave them here, they are required in struct.h
*/
pub const MAX_WP_CONNECTIONS: usize = 12;
pub const MAX_MAP_ROWS: usize = 255;
pub const MAX_MAP_COLS: usize = 255;
pub const MAX_ENEMYS_ON_SHIP: usize = 300;
pub const MAX_INFLU_POSITION_HISTORY: usize = 100;

pub const MAX_LIFTS: usize = 50; /* actually the entries to the lifts */
pub const MAX_LEVELS: usize = 29; /* don't change this easily */
/* corresponds to a reserved palette range ! */
pub const MAX_LIFT_ROWS: usize = 15; /* the different lift "rows" */
/* don't change this easily */
/* corresponds to a reserved palette range !*/
pub const MAX_LEVEL_RECTS: usize = 20; // how many rects compose a level

pub const MAXWAYPOINTS: usize = 100;
pub const MAX_DOORS_ON_LEVEL: usize = 60;
pub const MAX_REFRESHES_ON_LEVEL: usize = 40;
pub const MAX_ALERTS_ON_LEVEL: usize = 40;

pub const MAX_PHASES_IN_A_BULLET: usize = 12;

pub const PUSHSPEED: f32 = 2.;

/* Schusstypen */
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BulletKind {
    Pulse = 0,
    SinglePulse,
    Military,
    Flash,
    Exterminator,
    LaserRifle,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InvalidBulletKind<T>(T);

impl<T> fmt::Display for InvalidBulletKind<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid raw bullet kind {}", self.0)
    }
}

impl TryFrom<u8> for BulletKind {
    type Error = InvalidBulletKind<u8>;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use BulletKind::*;
        Ok(match value {
            0 => Pulse,
            1 => SinglePulse,
            2 => Military,
            3 => Flash,
            4 => Exterminator,
            5 => LaserRifle,
            _ => return Err(InvalidBulletKind(value)),
        })
    }
}

impl TryFrom<c_int> for BulletKind {
    type Error = InvalidBulletKind<c_int>;

    fn try_from(value: c_int) -> Result<Self, Self::Error> {
        u8::try_from(value)
            .map_err(|_| InvalidBulletKind(value))
            .and_then(|value| {
                BulletKind::try_from(value).map_err(|err| InvalidBulletKind(err.0.into()))
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Explosion {
    Bulletblast = 0,
    Druidblast,
    Rejectblast,
}

pub const BLINKENERGY: f32 = 25.;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(dead_code)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(dead_code)]
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

pub const DECKCOMPLETEBONUS: f32 = 500.;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MapTile {
    Floor,
    EckLu,
    Tu,
    EckRu,
    Tl,
    Kreuz,
    Tr,
    EckLo,
    To,
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

impl MapTile {
    pub fn refresh(offset: u8) -> Option<Self> {
        Some(match offset {
            0 => Self::Refresh1,
            1 => Self::Refresh2,
            2 => Self::Refresh3,
            3 => Self::Refresh4,
            _ => return None,
        })
    }

    pub fn next(self) -> Option<Self> {
        use MapTile::*;

        Some(match self {
            Floor => EckLu,
            EckLu => Tu,
            Tu => EckRu,
            EckRu => Tl,
            Tl => Kreuz,
            Kreuz => Tr,
            Tr => EckLo,
            EckLo => To,
            To => EckRo,
            EckRo => HWall,
            HWall => VWall,
            VWall => Invisible,
            Invisible => Block1,
            Block1 => Block2,
            Block2 => Block3,
            Block3 => Block4,
            Block4 => Block5,
            Block5 => HZutuere,
            HZutuere => HHalbtuere1,
            HHalbtuere1 => HHalbtuere2,
            HHalbtuere2 => HHalbtuere3,
            HHalbtuere3 => HGanztuere,
            HGanztuere => KonsoleL,
            KonsoleL => KonsoleR,
            KonsoleR => KonsoleO,
            KonsoleO => KonsoleU,
            KonsoleU => VZutuere,
            VZutuere => VHalbtuere1,
            VHalbtuere1 => VHalbtuere2,
            VHalbtuere2 => VHalbtuere3,
            VHalbtuere3 => VGanztuere,
            VGanztuere => Lift,
            Lift => Void,
            Void => Refresh1,
            Refresh1 => Refresh2,
            Refresh2 => Refresh3,
            Refresh3 => Refresh4,
            Refresh4 => AlertGreen,
            AlertGreen => AlertYellow,
            AlertYellow => AlertAmber,
            AlertAmber => AlertRed,
            AlertRed => Unused2,
            Unused2 => FineGrid,
            FineGrid | NumMapTiles => return None,
        })
    }

    pub fn prev(self) -> Option<Self> {
        use MapTile::*;

        Some(match self {
            Floor | NumMapTiles => return None,
            EckLu => Floor,
            Tu => EckLu,
            EckRu => Tu,
            Tl => EckRu,
            Kreuz => Tl,
            Tr => Kreuz,
            EckLo => Tr,
            To => EckLo,
            EckRo => To,
            HWall => EckRo,
            VWall => HWall,
            Invisible => VWall,
            Block1 => Invisible,
            Block2 => Block1,
            Block3 => Block2,
            Block4 => Block3,
            Block5 => Block4,
            HZutuere => Block5,
            HHalbtuere1 => HZutuere,
            HHalbtuere2 => HHalbtuere1,
            HHalbtuere3 => HHalbtuere2,
            HGanztuere => HHalbtuere3,
            KonsoleL => HGanztuere,
            KonsoleR => KonsoleL,
            KonsoleO => KonsoleR,
            KonsoleU => KonsoleO,
            VZutuere => KonsoleU,
            VHalbtuere1 => VZutuere,
            VHalbtuere2 => VHalbtuere1,
            VHalbtuere3 => VHalbtuere2,
            VGanztuere => VHalbtuere3,
            Lift => VGanztuere,
            Void => Lift,
            Refresh1 => Void,
            Refresh2 => Refresh1,
            Refresh3 => Refresh2,
            Refresh4 => Refresh3,
            AlertGreen => Refresh4,
            AlertYellow => AlertGreen,
            AlertAmber => AlertYellow,
            AlertRed => AlertAmber,
            Unused2 => AlertRed,
            FineGrid => Unused2,
        })
    }
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
                    2 => Tu,
                    3 => EckRu,
                    4 => Tl,
                    5 => Kreuz,
                    6 => Tr,
                    7 => EckLo,
                    8 => To,
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
