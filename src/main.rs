#![feature(array_map)]
#![feature(array_methods)]

macro_rules! rect {
    () => {
        rect!(0, 0, 0, 0)
    };
    ($x:expr, $y:expr, $w:expr, $h:expr $(,)?) => {
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
    DROID_ROTATION_TIME, MAXBLASTS, MAXBULLETS, MAX_ENEMYS_ON_SHIP, RESET, SHOW_WAIT,
    STANDARD_MISSION_C,
};
use global::Global;
use graphics::Graphics;
use highscore::Highscore;
use influencer::Influencer;
use init::Init;
use input::{Input, SDL_Delay};
use map::{ColorNames, Map};
use menu::Menu;
use misc::Misc;
use ship::ShipData;
use sound::Sound;
use structs::{Blast, Bullet, Enemy, Level, Ship};
use takeover::Takeover;
use text::Text;
use vars::Vars;

use sdl::{
    mouse::ll::{SDL_SetCursor, SDL_ShowCursor, SDL_DISABLE, SDL_ENABLE},
    sdl::ll::SDL_GetTicks,
    video::ll::{SDL_Flip, SDL_Surface},
    Rect,
};
use std::{
    convert::TryFrom,
    fmt,
    ops::Not,
    os::raw::{c_char, c_float},
    ptr::null_mut,
};

struct Main {
    last_got_into_blast_sound: c_float,
    last_refresh_sound: c_float,
    // Toggle TRUE/FALSE for turning sounds on/off
    sound_on: i32,
    // the current level data
    cur_level: *mut Level,
    // the current ship-data
    cur_ship: Ship,
    show_score: i64,
    real_score: f32,
    // a cumulative/draining counter of kills->determines Alert!
    death_count: f32,
    // drain per second
    death_count_drain_speed: f32,
    alert_level: i32,
    // threshold for FIRST Alert-color (yellow), the others are 2*, 3*..
    alert_threshold: i32,
    // bonus/sec for FIRST Alert-color, the others are 2*, 3*,...
    alert_bonus_per_sec: f32,
    all_enemys: [Enemy; MAX_ENEMYS_ON_SHIP],
    config_dir: [i8; 255],
    invincible_mode: i32,
    /* display enemys regardless of IsVisible() */
    show_all_droids: i32,
    /* for bullet debugging: stop where u are */
    stop_influencer: i32,
    num_enemys: i32,
    number_of_droid_types: i32,
    pre_take_energy: i32,
    all_bullets: [Bullet; MAXBULLETS + 10],
    all_blasts: [Blast; MAXBLASTS + 10],
    first_digit_rect: Rect,
    second_digit_rect: Rect,
    third_digit_rect: Rect,
    f_p_sover1: f32,
}

impl Default for Main {
    fn default() -> Self {
        Self {
            last_got_into_blast_sound: 2.,
            last_refresh_sound: 2.,
            sound_on: 1,
            cur_level: null_mut(),
            cur_ship: Ship::default(),
            show_score: 0,
            real_score: 0.,
            death_count: 0.,
            death_count_drain_speed: 0.,
            alert_level: 0,
            alert_threshold: 0,
            alert_bonus_per_sec: 0.,
            all_enemys: [Enemy::default(); MAX_ENEMYS_ON_SHIP],
            config_dir: [0; 255],
            invincible_mode: 0,
            show_all_droids: 0,
            stop_influencer: 0,
            num_enemys: 0,
            number_of_droid_types: 0,
            pre_take_energy: 0,
            all_bullets: [Bullet::default_const(); MAXBULLETS + 10],
            all_blasts: [Blast::default(); MAXBLASTS + 10],
            first_digit_rect: rect!(),
            second_digit_rect: rect!(),
            third_digit_rect: rect!(),
            f_p_sover1: 0.,
        }
    }
}

impl fmt::Debug for Main {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[derive(Debug)]
        struct Rect {
            x: i16,
            y: i16,
            w: u16,
            h: u16,
        }

        impl From<&::sdl::Rect> for Rect {
            fn from(rect: &::sdl::Rect) -> Rect {
                Rect {
                    x: rect.x,
                    y: rect.y,
                    w: rect.w,
                    h: rect.h,
                }
            }
        }

        let first_digit_rect = Rect::from(&self.first_digit_rect);
        let second_digit_rect = Rect::from(&self.second_digit_rect);
        let third_digit_rect = Rect::from(&self.third_digit_rect);

        f.debug_struct("Main")
            .field("last_got_into_blast_sound", &self.last_got_into_blast_sound)
            .field("last_refresh_sound", &self.last_refresh_sound)
            .field("sound_on", &self.sound_on)
            .field("cur_level", &self.cur_level)
            .field("cur_ship", &self.cur_ship)
            .field("show_score", &self.show_score)
            .field("real_score", &self.real_score)
            .field("death_count", &self.death_count)
            .field("death_count_drain_speed", &self.death_count_drain_speed)
            .field("alert_level", &self.alert_level)
            .field("alert_threshold", &self.alert_threshold)
            .field("alert_bonus_per_sec", &self.alert_bonus_per_sec)
            .field("all_enemys", &self.all_enemys)
            .field("config_dir", &self.config_dir)
            .field("invincible_mode", &self.invincible_mode)
            .field("show_all_droids", &self.show_all_droids)
            .field("stop_influencer", &self.stop_influencer)
            .field("num_enemys", &self.num_enemys)
            .field("number_of_droid_types", &self.number_of_droid_types)
            .field("pre_take_energy", &self.pre_take_energy)
            .field("all_bullets", &self.all_bullets)
            .field("all_blasts", &self.all_blasts)
            .field("first_digit_rect", &first_digit_rect)
            .field("second_digit_rect", &second_digit_rect)
            .field("third_digit_rect", &third_digit_rect)
            .field("f_p_sover1", &self.f_p_sover1)
            .finish()
    }
}

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
    input: Input,
    menu: Menu,
    global: Global,
    vars: Vars,
    takeover: Takeover,
    graphics: Graphics,
    main: Main,
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
            input: Default::default(),
            menu: Default::default(),
            global: Default::default(),
            vars: Default::default(),
            takeover: Default::default(),
            graphics: Default::default(),
            main: Default::default(),
        }
    }
}

