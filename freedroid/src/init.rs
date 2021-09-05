use crate::{
    b_font::font_height,
    defs::{
        self, scale_rect, AssembleCombatWindowFlags, Criticality, DisplayBannerFlags, Droid,
        Status, Themed, FD_DATADIR, GRAPHICS_DIR_C, LOCAL_DATADIR, MAP_DIR_C, MAXBULLETS,
        SHOW_WAIT, SLOWMO_FACTOR, TITLE_PIC_FILE_C, WAIT_AFTER_KILLED,
    },
    global::Global,
    graphics::Graphics,
    misc::{
        count_string_occurences, dealloc_c_string, locate_string_in_data, my_random,
        read_and_malloc_string_from_data, read_value_from_string,
    },
    structs::{BulletSpec, DruidSpec},
    Data,
};

#[cfg(target_os = "windows")]
use crate::input::wait_for_key_pressed;

use clap::{crate_version, Clap};
use cstr::cstr;
use log::{error, info, warn};
use sdl_sys::{
    Mix_HaltMusic, SDL_Delay, SDL_Flip, SDL_FreeSurface, SDL_GetTicks, SDL_Rect, SDL_SetClipRect,
    SDL_ShowCursor, SDL_UpperBlit, SDL_DISABLE,
};
use std::{
    alloc::{alloc_zeroed, dealloc, Layout},
    convert::{TryFrom, TryInto},
    ffi::CStr,
    ops::Not,
    os::raw::{c_char, c_float, c_int, c_long, c_uint, c_void},
    path::Path,
    ptr::null_mut,
};

#[derive(Debug)]
pub struct Init {
    debriefing_text: *mut c_char,
    debriefing_song: [c_char; 500],
    previous_mission_name: [c_char; 500],
}

impl Default for Init {
    fn default() -> Self {
        Self {
            debriefing_text: null_mut(),
            debriefing_song: [0; 500],
            previous_mission_name: [0; 500],
        }
    }
}

const MISSION_COMPLETE_BONUS: f32 = 1000.;
const COPYRIGHT: &str = "\nCopyright (C) 2003-2018 Johannes Prix, Reinhard Prix\n\
Freedroid comes with NO WARRANTY to the extent permitted by law.\n\
You may redistribute copies of Freedroid under the terms of the\n\
GNU General Public License.\n\
For more information about these matters, see the file named COPYING.";

impl Data {
    pub unsafe fn free_game_mem(&mut self) {
        // free bullet map
        if self.vars.bulletmap.is_null().not() {
            let bullet_map = std::slice::from_raw_parts_mut(
                self.vars.bulletmap,
                usize::try_from(self.graphics.number_of_bullet_types).unwrap(),
            );
            for bullet in bullet_map {
                for surface in &bullet.surface_pointer {
                    SDL_FreeSurface(*surface);
                }
            }
            dealloc(
                self.vars.bulletmap as *mut u8,
                Layout::array::<BulletSpec>(
                    usize::try_from(self.graphics.number_of_bullet_types).unwrap(),
                )
                .unwrap(),
            );
            self.vars.bulletmap = null_mut();
        }

        // free blast map
        for blast_type in &mut self.vars.blastmap {
            for surface in &mut blast_type.surface_pointer {
                SDL_FreeSurface(*surface);
                *surface = null_mut();
            }
        }

        // free droid map
        self.free_druidmap();

        // free highscores list
        drop(self.highscore.entries.take());

        // free constant text blobs
        dealloc_c_string(self.init.debriefing_text);
        self.init.debriefing_text = null_mut();
    }

    pub unsafe fn free_druidmap(&mut self) {
        if self.vars.droidmap.is_null() {
            return;
        }
        let droid_map = std::slice::from_raw_parts(
            self.vars.droidmap,
            usize::try_from(self.main.number_of_droid_types).unwrap(),
        );
        for droid in droid_map {
            dealloc_c_string(droid.notes);
        }

        dealloc(
            self.vars.droidmap as *mut u8,
            Layout::array::<DruidSpec>(usize::try_from(self.main.number_of_droid_types).unwrap())
                .unwrap(),
        );
        self.vars.droidmap = null_mut();
    }
}

/// put some ideology message for our poor friends enslaved by M$-Win32 ;)
#[cfg(target_os = "windows")]
pub unsafe fn win32_disclaimer() {
    SDL_SetClipRect(self.graphics.ne_screen, null_mut());
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
    SDL_Flip(self.graphics.ne_screen);

    wait_for_key_pressed();
}

impl Data {
    /// This function checks, if the influencer has succeeded in his given
    /// mission.  If not it returns, if yes the Debriefing is started.
    pub(crate) unsafe fn check_if_mission_is_complete(&mut self) {
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

    pub unsafe fn thou_art_victorious(&mut self) {
        self.switch_background_music_to(self.init.debriefing_song.as_ptr());

        SDL_ShowCursor(SDL_DISABLE);

        self.main.show_score = self.main.real_score as c_long;
        self.vars.me.status = Status::Victory as c_int;
        self.display_banner(
            null_mut(),
            null_mut(),
            DisplayBannerFlags::FORCE_UPDATE.bits().into(),
        );

        self.wait_for_all_keys_released();

        let now = SDL_GetTicks();

        while SDL_GetTicks() - now < WAIT_AFTER_KILLED {
            self.display_banner(null_mut(), null_mut(), 0);
            self.explode_blasts();
            self.move_bullets();
            self.assemble_combat_picture(AssembleCombatWindowFlags::DO_SCREEN_UPDATE.bits().into());
        }

        let mut rect = self.vars.full_user_rect;
        SDL_SetClipRect(self.graphics.ne_screen, null_mut());
        self.make_grid_on_screen(Some(&rect));
        SDL_Flip(self.graphics.ne_screen);
        rect.x += 10;
        rect.w -= 20; //leave some border
        self.b_font.current_font = self.global.para_b_font;
        self.scroll_text(self.init.debriefing_text, &mut rect, 6);

        self.wait_for_all_keys_released();
    }

    /// This function initializes the whole Freedroid game.
    ///
    /// This must not be confused with initnewgame, which
    /// only initializes a new mission for the game.
    pub unsafe fn init_freedroid(&mut self) {
        self.vars.bulletmap = null_mut(); // That will cause the memory to be allocated later

        for bullet in &mut self.main.all_bullets[..MAXBULLETS] {
            bullet.surfaces_were_generated = false.into();
        }

        self.global.skip_a_few_frames = false.into();
        self.vars.me.text_visible_time = 0.;
        self.vars.me.text_to_be_displayed = null_mut();

        // these are the hardcoded game-defaults, they can be overloaded by the config-file if present
        self.global.game_config.current_bg_music_volume = 0.3;
        self.global.game_config.current_sound_fx_volume = 0.5;

        self.global.game_config.wanted_text_visible_time = 3.;
        self.global.game_config.droid_talk = false.into();

        self.global.game_config.draw_framerate = false.into();
        self.global.game_config.draw_energy = false.into();
        self.global.game_config.draw_death_count = false.into();
        self.global.game_config.draw_position = false.into();

        std::ptr::copy_nonoverlapping(
            b"classic\0".as_ptr(),
            self.global.game_config.theme_name.as_mut_ptr() as *mut u8,
            b"classic\0".len(),
        );
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

        scale_rect(&mut self.vars.screen_rect, self.global.game_config.scale); // make sure we open a window of the right (rescaled) size!
        self.init_video();

        let image = self.find_file(
            TITLE_PIC_FILE_C.as_ptr() as *mut c_char,
            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
            Themed::NoTheme as c_int,
            Criticality::Critical as c_int,
        );
        self.display_image(image); // show title pic
        SDL_Flip(self.graphics.ne_screen);

        self.load_fonts(); // we need this for progress-meter!

        self.init_progress(cstr!("Loading Freedroid").as_ptr() as *mut c_char);

        self.find_all_themes(); // put all found themes into a list: AllThemes[]

        self.update_progress(5);

        self.init_audio();

        self.init_joy();

        self.init_game_data(cstr!("freedroid.ruleset").as_ptr() as *mut c_char); // load the default ruleset. This can be */
                                                                                 // overwritten from the mission file.

        self.update_progress(10);

        // The default should be, that no rescaling of the
        // combat window at all is done.
        self.global.current_combat_scale_factor = 1.;

        /*
         * Initialise random-number generator in order to make
         * level-start etc really different at each program start
         */
        libc::srand(SDL_GetTicks() as c_uint);

        /* initialize/load the highscore list */
        self.init_highscores();

        /* Now fill the pictures correctly to the structs */
        if self.init_pictures() == 0 {
            panic!("Error in InitPictures reported back...");
        }

        self.update_progress(100); // finished init
    }
}

#[derive(Clap)]
#[clap(version = crate_version!(), long_version = COPYRIGHT)]
struct Opt {
    #[clap(short, long)]
    _version: bool,

