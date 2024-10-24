use crate::{input::Input, structs::Point, vars::Vars, Sdl};

use bitflags::bitflags;
use sdl::convert::{u32_to_isize, u32_to_u16};
#[cfg(feature = "gcw0")]
use sdl_sys::{
    SDLKey_SDLK_BACKSPACE, SDLKey_SDLK_LALT, SDLKey_SDLK_LCTRL, SDLKey_SDLK_LSHIFT, SDLKey_SDLK_TAB,
};
#[cfg(not(target_os = "android"))]
use sdl_sys::{SDLKey_SDLK_ESCAPE, SDLKey_SDLK_SPACE};
use sdl_sys::{
    SDLKey_SDLK_LAST, SDLKey_SDLK_RETURN, SDLMod_KMOD_LALT, SDLMod_KMOD_LCTRL, SDLMod_KMOD_LSHIFT,
    SDLMod_KMOD_RALT, SDLMod_KMOD_RCTRL, SDLMod_KMOD_RSHIFT,
};
use std::{
    cell::Cell,
    ffi::CStr,
    fmt::{self, Display},
};

pub const MAX_THEMES: usize = 100;

pub const DROID_ROTATION_TIME: f32 = 3.0;
pub const NUM_DECAL_PICS: usize = 2;

#[inline]
#[allow(clippy::cast_possible_truncation)]
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
    MouseUp = u32_to_isize(SDLKey_SDLK_LAST) + 1,
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

impl PointerStates {
    #[inline]
    #[must_use]
    pub const fn to_u16(self) -> u16 {
        self as u16
    }

    #[inline]
    #[must_use]
    pub const fn to_usize(self) -> usize {
        self as usize
    }
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

impl crate::Data<'_> {
    #[inline]
    pub fn return_pressed_r(&mut self) -> bool {
        self.key_is_pressed_r(u32_to_u16(SDLKey_SDLK_RETURN))
    }

    #[inline]
    pub fn shift_pressed(&mut self) -> bool {
        self.mod_is_pressed(SDLMod_KMOD_LSHIFT | SDLMod_KMOD_RSHIFT)
    }

    #[inline]
    pub fn alt_pressed(&mut self) -> bool {
        self.mod_is_pressed(SDLMod_KMOD_LALT | SDLMod_KMOD_RALT)
    }

    #[inline]
    pub fn ctrl_pressed(&mut self) -> bool {
        self.mod_is_pressed(SDLMod_KMOD_LCTRL | SDLMod_KMOD_RCTRL)
    }

    #[cfg(not(target_os = "android"))]
    #[inline]
    pub fn mouse_left_pressed(&mut self) -> bool {
        self.key_is_pressed(PointerStates::MouseButton1.to_u16())
    }

    #[inline]
    pub fn mouse_left_pressed_r(&mut self) -> bool {
        self.key_is_pressed_r(PointerStates::MouseButton1.to_u16())
    }

    #[cfg(not(target_os = "android"))]
    #[inline]
    pub fn space_pressed(&mut self) -> bool {
        self.key_is_pressed(u32_to_u16(SDLKey_SDLK_SPACE))
    }

    #[cfg(not(target_os = "android"))]
    #[inline]
    pub fn escape_pressed_r(&mut self) -> bool {
        self.key_is_pressed_r(u32_to_u16(SDLKey_SDLK_ESCAPE))
    }

    #[inline]
    pub fn up_pressed(&mut self) -> bool {
        self.cmd_is_active(Cmds::Up)
    }

