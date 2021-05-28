macro_rules! rect {
    ($x:expr, $y:expr, $w:expr, $h:expr) => {
        ::sdl::Rect {
            x: $x,
            y: $y,
            w: $w,
            h: $h,
        }
    };
}

mod b_font;
mod bullet;
mod defs;
mod enemy;
mod global;
mod graphics;
mod highscore;
mod influencer;
mod init;
mod input;
mod level_editor;
mod map;
mod menu;
mod misc;
mod ship;
mod sound;
mod structs;
mod takeover;
mod text;
mod vars;
mod view;

use b_font::BFont;
use bullet::BulletData;
use defs::{
    scale_rect, AlertNames, AssembleCombatWindowFlags, DisplayBannerFlags, Status, BYCOLOR,
    DROID_ROTATION_TIME, MAXBLASTS, MAXBULLETS, MAX_ENEMYS_ON_SHIP, MAX_LEVELS, MAX_LEVEL_RECTS,
    MAX_LIFTS, MAX_LIFT_ROWS, RESET, SHOW_WAIT, STANDARD_MISSION_C,
};
use global::{GAME_CONFIG, LEVEL_DOORS_NOT_MOVED_TIME, SKIP_A_FEW_FRAMES};
use graphics::{clear_graph_mem, CROSSHAIR_CURSOR, NE_SCREEN};
use highscore::Highscore;
use influencer::Influencer;
use init::Init;
use input::{init_keystr, SDL_Delay, JOY_SENSITIVITY, SHOW_CURSOR};
use map::{move_level_doors, ColorNames, Map};
use misc::Misc;
use ship::ShipData;
use sound::Sound;
use structs::{Blast, Bullet, Enemy, Finepoint, Level, Lift, Ship};
use text::Text;
use vars::{CONS_DROID_RECT, ME, SHIP_EMPTY_COUNTER};

use sdl::{
    mouse::ll::{SDL_SetCursor, SDL_ShowCursor, SDL_DISABLE, SDL_ENABLE},
    sdl::ll::SDL_GetTicks,
    video::ll::{SDL_Flip, SDL_Surface},
    Rect,
};
use std::{
    convert::TryFrom,
    ops::Not,
    os::raw::{c_char, c_float},
    ptr::null_mut,
};

const RECT_ZERO: Rect = Rect {
    x: 0,
    y: 0,
    h: 0,
    w: 0,
};

static mut LAST_GOT_INTO_BLAST_SOUND: c_float = 2.;
static mut LAST_REFRESH_SOUND: c_float = 2.;
static mut CUR_LEVEL: *mut Level = null_mut(); /* the current level data */
static mut CUR_SHIP: Ship = Ship {
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
    lift_row_rect: [RECT_ZERO; MAX_LIFT_ROWS], /* the lift-row rectangles */
    level_rects: [[RECT_ZERO; MAX_LEVEL_RECTS]; MAX_LEVELS], /* level rectangles */
    num_level_rects: [0; MAX_LEVELS],          /* how many rects has a level */
}; /* the current ship-data */

static mut DEBUG_LEVEL: i32 = 0; /* 0=no debug 1=some debug messages 2=...etc */
static mut SOUND_ON: i32 = 1; /* Toggle TRUE/FALSE for turning sounds on/off */
static mut THIS_MESSAGE_TIME: i32 = 0;
static mut SHOW_SCORE: i64 = 0;
static mut REAL_SCORE: f32 = 0.;
static mut DEATH_COUNT: f32 = 0.; // a cumulative/draining counter of kills->determines Alert!
static mut DEATH_COUNT_DRAIN_SPEED: f32 = 0.; // drain per second
static mut ALERT_LEVEL: i32 = 0;
static mut ALERT_THRESHOLD: i32 = 0; // threshold for FIRST Alert-color (yellow), the others are 2*, 3*..
static mut ALERT_BONUS_PER_SEC: f32 = 0.; // bonus/sec for FIRST Alert-color, the others are 2*, 3*,...
static mut ALL_ENEMYS: [Enemy; MAX_ENEMYS_ON_SHIP] = [Enemy {
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
}; MAX_ENEMYS_ON_SHIP];

