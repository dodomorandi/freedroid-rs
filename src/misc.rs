#[cfg(feature = "gcw0")]
use crate::defs::{gcw0_ls_pressed_r, gcw0_rs_pressed_r};
use crate::{
    defs::{
        self, scale_rect, AssembleCombatWindowFlags, Cmds, Criticality, FirePressedR, Status,
        Themed, FD_DATADIR, GRAPHICS_DIR_C, LOCAL_DATADIR, PROGRESS_FILLER_FILE_C,
        PROGRESS_METER_FILE_C,
    },
    enemy::AnimateEnemys,
    global::{
        ConfigDir, FPSover1, GameConfig, Me, ProgressBar_Rect, ProgressMeter_Rect,
        ProgressText_Rect, SkipAFewFrames,
    },
    graphics::{
        ne_screen, progress_filler_pic, progress_meter_pic, BannerIsDestroyed, FreeGraphics,
        Load_Block, ScalePic,
    },
    highscore::SaveHighscores,
    influence::AnimateInfluence,
    init::FreeGameMem,
    input::{cmd_is_active, cmd_is_activeR, cmd_strings, key_cmds, KeyIsPressedR, SDL_Delay},
    map::{AnimateRefresh, FreeShipMemory},
    menu::FreeMenuData,
    ship::FreeDroidPics,
    sound::FreeSounds,
    text::printf_SDL,
    view::{Assemble_Combat_Picture, DisplayBanner},
};

use cstr::cstr;
use log::{error, info, warn};
use once_cell::sync::Lazy;
use sdl::{
    sdl::{
        ll::{SDL_GetTicks, SDL_Quit},
        Rect,
    },
    video::ll::{SDL_Flip, SDL_SetClipRect, SDL_UpdateRects, SDL_UpperBlit},
};
use std::{
    borrow::Cow,
    convert::{TryFrom, TryInto},
    ffi::CStr,
    fs::File,
    os::raw::{c_char, c_float, c_int, c_long, c_void},
    path::Path,
    process,
    ptr::null_mut,
    sync::RwLock,
};

extern "C" {
    pub static mut framenr: c_int;
    pub static mut One_Frame_SDL_Ticks: u32;
    pub static mut Now_SDL_Ticks: u32;
    pub static mut oneframedelay: c_long;
    pub fn LocateStringInData(
        SearchBeginPointer: *mut c_char,
        SearchTextPointer: *mut c_char,
    ) -> *mut c_char;
}

static CURRENT_TIME_FACTOR: Lazy<RwLock<f32>> = Lazy::new(|| RwLock::new(1.));