    #[inline]
    pub fn up_pressed_static(sdl: &Sdl, input: &mut Input, vars: &Vars, quit: &Cell<bool>) -> bool {
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

    #[cfg(not(target_os = "android"))]
    #[inline]
    pub fn up_pressed_r(&mut self) -> bool {
        self.cmd_is_active_r(Cmds::Up)
    }

    #[cfg(not(target_os = "android"))]
    #[inline]
    pub fn down_pressed_r(&mut self) -> bool {
        self.cmd_is_active_r(Cmds::Down)
    }

    #[cfg(not(target_os = "android"))]
    #[inline]
    pub fn left_pressed_r(&mut self) -> bool {
        self.cmd_is_active_r(Cmds::Left)
    }

    #[cfg(not(target_os = "android"))]
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

    #[cfg(feature = "gcw0")]
    #[inline]
    pub fn gcw0_a_pressed(&mut self) -> bool {
        self.key_is_pressed(u32_to_u16(SDLKey_SDLK_LCTRL))
    }

    #[cfg(feature = "gcw0")]
    #[inline]
    pub fn gcw0_b_pressed(&mut self) -> bool {
        self.key_is_pressed(u32_to_u16(SDLKey_SDLK_LALT))
    }

    #[cfg(feature = "gcw0")]
    #[inline]
    pub fn gcw0_x_pressed(&mut self) -> bool {
        self.key_is_pressed(u32_to_u16(SDLKey_SDLK_LSHIFT))
    }

    #[cfg(feature = "gcw0")]
    #[inline]
    pub fn gcw0_y_pressed(&mut self) -> bool {
        self.key_is_pressed(u32_to_u16(SDLKey_SDLK_SPACE))
    }

    #[cfg(feature = "gcw0")]
    #[inline]
    pub fn gcw0_rs_pressed(&mut self) -> bool {
        self.key_is_pressed(u32_to_u16(SDLKey_SDLK_BACKSPACE))
    }

    #[cfg(feature = "gcw0")]
    #[inline]
    pub fn gcw0_ls_pressed(&mut self) -> bool {
        self.key_is_pressed(u32_to_u16(SDLKey_SDLK_TAB))
    }

    #[cfg(feature = "gcw0")]
    #[inline]
    pub fn gcw0_start_pressed(&mut self) -> bool {
        self.key_is_pressed(u32_to_u16(SDLKey_SDLK_RETURN))
    }

    #[cfg(feature = "gcw0")]
    #[inline]
    pub fn gcw0_select_pressed(&mut self) -> bool {
        self.key_is_pressed(u32_to_u16(SDLKey_SDLK_ESCAPE))
    }

    #[cfg(feature = "gcw0")]
    #[inline]
    pub fn gcw0_any_button_pressed(&mut self) -> bool {
        self.gcw0_a_pressed()
            || self.gcw0_b_pressed()
            || self.gcw0_x_pressed()
            || self.gcw0_y_pressed()
            || self.gcw0_ls_pressed()
            || self.gcw0_rs_pressed()
            || self.gcw0_start_pressed()
            || self.gcw0_select_pressed()
    }

    #[cfg(feature = "gcw0")]
    #[inline]
    pub fn gcw0_a_pressed_r(&mut self) -> bool {
        self.key_is_pressed_r(u32_to_u16(SDLKey_SDLK_LCTRL))
    }

    #[cfg(feature = "gcw0")]
    #[inline]
    pub fn gcw0_b_pressed_r(&mut self) -> bool {
        self.key_is_pressed_r(u32_to_u16(SDLKey_SDLK_LALT))
    }

    #[cfg(feature = "gcw0")]
    #[inline]
    pub fn gcw0_x_pressed_r(&mut self) -> bool {
        self.key_is_pressed_r(u32_to_u16(SDLKey_SDLK_LSHIFT))
    }

    #[cfg(feature = "gcw0")]
    #[inline]
    pub fn gcw0_y_pressed_r(&mut self) -> bool {
        self.key_is_pressed_r(u32_to_u16(SDLKey_SDLK_SPACE))
    }

    #[cfg(feature = "gcw0")]
    #[inline]
    pub fn gcw0_rs_pressed_r(&mut self) -> bool {
        self.key_is_pressed(u32_to_u16(SDLKey_SDLK_BACKSPACE))
    }

    #[cfg(feature = "gcw0")]
    #[inline]
    pub fn gcw0_ls_pressed_r(&mut self) -> bool {
        self.key_is_pressed(u32_to_u16(SDLKey_SDLK_TAB))
    }

    #[cfg(feature = "gcw0")]
    #[inline]
    pub fn gcw0_start_pressed_r(&mut self) -> bool {
        self.key_is_pressed(u32_to_u16(SDLKey_SDLK_RETURN))
    }

    #[cfg(feature = "gcw0")]
    #[inline]
    pub fn gcw0_select_pressed_r(&mut self) -> bool {
        self.key_is_pressed(u32_to_u16(SDLKey_SDLK_ESCAPE))
    }

    #[cfg(feature = "gcw0")]
    #[inline]
    pub fn gcw0_any_button_pressed_r(&mut self) -> bool {
        self.gcw0_a_pressed_r()
            || self.gcw0_b_pressed_r()
            || self.gcw0_x_pressed_r()
            || self.gcw0_y_pressed_r()
            || self.gcw0_ls_pressed_r()
            || self.gcw0_rs_pressed_r()
            || self.gcw0_start_pressed_r()
            || self.gcw0_select_pressed_r()
    }
}

// ----------------------------------------

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
pub const MAX_NAME_LEN: u8 = 15; /* max len of highscore name entry */
pub const MAX_HIGHSCORES: u8 = 10; /* only keep Top10 */
pub const DATE_LEN: u8 = 10; /* reserved for the date-string */
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
        match value {
            0 => Ok(Criticality::Ignore),
            1 => Ok(Criticality::WarnOnly),
            2 => Ok(Criticality::Critical),
            _ => Err(InvalidCriticality),
        }
    }
}