static mut CONFIG_DIR: [i8; 255] = [0; 255];
static mut INVINCIBLE_MODE: i32 = 0;
static mut SHOW_ALL_DROIDS: i32 = 0; /* display enemys regardless of IsVisible() */
static mut STOP_INFLUENCER: i32 = 0; /* for bullet debugging: stop where u are */
static mut NUM_ENEMYS: i32 = 0;
static mut NUMBER_OF_DROID_TYPES: i32 = 0;
static mut PRE_TAKE_ENERGY: i32 = 0;
static mut ALL_BULLETS: [Bullet; MAXBULLETS + 10] = [Bullet::default_const(); MAXBULLETS + 10];
static mut ALL_BLASTS: [Blast; MAXBLASTS + 10] = [Blast {
    px: 0.,
    py: 0.,
    ty: 0,
    phase: 0.,
    message_was_done: 0,
    mine: false,
}; MAXBLASTS + 10];

static mut FIRST_DIGIT_RECT: Rect = RECT_ZERO;
static mut SECOND_DIGIT_RECT: Rect = RECT_ZERO;
static mut THIRD_DIGIT_RECT: Rect = RECT_ZERO;
static mut F_P_SOVER1: f32 = 0.;

#[derive(Debug)]
struct Data {
    game_over: bool,
    map: Map,
    b_font: BFont,
    highscore: Highscore,
    bullet: BulletData,
    influencer: Influencer,
    init: Init,
    text: Text,
    sound: Sound,
    misc: Misc,
    ship: ShipData,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            game_over: false,
            map: Default::default(),
            b_font: Default::default(),
            highscore: Default::default(),
            bullet: Default::default(),
            influencer: Default::default(),
            init: Default::default(),
            text: Default::default(),
            sound: Default::default(),
            misc: Default::default(),
            ship: Default::default(),
        }
    }
}

fn main() {
    env_logger::init();

    let mut data = Data::default();

    unsafe {
        JOY_SENSITIVITY = 1;

        init_keystr();

        data.init_freedroid(); // Initialisation of global variables and arrays

        SDL_ShowCursor(SDL_DISABLE);

        #[cfg(target_os = "windows")]
        {
            // spread the word :)
            win32_disclaimer();
        }

        loop {
            data.init_new_mission(STANDARD_MISSION_C.as_ptr() as *mut c_char);

            // scale Level-pic rects
            let scale = GAME_CONFIG.scale;
            #[allow(clippy::clippy::float_cmp)]
            if scale != 1.0 {
                CUR_SHIP.level_rects[0..usize::try_from(CUR_SHIP.num_levels).unwrap()]
                    .iter_mut()
                    .zip(CUR_SHIP.num_level_rects.iter())
                    .flat_map(|(rects, &num_rects)| {
                        rects[0..usize::try_from(num_rects).unwrap()].iter_mut()
                    })
                    .for_each(|rect| scale_rect(rect, scale));

                for rect in
                    &mut CUR_SHIP.lift_row_rect[0..usize::try_from(CUR_SHIP.num_lift_rows).unwrap()]
                {
                    scale_rect(rect, scale);
                }
            }

            // release all keys
            data.wait_for_all_keys_released();

            data.show_droid_info(ME.ty, -3, 0); // show unit-intro page
            data.show_droid_portrait(CONS_DROID_RECT, ME.ty, DROID_ROTATION_TIME, RESET);
            let now = SDL_GetTicks();
            while SDL_GetTicks() - now < SHOW_WAIT && !data.fire_pressed_r() {
                data.show_droid_portrait(CONS_DROID_RECT, ME.ty, DROID_ROTATION_TIME, 0);
                SDL_Delay(1);
            }

            clear_graph_mem();
            data.display_banner(
                null_mut(),
                null_mut(),
                (DisplayBannerFlags::FORCE_UPDATE | DisplayBannerFlags::NO_SDL_UPDATE)
                    .bits()
                    .into(),
            );
            SDL_Flip(NE_SCREEN);

            SDL_SetCursor(CROSSHAIR_CURSOR); // default cursor is a crosshair
            SDL_ShowCursor(SDL_ENABLE);

            while data.game_over.not() {
                data.start_taking_time_for_fps_calculation();

                data.update_counters_for_this_frame();

                data.react_to_special_keys();

                if SHOW_CURSOR {
                    SDL_ShowCursor(SDL_ENABLE);
                } else {
                    SDL_ShowCursor(SDL_DISABLE);
                }

                move_level_doors();

                data.animate_refresh();

                data.explode_blasts(); // move blasts to the right current "phase" of the blast

                data.alert_level_warning(); // tout tout, blink blink... Alert!!

                data.display_banner(null_mut(), null_mut(), 0);

                data.move_bullets(); // leave this in front of graphics output: time_in_frames should start with 1

                data.assemble_combat_picture(
                    AssembleCombatWindowFlags::DO_SCREEN_UPDATE.bits().into(),
                );

                for bullet in 0..i32::try_from(MAXBULLETS).unwrap() {
                    data.check_bullet_collisions(bullet);
                }

                // change Influ-speed depending on keys pressed, but
                // also change his status and position and "phase" of rotation
                data.move_influence();

                data.move_enemys(); // move all the enemys:
                                    // also do attacks on influ and also move "phase" or their rotation

                data.check_influence_wall_collisions(); /* Testen ob der Weg nicht durch Mauern verstellt ist */
                data.check_influence_enemy_collision();

                // control speed of time-flow: dark-levels=emptyLevelSpeedup, normal-levels=1.0
                if (*CUR_LEVEL).empty == 0 {
                    data.set_time_factor(1.0);
                } else if (*CUR_LEVEL).color == ColorNames::Dark as i32 {
                    // if level is already dark
                    data.set_time_factor(GAME_CONFIG.empty_level_speedup);
                } else if (*CUR_LEVEL).timer <= 0. {
                    // time to switch off the lights ...
                    (*CUR_LEVEL).color = ColorNames::Dark as i32;
                    data.switch_background_music_to(BYCOLOR.as_ptr()); // start new background music
                }

                data.check_if_mission_is_complete();

                if GAME_CONFIG.hog_cpu == 0 {
                    // don't use up 100% CPU unless requested
                    SDL_Delay(1);
                }

                data.compute_fps_for_this_frame();
            }
        }
    }
}

