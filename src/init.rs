use crate::{
    b_font::{GetCurrentFont, Para_BFont, SetCurrentFont},
    bullet::{DeleteBullet, ExplodeBlasts, MoveBullets},
    curShip, debug_level,
    defs::{
        self, scale_rect, AssembleCombatWindowFlags, Criticality, DisplayBannerFlags, Droid,
        Status, Themed, FD_DATADIR, GRAPHICS_DIR_C, LOCAL_DATADIR, MAP_DIR_C, MAXBULLETS,
        TITLE_PIC_FILE_C, WAIT_AFTER_KILLED,
    },
    enemy::ShuffleEnemys,
    global::{
        num_highscores, Blastmap, Bulletmap, CurrentCombatScaleFactor, Druidmap, Font0_BFont,
        GameConfig, Highscores, SkipAFewFrames,
    },
    graphics::{
        ne_screen, AllThemes, ClearGraphMem, DisplayImage, InitPictures, Init_Video, Load_Fonts,
        MakeGridOnScreen, Number_Of_Bullet_Types,
    },
    highscore::InitHighscores,
    influencer::InitInfluPositionHistory,
    input::{wait_for_all_keys_released, wait_for_key_pressed, Init_Joy},
    map::{GetCrew, GetLiftConnections, LoadShip},
    misc::{
        find_file, init_progress, set_time_factor, update_progress,
        Activate_Conservative_Frame_Computation, CountStringOccurences, LocateStringInData,
        MyMalloc, MyRandom, ReadAndMallocAndTerminateFile, ReadAndMallocStringFromData,
        ReadValueFromString, Terminate,
    },
    sound::{Init_Audio, Switch_Background_Music_To},
    sound_on,
    text::{DisplayText, ScrollText},
    vars::{Classic_User_Rect, Full_User_Rect, Screen_Rect, User_Rect},
    view::{Assemble_Combat_Picture, DisplayBanner},
    AllBlasts, AllBullets, AllEnemys, CurLevel, DeathCount, GameOver, LastGotIntoBlastSound,
    LastRefreshSound, LevelDoorsNotMovedTime, Me, NumEnemys, Number_Of_Droid_Types, RealScore,
    ShowScore, ThisMessageTime,
};

use clap::{crate_version, Clap};
use cstr::cstr;
use log::{error, info, warn};
use sdl::{
    event::ll::SDL_DISABLE,
    ll::SDL_GetTicks,
    mouse::ll::SDL_ShowCursor,
    video::ll::{SDL_Flip, SDL_FreeSurface, SDL_SetClipRect},
};
use std::{
    convert::{TryFrom, TryInto},
    ffi::CStr,
    ops::Not,
    os::raw::{c_char, c_float, c_int, c_long, c_uint, c_void},
    path::Path,
    ptr::null_mut,
};

extern "C" {
    pub fn LoadGameConfig();
    pub fn Init_Game_Data(data_filename: *mut c_char);
    pub fn Title(mission_briefing_pointer: *mut c_char);

    static mut DebriefingText: *mut c_char;
    static mut DebriefingSong: [c_char; 500];
    static mut Previous_Mission_Name: [c_char; 500];

}

const MISSION_COMPLETE_BONUS: f32 = 1000.;
const COPYRIGHT: &str = "\nCopyright (C) 2003-2018 Johannes Prix, Reinhard Prix\n\
Freedroid comes with NO WARRANTY to the extent permitted by law.\n\
You may redistribute copies of Freedroid under the terms of the\n\
GNU General Public License.\n\
For more information about these matters, see the file named COPYING.";

#[no_mangle]
pub unsafe extern "C" fn FreeGameMem() {
    // free bullet map
    if Bulletmap.is_null().not() {
        let bullet_map = std::slice::from_raw_parts_mut(
            Bulletmap,
            usize::try_from(Number_Of_Bullet_Types).unwrap(),
        );
        for bullet in bullet_map {
            for surface in &bullet.SurfacePointer {
                SDL_FreeSurface(*surface);
            }
        }
        libc::free(Bulletmap as *mut c_void);
        Bulletmap = null_mut();
    }

    // free blast map
    for blast_type in &mut Blastmap {
        for surface in &mut blast_type.SurfacePointer {
            SDL_FreeSurface(*surface);
            *surface = null_mut();
        }
    }

    // free droid map
    FreeDruidmap();

    // free highscores list
    if Highscores.is_null().not() {
        let highscores =
            std::slice::from_raw_parts(Highscores, usize::try_from(num_highscores).unwrap());
        for highscore in highscores {
            libc::free(*highscore as *mut c_void);
        }
        libc::free(Highscores as *mut c_void);
        Highscores = null_mut();
    }

    // free constant text blobs
    libc::free(DebriefingText as *mut c_void);
    DebriefingText = null_mut();
}

