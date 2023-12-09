use crate::{
    array_c_string::ArrayCString,
    b_font::font_height,
    defs::{
        self, AssembleCombatWindowFlags, Criticality, DisplayBannerFlags, Droid, Status, Themed,
        FD_DATADIR, GRAPHICS_DIR_C, LOCAL_DATADIR, MAP_DIR_C, MAXBULLETS, SHOW_WAIT, SLOWMO_FACTOR,
        TITLE_PIC_FILE, WAIT_AFTER_KILLED,
    },
    global::Global,
    graphics::Graphics,
    misc::{
        count_string_occurences, locate_string_in_data, my_random,
        read_and_malloc_string_from_data, read_float_from_string, read_i32_from_string,
        read_string_from_string,
    },
    read_and_malloc_and_terminate_file,
    sound::Sound,
    split_at_subslice,
    structs::{BulletSpec, DruidSpec, TextToBeDisplayed},
    ArrayIndex, Data,
};

#[cfg(target_os = "windows")]
use crate::input::wait_for_key_pressed;

use bstr::ByteSlice;
use clap::{crate_version, ArgAction, Parser};
use cstr::cstr;
use log::{error, info, warn};
use nom::Finish;
use std::{
    ffi::CString,
    ops::Not,
    os::raw::{c_float, c_int, c_long},
    path::Path,
};

#[derive(Debug, Default)]
pub struct Init {
    debriefing_text: CString,
    debriefing_song: ArrayCString<500>,
    previous_mission_name: ArrayCString<500>,
}

const MISSION_COMPLETE_BONUS: f32 = 1000.;
const COPYRIGHT: &str = "\nCopyright (C) 2003-2018 Johannes Prix, Reinhard Prix\n\
Freedroid comes with NO WARRANTY to the extent permitted by law.\n\
You may redistribute copies of Freedroid under the terms of the\n\
GNU General Public License.\n\
For more information about these matters, see the file named COPYING.";

/// put some ideology message for our poor friends enslaved by M$-Win32 ;)
#[cfg(target_os = "windows")]
pub fn win32_disclaimer() {
    self.graphics.ne_screen.as_mut().unwrap().clear_clip_rect();
    display_image(find_file(
        TITLE_PIC_FILE_C.as_ptr() as *mut c_char,
        GRAPHICS_DIR_C.as_ptr() as *mut c_char,
        Themed::NoTheme as c_int,
        Criticality::Critical as c_int,
    )); // show title pic
    make_grid_on_screen(Some(&Screen_Rect));

    set_current_font(self.global.para_b_font);

    let mut rect = Full_User_Rect;
    rect.x += 10;
    rect.w -= 10; //leave some border
    DisplayText(
        cstr!(
        "Windows disclaimer:\n\nThis program is 100% Free (as in Freedom), licenced under the GPL.\
         \nIt is developed on a free operating system (GNU/Linux) using exclusively free tools. \
         For more information about Free Software see the GPL licence (in the file COPYING)\n\
         or visit http://www.gnu.org.\n\n\n Press fire to play.")
        .as_ptr(),
        rect.x.into(),
        rect.y.into(),
        &rect,
    );
    assert!(self.graphics.ne_screen.as_mut().unwrap().flip());

    wait_for_key_pressed();
}

#[derive(Parser)]
#[clap(version = crate_version!(), long_version = COPYRIGHT)]
struct Opt {
    #[clap(short, long, conflicts_with = "nosound")]
    sound: bool,

    #[clap(short = 'q', long, conflicts_with = "sound")]
    nosound: bool,

    #[clap(short, long, action = ArgAction::Count)]
    debug: u8,

    #[clap(short, long, conflicts_with = "fullscreen")]
    window: bool,

    #[clap(short, long, conflicts_with = "window")]
    fullscreen: bool,

    #[clap(short = 'j', long)]
    sensitivity: Option<u8>,

    #[clap(short = 'r', long)]
    scale: Option<f32>,
}