// The flags for DisplayBanner are:
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct DisplayBannerFlags: u8 {
        const FORCE_UPDATE=1;
        const DONT_TOUCH_TEXT=2;
        const NO_SDL_UPDATE=4;
    }
}

// The flags for AssembleCombatWindow are:
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct AssembleCombatWindowFlags: u8 {
        const ONLY_SHOW_MAP = 0x01;
        const DO_SCREEN_UPDATE = 0x02;
        const SHOW_FULL_MAP = 0x04;
    }
}

// symbolic Alert-names
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertLevel {
    #[default]
    Green = 0,
    Yellow,
    Amber,
    Red,
}

impl AlertLevel {
    pub fn to_str(self) -> &'static str {
        self.into()
    }

    pub const fn to_tile(self) -> MapTile {
        match self {
            AlertLevel::Green => MapTile::AlertGreen,
            AlertLevel::Yellow => MapTile::AlertYellow,
            AlertLevel::Amber => MapTile::AlertAmber,
            AlertLevel::Red => MapTile::AlertRed,
        }
    }

    pub fn from_death_count(death_count: f32, alert_threshold: u16) -> Self {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let ratio = (death_count / f32::from(alert_threshold)) as u8;
        match ratio {
            0 => Self::Green,
            1 => Self::Yellow,
            2 => Self::Amber,
            _ => Self::Red,
        }
    }
}

impl From<AlertLevel> for &'static str {
    fn from(alert_name: AlertLevel) -> Self {
        match alert_name {
            AlertLevel::Green => "green",
            AlertLevel::Yellow => "yellow",
            AlertLevel::Amber => "amber",
            AlertLevel::Red => "red",
        }
    }
}

impl From<AlertLevel> for f32 {
    #[inline]
    fn from(value: AlertLevel) -> Self {
        f32::from(value as u8)
    }
}

impl Display for AlertLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.to_str())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct InvalidAlertName;

impl TryFrom<i32> for AlertLevel {
    type Error = InvalidAlertName;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => AlertLevel::Green,
            1 => AlertLevel::Yellow,
            2 => AlertLevel::Amber,
            3 => AlertLevel::Red,
            _ => return Err(InvalidAlertName),
        })
    }
}

// **********************************************************************
// Constants for Paths and names of Data-files
// the root "FD_DATADIR" should be defined in the Makefile as $(pkgdatadir)
// if not, we set it here:
// #ifndef FD_DATADIR