#[no_mangle]
pub unsafe extern "C" fn FreeDruidmap() {
    if Druidmap.is_null() {
        return;
    }
    let droid_map =
        std::slice::from_raw_parts(Druidmap, usize::try_from(Number_Of_Droid_Types).unwrap());
    for droid in droid_map {
        libc::free(droid.notes as *mut c_void);
    }

    libc::free(Druidmap as *mut c_void);
    Druidmap = null_mut();
}

/// put some ideology message for our poor friends enslaved by M$-Win32 ;)
#[no_mangle]
pub unsafe extern "C" fn Win32Disclaimer() {
    SDL_SetClipRect(ne_screen, null_mut());
    DisplayImage(find_file(
        TITLE_PIC_FILE_C.as_ptr() as *mut c_char,
        GRAPHICS_DIR_C.as_ptr() as *mut c_char,
        Themed::NoTheme as c_int,
        Criticality::Critical as c_int,
    )); // show title pic
    MakeGridOnScreen(Some(&Screen_Rect));

    SetCurrentFont(Para_BFont);

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
    SDL_Flip(ne_screen);

    wait_for_key_pressed();
}

/// This function checks, if the influencer has succeeded in his given
/// mission.  If not it returns, if yes the Debriefing is started.
#[no_mangle]
pub unsafe extern "C" fn CheckIfMissionIsComplete() {
    for enemy in AllEnemys.iter().take(NumEnemys.try_into().unwrap()) {
        if enemy.status != Status::Out as c_int && enemy.status != Status::Terminated as c_int {
            return;
        }
    }

    // mission complete: all droids have been killed
    RealScore += MISSION_COMPLETE_BONUS;
    ThouArtVictorious();
    GameOver = true.into();
}

#[no_mangle]
pub unsafe extern "C" fn ThouArtVictorious() {
    Switch_Background_Music_To(DebriefingSong.as_ptr());

    SDL_ShowCursor(SDL_DISABLE);

    ShowScore = RealScore as c_long;
    Me.status = Status::Victory as c_int;
    DisplayBanner(
        null_mut(),
        null_mut(),
        DisplayBannerFlags::FORCE_UPDATE.bits().into(),
    );

    wait_for_all_keys_released();

    let now = SDL_GetTicks();

    while SDL_GetTicks() - now < WAIT_AFTER_KILLED {
        DisplayBanner(null_mut(), null_mut(), 0);
        ExplodeBlasts();
        MoveBullets();
        Assemble_Combat_Picture(AssembleCombatWindowFlags::DO_SCREEN_UPDATE.bits().into());
    }

    let mut rect = Full_User_Rect;
    SDL_SetClipRect(ne_screen, null_mut());
    MakeGridOnScreen(Some(&rect));
    SDL_Flip(ne_screen);
    rect.x += 10;
    rect.w -= 20; //leave some border
    SetCurrentFont(Para_BFont);
    ScrollText(DebriefingText, &mut rect, 6);

    wait_for_all_keys_released();
}

