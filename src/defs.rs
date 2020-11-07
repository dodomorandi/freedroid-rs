use bitflags::bitflags;

const MAX_THEMES: usize = 100;

const JOY_MAX_VAL: usize = 32767; // maximal amplitude of joystick axis values

const RESET: usize = 0x01;
const UPDATE: usize = 0x02;
const INIT_ONLY: usize = 0x04;
const FREE_ONLY: usize = 0x08;

const DROID_ROTATION_TIME: f64 = 3.0;
const NUM_DECAL_PICS: usize = 2;

// #define UserCenter_x (User_Rect.x + User_Rect.w/2)
// #define UserCenter_y (User_Rect.y + User_Rect.h/2)

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

// #define ShiftPressed() ModIsPressed(KMOD_SHIFT)
// #define AltPressed() ModIsPressed(KMOD_ALT)
// #define CtrlPressed() ModIsPressed(KMOD_CTRL)

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
// #define Gcw0RSPressed() (KeyIsPressed(SDLK_BACKSPACE))
// #define Gcw0LSPressed() (KeyIsPressed(SDLK_TAB))
// #define Gcw0StartPressed() (KeyIsPressed(SDLK_RETURN))
// #define Gcw0SelectPressed() (KeyIsPressed(SDLK_ESCAPE))

// #define Gcw0AnyButtonPressed() (Gcw0APressed() || Gcw0BPressed()\
//         || Gcw0XPressed() || Gcw0YPressed() || Gcw0LSPressed() || Gcw0RSPressed()\
// 	|| Gcw0StartPressed() || Gcw0SelectPressed())

// #define Gcw0APressedR() (KeyIsPressedR(SDLK_LCTRL))
// #define Gcw0BPressedR() (KeyIsPressedR(SDLK_LALT))
// #define Gcw0XPressedR() (KeyIsPressedR(SDLK_LSHIFT))
// #define Gcw0YPressedR() (KeyIsPressedR(SDLK_SPACE))
// #define Gcw0RSPressedR() (KeyIsPressedR(SDLK_BACKSPACE))
// #define Gcw0LSPressedR() (KeyIsPressedR(SDLK_TAB))
// #define Gcw0StartPressedR() (KeyIsPressedR(SDLK_RETURN))
// #define Gcw0SelectPressedR() (KeyIsPressedR(SDLK_ESCAPE))

// #define Gcw0AnyButtonPressedR() (Gcw0APressedR() || Gcw0BPressedR()\
//         || Gcw0XPressedR() || Gcw0YPressedR() || Gcw0LSPressedR()\
// 	|| Gcw0RSPressedR() || Gcw0StartPressedR() || Gcw0SelectPressedR())
// #endif // GCW0 keys

// #define UpPressed() (cmd_is_active(CMD_UP))
// #define DownPressed() (cmd_is_active(CMD_DOWN))
// #define LeftPressed() (cmd_is_active(CMD_LEFT))
// #define RightPressed() (cmd_is_active(CMD_RIGHT))

// #define FirePressed() (cmd_is_active(CMD_FIRE))
// #define FirePressedR() (cmd_is_activeR(CMD_FIRE))

// #define UpPressedR() (cmd_is_activeR(CMD_UP))
// #define DownPressedR() (cmd_is_activeR(CMD_DOWN))
// #define LeftPressedR() (cmd_is_activeR(CMD_LEFT))
// #define RightPressedR() (cmd_is_activeR(CMD_RIGHT))

// #define AnyCmdActive() (cmd_is_active(CMD_FIRE) || cmd_is_active(CMD_ACTIVATE) || cmd_is_active(CMD_TAKEOVER) )
// #define AnyCmdActiveR() (cmd_is_activeR(CMD_FIRE) || cmd_is_activeR(CMD_ACTIVATE) || cmd_is_activeR(CMD_TAKEOVER) )

// ----------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum MenuAction {
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

const COLLISION_STEPSIZE: f64 = 0.1;

/* ************************************************************
 * Highscore related defines
 *************************************************************/
const HS_BACKGROUND_FILE: &str = "transfer.jpg";
const HS_EMPTY_ENTRY: &str = "--- empty ---";
const MAX_NAME_LEN: usize = 15; /* max len of highscore name entry */
const MAX_HIGHSCORES: usize = 10; /* only keep Top10 */
const DATE_LEN: usize = 10; /* reserved for the date-string */
//***************************************************************

// find_file(): use current-theme subdir in search or not
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Themed {
    NoTheme = 0,
    UseTheme,
}
// find_file(): how important is the file in question:
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Criticality {
    Ignore = 0, // ignore if not found and return NULL
    WarnOnly,   // warn if not found and return NULL
    Critical,   // Error-message and Terminate
}

// The flags for DisplayBanner are:
bitflags! {
    struct DisplayBannerFlags: u8 {
        const FORCE_UPDATE=1;
        const DONT_TOUCH_TEXT=2;
        const NO_SDL_UPDATE=4;
    }
}

