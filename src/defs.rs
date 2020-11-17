#[cfg(feature = "gcw0")]
use crate::input::{KeyIsPressed, KeyIsPressedR};
use crate::{
    global::User_Rect,
    input::{cmd_is_active, cmd_is_activeR, ModIsPressed},
};

use bitflags::bitflags;
#[cfg(feature = "gcw0")]
use sdl::keysym::{SDLK_BACKSPACE, SDLK_TAB};
use sdl::{event::Mod, sdl::Rect};
use static_assertions::const_assert;
#[cfg(feature = "gcw0")]
use std::os::raw::c_int;
use std::{convert::TryFrom, mem};

pub const MAX_THEMES: usize = 100;

pub const JOY_MAX_VAL: usize = 32767; // maximal amplitude of joystick axis values

pub const RESET: usize = 0x01;
pub const UPDATE: usize = 0x02;
pub const INIT_ONLY: usize = 0x04;
pub const FREE_ONLY: usize = 0x08;

pub const DROID_ROTATION_TIME: f64 = 3.0;
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

// #define ScaleRect(rect,scale) do {\
// (rect).x *= scale; (rect).y *= scale; (rect).w *= scale; (rect).h *= scale; } while(0)

// #define ScalePoint(point,scale) do {\
// (point).x *= scale; (point).y *= scale; } while(0)

// #define Set_Rect(rect, xx, yy, ww, hh) do {\
// (rect).x = (xx); (rect).y = (yy); (rect).w = (ww); (rect).h = (hh); } while(0)

// #define Copy_Rect(src, dst) do {\
// (dst).x = (src).x; (dst).y = (src).y; (dst).w = (src).w; (dst).h = (src).h; } while(0)

// #define FreeIfUsed(pt) do { if ((pt)) SDL_FreeSurface((pt)); } while(0)

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

// #define ReturnPressed() (KeyIsPressed(SDLK_RETURN))
// #define ReturnPressedR() (KeyIsPressedR(SDLK_RETURN))

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

// #ifdef GCW0 // GCW0 keys are currently mapped to SDL key by the firmware...
// #define Gcw0APressed() (KeyIsPressed(SDLK_LCTRL))
// #define Gcw0BPressed() (KeyIsPressed(SDLK_LALT))
// #define Gcw0XPressed() (KeyIsPressed(SDLK_LSHIFT))
// #define Gcw0YPressed() (KeyIsPressed(SDLK_SPACE))

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

// #define Gcw0StartPressed() (KeyIsPressed(SDLK_RETURN))
// #define Gcw0SelectPressed() (KeyIsPressed(SDLK_ESCAPE))

// #define Gcw0AnyButtonPressed() (Gcw0APressed() || Gcw0BPressed()\
//         || Gcw0XPressed() || Gcw0YPressed() || Gcw0LSPressed() || Gcw0RSPressed()\
// 	|| Gcw0StartPressed() || Gcw0SelectPressed())

// #define Gcw0APressedR() (KeyIsPressedR(SDLK_LCTRL))
// #define Gcw0BPressedR() (KeyIsPressedR(SDLK_LALT))
// #define Gcw0XPressedR() (KeyIsPressedR(SDLK_LSHIFT))
// #define Gcw0YPressedR() (KeyIsPressedR(SDLK_SPACE))

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

// #define Gcw0StartPressedR() (KeyIsPressedR(SDLK_RETURN))
// #define Gcw0SelectPressedR() (KeyIsPressedR(SDLK_ESCAPE))

