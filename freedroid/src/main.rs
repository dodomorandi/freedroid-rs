#![feature(array_methods)]

mod array_c_string;
mod array_index;
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

use array_c_string::ArrayCString;
pub use array_index::ArrayIndex;
use array_init::array_init;
use b_font::{BFont, BFontInfo};
use bullet::BulletData;
use defs::{
    AlertNames, AssembleCombatWindowFlags, DisplayBannerFlags, Status, BYCOLOR,
    DROID_ROTATION_TIME, MAXBLASTS, MAXBULLETS, MAX_ENEMYS_ON_SHIP, MAX_LEVELS, RESET, SHOW_WAIT,
    STANDARD_MISSION,
};
use global::Global;
use graphics::Graphics;
use highscore::Highscore;
use influencer::Influencer;
use init::Init;
use input::Input;
use log::info;
use map::{ColorNames, Map};
use menu::Menu;
use misc::Misc;
use once_cell::unsync::OnceCell;
use qcell::{TCell, TCellOwner};
use sdl::Rect;
use ship::ShipData;
use sound::Sound;
use structs::{Blast, Bullet, Enemy, Level, Ship};
use takeover::Takeover;
use text::Text;
use vars::Vars;

use std::{cell::Cell, fs::File, ops::Not, os::raw::c_float, path::Path};

struct Main<'sdl> {
    last_got_into_blast_sound: c_float,
    last_refresh_sound: c_float,
    // Toggle TRUE/FALSE for turning sounds on/off
    sound_on: i32,
    // the current level data
    cur_level_index: Option<ArrayIndex<MAX_LEVELS>>,
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
    number_of_droid_types: u8,
    pre_take_energy: i32,
    all_bullets: [Bullet<'sdl>; MAXBULLETS + 10],
    all_blasts: [Blast; MAXBLASTS + 10],
    first_digit_rect: Rect,
    second_digit_rect: Rect,
    third_digit_rect: Rect,
    f_p_sover1: f32,
}

impl Default for Main<'_> {
    fn default() -> Self {
        Self {
            last_got_into_blast_sound: 2.,
            last_refresh_sound: 2.,
            sound_on: 1,
            cur_level_index: None,
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
            all_bullets: array_init(|_| Bullet::default_const()),
            all_blasts: [Blast::default(); MAXBLASTS + 10],
            first_digit_rect: Default::default(),
            second_digit_rect: Default::default(),
            third_digit_rect: Default::default(),
            f_p_sover1: 0.,
        }
    }
}

type Sdl = sdl::Sdl<sdl::Video, sdl::Timer, OnceCell<sdl::JoystickSystem>, OnceCell<sdl::Mixer>>;

pub struct FontCellMarker;
type FontCell<'sdl> = TCell<FontCellMarker, BFontInfo<'sdl>>;
type FontCellOwner = TCellOwner<FontCellMarker>;

struct Data<'sdl> {
    game_over: bool,
    sdl: &'sdl Sdl,
    map: Map,
    b_font: BFont<'sdl>,
    highscore: Highscore,
    bullet: BulletData,
    influencer: Influencer,
    init: Init,
    text: Text,
    sound: Option<Sound<'sdl>>,
    misc: Misc,
    ship: ShipData<'sdl>,
    input: Input,
    menu: Menu<'sdl>,
    global: Global<'sdl>,
    vars: Vars<'sdl>,
    takeover: Takeover<'sdl>,
    graphics: Graphics<'sdl>,
    main: Main<'sdl>,
    quit: Cell<bool>,
    font_owner: FontCellOwner,
}

impl<'sdl> Data<'sdl> {
    fn new(sdl: &'sdl Sdl) -> Self {
        Self {
            game_over: false,
            sdl,
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
            quit: Cell::new(false),
            font_owner: FontCellOwner::new(),
        }
    }
}

fn init_sdl() -> Sdl {
    let sdl = sdl::init().video().timer().build().unwrap_or_else(|| {
        // Safety: no other SDL function will be used -- we are panicking.
        unsafe {
            sdl::get_error(|err| {
                panic!("Couldn't initialize SDL: {}", err.to_string_lossy());
            })
        }
    });
    info!("SDL initialisation successful.");
    sdl
}