#[cfg(target_os = "macos")]
pub const FD_DATADIR: &str = "FreeDroid.app/Contents/Resources"; // our local fallback

#[cfg(not(target_os = "macos"))]
pub const FD_DATADIR: &str = "."; // our local fallback

// #endif // !FD_DATADIR

// #ifndef LOCAL_DATADIR
pub const LOCAL_DATADIR: &str = ".."; // local fallback
                                      // #endif

pub const GRAPHICS_DIR_C: &CStr = c"graphics/";
pub const SOUND_DIR_C: &CStr = c"sound/";
pub const MAP_DIR_C: &CStr = c"map/";

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

impl SoundType {
    #[inline]
    #[must_use]
    pub const fn to_usize(self) -> usize {
        self as usize
    }
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
pub const WAIT_COLLISION: u8 = 1; // after a little collision with influ, enemys hold position for a while
                                  // this variable describes the amount of time in SECONDS
pub const ENEMYMAXWAIT: u8 = 2; // after each robot has reached its current destination waypoint is waits a
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

pub const MAXBULLETS: u8 = 100; /* maximum possible Bullets in the air */
pub const MAXBLASTS: u8 = 100; /* max. possible Blasts visible */
pub const AGGRESSIONMAX: u8 = 100;
pub const ROBOT_MAX_WAIT_BETWEEN_SHOTS: f32 = 5.; // how long shoud each droid wait at most until
                                                  // is considers fireing again?

/* Map-related defines:
    WARNING leave them here, they are required in struct.h
*/
pub const MAX_WP_CONNECTIONS: u8 = 12;
pub const MAX_MAP_ROWS: u8 = 255;
#[cfg(not(target_os = "android"))]
pub const MAX_MAP_COLS: u8 = 255;
pub const MAX_ENEMYS_ON_SHIP: usize = 300;
pub const MAX_INFLU_POSITION_HISTORY: usize = 100;

pub const MAX_LIFTS: usize = 50; /* actually the entries to the lifts */
pub const MAX_LEVELS: usize = 29; /* don't change this easily */
/* corresponds to a reserved palette range ! */
pub const MAX_LIFT_ROWS: usize = 15; /* the different lift "rows" */
/* don't change this easily */
/* corresponds to a reserved palette range !*/
pub const MAX_LEVEL_RECTS: usize = 20; // how many rects compose a level

pub const MAXWAYPOINTS: u8 = 100;
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

impl BulletKind {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Pulse => "none",
            Self::SinglePulse | Self::Military => "lasers",
            Self::Flash => "disruptor",
            Self::Exterminator => "exterminator",
            Self::LaserRifle => "laser rifle",
        }
    }

    #[inline]
    #[must_use]
    pub const fn to_usize(self) -> usize {
        self as usize
    }
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
        Ok(match value {
            0 => BulletKind::Pulse,
            1 => BulletKind::SinglePulse,
            2 => BulletKind::Military,
            3 => BulletKind::Flash,
            4 => BulletKind::Exterminator,
            5 => BulletKind::LaserRifle,
            _ => return Err(InvalidBulletKind(value)),
        })
    }
}

impl TryFrom<i32> for BulletKind {
    type Error = InvalidBulletKind<i32>;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        u8::try_from(value)
            .map_err(|_| InvalidBulletKind(value))
            .and_then(|value| {
                BulletKind::try_from(value).map_err(|err| InvalidBulletKind(err.0.into()))
            })
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Explosion {
    #[default]
    Bulletblast = 0,
    Druidblast {
        from_influencer: bool,
    },
}

impl Explosion {
    pub const fn to_u8(self) -> u8 {
        match self {
            Explosion::Bulletblast => 0,
            Explosion::Druidblast { .. } => 1,
        }
    }

    pub const fn is_from_influencer(self) -> bool {
        match self {
            Explosion::Bulletblast => false,
            Explosion::Druidblast { from_influencer } => from_influencer,
        }
    }
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

impl Droid {
    #[inline]
    #[must_use]
    pub const fn to_usize(self) -> usize {
        self as usize
    }

