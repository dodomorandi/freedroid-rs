#[cfg(feature = "gcw0")]
use crate::defs::{gcw0_ls_pressed_r, gcw0_rs_pressed_r};
use crate::{
    defs::{self, AssembleCombatWindowFlags, Cmds, FirePressedR, Status},
    enemy::AnimateEnemys,
    global::{
        ne_screen, progress_filler_pic, ConfigDir, FPSover1, GameConfig, Me, ProgressBar_Rect,
        ProgressMeter_Rect, SkipAFewFrames,
    },
    graphics::FreeGraphics,
    highscore::SaveHighscores,
    influence::AnimateInfluence,
    init::FreeGameMem,
    input::{cmd_is_active, cmd_is_activeR, cmd_strings, key_cmds, KeyIsPressedR, SDL_Delay},
    map::{AnimateRefresh, FreeShipMemory},
    menu::FreeMenuData,
    ship::FreeDroidPics,
    sound::FreeSounds,
    view::{Assemble_Combat_Picture, DisplayBanner},
};

use log::{info, warn};
use once_cell::sync::Lazy;
use sdl::{
    sdl::{
        ll::{SDL_GetTicks, SDL_Quit},
        Rect,
    },
    video::ll::{SDL_UpdateRects, SDL_UpperBlit},
};
use std::{
    ffi::CStr,
    fs::File,
    os::raw::{c_char, c_float, c_int, c_long},
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
    pub fn Activate_Conservative_Frame_Computation();
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