fn main() {
    env_logger::init();

    let sdl = init_sdl();
    let mut data = Data::new(&sdl);

    unsafe {
        data.input.joy_sensitivity = 1;

        data.init_freedroid(); // Initialisation of global variables and arrays

        sdl.cursor().hide();

        #[cfg(target_os = "windows")]
        {
            // spread the word :)
            win32_disclaimer();
        }

        while data.quit.get().not() {
            data.init_new_mission(STANDARD_MISSION);
            if data.quit.get() {
                break;
            }

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
                    .for_each(|rect| rect.scale(scale));

                for rect in &mut data.main.cur_ship.lift_row_rect
                    [0..usize::try_from(data.main.cur_ship.num_lift_rows).unwrap()]
                {
                    rect.scale(scale);
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
            let now = sdl.ticks_ms();
            while data.quit.get().not()
                && sdl.ticks_ms() - now < SHOW_WAIT
                && !data.fire_pressed_r()
            {
                data.show_droid_portrait(
                    data.vars.cons_droid_rect,
                    data.vars.me.ty,
                    DROID_ROTATION_TIME,
                    0,
                );
                sdl.delay_ms(1);
            }

            data.clear_graph_mem();
            data.display_banner(
                None,
                None,
                (DisplayBannerFlags::FORCE_UPDATE | DisplayBannerFlags::NO_SDL_UPDATE)
                    .bits()
                    .into(),
            );
            assert!(data.graphics.ne_screen.as_mut().unwrap().flip());

            data.game_over = false;

            data.graphics
                .crosshair_cursor
                .as_ref()
                .unwrap()
                .set_active(); // default cursor is a crosshair
            sdl.cursor().show();

            while data.quit.get().not() && data.game_over.not() {
                data.start_taking_time_for_fps_calculation();

                data.update_counters_for_this_frame();

                data.react_to_special_keys();

                if data.input.show_cursor {
                    sdl.cursor().show();
                } else {
                    sdl.cursor().hide();
                }

                data.move_level_doors();

                data.animate_refresh();

                data.explode_blasts(); // move blasts to the right current "phase" of the blast

                data.alert_level_warning(); // tout tout, blink blink... Alert!!

                data.display_banner(None, None, 0);

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
                let cur_level = data.main.cur_level_mut();
                if cur_level.empty == 0 {
                    data.set_time_factor(1.0);
                } else if cur_level.color == ColorNames::Dark as i32 {
                    // if level is already dark
                    data.set_time_factor(data.global.game_config.empty_level_speedup);
                } else if cur_level.timer <= 0. {
                    // time to switch off the lights ...
                    cur_level.color = ColorNames::Dark as i32;
                    data.switch_background_music_to(Some(BYCOLOR)); // start new background music
                }

                data.check_if_mission_is_complete();

                if data.global.game_config.hog_cpu == 0 {
                    // don't use up 100% CPU unless requested
                    sdl.delay_ms(1);
                }

                data.compute_fps_for_this_frame();
            }
        }

        info!("Termination of Freedroid initiated.");

        info!("Writing config file");
        data.save_game_config();
        info!("Writing highscores to disk");
        data.save_highscores();

        // ----- free memory
        data.free_graphics();
        data.sound = None;
        data.free_menu_data();
        data.free_game_mem();

        info!("Thank you for playing Freedroid.");
    }
}

impl Data<'_> {
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

        let mut timer = self.main.cur_level().timer;
        if timer >= 0.0 {
            timer -= self.frame_time();
            self.main.cur_level_mut().timer = timer;
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
        let cur_level = self.main.cur_level_mut();
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

#[inline]
fn find_subslice(data: &[u8], needle: &[u8]) -> Option<usize> {
    data.windows(needle.len()).position(|s| s == needle)
}

#[inline]
fn split_at_subslice<'a>(data: &'a [u8], needle: &[u8]) -> Option<(&'a [u8], &'a [u8])> {
    let pos = find_subslice(data, needle)?;
    let (before, after) = data.split_at(pos);
    Some((before, &after[needle.len()..]))
}

#[inline]
fn split_at_subslice_mut<'a>(
    data: &'a mut [u8],
    needle: &[u8],
) -> Option<(&'a mut [u8], &'a mut [u8])> {
    let pos = find_subslice(data, needle)?;
    let (before, after) = data.split_at_mut(pos);
    Some((before, &mut after[needle.len()..]))
}