#[no_mangle]
pub unsafe extern "C" fn update_progress(percent: c_int) {
    let h = (f64::from(ProgressBar_Rect.h) * f64::from(percent) / 100.) as u16;
    let mut dst = Rect::new(
        ProgressBar_Rect.x + ProgressMeter_Rect.x,
        ProgressBar_Rect.y + ProgressMeter_Rect.y + ProgressBar_Rect.h as i16 - h as i16,
        ProgressBar_Rect.w,
        h,
    );

    let mut src = Rect::new(0, ProgressBar_Rect.h as i16 - dst.h as i16, dst.h, 0);

    SDL_UpperBlit(progress_filler_pic, &mut src, ne_screen, &mut dst);
    SDL_UpdateRects(ne_screen, 1, &mut dst);
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
/// This counter is most conveniently set via the function Activate_Conservative_Frame_Computation,
/// which can be conveniently called from eveywhere.
#[no_mangle]
pub unsafe extern "C" fn Frame_Time() -> c_float {
    static mut PREVIOUS_TIME: c_float = 0.1;

    if SkipAFewFrames != 0 {
        return PREVIOUS_TIME;
    }

    if FPSover1 > 0. {
        PREVIOUS_TIME = 1.0 / FPSover1;
    }

    PREVIOUS_TIME * *CURRENT_TIME_FACTOR.read().unwrap()
}

/// Update the factor affecting the current speed of 'time flow'
#[no_mangle]
pub unsafe extern "C" fn set_time_factor(time_factor: c_float) {
    *CURRENT_TIME_FACTOR.write().unwrap() = time_factor;
}

/// This function is used for terminating freedroid.  It will close
/// the SDL submodules and exit.
#[no_mangle]
pub unsafe extern "C" fn Terminate(exit_code: c_int) -> ! {
    info!("Termination of Freedroid initiated.");

    if exit_code == defs::OK.into() {
        info!("Writing config file");
        SaveGameConfig();
        info!("Writing highscores to disk");
        SaveHighscores();
    }

    // ----- free memory
    FreeShipMemory();
    FreeDroidPics();
    FreeGraphics();
    FreeSounds();
    FreeMenuData();
    FreeGameMem();

    // ----- exit
    info!("Thank you for playing Freedroid.");
    SDL_Quit();
    process::exit(exit_code);
}

/// This function is used to generate a random integer in the range
/// from [0 to upper_bound] (inclusive), distributed uniformly.
#[no_mangle]
pub unsafe extern "C" fn MyRandom(upper_bound: c_int) -> c_int {
    // random float in [0,upper_bound+1)
    let tmp = (f64::from(upper_bound) + 1.0)
        * (f64::from(libc::rand()) / (f64::from(libc::RAND_MAX) + 1.0));
    let dice_val = tmp as c_int;

    if dice_val < 0 || dice_val > upper_bound {
        panic!("dice_val = {} not in [0, {}]", dice_val, upper_bound);
    }

    dice_val
}

/// realise Pause-Mode: the game process is halted,
/// while the graphics and animations are not.  This mode
/// can further be toggled from PAUSE to CHEESE, which is
/// a feature from the original program that should probably
/// allow for better screenshots.
#[no_mangle]
pub unsafe extern "C" fn Pause() {
    Me.status = Status::Pause as i32;
    Assemble_Combat_Picture(AssembleCombatWindowFlags::DO_SCREEN_UPDATE.bits().into());

    let mut cheese = false;
    loop {
        StartTakingTimeForFPSCalculation();

        if !cheese {
            AnimateInfluence();
            AnimateRefresh();
            AnimateEnemys();
        }

        DisplayBanner(null_mut(), null_mut(), 0);
        Assemble_Combat_Picture(AssembleCombatWindowFlags::DO_SCREEN_UPDATE.bits().into());

        SDL_Delay(1);

        ComputeFPSForThisFrame();

        #[cfg(feature = "gcw0")]
        let cond = gcw0_ls_pressed_r() || gcw0_rs_pressed_r();
        #[cfg(not(feature = "gcw0"))]
        let cond = KeyIsPressedR(b'c'.into());

        if cond {
            if Me.status != Status::Cheese as i32 {
                Me.status = Status::Cheese as i32;
            } else {
                Me.status = Status::Pause as i32;
            }
            cheese = !cheese;
        }

        if FirePressedR() || cmd_is_activeR(Cmds::Pause) {
            while cmd_is_active(Cmds::Pause) {
                SDL_Delay(1);
            }
            break;
        }
    }
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

#[no_mangle]
pub unsafe extern "C" fn SaveGameConfig() -> c_int {
    use std::io::Write;
    if ConfigDir[0] == b'\0' as c_char {
        return defs::ERR.into();
    }

    let config_path =
        Path::new(&CStr::from_ptr(ConfigDir.as_ptr()).to_str().unwrap()).join("config");
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
    writeln!(config, "{} = {}", DRAW_FRAMERATE, GameConfig.Draw_Framerate).unwrap();
    writeln!(config, "{} = {}", DRAW_ENERGY, GameConfig.Draw_Energy).unwrap();
    writeln!(config, "{} = {}", DRAW_POSITION, GameConfig.Draw_Position).unwrap();
    writeln!(
        config,
        "{} = {}",
        DRAW_DEATHCOUNT, GameConfig.Draw_DeathCount
    )
    .unwrap();
    writeln!(config, "{} = {}", DROID_TALK, GameConfig.Droid_Talk).unwrap();
    writeln!(
        config,
        "{} = {}",
        WANTED_TEXT_VISIBLE_TIME, GameConfig.WantedTextVisibleTime,
    )
    .unwrap();
    writeln!(
        config,
        "{} = {}",
        CURRENT_BG_MUSIC_VOLUME, GameConfig.Current_BG_Music_Volume,
    )
    .unwrap();
    writeln!(
        config,
        "{} = {}",
        CURRENT_SOUND_FX_VOLUME, GameConfig.Current_Sound_FX_Volume,
    )
    .unwrap();
    writeln!(
        config,
        "{} = {}",
        CURRENT_GAMMA_CORRECTION, GameConfig.Current_Gamma_Correction,
    )
    .unwrap();
    writeln!(
        config,
        "{} = {}",
        THEME_NAME,
        CStr::from_ptr(GameConfig.Theme_Name.as_ptr())
            .to_str()
            .unwrap()
    )
    .unwrap();
    writeln!(config, "{} = {}", FULL_USER_RECT, GameConfig.FullUserRect).unwrap();
    writeln!(config, "{} = {}", USE_FULLSCREEN, GameConfig.UseFullscreen).unwrap();
    writeln!(
        config,
        "{} = {}",
        TAKEOVER_ACTIVATES, GameConfig.TakeoverActivates,
    )
    .unwrap();
    writeln!(
        config,
        "{} = {}",
        FIRE_HOLD_TAKEOVER, GameConfig.FireHoldTakeover,
    )
    .unwrap();
    writeln!(config, "{} = {}", SHOW_DECALS, GameConfig.ShowDecals).unwrap();
    writeln!(config, "{} = {}", ALL_MAP_VISIBLE, GameConfig.AllMapVisible).unwrap();
    writeln!(config, "{} = {}", VID_SCALE_FACTOR, GameConfig.scale).unwrap();
    writeln!(config, "{} = {}", HOG_CPU, GameConfig.HogCPU).unwrap();
    writeln!(
        config,
        "{} = {}",
        EMPTY_LEVEL_SPEEDUP, GameConfig.emptyLevelSpeedup,
    )
    .unwrap();

    // now write the keyboard->cmd mappings
    for i in 0..Cmds::Last as usize {
        writeln!(
            config,
            "{} \t= {}_{}_{}",
            CStr::from_ptr(cmd_strings[i]).to_str().unwrap(),
            key_cmds[i][0],
            key_cmds[i][1],
            key_cmds[i][2],
        )
        .unwrap();
    }

    config.flush().unwrap();
    defs::OK.into()
}

/// This function starts the time-taking process.  Later the results
/// of this function will be used to calculate the current framerate
#[no_mangle]
pub unsafe extern "C" fn StartTakingTimeForFPSCalculation() {
    /* This ensures, that 0 is never an encountered framenr,
     * therefore count to 100 here
     * Take the time now for calculating the frame rate
     * (DO NOT MOVE THIS COMMAND PLEASE!) */
    framenr += 1;

    One_Frame_SDL_Ticks = SDL_GetTicks();
}

#[no_mangle]
pub unsafe extern "C" fn ComputeFPSForThisFrame() {
    // In the following paragraph the framerate calculation is done.
    // There are basically two ways to do this:
    // The first way is to use SDL_GetTicks(), a function measuring milliseconds
    // since the initialisation of the SDL.
    // The second way is to use gettimeofday, a standard ANSI C function I guess,
    // defined in time.h or so.
    //
    // I have arranged for a definition set in defs.h to switch between the two
    // methods of ramerate calculation.  THIS MIGHT INDEED MAKE SENSE, SINCE THERE
    // ARE SOME UNEXPLAINED FRAMERATE PHENOMENA WHICH HAVE TO TO WITH KEYBOARD
    // SPACE KEY, SO PLEASE DO NOT ERASE EITHER METHOD.  PLEASE ASK JP FIRST.
    //

    if SkipAFewFrames != 0 {
        return;
    }

    Now_SDL_Ticks = SDL_GetTicks();
    oneframedelay = c_long::from(Now_SDL_Ticks) - c_long::from(One_Frame_SDL_Ticks);
    oneframedelay = if oneframedelay > 0 { oneframedelay } else { 1 }; // avoid division by zero
    FPSover1 = (1000. / oneframedelay as f64) as f32;
}

#[no_mangle]
pub unsafe extern "C" fn Activate_Conservative_Frame_Computation() {
    SkipAFewFrames = true.into();

    // Now we are in some form of pause.  It can't
    // hurt to have the top status bar redrawn after that,
    // so we set this variable...
    BannerIsDestroyed = true.into();
}

/// This function usese calloc, so memory is automatically 0-initialized!
/// The function also checks for success and terminates in case of
/// "out of memory", so we dont need to do this always in the code.
#[no_mangle]
pub unsafe extern "C" fn MyMalloc(mut size: c_long) -> *mut c_void {
    // make Gnu-compatible even if on a broken system:
    if size == 0 {
        size = 1;
    }

    let ptr = libc::calloc(1, size.try_into().unwrap());
    if ptr.is_null() {
        error!("MyMalloc({}) did not succeed!", size);
        Terminate(defs::ERR.into());
    }

    ptr
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
#[no_mangle]
pub unsafe extern "C" fn find_file(
    fname: *const c_char,
    mut subdir: *mut c_char,
    use_theme: c_int,
    mut critical: c_int,
) -> *mut c_char {
    use std::io::Write;

    static mut FILE_PATH: [u8; 1024] = [0u8; 1024]; /* hope this will be enough */

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

    if fname.is_null() {
        error!("find_file() called with empty filename!");
        return null_mut();
    }
    if subdir.is_null() {
        subdir = cstr!("").as_ptr() as *mut c_char;
    }

    let inner = |datadir| {
        let theme_dir = if use_theme == Themed::UseTheme as c_int {
            Cow::Owned(format!(
                "{}_theme/",
                CStr::from_ptr(GameConfig.Theme_Name.as_ptr()).to_string_lossy(),
            ))
        } else {
            Cow::Borrowed("")
        };

        write!(
            &mut FILE_PATH[..],
            "{}/{}/{}/{}\0",
            datadir,
            CStr::from_ptr(subdir).to_string_lossy(),
            theme_dir,
            CStr::from_ptr(fname).to_string_lossy(),
        )
        .unwrap();

        CStr::from_ptr(FILE_PATH.as_ptr() as *const c_char)
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
                error!("ERROR in find_file(): Code should never reach this line!! Harakiri",);
                Terminate(defs::ERR.into());
            }
        };
        // how critical is this file for the game:
        match critical {
            Criticality::WarnOnly => {
                let fname = CStr::from_ptr(fname).to_string_lossy();
                if use_theme == Themed::UseTheme as c_int {
                    warn!(
                        "file {} not found in theme-dir: graphics/{}_theme/",
                        fname,
                        CStr::from_ptr(GameConfig.Theme_Name.as_ptr()).to_string_lossy(),
                    );
                } else {
                    warn!("file {} not found ", fname);
                }
                return null_mut();
            }
            Criticality::Ignore => return null_mut(),
            Criticality::Critical => {
                let fname = CStr::from_ptr(fname).to_string_lossy();
                if use_theme == Themed::UseTheme as c_int {
                    error!(
                        "file {} not found in theme-dir: graphics/{}_theme/, cannot run without it!",
                        fname,
                        CStr::from_ptr(GameConfig.Theme_Name.as_ptr()).to_string_lossy(),
                    );
                } else {
                    error!("file {} not found, cannot run without it!", fname);
                }
                Terminate(defs::ERR.into());
            }
        }
    }

    FILE_PATH.as_mut_ptr() as *mut c_char
}

/// show_progress: display empty progress meter with given text
#[no_mangle]
pub unsafe extern "C" fn init_progress(mut text: *mut c_char) {
    if text.is_null() {
        text = cstr!("Progress...").as_ptr() as *mut c_char;
    }

    if progress_meter_pic.is_null() {
        let mut fpath = find_file(
            PROGRESS_METER_FILE_C.as_ptr() as *mut c_char,
            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
            Themed::NoTheme as c_int,
            Criticality::Critical as c_int,
        );
        progress_meter_pic = Load_Block(fpath, 0, 0, null_mut(), 0);
        ScalePic(&mut progress_meter_pic, GameConfig.scale);
        fpath = find_file(
            PROGRESS_FILLER_FILE_C.as_ptr() as *mut c_char,
            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
            Themed::NoTheme as c_int,
            Criticality::Critical as c_int,
        );
        progress_filler_pic = Load_Block(fpath, 0, 0, null_mut(), 0);
        ScalePic(&mut progress_filler_pic, GameConfig.scale);

        scale_rect(&mut ProgressMeter_Rect, GameConfig.scale);
        scale_rect(&mut ProgressBar_Rect, GameConfig.scale);
        scale_rect(&mut ProgressText_Rect, GameConfig.scale);
    }

    SDL_SetClipRect(ne_screen, null_mut()); // this unsets the clipping rectangle
    SDL_UpperBlit(
        progress_meter_pic,
        null_mut(),
        ne_screen,
        &mut ProgressMeter_Rect,
    );

    let mut dst = ProgressText_Rect;
    dst.x += ProgressMeter_Rect.x;
    dst.y += ProgressMeter_Rect.y;

    printf_SDL(ne_screen, dst.x.into(), dst.y.into(), text);

    SDL_Flip(ne_screen);
}

/// This function read in a file with the specified name, allocated
/// memory for it of course, looks for the file end string and then
/// terminates the whole read in file with a 0 character, so that it
/// can easily be treated like a common string.
#[no_mangle]
pub unsafe extern "C" fn ReadAndMallocAndTerminateFile(
    filename: *mut c_char,
    file_end_string: *mut c_char,
) -> *mut c_char {
    use bstr::ByteSlice;
    use std::io::Read;

    let filename = CStr::from_ptr(filename).to_str().unwrap();
    let file_end_string = CStr::from_ptr(file_end_string);
    info!(
        "ReadAndMallocAndTerminateFile: The filename is: {}",
        filename
    );

    // Read the whole theme data to memory
    let mut file = match File::open(filename) {
        Ok(file) => {
            info!("ReadAndMallocAndTerminateFile: Opening file succeeded...");
            file
        }
        Err(_) => {
            error!(
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
                filename
            );
            Terminate(defs::ERR.into());
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
            error!("ReadAndMallocAndTerminateFile: Error fstat-ing File....");
            Terminate(defs::ERR.into());
        }
    };

    let data = MyMalloc((file_len + 64 * 2 + 10000).try_into().unwrap()) as *mut c_char;
    if data.is_null() {
        error!("ReadAndMallocAndTerminateFile: Out of Memory?");
        Terminate(defs::ERR.into());
    }

    let all_data = std::slice::from_raw_parts_mut(data as *mut u8, file_len + 64 * 2 + 10000);
    {
        let data = &mut all_data[..file_len];
        match file.read_exact(data) {
            Ok(()) => info!("ReadAndMallocAndTerminateFile: Reading file succeeded..."),
            Err(_) => {
                error!("ReadAndMallocAndTerminateFile: Reading file failed...");
                Terminate(defs::ERR.into());
            }
        }
    }
    all_data[file_len..].iter_mut().for_each(|c| *c = 0);

    drop(file);

    info!("ReadAndMallocAndTerminateFile: Adding a 0 at the end of read data....");

    match all_data.find(file_end_string.to_bytes()) {
        None => {
            error!(
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
                filename,
                file_end_string.to_string_lossy()
            );
            Terminate(defs::ERR.into());
        }
        Some(pos) => all_data[pos] = 0,
    }

    info!(
        "ReadAndMallocAndTerminateFile: The content of the read file: \n{}",
        CStr::from_ptr(data).to_string_lossy()
    );

    data
}

/// find label in data and read stuff after label into dst using the FormatString
///
/// NOTE!!: be sure dst is large enough for data read by FormatString, or
/// sscanf will crash!!
#[no_mangle]
pub unsafe extern "C" fn ReadValueFromString(
    data: *mut c_char,
    label: *mut c_char,
    format_string: *mut c_char,
    dst: *mut c_void,
) {
    // Now we locate the label in data and position pointer right after the label
    // ..will Terminate itself if not found...
    let pos = LocateStringInData(data, label).add(CStr::from_ptr(label).to_bytes().len());

    if libc::sscanf(pos, format_string, dst) == libc::EOF {
        error!(
            "ReadValueFromString(): could not read value {} of label {} with format {}",
            CStr::from_ptr(pos).to_string_lossy(),
            CStr::from_ptr(format_string).to_string_lossy(),
            CStr::from_ptr(label).to_string_lossy(),
        );
        Terminate(defs::ERR.into());
    } else {
        info!("ReadValueFromString: value read in successfully.");
    }
}