    #[inline]
    #[must_use]
    pub const fn to_u16(self) -> u16 {
        self as u16
    }

    #[must_use]
    pub const fn next(self) -> Option<Self> {
        match self {
            Droid::Droid001 => Some(Droid::Droid123),
            Droid::Droid123 => Some(Droid::Droid139),
            Droid::Droid139 => Some(Droid::Droid247),
            Droid::Droid247 => Some(Droid::Droid249),
            Droid::Droid249 => Some(Droid::Droid296),
            Droid::Droid296 => Some(Droid::Droid302),
            Droid::Droid302 => Some(Droid::Droid329),
            Droid::Droid329 => Some(Droid::Droid420),
            Droid::Droid420 => Some(Droid::Droid476),
            Droid::Droid476 => Some(Droid::Droid493),
            Droid::Droid493 => Some(Droid::Droid516),
            Droid::Droid516 => Some(Droid::Droid571),
            Droid::Droid571 => Some(Droid::Droid598),
            Droid::Droid598 => Some(Droid::Droid614),
            Droid::Droid614 => Some(Droid::Droid615),
            Droid::Droid615 => Some(Droid::Droid629),
            Droid::Droid629 => Some(Droid::Droid711),
            Droid::Droid711 => Some(Droid::Droid742),
            Droid::Droid742 => Some(Droid::Droid751),
            Droid::Droid751 => Some(Droid::Droid821),
            Droid::Droid821 => Some(Droid::Droid834),
            Droid::Droid834 => Some(Droid::Droid883),
            Droid::Droid883 => Some(Droid::Droid999),
            Droid::Droid999 => None,
            Droid::NumDroids => panic!("invalid droid"),
        }
    }

    pub const fn previous(self) -> Option<Self> {
        match self {
            Droid::Droid001 => None,
            Droid::Droid123 => Some(Droid::Droid001),
            Droid::Droid139 => Some(Droid::Droid123),
            Droid::Droid247 => Some(Droid::Droid139),
            Droid::Droid249 => Some(Droid::Droid247),
            Droid::Droid296 => Some(Droid::Droid249),
            Droid::Droid302 => Some(Droid::Droid296),
            Droid::Droid329 => Some(Droid::Droid302),
            Droid::Droid420 => Some(Droid::Droid329),
            Droid::Droid476 => Some(Droid::Droid420),
            Droid::Droid493 => Some(Droid::Droid476),
            Droid::Droid516 => Some(Droid::Droid493),
            Droid::Droid571 => Some(Droid::Droid516),
            Droid::Droid598 => Some(Droid::Droid571),
            Droid::Droid614 => Some(Droid::Droid598),
            Droid::Droid615 => Some(Droid::Droid614),
            Droid::Droid629 => Some(Droid::Droid615),
            Droid::Droid711 => Some(Droid::Droid629),
            Droid::Droid742 => Some(Droid::Droid711),
            Droid::Droid751 => Some(Droid::Droid742),
            Droid::Droid821 => Some(Droid::Droid751),
            Droid::Droid834 => Some(Droid::Droid821),
            Droid::Droid883 => Some(Droid::Droid834),
            Droid::Droid999 => Some(Droid::Droid883),
            Droid::NumDroids => panic!("invalid droid"),
        }
    }
}

impl TryFrom<u8> for Droid {
    type Error = InvalidDroid;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Droid001),
            1 => Ok(Self::Droid123),
            2 => Ok(Self::Droid139),
            3 => Ok(Self::Droid247),
            4 => Ok(Self::Droid249),
            5 => Ok(Self::Droid296),
            6 => Ok(Self::Droid302),
            7 => Ok(Self::Droid329),
            8 => Ok(Self::Droid420),
            9 => Ok(Self::Droid476),
            10 => Ok(Self::Droid493),
            11 => Ok(Self::Droid516),
            12 => Ok(Self::Droid571),
            13 => Ok(Self::Droid598),
            14 => Ok(Self::Droid614),
            15 => Ok(Self::Droid615),
            16 => Ok(Self::Droid629),
            17 => Ok(Self::Droid711),
            18 => Ok(Self::Droid742),
            19 => Ok(Self::Droid751),
            20 => Ok(Self::Droid821),
            21 => Ok(Self::Droid834),
            22 => Ok(Self::Droid883),
            23 => Ok(Self::Droid999),
            _ => Err(InvalidDroid),
        }
    }
}

