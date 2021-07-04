use crate::defs::*;

use sdl_sys::{SDL_Rect, SDL_Surface};
use std::ptr::null_mut;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ThemeList {
    pub num_themes: i32,
    pub cur_tnum: i32,
    pub theme_name: [*mut u8; MAX_THEMES],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HighscoreEntry {
    pub name: [i8; MAX_NAME_LEN + 5],
    pub score: i64, /* use -1 for an empty entry */
    pub date: [i8; DATE_LEN + 5],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub rot: u8,
    pub gruen: u8,
    pub blau: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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
    pub theme_name: [i8; 100], // name of graphics-theme : dirname = graphics/TNAME_theme/
    pub full_user_rect: i32,   // use "full" or "classic" (=small) User_Rect
    pub use_fullscreen: i32,   // toggle for use of fullscreen vs. X11-window
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
pub struct GrobPoint {
    pub x: i8,
    pub y: i8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gps {
    pub x: f32,
    pub y: f32,
    pub z: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DruidSpec {
    pub druidname: [i8; 20],
    pub maxspeed: f32, /* the maximum of speed it can go */
    pub class: i32,
    pub accel: f32,       /* its acceleration */
    pub maxenergy: f32,   /* the maximum energy the batteries can carry */
    pub lose_health: f32, /* the energy/time the duid loses under influence-control */
    pub gun: i32,         /* Which gun does this druid use */
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
    pub notes: *mut i8, /* notes on the druid of this type */
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Influence {
    pub ty: i32,          /* what kind of druid is this ? */
    pub status: i32,      /* attacking, defense, dead, ... */
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
    pub text_to_be_displayed: *mut i8,
    pub position_history_ring_buffer: [Gps; MAX_INFLU_POSITION_HISTORY],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Enemy {
    pub ty: i32,           /* gibt die Nummer in Druidmap an */
    pub levelnum: i32,     /* Level in dem sich enemy befindet */
    pub pos: Finepoint,    /* gibt die Koordinaten der Momentanposition an */
    pub speed: Finepoint,  /* current speed  */
    pub energy: f32,       /* gibt die Energie dieses Robots an */
    pub phase: f32,        /* gibt die Phase an in der der Feind gedreht ist */
    pub nextwaypoint: i32, /* gibt den naechsten Zielpunkt an */
    pub lastwaypoint: i32, /* Waypoint, von dem ausgegangen wurde */
    pub status: i32,       /* gibt z.B. an ob der Robotter abgeschossen wurde */
    pub warten: f32,       // time till the droid will start to move again
    pub passable: u8,      /* Zeit (counter), in der druid passable ist */
    pub firewait: f32,     /* gibt die Zeit bis zum naechsten Schuss an */
    pub text_visible_time: f32,
    pub text_to_be_displayed: *mut i8,
    pub number_of_periodic_special_statements: i32,
    pub periodic_special_statements: *mut *mut i8,
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            ty: 0,
            levelnum: 0,
            pos: Finepoint { x: 0., y: 0. },
            speed: Finepoint { x: 0., y: 0. },
            energy: 0.,
            phase: 0.,
            nextwaypoint: 0,
            lastwaypoint: 0,
            status: 0,
            warten: 0.,
            passable: 0,
            firewait: 0.,
            text_visible_time: 0.,
            text_to_be_displayed: null_mut(),
            number_of_periodic_special_statements: 0,
            periodic_special_statements: null_mut(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BulletSpec {
    pub recharging_time: f32, // time until the next shot can be made, measures in seconds
    pub speed: f32,           /* speed of the bullet */
    pub damage: i32,          /* damage done by this bullettype */
    pub phases: i32,          /* how many phases in motion to show */
    pub phase_changes_per_second: f32, // how many different phases to display every second
    pub blast: i32,           /* which blast does this bullet create */
    pub surface_pointer: [*mut SDL_Surface; MAX_PHASES_IN_A_BULLET], // A pointer to the surfaces containing
                                                                     // the bullet images of this bullet
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bullet {
    pub pos: Finepoint,
    pub prev_pos: Finepoint, // use this for improved collision checks (for low FPS machines)
    pub speed: Finepoint,
    pub ty: u8,
    pub phase: u8,
    pub time_in_frames: i32, // how i64 does the bullet exist, measured in number of frames
    pub time_in_seconds: f32, // how i64 does the bullet exist in seconds
    pub mine: bool,
    pub owner: i32,
    pub angle: f32,
    pub surfaces_were_generated: i32,
    pub surface_pointer: [*mut SDL_Surface; MAX_PHASES_IN_A_BULLET],
}

impl Bullet {
    pub const fn default_const() -> Self {
        Bullet {
            pos: Finepoint::default_const(),
            prev_pos: Finepoint::default_const(),
            speed: Finepoint::default_const(),
            ty: 0,
            phase: 0,
            time_in_frames: 0,
            time_in_seconds: 0.,
            mine: false,
            owner: 0,
            angle: 0.,
            surfaces_were_generated: 0,
            surface_pointer: [null_mut(); MAX_PHASES_IN_A_BULLET],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlastSpec {
    pub phases: i32,
    pub picpointer: *mut u8,
    pub block: *mut SDL_Rect, /* the coordinates of the blocks in ne_blocks */
    pub total_animation_time: f32,
    pub surface_pointer: [*mut SDL_Surface; MAX_PHASES_IN_A_BULLET], // A pointer to the surfaces containing
                                                                     // the blast images of this blast type
}

impl BlastSpec {
    pub const fn default_const() -> Self {
        Self {
            phases: 0,
            picpointer: null_mut(),
            block: null_mut(),
            total_animation_time: 0.,
            surface_pointer: [null_mut(); MAX_PHASES_IN_A_BULLET],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Lift {
    pub level: i32, // The level, where this elevtor entrance is located
    pub x: i32,     // The position in x of this elevator entrance within the level
    pub y: i32,     // The position in y of this elevator entrance within the level

    /* connections: Numbers in Lift-Array */
    pub up: i32,
    pub down: i32,

    pub lift_row: i32, // which lift column does this lift entrance belong to?
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Waypoint {
    pub x: u8, /* Grob */
    pub y: u8,
    pub num_connections: i32,
    pub connections: [i32; MAX_WP_CONNECTIONS],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Level {
    pub empty: i32,
    pub timer: f32,
    pub levelnum: i32,      /* Number of this level */
    pub levelname: *mut i8, /* Name of this level */
    pub background_song_name: *mut i8,
    pub level_enter_comment: *mut i8,
    pub xlen: i32, /* X dimension */
    pub ylen: i32,
    pub color: i32,
    pub map: [*mut i8; MAX_MAP_ROWS], /* this is a vector of pointers ! */
    pub refreshes: [GrobPoint; MAX_REFRESHES_ON_LEVEL],
    pub doors: [GrobPoint; MAX_DOORS_ON_LEVEL],
    pub alerts: [GrobPoint; MAX_ALERTS_ON_LEVEL],
    pub num_waypoints: i32,
    pub all_waypoints: [Waypoint; MAXWAYPOINTS],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ship {
    pub num_levels: i32,
    pub num_lifts: i32,
    pub num_lift_rows: i32,
    pub area_name: [i8; 100],
    pub all_levels: [*mut Level; MAX_LEVELS],
    pub all_lifts: [Lift; MAX_LIFTS],
    pub lift_row_rect: [SDL_Rect; MAX_LIFT_ROWS], /* the lift-row rectangles */
    pub level_rects: [[SDL_Rect; MAX_LEVEL_RECTS]; MAX_LEVELS], /* level rectangles */
    pub num_level_rects: [i32; MAX_LEVELS],       /* how many rects has a level */
}

impl Default for Ship {
    fn default() -> Self {
        Self {
            num_levels: 0,
            num_lifts: 0,
            num_lift_rows: 0,
            area_name: [0; 100],
            all_levels: [null_mut(); MAX_LEVELS],
            all_lifts: [Lift {
                level: 0,
                x: 0,
                y: 0,
                up: 0,
                down: 0,
                lift_row: 0,
            }; MAX_LIFTS],
            lift_row_rect: [rect!(); MAX_LIFT_ROWS],
            level_rects: [[rect!(); MAX_LEVEL_RECTS]; MAX_LEVELS],
            num_level_rects: [0; MAX_LEVELS],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bar {
    pub pos: Point,
    pub len: i32,
    pub hgt: i32,
    pub oldval: i32,
    pub col: i32,
}
