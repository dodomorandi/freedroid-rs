use crate::{
    b_font::{FontHeight, GetCurrentFont, Para_BFont, SetCurrentFont},
    bullet::{DeleteBullet, ExplodeBlasts, MoveBullets},
    curShip, debug_level,
    defs::{
        self, get_user_center, scale_rect, AssembleCombatWindowFlags, Criticality,
        DisplayBannerFlags, Droid, Status, Themed, FD_DATADIR, GRAPHICS_DIR_C, LOCAL_DATADIR,
        MAP_DIR_C, MAXBULLETS, SHOW_WAIT, SLOWMO_FACTOR, TITLE_PIC_FILE_C, WAIT_AFTER_KILLED,
    },
    enemy::{MoveEnemys, ShuffleEnemys},
    global::{
        collision_lose_energy_calibrator, num_highscores, Blast_Damage_Per_Second, Blast_Radius,
        Blastmap, Bulletmap, CurrentCombatScaleFactor, Droid_Radius, Druidmap, Font0_BFont,
        GameConfig, Highscores, SkipAFewFrames, Time_For_Each_Phase_Of_Door_Movement,
    },
    graphics::{
        ne_screen, pic999, white_noise, AllThemes, ClearGraphMem, DisplayImage, InitPictures,
        Init_Video, Load_Fonts, MakeGridOnScreen, Number_Of_Bullet_Types,
    },
    highscore::{InitHighscores, UpdateHighscores},
    influencer::{ExplodeInfluencer, InitInfluPositionHistory},
    input::{
        any_key_just_pressed, wait_for_all_keys_released, wait_for_key_pressed, Init_Joy, SDL_Delay,
    },
    map::{GetCrew, GetLiftConnections, LoadShip},
    misc::{
        find_file, init_progress, set_time_factor, update_progress,
        Activate_Conservative_Frame_Computation, ComputeFPSForThisFrame, CountStringOccurences,
        LoadGameConfig, LocateStringInData, MyMalloc, MyRandom, ReadAndMallocAndTerminateFile,
        ReadAndMallocStringFromData, ReadValueFromString, StartTakingTimeForFPSCalculation,
        Terminate,
    },
    sound::{Init_Audio, Switch_Background_Music_To, ThouArtDefeatedSound},
    sound_on,
    structs::{BulletSpec, DruidSpec},
    text::{printf_SDL, DisplayText, ScrollText},
    vars::{Classic_User_Rect, Full_User_Rect, Portrait_Rect, Screen_Rect, User_Rect},
    view::{Assemble_Combat_Picture, DisplayBanner},
    AlertBonusPerSec, AlertThreshold, AllBlasts, AllBullets, AllEnemys, CurLevel, DeathCount,
    DeathCountDrainSpeed, GameOver, LastGotIntoBlastSound, LastRefreshSound,
    LevelDoorsNotMovedTime, Me, NumEnemys, Number_Of_Droid_Types, RealScore, ShowScore,
    ThisMessageTime,
};

use clap::{crate_version, Clap};
use cstr::cstr;
use log::{error, info, warn};
use sdl::{
    event::ll::SDL_DISABLE,
    ll::SDL_GetTicks,
    mouse::ll::SDL_ShowCursor,
    video::ll::{SDL_Flip, SDL_FreeSurface, SDL_SetClipRect, SDL_UpperBlit},
    Rect,
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
    pub fn Mix_HaltMusic() -> c_int;
}

static mut DEBRIEFING_TEXT: *mut c_char = null_mut();
static mut DEBRIEFING_SONG: [c_char; 500] = [0; 500];
static mut PREVIOUS_MISSION_NAME: [c_char; 500] = [0; 500];

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
    libc::free(DEBRIEFING_TEXT as *mut c_void);
    DEBRIEFING_TEXT = null_mut();
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
    Switch_Background_Music_To(DEBRIEFING_SONG.as_ptr());

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
    ScrollText(DEBRIEFING_TEXT, &mut rect, 6);

    wait_for_all_keys_released();
}

