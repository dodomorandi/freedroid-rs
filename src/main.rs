#![feature(c_variadic)]
#![feature(const_maybe_uninit_assume_init)]
#![feature(extern_types)]

macro_rules! rect {
    ($x:expr, $y:expr, $w:expr, $h:expr) => {
        ::sdl::Rect {
            x: $x,
            y: $y,
            w: $w,
            h: $h,
        }
    };
}

mod b_font;
mod bullet;
mod defs;
mod enemy;
mod global;
mod graphics;
mod highscore;
mod influencer;
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

use bullet::{CheckBulletCollisions, ExplodeBlasts, MoveBullets};
use defs::{
    scale_rect, AlertNames, AssembleCombatWindowFlags, DisplayBannerFlags, FirePressedR, Status,
    BYCOLOR, DROID_ROTATION_TIME, MAXBULLETS, RESET, SHOW_WAIT, STANDARD_MISSION_C,
};
use enemy::MoveEnemys;
use global::{
    curShip, debug_level, sound_on, AlertBonusPerSec, AlertLevel, AlertThreshold, AllEnemys,
    CurLevel, DeathCount, DeathCountDrainSpeed, GameConfig, GameOver, LastGotIntoBlastSound,
    LastRefreshSound, LevelDoorsNotMovedTime, QuitProgram, RealScore, ShowScore, SkipAFewFrames,
    ThisMessageTime,
};
use graphics::{crosshair_cursor, ne_screen, ClearGraphMem};
use influencer::{CheckInfluenceEnemyCollision, CheckInfluenceWallCollisions, MoveInfluence};
use init::{CheckIfMissionIsComplete, InitFreedroid, InitNewMission};
use input::{
    init_keystr, joy_sensitivity, show_cursor, wait_for_all_keys_released, ReactToSpecialKeys,
    SDL_Delay,
};
use map::{AnimateRefresh, ColorNames, MoveLevelDoors};
use misc::{
    set_time_factor, ComputeFPSForThisFrame, Frame_Time, StartTakingTimeForFPSCalculation,
    Terminate,
};
use ship::{show_droid_info, show_droid_portrait, AlertLevelWarning};
use sound::Switch_Background_Music_To;
use vars::{Cons_Droid_Rect, Me, ShipEmptyCounter};
use view::{Assemble_Combat_Picture, DisplayBanner};

use sdl::{
    mouse::ll::{SDL_SetCursor, SDL_ShowCursor, SDL_DISABLE, SDL_ENABLE},
    sdl::ll::SDL_GetTicks,
    video::ll::{SDL_Flip, SDL_Surface},
};
use std::{
    convert::TryFrom,
    env,
    ffi::CString,
    os::raw::{c_char, c_int},
    ptr::null_mut,
};

fn main() {
    env_logger::init();

    let args: Vec<_> = env::args().map(|arg| CString::new(arg).unwrap()).collect();
    let mut c_args: Vec<_> = args.iter().map(|arg| arg.as_ptr()).collect();

    unsafe {
        GameOver = false.into();
        QuitProgram = false.into();

        debug_level = 0; /* 0=no debug 1=first debug level (at the moment=all) */

        joy_sensitivity = 1;
        sound_on = true.into(); /* default value, can be overridden by command-line */

        init_keystr();

        InitFreedroid(c_args.len() as c_int, c_args.as_mut_ptr()); // Initialisation of global variables and arrays

        SDL_ShowCursor(SDL_DISABLE);

        #[cfg(target_os = "windows")]
        {
            // spread the word :)
            Win32Disclaimer();
        }

        while QuitProgram == 0 {
            InitNewMission(STANDARD_MISSION_C.as_ptr() as *mut c_char);

            // scale Level-pic rects
            let scale = GameConfig.scale;
            #[allow(clippy::clippy::float_cmp)]
            if scale != 1.0 {
                curShip.Level_Rects[0..usize::try_from(curShip.num_levels).unwrap()]
                    .iter_mut()
                    .zip(curShip.num_level_rects.iter())
                    .flat_map(|(rects, &num_rects)| {
                        rects[0..usize::try_from(num_rects).unwrap()].iter_mut()
                    })
                    .for_each(|rect| scale_rect(rect, scale));

                for rect in
                    &mut curShip.LiftRow_Rect[0..usize::try_from(curShip.num_lift_rows).unwrap()]
                {
                    scale_rect(rect, scale);
                }
            }

            // release all keys
            wait_for_all_keys_released();

            show_droid_info(Me.ty, -3, 0); // show unit-intro page
            show_droid_portrait(Cons_Droid_Rect, Me.ty, DROID_ROTATION_TIME, RESET);
            let now = SDL_GetTicks();
            while SDL_GetTicks() - now < SHOW_WAIT && !FirePressedR() {
                show_droid_portrait(Cons_Droid_Rect, Me.ty, DROID_ROTATION_TIME, 0);
                SDL_Delay(1);
            }

            ClearGraphMem();
            DisplayBanner(
                null_mut(),
                null_mut(),
                (DisplayBannerFlags::FORCE_UPDATE | DisplayBannerFlags::NO_SDL_UPDATE)
                    .bits()
                    .into(),
            );
            SDL_Flip(ne_screen);

            GameOver = false.into();

            SDL_SetCursor(crosshair_cursor); // default cursor is a crosshair
            SDL_ShowCursor(SDL_ENABLE);

            while GameOver == 0 && QuitProgram == 0 {
                StartTakingTimeForFPSCalculation();

                UpdateCountersForThisFrame();

                ReactToSpecialKeys();

                if show_cursor {
                    SDL_ShowCursor(SDL_ENABLE);
                } else {
                    SDL_ShowCursor(SDL_DISABLE);
                }

                MoveLevelDoors();

                AnimateRefresh();

                ExplodeBlasts(); // move blasts to the right current "phase" of the blast

                AlertLevelWarning(); // tout tout, blink blink... Alert!!

                DisplayBanner(null_mut(), null_mut(), 0);

                MoveBullets(); // leave this in front of graphics output: time_in_frames should start with 1

                Assemble_Combat_Picture(AssembleCombatWindowFlags::DO_SCREEN_UPDATE.bits().into());

                for bullet in 0..i32::try_from(MAXBULLETS).unwrap() {
                    CheckBulletCollisions(bullet);
                }

                MoveInfluence(); // change Influ-speed depending on keys pressed, but
                                 // also change his status and position and "phase" of rotation

                MoveEnemys(); // move all the enemys:
                              // also do attacks on influ and also move "phase" or their rotation

                CheckInfluenceWallCollisions(); /* Testen ob der Weg nicht durch Mauern verstellt ist */
                CheckInfluenceEnemyCollision();

                // control speed of time-flow: dark-levels=emptyLevelSpeedup, normal-levels=1.0
                if (*CurLevel).empty == 0 {
                    set_time_factor(1.0);
                } else if (*CurLevel).color == ColorNames::Dark as i32 {
                    // if level is already dark
                    set_time_factor(GameConfig.emptyLevelSpeedup);
                } else if (*CurLevel).timer <= 0. {
                    // time to switch off the lights ...
                    (*CurLevel).color = ColorNames::Dark as i32;
                    Switch_Background_Music_To(BYCOLOR.as_ptr()); // start new background music
                }

                CheckIfMissionIsComplete();

                if GameConfig.HogCPU == 0 {
                    // don't use up 100% CPU unless requested
                    SDL_Delay(1);
                }

                ComputeFPSForThisFrame();
            }
        }

        Terminate(0);
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