impl Data<'_> {
    pub fn free_game_mem(&mut self) {
        // free bullet map
        if self.vars.bulletmap.is_empty().not() {
            let bullet_map = &mut *self.vars.bulletmap;
            for bullet in bullet_map {
                for surface in &mut bullet.surfaces {
                    *surface = None;
                }
            }
            self.vars.bulletmap.clear();
        }

        // free blast map
        for blast_type in &mut self.vars.blastmap {
            for surface in &mut blast_type.surfaces {
                *surface = None;
            }
        }

        // free droid map
        self.free_druidmap();

        // free highscores list
        drop(self.highscore.entries.take());

        // free constant text blobs
        self.init.debriefing_text = Default::default();
    }

    pub fn free_druidmap(&mut self) {
        if self.vars.droidmap.is_empty() {
            return;
        }
        for droid in &mut self.vars.droidmap {
            droid.notes = Default::default();
        }

        self.vars.droidmap.clear();
    }

    /// This function checks, if the influencer has succeeded in his given
    /// mission.  If not it returns, if yes the Debriefing is started.
    pub(crate) fn check_if_mission_is_complete(&mut self) {
        for enemy in self
            .main
            .all_enemys
            .iter()
            .take(self.main.num_enemys.try_into().unwrap())
        {
            if enemy.status != Status::Out as c_int && enemy.status != Status::Terminated as c_int {
                return;
            }
        }

        // mission complete: all droids have been killed
        self.main.real_score += MISSION_COMPLETE_BONUS;
        self.thou_art_victorious();
        self.game_over = true;
    }

    pub fn thou_art_victorious(&mut self) {
        Self::switch_background_music_to_static(
            self.sound.as_mut().unwrap(),
            &self.main,
            &self.global,
            &mut self.misc,
            self.sdl,
            Some(self.init.debriefing_song.to_bytes()),
        );

        self.sdl.cursor().hide();

        self.main.show_score = self.main.real_score as c_long;
        self.vars.me.status = Status::Victory as c_int;
        self.display_banner(None, None, DisplayBannerFlags::FORCE_UPDATE.bits().into());

        self.wait_for_all_keys_released();

        let now = self.sdl.ticks_ms();

        while self.sdl.ticks_ms() - now < WAIT_AFTER_KILLED {
            self.display_banner(None, None, 0);
            self.explode_blasts();
            self.move_bullets();
            self.assemble_combat_picture(AssembleCombatWindowFlags::DO_SCREEN_UPDATE.bits().into());
        }

        let mut rect = self.vars.full_user_rect;
        self.graphics.ne_screen.as_mut().unwrap().clear_clip_rect();
        self.make_grid_on_screen(Some(&rect));
        assert!(self.graphics.ne_screen.as_mut().unwrap().flip());
        rect.inc_x(10);
        rect.dec_width(20); //leave some border
        self.b_font.current_font = self.global.para_b_font.clone();
        Self::scroll_text_static(
            &mut self.graphics,
            &mut self.input,
            self.sdl,
            &self.vars,
            &self.global,
            &mut self.text,
            &self.b_font,
            &mut self.font_owner,
            &self.quit,
            self.init.debriefing_text.to_bytes(),
            &mut rect,
            6,
        );

        self.wait_for_all_keys_released();
    }

    /// This function initializes the whole Freedroid game.
    ///
    /// This must not be confused with initnewgame, which
    /// only initializes a new mission for the game.
    pub fn init_freedroid(&mut self) {
        self.vars.bulletmap.clear(); // That will cause the memory to be allocated later

        for bullet in &mut self.main.all_bullets[..MAXBULLETS] {
            bullet.surfaces_were_generated = false.into();
        }

        self.global.skip_a_few_frames = false.into();
        self.vars.me.text_visible_time = 0.;
        self.vars.me.text_to_be_displayed = TextToBeDisplayed::None;

        // these are the hardcoded game-defaults, they can be overloaded by the config-file if present
        self.global.game_config.current_bg_music_volume = 0.3;
        self.global.game_config.current_sound_fx_volume = 0.5;

        self.global.game_config.wanted_text_visible_time = 3.;
        self.global.game_config.droid_talk = false.into();

        self.global.game_config.draw_framerate = false.into();
        self.global.game_config.draw_energy = false.into();
        self.global.game_config.draw_death_count = false.into();
        self.global.game_config.draw_position = false.into();

        self.global.game_config.theme_name.set_slice("classic");
        self.global.game_config.full_user_rect = true.into();
        self.global.game_config.use_fullscreen = false.into();
        self.global.game_config.takeover_activates = true.into();
        self.global.game_config.fire_hold_takeover = true.into();
        self.global.game_config.show_decals = false.into();
        self.global.game_config.all_map_visible = true.into(); // classic setting: map always visible

        let scale = if cfg!(feature = "gcw0") {
            0.5 // Default for 320x200 device (GCW0)
        } else {
            1.0 // overall scaling of _all_ graphics (e.g. for 320x200 displays)
        };
        self.global.game_config.scale = scale;

        self.global.game_config.hog_cpu = false.into(); // default to being nice
        self.global.game_config.empty_level_speedup = 1.0; // speed up *time* in empty levels (ie also energy-loss rate)

        // now load saved options from the config-file
        self.load_game_config();

        // call this _after_ default settings and LoadGameConfig() ==> cmdline has highest priority!
        self.parse_command_line();

        self.vars.user_rect = if self.global.game_config.full_user_rect != 0 {
            self.vars.full_user_rect
        } else {
            self.vars.classic_user_rect
        };

        self.vars.screen_rect.scale(self.global.game_config.scale); // make sure we open a window of the right (rescaled) size!
        self.init_video();

        let image = Self::find_file_static(
            &self.global,
            &mut self.misc,
            TITLE_PIC_FILE,
            Some(GRAPHICS_DIR_C),
            Themed::NoTheme as c_int,
            Criticality::Critical as c_int,
        )
        .unwrap();
        Self::display_image(self.sdl, &self.global, &mut self.graphics, image); // show title pic
        assert!(self.graphics.ne_screen.as_mut().unwrap().flip());

        self.load_fonts(); // we need this for progress-meter!

        self.init_progress("Loading Freedroid");

        self.find_all_themes(); // put all found themes into a list: AllThemes[]

        self.update_progress(5);

        let &mut Self {
            ref mut sound,
            ref mut misc,
            ref global,
            ref mut main,
            sdl,
            ..
        } = self;
        *sound = Sound::new(main, sdl, global, misc);

        self.init_joy();

        self.init_game_data(b"freedroid.ruleset"); // load the default ruleset. This can be
                                                   // overwritten from the mission file.

        self.update_progress(10);

        // The default should be, that no rescaling of the
        // combat window at all is done.
        self.global.current_combat_scale_factor = 1.;

        /* initialize/load the highscore list */
        self.init_highscores();

        /* Now fill the pictures correctly to the structs */
        assert!(
            self.init_pictures() != 0,
            "Error in InitPictures reported back..."
        );

        self.update_progress(100); // finished init
    }

    /// parse command line arguments and set global switches
    /// exit on error, so we don't need to return success status
    fn parse_command_line(&mut self) {
        let opt = Opt::parse();

        if opt.nosound {
            self.main.sound_on = false.into();
        } else if opt.sound {
            self.main.sound_on = true.into();
        }

        if let Some(sensitivity) = opt.sensitivity {
            assert!(
                sensitivity <= 32,
                "\nJoystick sensitivity must lie in the range [0;32]"
            );

            self.input.joy_sensitivity = sensitivity.into();
        }

        let log_level = match opt.debug {
            0 => None,
            1 => Some(log::LevelFilter::Error),
            2 => Some(log::LevelFilter::Warn),
            3 => Some(log::LevelFilter::Info),
            4 => Some(log::LevelFilter::Debug),
            _ => Some(log::LevelFilter::Trace),
        };
        if let Some(log_level) = log_level {
            log::set_max_level(log_level);
        }

        if let Some(scale) = opt.scale {
            assert!(
                scale > 0.,
                "illegal scale entered, needs to be >0: {}",
                scale
            );
            self.global.game_config.scale = scale;
            info!("Graphics scale set to {}", scale);
        }

        if opt.fullscreen {
            self.global.game_config.use_fullscreen = true.into();
        } else if opt.window {
            self.global.game_config.use_fullscreen = false.into();
        }
    }

    /// find all themes and put them in AllThemes
    pub fn find_all_themes(&mut self) {
        use std::fs;

        let mut classic_theme_index: usize = 0; // default: override when we actually find 'classic' theme

        // just to make sure...
        self.graphics.all_themes.num_themes = 0;
        self.graphics.all_themes.theme_name.fill(Default::default());

        let mut add_theme_from_dir = |dir_name: &Path| {
            let dir_name = dir_name.join("graphics");
            let read_dir = match fs::read_dir(&dir_name) {
                Ok(read_dir) => read_dir,
                Err(err) => {
                    warn!("can't open data-directory {}: {}.", dir_name.display(), err);
                    return;
                }
            };

            for entry in read_dir {
                {
                    let entry = match entry {
                        Ok(entry) => entry,
                        Err(err) => {
                            warn!(
                                "cannot get next entry from dir {}: {}",
                                dir_name.display(),
                                err
                            );
                            continue;
                        }
                    };

                    let file_type = match entry.file_type() {
                        Ok(file_type) => file_type,
                        Err(err) => {
                            error!(
                                "could not get file type for {}: {}",
                                entry.path().display(),
                                err
                            );
                            continue;
                        }
                    };

                    if file_type.is_dir().not() {
                        continue;
                    }

                    let theme_name = entry.file_name();
                    let theme_name = match theme_name
                        .to_str()
                        .and_then(|name| name.strip_suffix("_theme"))
                    {
                        Some(theme_name) => theme_name,
                        None => continue,
                    };

                    let theme_path = entry.path();
                    if theme_name.len() >= 100 {
                        warn!(
                            "theme-name of '{}' longer than allowed 100 chars... discarded!",
                            theme_path.display()
                        );
                        continue;
                    }

                    info!("Found a new theme: {}", theme_name);
                    // check readabiltiy of "config.theme"
                    let config_path = theme_path.join(Path::new("config.theme"));

                    match fs::File::open(config_path) {
                        Ok(_) => {
                            info!("The theme file is readable");
                            // last check: is this theme already in the list??

                            let theme_exists = self
                                .graphics
                                .all_themes
                                .theme_name
                                .iter()
                                .filter_map(|s| s.to_str().ok())
                                .any(|theme| theme == theme_name);

                            if theme_exists {
                                info!("Theme '{}' is already listed", theme_name);
                                continue;
                            } else {
                                info!("Found new graphics-theme: {}", theme_name);
                                if theme_name == "classic" {
                                    classic_theme_index =
                                        self.graphics.all_themes.num_themes.try_into().unwrap();
                                }
                                self.graphics.all_themes.theme_name[usize::try_from(
                                    self.graphics.all_themes.num_themes,
                                )
                                .unwrap()] = CString::new(theme_name).unwrap();

                                self.graphics.all_themes.num_themes += 1;
                            }
                        }
                        Err(err) => {
                            warn!(
                                "config.theme of theme '{}' not readable: {}. Discarded.",
                                theme_name, err
                            );
                            continue;
                        }
                    }
                }
            }
        };

        add_theme_from_dir(Path::new(FD_DATADIR));
        add_theme_from_dir(Path::new(LOCAL_DATADIR));

        // now have a look at what we found:
        assert!(
            self.graphics.all_themes.num_themes != 0,
            "No valid graphic-themes found!! You need to install at least one to run Freedroid!!"
        );

        let Self {
            graphics: Graphics { all_themes, .. },
            global: Global { game_config, .. },
            ..
        } = self;
        let selected_theme_index = all_themes.theme_name
            [..usize::try_from(all_themes.num_themes).unwrap()]
            .iter()
            .position(|theme_name| **theme_name == game_config.theme_name);

        match selected_theme_index {
            Some(index) => {
                info!(
                    "Found selected theme {} from GameConfig.",
                    self.global.game_config.theme_name.to_string_lossy(),
                );
                self.graphics.all_themes.cur_tnum = index.try_into().unwrap();
            }
            None => {
                warn!(
                    "selected theme {} not valid! Using classic theme.",
                    self.global.game_config.theme_name.to_string_lossy(),
                );
                self.global
                    .game_config
                    .theme_name
                    .set(&self.graphics.all_themes.theme_name[classic_theme_index]);
                self.graphics.all_themes.cur_tnum = classic_theme_index.try_into().unwrap();
            }
        }

        info!(
            "Game starts using theme: {}",
            self.global.game_config.theme_name.to_str().unwrap()
        );
    }

    pub fn init_new_mission(&mut self, mission_name: &str) {
        const END_OF_MISSION_DATA_STRING: &[u8] = b"*** End of Mission File ***";
        const MISSION_BRIEFING_BEGIN_STRING: &[u8] =
            b"** Start of Mission Briefing Text Section **";
        const MISSION_ENDTITLE_SONG_NAME_STRING: &[u8] =
            b"Song name to play in the end title if the mission is completed: ";
        const SHIPNAME_INDICATION_STRING: &[u8] = b"Ship file to use for this mission: ";
        const ELEVATORNAME_INDICATION_STRING: &[u8] = b"Lift file to use for this mission: ";
        const CREWNAME_INDICATION_STRING: &[u8] = b"Crew file to use for this mission: ";
        const GAMEDATANAME_INDICATION_STRING: &[u8] =
            b"Physics ('game.dat') file to use for this mission: ";
        const MISSION_ENDTITLE_BEGIN_STRING: &[u8] = b"** Beginning of End Title Text Section **";
        const MISSION_ENDTITLE_END_STRING: &[u8] = b"** End of End Title Text Section **";
        const MISSION_START_POINT_STRING: &[u8] = b"Possible Start Point : ";

        // We store the mission name in case the influ
        // gets destroyed so we know where to continue in
        // case the player doesn't want to return to the very beginning
        // but just to replay this mission.
        self.init.previous_mission_name.clear();
        self.init.previous_mission_name.push_str(mission_name);

        info!(
            "A new mission is being initialized from file {}.",
            mission_name
        );

        //--------------------
        //At first we do the things that must be done for all
        //missions, regardless of mission file given
        self.activate_conservative_frame_computation();
        self.main.last_got_into_blast_sound = 2.;
        self.main.last_refresh_sound = 2.;
        self.global.level_doors_not_moved_time = 0.0;
        self.main.death_count = 0.;
        self.set_time_factor(1.0);

        /* Delete all bullets and blasts */
        for bullet in 0..MAXBULLETS {
            self.delete_bullet(bullet.try_into().unwrap());
        }

        info!("InitNewMission: All bullets have been deleted.");
        for blast in &mut self.main.all_blasts {
            blast.phase = Status::Out as c_int as c_float;
            blast.ty = Status::Out as c_int;
        }
        info!("InitNewMission: All blasts have been deleted.");
        for enemy in &mut self.main.all_enemys {
            enemy.ty = Status::Out as c_int;
            enemy.energy = -1.;
        }
        info!("InitNewMission: All enemys have been deleted...");

        //Now its time to start decoding the mission file.
        //For that, we must get it into memory first.
        //The procedure is the same as with LoadShip

        let oldfont = std::mem::replace(
            &mut self.b_font.current_font,
            self.global.font0_b_font.clone(),
        );

        /* Read the whole mission data to memory */
        let fpath = self
            .find_file(
                mission_name.as_bytes(),
                Some(MAP_DIR_C),
                Themed::NoTheme as c_int,
                Criticality::Critical as c_int,
            )
            .unwrap();
        let fpath = Path::new(
            fpath
                .to_str()
                .expect("Unable to convert C string to UTF-8 string"),
        );

        let main_mission_data =
            read_and_malloc_and_terminate_file(fpath, END_OF_MISSION_DATA_STRING);

        //--------------------
        // Now the mission file is read into memory.  That means we can start to decode the details given
        // in the body of the mission file.

        //--------------------
        // First we extract the game physics file name from the
        // mission file and load the game data.
        //
        let indication =
            read_string_from_string(&main_mission_data, GAMEDATANAME_INDICATION_STRING);

        self.init_game_data(indication);

        //--------------------
        // Now its time to get the shipname from the mission file and
        // read the ship file into the right memory structures
        //
        let indication = read_string_from_string(&main_mission_data, SHIPNAME_INDICATION_STRING);

        assert!(
            self.load_ship(indication) != defs::ERR.into(),
            "Error in LoadShip"
        );
        //--------------------
        // Now its time to get the elevator file name from the mission file and
        // read the elevator file into the right memory structures
        //
        let indication =
            read_string_from_string(&main_mission_data, ELEVATORNAME_INDICATION_STRING);

        assert!(
            self.get_lift_connections(indication) != defs::ERR.into(),
            "Error in GetLiftConnections"
        );
        //--------------------
        // We also load the comment for the influencer to say at the beginning of the mission
        //

        // NO! these strings are allocated elsewhere or even static, so free'ing them
        // here would SegFault eventually!
        //  if (Me.TextToBeDisplayed) free (Me.TextToBeDisplayed);

        self.vars.me.text_to_be_displayed =
            TextToBeDisplayed::String(cstr!("Ok. I'm on board.  Let's get to work.")); // taken from Paradroid.mission
        self.vars.me.text_visible_time = 0.;

        //--------------------
        // Now its time to get the crew file name from the mission file and
        // assemble an appropriate crew out of it
        //
        let indication = read_string_from_string(&main_mission_data, CREWNAME_INDICATION_STRING);

        /* initialize enemys according to crew file */
        // WARNING!! THIS REQUIRES THE freedroid.ruleset FILE TO BE READ ALREADY, BECAUSE
        // ROBOT SPECIFICATIONS ARE ALREADY REQUIRED HERE!!!!!
        assert!(
            self.get_crew(indication) != defs::ERR.into(),
            "InitNewGame(): Initialization of enemys failed."
        );

        //--------------------
        // Now its time to get the debriefing text from the mission file so that it
        // can be used, if the mission is completed and also the end title music name
        // must be read in as well
        let song_name =
            read_string_from_string(&main_mission_data, MISSION_ENDTITLE_SONG_NAME_STRING);
        self.init.debriefing_song.set_slice(song_name);

        self.init.debriefing_text = read_and_malloc_string_from_data(
            &main_mission_data,
            MISSION_ENDTITLE_BEGIN_STRING,
            MISSION_ENDTITLE_END_STRING,
        );

        //--------------------
        // Now we read all the possible starting points for the
        // current mission file, so that we know where to place the
        // influencer at the beginning of the mission.

        let number_of_start_points =
            count_string_occurences(&main_mission_data, MISSION_START_POINT_STRING);

        assert!(
            number_of_start_points != 0,
            "NOT EVEN ONE SINGLE STARTING POINT ENTRY FOUND!  TERMINATING!"
        );
        info!(
            "Found {} different starting points for the mission in the mission file.",
            number_of_start_points,
        );

        // Now that we know how many different starting points there are, we can randomly select
        // one of them and read then in this one starting point into the right structures...
        let start_point_index = main_mission_data
            .windows(MISSION_START_POINT_STRING.len())
            .enumerate()
            .filter(|&(_, slice)| slice == MISSION_START_POINT_STRING)
            .map(|(index, _)| index)
            .nth(
                usize::try_from(my_random((number_of_start_points - 1).try_into().unwrap()))
                    .unwrap(),
            )
            .unwrap();

        let start_point_slice = split_at_subslice(
            &main_mission_data[(start_point_index + MISSION_START_POINT_STRING.len())..],
            b"Level=",
        )
        .expect("unable to find Level parameter in mission data")
        .1;
        let starting_level = nom::character::complete::i32::<_, ()>(start_point_slice)
            .finish()
            .unwrap()
            .1;
        self.main.cur_level_index = Some(ArrayIndex::new(usize::try_from(starting_level).unwrap()));

        let start_point_slice = split_at_subslice(start_point_slice, b"XPos=").unwrap().1;
        let starting_x_pos = nom::character::complete::i32::<_, ()>(start_point_slice)
            .finish()
            .expect("unable to find XPos parameter in mission data")
            .1;
        self.vars.me.pos.x = starting_x_pos as c_float;

        let start_point_slice = split_at_subslice(start_point_slice, b"YPos=").unwrap().1;
        let starting_y_pos = nom::character::complete::i32::<_, ()>(start_point_slice)
            .finish()
            .expect("unable to find YPos parameter in mission data")
            .1;

        self.vars.me.pos.y = starting_y_pos as c_float;
        info!(
            "Final starting position: Level={} XPos={} YPos={}.",
            starting_level, starting_x_pos, starting_y_pos,
        );

        /* Reactivate the light on alle Levels, that might have been dark */
        for level in &mut self.main.cur_ship.all_levels
            [0..usize::try_from(self.main.cur_ship.num_levels).unwrap()]
        {
            level.as_mut().unwrap().empty = false.into();
        }

        info!("InitNewMission: All levels have been set to 'active'...",);

        //--------------------
        // At this point the position history can be initialized
        //
        self.init_influ_position_history();
        self.b_font.current_font = oldfont;
        //--------------------
        // We start with doing the briefing things...
        // Now we search for the beginning of the mission briefing big section NOT subsection.
        // We display the title and explanation of controls and such...
        let briefing_section_pos =
            locate_string_in_data(&main_mission_data, MISSION_BRIEFING_BEGIN_STRING);
        self.title(&main_mission_data[briefing_section_pos..]);

        if self.quit.get() {
            return;
        }

        /* Den Banner fuer das Spiel anzeigen */
        self.clear_graph_mem();
        self.display_banner(None, None, DisplayBannerFlags::FORCE_UPDATE.bits().into());

        // Switch_Background_Music_To (COMBAT_BACKGROUND_MUSIC_SOUND);
        Self::switch_background_music_to_static(
            self.sound.as_mut().unwrap(),
            &self.main,
            &self.global,
            &mut self.misc,
            self.sdl,
            Some(self.main.cur_level().background_song_name.to_bytes()),
        );

        for level_index in 0..usize::try_from(self.main.cur_ship.num_levels).unwrap() {
            self.main.cur_level_index = Some(ArrayIndex::new(level_index));
            self.shuffle_enemys();
        }

        self.main.cur_level_index = Some(ArrayIndex::new(usize::try_from(starting_level).unwrap()));

        // Now that the briefing and all that is done,
        // the influence structure can be initialized for
        // the new mission:
        self.vars.me.ty = Droid::Droid001 as c_int;
        self.vars.me.speed.x = 0.;
        self.vars.me.speed.y = 0.;
        self.vars.me.energy = self.vars.droidmap[Droid::Droid001 as usize].maxenergy;
        self.vars.me.health = self.vars.me.energy; /* start with max. health */
        self.vars.me.status = Status::Mobile as c_int;
        self.vars.me.phase = 0.;
        self.vars.me.timer = 0.0; // set clock to 0

        info!("done."); // this matches the printf at the beginning of this function
    }

    ///  This function does the mission briefing.  It assumes,
    ///  that a mission file has already been successfully loaded into
    ///  memory.  The briefing texts will be extracted and displayed in
    ///  scrolling font.
    pub fn title(&mut self, mission_briefing_data: &[u8]) {
        const BRIEFING_TITLE_PICTURE_STRING: &[u8] =
            b"The title picture in the graphics subdirectory for this mission is : ";
        const BRIEFING_TITLE_SONG_STRING: &[u8] =
            b"The title song in the sound subdirectory for this mission is : ";
        const NEXT_BRIEFING_SUBSECTION_START_STRING: &[u8] =
            b"* New Mission Briefing Text Subsection *";
        const END_OF_BRIEFING_SUBSECTION_STRING: &[u8] =
            b"* End of Mission Briefing Text Subsection *";

        let song_title = read_string_from_string(mission_briefing_data, BRIEFING_TITLE_SONG_STRING);
        self.switch_background_music_to(Some(song_title));

        self.graphics.ne_screen.as_mut().unwrap().clear_clip_rect();
        let pic_title =
            read_string_from_string(mission_briefing_data, BRIEFING_TITLE_PICTURE_STRING);
        let image = Self::find_file_static(
            &self.global,
            &mut self.misc,
            pic_title,
            Some(GRAPHICS_DIR_C),
            Themed::NoTheme as c_int,
            Criticality::Critical as c_int,
        )
        .unwrap();
        Self::display_image(self.sdl, &self.global, &mut self.graphics, image);
        self.make_grid_on_screen(Some(&self.vars.screen_rect.clone()));
        self.vars.me.status = Status::Briefing as c_int;

        self.b_font.current_font = self.global.para_b_font.clone();

        self.display_banner(None, None, DisplayBannerFlags::FORCE_UPDATE.bits().into());

        // Next we display all the subsections of the briefing section
        // with scrolling font
        let next_subsection_data = mission_briefing_data;
        while let Some(pos) = next_subsection_data.find(NEXT_BRIEFING_SUBSECTION_START_STRING) {
            let next_subsection_data =
                &next_subsection_data[(pos + NEXT_BRIEFING_SUBSECTION_START_STRING.len())..];
            let this_text_length = next_subsection_data
                .find(END_OF_BRIEFING_SUBSECTION_STRING)
                .expect("Title: Unterminated Subsection in Mission briefing....Terminating...");

            let mut rect = self.vars.full_user_rect;
            rect.inc_x(10);
            rect.dec_width(10); //leave some border
            if self.scroll_text(&next_subsection_data[..this_text_length], &mut rect, 0) == 1 {
                break; // User pressed 'fire'
            }
        }
    }

    /// This function loads all the constant variables of the game from
    /// a dat file, that should be optimally human readable.
    pub fn init_game_data(&mut self, data_filename: &[u8]) {
        const END_OF_GAME_DAT_STRING: &[u8] = b"*** End of game.dat File ***";

        /* Read the whole game data to memory */
        let fpath = self
            .find_file(
                data_filename,
                Some(MAP_DIR_C),
                Themed::NoTheme as c_int,
                Criticality::Critical as c_int,
            )
            .unwrap();
        let fpath = Path::new(
            fpath
                .to_str()
                .expect("unable to convert C string to UTF-8 string"),
        );

        let data = read_and_malloc_and_terminate_file(fpath, END_OF_GAME_DAT_STRING);

        self.get_general_game_constants(&data);
        self.get_robot_data(&data);
        self.get_bullet_data(&data);

        // Now we read in the total time amount for the blast animations
        const BLAST_ONE_TOTAL_AMOUNT_OF_TIME_STRING: &[u8] =
            b"Time in seconds for the animation of blast one :";
        const BLAST_TWO_TOTAL_AMOUNT_OF_TIME_STRING: &[u8] =
            b"Time in seconds for the animation of blast one :";

        self.vars.blastmap[0].total_animation_time =
            read_float_from_string(&data, BLAST_ONE_TOTAL_AMOUNT_OF_TIME_STRING);
        self.vars.blastmap[1].total_animation_time =
            read_float_from_string(&data, BLAST_TWO_TOTAL_AMOUNT_OF_TIME_STRING);
    }

    /// This function loads all the constant variables of the game from
    /// a dat file, that should be optimally human readable.
    pub fn get_robot_data(&mut self, data_slice: &[u8]) {
        const MAXSPEED_CALIBRATOR_STRING: &[u8] = b"Common factor for all droids maxspeed values: ";
        const ACCELERATION_CALIBRATOR_STRING: &[u8] =
            b"Common factor for all droids acceleration values: ";
        const MAXENERGY_CALIBRATOR_STRING: &[u8] =
            b"Common factor for all droids maximum energy values: ";
        const ENERGYLOSS_CALIBRATOR_STRING: &[u8] =
            b"Common factor for all droids energyloss values: ";
        const AGGRESSION_CALIBRATOR_STRING: &[u8] =
            b"Common factor for all droids aggression values: ";
        const SCORE_CALIBRATOR_STRING: &[u8] = b"Common factor for all droids score values: ";

        const ROBOT_SECTION_BEGIN_STRING: &[u8] = b"*** Start of Robot Data Section: ***";
        // const ROBOT_SECTION_END_STRING: &CStr = cstr!("*** End of Robot Data Section: ***");
        const NEW_ROBOT_BEGIN_STRING: &[u8] = b"** Start of new Robot: **";
        const DROIDNAME_BEGIN_STRING: &[u8] = b"Droidname: ";
        const MAXSPEED_BEGIN_STRING: &[u8] = b"Maximum speed of this droid: ";
        const CLASS_BEGIN_STRING: &[u8] = b"Class of this droid: ";
        const ACCELERATION_BEGIN_STRING: &[u8] = b"Maximum acceleration of this droid: ";
        const MAXENERGY_BEGIN_STRING: &[u8] = b"Maximum energy of this droid: ";
        const LOSEHEALTH_BEGIN_STRING: &[u8] = b"Rate of energyloss under influence control: ";
        const GUN_BEGIN_STRING: &[u8] = b"Weapon type this droid uses: ";
        const AGGRESSION_BEGIN_STRING: &[u8] = b"Aggression rate of this droid: ";
        const FLASHIMMUNE_BEGIN_STRING: &[u8] = b"Is this droid immune to disruptor blasts? ";
        const SCORE_BEGIN_STRING: &[u8] = b"Score gained for destroying one of this type: ";
        const HEIGHT_BEGIN_STRING: &[u8] = b"Height of this droid : ";
        const WEIGHT_BEGIN_STRING: &[u8] = b"Weight of this droid : ";
        const DRIVE_BEGIN_STRING: &[u8] = b"Drive of this droid : ";
        const BRAIN_BEGIN_STRING: &[u8] = b"Brain of this droid : ";
        const SENSOR1_BEGIN_STRING: &[u8] = b"Sensor 1 of this droid : ";
        const SENSOR2_BEGIN_STRING: &[u8] = b"Sensor 2 of this droid : ";
        const SENSOR3_BEGIN_STRING: &[u8] = b"Sensor 3 of this droid : ";
        // const ADVANCED_FIGHTING_BEGIN_STRING: &CStr =
        //     cstr!("Advanced Fighting present in this droid : ");
        // const GO_REQUEST_REINFORCEMENTS_BEGIN_STRING: &CStr =
        //     cstr!("Going to request reinforcements typical for this droid : ");
        const NOTES_BEGIN_STRING: &[u8] = b"Notes concerning this droid : ";

        let mut robot_slice =
            &data_slice[locate_string_in_data(data_slice, ROBOT_SECTION_BEGIN_STRING)..];

        info!("Starting to read robot calibration section");

        // Now we read in the speed calibration factor for all droids
        let maxspeed_calibrator = read_float_from_string(robot_slice, MAXSPEED_CALIBRATOR_STRING);

        // Now we read in the acceleration calibration factor for all droids
        let acceleration_calibrator =
            read_float_from_string(robot_slice, ACCELERATION_CALIBRATOR_STRING);

        // Now we read in the maxenergy calibration factor for all droids
        let maxenergy_calibrator = read_float_from_string(robot_slice, MAXENERGY_CALIBRATOR_STRING);

        // Now we read in the energy_loss calibration factor for all droids
        let energyloss_calibrator =
            read_float_from_string(robot_slice, ENERGYLOSS_CALIBRATOR_STRING);

        // Now we read in the aggression calibration factor for all droids
        let aggression_calibrator =
            read_float_from_string(robot_slice, AGGRESSION_CALIBRATOR_STRING);

        // Now we read in the score calibration factor for all droids
        let score_calibrator = read_float_from_string(robot_slice, SCORE_CALIBRATOR_STRING);

        info!("Starting to read Robot data...");

        // cleanup if previously allocated:
        self.free_druidmap();

        // At first, we must allocate memory for the droid specifications.
        // How much?  That depends on the number of droids defined in freedroid.ruleset.
        // So we have to count those first.  ok.  lets do it.
        self.main.number_of_droid_types =
            count_string_occurences(data_slice, NEW_ROBOT_BEGIN_STRING)
                .try_into()
                .unwrap();

        // Now that we know how many robots are defined in freedroid.ruleset, we can allocate
        // a fitting amount of memory.
        self.vars
            .droidmap
            .reserve(self.main.number_of_droid_types.into());
        info!(
            "We have counted {} different druid types in the game data file.",
            self.main.number_of_droid_types,
        );
        info!("MEMORY HAS BEEN ALLOCATED. THE READING CAN BEGIN.");

        //Now we start to read the values for each robot:
        //Of which parts is it composed, which stats does it have?
        loop {
            robot_slice = match robot_slice.find(NEW_ROBOT_BEGIN_STRING) {
                Some(pos) => &robot_slice[(pos + 1)..],
                None => break,
            };

            info!("Found another Robot specification entry!  Lets add that to the others!");

            // Now we read in the Name of this droid.  We consider as a name the rest of the
            let mut droid = DruidSpec::default();
            droid
                .druidname
                .set_slice(read_string_from_string(robot_slice, DROIDNAME_BEGIN_STRING));

            // Now we read in the maximal speed this droid can go.
            droid.maxspeed = read_float_from_string(robot_slice, MAXSPEED_BEGIN_STRING);

            // Now we read in the class of this droid.
            droid.class = read_i32_from_string(robot_slice, CLASS_BEGIN_STRING);

            // Now we read in the maximal acceleration this droid can go.
            droid.accel = read_float_from_string(robot_slice, ACCELERATION_BEGIN_STRING);

            // Now we read in the maximal energy this droid can store.
            droid.maxenergy = read_float_from_string(robot_slice, MAXENERGY_BEGIN_STRING);

            // Now we read in the lose_health rate.
            droid.lose_health = read_float_from_string(robot_slice, LOSEHEALTH_BEGIN_STRING);

            // Now we read in the class of this droid.
            droid.gun = read_i32_from_string(robot_slice, GUN_BEGIN_STRING);

            // Now we read in the aggression rate of this droid.
            droid.aggression = read_i32_from_string(robot_slice, AGGRESSION_BEGIN_STRING);

            // Now we read in the flash immunity of this droid.
            droid.flashimmune = read_i32_from_string(robot_slice, FLASHIMMUNE_BEGIN_STRING);

            // Now we score to be had for destroying one droid of this type
            droid.score = read_i32_from_string(robot_slice, SCORE_BEGIN_STRING);

            // Now we read in the height of this droid of this type
            droid.height = read_float_from_string(robot_slice, HEIGHT_BEGIN_STRING);

            // Now we read in the weight of this droid type
            droid.weight = read_i32_from_string(robot_slice, WEIGHT_BEGIN_STRING);

            // Now we read in the drive of this droid of this type
            droid.drive = read_i32_from_string(robot_slice, DRIVE_BEGIN_STRING);

            // Now we read in the brain of this droid of this type
            droid.brain = read_i32_from_string(robot_slice, BRAIN_BEGIN_STRING);

            // Now we read in the sensor 1, 2 and 3 of this droid type
            droid.sensor1 = read_i32_from_string(robot_slice, SENSOR1_BEGIN_STRING);
            droid.sensor2 = read_i32_from_string(robot_slice, SENSOR2_BEGIN_STRING);
            droid.sensor3 = read_i32_from_string(robot_slice, SENSOR3_BEGIN_STRING);

            // Now we read in the notes concerning this droid.  We consider as notes all the rest of the
            // line after the NOTES_BEGIN_STRING until the "\n" is found.
            droid.notes = read_and_malloc_string_from_data(robot_slice, NOTES_BEGIN_STRING, b"\n");

            self.vars.droidmap.push(droid);
        }

        info!("That must have been the last robot.  We're done reading the robot data.");
        info!("Applying the calibration factors to all droids...");

        for droid in &mut self.vars.droidmap {
            droid.maxspeed *= maxspeed_calibrator;
            droid.accel *= acceleration_calibrator;
            droid.maxenergy *= maxenergy_calibrator;
            droid.lose_health *= energyloss_calibrator;
            droid.aggression = (droid.aggression as f32 * aggression_calibrator) as c_int;
            droid.score = (droid.score as f32 * score_calibrator) as c_int;
        }
    }

    /// This function reads in all the bullet data from the freedroid.ruleset file,
    /// but IT DOES NOT LOAD THE FILE, IT ASSUMES IT IS ALREADY LOADED and
    /// it only receives a pointer to the start of the bullet section from
    /// the calling function.
    pub fn get_bullet_data(&mut self, data_slice: &[u8]) {
        // const BULLET_SECTION_BEGIN_STRING: &CStr = cstr!("*** Start of Bullet Data Section: ***");
        // const BULLET_SECTION_END_STRING: &CStr = cstr!("*** End of Bullet Data Section: ***");
        const NEW_BULLET_TYPE_BEGIN_STRING: &[u8] =
            b"** Start of new bullet specification subsection **";

        const BULLET_RECHARGE_TIME_BEGIN_STRING: &[u8] =
            b"Time is takes to recharge this bullet/weapon in seconds :";
        const BULLET_SPEED_BEGIN_STRING: &[u8] = b"Flying speed of this bullet type :";
        const BULLET_DAMAGE_BEGIN_STRING: &[u8] = b"Damage cause by a hit of this bullet type :";
        // #define BULLET_NUMBER_OF_PHASES_BEGIN_STRING "Number of different phases that were designed for this bullet type :"
        // const BULLET_ONE_SHOT_ONLY_AT_A_TIME: &CStr =
        //     cstr!("Cannot fire until previous bullet has been deleted : ");
        const BULLET_BLAST_TYPE_CAUSED_BEGIN_STRING: &[u8] =
            b"Type of blast this bullet causes when crashing e.g. against a wall :";

        const BULLET_SPEED_CALIBRATOR_STRING: &[u8] =
            b"Common factor for all bullet's speed values: ";
        const BULLET_DAMAGE_CALIBRATOR_STRING: &[u8] =
            b"Common factor for all bullet's damage values: ";

        info!("Starting to read bullet data...");
        //--------------------
        // At first, we must allocate memory for the droid specifications.
        // How much?  That depends on the number of droids defined in freedroid.ruleset.
        // So we have to count those first.  ok.  lets do it.

        self.graphics.number_of_bullet_types =
            count_string_occurences(data_slice, NEW_BULLET_TYPE_BEGIN_STRING)
                .try_into()
                .unwrap();

        // Now that we know how many bullets are defined in freedroid.ruleset, we can allocate
        // a fitting amount of memory, but of course only if the memory hasn't been allocated
        // aready!!!
        //
        // If we would do that in any case, every Init_Game_Data call would destroy the loaded
        // image files AND MOST LIKELY CAUSE A SEGFAULT!!!
        //
        if self.vars.bulletmap.is_empty() {
            self.vars
                .bulletmap
                .reserve(usize::try_from(self.graphics.number_of_bullet_types).unwrap());
            info!(
                "We have counted {} different bullet types in the game data file.",
                self.graphics.number_of_bullet_types
            );
            info!("MEMORY HAS BEEN ALLOCATED. THE READING CAN BEGIN.");
        }

        //--------------------
        // Now we start to read the values for each bullet type:
        //
        let mut bullet_slice = data_slice;
        loop {
            bullet_slice = match bullet_slice.find(NEW_BULLET_TYPE_BEGIN_STRING) {
                Some(pos) => &bullet_slice[(pos + 1)..],
                None => break,
            };

            info!("Found another Bullet specification entry!  Lets add that to the others!");

            // Now we read in the recharging time for this bullettype(=weapontype)
            let recharging_time =
                read_float_from_string(bullet_slice, BULLET_RECHARGE_TIME_BEGIN_STRING);

            // Now we read in the maximal speed this type of bullet can go.
            let speed = read_float_from_string(bullet_slice, BULLET_SPEED_BEGIN_STRING);

            // Now we read in the damage this bullet can do
            let damage = read_i32_from_string(bullet_slice, BULLET_DAMAGE_BEGIN_STRING);

            // Now we read in the number of phases that are designed for this bullet type
            // THIS IS NOW SPECIFIED IN THE THEME CONFIG FILE
            // ReadValueFromString( BulletPointer ,  BULLET_NUMBER_OF_PHASES_BEGIN_STRING , "%d" ,
            // &(*Bulletmap.add(BulletIndex)).phases , EndOfBulletData );

            // Now we read in the type of blast this bullet will cause when crashing e.g. against the wall
            let blast = read_i32_from_string(bullet_slice, BULLET_BLAST_TYPE_CAUSED_BEGIN_STRING);

            self.vars.bulletmap.push(BulletSpec {
                recharging_time,
                speed,
                damage,
                blast,
                ..Default::default()
            });
        }

        //--------------------
        // Now that the detailed values for the bullets have been read in,
        // we now read in the general calibration contants and after that
        // the start to apply them right now, so they also take effect.

        info!("Starting to read bullet calibration section");

        // Now we read in the speed calibration factor for all bullets
        let bullet_speed_calibrator =
            read_float_from_string(data_slice, BULLET_SPEED_CALIBRATOR_STRING);

        // Now we read in the damage calibration factor for all bullets
        let bullet_damage_calibrator =
            read_float_from_string(data_slice, BULLET_DAMAGE_CALIBRATOR_STRING);

        // Now that all the calibrations factors have been read in, we can start to
        // apply them to all the bullet types
        for bullet in &mut self.vars.bulletmap {
            bullet.speed *= bullet_speed_calibrator;
            bullet.damage = (bullet.damage as f32 * bullet_damage_calibrator) as c_int;
        }
    }

    /// This function loads all the constant variables of the game from
    /// a dat file, that should be optimally human readable.
    pub fn get_general_game_constants(&mut self, data: &[u8]) {
        // const CONSTANTS_SECTION_BEGIN_STRING: &CStr =
        //     cstr!("*** Start of General Game Constants Section: ***");
        // const CONSTANTS_SECTION_END_STRING: &CStr =
        //     cstr!("*** End of General Game Constants Section: ***");
        const COLLISION_LOSE_ENERGY_CALIBRATOR_STRING: &[u8] =
            b"Energy-Loss-factor for Collisions of Influ with hostile robots=";
        const BLAST_RADIUS_SPECIFICATION_STRING: &[u8] =
            b"Radius of explosions (as far as damage is concerned) in multiples of tiles=";
        const DROID_RADIUS_SPECIFICATION_STRING: &[u8] = b"Droid radius:";
        const BLAST_DAMAGE_SPECIFICATION_STRING: &[u8] =
            b"Amount of damage done by contact to a blast per second of time=";
        const TIME_FOR_DOOR_MOVEMENT_SPECIFICATION_STRING: &[u8] =
            b"Time for the doors to move by one subphase of their movement=";

        const DEATHCOUNT_DRAIN_SPEED_STRING: &[u8] = b"Deathcount drain speed =";
        const ALERT_THRESHOLD_STRING: &[u8] = b"First alert threshold =";
        const ALERT_BONUS_PER_SEC_STRING: &[u8] = b"Alert bonus per second =";

        info!("Starting to read contents of General Game Constants section");

        // read in Alert-related parameters:
        self.main.death_count_drain_speed =
            read_float_from_string(data, DEATHCOUNT_DRAIN_SPEED_STRING);
        self.main.alert_threshold = read_i32_from_string(data, ALERT_THRESHOLD_STRING);
        self.main.alert_bonus_per_sec = read_float_from_string(data, ALERT_BONUS_PER_SEC_STRING);

        // Now we read in the speed calibration factor for all bullets
        self.global.collision_lose_energy_calibrator =
            read_float_from_string(data, COLLISION_LOSE_ENERGY_CALIBRATOR_STRING);

        // Now we read in the blast radius
        self.global.blast_radius = read_float_from_string(data, BLAST_RADIUS_SPECIFICATION_STRING);

        // Now we read in the druid 'radius' in x direction
        self.global.droid_radius = read_float_from_string(data, DROID_RADIUS_SPECIFICATION_STRING);

        // Now we read in the blast damage amount per 'second' of contact with the blast
        self.global.blast_damage_per_second =
            read_float_from_string(data, BLAST_DAMAGE_SPECIFICATION_STRING);

        // Now we read in the time is takes for the door to move one phase
        self.global.time_for_each_phase_of_door_movement =
            read_float_from_string(data, TIME_FOR_DOOR_MOVEMENT_SPECIFICATION_STRING);
    }

    /// Show end-screen
    pub(crate) fn thou_art_defeated(&mut self) {
        self.vars.me.status = Status::Terminated as c_int;
        self.sdl.cursor().hide();

        self.explode_influencer();

        self.wait_for_all_keys_released();

        let mut now = self.sdl.ticks_ms();

        while (self.sdl.ticks_ms() - now) < WAIT_AFTER_KILLED {
            // add "slow motion effect" for final explosion
            self.set_time_factor(SLOWMO_FACTOR);

            self.start_taking_time_for_fps_calculation();
            self.display_banner(None, None, 0);
            self.explode_blasts();
            self.move_bullets();
            self.move_enemys();
            self.assemble_combat_picture(AssembleCombatWindowFlags::DO_SCREEN_UPDATE.bits().into());
            self.compute_fps_for_this_frame();
            if self.any_key_just_pressed() != 0 {
                break;
            }
        }
        self.set_time_factor(1.0);

        self.sdl.mixer.get().unwrap().halt_music();

        // important!!: don't forget to stop fps calculation here (bugfix: enemy piles after gameOver)
        self.activate_conservative_frame_computation();

        // TODO: avoid a temporary backup
        let mut user_rect = std::mem::take(&mut self.vars.user_rect);
        let mut ne_screen = self.graphics.ne_screen.take().unwrap();
        self.white_noise(
            &mut ne_screen,
            &mut user_rect,
            WAIT_AFTER_KILLED.try_into().unwrap(),
        );
        self.vars.user_rect = user_rect;
        self.graphics.ne_screen = Some(ne_screen);

        self.assemble_combat_picture(AssembleCombatWindowFlags::DO_SCREEN_UPDATE.bits().into());
        self.make_grid_on_screen(Some(&self.vars.user_rect.clone()));

        let mut dst = self.vars.portrait_rect.with_xy(
            self.vars.get_user_center().x()
                - i16::try_from(self.vars.portrait_rect.width() / 2).unwrap(),
            self.vars.get_user_center().y()
                - i16::try_from(self.vars.portrait_rect.height() / 2).unwrap(),
        );
        let Graphics {
            pic999, ne_screen, ..
        } = &mut self.graphics;
        pic999
            .as_mut()
            .unwrap()
            .blit_to(ne_screen.as_mut().unwrap(), &mut dst);
        self.thou_art_defeated_sound();

        self.b_font.current_font = self.global.para_b_font.clone();
        let h = font_height(
            self.global
                .para_b_font
                .as_deref()
                .unwrap()
                .ro(&self.font_owner),
        );
        self.display_text(
            b"Transmission",
            i32::from(dst.x()) - h,
            i32::from(dst.y()) - h,
            Some(self.vars.user_rect),
        );
        self.display_text(
            b"Terminated",
            i32::from(dst.x()) - h,
            i32::from(dst.y()) + i32::from(dst.height()),
            Some(self.vars.user_rect),
        );
        let mut ne_screen = self.graphics.ne_screen.take().unwrap();
        self.printf_sdl(&mut ne_screen, -1, -1, format_args!("\n"));
        assert!(ne_screen.flip());
        self.graphics.ne_screen = Some(ne_screen);

        now = self.sdl.ticks_ms();

        self.wait_for_all_keys_released();
        while self.sdl.ticks_ms() - now < SHOW_WAIT {
            self.sdl.delay_ms(1);
            if self.any_key_just_pressed() != 0 {
                break;
            }
        }

        self.update_highscores();

        self.game_over = true;
    }
}
