use crate::{array_c_string::ArrayCString, structs::Config, FontCell};

use std::{fmt, rc::Rc};

pub struct Global<'sdl> {
    pub menu_b_font: Option<Rc<FontCell<'sdl>>>,
    pub para_b_font: Option<Rc<FontCell<'sdl>>>,
    pub highscore_b_font: Option<Rc<FontCell<'sdl>>>,
    pub font0_b_font: Option<Rc<FontCell<'sdl>>>,
    pub font1_b_font: Option<Rc<FontCell<'sdl>>>,
    pub font2_b_font: Option<Rc<FontCell<'sdl>>>,
    pub skip_a_few_frames: bool,
    pub level_doors_not_moved_time: f32,
    pub droid_radius: f32,
    pub time_for_each_phase_of_door_movement: f32,
    pub blast_radius: f32,
    pub blast_damage_per_second: f32,
    pub current_combat_scale_factor: f32,
    pub collision_lose_energy_calibrator: f32,
    pub game_config: Config,
}

impl fmt::Debug for Global<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn debug_opt_fontcell(fontcell: &Option<Rc<FontCell<'_>>>) -> &'static str {
            match fontcell {
                Some(_) => "Some(Rc(FontCell))",
                None => "None",
            }
        }

        f.debug_struct("Global")
            .field("menu_b_font", &debug_opt_fontcell(&self.menu_b_font))
            .field("para_b_font", &debug_opt_fontcell(&self.para_b_font))
            .field(
                "highscore_b_font",
                &debug_opt_fontcell(&self.highscore_b_font),
            )
            .field("font0_b_font", &debug_opt_fontcell(&self.font0_b_font))
            .field("font1_b_font", &debug_opt_fontcell(&self.font1_b_font))
            .field("font2_b_font", &debug_opt_fontcell(&self.font2_b_font))
            .field("skip_a_few_frames", &self.skip_a_few_frames)
            .field(
                "level_doors_not_moved_time",
                &self.level_doors_not_moved_time,
            )
            .field("droid_radius", &self.droid_radius)
            .field(
                "time_for_each_phase_of_door_movement",
                &self.time_for_each_phase_of_door_movement,
            )
            .field("blast_radius", &self.blast_radius)
            .field("blast_damage_per_second", &self.blast_damage_per_second)
            .field(
                "current_combat_scale_factor",
                &self.current_combat_scale_factor,
            )
            .field(
                "collision_lose_energy_calibrator",
                &self.collision_lose_energy_calibrator,
            )
            .field("game_config", &self.game_config)
            .finish()
    }
}

impl Default for Global<'_> {
    fn default() -> Self {
        Self {
            menu_b_font: Option::default(),
            para_b_font: Option::default(),
            highscore_b_font: Option::default(),
            font0_b_font: Option::default(),
            font1_b_font: Option::default(),
            font2_b_font: Option::default(),
            skip_a_few_frames: false,
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
                theme_name: ArrayCString::default(),
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
