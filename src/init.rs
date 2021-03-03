use crate::{
    global::{num_highscores, Blastmap, Bulletmap, Druidmap, Highscores},
    graphics::Number_Of_Bullet_Types,
    Number_Of_Droid_Types,
};

use sdl::video::ll::SDL_FreeSurface;
use std::{
    convert::TryFrom,
    ops::Not,
    os::raw::{c_char, c_int, c_void},
    ptr::null_mut,
};

extern "C" {
    pub fn InitFreedroid(argc: c_int, argv: *mut *const c_char);
    pub fn InitNewMission(mission_name: *mut c_char);
    pub fn CheckIfMissionIsComplete();

    static mut DebriefingText: *mut c_char;
}

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