#[derive(Debug)]
pub struct InvalidDroid;

impl Display for InvalidDroid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("raw droid index is invalid")
    }
}

impl std::error::Error for InvalidDroid {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Status {
    Mobile,
    Transfermode,
    Weapon,
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

impl Status {
    pub const fn c_name(self) -> &'static CStr {
        match self {
            Self::Mobile => c"Mobile",
            Self::Transfermode => c"Transfer",
            Self::Weapon => c"Weapon",
            Self::Console => c"Logged In",
            Self::Debriefing => c"Debriefing",
            Self::Terminated => c"Terminated",
            Self::Pause => c"Pause",
            Self::Cheese => c"Cheese",
            Self::Elevator => c"Elevator",
            Self::Briefing => c"Briefing",
            Self::Menu => c"Menu",
            Self::Victory => c"Victory",
            Self::Activate => c"Activate",
            Self::Out => c"-- OUT --",
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::Mobile => "Mobile",
            Self::Transfermode => "Transfer",
            Self::Weapon => "Weapon",
            Self::Console => "Logged In",
            Self::Debriefing => "Debriefing",
            Self::Terminated => "Terminated",
            Self::Pause => "Pause",
            Self::Cheese => "Cheese",
            Self::Elevator => "Elevator",
            Self::Briefing => "Briefing",
            Self::Menu => "Menu",
            Self::Victory => "Victory",
            Self::Activate => "Activate",
            Self::Out => "-- OUT --",
        }
    }
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
        use MapTile as M;

        Some(match self {
            M::Floor => M::EckLu,
            M::EckLu => M::Tu,
            M::Tu => M::EckRu,
            M::EckRu => M::Tl,
            M::Tl => M::Kreuz,
            M::Kreuz => M::Tr,
            M::Tr => M::EckLo,
            M::EckLo => M::To,
            M::To => M::EckRo,
            M::EckRo => M::HWall,
            M::HWall => M::VWall,
            M::VWall => M::Invisible,
            M::Invisible => M::Block1,
            M::Block1 => M::Block2,
            M::Block2 => M::Block3,
            M::Block3 => M::Block4,
            M::Block4 => M::Block5,
            M::Block5 => M::HZutuere,
            M::HZutuere => M::HHalbtuere1,
            M::HHalbtuere1 => M::HHalbtuere2,
            M::HHalbtuere2 => M::HHalbtuere3,
            M::HHalbtuere3 => M::HGanztuere,
            M::HGanztuere => M::KonsoleL,
            M::KonsoleL => M::KonsoleR,
            M::KonsoleR => M::KonsoleO,
            M::KonsoleO => M::KonsoleU,
            M::KonsoleU => M::VZutuere,
            M::VZutuere => M::VHalbtuere1,
            M::VHalbtuere1 => M::VHalbtuere2,
            M::VHalbtuere2 => M::VHalbtuere3,
            M::VHalbtuere3 => M::VGanztuere,
            M::VGanztuere => M::Lift,
            M::Lift => M::Void,
            M::Void => M::Refresh1,
            M::Refresh1 => M::Refresh2,
            M::Refresh2 => M::Refresh3,
            M::Refresh3 => M::Refresh4,
            M::Refresh4 => M::AlertGreen,
            M::AlertGreen => M::AlertYellow,
            M::AlertYellow => M::AlertAmber,
            M::AlertAmber => M::AlertRed,
            M::AlertRed => M::Unused2,
            M::Unused2 => M::FineGrid,
            M::FineGrid | M::NumMapTiles => return None,
        })
    }

