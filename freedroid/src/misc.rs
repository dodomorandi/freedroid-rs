use crate::{
    defs::{
        self, AssembleCombatWindowFlags, Cmds, Criticality, Status, Themed, FD_DATADIR,
        GRAPHICS_DIR_C, LOCAL_DATADIR, MAXBLASTS, PROGRESS_FILLER_FILE, PROGRESS_METER_FILE,
    },
    graphics::{scale_pic, Graphics},
    input::CMD_STRINGS,
    ArrayCString, ArrayIndex, Data, Global,
};

use bstr::{BStr, ByteSlice};
use defs::MAXBULLETS;
use log::{error, info, trace, warn};
use nom::{Finish, Parser};
use rand::{thread_rng, Rng};
use sdl::Rect;
use std::{
    borrow::Cow,
    env,
    ffi::{CStr, CString},
    fs::{self, File},
    os::raw::{c_float, c_int, c_long},
    path::Path,
};

#[derive(Debug)]
pub struct Misc {
    previous_time: f32,
    current_time_factor: f32,
    file_path: ArrayCString<1024>,
    frame_nr: c_int,
    one_frame_sdl_ticks: u32,
    now_sdl_ticks: u32,
    one_frame_delay: c_long,
}

impl Default for Misc {
    fn default() -> Self {
        Self {
            previous_time: 0.1,
            current_time_factor: 1.,
            file_path: Default::default(),
            frame_nr: 0,
            one_frame_sdl_ticks: 0,
            now_sdl_ticks: 0,
            one_frame_delay: 0,
        }
    }
}

/// This function is used to generate a random integer in the range
/// from [0 to upper_bound] (inclusive), distributed uniformly.
pub fn my_random(upper_bound: c_int) -> c_int {
    thread_rng().gen_range(0..=upper_bound)
}

const VERSION_STRING: &str = "Freedroid Version";
const DRAW_FRAMERATE: &str = "Draw_Framerate";
const DRAW_ENERGY: &str = "Draw_Energy";
const DRAW_POSITION: &str = "Draw_Position";
const DRAW_DEATHCOUNT: &str = "Draw_DeathCount";
const DROID_TALK: &str = "Droid_Talk";
const WANTED_TEXT_VISIBLE_TIME: &str = "WantedTextVisibleTime";
const CURRENT_BG_MUSIC_VOLUME: &str = "Current_BG_Music_Volume";
const CURRENT_SOUND_FX_VOLUME: &str = "Current_Sound_FX_Volume";
const CURRENT_GAMMA_CORRECTION: &str = "Current_Gamma_Correction";
const THEME_NAME: &str = "Theme_Name";
const FULL_USER_RECT: &str = "FullUserRect";
const USE_FULLSCREEN: &str = "UseFullscreen";
const TAKEOVER_ACTIVATES: &str = "TakeoverActivates";
const FIRE_HOLD_TAKEOVER: &str = "FireHoldTakeover";
const SHOW_DECALS: &str = "ShowDecals";
const ALL_MAP_VISIBLE: &str = "AllMapVisible";
const VID_SCALE_FACTOR: &str = "Vid_ScaleFactor";
const HOG_CPU: &str = "Hog_Cpu";
const EMPTY_LEVEL_SPEEDUP: &str = "EmptyLevelSpeedup";

pub fn read_float_from_string(data: &[u8], label: &[u8]) -> f32 {
    use nom::{character::complete::space0, number::complete::float};

    let pos = locate_string_in_data(data, label) + label.len();
    let data = &data[pos..];

    let (_, (_, out)) = space0
        .and(float)
        .parse(data)
        .finish()
        .unwrap_or_else(|_: ()| {
            panic!(
                "ReadValueFromString(): could not read float {} of label {}",
                <&BStr>::from(data),
                <&BStr>::from(label),
            );
        });
    out
}

pub fn read_i32_from_string(data: &[u8], label: &[u8]) -> i32 {
    use nom::character::complete::{i32, space0};

    let pos = locate_string_in_data(data, label) + label.len();
    let data = &data[pos..];

    let (_, (_, out)) = space0
        .and(i32)
        .parse(data)
        .finish()
        .unwrap_or_else(|_: ()| {
            panic!(
                "could not read float {} of label {}",
                <&BStr>::from(data),
                <&BStr>::from(label),
            );
        });
    out
}