// The flags for AssembleCombatWindow are:
bitflags! {
    struct AssembleCombatWindowFlags: u8 {
        const ONLY_SHOW_MAP = 0x01;
        const DO_SCREEN_UPDATE = 0x02;
        const SHOW_FULL_MAP = 0x04;
    }
}

// symbolic Alert-names
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum AlertNames {
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
const FD_DATADIR: &str = "FreeDroid.app/Contents/Resources"; // our local fallback

#[cfg(not(target_os = "macosx"))]
const FD_DATADIR: &str = "."; // our local fallback

// #endif // !FD_DATADIR

// #ifndef LOCAL_DATADIR
const LOCAL_DATADIR: &str = ".."; // local fallback
                                  // #endif

const GRAPHICS_DIR: &str = "graphics/";
const SOUND_DIR: &str = "sound/";
const MAP_DIR: &str = "map/";

const MAP_BLOCK_FILE: &str = "map_blocks.png";
const DROID_BLOCK_FILE: &str = "droids.png";
const BULLET_BLOCK_FILE: &str = "bullet.png";
const BLAST_BLOCK_FILE: &str = "blast.png";
const DIGIT_BLOCK_FILE: &str = "digits.png";

const BANNER_BLOCK_FILE: &str = "banner.png";
const TITLE_PIC_FILE: &str = "title.jpg";
const CONSOLE_PIC_FILE: &str = "console_fg.png";
const CONSOLE_BG_PIC1_FILE: &str = "console_bg1.jpg";
const CONSOLE_BG_PIC2_FILE: &str = "console_bg2.jpg";
const TAKEOVER_BG_PIC_FILE: &str = "takeover_bg.jpg";
const CREDITS_PIC_FILE: &str = "credits.jpg";

const SHIP_ON_PIC_FILE: &str = "ship_on.png";
const SHIP_OFF_PIC_FILE: &str = "ship_off.png";

const PROGRESS_METER_FILE: &str = "progress_meter.png";
const PROGRESS_FILLER_FILE: &str = "progress_filler.png";

const STANDARD_MISSION: &str = "Paradroid.mission";
const NEW_MISSION: &str = "CleanPrivateGoodsStorageCellar.mission";

const PARA_FONT_FILE: &str = "parafont.png";
const FONT0_FILE: &str = "font05.png";
const FONT1_FILE: &str = "font05_green.png";
const FONT2_FILE: &str = "font05_red.png";
// const ICON_FILE: &str =		"paraicon.bmp";
const ICON_FILE: &str = "paraicon_48x48.png";

// **********************************************************************

const DIGITNUMBER: usize = 10;

const TEXT_STRETCH: f64 = 1.2;
const LEFT_TEXT_LEN: usize = 10;
const RIGHT_TEXT_LEN: usize = 6;

const BULLET_BULLET_COLLISION_DIST: f64 = 10.0 / 64.0;
const BULLET_COLL_DIST2: f64 = 0.0244140625;
// **********************************************************************
//
//

// The following is the definition of the sound file names used in freedroid
// DO NOT EVER CHANGE THE ORDER OF APPEARENCE IN THIS LIST PLEASE!!!!!
// The order of appearance here should match the order of appearance
// in the SoundSampleFilenames definition located in sound.c!
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
const BYCOLOR: &str = "BYCOLOR";

// The sounds when the influencers energy is low or when he is in transfer mode
// occur periodically.  These constants specify which intervals are to be used
// for these periodic happenings...
const CRY_SOUND_INTERVAL: usize = 2;
const TRANSFER_SOUND_INTERVAL: f64 = 1.1;

// **********************************************************************

const ERR: i8 = -1;
const OK: i8 = 0;

/* Ship-Elevator Picture */

const DIRECTIONS: usize = 8;

const ALLSHIPS: usize = 4;
const ENEMYPHASES: usize = 8;
const DROID_PHASES: usize = ENEMYPHASES;

const WAIT_LEVELEMPTY: f64 = 0.5; /* warte bevor Graufaerben (in seconds)*/
const SLOWMO_FACTOR: f64 = 0.33; // slow-motion effect on last blast when level is going empty
const WAIT_AFTER_KILLED: usize = 2000; // time (in ms) to wait and still display pictures after the destruction of
const SHOW_WAIT: usize = 3500; // std amount of time to show something
                               // the players droid.  This is now measured in seconds and can be a float
const WAIT_SHIPEMPTY: usize = 20;
const WAIT_TRANSFERMODE: f64 = 0.3; /* this is a "float" indicating the number of seconds the influence
                                    stand still with space pressed, before switching into transfermode
                                    This variable describes the amount in SECONDS */
const WAIT_COLLISION: usize = 1; // after a little collision with influ, enemys hold position for a while
                                 // this variable describes the amount of time in SECONDS
const ENEMYMAXWAIT: f64 = 2.0; // after each robot has reached its current destination waypoint is waits a
                               // while.  This variable describes the amount of time in SECONDS.  However,
                               // the final wait time is a random number within [0,ENEMYMAXWAIT].
const FLASH_DURATION: f64 = 0.1; // in seconds

/* direction definitions (fireing bullets and testing blockedness of positions) */
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Direction {
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

/* Maximal number of ... */

const NUM_MAP_BLOCKS: usize = 51; // total number of map-blocks
const NUM_COLORS: usize = 7; // how many different level colorings?/different tilesets?

// const #define: usize = ALLBULLETTYPES;		4	/* number of bullet-types */
const ALLBLASTTYPES: usize = 2; /* number of different exposions */

const MAXBULLETS: usize = 100; /* maximum possible Bullets in the air */
const MAXBLASTS: usize = 100; /* max. possible Blasts visible */
const AGGRESSIONMAX: usize = 100;
const ROBOT_MAX_WAIT_BETWEEN_SHOTS: usize = 5; // how long shoud each droid wait at most until
                                               // is considers fireing again?

/* Map-related defines:
    WARNING leave them here, they are required in struct.h
*/
const MAX_WP_CONNECTIONS: usize = 12;
const MAX_MAP_ROWS: usize = 255;
const MAX_MAP_COLS: usize = 255;
const MAX_ENEMYS_ON_SHIP: usize = 300;
const MAX_CHAT_KEYWORDS_PER_DROID: usize = 30;
const MAX_INFLU_POSITION_HISTORY: usize = 100;

const MAX_LIFTS: usize = 50; /* actually the entries to the lifts */
const MAX_LEVELS: usize = 29; /* don't change this easily */
/* corresponds to a reserved palette range ! */
const MAX_LIFT_ROWS: usize = 15; /* the different lift "rows" */
/* don't change this easily */
/* corresponds to a reserved palette range !*/
const MAX_LEVEL_RECTS: usize = 20; // how many rects compose a level
const MAX_EVENT_TRIGGERS: usize = 20; // how many event triggers at most to allow
const MAX_TRIGGERED_ACTIONS: usize = 20; // how many triggerable actions to allow at most

const MAXWAYPOINTS: usize = 100;
const MAX_DOORS_ON_LEVEL: usize = 60;
const MAX_REFRESHES_ON_LEVEL: usize = 40;
const MAX_ALERTS_ON_LEVEL: usize = 40;
const MAX_TELEPORTERS_ON_LEVEL: usize = 10;

const MAX_PHASES_IN_A_BULLET: usize = 12;
const MAX_STEPS_IN_GIVEN_COURSE: usize = 1000;

const BREMSDREHUNG: usize = 3; /* warte 3*, bevor Influencer weitergedreht wird */

/* Wegstossgeschw. von Tueren u.ae. */
// NORMALISATION #define PUSHSPEED 2
const PUSHSPEED: usize = 2;

/* Schusstypen */
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Bullet {
    Pulse = 0,
    SinglePulse,
    Military,
    Flash,
    Exterminator,
    LaserRifle,
}

/* Explosionstypen */
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Explosion {
    Bulletblast = 0,
    Druidblast,
    Rejectblast,
}

const BLINKENERGY: usize = 25;

/* Druidtypen */
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Druid {
    Druid001 = 0, /* You will know why are the numbers there, when you */
    Druid123 = 1, /* enter the crew of a level !! */
    Druid139 = 2,
    Druid247 = 3,
    Druid249 = 4,
    Druid296 = 5,
    Druid302 = 6,
    Druid329 = 7,
    Druid420 = 8,
    Druid476 = 9,
    Druid493 = 10,
    Druid516 = 11,
    Druid571 = 12,
    Druid598 = 13,
    Druid614 = 14,
    Druid615 = 15,
    Druid629 = 16,
    Druid711 = 17,
    Druid742 = 18,
    Druid751 = 19,
    Druid821 = 20,
    Druid834 = 21,
    Druid883 = 22,
    Druid999 = 23,
    NumDroids,
}

/* Status- Werte der Druids */
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

const DECKCOMPLETEBONUS: usize = 500;

/* Konstanten die die Kartenwerte anschaulich machen */
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UnknownVariant {
    Floor = 0,
    EckLu = 1,
    TU,
    EckRu,
    TL,
    Kreuz,
    TR,
    EckLo,
    TO,
    EckRo,
    HWall = 10,
    VWall,
    Invisible,
    Block1,
    Block2,
    Block3,
    Block4,
    Block5,
    HZutuere = 18,
    HHalbtuere1,
    HHalbtuere2,
    HHalbtuere3,
    HGanztuere,
    KonsoleL = 23,
    KonsoleR,
    KonsoleO,
    KonsoleU,
    VZutuere = 27,
    VHalbtuere1,
    VHalbtuere2,
    VHalbtuere3,
    VGanztuere,
    Lift = 32,
    Void = 33,
    Refresh1 = 34,
    Refresh2,
    Refresh3,
    Refresh4,
    AlertGreen = 38,
    AlertYellow,
    AlertAmber,
    AlertRed,
    Unused2 = 42,
    FineGrid,
    NumMapTiles,
}
