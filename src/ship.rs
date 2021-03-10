use crate::{
    defs::{AlertNames, Sound},
    map::GetMapBrick,
    sound::Play_Sound,
    structs::Level,
    AlertLevel, CurLevel,
};

use log::warn;
use sdl::{
    ll::SDL_GetTicks,
    video::ll::{SDL_FreeSurface, SDL_Surface},
    Rect,
};
use std::{
    convert::TryFrom,
    os::raw::{c_float, c_int},
};

extern "C" {
    pub fn show_droid_info(droid_type: c_int, page: c_int, flags: c_int);
    pub fn show_droid_portrait(dst: Rect, droid_type: c_int, cycle_time: c_float, flags: c_int);
    pub fn ShowDeckMap(deck: Level);
    pub fn EnterLift();
    pub fn EnterKonsole();

    pub static droid_background: *mut SDL_Surface;
    pub static droid_pics: *mut SDL_Surface;
}

#[no_mangle]
pub unsafe extern "C" fn FreeDroidPics() {
    SDL_FreeSurface(droid_pics);
    SDL_FreeSurface(droid_background);
}

/// do all alert-related agitations: alert-sirens and alert-lights
#[no_mangle]
pub unsafe extern "C" fn AlertLevelWarning() {
    const SIREN_WAIT: f32 = 2.5;

    static mut LAST_SIREN: u32 = 0;

    use AlertNames::*;
    match AlertNames::try_from(AlertLevel).ok() {
        Some(Green) => {}
        Some(Yellow) | Some(Amber) | Some(Red) => {
            if SDL_GetTicks() - LAST_SIREN > (SIREN_WAIT * 1000.0 / (AlertLevel as f32)) as u32 {
                // higher alert-> faster sirens!
                Play_Sound(Sound::Alert as c_int);
                LAST_SIREN = SDL_GetTicks();
            }
        }
        Some(Last) | None => {
            warn!(
                "illegal AlertLevel = {} > {}.. something's gone wrong!!\n",
                AlertLevel,
                AlertNames::Red as c_int
            );
        }
    }

    // so much to the sirens, now make sure the alert-tiles are updated correctly:
    let posx = (*CurLevel).alerts[0].x;
    let posy = (*CurLevel).alerts[0].y;
    if posx == -1 {
        // no alerts here...
        return;
    }

    let cur_alert = AlertNames::try_from(AlertLevel).unwrap();

    // check if alert-tiles are up-to-date
    if GetMapBrick(&*CurLevel, posx.into(), posy.into()) == cur_alert as u8 {
        // ok
        return;
    }

    for alert in &mut (*CurLevel).alerts {
        let posx = alert.x;
        let posy = alert.y;
        if posx == -1 {
            break;
        }

        *(*CurLevel).map[usize::try_from(posy).unwrap()].add(usize::try_from(posx).unwrap()) =
            cur_alert as i8;
    }
}