    pub fn prev(self) -> Option<Self> {
        use MapTile as M;

        Some(match self {
            M::Floor | M::NumMapTiles => return None,
            M::EckLu => M::Floor,
            M::Tu => M::EckLu,
            M::EckRu => M::Tu,
            M::Tl => M::EckRu,
            M::Kreuz => M::Tl,
            M::Tr => M::Kreuz,
            M::EckLo => M::Tr,
            M::To => M::EckLo,
            M::EckRo => M::To,
            M::HWall => M::EckRo,
            M::VWall => M::HWall,
            M::Invisible => M::VWall,
            M::Block1 => M::Invisible,
            M::Block2 => M::Block1,
            M::Block3 => M::Block2,
            M::Block4 => M::Block3,
            M::Block5 => M::Block4,
            M::HZutuere => M::Block5,
            M::HHalbtuere1 => M::HZutuere,
            M::HHalbtuere2 => M::HHalbtuere1,
            M::HHalbtuere3 => M::HHalbtuere2,
            M::HGanztuere => M::HHalbtuere3,
            M::KonsoleL => M::HGanztuere,
            M::KonsoleR => M::KonsoleL,
            M::KonsoleO => M::KonsoleR,
            M::KonsoleU => M::KonsoleO,
            M::VZutuere => M::KonsoleU,
            M::VHalbtuere1 => M::VZutuere,
            M::VHalbtuere2 => M::VHalbtuere1,
            M::VHalbtuere3 => M::VHalbtuere2,
            M::VGanztuere => M::VHalbtuere3,
            M::Lift => M::VGanztuere,
            M::Void => M::Lift,
            M::Refresh1 => M::Void,
            M::Refresh2 => M::Refresh1,
            M::Refresh3 => M::Refresh2,
            M::Refresh4 => M::Refresh3,
            M::AlertGreen => M::Refresh4,
            M::AlertYellow => M::AlertGreen,
            M::AlertAmber => M::AlertYellow,
            M::AlertRed => M::AlertAmber,
            M::Unused2 => M::AlertRed,
            M::FineGrid => M::Unused2,
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
                use MapTile as M;
                Ok(match value {
                    0 => M::Floor,
                    1 => M::EckLu,
                    2 => M::Tu,
                    3 => M::EckRu,
                    4 => M::Tl,
                    5 => M::Kreuz,
                    6 => M::Tr,
                    7 => M::EckLo,
                    8 => M::To,
                    9 => M::EckRo,
                    10 => M::HWall,
                    11 => M::VWall,
                    12 => M::Invisible,
                    13 => M::Block1,
                    14 => M::Block2,
                    15 => M::Block3,
                    16 => M::Block4,
                    17 => M::Block5,
                    18 => M::HZutuere,
                    19 => M::HHalbtuere1,
                    20 => M::HHalbtuere2,
                    21 => M::HHalbtuere3,
                    22 => M::HGanztuere,
                    23 => M::KonsoleL,
                    24 => M::KonsoleR,
                    25 => M::KonsoleO,
                    26 => M::KonsoleU,
                    27 => M::VZutuere,
                    28 => M::VHalbtuere1,
                    29 => M::VHalbtuere2,
                    30 => M::VHalbtuere3,
                    31 => M::VGanztuere,
                    32 => M::Lift,
                    33 => M::Void,
                    34 => M::Refresh1,
                    35 => M::Refresh2,
                    36 => M::Refresh3,
                    37 => M::Refresh4,
                    38 => M::AlertGreen,
                    39 => M::AlertYellow,
                    40 => M::AlertAmber,
                    41 => M::AlertRed,
                    42 => M::Unused2,
                    43 => M::FineGrid,
                    44 => M::NumMapTiles,
                    _ => return Err(InvalidMapTile),
                })
            }
        }

        $(
            impl_try_from_map_tile!($rest);
        )*
    };
}

impl_try_from_map_tile!(i8, u8, i32);