/// This function read in a file with the specified name, allocated
/// memory for it of course, looks for the file end string and then
/// terminates the whole read in file with a 0 character, so that it
/// can easily be treated like a common string.
pub fn read_and_malloc_and_terminate_file(filename: &Path, file_end_string: &[u8]) -> Box<[u8]> {
    use bstr::ByteSlice;
    use std::io::Read;

    info!(
        "ReadAndMallocAndTerminateFile: The filename is: {}",
        filename.display()
    );

    // Read the whole theme data to memory
    let mut file = match File::open(filename) {
        Ok(file) => {
            info!("ReadAndMallocAndTerminateFile: Opening file succeeded...");
            file
        }
        Err(_) => {
            panic!(
                "\n\
        ----------------------------------------------------------------------\n\
        Freedroid has encountered a problem:\n\
        In function 'char* ReadAndMallocAndTerminateFile ( char* filename ):\n\
        \n\
        Freedroid was unable to open a given text file, that should be there and\n\
        should be accessible.\n\
        \n\
        This might be due to a wrong file name in a mission file, a wrong filename\n\
        in the source or a serious bug in the source.\n\
        \n\
        The file that couldn't be located was: {}\n\
        \n\
        Please check that your external text files are properly set up.\n\
        \n\
        Please also don't forget, that you might have to run 'make install'\n\
        again after you've made modifications to the data files in the source tree.\n\
        \n\
        Freedroid will terminate now to draw attention to the data problem it could\n\
        not resolve.... Sorry, if that interrupts a major game of yours.....\n\
        ----------------------------------------------------------------------\n\
        ",
                filename.display()
            );
        }
    };
    let file_len = match file
        .metadata()
        .ok()
        .and_then(|metadata| usize::try_from(metadata.len()).ok())
    {
        Some(file_len) => {
            info!("ReadAndMallocAndTerminateFile: fstating file succeeded...");
            file_len
        }
        None => {
            panic!("ReadAndMallocAndTerminateFile: Error fstat-ing File....");
        }
    };

    let mut all_data: Box<[u8]> = vec![0; file_len + 64 * 2 + 10000].into_boxed_slice();

    match file.read_exact(&mut all_data[..file_len]) {
        Ok(()) => info!("ReadAndMallocAndTerminateFile: Reading file succeeded..."),
        Err(_) => {
            panic!("ReadAndMallocAndTerminateFile: Reading file failed...");
        }
    }
    all_data[file_len..].fill(0);

    drop(file);

    info!("ReadAndMallocAndTerminateFile: Adding a 0 at the end of read data....");

    match all_data.find(file_end_string) {
        None => {
            panic!(
                "\n\
                ----------------------------------------------------------------------\n\
                Freedroid has encountered a problem:\n\
                In function 'char* ReadAndMallocAndTerminateFile ( char* filename ):\n\
                \n\
                Freedroid was unable to find the string, that should terminate the given\n\
                file within this file.\n\
                \n\
                This might be due to a corrupt text file on disk that does not confirm to\n\
                the file standards of this version of freedroid or (less likely) to a serious\n\
                bug in the reading function.\n\
                \n\
                The file that is concerned is: {}\n\
                The string, that could not be located was: {}\n\
                \n\
                Please check that your external text files are properly set up.\n\
                \n\
                Please also don't forget, that you might have to run 'make install'\n\
                again after you've made modifications to the data files in the source tree.\n\
                \n\
                Freedroid will terminate now to draw attention to the data problem it could\n\
                not resolve.... Sorry, if that interrupts a major game of yours.....\n\
                ----------------------------------------------------------------------\n\
                \n",
                filename.display(),
                String::from_utf8_lossy(file_end_string)
            );
        }
        Some(pos) => all_data[pos] = 0,
    }

    info!(
        "ReadAndMallocAndTerminateFile: The content of the read file: \n{}",
        String::from_utf8_lossy(all_data.split(|&c| c == b'\0').next().unwrap_or(b""))
    );

    all_data
}

impl Main<'_> {
    pub fn cur_level_mut(&mut self) -> &mut Level {
        cur_level!(mut self)
    }

    pub fn cur_level(&self) -> &Level {
        cur_level!(self)
    }
}

macro_rules! cur_level {
    (mut $main:expr) => {
        $main.cur_ship.all_levels[$main
            .cur_level_index
            .expect("no current level index available")]
        .as_mut()
        .expect("current level is None")
    };

    ($main:expr) => {
        $main.cur_ship.all_levels[$main
            .cur_level_index
            .expect("no current level index available")]
        .as_ref()
        .expect("current level is None")
    };
}
pub(crate) use cur_level;
