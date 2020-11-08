use crate::{
    b_font::BFontInfo,
    defs::{
        AlertNames, Droid, ALLBLASTTYPES, ALLSHIPS, DIGITNUMBER, ENEMYPHASES, MAXBLASTS,
        MAXBULLETS, MAX_ENEMYS_ON_SHIP, MAX_LEVELS, MAX_LIFT_ROWS, NUM_COLORS, NUM_DECAL_PICS,
        NUM_MAP_BLOCKS,
    },
    highscore::HighscoreEntry,
    structs::{
        Blast, BlastSpec, Bullet, BulletSpec, Config, DruidSpec, Enemy, Influence, Level, Point,
        Ship, ThemeList,
    },
    takeover::{NUM_CAPS_BLOCKS, NUM_FILL_BLOCKS, NUM_GROUND_BLOCKS, NUM_TO_BLOCKS, TO_COLORS},
};

use sdl::{
    joy::ll::SDL_Joystick,
    mouse::ll::SDL_Cursor,
    video::ll::{SDL_Color, SDL_RWops, SDL_Rect, SDL_Surface},
};

extern "C" {
    #[no_mangle]
    pub static mut ConfigDir: [i8; 255];

    #[no_mangle]
    pub static mut OrigBlock_Rect: SDL_Rect;
    #[no_mangle]
    pub static mut Block_Rect: SDL_Rect;
    #[no_mangle]
    pub static mut Screen_Rect: SDL_Rect;
    #[no_mangle]
    pub static mut User_Rect: SDL_Rect;
    #[no_mangle]
    pub static mut Classic_User_Rect: SDL_Rect;
    #[no_mangle]
    pub static mut Full_User_Rect: SDL_Rect;
    #[no_mangle]
    pub static mut Banner_Rect: SDL_Rect;
    #[no_mangle]
    pub static mut Portrait_Rect: SDL_Rect;
    #[no_mangle]
    pub static mut Cons_Droid_Rect: SDL_Rect;
    #[no_mangle]
    pub static mut Menu_Rect: SDL_Rect;
    #[no_mangle]
    pub static mut OptionsMenu_Rect: SDL_Rect;
    #[no_mangle]
    pub static mut OrigDigit_Rect: SDL_Rect;
    #[no_mangle]
    pub static mut Digit_Rect: SDL_Rect;
    #[no_mangle]
    pub static mut FirstDigit_Rect: SDL_Rect;
    #[no_mangle]
    pub static mut SecondDigit_Rect: SDL_Rect;
    #[no_mangle]
    pub static mut ThirdDigit_Rect: SDL_Rect;

    #[no_mangle]
    pub static mut Cons_Header_Rect: SDL_Rect;
    #[no_mangle]
    pub static mut Cons_Menu_Rect: SDL_Rect;
    #[no_mangle]
    pub static mut Cons_Text_Rect: SDL_Rect;
    #[no_mangle]
    pub static mut Cons_Menu_Rects: [SDL_Rect; 4];

    #[no_mangle]
    pub static mut LeftInfo_Rect: SDL_Rect;
    #[no_mangle]
    pub static mut RightInfo_Rect: SDL_Rect;

    #[no_mangle]
    pub static mut ConsMenuItem_Rect: SDL_Rect;

    #[no_mangle]
    pub static mut ProgressMeter_Rect: SDL_Rect;
    #[no_mangle]
    pub static mut ProgressBar_Rect: SDL_Rect;
    #[no_mangle]
    pub static mut ProgressText_Rect: SDL_Rect;

    #[no_mangle]
    pub static mut LastRefreshSound: f32;
    #[no_mangle]
    pub static mut LastGotIntoBlastSound: f32;
    #[no_mangle]
    pub static mut FPSover1: f32;
    #[no_mangle]
    pub static mut Alertcolor: [*mut i8; AlertNames::Last as usize];
    #[no_mangle]
    pub static mut Shipnames: [*mut i8; ALLSHIPS];

    #[no_mangle]
    pub static mut Classname: *mut *mut i8;
    #[no_mangle]
    pub static mut Classes: *mut *mut i8;
    #[no_mangle]
    pub static mut Weaponnames: *mut *mut i8;
    #[no_mangle]
    pub static mut Sensornames: *mut *mut i8;
    #[no_mangle]
    pub static mut Brainnames: *mut *mut i8;
    #[no_mangle]
    pub static mut Drivenames: *mut *mut i8;
    #[no_mangle]
    pub static mut InfluenceModeNames: *mut *mut i8;
    #[no_mangle]
    pub static mut ThisMessageTime: i32;

    #[no_mangle]
    pub static mut Me: Influence; /* the influence data */
    #[no_mangle]
    pub static mut Druidmap: *mut DruidSpec;
    #[no_mangle]
    pub static mut Bulletmap: *mut BulletSpec;
    #[no_mangle]
    pub static mut Blastmap: [BlastSpec; ALLBLASTTYPES];

    #[no_mangle]
    pub static mut Number_Of_Droid_Types: i32;
    #[no_mangle]
    pub static mut PreTakeEnergy: i32;
    #[no_mangle]
    pub static mut QuitProgram: i32;
    #[no_mangle]
    pub static mut GameOver: i32;
    #[no_mangle]
    pub static mut InvincibleMode: i32;
    #[no_mangle]
    pub static mut HideInvisibleMap: i32;
    #[no_mangle]
    pub static mut AlertLevel: i32;
    #[no_mangle]
    pub static mut AlertThreshold: i32; // threshold for FIRST Alert-color (yellow), the others are 2*, 3*..
    #[no_mangle]
    pub static mut AlertBonusPerSec: f32; // bonus/sec for FIRST Alert-color, the others are 2*, 3*,...
    #[no_mangle]
    pub static mut DeathCount: f32; // a cumulative/draining counter of kills->determines Alert!
    #[no_mangle]
    pub static mut DeathCountDrainSpeed: f32; // drain per second
    #[no_mangle]
    pub static mut RealScore: f32;
    #[no_mangle]
    pub static mut ShowScore: i64;

    #[no_mangle]
    pub static mut AllEnemys: [Enemy; MAX_ENEMYS_ON_SHIP];

    #[no_mangle]
    pub static mut NumEnemys: i32;

    #[no_mangle]
    pub static mut CurLevel: *mut Level; /* the current level data */
    #[no_mangle]
    pub static mut curShip: Ship; /* the current ship-data */

    #[no_mangle]
    pub static mut AllBullets: [Bullet; MAXBULLETS + 10];
    #[no_mangle]
    pub static mut AllBlasts: [Blast; MAXBLASTS + 10];

    #[no_mangle]
    pub static mut sound_on: i32; /* Toggle TRUE/FALSE for turning sounds on/off */
    #[no_mangle]
    pub static mut debug_level: i32; /* 0=no debug 1=some debug messages 2=...etc */
    /* (currently only 0 or !=0 is implemented) */
    #[no_mangle]
    pub static mut show_all_droids: i32; /* display enemys regardless of IsVisible() */
    #[no_mangle]
    pub static mut stop_influencer: i32; /* for bullet debugging: stop where u are */

    #[no_mangle]
    pub static mut Time_For_Each_Phase_Of_Door_Movement: f32;
    #[no_mangle]
    pub static mut Blast_Damage_Per_Second: f32;
    #[no_mangle]
    pub static mut Blast_Radius: f32;
    #[no_mangle]
    pub static mut Droid_Radius: f32;
    #[no_mangle]
    pub static mut LevelDoorsNotMovedTime: f32;
    #[no_mangle]
    pub static mut collision_lose_energy_calibrator: f32;
    #[no_mangle]
    pub static mut GameConfig: Config;
    #[no_mangle]
    pub static mut CurrentCombatScaleFactor: f32;
    #[no_mangle]
    pub static mut Menu_BFont: *mut BFontInfo;
    #[no_mangle]
    pub static mut Para_BFont: *mut BFontInfo;
    #[no_mangle]
    pub static mut Highscore_BFont: *mut BFontInfo;
    #[no_mangle]
    pub static mut Font0_BFont: *mut BFontInfo;
    #[no_mangle]
    pub static mut Font1_BFont: *mut BFontInfo;
    #[no_mangle]
    pub static mut Font2_BFont: *mut BFontInfo;
    #[no_mangle]
    pub static mut SkipAFewFrames: i32;

    #[no_mangle]
    pub static mut Black: SDL_Color;

    #[no_mangle]
    pub static mut AllThemes: ThemeList;
    #[no_mangle]
    pub static mut classic_theme_index: i32;
    #[no_mangle]
    pub static mut crosshair_cursor: *mut SDL_Cursor;
    #[no_mangle]
    pub static mut arrow_cursor: *mut SDL_Cursor;
    #[no_mangle]
    pub static mut Number_Of_Bullet_Types: i32;
    #[no_mangle]
    pub static mut ne_screen: *mut SDL_Surface; /* the graphics display */

    #[no_mangle]
    pub static mut EnemySurfacePointer: [*mut SDL_Surface; ENEMYPHASES]; // A pointer to the surfaces containing the pictures of the
                                                                         // enemys in different phases of rotation
    #[no_mangle]
    pub static mut InfluencerSurfacePointer: [*mut SDL_Surface; ENEMYPHASES]; // A pointer to the surfaces containing the pictures of the
                                                                              // influencer in different phases of rotation
    #[no_mangle]
    pub static mut InfluDigitSurfacePointer: [*mut SDL_Surface; DIGITNUMBER]; // A pointer to the surfaces containing the pictures of the
                                                                              // influencer in different phases of rotation
    #[no_mangle]
    pub static mut EnemyDigitSurfacePointer: [*mut SDL_Surface; DIGITNUMBER]; // A pointer to the surfaces containing the pictures of the
                                                                              // influencer in different phases of rotation
    #[no_mangle]
    pub static mut MapBlockSurfacePointer: [[*mut SDL_Surface; NUM_MAP_BLOCKS]; NUM_COLORS]; // A pointer to the surfaces containing the map-pics, which may be rescaled with respect to
    #[no_mangle]
    pub static mut OrigMapBlockSurfacePointer: [[*mut SDL_Surface; NUM_MAP_BLOCKS]; NUM_COLORS]; // A pointer to the surfaces containing the original map-pics as read from disk
    #[no_mangle]
    pub static mut BuildBlock: *mut SDL_Surface; // a block for temporary pic-construction

    #[no_mangle]
    pub static mut BannerIsDestroyed: i32;

    #[no_mangle]
    pub static mut banner_pic: *mut SDL_Surface; /* the banner pic */
    #[no_mangle]
    pub static mut pic999: *mut SDL_Surface;
    #[no_mangle]
    pub static mut packed_portraits: [*mut SDL_RWops; Droid::NumDroids as usize];

    #[no_mangle]
    pub static mut Decal_pics: [*mut SDL_Surface; NUM_DECAL_PICS];

    #[no_mangle]
    pub static mut takeover_bg_pic: *mut SDL_Surface;
    #[no_mangle]
    pub static mut console_pic: *mut SDL_Surface;
    #[no_mangle]
    pub static mut console_bg_pic1: *mut SDL_Surface;
    #[no_mangle]
    pub static mut console_bg_pic2: *mut SDL_Surface;

    #[no_mangle]
    pub static mut arrow_up: *mut SDL_Surface;
    #[no_mangle]
    pub static mut arrow_down: *mut SDL_Surface;
    #[no_mangle]
    pub static mut arrow_right: *mut SDL_Surface;
    #[no_mangle]
    pub static mut arrow_left: *mut SDL_Surface;

    #[no_mangle]
    pub static mut ship_off_pic: *mut SDL_Surface; /* Side-view of ship: lights off */
    #[no_mangle]
    pub static mut ship_on_pic: *mut SDL_Surface; /* Side-view of ship: lights on */

    #[no_mangle]
    pub static mut progress_meter_pic: *mut SDL_Surface;
    #[no_mangle]
    pub static mut progress_filler_pic: *mut SDL_Surface;

    #[no_mangle]
    pub static mut level_rect: [SDL_Rect; MAX_LEVELS]; /* rect's of levels in side-view */
    #[no_mangle]
    pub static mut liftrow_rect: [SDL_Rect; MAX_LIFT_ROWS]; /* the lift-row rect's in side-view*/

    #[no_mangle]
    pub static mut joy: *mut SDL_Joystick;
    #[no_mangle]
    pub static mut joy_num_axes: i32; /* number of joystick axes */
    #[no_mangle]
    pub static mut joy_sensitivity: i32;
    #[no_mangle]
    pub static mut input_axis: Point; /* joystick (and mouse) axis values */
    #[no_mangle]
    pub static mut axis_is_active: i32; /* is firing to use axis-values or not */
    #[no_mangle]
    pub static mut last_mouse_event: u32; // SDL-ticks of last mouse event

    #[no_mangle]
    pub static mut Highscores: *mut *mut HighscoreEntry;
    #[no_mangle]
    pub static mut num_highscores: i32; /* total number of entries in our list (fixed) */

    #[no_mangle]
    pub static mut to_blocks: *mut SDL_Surface; /* the global surface containing all game-blocks */
    /* the rectangles containing the blocks */
    #[no_mangle]
    pub static mut FillBlocks: [SDL_Rect; NUM_FILL_BLOCKS];
    #[no_mangle]
    pub static mut CapsuleBlocks: [SDL_Rect; NUM_CAPS_BLOCKS];
    #[no_mangle]
    pub static mut ToGameBlocks: [SDL_Rect; NUM_TO_BLOCKS];
    #[no_mangle]
    pub static mut ToGroundBlocks: [SDL_Rect; NUM_GROUND_BLOCKS];
    #[no_mangle]
    pub static mut ToColumnBlock: SDL_Rect;
    #[no_mangle]
    pub static mut ToLeaderBlock: SDL_Rect;

    #[no_mangle]
    pub static mut LeftCapsulesStart: [Point; TO_COLORS];
    #[no_mangle]
    pub static mut CurCapsuleStart: [Point; TO_COLORS];
    #[no_mangle]
    pub static mut PlaygroundStart: [Point; TO_COLORS];
    #[no_mangle]
    pub static mut DruidStart: [Point; TO_COLORS];
    #[no_mangle]
    pub static mut TO_LeftGroundStart: Point;
    #[no_mangle]
    pub static mut TO_RightGroundStart: Point;
    #[no_mangle]
    pub static mut TO_ColumnStart: Point;
    #[no_mangle]
    pub static mut TO_LeaderBlockStart: Point;

    #[no_mangle]
    pub static mut TO_LeaderLed: SDL_Rect;
    #[no_mangle]
    pub static mut TO_FillBlock: SDL_Rect;
    #[no_mangle]
    pub static mut TO_ElementRect: SDL_Rect;
    #[no_mangle]
    pub static mut TO_CapsuleRect: SDL_Rect;
    #[no_mangle]
    pub static mut TO_GroundRect: SDL_Rect;
    #[no_mangle]
    pub static mut TO_ColumnRect: SDL_Rect;

    #[no_mangle]
    pub static mut quit_LevelEditor: bool;
    #[no_mangle]
    pub static mut quit_Menu: bool;
}