pub fn read_i16_from_string(data: &[u8], label: &[u8]) -> i16 {
    use nom::character::complete::{i16, space0};

    let pos = locate_string_in_data(data, label) + label.len();
    let data = &data[pos..];

    let (_, (_, out)) = space0
        .and(i16)
        .parse(data)
        .finish()
        .unwrap_or_else(|_: ()| {
            panic!(
                "could not read float {} of label {}",
                <&BStr>::from(data),
                <&BStr>::from(label),
            );
        });
    out
}

pub fn read_string_from_string<'a>(data: &'a [u8], label: &[u8]) -> &'a [u8] {
    let pos = locate_string_in_data(data, label) + label.len();
    let data = &data[pos..];
    data.iter()
        .position(|c| c.is_ascii_whitespace())
        .map(|pos| &data[..pos])
        .unwrap_or(data)
}

/// This function tries to locate a string in some given data string.
/// The data string is assumed to be null terminated.  Otherwise SEGFAULTS
/// might happen.
///
/// The return value is a pointer to the first instance where the substring
/// we are searching is found in the main text.
pub fn locate_string_in_data(haystack: &[u8], needle: &[u8]) -> usize {
    let pos = haystack
        .windows(needle.len())
        .position(|s| s == needle)
        .unwrap_or_else(|| {
            panic!(
                "\n\
             \n\
             ----------------------------------------------------------------------\n\
             Freedroid has encountered a problem:\n\
             In function 'char* LocateStringInData ( char* SearchBeginPointer, char* \
             SearchTextPointer ):\n\
             A string that was supposed to be in some data, most likely from an external\n\
             data file could not be found, which indicates a corrupted data file or \n\
             a serious bug in the reading functions.\n\
             \n\
             The string that couldn't be located was: {}\n\
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
                <&BStr>::from(needle)
            );
        });

    trace!(
        "LocateStringInDate: String {} successfully located within data. ",
        <&BStr>::from(needle)
    );
    pos
}

/// This function counts the number of occurences of a string in a given
/// other string.
#[inline]
pub fn count_string_occurences(search: &[u8], target: &[u8]) -> usize {
    search
        .windows(target.len())
        .filter(|&s| s == target)
        .count()
}

/// This function looks for a sting begin indicator and takes the string
/// from after there up to a sting end indicator and mallocs memory for
/// it, copys it there and returns it.
/// The original source string specified should in no way be modified.
pub fn read_and_malloc_string_from_data(
    search_string: &[u8],
    start_indication_string: &[u8],
    end_indication_string: &[u8],
) -> CString {
    let mut search_pos = search_string
        .windows(start_indication_string.len())
        .position(|s| s == start_indication_string)
        .unwrap_or_else(|| {
            panic!(
                "\n\
                 \n\
                 ----------------------------------------------------------------------\n\
                 Freedroid has encountered a problem:\n\
                 In function 'char* ReadAndMalocStringFromData ( ... ):\n\
                 A starter string that was supposed to be in some data, most likely from an external\n\
                 data file could not be found, which indicates a corrupted data file or \n\
                 a serious bug in the reading functions.\n\
                 \n\
                 The string that couldn't be located was: {}\n\
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
                <&BStr>::from(start_indication_string)
            )
        });

    // Now we move to the beginning
    search_pos += start_indication_string.len();
    let search_slice = &search_string[search_pos..];
    let string_length = search_slice
        .windows(end_indication_string.len())
        .position(|s| s == end_indication_string)
        .unwrap_or_else(|| {
            panic!(
                "\n\
                 \n\
                 ----------------------------------------------------------------------\n\
                 Freedroid has encountered a problem:\n\
                 In function 'char* ReadAndMalocStringFromData ( ... ):\n\
                 A terminating string that was supposed to be in some data, most likely from an \
                 external\n\
                 data file could not be found, which indicates a corrupted data file or \n\
                 a serious bug in the reading functions.\n\
                 \n\
                 The string that couldn't be located was: {}\n\
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
                <&BStr>::from(end_indication_string)
            )
        });

    let return_string = CString::new(&search_slice[..string_length]).unwrap();

    info!(
        "ReadAndMalocStringFromData): Successfully identified string: {}.",
        return_string.to_string_lossy()
    );

    return_string
}

