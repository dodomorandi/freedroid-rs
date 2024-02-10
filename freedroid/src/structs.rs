use crate::{
    array_c_string::ArrayCString,
    defs::{
        BulletKind, Droid, MapTile, Status, DATE_LEN, MAXWAYPOINTS, MAX_ALERTS_ON_LEVEL,
        MAX_DOORS_ON_LEVEL, MAX_INFLU_POSITION_HISTORY, MAX_LEVELS, MAX_LEVEL_RECTS, MAX_LIFTS,
        MAX_LIFT_ROWS, MAX_MAP_ROWS, MAX_NAME_LEN, MAX_PHASES_IN_A_BULLET, MAX_REFRESHES_ON_LEVEL,
        MAX_THEMES, MAX_WP_CONNECTIONS,
    },
    map,
};

use arrayvec::ArrayVec;
use sdl::{convert::u8_to_usize, Rect, Surface};
use std::{
    array,
    ffi::{CStr, CString},
    num::NonZeroU8,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ThemeList {
    pub len: NonZeroU8,
    pub current: u8,
    pub names: [CString; MAX_THEMES],
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HighscoreEntry {
    pub name: [i8; u8_to_usize(MAX_NAME_LEN) + 5],
    pub score: i64, /* use -1 for an empty entry */
    pub date: [i8; u8_to_usize(DATE_LEN) + 5],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub rot: u8,
    pub gruen: u8,
    pub blau: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub wanted_text_visible_time: f32,
    pub draw_framerate: i32,
    pub draw_energy: i32,
    pub draw_position: i32,
    pub draw_death_count: i32,
    pub droid_talk: i32,
    pub current_bg_music_volume: f32,
    pub current_sound_fx_volume: f32,
    pub current_gamma_correction: f32,
    pub theme_name: ArrayCString<100>, // name of graphics-theme : dirname = graphics/TNAME_theme/
    pub full_user_rect: i32,           // use "full" or "classic" (=small) User_Rect
    pub use_fullscreen: i32,           // toggle for use of fullscreen vs. X11-window
    pub takeover_activates: i32, // toggle if takeover-mode also does 'Activate' (i.e. lifts/consoles)
    pub fire_hold_takeover: i32, // Activate Takeover-mode after a delay if fire is held without a direction
    pub show_decals: i32,        // show dead droids-ashes...
    pub all_map_visible: i32,    // complete map is visible?
    pub scale: f32,              // scale the whole graphics by this at load-time
    pub hog_cpu: i32,            // use 100% CPU or leave it some air to breathe?
    pub empty_level_speedup: f32, // time speedup factor to use on empty levels
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Finepoint {
    pub x: f32,
    pub y: f32,
}

impl Finepoint {
    pub const fn default_const() -> Self {
        Self { x: 0., y: 0. }
    }
}

pub type Vect = Finepoint;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CoarsePoint<T> {
    pub x: T,
    pub y: T,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gps {
    pub x: f32,
    pub y: f32,
    pub z: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DruidSpec {
    pub druidname: ArrayCString<20>,
    pub maxspeed: f32, /* the maximum of speed it can go */
    pub class: u8,
    pub accel: f32,       /* its acceleration */
    pub maxenergy: f32,   /* the maximum energy the batteries can carry */
    pub lose_health: f32, /* the energy/time the duid loses under influence-control */
    pub gun: BulletKind,  /* Which gun does this druid use */
    pub aggression: i32,  /* The aggressiveness of this druidtype */
    pub flashimmune: i32, /* is the droid immune to FLASH-bullets */
    pub score: i32,       /* score for the elimination of one droid of this type */
    pub height: f32,      // the height of this droid
    pub weight: i32,      // the weight of this droid
    pub drive: i32,
    pub brain: i32,
    pub sensor1: i32,
    pub sensor2: i32,
    pub sensor3: i32,
    pub notes: CString, /* notes on the druid of this type */
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextToBeDisplayed {
    None,
    String(&'static CStr),
    LevelEnterComment,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Influence {
    pub ty: Droid,        /* what kind of druid is this ? */
    pub status: Status,   /* attacking, defense, dead, ... */
    pub speed: Finepoint, /* the current speed of the druid */
    pub pos: Finepoint,   /* current position in level levelnum */
    pub health: f32,      /* the max. possible energy in the moment */
    pub energy: f32,      /* current energy */
    pub firewait: f32,    /* counter after fire */
    pub phase: f32,       /* the current phase of animation */
    pub timer: f32,
    pub last_crysound_time: f32,
    pub last_transfer_sound_time: f32,
    pub text_visible_time: f32,
    pub text_to_be_displayed: TextToBeDisplayed,
    pub position_history_ring_buffer: [Gps; MAX_INFLU_POSITION_HISTORY],
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enemy {
    pub ty: Droid,        /* gibt die Nummer in Druidmap an */
    pub levelnum: u8,     /* Level in dem sich enemy befindet */
    pub pos: Finepoint,   /* gibt die Koordinaten der Momentanposition an */
    pub speed: Finepoint, /* current speed  */
    pub energy: f32,      /* gibt die Energie dieses Robots an */
    pub phase: f32,       /* gibt die Phase an in der der Feind gedreht ist */
    pub nextwaypoint: u8, /* gibt den naechsten Zielpunkt an */
    pub lastwaypoint: u8, /* Waypoint, von dem ausgegangen wurde */
    pub status: Status,   /* gibt z.B. an ob der Robotter abgeschossen wurde */
    pub warten: f32,      // time till the droid will start to move again
    pub firewait: f32,    /* gibt die Zeit bis zum naechsten Schuss an */
    pub text_visible_time: f32,
    pub text_to_be_displayed: &'static str,
}

impl Enemy {
    pub fn new(ty: Droid, levelnum: u8) -> Self {
        Self {
            ty,
            levelnum,
            pos: Finepoint::default(),
            speed: Finepoint::default(),
            energy: 0.,
            phase: 0.,
            nextwaypoint: 0,
            lastwaypoint: 0,
            status: Status::Mobile,
            warten: 0.,
            firewait: 0.,
            text_visible_time: 0.,
            text_to_be_displayed: "",
        }
    }
}

#[derive(Debug, Default)]
pub struct BulletSpec<'sdl> {
    pub recharging_time: f32, // time until the next shot can be made, measures in seconds
    pub speed: f32,           /* speed of the bullet */
    pub damage: u16,          /* damage done by this bullettype */
    pub phases: u8,           /* how many phases in motion to show */
    pub phase_changes_per_second: u16, // how many different phases to display every second
    pub surfaces: [Option<Surface<'sdl>>; MAX_PHASES_IN_A_BULLET], // A pointer to the surfaces containing
                                                                   // the bullet images of this bullet
}

#[derive(Debug)]
pub struct Bullet<'sdl> {
    pub pos: Finepoint,
    pub prev_pos: Finepoint, // use this for improved collision checks (for low FPS machines)
    pub speed: Finepoint,
    pub ty: BulletKind,
    pub phase: u8,
    pub time_in_frames: i32, // how i64 does the bullet exist, measured in number of frames
    pub time_in_seconds: f32, // how i64 does the bullet exist in seconds
    pub mine: bool,
    pub angle: f32,
    pub surfaces_were_generated: i32,
    pub surfaces: [Option<Surface<'sdl>>; MAX_PHASES_IN_A_BULLET],
}

impl Bullet<'_> {
    pub const fn default_const() -> Self {
        Bullet {
            pos: Finepoint::default_const(),
            prev_pos: Finepoint::default_const(),
            speed: Finepoint::default_const(),
            ty: BulletKind::Pulse,
            phase: 0,
            time_in_frames: 0,
            time_in_seconds: 0.,
            mine: false,
            angle: 0.,
            surfaces_were_generated: 0,
            surfaces: [
                None, None, None, None, None, None, None, None, None, None, None, None,
            ],
        }
    }
}

#[derive(Debug)]
pub struct BlastSpec<'sdl> {
    pub phases: i32,
    pub total_animation_time: f32,
    pub surfaces: [Option<Surface<'sdl>>; MAX_PHASES_IN_A_BULLET], // A pointer to the surfaces containing
                                                                   // the blast images of this blast type
}

impl BlastSpec<'_> {
    pub const fn default_const() -> Self {
        Self {
            phases: 0,
            total_animation_time: 0.,
            surfaces: [
                None, None, None, None, None, None, None, None, None, None, None, None,
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Blast {
    pub px: f32, /* PosX */
    pub py: f32, /* PosY */
    pub ty: i32,
    pub phase: f32,
    pub message_was_done: i32,
    pub mine: bool,
}

impl Default for Blast {
    fn default() -> Self {
        Self {
            px: 0.,
            py: 0.,
            ty: 0,
            phase: 0.,
            message_was_done: 0,
            mine: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Lift {
    pub level: u8, // The level, where this elevtor entrance is located
    pub x: i32,    // The position in x of this elevator entrance within the level
    pub y: i32,    // The position in y of this elevator entrance within the level

    /* connections: Numbers in Lift-Array */
    pub up: i32,
    pub down: i32,

    pub row: i32, // which lift column does this lift entrance belong to?
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Waypoint {
    pub x: u8, /* Coarse */
    pub y: u8,
    pub connections: ArrayVec<u8, { u8_to_usize(MAX_WP_CONNECTIONS) }>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Level {
    pub empty: i32,
    pub timer: f32,
    pub levelnum: u8,       /* Number of this level */
    pub levelname: CString, /* Name of this level */
    pub background_song_name: CString,
    pub enter_comment: CString,
    pub xlen: u8, /* X dimension */
    pub ylen: u8,
    pub color: map::Color,
    pub map: [Vec<MapTile>; u8_to_usize(MAX_MAP_ROWS)],
    pub refreshes: [Option<CoarsePoint<u8>>; MAX_REFRESHES_ON_LEVEL],
    pub doors: [Option<CoarsePoint<u8>>; MAX_DOORS_ON_LEVEL],
    pub alerts: [Option<CoarsePoint<u8>>; MAX_ALERTS_ON_LEVEL],
    pub num_waypoints: u8,
    pub all_waypoints: [Waypoint; u8_to_usize(MAXWAYPOINTS)],
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ship {
    pub num_levels: u8,
    pub num_lifts: i32,
    pub num_lift_rows: u8,
    pub area_name: ArrayCString<100>,
    pub all_levels: [Option<Level>; MAX_LEVELS],
    pub all_lifts: [Lift; MAX_LIFTS],
    pub lift_row_rect: [Rect; MAX_LIFT_ROWS], /* the lift-row rectangles */
    pub level_rects: [[Rect; MAX_LEVEL_RECTS]; MAX_LEVELS], /* level rectangles */
    pub num_level_rects: [u8; MAX_LEVELS],    /* how many rects has a level */
}

impl Default for Ship {
    fn default() -> Self {
        Self {
            num_levels: 0,
            num_lifts: 0,
            num_lift_rows: 0,
            area_name: ArrayCString::default(),
            all_levels: array::from_fn(|_| None),
            all_lifts: array::from_fn(|_| Lift {
                level: 0,
                x: 0,
                y: 0,
                up: 0,
                down: 0,
                row: 0,
            }),
            lift_row_rect: [Rect::default(); MAX_LIFT_ROWS],
            level_rects: [[Rect::default(); MAX_LEVEL_RECTS]; MAX_LEVELS],
            num_level_rects: [0; MAX_LEVELS],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Bar {
    pub pos: Point,
    pub len: i32,
    pub hgt: i32,
    pub oldval: i32,
    pub col: i32,
}
