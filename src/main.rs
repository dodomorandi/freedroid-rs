#![feature(c_variadic)]
#![feature(slice_strip)]
#![feature(const_maybe_uninit_assume_init)]
#![feature(extern_types)]

mod b_font;
mod bullet;
mod defs;
mod enemy;
mod global;
mod graphics;
mod highscore;
mod influence;
mod init;
mod input;
mod level_editor;
mod map;
mod menu;
mod misc;
mod ship;
mod sound;
mod structs;
mod takeover;
mod text;
mod vars;
mod view;

use defs::{AlertNames, Status};
use global::{
    AlertBonusPerSec, AlertLevel, AlertThreshold, AllEnemys, CurLevel, DeathCount,
    DeathCountDrainSpeed, LastGotIntoBlastSound, LastRefreshSound, LevelDoorsNotMovedTime, Me,
    RealScore, ShowScore, SkipAFewFrames, ThisMessageTime,
};
use misc::Frame_Time;
use vars::ShipEmptyCounter;

use sdl::video::ll::SDL_Surface;
use std::{
    env,
    ffi::CString,
    os::raw::{c_char, c_int},
    process,
};

extern "C" {
    fn c_main(argc: c_int, argv: *const *const c_char) -> c_int;
}

fn main() {
    env_logger::init();

    let args: Vec<_> = env::args().map(|arg| CString::new(arg).unwrap()).collect();
    let c_args: Vec<_> = args.iter().map(|arg| arg.as_ptr()).collect();

    let retval = unsafe { c_main(c_args.len() as c_int, c_args.as_ptr()) };
    if retval != 0 {
        process::exit(retval);
    }
}

#[inline]
fn sdl_must_lock(surface: &SDL_Surface) -> bool {
    use sdl::video::SurfaceFlag::*;
    surface.offset != 0
        && (surface.flags & (HWSurface as u32 | AsyncBlit as u32 | RLEAccel as u32)) != 0
}

/// This function updates counters and is called ONCE every frame.
/// The counters include timers, but framerate-independence of game speed
/// is preserved because everything is weighted with the Frame_Time()
/// function.
#[no_mangle]
unsafe extern "C" fn UpdateCountersForThisFrame() {
    // Here are some things, that were previously done by some periodic */
    // interrupt function
    ThisMessageTime += 1;

    LastGotIntoBlastSound += Frame_Time();
    LastRefreshSound += Frame_Time();
    Me.LastCrysoundTime += Frame_Time();
    Me.timer += Frame_Time();

    let cur_level = &mut *CurLevel;
    if cur_level.timer >= 0.0 {
        cur_level.timer -= Frame_Time();
    }

    Me.LastTransferSoundTime += Frame_Time();
    Me.TextVisibleTime += Frame_Time();
    LevelDoorsNotMovedTime += Frame_Time();
    if SkipAFewFrames != 0 {
        SkipAFewFrames = 0;
    }

    if Me.firewait > 0. {
        Me.firewait -= Frame_Time();
        if Me.firewait < 0. {
            Me.firewait = 0.;
        }
    }
    if ShipEmptyCounter > 1 {
        ShipEmptyCounter -= 1;
    }
    if cur_level.empty > 2 {
        cur_level.empty -= 1;
    }
    if RealScore > ShowScore as f32 {
        ShowScore += 1;
    }
    if RealScore < ShowScore as f32 {
        ShowScore -= 1;
    }

    // drain Death-count, responsible for Alert-state
    if DeathCount > 0. {
        DeathCount -= DeathCountDrainSpeed * Frame_Time();
    }
    if DeathCount < 0. {
        DeathCount = 0.;
    }
    // and switch Alert-level according to DeathCount
    AlertLevel = (DeathCount / AlertThreshold as f32) as i32;
    if AlertLevel > AlertNames::Red as i32 {
        AlertLevel = AlertNames::Red as i32;
    }
    // player gets a bonus/second in AlertLevel
    RealScore += AlertLevel as f32 * AlertBonusPerSec * Frame_Time();

    for enemy in &mut AllEnemys {
        if enemy.status == Status::Out as i32 {
            continue;
        }

        if enemy.warten > 0. {
            enemy.warten -= Frame_Time();
            if enemy.warten < 0. {
                enemy.warten = 0.;
            }
        }

        if enemy.firewait > 0. {
            enemy.firewait -= Frame_Time();
            if enemy.firewait <= 0. {
                enemy.firewait = 0.;
            }
        }

        enemy.TextVisibleTime += Frame_Time();
    }
}
