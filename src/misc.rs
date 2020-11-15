use crate::{
    defs,
    global::{
        ne_screen, progress_filler_pic, FPSover1, ProgressBar_Rect, ProgressMeter_Rect,
        SkipAFewFrames,
    },
    graphics::FreeGraphics,
    highscore::SaveHighscores,
    init::FreeGameMem,
    map::FreeShipMemory,
    menu::FreeMenuData,
    ship::FreeDroidPics,
    sound::FreeSounds,
};

use log::info;
use once_cell::sync::Lazy;
use sdl::{
    sdl::{ll::SDL_Quit, Rect},
    video::ll::{SDL_UpdateRects, SDL_UpperBlit},
};
use std::{
    os::raw::{c_float, c_int},
    process,
    sync::RwLock,
};

extern "C" {
    pub fn Pause();
    pub fn SaveGameConfig() -> c_int;

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