/// This function initializes the whole Freedroid game.
///
/// This must not be confused with initnewgame, which
/// only initializes a new mission for the game.
#[no_mangle]
pub unsafe extern "C" fn InitFreedroid(argc: c_int, argv: *mut *const c_char) {
    Bulletmap = null_mut(); // That will cause the memory to be allocated later

    for bullet in &mut AllBullets {
        bullet.Surfaces_were_generated = false.into();
    }

    SkipAFewFrames = false.into();
    Me.TextVisibleTime = 0.;
    Me.TextToBeDisplayed = null_mut();

    // these are the hardcoded game-defaults, they can be overloaded by the config-file if present
    GameConfig.Current_BG_Music_Volume = 0.3;
    GameConfig.Current_Sound_FX_Volume = 0.5;

    GameConfig.WantedTextVisibleTime = 3.;
    GameConfig.Droid_Talk = false.into();

    GameConfig.Draw_Framerate = false.into();
    GameConfig.Draw_Energy = false.into();
    GameConfig.Draw_DeathCount = false.into();
    GameConfig.Draw_Position = false.into();

    std::ptr::copy_nonoverlapping(
        b"classic\0".as_ptr(),
        GameConfig.Theme_Name.as_mut_ptr() as *mut u8,
        b"classic\0".len(),
    );
    GameConfig.FullUserRect = true.into();
    GameConfig.UseFullscreen = false.into();
    GameConfig.TakeoverActivates = true.into();
    GameConfig.FireHoldTakeover = true.into();
    GameConfig.ShowDecals = false.into();
    GameConfig.AllMapVisible = true.into(); // classic setting: map always visible

    let scale = if cfg!(feature = "gcw0") {
        0.5 // Default for 320x200 device (GCW0)
    } else {
        1.0 // overall scaling of _all_ graphics (e.g. for 320x200 displays)
    };
    GameConfig.scale = scale;

    GameConfig.HogCPU = false.into(); // default to being nice
    GameConfig.emptyLevelSpeedup = 1.0; // speed up *time* in empty levels (ie also energy-loss rate)

    // now load saved options from the config-file
    LoadGameConfig();

    // call this _after_ default settings and LoadGameConfig() ==> cmdline has highest priority!
    parse_command_line();

    User_Rect = if GameConfig.FullUserRect != 0 {
        Full_User_Rect
    } else {
        Classic_User_Rect
    };

    scale_rect(&mut Screen_Rect, GameConfig.scale); // make sure we open a window of the right (rescaled) size!
    Init_Video();

    DisplayImage(find_file(
        TITLE_PIC_FILE_C.as_ptr() as *mut c_char,
        GRAPHICS_DIR_C.as_ptr() as *mut c_char,
        Themed::NoTheme as c_int,
        Criticality::Critical as c_int,
    )); // show title pic
    SDL_Flip(ne_screen);

    Load_Fonts(); // we need this for progress-meter!

    init_progress(cstr!("Loading Freedroid").as_ptr() as *mut c_char);

    FindAllThemes(); // put all found themes into a list: AllThemes[]

    update_progress(5);

    Init_Audio();

    Init_Joy();

    Init_Game_Data(cstr!("freedroid.ruleset").as_ptr() as *mut c_char); // load the default ruleset. This can be */
                                                                        // overwritten from the mission file.

    update_progress(10);

    // The default should be, that no rescaling of the
    // combat window at all is done.
    CurrentCombatScaleFactor = 1.;

    /*
     * Initialise random-number generator in order to make
     * level-start etc really different at each program start
     */
    libc::srand(SDL_GetTicks() as c_uint);

    /* initialize/load the highscore list */
    InitHighscores();

    /* Now fill the pictures correctly to the structs */
    if InitPictures() == 0 {
        error!("Error in InitPictures reported back...");
        Terminate(defs::ERR.into());
    }

    update_progress(100); // finished init
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

/// parse command line arguments and set global switches
/// exit on error, so we don't need to return success status
unsafe fn parse_command_line() {
    let opt = Opt::parse();

    if opt.nosound {
        sound_on = false.into();
    } else if opt.sound {
        sound_on = true.into();
    }

    if let Some(sensitivity) = opt.sensitivity {
        if sensitivity > 32 {
            println!("\nJoystick sensitivity must lie in the range [0;32]");
            Terminate(defs::ERR.into());
        }
    }

    if opt.debug > 0 {
        debug_level = opt.debug.into();
    }

    if let Some(scale) = opt.scale {
        if scale <= 0. {
            error!("illegal scale entered, needs to be >0: {}", scale);
            Terminate(defs::ERR.into());
        }
        GameConfig.scale = scale;
        info!("Graphics scale set to {}", scale);
    }

    if opt.fullscreen {
        GameConfig.UseFullscreen = true.into();
    } else if opt.window {
        GameConfig.UseFullscreen = false.into();
    }
}

/// find all themes and put them in AllThemes
#[no_mangle]
pub unsafe extern "C" fn FindAllThemes() {
    use std::fs;

    let mut classic_theme_index: usize = 0; // default: override when we actually find 'classic' theme

    // just to make sure...
    AllThemes.num_themes = 0;
    AllThemes
        .theme_name
        .iter_mut()
        .filter(|name| name.is_null().not())
        .for_each(|name| {
            libc::free(*name as *mut c_void);
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

                        let theme_exists = AllThemes
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
                                classic_theme_index = AllThemes.num_themes.try_into().unwrap();
                            }
                            let new_theme = &mut AllThemes.theme_name
                                [usize::try_from(AllThemes.num_themes).unwrap()];
                            *new_theme =
                                MyMalloc((theme_name.len() + 1).try_into().unwrap()) as *mut u8;
                            std::ptr::copy_nonoverlapping(
                                theme_name.as_ptr(),
                                *new_theme,
                                theme_name.len(),
                            );
                            *new_theme.add(theme_name.len()) = b'\0';

                            AllThemes.num_themes += 1;
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
    if AllThemes.num_themes == 0 {
        error!("no valid graphic-themes found!!");
        error!("You need to install at least one to run Freedroid!!");
        Terminate(defs::ERR.into());
    }

    let selected_theme_index = AllThemes.theme_name
        [..usize::try_from(AllThemes.num_themes).unwrap()]
        .iter()
        .copied()
        .position(|theme_name| {
            libc::strcmp(theme_name as *const _, GameConfig.Theme_Name.as_mut_ptr()) == 0
        });

    match selected_theme_index {
        Some(index) => {
            info!(
                "Found selected theme {} from GameConfig.",
                CStr::from_ptr(GameConfig.Theme_Name.as_ptr()).to_string_lossy(),
            );
            AllThemes.cur_tnum = index.try_into().unwrap();
        }
        None => {
            warn!(
                "selected theme {} not valid! Using classic theme.",
                CStr::from_ptr(GameConfig.Theme_Name.as_ptr()).to_string_lossy(),
            );
            libc::strcpy(
                GameConfig.Theme_Name.as_mut_ptr(),
                AllThemes.theme_name[classic_theme_index] as *const _,
            );
            AllThemes.cur_tnum = classic_theme_index.try_into().unwrap();
        }
    }

    info!(
        "Game starts using theme: {}",
        CStr::from_ptr(GameConfig.Theme_Name.as_ptr()).to_string_lossy()
    );
}

#[no_mangle]
pub unsafe extern "C" fn InitNewMission(mission_name: *mut c_char) {
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
    const MISSION_ENDTITLE_BEGIN_STRING: &CStr = cstr!("** Beginning of End Title Text Section **");
    const MISSION_ENDTITLE_END_STRING: &CStr = cstr!("** End of End Title Text Section **");
    const MISSION_START_POINT_STRING: &CStr = cstr!("Possible Start Point : ");

    // We store the mission name in case the influ
    // gets destroyed so we know where to continue in
    // case the player doesn't want to return to the very beginning
    // but just to replay this mission.
    libc::strcpy(Previous_Mission_Name.as_mut_ptr(), mission_name);

    info!(
        "A new mission is being initialized from file {}.",
        CStr::from_ptr(mission_name).to_string_lossy()
    );

    //--------------------
    //At first we do the things that must be done for all
    //missions, regardless of mission file given
    Activate_Conservative_Frame_Computation();
    LastGotIntoBlastSound = 2.;
    LastRefreshSound = 2.;
    ThisMessageTime = 0;
    LevelDoorsNotMovedTime = 0.0;
    DeathCount = 0.;
    set_time_factor(1.0);

    /* Delete all bullets and blasts */
    for bullet in 0..MAXBULLETS {
        DeleteBullet(bullet.try_into().unwrap());
    }

    info!("InitNewMission: All bullets have been deleted.");
    for blast in &mut AllBlasts {
        blast.phase = Status::Out as c_int as c_float;
        blast.ty = Status::Out as c_int;
    }
    info!("InitNewMission: All blasts have been deleted.");
    for enemy in &mut AllEnemys {
        enemy.ty = Status::Out as c_int;
        enemy.energy = -1.;
    }
    info!("InitNewMission: All enemys have been deleted...");

    //Now its time to start decoding the mission file.
    //For that, we must get it into memory first.
    //The procedure is the same as with LoadShip

    let oldfont = GetCurrentFont();

    SetCurrentFont(Font0_BFont);
    //  printf_SDL (ne_screen, User_Rect.x + 50, -1, "Loading mission data ");

    /* Read the whole mission data to memory */
    let fpath = find_file(
        mission_name,
        MAP_DIR_C.as_ptr() as *mut c_char,
        Themed::NoTheme as c_int,
        Criticality::Critical as c_int,
    );

    let main_mission_pointer =
        ReadAndMallocAndTerminateFile(fpath, END_OF_MISSION_DATA_STRING.as_ptr() as *mut c_char);

    //--------------------
    // Now the mission file is read into memory.  That means we can start to decode the details given
    // in the body of the mission file.

    //--------------------
    // First we extract the game physics file name from the
    // mission file and load the game data.
    //
    let mut buffer: [c_char; 500] = [0; 500];
    ReadValueFromString(
        main_mission_pointer,
        GAMEDATANAME_INDICATION_STRING.as_ptr() as *mut c_char,
        cstr!("%s").as_ptr() as *mut c_char,
        buffer.as_mut_ptr() as *mut c_void,
    );

    Init_Game_Data(buffer.as_mut_ptr());

    //--------------------
    // Now its time to get the shipname from the mission file and
    // read the ship file into the right memory structures
    //
    ReadValueFromString(
        main_mission_pointer,
        SHIPNAME_INDICATION_STRING.as_ptr() as *mut c_char,
        cstr!("%s").as_ptr() as *mut c_char,
        buffer.as_mut_ptr() as *mut c_void,
    );

    if LoadShip(buffer.as_mut_ptr()) == defs::ERR.into() {
        error!("Error in LoadShip");
        Terminate(defs::ERR.into());
    }
    //--------------------
    // Now its time to get the elevator file name from the mission file and
    // read the elevator file into the right memory structures
    //
    ReadValueFromString(
        main_mission_pointer,
        ELEVATORNAME_INDICATION_STRING.as_ptr() as *mut c_char,
        cstr!("%s").as_ptr() as *mut c_char,
        buffer.as_mut_ptr() as *mut c_void,
    );

    if GetLiftConnections(buffer.as_mut_ptr()) == defs::ERR.into() {
        error!("Error in GetLiftConnections");
        Terminate(defs::ERR.into());
    }
    //--------------------
    // We also load the comment for the influencer to say at the beginning of the mission
    //

    // NO! these strings are allocated elsewhere or even static, so free'ing them
    // here would SegFault eventually!
    //  if (Me.TextToBeDisplayed) free (Me.TextToBeDisplayed);

    Me.TextToBeDisplayed = cstr!("Ok. I'm on board.  Let's get to work.").as_ptr() as *mut c_char; // taken from Paradroid.mission
    Me.TextVisibleTime = 0.;

    //--------------------
    // Now its time to get the crew file name from the mission file and
    // assemble an appropriate crew out of it
    //
    ReadValueFromString(
        main_mission_pointer,
        CREWNAME_INDICATION_STRING.as_ptr() as *mut c_char,
        cstr!("%s").as_ptr() as *mut c_char,
        buffer.as_mut_ptr() as *mut c_void,
    );

    /* initialize enemys according to crew file */
    // WARNING!! THIS REQUIRES THE freedroid.ruleset FILE TO BE READ ALREADY, BECAUSE
    // ROBOT SPECIFICATIONS ARE ALREADY REQUIRED HERE!!!!!
    if GetCrew(buffer.as_mut_ptr()) == defs::ERR.into() {
        error!("InitNewGame(): Initialization of enemys failed.",);
        Terminate(defs::ERR.into());
    }

    //--------------------
    // Now its time to get the debriefing text from the mission file so that it
    // can be used, if the mission is completed and also the end title music name
    // must be read in as well
    ReadValueFromString(
        main_mission_pointer,
        MISSION_ENDTITLE_SONG_NAME_STRING.as_ptr() as *mut c_char,
        cstr!("%s").as_ptr() as *mut c_char,
        DebriefingSong.as_mut_ptr() as *mut c_void,
    );

    if DebriefingText.is_null().not() {
        libc::free(DebriefingText as *mut c_void);
    }
    DebriefingText = ReadAndMallocStringFromData(
        main_mission_pointer,
        MISSION_ENDTITLE_BEGIN_STRING.as_ptr() as *mut c_char,
        MISSION_ENDTITLE_END_STRING.as_ptr() as *mut c_char,
    );

    //--------------------
    // Now we read all the possible starting points for the
    // current mission file, so that we know where to place the
    // influencer at the beginning of the mission.

    let number_of_start_points = CountStringOccurences(
        main_mission_pointer,
        MISSION_START_POINT_STRING.as_ptr() as *mut c_char,
    );

    if number_of_start_points == 0 {
        error!("NOT EVEN ONE SINGLE STARTING POINT ENTRY FOUND!  TERMINATING!",);
        Terminate(defs::ERR.into());
    }
    info!(
        "Found {} different starting points for the mission in the mission file.",
        number_of_start_points,
    );

    // Now that we know how many different starting points there are, we can randomly select
    // one of them and read then in this one starting point into the right structures...
    let real_start_point = MyRandom(number_of_start_points - 1) + 1;
    let mut start_point_pointer = main_mission_pointer;
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
    CurLevel = curShip.AllLevels[usize::try_from(starting_level).unwrap()];
    start_point_pointer = libc::strstr(start_point_pointer, cstr!("XPos=").as_ptr())
        .add(libc::strlen(cstr!("XPos=").as_ptr()));
    libc::sscanf(
        start_point_pointer,
        cstr!("%d").as_ptr() as *mut c_char,
        &mut starting_x_pos,
    );
    Me.pos.x = starting_x_pos as c_float;
    start_point_pointer = libc::strstr(start_point_pointer, cstr!("YPos=").as_ptr())
        .add(libc::strlen(cstr!("YPos=").as_ptr()));
    libc::sscanf(
        start_point_pointer,
        cstr!("%d").as_ptr() as *mut c_char,
        &mut starting_y_pos,
    );
    Me.pos.y = starting_y_pos as c_float;
    info!(
        "Final starting position: Level={} XPos={} YPos={}.",
        starting_level, starting_x_pos, starting_y_pos,
    );

    /* Reactivate the light on alle Levels, that might have been dark */
    for &level in &curShip.AllLevels[0..usize::try_from(curShip.num_levels).unwrap()] {
        (*level).empty = false.into();
    }

    info!("InitNewMission: All levels have been set to 'active'...",);

    //--------------------
    // At this point the position history can be initialized
    //
    InitInfluPositionHistory();
    //  printf_SDL (ne_screen, -1, -1, ".");

    //  printf_SDL (ne_screen, -1, -1, " ok\n");
    SetCurrentFont(oldfont);
    //--------------------
    // We start with doing the briefing things...
    // Now we search for the beginning of the mission briefing big section NOT subsection.
    // We display the title and explanation of controls and such...
    let briefing_section_pointer = LocateStringInData(
        main_mission_pointer,
        MISSION_BRIEFING_BEGIN_STRING.as_ptr() as *mut c_char,
    );
    Title(briefing_section_pointer);

    /* Den Banner fuer das Spiel anzeigen */
    ClearGraphMem();
    DisplayBanner(
        null_mut(),
        null_mut(),
        DisplayBannerFlags::FORCE_UPDATE.bits().into(),
    );

    // Switch_Background_Music_To (COMBAT_BACKGROUND_MUSIC_SOUND);
    Switch_Background_Music_To((*CurLevel).Background_Song_Name);

    for level in &curShip.AllLevels[..usize::try_from(curShip.num_levels).unwrap()] {
        CurLevel = *level;
        ShuffleEnemys();
    }

    CurLevel = curShip.AllLevels[usize::try_from(starting_level).unwrap()];

    // Now that the briefing and all that is done,
    // the influence structure can be initialized for
    // the new mission:
    Me.ty = Droid::Droid001 as c_int;
    Me.speed.x = 0.;
    Me.speed.y = 0.;
    Me.energy = (*Druidmap.add(Droid::Droid001 as usize)).maxenergy;
    Me.health = Me.energy; /* start with max. health */
    Me.status = Status::Mobile as c_int;
    Me.phase = 0.;
    Me.timer = 0.0; // set clock to 0

    info!("done."); // this matches the printf at the beginning of this function

    libc::free(main_mission_pointer as *mut c_void);
}
