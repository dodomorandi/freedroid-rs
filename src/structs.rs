use crate::defs::*;

use sdl::video::ll::{SDL_Rect, SDL_Surface};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ThemeList {
    num_themes: i32,
    cur_tnum: i32,
    theme_name: [*mut u8; MAX_THEMES],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HighscoreEntry {
    name: [i8; MAX_NAME_LEN + 5],
    score: i64, /* use -1 for an empty entry */
    date: [i8; DATE_LEN + 5],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    rot: u8,
    gruen: u8,
    blau: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Config {
    WantedTextVisibleTime: f32,
    Draw_Framerate: i32,
    Draw_Energy: i32,
    Draw_Position: i32,
    Draw_DeathCount: i32,
    Droid_Talk: i32,
    Current_BG_Music_Volume: f32,
    Current_Sound_FX_Volume: f32,
    Current_Gamma_Correction: f32,
    Theme_Name: [i8; 100], // name of graphics-theme : dirname = graphics/TNAME_theme/
    FullUserRect: i32,     // use "full" or "classic" (=small) User_Rect
    UseFullscreen: i32,    // toggle for use of fullscreen vs. X11-window
    TakeoverActivates: i32, // toggle if takeover-mode also does 'Activate' (i.e. lifts/consoles)
    FireHoldTakeover: i32, // Activate Takeover-mode after a delay if fire is held without a direction
    ShowDecals: i32,       // show dead droids-ashes...
    AllMapVisible: i32,    // complete map is visible?
    scale: f32,            // scale the whole graphics by this at load-time
    HogCPU: i32,           // use 100% CPU or leave it some air to breathe?
    emptyLevelSpeedup: f32, // time speedup factor to use on empty levels
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Finepoint {
    x: f32,
    y: f32,
}

pub type Vect = Finepoint;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GrobPoint {
    x: i8,
    y: i8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gps {
    x: f32,
    y: f32,
    z: i32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DruidSpec {
    druidname: [i8; 20],
    maxspeed: f32, /* the maximum of speed it can go */
    class: i32,
    accel: f32,       /* its acceleration */
    maxenergy: f32,   /* the maximum energy the batteries can carry */
    lose_health: f32, /* the energy/time the duid loses under influence-control */
    gun: i32,         /* Which gun does this druid use */
    aggression: i32,  /* The aggressiveness of this druidtype */
    flashimmune: i32, /* is the droid immune to FLASH-bullets */
    score: i32,       /* score for the elimination of one droid of this type */
    height: f32,      // the height of this droid
    weight: i32,      // the weight of this droid
    drive: i32,
    brain: i32,
    sensor1: i32,
    sensor2: i32,
    sensor3: i32,
    notes: *mut i8, /* notes on the druid of this type */
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Influence {
    ty: i32,          /* what kind of druid is this ? */
    status: i32,      /* attacking, defense, dead, ... */
    speed: Finepoint, /* the current speed of the druid */
    pos: Finepoint,   /* current position in level levelnum */
    health: f32,      /* the max. possible energy in the moment */
    energy: f32,      /* current energy */
    firewait: f32,    /* counter after fire */
    phase: f32,       /* the current phase of animation */
    timer: f32,
    LastCrysoundTime: f32,
    LastTransferSoundTime: f32,
    TextVisibleTime: f32,
    TextToBeDisplayed: *mut i8,
    Position_History_Ring_Buffer: [Gps; MAX_INFLU_POSITION_HISTORY],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Enemy {
    ty: i32,           /* gibt die Nummer in Druidmap an */
    levelnum: i32,     /* Level in dem sich enemy befindet */
    pos: Finepoint,    /* gibt die Koordinaten der Momentanposition an */
    speed: Finepoint,  /* current speed  */
    energy: f32,       /* gibt die Energie dieses Robots an */
    phase: f32,        /* gibt die Phase an in der der Feind gedreht ist */
    nextwaypoint: i32, /* gibt den naechsten Zielpunkt an */
    lastwaypoint: i32, /* Waypoint, von dem ausgegangen wurde */
    status: i32,       /* gibt z.B. an ob der Robotter abgeschossen wurde */
    warten: f32,       // time till the droid will start to move again
    passable: u8,      /* Zeit (counter), in der druid passable ist */
    firewait: f32,     /* gibt die Zeit bis zum naechsten Schuss an */
    TextVisibleTime: f32,
    TextToBeDisplayed: *mut i8,
    NumberOfPeriodicSpecialStatements: i32,
    PeriodicSpecialStatements: *mut *mut i8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BulletSpec {
    recharging_time: f32, // time until the next shot can be made, measures in seconds
    speed: f32,           /* speed of the bullet */
    damage: i32,          /* damage done by this bullettype */
    phases: i32,          /* how many phases in motion to show */
    phase_changes_per_second: f32, // how many different phases to display every second
    blast: i32,           /* which blast does this bullet create */
    SurfacePointer: [*mut SDL_Surface; MAX_PHASES_IN_A_BULLET], // A pointer to the surfaces containing
                                                                // the bullet images of this bullet
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bullet {
    pos: Finepoint,
    prev_pos: Finepoint, // use this for improved collision checks (for low FPS machines)
    speed: Finepoint,
    ty: u8,
    phase: u8,
    time_in_frames: i32, // how i64 does the bullet exist, measured in number of frames
    time_in_seconds: f32, // how i64 does the bullet exist in seconds
    mine: bool,
    owner: i32,
    angle: f32,
    Surfaces_were_generated: i32,
    SurfacePointer: [*mut SDL_Surface; MAX_PHASES_IN_A_BULLET],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlastSpec {
    phases: i32,
    picpointer: *mut u8,
    block: *mut SDL_Rect, /* the coordinates of the blocks in ne_blocks */
    total_animation_time: f32,
    SurfacePointer: [*mut SDL_Surface; MAX_PHASES_IN_A_BULLET], // A pointer to the surfaces containing
                                                                // the blast images of this blast type
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Blast {
    PX: f32, /* PosX */
    PY: f32, /* PosY */
    ty: i32,
    phase: f32,
    MessageWasDone: i32,
    mine: bool,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Lift {
    level: i32, // The level, where this elevtor entrance is located
    x: i32,     // The position in x of this elevator entrance within the level
    y: i32,     // The position in y of this elevator entrance within the level

    /* connections: Numbers in Lift-Array */
    up: i32,
    down: i32,

    lift_row: i32, // which lift column does this lift entrance belong to?
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Waypoint {
    x: u8, /* Grob */
    y: u8,
    num_connections: i32,
    connections: [i32; MAX_WP_CONNECTIONS],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Level {
    empty: i32,
    timer: f32,
    levelnum: i32,      /* Number of this level */
    Levelname: *mut i8, /* Name of this level */
    Background_Song_Name: *mut i8,
    Level_Enter_Comment: *mut i8,
    xlen: i32, /* X dimension */
    ylen: i32,
    color: i32,
    map: [*mut i8; MAX_MAP_ROWS], /* this is a vector of pointers ! */
    refreshes: [GrobPoint; MAX_REFRESHES_ON_LEVEL],
    doors: [GrobPoint; MAX_DOORS_ON_LEVEL],
    alerts: [GrobPoint; MAX_ALERTS_ON_LEVEL],
    num_waypoints: i32,
    AllWaypoints: [Waypoint; MAXWAYPOINTS],
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub struct Ship {
    num_levels: i32,
    num_lifts: i32,
    num_lift_rows: i32,
    AreaName: [i8; 100],
    AllLevels: [Level; MAX_LEVELS],
    AllLifts: [Lift; MAX_LIFTS],
    LiftRow_Rect: [SDL_Rect; MAX_LIFT_ROWS], /* the lift-row rectangles */
    Level_Rects: [[SDL_Rect; MAX_LEVELS]; MAX_LEVEL_RECTS], /* level rectangles */
    num_level_rects: [i32; MAX_LEVELS],      /* how many rects has a level */
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bar {
    pos: Point,
    len: i32,
    hgt: i32,
    oldval: i32,
    col: i32,
}