impl Data<'_> {
    pub fn update_progress(&mut self, percent: c_int) {
        let h =
            (f64::from(self.vars.progress_bar_rect.height()) * f64::from(percent) / 100.) as u16;
        let mut dst = Rect::new(
            self.vars.progress_bar_rect.x() + self.vars.progress_meter_rect.x(),
            self.vars.progress_bar_rect.y()
                + self.vars.progress_meter_rect.y()
                + self.vars.progress_bar_rect.height() as i16
                - h as i16,
            self.vars.progress_bar_rect.width(),
            h,
        );

        let src = Rect::new(
            0,
            self.vars.progress_bar_rect.height() as i16 - dst.height() as i16,
            dst.height(),
            0,
        );

        let Data {
            graphics:
                Graphics {
                    progress_filler_pic,
                    ne_screen,
                    ..
                },
            ..
        } = self;
        progress_filler_pic.as_mut().unwrap().blit_from_to(
            &src,
            ne_screen.as_mut().unwrap(),
            &mut dst,
        );
        ne_screen.as_mut().unwrap().update_rects(&[dst]);
    }

    /// This function is the key to independence of the framerate for various game elements.
    /// It returns the average time needed to draw one frame.
    /// Other functions use this to calculate new positions of moving objects, etc..
    ///
    /// Also there is of course a serious problem when some interuption occurs, like e.g.
    /// the options menu is called or the debug menu is called or the console or the elevator
    /// is entered or a takeover game takes place.  This might cause HUGE framerates, that could
    /// box the influencer out of the ship if used to calculate the new position.
    ///
    /// To counter unwanted effects after such events we have the SkipAFewFramerates counter,
    /// which instructs Rate_To_Be_Returned to return only the overall default framerate since
    /// no better substitute exists at this moment.  But on the other hand, this seems to
    /// work REALLY well this way.
    ///
    /// This counter is most conveniently set via the function
    /// Activate_Conservative_Frame_Computation, which can be conveniently called from eveywhere.
    pub fn frame_time(&mut self) -> c_float {
        let Self {
            global, misc, main, ..
        } = self;

        misc.frame_time(global, main.f_p_sover1)
    }

    /// Update the factor affecting the current speed of 'time flow'
    pub fn set_time_factor(&mut self, time_factor: c_float) {
        self.misc.current_time_factor = time_factor;
    }

    /// realise Pause-Mode: the game process is halted,
    /// while the graphics and animations are not.  This mode
    /// can further be toggled from PAUSE to CHEESE, which is
    /// a feature from the original program that should probably
    /// allow for better screenshots.
    pub fn pause(&mut self) {
        self.vars.me.status = Status::Pause as i32;
        self.assemble_combat_picture(AssembleCombatWindowFlags::DO_SCREEN_UPDATE.bits().into());

        let mut cheese = false;
        loop {
            self.start_taking_time_for_fps_calculation();

            if !cheese {
                self.animate_influence();
                self.animate_refresh();
                self.animate_enemys();
            }

            self.display_banner(None, None, 0);
            self.assemble_combat_picture(AssembleCombatWindowFlags::DO_SCREEN_UPDATE.bits().into());

            self.sdl.delay_ms(1);

            self.compute_fps_for_this_frame();

            #[cfg(feature = "gcw0")]
            let cond = self.gcw0_ls_pressed_r() || self.gcw0_rs_pressed_r();
            #[cfg(not(feature = "gcw0"))]
            let cond = self.key_is_pressed_r(b'c'.into());

            if cond {
                if self.vars.me.status != Status::Cheese as i32 {
                    self.vars.me.status = Status::Cheese as i32;
                } else {
                    self.vars.me.status = Status::Pause as i32;
                }
                cheese = !cheese;
            }

            if self.fire_pressed_r() || self.cmd_is_active_r(Cmds::Pause) {
                while self.cmd_is_active(Cmds::Pause) {
                    self.sdl.delay_ms(1);
                }
                break;
            }
        }
    }

    pub fn save_game_config(&self) -> c_int {
        use std::io::Write;
        if self.main.config_dir.is_empty() {
            return defs::ERR.into();
        }

        let config_path = Path::new(self.main.config_dir.to_str().unwrap()).join("config");
        let mut config = match File::create(&config_path) {
            Ok(config) => config,
            Err(_) => {
                warn!(
                    "WARNING: failed to create config-file: {}",
                    config_path.display()
                );
                return defs::ERR.into();
            }
        };

        // Now write the actual data, line by line
        writeln!(config, "{} = {}", VERSION_STRING, env!("CARGO_PKG_VERSION")).unwrap();
        writeln!(
            config,
            "{} = {}",
            DRAW_FRAMERATE, self.global.game_config.draw_framerate
        )
        .unwrap();
        writeln!(
            config,
            "{} = {}",
            DRAW_ENERGY, self.global.game_config.draw_energy
        )
        .unwrap();
        writeln!(
            config,
            "{} = {}",
            DRAW_POSITION, self.global.game_config.draw_position
        )
        .unwrap();
        writeln!(
            config,
            "{} = {}",
            DRAW_DEATHCOUNT, self.global.game_config.draw_death_count
        )
        .unwrap();
        writeln!(
            config,
            "{} = {}",
            DROID_TALK, self.global.game_config.droid_talk
        )
        .unwrap();
        writeln!(
            config,
            "{} = {}",
            WANTED_TEXT_VISIBLE_TIME, self.global.game_config.wanted_text_visible_time,
        )
        .unwrap();
        writeln!(
            config,
            "{} = {}",
            CURRENT_BG_MUSIC_VOLUME, self.global.game_config.current_bg_music_volume,
        )
        .unwrap();
        writeln!(
            config,
            "{} = {}",
            CURRENT_SOUND_FX_VOLUME, self.global.game_config.current_sound_fx_volume,
        )
        .unwrap();
        writeln!(
            config,
            "{} = {}",
            CURRENT_GAMMA_CORRECTION, self.global.game_config.current_gamma_correction,
        )
        .unwrap();
        writeln!(
            config,
            "{} = {}",
            THEME_NAME,
            self.global.game_config.theme_name.to_str().unwrap()
        )
        .unwrap();
        writeln!(
            config,
            "{} = {}",
            FULL_USER_RECT, self.global.game_config.full_user_rect
        )
        .unwrap();
        writeln!(
            config,
            "{} = {}",
            USE_FULLSCREEN, self.global.game_config.use_fullscreen
        )
        .unwrap();
        writeln!(
            config,
            "{} = {}",
            TAKEOVER_ACTIVATES, self.global.game_config.takeover_activates,
        )
        .unwrap();
        writeln!(
            config,
            "{} = {}",
            FIRE_HOLD_TAKEOVER, self.global.game_config.fire_hold_takeover,
        )
        .unwrap();
        writeln!(
            config,
            "{} = {}",
            SHOW_DECALS, self.global.game_config.show_decals
        )
        .unwrap();
        writeln!(
            config,
            "{} = {}",
            ALL_MAP_VISIBLE, self.global.game_config.all_map_visible
        )
        .unwrap();
        writeln!(
            config,
            "{} = {}",
            VID_SCALE_FACTOR, self.global.game_config.scale
        )
        .unwrap();
        writeln!(config, "{} = {}", HOG_CPU, self.global.game_config.hog_cpu).unwrap();
        writeln!(
            config,
            "{} = {}",
            EMPTY_LEVEL_SPEEDUP, self.global.game_config.empty_level_speedup,
        )
        .unwrap();

        // now write the keyboard->cmd mappings
        for (cmd_string, key_cmd) in CMD_STRINGS[0..Cmds::Last as usize]
            .iter()
            .copied()
            .zip(&self.input.key_cmds)
        {
            writeln!(
                config,
                "{} \t= {}_{}_{}",
                cmd_string, key_cmd[0], key_cmd[1], key_cmd[2],
            )
            .unwrap();
        }

        config.flush().unwrap();
        defs::OK.into()
    }

    /// This function starts the time-taking process.  Later the results
    /// of this function will be used to calculate the current framerate
    pub fn start_taking_time_for_fps_calculation(&mut self) {
        /* This ensures, that 0 is never an encountered framenr,
         * therefore count to 100 here
         * Take the time now for calculating the frame rate
         * (DO NOT MOVE THIS COMMAND PLEASE!) */
        self.misc.frame_nr += 1;

        self.misc.one_frame_sdl_ticks = self.sdl.ticks_ms();
    }

    pub fn compute_fps_for_this_frame(&mut self) {
        // In the following paragraph the framerate calculation is done.
        // There are basically two ways to do this:
        // The first way is to use self.sdl.ticks_ms(), a function measuring milliseconds
        // since the initialisation of the SDL.
        // The second way is to use gettimeofday, a standard ANSI C function I guess,
        // defined in time.h or so.
        //
        // I have arranged for a definition set in defs.h to switch between the two
        // methods of ramerate calculation.  THIS MIGHT INDEED MAKE SENSE, SINCE THERE
        // ARE SOME UNEXPLAINED FRAMERATE PHENOMENA WHICH HAVE TO TO WITH KEYBOARD
        // SPACE KEY, SO PLEASE DO NOT ERASE EITHER METHOD.  PLEASE ASK JP FIRST.
        //

        if self.global.skip_a_few_frames != 0 {
            return;
        }

        let Misc {
            now_sdl_ticks,
            one_frame_delay,
            ref one_frame_sdl_ticks,
            ..
        } = &mut self.misc;

        *now_sdl_ticks = self.sdl.ticks_ms();
        *one_frame_delay = c_long::from(*now_sdl_ticks) - c_long::from(*one_frame_sdl_ticks);
        *one_frame_delay = if *one_frame_delay > 0 {
            *one_frame_delay
        } else {
            1
        }; // avoid division by zero
        self.main.f_p_sover1 = (1000. / *one_frame_delay as f64) as f32;
    }

    pub fn activate_conservative_frame_computation(&mut self) {
        self.global.skip_a_few_frames = true.into();

        // Now we are in some form of pause.  It can't
        // hurt to have the top status bar redrawn after that,
        // so we set this variable...
        self.graphics.banner_is_destroyed = true.into();
    }

    /// Find a given filename in subdir relative to FD_DATADIR,
    ///
    /// if you pass NULL as "subdir", it will be ignored
    ///
    /// use current-theme subdir if "use_theme" == USE_THEME, otherwise NO_THEME
    ///
    /// behavior on file-not-found depends on parameter "critical"
    ///  IGNORE: just return NULL
    ///  WARNONLY: warn and return NULL
    ///  CRITICAL: Error-message and Terminate
    ///
    /// returns pointer to _static_ string array File_Path, which
    /// contains the full pathname of the file.
    ///
    /// !! do never try to free the returned string !!
    /// or to keep using it after a new call to find_file!
    pub fn find_file<'a>(
        &'a mut self,
        fname: &[u8],
        subdir: Option<&CStr>,
        use_theme: c_int,
        critical: c_int,
    ) -> Option<&'a CStr> {
        let Self { global, misc, .. } = self;
        Self::find_file_static(global, misc, fname, subdir, use_theme, critical)
    }

    pub fn find_file_static<'a>(
        global: &Global,
        misc: &'a mut Misc,
        fname: &[u8],
        subdir: Option<&CStr>,
        use_theme: c_int,
        mut critical: c_int,
    ) -> Option<&'a CStr> {
        use std::fmt::Write;

        if critical != Criticality::Ignore as c_int
            && critical != Criticality::WarnOnly as c_int
            && critical != Criticality::Critical as c_int
        {
            warn!(
                "WARNING: unknown critical-value passed to find_file(): {}. Assume CRITICAL",
                critical
            );
            critical = Criticality::Critical as c_int;
        }

        let fname: &BStr = fname.into();
        let mut inner = |datadir| {
            let theme_dir = if use_theme == Themed::UseTheme as c_int {
                Cow::Owned(format!(
                    "{}_theme/",
                    global.game_config.theme_name.to_string_lossy(),
                ))
            } else {
                Cow::Borrowed("")
            };

            misc.file_path.clear();
            write!(misc.file_path, "{datadir}",).unwrap();
            if let Some(subdir) = subdir {
                write!(misc.file_path, "/{}", subdir.to_string_lossy()).unwrap();
            }
            write!(misc.file_path, "/{theme_dir}/{}", fname).unwrap();

            misc.file_path
                .to_str()
                .map(|file_path| Path::new(file_path).exists())
                .unwrap_or(false)
        };

        let mut found = inner(LOCAL_DATADIR);
        if !found {
            found = inner(FD_DATADIR);
        }

        if !found {
            let critical = match critical.try_into() {
                Ok(critical) => critical,
                Err(_) => {
                    panic!("ERROR in find_file(): Code should never reach this line!! Harakiri",);
                }
            };
            // how critical is this file for the game:
            match critical {
                Criticality::WarnOnly => {
                    if use_theme == Themed::UseTheme as c_int {
                        warn!(
                            "file {} not found in theme-dir: graphics/{}_theme/",
                            fname,
                            global.game_config.theme_name.to_string_lossy(),
                        );
                    } else {
                        warn!("file {} not found ", fname);
                    }
                    return None;
                }
                Criticality::Ignore => return None,
                Criticality::Critical => {
                    if use_theme == Themed::UseTheme as c_int {
                        panic!(
                        "file {} not found in theme-dir: graphics/{}_theme/, cannot run without it!",
                        fname,
                        global.game_config.theme_name.to_string_lossy(),
                    );
                    } else {
                        panic!("file {} not found, cannot run without it!", fname);
                    }
                }
            }
        }

        Some(&*misc.file_path)
    }

    /// show_progress: display empty progress meter with given text
    pub fn init_progress(&mut self, text: &str) {
        if self.graphics.progress_meter_pic.is_none() {
            let fpath = Self::find_file_static(
                &self.global,
                &mut self.misc,
                PROGRESS_METER_FILE,
                Some(GRAPHICS_DIR_C),
                Themed::NoTheme as c_int,
                Criticality::Critical as c_int,
            );
            self.graphics.progress_meter_pic =
                self.graphics.load_block(fpath, 0, 0, None, 0, self.sdl);
            scale_pic(
                self.graphics.progress_meter_pic.as_mut().unwrap(),
                self.global.game_config.scale,
            );
            let fpath = Self::find_file_static(
                &self.global,
                &mut self.misc,
                PROGRESS_FILLER_FILE,
                Some(GRAPHICS_DIR_C),
                Themed::NoTheme as c_int,
                Criticality::Critical as c_int,
            );
            self.graphics.progress_filler_pic =
                self.graphics.load_block(fpath, 0, 0, None, 0, self.sdl);
            scale_pic(
                self.graphics.progress_filler_pic.as_mut().unwrap(),
                self.global.game_config.scale,
            );

            self.vars
                .progress_meter_rect
                .scale(self.global.game_config.scale);
            self.vars
                .progress_bar_rect
                .scale(self.global.game_config.scale);
            self.vars
                .progress_text_rect
                .scale(self.global.game_config.scale);
        }

        self.graphics.ne_screen.as_mut().unwrap().clear_clip_rect();
        let Data {
            graphics:
                Graphics {
                    progress_meter_pic,
                    ne_screen,
                    ..
                },
            ..
        } = self;
        progress_meter_pic.as_mut().unwrap().blit_to(
            ne_screen.as_mut().unwrap(),
            &mut self.vars.progress_meter_rect,
        );

        let mut dst = self.vars.progress_text_rect;
        dst.inc_x(self.vars.progress_meter_rect.x());
        dst.inc_y(self.vars.progress_meter_rect.y());

        let mut ne_screen = self.graphics.ne_screen.take().unwrap();
        self.printf_sdl(
            &mut ne_screen,
            dst.x().into(),
            dst.y().into(),
            format_args!("{}", text),
        );

        assert!(ne_screen.flip());
        self.graphics.ne_screen = Some(ne_screen);
    }

    /// This function teleports the influencer to a new position on the
    /// ship.  THIS CAN BE A POSITION ON A DIFFERENT LEVEL.
    pub fn teleport(&mut self, level_num: c_int, x: c_int, y: c_int) {
        let cur_level = level_num;
        let mut array_num = 0;

        if cur_level != self.main.cur_level().levelnum {
            //--------------------
            // In case a real level change has happend,
            // we need to do a lot of work:

            while let Some(level) = &self.main.cur_ship.all_levels[array_num] {
                if level.levelnum == cur_level {
                    break;
                } else {
                    array_num += 1;
                }
            }

            self.main.cur_level_index = Some(ArrayIndex::new(array_num));

            self.shuffle_enemys();

            self.vars.me.pos.x = x as f32;
            self.vars.me.pos.y = y as f32;

            // turn off all blasts and bullets from the old level
            self.main
                .all_blasts
                .iter_mut()
                .take(MAXBLASTS)
                .for_each(|blast| blast.ty = Status::Out as i32);
            (0..MAXBULLETS).for_each(|bullet| self.delete_bullet(bullet.try_into().unwrap()));
        } else {
            //--------------------
            // If no real level change has occured, everything
            // is simple and we just need to set the new coordinates, haha
            //
            self.vars.me.pos.x = x as f32;
            self.vars.me.pos.y = y as f32;
        }

        self.leave_lift_sound();
    }

    /// This function is kills all enemy robots on the whole ship.
    /// It querys the user once for safety.
    pub fn armageddon(&mut self) {
        self.main
            .all_enemys
            .iter_mut()
            .take(self.main.num_enemys.try_into().unwrap())
            .for_each(|enemy| {
                enemy.energy = 0.;
                enemy.status = Status::Out as c_int;
            });
    }

    /// LoadGameConfig(): load saved options from config-file
    ///
    /// this should be the first of all load/save functions called
    /// as here we read the $HOME-dir and create the config-subdir if neccessary
    pub fn load_game_config(&mut self) -> c_int {
        use std::fmt::Write;

        // ----------------------------------------------------------------------
        // Game-config maker-strings for config-file:

        const VERSION_STRING: &str = "Freedroid Version";
        const DRAW_FRAMERATE: &str = "Draw_Framerate";
        const DRAW_ENERGY: &str = "Draw_Energy";
        const DRAW_POSITION: &str = "Draw_Position";
        const DRAW_DEATHCOUNT: &str = "Draw_DeathCount";
        const DROID_TALK: &str = "Droid_Talk";
        const WANTED_TEXT_VISIBLE_TIME: &str = "WantedTextVisibleTime";
        const CURRENT_BG_MUSIC_VOLUME: &str = "Current_BG_Music_Volume";
        const CURRENT_SOUND_FX_VOLUME: &str = "Current_Sound_FX_Volume";
        const CURRENT_GAMMA_CORRECTION: &str = "Current_Gamma_Correction";
        const THEME_NAME: &str = "Theme_Name";
        const FULL_USER_RECT: &str = "FullUserRect";
        const USE_FULLSCREEN: &str = "UseFullscreen";
        const TAKEOVER_ACTIVATES: &str = "TakeoverActivates";
        const FIRE_HOLD_TAKEOVER: &str = "FireHoldTakeover";
        const SHOW_DECALS: &str = "ShowDecals";
        const ALL_MAP_VISIBLE: &str = "AllMapVisible";
        const VID_SCALE_FACTOR: &str = "Vid_ScaleFactor";
        const HOG_CPU: &str = "Hog_Cpu";
        const EMPTY_LEVEL_SPEEDUP: &str = "EmptyLevelSpeedup";

        // first we need the user's homedir for loading/saving stuff
        let homedir = match env::var("HOME") {
            Err(_) => {
                warn!("Environment does not contain HOME variable...using local dir");
                Cow::Borrowed(Path::new("."))
            }
            Ok(homedir) => {
                info!("found environment HOME = '{}'", homedir);
                Cow::Owned(homedir.into())
            }
        };

        let config_dir = homedir.join(".freedroidClassic");
        self.main.config_dir.clear();
        write!(self.main.config_dir, "{}", config_dir.display()).unwrap();

        if !config_dir.exists() {
            warn!(
                "Couldn't stat Config-dir {}, I'll try to create it...",
                config_dir.display()
            );
            match fs::create_dir(&config_dir) {
                Ok(()) => {
                    info!("Successfully created config-dir '{}'", config_dir.display());
                    return defs::OK.into();
                }
                Err(_) => {
                    error!(
                        "Failed to create config-dir: {}. Giving up...",
                        config_dir.display()
                    );
                    return defs::ERR.into();
                }
            }
        }

        let config_path = config_dir.join("config");
        let data = match fs::read(&config_path) {
            Ok(data) => {
                info!("Successfully read config-file '{}'", config_path.display());
                data
            }
            Err(_) => {
                error!("failed to open config-file: {}", config_path.display());
                return defs::ERR.into();
            }
        };

        if read_variable(&data, VERSION_STRING).is_none() {
            error!("Version string could not be read in config-file...");
            return defs::ERR.into();
        }

        macro_rules! parse_variable {
        (@@inner = $name:expr; $($var:tt)+) => {
            {
                let value = read_variable(&data, $name)
                    .and_then(|slice| std::str::from_utf8(slice).ok())
                    .and_then(|value| value.parse().ok());
                if let Some(value) = value {
                    $($var)+ = value;
                }
            }
        };
        (@@inner $tt:tt $($rest:tt)+) => {
                parse_variable!(@@inner $($rest)+ $tt);
        };
        ($($tt:tt)+) => {
            parse_variable!(@@inner $($tt)+);
        };
    }

        parse_variable! { self.global.game_config.draw_framerate = DRAW_FRAMERATE; };
        parse_variable! { self.global.game_config.draw_energy = DRAW_ENERGY; };
        parse_variable! { self.global.game_config.draw_position = DRAW_POSITION; };
        parse_variable! { self.global.game_config.draw_death_count = DRAW_DEATHCOUNT; };
        parse_variable! { self.global.game_config.droid_talk = DROID_TALK; };
        parse_variable! { self.global.game_config.wanted_text_visible_time = WANTED_TEXT_VISIBLE_TIME; };
        parse_variable! { self.global.game_config.current_bg_music_volume = CURRENT_BG_MUSIC_VOLUME; };
        parse_variable! { self.global.game_config.current_sound_fx_volume = CURRENT_SOUND_FX_VOLUME; };
        parse_variable! { self.global.game_config.current_gamma_correction = CURRENT_GAMMA_CORRECTION; };
        {
            let value = read_variable(&data, THEME_NAME);
            if let Some(value) = value {
                self.global.game_config.theme_name.set_slice(value);
            }
        }
        parse_variable! { self.global.game_config.full_user_rect = FULL_USER_RECT; };
        parse_variable! { self.global.game_config.use_fullscreen = USE_FULLSCREEN; };
        parse_variable! { self.global.game_config.takeover_activates = TAKEOVER_ACTIVATES; };
        parse_variable! { self.global.game_config.fire_hold_takeover = FIRE_HOLD_TAKEOVER; };
        parse_variable! { self.global.game_config.show_decals = SHOW_DECALS; };
        parse_variable! { self.global.game_config.all_map_visible = ALL_MAP_VISIBLE; };
        parse_variable! { self.global.game_config.scale = VID_SCALE_FACTOR; };
        parse_variable! { self.global.game_config.hog_cpu = HOG_CPU; };
        parse_variable! { self.global.game_config.empty_level_speedup = EMPTY_LEVEL_SPEEDUP; };

        // read in keyboard-config
        for (index, &cmd_string) in CMD_STRINGS.iter().enumerate() {
            let value = read_variable(&data, cmd_string);
            if let Some(value) = value {
                let value = std::str::from_utf8(value).unwrap();
                self.input.key_cmds[index]
                    .iter_mut()
                    .zip(value.splitn(3, '_').map(|x| x.parse().unwrap()))
                    .for_each(|(key_cmd, value)| *key_cmd = value);
            }
        }

        defs::OK.into()
    }
}

fn read_variable<'a>(data: &'a [u8], var_name: &str) -> Option<&'a [u8]> {
    data.lines()
        .filter_map(|line| line.trim_start().strip_prefix(var_name.as_bytes()))
        .filter_map(|line| line.trim_start().strip_prefix(b"="))
        .map(|line| line.trim())
        .next()
}

impl Misc {
    pub fn frame_time(&mut self, global: &Global, f_p_sover1: f32) -> c_float {
        if global.skip_a_few_frames != 0 {
            return self.previous_time;
        }

        if f_p_sover1 > 0. {
            self.previous_time = 1.0 / f_p_sover1;
        }

        self.previous_time * self.current_time_factor
    }
}