fn main() {
    env_logger::init();

    let mut data = Data::default();

    unsafe {
        data.input.joy_sensitivity = 1;

        data.init_keystr();

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
            let scale = data.global.game_config.scale;
            #[allow(clippy::float_cmp)]
            if scale != 1.0 {
                data.main.cur_ship.level_rects
                    [0..usize::try_from(data.main.cur_ship.num_levels).unwrap()]
                    .iter_mut()
                    .zip(data.main.cur_ship.num_level_rects.iter())
                    .flat_map(|(rects, &num_rects)| {
                        rects[0..usize::try_from(num_rects).unwrap()].iter_mut()
                    })
                    .for_each(|rect| scale_rect(rect, scale));

                for rect in &mut data.main.cur_ship.lift_row_rect
                    [0..usize::try_from(data.main.cur_ship.num_lift_rows).unwrap()]
                {
                    scale_rect(rect, scale);
                }
            }

            // release all keys
            data.wait_for_all_keys_released();

            data.show_droid_info(data.vars.me.ty, -3, 0); // show unit-intro page
            data.show_droid_portrait(
                data.vars.cons_droid_rect,
                data.vars.me.ty,
                DROID_ROTATION_TIME,
                RESET,
            );
            let now = SDL_GetTicks();
            while SDL_GetTicks() - now < SHOW_WAIT && !data.fire_pressed_r() {
                data.show_droid_portrait(
                    data.vars.cons_droid_rect,
                    data.vars.me.ty,
                    DROID_ROTATION_TIME,
                    0,
                );
                SDL_Delay(1);
            }

            data.clear_graph_mem();
            data.display_banner(
                null_mut(),
                null_mut(),
                (DisplayBannerFlags::FORCE_UPDATE | DisplayBannerFlags::NO_SDL_UPDATE)
                    .bits()
                    .into(),
            );
            SDL_Flip(data.graphics.ne_screen);

            SDL_SetCursor(data.graphics.crosshair_cursor); // default cursor is a crosshair
            SDL_ShowCursor(SDL_ENABLE);

            while data.game_over.not() {
                data.start_taking_time_for_fps_calculation();

                data.update_counters_for_this_frame();

                data.react_to_special_keys();

                if data.input.show_cursor {
                    SDL_ShowCursor(SDL_ENABLE);
                } else {
                    SDL_ShowCursor(SDL_DISABLE);
                }

                data.move_level_doors();

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
                if (*data.main.cur_level).empty == 0 {
                    data.set_time_factor(1.0);
                } else if (*data.main.cur_level).color == ColorNames::Dark as i32 {
                    // if level is already dark
                    data.set_time_factor(data.global.game_config.empty_level_speedup);
                } else if (*data.main.cur_level).timer <= 0. {
                    // time to switch off the lights ...
                    (*data.main.cur_level).color = ColorNames::Dark as i32;
                    data.switch_background_music_to(BYCOLOR.as_ptr()); // start new background music
                }

                data.check_if_mission_is_complete();

                if data.global.game_config.hog_cpu == 0 {
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
        self.main.last_got_into_blast_sound += self.frame_time();
        self.main.last_refresh_sound += self.frame_time();
        self.vars.me.last_crysound_time += self.frame_time();
        self.vars.me.timer += self.frame_time();

        let cur_level = &mut *self.main.cur_level;
        if cur_level.timer >= 0.0 {
            cur_level.timer -= self.frame_time();
        }

        self.vars.me.last_transfer_sound_time += self.frame_time();
        self.vars.me.text_visible_time += self.frame_time();
        self.global.level_doors_not_moved_time += self.frame_time();
        if self.global.skip_a_few_frames != 0 {
            self.global.skip_a_few_frames = 0;
        }

        if self.vars.me.firewait > 0. {
            self.vars.me.firewait -= self.frame_time();
            if self.vars.me.firewait < 0. {
                self.vars.me.firewait = 0.;
            }
        }
        if self.vars.ship_empty_counter > 1 {
            self.vars.ship_empty_counter -= 1;
        }
        if cur_level.empty > 2 {
            cur_level.empty -= 1;
        }
        if self.main.real_score > self.main.show_score as f32 {
            self.main.show_score += 1;
        }
        if self.main.real_score < self.main.show_score as f32 {
            self.main.show_score -= 1;
        }

        // drain Death-count, responsible for Alert-state
        if self.main.death_count > 0. {
            self.main.death_count -= self.main.death_count_drain_speed * self.frame_time();
        }
        if self.main.death_count < 0. {
            self.main.death_count = 0.;
        }
        // and switch Alert-level according to DeathCount
        self.main.alert_level = (self.main.death_count / self.main.alert_threshold as f32) as i32;
        if self.main.alert_level > AlertNames::Red as i32 {
            self.main.alert_level = AlertNames::Red as i32;
        }
        // player gets a bonus/second in AlertLevel
        self.main.real_score +=
            self.main.alert_level as f32 * self.main.alert_bonus_per_sec * self.frame_time();

        let Self {
            main, misc, global, ..
        } = self;
        for enemy in &mut main.all_enemys[..usize::try_from(main.num_enemys).unwrap()] {
            if enemy.status == Status::Out as i32 {
                continue;
            }

            if enemy.warten > 0. {
                enemy.warten -= misc.frame_time(global, main.f_p_sover1);
                if enemy.warten < 0. {
                    enemy.warten = 0.;
                }
            }

            if enemy.firewait > 0. {
                enemy.firewait -= misc.frame_time(global, main.f_p_sover1);
                if enemy.firewait <= 0. {
                    enemy.firewait = 0.;
                }
            }

            enemy.text_visible_time += misc.frame_time(global, main.f_p_sover1);
        }
    }
}