#[inline]
fn sdl_must_lock(surface: &SDL_Surface) -> bool {
    use sdl::video::SurfaceFlag::*;
    surface.offset != 0
        && (surface.flags & (HWSurface as u32 | AsyncBlit as u32 | RLEAccel as u32)) != 0
}

impl Data {
    /// This function updates counters and is called ONCE every frame.
    /// The counters include timers, but framerate-independence of game speed
    /// is preserved because everything is weighted with the Frame_Time()
    /// function.
    unsafe fn update_counters_for_this_frame(&mut self) {
        // Here are some things, that were previously done by some periodic */
        // interrupt function
        THIS_MESSAGE_TIME += 1;

        LAST_GOT_INTO_BLAST_SOUND += self.frame_time();
        LAST_REFRESH_SOUND += self.frame_time();
        ME.last_crysound_time += self.frame_time();
        ME.timer += self.frame_time();

        let cur_level = &mut *CUR_LEVEL;
        if cur_level.timer >= 0.0 {
            cur_level.timer -= self.frame_time();
        }

        ME.last_transfer_sound_time += self.frame_time();
        ME.text_visible_time += self.frame_time();
        LEVEL_DOORS_NOT_MOVED_TIME += self.frame_time();
        if SKIP_A_FEW_FRAMES != 0 {
            SKIP_A_FEW_FRAMES = 0;
        }

        if ME.firewait > 0. {
            ME.firewait -= self.frame_time();
            if ME.firewait < 0. {
                ME.firewait = 0.;
            }
        }
        if SHIP_EMPTY_COUNTER > 1 {
            SHIP_EMPTY_COUNTER -= 1;
        }
        if cur_level.empty > 2 {
            cur_level.empty -= 1;
        }
        if REAL_SCORE > SHOW_SCORE as f32 {
            SHOW_SCORE += 1;
        }
        if REAL_SCORE < SHOW_SCORE as f32 {
            SHOW_SCORE -= 1;
        }

        // drain Death-count, responsible for Alert-state
        if DEATH_COUNT > 0. {
            DEATH_COUNT -= DEATH_COUNT_DRAIN_SPEED * self.frame_time();
        }
        if DEATH_COUNT < 0. {
            DEATH_COUNT = 0.;
        }
        // and switch Alert-level according to DeathCount
        ALERT_LEVEL = (DEATH_COUNT / ALERT_THRESHOLD as f32) as i32;
        if ALERT_LEVEL > AlertNames::Red as i32 {
            ALERT_LEVEL = AlertNames::Red as i32;
        }
        // player gets a bonus/second in AlertLevel
        REAL_SCORE += ALERT_LEVEL as f32 * ALERT_BONUS_PER_SEC * self.frame_time();

        for enemy in &mut ALL_ENEMYS {
            if enemy.status == Status::Out as i32 {
                continue;
            }

            if enemy.warten > 0. {
                enemy.warten -= self.frame_time();
                if enemy.warten < 0. {
                    enemy.warten = 0.;
                }
            }

            if enemy.firewait > 0. {
                enemy.firewait -= self.frame_time();
                if enemy.firewait <= 0. {
                    enemy.firewait = 0.;
                }
            }

            enemy.text_visible_time += self.frame_time();
        }
    }
}