    #[clap(short, long, conflicts_with = "nosound")]
    sound: bool,

    #[clap(short = 'q', long, conflicts_with = "sound")]
    nosound: bool,

    #[clap(short, long, parse(from_occurrences))]
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

impl Data {
    /// parse command line arguments and set global switches
    /// exit on error, so we don't need to return success status
    unsafe fn parse_command_line(&mut self) {
        let opt = Opt::parse();

        if opt.nosound {
            self.main.sound_on = false.into();
        } else if opt.sound {
            self.main.sound_on = true.into();
        }

        if let Some(sensitivity) = opt.sensitivity {
            if sensitivity > 32 {
                panic!("\nJoystick sensitivity must lie in the range [0;32]");
            }

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
            if scale <= 0. {
                panic!("illegal scale entered, needs to be >0: {}", scale);
            }
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
    pub unsafe fn find_all_themes(&mut self) {
        use std::fs;

        let mut classic_theme_index: usize = 0; // default: override when we actually find 'classic' theme

        // just to make sure...
        self.graphics.all_themes.num_themes = 0;
        self.graphics
            .all_themes
            .theme_name
            .iter_mut()
            .filter(|name| name.is_null().not())
            .for_each(|name| {
                dealloc_c_string(*name as *mut i8);
                *name = null_mut();
            });

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
                                .copied()
                                .filter(|theme| theme.is_null().not())
                                .filter_map(|theme| {
                                    CStr::from_ptr(theme as *const c_char).to_str().ok()
                                })
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
                                let new_theme = &mut self.graphics.all_themes.theme_name
                                    [usize::try_from(self.graphics.all_themes.num_themes).unwrap()];
                                *new_theme = alloc_zeroed(
                                    Layout::array::<u8>(theme_name.len() + 1).unwrap(),
                                ) as *mut u8;
                                std::ptr::copy_nonoverlapping(
                                    theme_name.as_ptr(),
                                    *new_theme,
                                    theme_name.len(),
                                );
                                *new_theme.add(theme_name.len()) = b'\0';

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
        if self.graphics.all_themes.num_themes == 0 {
            panic!("No valid graphic-themes found!! You need to install at least one to run Freedroid!!");
        }

        let Self {
            graphics: Graphics { all_themes, .. },
            global: Global { game_config, .. },
            ..
        } = self;
        let selected_theme_index = all_themes.theme_name
            [..usize::try_from(all_themes.num_themes).unwrap()]
            .iter()
            .copied()
            .position(|theme_name| {
                libc::strcmp(theme_name as *const _, game_config.theme_name.as_mut_ptr()) == 0
            });

        match selected_theme_index {
            Some(index) => {
                info!(
                    "Found selected theme {} from GameConfig.",
                    CStr::from_ptr(self.global.game_config.theme_name.as_ptr()).to_string_lossy(),
                );
                self.graphics.all_themes.cur_tnum = index.try_into().unwrap();
            }
            None => {
                warn!(
                    "selected theme {} not valid! Using classic theme.",
                    CStr::from_ptr(self.global.game_config.theme_name.as_ptr()).to_string_lossy(),
                );
                libc::strcpy(
                    self.global.game_config.theme_name.as_mut_ptr(),
                    self.graphics.all_themes.theme_name[classic_theme_index] as *const _,
                );
                self.graphics.all_themes.cur_tnum = classic_theme_index.try_into().unwrap();
            }
        }

        info!(
            "Game starts using theme: {}",
            CStr::from_ptr(self.global.game_config.theme_name.as_ptr()).to_string_lossy()
        );
    }

    pub unsafe fn init_new_mission(&mut self, mission_name: *mut c_char) {
        const END_OF_MISSION_DATA_STRING: &CStr = cstr!("*** End of Mission File ***");
        const MISSION_BRIEFING_BEGIN_STRING: &CStr =
            cstr!("** Start of Mission Briefing Text Section **");
        const MISSION_ENDTITLE_SONG_NAME_STRING: &CStr =
            cstr!("Song name to play in the end title if the mission is completed: ");
        const SHIPNAME_INDICATION_STRING: &CStr = cstr!("Ship file to use for this mission: ");
        const ELEVATORNAME_INDICATION_STRING: &CStr = cstr!("Lift file to use for this mission: ");
        const CREWNAME_INDICATION_STRING: &CStr = cstr!("Crew file to use for this mission: ");
        const GAMEDATANAME_INDICATION_STRING: &CStr =
            cstr!("Physics ('game.dat') file to use for this mission: ");
        const MISSION_ENDTITLE_BEGIN_STRING: &CStr =
            cstr!("** Beginning of End Title Text Section **");
        const MISSION_ENDTITLE_END_STRING: &CStr = cstr!("** End of End Title Text Section **");
        const MISSION_START_POINT_STRING: &CStr = cstr!("Possible Start Point : ");

        // We store the mission name in case the influ
        // gets destroyed so we know where to continue in
        // case the player doesn't want to return to the very beginning
        // but just to replay this mission.
        libc::strcpy(self.init.previous_mission_name.as_mut_ptr(), mission_name);

        info!(
            "A new mission is being initialized from file {}.",
            CStr::from_ptr(mission_name).to_string_lossy()
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

        let oldfont = std::mem::replace(&mut self.b_font.current_font, self.global.font0_b_font);

        /* Read the whole mission data to memory */
        let fpath = self.find_file(
            mission_name,
            MAP_DIR_C.as_ptr() as *mut c_char,
            Themed::NoTheme as c_int,
            Criticality::Critical as c_int,
        );

        let mut main_mission_pointer = self.read_and_malloc_and_terminate_file(
            fpath,
            END_OF_MISSION_DATA_STRING.as_ptr() as *mut c_char,
        );

        //--------------------
        // Now the mission file is read into memory.  That means we can start to decode the details given
        // in the body of the mission file.

        //--------------------
        // First we extract the game physics file name from the
        // mission file and load the game data.
        //
        let mut buffer: [c_char; 500] = [0; 500];
        read_value_from_string(
            main_mission_pointer.as_mut_ptr(),
            GAMEDATANAME_INDICATION_STRING.as_ptr() as *mut c_char,
            cstr!("%s").as_ptr() as *mut c_char,
            buffer.as_mut_ptr() as *mut c_void,
        );

        self.init_game_data(buffer.as_mut_ptr());

        //--------------------
        // Now its time to get the shipname from the mission file and
        // read the ship file into the right memory structures
        //
        read_value_from_string(
            main_mission_pointer.as_mut_ptr(),
            SHIPNAME_INDICATION_STRING.as_ptr() as *mut c_char,
            cstr!("%s").as_ptr() as *mut c_char,
            buffer.as_mut_ptr() as *mut c_void,
        );

        if self.load_ship(buffer.as_mut_ptr()) == defs::ERR.into() {
            panic!("Error in LoadShip");
        }
        //--------------------
        // Now its time to get the elevator file name from the mission file and
        // read the elevator file into the right memory structures
        //
        read_value_from_string(
            main_mission_pointer.as_mut_ptr(),
            ELEVATORNAME_INDICATION_STRING.as_ptr() as *mut c_char,
            cstr!("%s").as_ptr() as *mut c_char,
            buffer.as_mut_ptr() as *mut c_void,
        );

        if self.get_lift_connections(buffer.as_mut_ptr()) == defs::ERR.into() {
            panic!("Error in GetLiftConnections");
        }
        //--------------------
        // We also load the comment for the influencer to say at the beginning of the mission
        //

        // NO! these strings are allocated elsewhere or even static, so free'ing them
        // here would SegFault eventually!
        //  if (Me.TextToBeDisplayed) free (Me.TextToBeDisplayed);

        self.vars.me.text_to_be_displayed =
            cstr!("Ok. I'm on board.  Let's get to work.").as_ptr() as *mut c_char; // taken from Paradroid.mission
        self.vars.me.text_visible_time = 0.;

        //--------------------
        // Now its time to get the crew file name from the mission file and
        // assemble an appropriate crew out of it
        //
        read_value_from_string(
            main_mission_pointer.as_mut_ptr(),
            CREWNAME_INDICATION_STRING.as_ptr() as *mut c_char,
            cstr!("%s").as_ptr() as *mut c_char,
            buffer.as_mut_ptr() as *mut c_void,
        );

        /* initialize enemys according to crew file */
        // WARNING!! THIS REQUIRES THE freedroid.ruleset FILE TO BE READ ALREADY, BECAUSE
        // ROBOT SPECIFICATIONS ARE ALREADY REQUIRED HERE!!!!!
        if self.get_crew(buffer.as_mut_ptr()) == defs::ERR.into() {
            panic!("InitNewGame(): Initialization of enemys failed.",);
        }

        //--------------------
        // Now its time to get the debriefing text from the mission file so that it
        // can be used, if the mission is completed and also the end title music name
        // must be read in as well
        read_value_from_string(
            main_mission_pointer.as_mut_ptr(),
            MISSION_ENDTITLE_SONG_NAME_STRING.as_ptr() as *mut c_char,
            cstr!("%s").as_ptr() as *mut c_char,
            self.init.debriefing_song.as_mut_ptr() as *mut c_void,
        );

        if self.init.debriefing_text.is_null().not() {
            dealloc_c_string(self.init.debriefing_text);
        }
        self.init.debriefing_text = read_and_malloc_string_from_data(
            main_mission_pointer.as_mut_ptr(),
            MISSION_ENDTITLE_BEGIN_STRING.as_ptr() as *mut c_char,
            MISSION_ENDTITLE_END_STRING.as_ptr() as *mut c_char,
        );

        //--------------------
        // Now we read all the possible starting points for the
        // current mission file, so that we know where to place the
        // influencer at the beginning of the mission.

        let number_of_start_points = count_string_occurences(
            main_mission_pointer.as_mut_ptr(),
            MISSION_START_POINT_STRING.as_ptr() as *mut c_char,
        );

        if number_of_start_points == 0 {
            panic!("NOT EVEN ONE SINGLE STARTING POINT ENTRY FOUND!  TERMINATING!",);
        }
        info!(
            "Found {} different starting points for the mission in the mission file.",
            number_of_start_points,
        );

        // Now that we know how many different starting points there are, we can randomly select
        // one of them and read then in this one starting point into the right structures...
        let real_start_point = my_random(number_of_start_points - 1) + 1;
        let mut start_point_pointer = main_mission_pointer.as_mut_ptr();
        for _ in 0..real_start_point {
            start_point_pointer = libc::strstr(
                start_point_pointer,
                MISSION_START_POINT_STRING.as_ptr() as *mut c_char,
            );
            start_point_pointer = start_point_pointer.add(libc::strlen(
                MISSION_START_POINT_STRING.as_ptr() as *mut c_char,
            ));
        }
        start_point_pointer = libc::strstr(start_point_pointer, cstr!("Level=").as_ptr())
            .add(libc::strlen(cstr!("Level=").as_ptr()));
        let mut starting_level: c_int = 0;
        let mut starting_x_pos: c_int = 0;
        let mut starting_y_pos: c_int = 0;
        libc::sscanf(
            start_point_pointer,
            cstr!("%d").as_ptr() as *mut c_char,
            &mut starting_level,
        );
        self.main.cur_level =
            self.main.cur_ship.all_levels[usize::try_from(starting_level).unwrap()];
        start_point_pointer = libc::strstr(start_point_pointer, cstr!("XPos=").as_ptr())
            .add(libc::strlen(cstr!("XPos=").as_ptr()));
        libc::sscanf(
            start_point_pointer,
            cstr!("%d").as_ptr() as *mut c_char,
            &mut starting_x_pos,
        );
        self.vars.me.pos.x = starting_x_pos as c_float;
        start_point_pointer = libc::strstr(start_point_pointer, cstr!("YPos=").as_ptr())
            .add(libc::strlen(cstr!("YPos=").as_ptr()));
        libc::sscanf(
            start_point_pointer,
            cstr!("%d").as_ptr() as *mut c_char,
            &mut starting_y_pos,
        );
        self.vars.me.pos.y = starting_y_pos as c_float;
        info!(
            "Final starting position: Level={} XPos={} YPos={}.",
            starting_level, starting_x_pos, starting_y_pos,
        );

        /* Reactivate the light on alle Levels, that might have been dark */
        for &level in &self.main.cur_ship.all_levels
            [0..usize::try_from(self.main.cur_ship.num_levels).unwrap()]
        {
            (*level).empty = false.into();
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
        let briefing_section_pointer = locate_string_in_data(
            main_mission_pointer.as_mut_ptr(),
            MISSION_BRIEFING_BEGIN_STRING.as_ptr() as *mut c_char,
        );
        self.title(briefing_section_pointer);

        /* Den Banner fuer das Spiel anzeigen */
        self.clear_graph_mem();
        self.display_banner(
            null_mut(),
            null_mut(),
            DisplayBannerFlags::FORCE_UPDATE.bits().into(),
        );

        // Switch_Background_Music_To (COMBAT_BACKGROUND_MUSIC_SOUND);
        self.switch_background_music_to((*self.main.cur_level).background_song_name);

        for level_index in 0..usize::try_from(self.main.cur_ship.num_levels).unwrap() {
            self.main.cur_level = self.main.cur_ship.all_levels[level_index];
            self.shuffle_enemys();
        }

        self.main.cur_level =
            self.main.cur_ship.all_levels[usize::try_from(starting_level).unwrap()];

        // Now that the briefing and all that is done,
        // the influence structure can be initialized for
        // the new mission:
        self.vars.me.ty = Droid::Droid001 as c_int;
        self.vars.me.speed.x = 0.;
        self.vars.me.speed.y = 0.;
        self.vars.me.energy = (*self.vars.droidmap.add(Droid::Droid001 as usize)).maxenergy;
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
    pub unsafe fn title(&mut self, mission_briefing_pointer: *mut c_char) {
        const BRIEFING_TITLE_PICTURE_STRING: &CStr =
            cstr!("The title picture in the graphics subdirectory for this mission is : ");
        const BRIEFING_TITLE_SONG_STRING: &CStr =
            cstr!("The title song in the sound subdirectory for this mission is : ");
        const NEXT_BRIEFING_SUBSECTION_START_STRING: &CStr =
            cstr!("* New Mission Briefing Text Subsection *");
        const END_OF_BRIEFING_SUBSECTION_STRING: &CStr =
            cstr!("* End of Mission Briefing Text Subsection *");

        let mut buffer: [c_char; 500] = [0; 500];
        read_value_from_string(
            mission_briefing_pointer,
            BRIEFING_TITLE_SONG_STRING.as_ptr() as *mut c_char,
            cstr!("%s").as_ptr() as *mut c_char,
            buffer.as_mut_ptr() as *mut c_void,
        );
        self.switch_background_music_to(buffer.as_mut_ptr());

        SDL_SetClipRect(self.graphics.ne_screen, null_mut());
        read_value_from_string(
            mission_briefing_pointer,
            BRIEFING_TITLE_PICTURE_STRING.as_ptr() as *mut c_char,
            cstr!("%s").as_ptr() as *mut c_char,
            buffer.as_mut_ptr() as *mut c_void,
        );
        let image = self.find_file(
            buffer.as_mut_ptr(),
            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
            Themed::NoTheme as c_int,
            Criticality::Critical as c_int,
        );
        self.display_image(image);
        self.make_grid_on_screen(Some(&self.vars.screen_rect));
        self.vars.me.status = Status::Briefing as c_int;

        self.b_font.current_font = self.global.para_b_font;

        self.display_banner(
            null_mut(),
            null_mut(),
            DisplayBannerFlags::FORCE_UPDATE.bits().into(),
        );

        // Next we display all the subsections of the briefing section
        // with scrolling font
        let mut next_subsection_start_pointer = mission_briefing_pointer;
        let mut prepared_briefing_text: *mut i8 = null_mut();
        loop {
            next_subsection_start_pointer = libc::strstr(
                next_subsection_start_pointer,
                NEXT_BRIEFING_SUBSECTION_START_STRING.as_ptr(),
            );
            if next_subsection_start_pointer.is_null() {
                break;
            }

            next_subsection_start_pointer = next_subsection_start_pointer
                .add(NEXT_BRIEFING_SUBSECTION_START_STRING.to_bytes().len());
            let termination_pointer = libc::strstr(
                next_subsection_start_pointer,
                END_OF_BRIEFING_SUBSECTION_STRING.as_ptr(),
            );
            if termination_pointer.is_null() {
                panic!("Title: Unterminated Subsection in Mission briefing....Terminating...");
            }
            let this_text_length = termination_pointer.offset_from(next_subsection_start_pointer);
            if prepared_briefing_text.is_null().not() {
                let len = CStr::from_ptr(prepared_briefing_text).to_bytes().len() + 10;
                dealloc(
                    prepared_briefing_text as *mut u8,
                    Layout::array::<i8>(len).unwrap(),
                );
            }
            prepared_briefing_text = alloc_zeroed(
                Layout::array::<i8>(usize::try_from(this_text_length).unwrap() + 10).unwrap(),
            ) as *mut c_char;
            libc::strncpy(
                prepared_briefing_text,
                next_subsection_start_pointer,
                this_text_length.try_into().unwrap(),
            );
            *prepared_briefing_text.offset(this_text_length) = 0;

            let mut rect = self.vars.full_user_rect;
            rect.x += 10;
            rect.w -= 10; //leave some border
            if self.scroll_text(prepared_briefing_text, &mut rect, 0) == 1 {
                break; // User pressed 'fire'
            }
        }

        if prepared_briefing_text.is_null().not() {
            let len = CStr::from_ptr(prepared_briefing_text).to_bytes().len() + 10;
            dealloc(
                prepared_briefing_text as *mut u8,
                Layout::array::<i8>(len).unwrap(),
            );
        }
    }

    /// This function loads all the constant variables of the game from
    /// a dat file, that should be optimally human readable.
    pub unsafe fn init_game_data(&mut self, data_filename: *mut c_char) {
        const END_OF_GAME_DAT_STRING: &CStr = cstr!("*** End of game.dat File ***");

        /* Read the whole game data to memory */
        let fpath = self.find_file(
            data_filename,
            MAP_DIR_C.as_ptr() as *mut c_char,
            Themed::NoTheme as c_int,
            Criticality::Critical as c_int,
        );

        let mut data = self.read_and_malloc_and_terminate_file(
            fpath,
            END_OF_GAME_DAT_STRING.as_ptr() as *mut c_char,
        );

        self.get_general_game_constants(data.as_mut_ptr());
        self.get_robot_data(data.as_mut_ptr() as *mut c_void);
        self.get_bullet_data(data.as_mut_ptr() as *mut c_void);

        // Now we read in the total time amount for the blast animations
        const BLAST_ONE_TOTAL_AMOUNT_OF_TIME_STRING: &CStr =
            cstr!("Time in seconds for the animation of blast one :");
        const BLAST_TWO_TOTAL_AMOUNT_OF_TIME_STRING: &CStr =
            cstr!("Time in seconds for the animation of blast one :");

        read_value_from_string(
            data.as_mut_ptr(),
            BLAST_ONE_TOTAL_AMOUNT_OF_TIME_STRING.as_ptr() as *mut c_char,
            cstr!("%f").as_ptr() as *mut c_char,
            &mut self.vars.blastmap[0].total_animation_time as *mut f32 as *mut c_void,
        );
        read_value_from_string(
            data.as_mut_ptr(),
            BLAST_TWO_TOTAL_AMOUNT_OF_TIME_STRING.as_ptr() as *mut c_char,
            cstr!("%f").as_ptr() as *mut c_char,
            &mut self.vars.blastmap[1].total_animation_time as *mut f32 as *mut c_void,
        );
    }

    /// This function loads all the constant variables of the game from
    /// a dat file, that should be optimally human readable.
    pub unsafe fn get_robot_data(&mut self, data_pointer: *mut c_void) {
        const MAXSPEED_CALIBRATOR_STRING: &CStr =
            cstr!("Common factor for all droids maxspeed values: ");
        const ACCELERATION_CALIBRATOR_STRING: &CStr =
            cstr!("Common factor for all droids acceleration values: ");
        const MAXENERGY_CALIBRATOR_STRING: &CStr =
            cstr!("Common factor for all droids maximum energy values: ");
        const ENERGYLOSS_CALIBRATOR_STRING: &CStr =
            cstr!("Common factor for all droids energyloss values: ");
        const AGGRESSION_CALIBRATOR_STRING: &CStr =
            cstr!("Common factor for all droids aggression values: ");
        const SCORE_CALIBRATOR_STRING: &CStr = cstr!("Common factor for all droids score values: ");

        const ROBOT_SECTION_BEGIN_STRING: &CStr = cstr!("*** Start of Robot Data Section: ***");
        // const ROBOT_SECTION_END_STRING: &CStr = cstr!("*** End of Robot Data Section: ***");
        const NEW_ROBOT_BEGIN_STRING: &CStr = cstr!("** Start of new Robot: **");
        const DROIDNAME_BEGIN_STRING: &CStr = cstr!("Droidname: ");
        const MAXSPEED_BEGIN_STRING: &CStr = cstr!("Maximum speed of this droid: ");
        const CLASS_BEGIN_STRING: &CStr = cstr!("Class of this droid: ");
        const ACCELERATION_BEGIN_STRING: &CStr = cstr!("Maximum acceleration of this droid: ");
        const MAXENERGY_BEGIN_STRING: &CStr = cstr!("Maximum energy of this droid: ");
        const LOSEHEALTH_BEGIN_STRING: &CStr =
            cstr!("Rate of energyloss under influence control: ");
        const GUN_BEGIN_STRING: &CStr = cstr!("Weapon type this droid uses: ");
        const AGGRESSION_BEGIN_STRING: &CStr = cstr!("Aggression rate of this droid: ");
        const FLASHIMMUNE_BEGIN_STRING: &CStr = cstr!("Is this droid immune to disruptor blasts? ");
        const SCORE_BEGIN_STRING: &CStr = cstr!("Score gained for destroying one of this type: ");
        const HEIGHT_BEGIN_STRING: &CStr = cstr!("Height of this droid : ");
        const WEIGHT_BEGIN_STRING: &CStr = cstr!("Weight of this droid : ");
        const DRIVE_BEGIN_STRING: &CStr = cstr!("Drive of this droid : ");
        const BRAIN_BEGIN_STRING: &CStr = cstr!("Brain of this droid : ");
        const SENSOR1_BEGIN_STRING: &CStr = cstr!("Sensor 1 of this droid : ");
        const SENSOR2_BEGIN_STRING: &CStr = cstr!("Sensor 2 of this droid : ");
        const SENSOR3_BEGIN_STRING: &CStr = cstr!("Sensor 3 of this droid : ");
        // const ADVANCED_FIGHTING_BEGIN_STRING: &CStr =
        //     cstr!("Advanced Fighting present in this droid : ");
        // const GO_REQUEST_REINFORCEMENTS_BEGIN_STRING: &CStr =
        //     cstr!("Going to request reinforcements typical for this droid : ");
        const NOTES_BEGIN_STRING: &CStr = cstr!("Notes concerning this droid : ");

        let mut maxspeed_calibrator = 0f32;
        let mut acceleration_calibrator = 0f32;
        let mut maxenergy_calibrator = 0f32;
        let mut energyloss_calibrator = 0f32;
        let mut aggression_calibrator = 0f32;
        let mut score_calibrator = 0f32;

        let mut robot_pointer = locate_string_in_data(
            data_pointer as *mut c_char,
            ROBOT_SECTION_BEGIN_STRING.as_ptr() as *mut c_char,
        );

        info!("Starting to read robot calibration section");

        // Now we read in the speed calibration factor for all droids
        read_value_from_string(
            robot_pointer,
            MAXSPEED_CALIBRATOR_STRING.as_ptr() as *mut c_char,
            cstr!("%f").as_ptr() as *mut c_char,
            &mut maxspeed_calibrator as *mut _ as *mut c_void,
        );

        // Now we read in the acceleration calibration factor for all droids
        read_value_from_string(
            robot_pointer,
            ACCELERATION_CALIBRATOR_STRING.as_ptr() as *mut c_char,
            cstr!("%f").as_ptr() as *mut c_char,
            &mut acceleration_calibrator as *mut _ as *mut c_void,
        );

        // Now we read in the maxenergy calibration factor for all droids
        read_value_from_string(
            robot_pointer,
            MAXENERGY_CALIBRATOR_STRING.as_ptr() as *mut c_char,
            cstr!("%f").as_ptr() as *mut c_char,
            &mut maxenergy_calibrator as *mut _ as *mut c_void,
        );

        // Now we read in the energy_loss calibration factor for all droids
        read_value_from_string(
            robot_pointer,
            ENERGYLOSS_CALIBRATOR_STRING.as_ptr() as *mut c_char,
            cstr!("%f").as_ptr() as *mut c_char,
            &mut energyloss_calibrator as *mut _ as *mut c_void,
        );

        // Now we read in the aggression calibration factor for all droids
        read_value_from_string(
            robot_pointer,
            AGGRESSION_CALIBRATOR_STRING.as_ptr() as *mut c_char,
            cstr!("%f").as_ptr() as *mut c_char,
            &mut aggression_calibrator as *mut _ as *mut c_void,
        );

        // Now we read in the score calibration factor for all droids
        read_value_from_string(
            robot_pointer,
            SCORE_CALIBRATOR_STRING.as_ptr() as *mut c_char,
            cstr!("%f").as_ptr() as *mut c_char,
            &mut score_calibrator as *mut _ as *mut c_void,
        );

        info!("Starting to read Robot data...");

        // cleanup if previously allocated:
        self.free_druidmap();

        // At first, we must allocate memory for the droid specifications.
        // How much?  That depends on the number of droids defined in freedroid.ruleset.
        // So we have to count those first.  ok.  lets do it.
        self.main.number_of_droid_types = count_string_occurences(
            data_pointer as *mut c_char,
            NEW_ROBOT_BEGIN_STRING.as_ptr() as *mut c_char,
        );

        // Now that we know how many robots are defined in freedroid.ruleset, we can allocate
        // a fitting amount of memory.
        self.vars.droidmap = alloc_zeroed(
            Layout::array::<DruidSpec>(usize::try_from(self.main.number_of_droid_types).unwrap())
                .unwrap(),
        ) as *mut DruidSpec;
        info!(
            "We have counted {} different druid types in the game data file.",
            self.main.number_of_droid_types,
        );
        info!("MEMORY HAS BEEN ALLOCATED. THE READING CAN BEGIN.");

        //Now we start to read the values for each robot:
        //Of which parts is it composed, which stats does it have?
        let mut robot_index = 0;
        loop {
            robot_pointer = libc::strstr(robot_pointer, NEW_ROBOT_BEGIN_STRING.as_ptr());
            if robot_pointer.is_null() {
                break;
            }

            info!("Found another Robot specification entry!  Lets add that to the others!");
            robot_pointer = robot_pointer.add(1); // to avoid doubly taking this entry

            // Now we read in the Name of this droid.  We consider as a name the rest of the
            read_value_from_string(
                robot_pointer,
                DROIDNAME_BEGIN_STRING.as_ptr() as *mut c_char,
                cstr!("%s").as_ptr() as *mut c_char,
                &mut (*self.vars.droidmap.add(robot_index)).druidname as *mut _ as *mut c_void,
            );

            // Now we read in the maximal speed this droid can go.
            read_value_from_string(
                robot_pointer,
                MAXSPEED_BEGIN_STRING.as_ptr() as *mut c_char,
                cstr!("%f").as_ptr() as *mut c_char,
                &mut (*self.vars.droidmap.add(robot_index)).maxspeed as *mut _ as *mut c_void,
            );

            // Now we read in the class of this droid.
            read_value_from_string(
                robot_pointer,
                CLASS_BEGIN_STRING.as_ptr() as *mut c_char,
                cstr!("%d").as_ptr() as *mut c_char,
                &mut (*self.vars.droidmap.add(robot_index)).class as *mut _ as *mut c_void,
            );

            // Now we read in the maximal acceleration this droid can go.
            read_value_from_string(
                robot_pointer,
                ACCELERATION_BEGIN_STRING.as_ptr() as *mut c_char,
                cstr!("%f").as_ptr() as *mut c_char,
                &mut (*self.vars.droidmap.add(robot_index)).accel as *mut _ as *mut c_void,
            );

            // Now we read in the maximal energy this droid can store.
            read_value_from_string(
                robot_pointer,
                MAXENERGY_BEGIN_STRING.as_ptr() as *mut c_char,
                cstr!("%f").as_ptr() as *mut c_char,
                &mut (*self.vars.droidmap.add(robot_index)).maxenergy as *mut _ as *mut c_void,
            );

            // Now we read in the lose_health rate.
            read_value_from_string(
                robot_pointer,
                LOSEHEALTH_BEGIN_STRING.as_ptr() as *mut c_char,
                cstr!("%f").as_ptr() as *mut c_char,
                &mut (*self.vars.droidmap.add(robot_index)).lose_health as *mut _ as *mut c_void,
            );

            // Now we read in the class of this droid.
            read_value_from_string(
                robot_pointer,
                GUN_BEGIN_STRING.as_ptr() as *mut c_char,
                cstr!("%d").as_ptr() as *mut c_char,
                &mut (*self.vars.droidmap.add(robot_index)).gun as *mut _ as *mut c_void,
            );

            // Now we read in the aggression rate of this droid.
            read_value_from_string(
                robot_pointer,
                AGGRESSION_BEGIN_STRING.as_ptr() as *mut c_char,
                cstr!("%d").as_ptr() as *mut c_char,
                &mut (*self.vars.droidmap.add(robot_index)).aggression as *mut _ as *mut c_void,
            );

            // Now we read in the flash immunity of this droid.
            read_value_from_string(
                robot_pointer,
                FLASHIMMUNE_BEGIN_STRING.as_ptr() as *mut c_char,
                cstr!("%d").as_ptr() as *mut c_char,
                &mut (*self.vars.droidmap.add(robot_index)).flashimmune as *mut _ as *mut c_void,
            );

            // Now we score to be had for destroying one droid of this type
            read_value_from_string(
                robot_pointer,
                SCORE_BEGIN_STRING.as_ptr() as *mut c_char,
                cstr!("%d").as_ptr() as *mut c_char,
                &mut (*self.vars.droidmap.add(robot_index)).score as *mut _ as *mut c_void,
            );

            // Now we read in the height of this droid of this type
            read_value_from_string(
                robot_pointer,
                HEIGHT_BEGIN_STRING.as_ptr() as *mut c_char,
                cstr!("%f").as_ptr() as *mut c_char,
                &mut (*self.vars.droidmap.add(robot_index)).height as *mut _ as *mut c_void,
            );

            // Now we read in the weight of this droid type
            read_value_from_string(
                robot_pointer,
                WEIGHT_BEGIN_STRING.as_ptr() as *mut c_char,
                cstr!("%d").as_ptr() as *mut c_char,
                &mut (*self.vars.droidmap.add(robot_index)).weight as *mut _ as *mut c_void,
            );

            // Now we read in the drive of this droid of this type
            read_value_from_string(
                robot_pointer,
                DRIVE_BEGIN_STRING.as_ptr() as *mut c_char,
                cstr!("%d").as_ptr() as *mut c_char,
                &mut (*self.vars.droidmap.add(robot_index)).drive as *mut _ as *mut c_void,
            );

            // Now we read in the brain of this droid of this type
            read_value_from_string(
                robot_pointer,
                BRAIN_BEGIN_STRING.as_ptr() as *mut c_char,
                cstr!("%d").as_ptr() as *mut c_char,
                &mut (*self.vars.droidmap.add(robot_index)).brain as *mut _ as *mut c_void,
            );

            // Now we read in the sensor 1, 2 and 3 of this droid type
            read_value_from_string(
                robot_pointer,
                SENSOR1_BEGIN_STRING.as_ptr() as *mut c_char,
                cstr!("%d").as_ptr() as *mut c_char,
                &mut (*self.vars.droidmap.add(robot_index)).sensor1 as *mut _ as *mut c_void,
            );
            read_value_from_string(
                robot_pointer,
                SENSOR2_BEGIN_STRING.as_ptr() as *mut c_char,
                cstr!("%d").as_ptr() as *mut c_char,
                &mut (*self.vars.droidmap.add(robot_index)).sensor2 as *mut _ as *mut c_void,
            );
            read_value_from_string(
                robot_pointer,
                SENSOR3_BEGIN_STRING.as_ptr() as *mut c_char,
                cstr!("%d").as_ptr() as *mut c_char,
                &mut (*self.vars.droidmap.add(robot_index)).sensor3 as *mut _ as *mut c_void,
            );

            // Now we read in the notes concerning this droid.  We consider as notes all the rest of the
            // line after the NOTES_BEGIN_STRING until the "\n" is found.
            (*self.vars.droidmap.add(robot_index)).notes = read_and_malloc_string_from_data(
                robot_pointer,
                NOTES_BEGIN_STRING.as_ptr() as *mut c_char,
                cstr!("\n").as_ptr() as *mut c_char,
            );

            // Now we're potentially ready to process the next droid.  Therefore we proceed to
            // the next number in the Droidmap array.
            robot_index += 1;
        }

        info!("That must have been the last robot.  We're done reading the robot data.");
        info!("Applying the calibration factors to all droids...");

        for droid in std::slice::from_raw_parts_mut(
            self.vars.droidmap,
            self.main.number_of_droid_types.try_into().unwrap(),
        ) {
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
    pub unsafe fn get_bullet_data(&mut self, data_pointer: *mut c_void) {
        // const BULLET_SECTION_BEGIN_STRING: &CStr = cstr!("*** Start of Bullet Data Section: ***");
        // const BULLET_SECTION_END_STRING: &CStr = cstr!("*** End of Bullet Data Section: ***");
        const NEW_BULLET_TYPE_BEGIN_STRING: &CStr =
            cstr!("** Start of new bullet specification subsection **");

        const BULLET_RECHARGE_TIME_BEGIN_STRING: &CStr =
            cstr!("Time is takes to recharge this bullet/weapon in seconds :");
        const BULLET_SPEED_BEGIN_STRING: &CStr = cstr!("Flying speed of this bullet type :");
        const BULLET_DAMAGE_BEGIN_STRING: &CStr =
            cstr!("Damage cause by a hit of this bullet type :");
        // #define BULLET_NUMBER_OF_PHASES_BEGIN_STRING "Number of different phases that were designed for this bullet type :"
        // const BULLET_ONE_SHOT_ONLY_AT_A_TIME: &CStr =
        //     cstr!("Cannot fire until previous bullet has been deleted : ");
        const BULLET_BLAST_TYPE_CAUSED_BEGIN_STRING: &CStr =
            cstr!("Type of blast this bullet causes when crashing e.g. against a wall :");

        const BULLET_SPEED_CALIBRATOR_STRING: &CStr =
            cstr!("Common factor for all bullet's speed values: ");
        const BULLET_DAMAGE_CALIBRATOR_STRING: &CStr =
            cstr!("Common factor for all bullet's damage values: ");

        info!("Starting to read bullet data...");
        //--------------------
        // At first, we must allocate memory for the droid specifications.
        // How much?  That depends on the number of droids defined in freedroid.ruleset.
        // So we have to count those first.  ok.  lets do it.

        self.graphics.number_of_bullet_types = count_string_occurences(
            data_pointer as *mut c_char,
            NEW_BULLET_TYPE_BEGIN_STRING.as_ptr() as *mut c_char,
        );

        // Now that we know how many bullets are defined in freedroid.ruleset, we can allocate
        // a fitting amount of memory, but of course only if the memory hasn't been allocated
        // aready!!!
        //
        // If we would do that in any case, every Init_Game_Data call would destroy the loaded
        // image files AND MOST LIKELY CAUSE A SEGFAULT!!!
        //
        if self.vars.bulletmap.is_null() {
            self.vars.bulletmap = alloc_zeroed(
                Layout::array::<BulletSpec>(
                    usize::try_from(self.graphics.number_of_bullet_types).unwrap(),
                )
                .unwrap(),
            ) as *mut BulletSpec;
            std::ptr::write_bytes(
                self.vars.bulletmap,
                0,
                usize::try_from(self.graphics.number_of_bullet_types).unwrap(),
            );
            info!(
                "We have counted {} different bullet types in the game data file.",
                self.graphics.number_of_bullet_types
            );
            info!("MEMORY HAS BEEN ALLOCATED. THE READING CAN BEGIN.");
        }

        //--------------------
        // Now we start to read the values for each bullet type:
        //
        let mut bullet_pointer = data_pointer as *mut c_char;
        let mut bullet_index = 0;
        loop {
            bullet_pointer = libc::strstr(bullet_pointer, NEW_BULLET_TYPE_BEGIN_STRING.as_ptr());
            if bullet_pointer.is_null() {
                break;
            }

            info!("Found another Bullet specification entry!  Lets add that to the others!");
            bullet_pointer = bullet_pointer.add(1); // to avoid doubly taking this entry

            // Now we read in the recharging time for this bullettype(=weapontype)
            read_value_from_string(
                bullet_pointer,
                BULLET_RECHARGE_TIME_BEGIN_STRING.as_ptr() as *mut c_char,
                cstr!("%f").as_ptr() as *mut c_char,
                &mut (*self.vars.bulletmap.add(bullet_index)).recharging_time as *mut _
                    as *mut c_void,
            );

            // Now we read in the maximal speed this type of bullet can go.
            read_value_from_string(
                bullet_pointer,
                BULLET_SPEED_BEGIN_STRING.as_ptr() as *mut c_char,
                cstr!("%f").as_ptr() as *mut c_char,
                &mut (*self.vars.bulletmap.add(bullet_index)).speed as *mut _ as *mut c_void,
            );

            // Now we read in the damage this bullet can do
            read_value_from_string(
                bullet_pointer,
                BULLET_DAMAGE_BEGIN_STRING.as_ptr() as *mut c_char,
                cstr!("%d").as_ptr() as *mut c_char,
                &mut (*self.vars.bulletmap.add(bullet_index)).damage as *mut _ as *mut c_void,
            );

            // Now we read in the number of phases that are designed for this bullet type
            // THIS IS NOW SPECIFIED IN THE THEME CONFIG FILE
            // ReadValueFromString( BulletPointer ,  BULLET_NUMBER_OF_PHASES_BEGIN_STRING , "%d" ,
            // &(*Bulletmap.add(BulletIndex)).phases , EndOfBulletData );

            // Now we read in the type of blast this bullet will cause when crashing e.g. against the wall
            read_value_from_string(
                bullet_pointer,
                BULLET_BLAST_TYPE_CAUSED_BEGIN_STRING.as_ptr() as *mut c_char,
                cstr!("%d").as_ptr() as *mut c_char,
                &mut (*self.vars.bulletmap.add(bullet_index)).blast as *mut _ as *mut c_void,
            );

            bullet_index += 1;
        }

        //--------------------
        // Now that the detailed values for the bullets have been read in,
        // we now read in the general calibration contants and after that
        // the start to apply them right now, so they also take effect.

        info!("Starting to read bullet calibration section");
        let mut bullet_speed_calibrator = 0f32;
        let mut bullet_damage_calibrator = 0f32;

        // Now we read in the speed calibration factor for all bullets
        read_value_from_string(
            data_pointer as *mut c_char,
            BULLET_SPEED_CALIBRATOR_STRING.as_ptr() as *mut c_char,
            cstr!("%f").as_ptr() as *mut c_char,
            &mut bullet_speed_calibrator as *mut _ as *mut c_void,
        );

        // Now we read in the damage calibration factor for all bullets
        read_value_from_string(
            data_pointer as *mut c_char,
            BULLET_DAMAGE_CALIBRATOR_STRING.as_ptr() as *mut c_char,
            cstr!("%f").as_ptr() as *mut c_char,
            &mut bullet_damage_calibrator as *mut _ as *mut c_void,
        );

        // Now that all the calibrations factors have been read in, we can start to
        // apply them to all the bullet types
        for bullet in std::slice::from_raw_parts_mut(
            self.vars.bulletmap,
            usize::try_from(self.graphics.number_of_bullet_types).unwrap(),
        ) {
            bullet.speed *= bullet_speed_calibrator;
            bullet.damage = (bullet.damage as f32 * bullet_damage_calibrator) as c_int;
        }
    }

    /// This function loads all the constant variables of the game from
    /// a dat file, that should be optimally human readable.
    pub unsafe fn get_general_game_constants(&mut self, data: *mut c_char) {
        // const CONSTANTS_SECTION_BEGIN_STRING: &CStr =
        //     cstr!("*** Start of General Game Constants Section: ***");
        // const CONSTANTS_SECTION_END_STRING: &CStr =
        //     cstr!("*** End of General Game Constants Section: ***");
        const COLLISION_LOSE_ENERGY_CALIBRATOR_STRING: &CStr =
            cstr!("Energy-Loss-factor for Collisions of Influ with hostile robots=");
        const BLAST_RADIUS_SPECIFICATION_STRING: &CStr =
            cstr!("Radius of explosions (as far as damage is concerned) in multiples of tiles=");
        const DROID_RADIUS_SPECIFICATION_STRING: &CStr = cstr!("Droid radius:");
        const BLAST_DAMAGE_SPECIFICATION_STRING: &CStr =
            cstr!("Amount of damage done by contact to a blast per second of time=");
        const TIME_FOR_DOOR_MOVEMENT_SPECIFICATION_STRING: &CStr =
            cstr!("Time for the doors to move by one subphase of their movement=");

        const DEATHCOUNT_DRAIN_SPEED_STRING: &CStr = cstr!("Deathcount drain speed =");
        const ALERT_THRESHOLD_STRING: &CStr = cstr!("First alert threshold =");
        const ALERT_BONUS_PER_SEC_STRING: &CStr = cstr!("Alert bonus per second =");

        info!("Starting to read contents of General Game Constants section");

        // read in Alert-related parameters:
        read_value_from_string(
            data,
            DEATHCOUNT_DRAIN_SPEED_STRING.as_ptr() as *mut c_char,
            cstr!("%f").as_ptr() as *mut c_char,
            &mut self.main.death_count_drain_speed as *mut _ as *mut c_void,
        );
        read_value_from_string(
            data,
            ALERT_THRESHOLD_STRING.as_ptr() as *mut c_char,
            cstr!("%d").as_ptr() as *mut c_char,
            &mut self.main.alert_threshold as *mut _ as *mut c_void,
        );
        read_value_from_string(
            data,
            ALERT_BONUS_PER_SEC_STRING.as_ptr() as *mut c_char,
            cstr!("%f").as_ptr() as *mut c_char,
            &mut self.main.alert_bonus_per_sec as *mut _ as *mut c_void,
        );

        // Now we read in the speed calibration factor for all bullets
        read_value_from_string(
            data,
            COLLISION_LOSE_ENERGY_CALIBRATOR_STRING.as_ptr() as *mut c_char,
            cstr!("%f").as_ptr() as *mut c_char,
            &mut self.global.collision_lose_energy_calibrator as *mut _ as *mut c_void,
        );

        // Now we read in the blast radius
        read_value_from_string(
            data,
            BLAST_RADIUS_SPECIFICATION_STRING.as_ptr() as *mut c_char,
            cstr!("%f").as_ptr() as *mut c_char,
            &mut self.global.blast_radius as *mut _ as *mut c_void,
        );

        // Now we read in the druid 'radius' in x direction
        read_value_from_string(
            data,
            DROID_RADIUS_SPECIFICATION_STRING.as_ptr() as *mut c_char,
            cstr!("%f").as_ptr() as *mut c_char,
            &mut self.global.droid_radius as *mut _ as *mut c_void,
        );

        // Now we read in the blast damage amount per 'second' of contact with the blast
        read_value_from_string(
            data,
            BLAST_DAMAGE_SPECIFICATION_STRING.as_ptr() as *mut c_char,
            cstr!("%f").as_ptr() as *mut c_char,
            &mut self.global.blast_damage_per_second as *mut _ as *mut c_void,
        );

        // Now we read in the time is takes for the door to move one phase
        read_value_from_string(
            data,
            TIME_FOR_DOOR_MOVEMENT_SPECIFICATION_STRING.as_ptr() as *mut c_char,
            cstr!("%f").as_ptr() as *mut c_char,
            &mut self.global.time_for_each_phase_of_door_movement as *mut _ as *mut c_void,
        );
    }

    /// Show end-screen
    pub(crate) unsafe fn thou_art_defeated(&mut self) {
        self.vars.me.status = Status::Terminated as c_int;
        SDL_ShowCursor(SDL_DISABLE);

        self.explode_influencer();

        self.wait_for_all_keys_released();

        let mut now = SDL_GetTicks();

        while (SDL_GetTicks() - now) < WAIT_AFTER_KILLED {
            // add "slow motion effect" for final explosion
            self.set_time_factor(SLOWMO_FACTOR);

            self.start_taking_time_for_fps_calculation();
            self.display_banner(null_mut(), null_mut(), 0);
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

        Mix_HaltMusic();

        // important!!: don't forget to stop fps calculation here (bugfix: enemy piles after gameOver)
        self.activate_conservative_frame_computation();

        // TODO: avoid a temporary backup
        let mut user_rect = std::mem::replace(&mut self.vars.user_rect, rect!(0, 0, 0, 0));
        self.white_noise(
            self.graphics.ne_screen,
            &mut user_rect,
            WAIT_AFTER_KILLED.try_into().unwrap(),
        );
        self.vars.user_rect = user_rect;

        self.assemble_combat_picture(AssembleCombatWindowFlags::DO_SCREEN_UPDATE.bits().into());
        self.make_grid_on_screen(Some(&self.vars.user_rect));

        let mut dst = SDL_Rect {
            x: self.get_user_center().x - i16::try_from(self.vars.portrait_rect.w / 2).unwrap(),
            y: self.get_user_center().y - i16::try_from(self.vars.portrait_rect.h / 2).unwrap(),
            w: self.vars.portrait_rect.w,
            h: self.vars.portrait_rect.h,
        };
        SDL_UpperBlit(
            self.graphics.pic999,
            null_mut(),
            self.graphics.ne_screen,
            &mut dst,
        );
        self.thou_art_defeated_sound();

        self.b_font.current_font = self.global.para_b_font;
        let h = font_height(&*self.global.para_b_font);
        self.display_text(
            cstr!("Transmission").as_ptr() as *mut c_char,
            i32::from(dst.x) - h,
            i32::from(dst.y) - h,
            &self.vars.user_rect,
        );
        self.display_text(
            cstr!("Terminated").as_ptr() as *mut c_char,
            i32::from(dst.x) - h,
            i32::from(dst.y) + i32::from(dst.h),
            &self.vars.user_rect,
        );
        self.printf_sdl(self.graphics.ne_screen, -1, -1, format_args!("\n"));
        SDL_Flip(self.graphics.ne_screen);

        now = SDL_GetTicks();

        self.wait_for_all_keys_released();
        while SDL_GetTicks() - now < SHOW_WAIT {
            SDL_Delay(1);
            if self.any_key_just_pressed() != 0 {
                break;
            }
        }

        self.update_highscores();

        self.game_over = true;
    }
}