// #define Gcw0AnyButtonPressedR() (Gcw0APressedR() || Gcw0BPressedR()\
//         || Gcw0XPressedR() || Gcw0YPressedR() || Gcw0LSPressedR()\
// 	|| Gcw0RSPressedR() || Gcw0StartPressedR() || Gcw0SelectPressedR())
// #endif // GCW0 keys

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum MenuAction {
    None = 0b0000_0000_0000,
    Info = 0b0000_0000_0001,
    Back = 0b0000_0000_0010,
    Click = 0b0000_0000_0100,
    Left = 0b0000_0000_1000,
    Right = 0b0000_0001_0000,
    Up = 0b0000_0010_0000,
    Down = 0b0000_0100_0000,
    Delete = 0b0000_1000_0000,
    UpWheel = 0b0001_0000_0000,
    DownWheel = 0b0010_0000_0000,
    Last = 0b0100_0000_0000,
}

pub const COLLISION_STEPSIZE: f64 = 0.1;

/* ************************************************************
 * Highscore related defines
 *************************************************************/
pub const HS_BACKGROUND_FILE: &str = "transfer.jpg";
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
pub const SOUND_DIR: &str = "sound/";
pub const MAP_DIR: &str = "map/";

pub const MAP_BLOCK_FILE: &str = "map_blocks.png";
pub const DROID_BLOCK_FILE: &str = "droids.png";
pub const BULLET_BLOCK_FILE: &str = "bullet.png";
pub const BLAST_BLOCK_FILE: &str = "blast.png";
pub const DIGIT_BLOCK_FILE: &str = "digits.png";

pub const BANNER_BLOCK_FILE: &str = "banner.png";
pub const TITLE_PIC_FILE: &str = "title.jpg";
pub const CONSOLE_PIC_FILE: &str = "console_fg.png";
pub const CONSOLE_BG_PIC1_FILE: &str = "console_bg1.jpg";
pub const CONSOLE_BG_PIC2_FILE: &str = "console_bg2.jpg";
pub const TAKEOVER_BG_PIC_FILE: &str = "takeover_bg.jpg";
pub const CREDITS_PIC_FILE: &str = "credits.jpg";

pub const SHIP_ON_PIC_FILE: &str = "ship_on.png";
pub const SHIP_OFF_PIC_FILE: &str = "ship_off.png";

pub const PROGRESS_METER_FILE: &str = "progress_meter.png";
pub const PROGRESS_FILLER_FILE: &str = "progress_filler.png";

pub const STANDARD_MISSION: &str = "Paradroid.mission";
pub const NEW_MISSION: &str = "CleanPrivateGoodsStorageCellar.mission";

pub const PARA_FONT_FILE: &str = "parafont.png";
pub const FONT0_FILE: &str = "font05.png";
pub const FONT1_FILE: &str = "font05_green.png";
pub const FONT2_FILE: &str = "font05_red.png";
// const ICON_FILE: &str =		"paraicon.bmp";
pub const ICON_FILE: &str = "paraicon_48x48.png";

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
pub const BYCOLOR: &str = "BYCOLOR";

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
pub const SLOWMO_FACTOR: f64 = 0.33; // slow-motion effect on last blast when level is going empty
pub const WAIT_AFTER_KILLED: usize = 2000; // time (in ms) to wait and still display pictures after the destruction of
pub const SHOW_WAIT: usize = 3500; // std amount of time to show something
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

const_assert!(Direction::Oben as u8 == 0);
const_assert!(Direction::Light as u8 == 9);

macro_rules! direction_try_from {
    () => {};

    ($ty:ty $(, $( $rest:ty ),* )? $(,)* ) => {
        impl TryFrom<$ty> for Direction {
            type Error = InvalidDirection;

            fn try_from(value: $ty) -> Result<Self, Self::Error> {
                if value >= 0 && value <= 9 {
                    Ok(unsafe { mem::transmute(value as i32) })
                } else {
                    Err(InvalidDirection)
                }
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
pub enum Bullet {
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
pub struct InvalidFrameType;

impl TryFrom<u8> for MapTile {
    type Error = InvalidFrameType;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        const_assert!(MapTile::Floor as u8 == 0);
        const_assert!(MapTile::NumMapTiles as u8 == 44);
        if value < 44 {
            Ok(unsafe { mem::transmute(value as i32) })
        } else {
            Err(InvalidFrameType)
        }
    }
}
