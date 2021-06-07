use crate::{b_font::BFontInfo, structs::Config};

use cstr::cstr;
use std::{ffi::CStr, ptr::null_mut};

pub const INFLUENCE_MODE_NAMES: [&CStr; 17] = [
    cstr!("Mobile"),
    cstr!("Transfer"),
    cstr!("Weapon"),
    cstr!("Captured"),
    cstr!("Complete"),
    cstr!("Rejected"),
    cstr!("Logged In"),
    cstr!("Debriefing"),
    cstr!("Terminated"),
    cstr!("Pause"),
    cstr!("Cheese"),
    cstr!("Elevator"),
    cstr!("Briefing"),
    cstr!("Menu"),
    cstr!("Victory"),
    cstr!("Activate"),
    cstr!("-- OUT --"),
];

#[derive(Debug)]
pub struct Global {
    pub menu_b_font: *mut BFontInfo,
    pub para_b_font: *mut BFontInfo,
    pub highscore_b_font: *mut BFontInfo,
    pub font0_b_font: *mut BFontInfo,
    pub font1_b_font: *mut BFontInfo,
    pub font2_b_font: *mut BFontInfo,
    pub skip_a_few_frames: i32,
    pub level_doors_not_moved_time: f32,
    pub droid_radius: f32,
    pub time_for_each_phase_of_door_movement: f32,
    pub blast_radius: f32,
    pub blast_damage_per_second: f32,
    pub current_combat_scale_factor: f32,
    pub collision_lose_energy_calibrator: f32,
    pub game_config: Config,
}

impl Default for Global {
    fn default() -> Self {
        Self {
            menu_b_font: null_mut(),
            para_b_font: null_mut(),
            highscore_b_font: null_mut(),
            font0_b_font: null_mut(),
            font1_b_font: null_mut(),
            font2_b_font: null_mut(),
            skip_a_few_frames: 0,
            level_doors_not_moved_time: 0.,
            droid_radius: 0.,
            time_for_each_phase_of_door_movement: 0.,
            blast_radius: 0.,
            blast_damage_per_second: 0.,
            current_combat_scale_factor: 0.,
            collision_lose_energy_calibrator: 0.,
            game_config: Config {
                wanted_text_visible_time: 0.,
                draw_framerate: 0,
                draw_energy: 0,
                draw_position: 0,
                draw_death_count: 0,
                droid_talk: 0,
                current_bg_music_volume: 0.,
                current_sound_fx_volume: 0.,
                current_gamma_correction: 0.,
                theme_name: [0; 100],
                full_user_rect: 0,
                use_fullscreen: 0,
                takeover_activates: 0,
                fire_hold_takeover: 0,
                show_decals: 0,
                all_map_visible: 0,
                scale: 0.,
                hog_cpu: 0,
                empty_level_speedup: 0.,
            },
        }
    }
}

pub static mut MENU_B_FONT: *mut BFontInfo = null_mut();
pub static mut PARA_B_FONT: *mut BFontInfo = null_mut();
pub static mut HIGHSCORE_B_FONT: *mut BFontInfo = null_mut();
pub static mut FONT0_B_FONT: *mut BFontInfo = null_mut();
pub static mut FONT1_B_FONT: *mut BFontInfo = null_mut();
pub static mut FONT2_B_FONT: *mut BFontInfo = null_mut();
pub static mut SKIP_A_FEW_FRAMES: i32 = 0;
pub static mut LEVEL_DOORS_NOT_MOVED_TIME: f32 = 0.;
pub static mut DROID_RADIUS: f32 = 0.;
pub static mut TIME_FOR_EACH_PHASE_OF_DOOR_MOVEMENT: f32 = 0.;
pub static mut BLAST_RADIUS: f32 = 0.;
pub static mut BLAST_DAMAGE_PER_SECOND: f32 = 0.;
pub static mut CURRENT_COMBAT_SCALE_FACTOR: f32 = 0.;
pub static mut COLLISION_LOSE_ENERGY_CALIBRATOR: f32 = 0.;