/// This function initializes the whole Freedroid game.
///
/// This must not be confused with initnewgame, which
/// only initializes a new mission for the game.
#[no_mangle]
pub unsafe extern "C" fn InitFreedroid() {
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
    libc::strcpy(PREVIOUS_MISSION_NAME.as_mut_ptr(), mission_name);

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
        DEBRIEFING_SONG.as_mut_ptr() as *mut c_void,
    );

    if DEBRIEFING_TEXT.is_null().not() {
        libc::free(DEBRIEFING_TEXT as *mut c_void);
    }
    DEBRIEFING_TEXT = ReadAndMallocStringFromData(
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

///  This function does the mission briefing.  It assumes,
///  that a mission file has already been successfully loaded into
///  memory.  The briefing texts will be extracted and displayed in
///  scrolling font.
#[no_mangle]
pub unsafe extern "C" fn Title(mission_briefing_pointer: *mut c_char) {
    const BRIEFING_TITLE_PICTURE_STRING: &CStr =
        cstr!("The title picture in the graphics subdirectory for this mission is : ");
    const BRIEFING_TITLE_SONG_STRING: &CStr =
        cstr!("The title song in the sound subdirectory for this mission is : ");
    const NEXT_BRIEFING_SUBSECTION_START_STRING: &CStr =
        cstr!("* New Mission Briefing Text Subsection *");
    const END_OF_BRIEFING_SUBSECTION_STRING: &CStr =
        cstr!("* End of Mission Briefing Text Subsection *");

    let mut buffer: [c_char; 500] = [0; 500];
    ReadValueFromString(
        mission_briefing_pointer,
        BRIEFING_TITLE_SONG_STRING.as_ptr() as *mut c_char,
        cstr!("%s").as_ptr() as *mut c_char,
        buffer.as_mut_ptr() as *mut c_void,
    );
    Switch_Background_Music_To(buffer.as_mut_ptr());

    SDL_SetClipRect(ne_screen, null_mut());
    ReadValueFromString(
        mission_briefing_pointer,
        BRIEFING_TITLE_PICTURE_STRING.as_ptr() as *mut c_char,
        cstr!("%s").as_ptr() as *mut c_char,
        buffer.as_mut_ptr() as *mut c_void,
    );
    DisplayImage(find_file(
        buffer.as_mut_ptr(),
        GRAPHICS_DIR_C.as_ptr() as *mut c_char,
        Themed::NoTheme as c_int,
        Criticality::Critical as c_int,
    ));
    MakeGridOnScreen(Some(&Screen_Rect));
    Me.status = Status::Briefing as c_int;
    //  SDL_Flip (ne_screen);

    SetCurrentFont(Para_BFont);

    DisplayBanner(
        null_mut(),
        null_mut(),
        DisplayBannerFlags::FORCE_UPDATE.bits().into(),
    );

    // Next we display all the subsections of the briefing section
    // with scrolling font
    let mut next_subsection_start_pointer = mission_briefing_pointer;
    let mut prepared_briefing_text = null_mut();
    loop {
        next_subsection_start_pointer = libc::strstr(
            next_subsection_start_pointer,
            NEXT_BRIEFING_SUBSECTION_START_STRING.as_ptr(),
        );
        if next_subsection_start_pointer.is_null() {
            break;
        }

        next_subsection_start_pointer = next_subsection_start_pointer
            .add(libc::strlen(NEXT_BRIEFING_SUBSECTION_START_STRING.as_ptr()));
        let termination_pointer = libc::strstr(
            next_subsection_start_pointer,
            END_OF_BRIEFING_SUBSECTION_STRING.as_ptr(),
        );
        if termination_pointer.is_null() {
            error!("Title: Unterminated Subsection in Mission briefing....Terminating...");
            Terminate(defs::ERR.into());
        }
        let this_text_length = termination_pointer.offset_from(next_subsection_start_pointer);
        libc::free(prepared_briefing_text as *mut c_void);
        prepared_briefing_text =
            MyMalloc(c_long::try_from(this_text_length).unwrap() + 10) as *mut c_char;
        libc::strncpy(
            prepared_briefing_text,
            next_subsection_start_pointer,
            this_text_length.try_into().unwrap(),
        );
        *prepared_briefing_text.offset(this_text_length) = 0;

        let mut rect = Full_User_Rect;
        rect.x += 10;
        rect.w -= 10; //leave some border
        if ScrollText(prepared_briefing_text, &mut rect, 0) == 1 {
            break; // User pressed 'fire'
        }
    }

    libc::free(prepared_briefing_text as *mut c_void);
}

/// This function loads all the constant variables of the game from
/// a dat file, that should be optimally human readable.
#[no_mangle]
pub unsafe extern "C" fn Init_Game_Data(data_filename: *mut c_char) {
    const END_OF_GAME_DAT_STRING: &CStr = cstr!("*** End of game.dat File ***");

    /* Read the whole game data to memory */
    let fpath = find_file(
        data_filename,
        MAP_DIR_C.as_ptr() as *mut c_char,
        Themed::NoTheme as c_int,
        Criticality::Critical as c_int,
    );

    let data = ReadAndMallocAndTerminateFile(fpath, END_OF_GAME_DAT_STRING.as_ptr() as *mut c_char);

    Get_General_Game_Constants(data);
    Get_Robot_Data(data as *mut c_void);
    Get_Bullet_Data(data as *mut c_void);

    // Now we read in the total time amount for the blast animations
    const BLAST_ONE_TOTAL_AMOUNT_OF_TIME_STRING: &CStr =
        cstr!("Time in seconds for the animation of blast one :");
    const BLAST_TWO_TOTAL_AMOUNT_OF_TIME_STRING: &CStr =
        cstr!("Time in seconds for the animation of blast one :");

    ReadValueFromString(
        data,
        BLAST_ONE_TOTAL_AMOUNT_OF_TIME_STRING.as_ptr() as *mut c_char,
        cstr!("%f").as_ptr() as *mut c_char,
        &mut Blastmap[0].total_animation_time as *mut f32 as *mut c_void,
    );
    ReadValueFromString(
        data,
        BLAST_TWO_TOTAL_AMOUNT_OF_TIME_STRING.as_ptr() as *mut c_char,
        cstr!("%f").as_ptr() as *mut c_char,
        &mut Blastmap[1].total_animation_time as *mut f32 as *mut c_void,
    );

    libc::free(data as *mut c_void);
}

/// This function loads all the constant variables of the game from
/// a dat file, that should be optimally human readable.
#[no_mangle]
pub unsafe extern "C" fn Get_Robot_Data(data_pointer: *mut c_void) {
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
    const LOSEHEALTH_BEGIN_STRING: &CStr = cstr!("Rate of energyloss under influence control: ");
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

    let mut robot_pointer = LocateStringInData(
        data_pointer as *mut c_char,
        ROBOT_SECTION_BEGIN_STRING.as_ptr() as *mut c_char,
    );

    info!("Starting to read robot calibration section");

    // Now we read in the speed calibration factor for all droids
    ReadValueFromString(
        robot_pointer,
        MAXSPEED_CALIBRATOR_STRING.as_ptr() as *mut c_char,
        cstr!("%f").as_ptr() as *mut c_char,
        &mut maxspeed_calibrator as *mut _ as *mut c_void,
    );

    // Now we read in the acceleration calibration factor for all droids
    ReadValueFromString(
        robot_pointer,
        ACCELERATION_CALIBRATOR_STRING.as_ptr() as *mut c_char,
        cstr!("%f").as_ptr() as *mut c_char,
        &mut acceleration_calibrator as *mut _ as *mut c_void,
    );

    // Now we read in the maxenergy calibration factor for all droids
    ReadValueFromString(
        robot_pointer,
        MAXENERGY_CALIBRATOR_STRING.as_ptr() as *mut c_char,
        cstr!("%f").as_ptr() as *mut c_char,
        &mut maxenergy_calibrator as *mut _ as *mut c_void,
    );

    // Now we read in the energy_loss calibration factor for all droids
    ReadValueFromString(
        robot_pointer,
        ENERGYLOSS_CALIBRATOR_STRING.as_ptr() as *mut c_char,
        cstr!("%f").as_ptr() as *mut c_char,
        &mut energyloss_calibrator as *mut _ as *mut c_void,
    );

    // Now we read in the aggression calibration factor for all droids
    ReadValueFromString(
        robot_pointer,
        AGGRESSION_CALIBRATOR_STRING.as_ptr() as *mut c_char,
        cstr!("%f").as_ptr() as *mut c_char,
        &mut aggression_calibrator as *mut _ as *mut c_void,
    );

    // Now we read in the score calibration factor for all droids
    ReadValueFromString(
        robot_pointer,
        SCORE_CALIBRATOR_STRING.as_ptr() as *mut c_char,
        cstr!("%f").as_ptr() as *mut c_char,
        &mut score_calibrator as *mut _ as *mut c_void,
    );

    info!("Starting to read Robot data...");

    // cleanup if previously allocated:
    FreeDruidmap();

    // At first, we must allocate memory for the droid specifications.
    // How much?  That depends on the number of droids defined in freedroid.ruleset.
    // So we have to count those first.  ok.  lets do it.
    Number_Of_Droid_Types = CountStringOccurences(
        data_pointer as *mut c_char,
        NEW_ROBOT_BEGIN_STRING.as_ptr() as *mut c_char,
    );

    // Now that we know how many robots are defined in freedroid.ruleset, we can allocate
    // a fitting amount of memory.
    let mem = usize::try_from(Number_Of_Droid_Types).unwrap() * std::mem::size_of::<DruidSpec>();
    Druidmap = MyMalloc(mem.try_into().unwrap()) as *mut DruidSpec;
    info!(
        "We have counted {} different druid types in the game data file.",
        Number_Of_Droid_Types,
    );
    info!("MEMORY HAS BEEN ALLOCATED. THE READING CAN BEGIN.");

    //Now we start to read the values for each robot:
    //Of which parts is it composed, which stats does it have?
    let mut robot_index = 0;
    while {
        robot_pointer = libc::strstr(robot_pointer, NEW_ROBOT_BEGIN_STRING.as_ptr());
        robot_pointer.is_null().not()
    } {
        info!("Found another Robot specification entry!  Lets add that to the others!");
        robot_pointer = robot_pointer.add(1); // to avoid doubly taking this entry

        // Now we read in the Name of this droid.  We consider as a name the rest of the
        ReadValueFromString(
            robot_pointer,
            DROIDNAME_BEGIN_STRING.as_ptr() as *mut c_char,
            cstr!("%s").as_ptr() as *mut c_char,
            &mut (*Druidmap.add(robot_index)).druidname as *mut _ as *mut c_void,
        );

        // Now we read in the maximal speed this droid can go.
        ReadValueFromString(
            robot_pointer,
            MAXSPEED_BEGIN_STRING.as_ptr() as *mut c_char,
            cstr!("%f").as_ptr() as *mut c_char,
            &mut (*Druidmap.add(robot_index)).maxspeed as *mut _ as *mut c_void,
        );

        // Now we read in the class of this droid.
        ReadValueFromString(
            robot_pointer,
            CLASS_BEGIN_STRING.as_ptr() as *mut c_char,
            cstr!("%d").as_ptr() as *mut c_char,
            &mut (*Druidmap.add(robot_index)).class as *mut _ as *mut c_void,
        );

        // Now we read in the maximal acceleration this droid can go.
        ReadValueFromString(
            robot_pointer,
            ACCELERATION_BEGIN_STRING.as_ptr() as *mut c_char,
            cstr!("%f").as_ptr() as *mut c_char,
            &mut (*Druidmap.add(robot_index)).accel as *mut _ as *mut c_void,
        );

        // Now we read in the maximal energy this droid can store.
        ReadValueFromString(
            robot_pointer,
            MAXENERGY_BEGIN_STRING.as_ptr() as *mut c_char,
            cstr!("%f").as_ptr() as *mut c_char,
            &mut (*Druidmap.add(robot_index)).maxenergy as *mut _ as *mut c_void,
        );

        // Now we read in the lose_health rate.
        ReadValueFromString(
            robot_pointer,
            LOSEHEALTH_BEGIN_STRING.as_ptr() as *mut c_char,
            cstr!("%f").as_ptr() as *mut c_char,
            &mut (*Druidmap.add(robot_index)).lose_health as *mut _ as *mut c_void,
        );

        // Now we read in the class of this droid.
        ReadValueFromString(
            robot_pointer,
            GUN_BEGIN_STRING.as_ptr() as *mut c_char,
            cstr!("%d").as_ptr() as *mut c_char,
            &mut (*Druidmap.add(robot_index)).gun as *mut _ as *mut c_void,
        );

        // Now we read in the aggression rate of this droid.
        ReadValueFromString(
            robot_pointer,
            AGGRESSION_BEGIN_STRING.as_ptr() as *mut c_char,
            cstr!("%d").as_ptr() as *mut c_char,
            &mut (*Druidmap.add(robot_index)).aggression as *mut _ as *mut c_void,
        );

        // Now we read in the flash immunity of this droid.
        ReadValueFromString(
            robot_pointer,
            FLASHIMMUNE_BEGIN_STRING.as_ptr() as *mut c_char,
            cstr!("%d").as_ptr() as *mut c_char,
            &mut (*Druidmap.add(robot_index)).flashimmune as *mut _ as *mut c_void,
        );

        // Now we score to be had for destroying one droid of this type
        ReadValueFromString(
            robot_pointer,
            SCORE_BEGIN_STRING.as_ptr() as *mut c_char,
            cstr!("%d").as_ptr() as *mut c_char,
            &mut (*Druidmap.add(robot_index)).score as *mut _ as *mut c_void,
        );

        // Now we read in the height of this droid of this type
        ReadValueFromString(
            robot_pointer,
            HEIGHT_BEGIN_STRING.as_ptr() as *mut c_char,
            cstr!("%f").as_ptr() as *mut c_char,
            &mut (*Druidmap.add(robot_index)).height as *mut _ as *mut c_void,
        );

        // Now we read in the weight of this droid type
        ReadValueFromString(
            robot_pointer,
            WEIGHT_BEGIN_STRING.as_ptr() as *mut c_char,
            cstr!("%d").as_ptr() as *mut c_char,
            &mut (*Druidmap.add(robot_index)).weight as *mut _ as *mut c_void,
        );

        // Now we read in the drive of this droid of this type
        ReadValueFromString(
            robot_pointer,
            DRIVE_BEGIN_STRING.as_ptr() as *mut c_char,
            cstr!("%d").as_ptr() as *mut c_char,
            &mut (*Druidmap.add(robot_index)).drive as *mut _ as *mut c_void,
        );

        // Now we read in the brain of this droid of this type
        ReadValueFromString(
            robot_pointer,
            BRAIN_BEGIN_STRING.as_ptr() as *mut c_char,
            cstr!("%d").as_ptr() as *mut c_char,
            &mut (*Druidmap.add(robot_index)).brain as *mut _ as *mut c_void,
        );

        // Now we read in the sensor 1, 2 and 3 of this droid type
        ReadValueFromString(
            robot_pointer,
            SENSOR1_BEGIN_STRING.as_ptr() as *mut c_char,
            cstr!("%d").as_ptr() as *mut c_char,
            &mut (*Druidmap.add(robot_index)).sensor1 as *mut _ as *mut c_void,
        );
        ReadValueFromString(
            robot_pointer,
            SENSOR2_BEGIN_STRING.as_ptr() as *mut c_char,
            cstr!("%d").as_ptr() as *mut c_char,
            &mut (*Druidmap.add(robot_index)).sensor2 as *mut _ as *mut c_void,
        );
        ReadValueFromString(
            robot_pointer,
            SENSOR3_BEGIN_STRING.as_ptr() as *mut c_char,
            cstr!("%d").as_ptr() as *mut c_char,
            &mut (*Druidmap.add(robot_index)).sensor3 as *mut _ as *mut c_void,
        );

        // Now we read in the notes concerning this droid.  We consider as notes all the rest of the
        // line after the NOTES_BEGIN_STRING until the "\n" is found.
        (*Druidmap.add(robot_index)).notes = ReadAndMallocStringFromData(
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

    for droid in std::slice::from_raw_parts_mut(Druidmap, Number_Of_Droid_Types.try_into().unwrap())
    {
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
#[no_mangle]
pub unsafe extern "C" fn Get_Bullet_Data(data_pointer: *mut c_void) {
    // const BULLET_SECTION_BEGIN_STRING: &CStr = cstr!("*** Start of Bullet Data Section: ***");
    // const BULLET_SECTION_END_STRING: &CStr = cstr!("*** End of Bullet Data Section: ***");
    const NEW_BULLET_TYPE_BEGIN_STRING: &CStr =
        cstr!("** Start of new bullet specification subsection **");

    const BULLET_RECHARGE_TIME_BEGIN_STRING: &CStr =
        cstr!("Time is takes to recharge this bullet/weapon in seconds :");
    const BULLET_SPEED_BEGIN_STRING: &CStr = cstr!("Flying speed of this bullet type :");
    const BULLET_DAMAGE_BEGIN_STRING: &CStr = cstr!("Damage cause by a hit of this bullet type :");
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

    Number_Of_Bullet_Types = CountStringOccurences(
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
    if Bulletmap.is_null() {
        let mem =
            usize::try_from(Number_Of_Bullet_Types).unwrap() * std::mem::size_of::<BulletSpec>();
        Bulletmap = MyMalloc(mem.try_into().unwrap()) as *mut BulletSpec;
        std::ptr::write_bytes(
            Bulletmap,
            0,
            usize::try_from(Number_Of_Bullet_Types).unwrap(),
        );
        info!(
            "We have counted {} different bullet types in the game data file.",
            Number_Of_Bullet_Types
        );
        info!("MEMORY HAS BEEN ALLOCATED. THE READING CAN BEGIN.");
    }

    //--------------------
    // Now we start to read the values for each bullet type:
    //
    let mut bullet_pointer = data_pointer as *mut c_char;
    let mut bullet_index = 0;
    while {
        bullet_pointer = libc::strstr(bullet_pointer, NEW_BULLET_TYPE_BEGIN_STRING.as_ptr());
        bullet_pointer.is_null().not()
    } {
        info!("Found another Bullet specification entry!  Lets add that to the others!");
        bullet_pointer = bullet_pointer.add(1); // to avoid doubly taking this entry

        // Now we read in the recharging time for this bullettype(=weapontype)
        ReadValueFromString(
            bullet_pointer,
            BULLET_RECHARGE_TIME_BEGIN_STRING.as_ptr() as *mut c_char,
            cstr!("%f").as_ptr() as *mut c_char,
            &mut (*Bulletmap.add(bullet_index)).recharging_time as *mut _ as *mut c_void,
        );

        // Now we read in the maximal speed this type of bullet can go.
        ReadValueFromString(
            bullet_pointer,
            BULLET_SPEED_BEGIN_STRING.as_ptr() as *mut c_char,
            cstr!("%f").as_ptr() as *mut c_char,
            &mut (*Bulletmap.add(bullet_index)).speed as *mut _ as *mut c_void,
        );

        // Now we read in the damage this bullet can do
        ReadValueFromString(
            bullet_pointer,
            BULLET_DAMAGE_BEGIN_STRING.as_ptr() as *mut c_char,
            cstr!("%d").as_ptr() as *mut c_char,
            &mut (*Bulletmap.add(bullet_index)).damage as *mut _ as *mut c_void,
        );

        // Now we read in the number of phases that are designed for this bullet type
        // THIS IS NOW SPECIFIED IN THE THEME CONFIG FILE
        // ReadValueFromString( BulletPointer ,  BULLET_NUMBER_OF_PHASES_BEGIN_STRING , "%d" ,
        // &(*Bulletmap.add(BulletIndex)).phases , EndOfBulletData );

        // Now we read in the type of blast this bullet will cause when crashing e.g. against the wall
        ReadValueFromString(
            bullet_pointer,
            BULLET_BLAST_TYPE_CAUSED_BEGIN_STRING.as_ptr() as *mut c_char,
            cstr!("%d").as_ptr() as *mut c_char,
            &mut (*Bulletmap.add(bullet_index)).blast as *mut _ as *mut c_void,
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
    ReadValueFromString(
        data_pointer as *mut c_char,
        BULLET_SPEED_CALIBRATOR_STRING.as_ptr() as *mut c_char,
        cstr!("%f").as_ptr() as *mut c_char,
        &mut bullet_speed_calibrator as *mut _ as *mut c_void,
    );

    // Now we read in the damage calibration factor for all bullets
    ReadValueFromString(
        data_pointer as *mut c_char,
        BULLET_DAMAGE_CALIBRATOR_STRING.as_ptr() as *mut c_char,
        cstr!("%f").as_ptr() as *mut c_char,
        &mut bullet_damage_calibrator as *mut _ as *mut c_void,
    );

    // Now that all the calibrations factors have been read in, we can start to
    // apply them to all the bullet types
    for bullet in
        std::slice::from_raw_parts_mut(Bulletmap, usize::try_from(Number_Of_Bullet_Types).unwrap())
    {
        bullet.speed *= bullet_speed_calibrator;
        bullet.damage = (bullet.damage as f32 * bullet_damage_calibrator) as c_int;
    }
}

/// This function loads all the constant variables of the game from
/// a dat file, that should be optimally human readable.
#[no_mangle]
pub unsafe extern "C" fn Get_General_Game_Constants(data: *mut c_char) {
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
    ReadValueFromString(
        data,
        DEATHCOUNT_DRAIN_SPEED_STRING.as_ptr() as *mut c_char,
        cstr!("%f").as_ptr() as *mut c_char,
        &mut DeathCountDrainSpeed as *mut _ as *mut c_void,
    );
    ReadValueFromString(
        data,
        ALERT_THRESHOLD_STRING.as_ptr() as *mut c_char,
        cstr!("%d").as_ptr() as *mut c_char,
        &mut AlertThreshold as *mut _ as *mut c_void,
    );
    ReadValueFromString(
        data,
        ALERT_BONUS_PER_SEC_STRING.as_ptr() as *mut c_char,
        cstr!("%f").as_ptr() as *mut c_char,
        &mut AlertBonusPerSec as *mut _ as *mut c_void,
    );

    // Now we read in the speed calibration factor for all bullets
    ReadValueFromString(
        data,
        COLLISION_LOSE_ENERGY_CALIBRATOR_STRING.as_ptr() as *mut c_char,
        cstr!("%f").as_ptr() as *mut c_char,
        &mut collision_lose_energy_calibrator as *mut _ as *mut c_void,
    );

    // Now we read in the blast radius
    ReadValueFromString(
        data,
        BLAST_RADIUS_SPECIFICATION_STRING.as_ptr() as *mut c_char,
        cstr!("%f").as_ptr() as *mut c_char,
        &mut Blast_Radius as *mut _ as *mut c_void,
    );

    // Now we read in the druid 'radius' in x direction
    ReadValueFromString(
        data,
        DROID_RADIUS_SPECIFICATION_STRING.as_ptr() as *mut c_char,
        cstr!("%f").as_ptr() as *mut c_char,
        &mut Droid_Radius as *mut _ as *mut c_void,
    );

    // Now we read in the blast damage amount per 'second' of contact with the blast
    ReadValueFromString(
        data,
        BLAST_DAMAGE_SPECIFICATION_STRING.as_ptr() as *mut c_char,
        cstr!("%f").as_ptr() as *mut c_char,
        &mut Blast_Damage_Per_Second as *mut _ as *mut c_void,
    );

    // Now we read in the time is takes for the door to move one phase
    ReadValueFromString(
        data,
        TIME_FOR_DOOR_MOVEMENT_SPECIFICATION_STRING.as_ptr() as *mut c_char,
        cstr!("%f").as_ptr() as *mut c_char,
        &mut Time_For_Each_Phase_Of_Door_Movement as *mut _ as *mut c_void,
    );
}

/// Show end-screen
#[no_mangle]
pub unsafe extern "C" fn ThouArtDefeated() {
    Me.status = Status::Terminated as c_int;
    SDL_ShowCursor(SDL_DISABLE);

    ExplodeInfluencer();

    wait_for_all_keys_released();

    let mut now = SDL_GetTicks();

    while (SDL_GetTicks() - now) < WAIT_AFTER_KILLED {
        // add "slow motion effect" for final explosion
        set_time_factor(SLOWMO_FACTOR);

        StartTakingTimeForFPSCalculation();
        DisplayBanner(null_mut(), null_mut(), 0);
        ExplodeBlasts();
        MoveBullets();
        MoveEnemys();
        Assemble_Combat_Picture(AssembleCombatWindowFlags::DO_SCREEN_UPDATE.bits().into());
        ComputeFPSForThisFrame();
        if any_key_just_pressed() != 0 {
            break;
        }
    }
    set_time_factor(1.0);

    Mix_HaltMusic();

    // important!!: don't forget to stop fps calculation here (bugfix: enemy piles after gameOver)
    Activate_Conservative_Frame_Computation();

    white_noise(
        ne_screen,
        &mut User_Rect,
        WAIT_AFTER_KILLED.try_into().unwrap(),
    );

    Assemble_Combat_Picture(AssembleCombatWindowFlags::DO_SCREEN_UPDATE.bits().into());
    MakeGridOnScreen(Some(&User_Rect));

    let mut dst = Rect {
        x: get_user_center().x - i16::try_from(Portrait_Rect.w / 2).unwrap(),
        y: get_user_center().y - i16::try_from(Portrait_Rect.h / 2).unwrap(),
        w: Portrait_Rect.w,
        h: Portrait_Rect.h,
    };
    SDL_UpperBlit(pic999, null_mut(), ne_screen, &mut dst);
    ThouArtDefeatedSound();

    SetCurrentFont(Para_BFont);
    let h = FontHeight(&*Para_BFont);
    DisplayText(
        cstr!("Transmission").as_ptr() as *mut c_char,
        i32::from(dst.x) - h,
        i32::from(dst.y) - h,
        &User_Rect,
    );
    DisplayText(
        cstr!("Terminated").as_ptr() as *mut c_char,
        i32::from(dst.x) - h,
        i32::from(dst.y) + i32::from(dst.h),
        &User_Rect,
    );
    printf_SDL(ne_screen, -1, -1, cstr!("\n").as_ptr() as *mut c_char);
    SDL_Flip(ne_screen);

    now = SDL_GetTicks();

    wait_for_all_keys_released();
    while SDL_GetTicks() - now < SHOW_WAIT {
        SDL_Delay(1);
        if any_key_just_pressed() != 0 {
            break;
        }
    }

    UpdateHighscores();

    GameOver = true.into();
}
