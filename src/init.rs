use crate::{
    b_font::{Para_BFont, SetCurrentFont},
    bullet::{ExplodeBlasts, MoveBullets},
    defs::{
        AssembleCombatWindowFlags, Criticality, DisplayBannerFlags, Status, Themed, GRAPHICS_DIR_C,
        TITLE_PIC_FILE_C, WAIT_AFTER_KILLED,
    },
    global::{num_highscores, Blastmap, Bulletmap, Druidmap, Highscores},
    graphics::{ne_screen, DisplayImage, MakeGridOnScreen, Number_Of_Bullet_Types},
    input::{wait_for_all_keys_released, wait_for_key_pressed},
    misc::find_file,
    sound::Switch_Background_Music_To,
    text::{DisplayText, ScrollText},
    vars::{Full_User_Rect, Screen_Rect},
    view::{Assemble_Combat_Picture, DisplayBanner},
    AllEnemys, GameOver, Me, NumEnemys, Number_Of_Droid_Types, RealScore, ShowScore,
};

use cstr::cstr;
use sdl::{
    event::ll::SDL_DISABLE,
    ll::SDL_GetTicks,
    mouse::ll::SDL_ShowCursor,
    video::ll::{SDL_Flip, SDL_FreeSurface, SDL_SetClipRect},
};
use std::{
    convert::{TryFrom, TryInto},
    ops::Not,
    os::raw::{c_char, c_int, c_long, c_void},
    ptr::null_mut,
};

extern "C" {
    pub fn InitFreedroid(argc: c_int, argv: *mut *const c_char);
    pub fn InitNewMission(mission_name: *mut c_char);

    static mut DebriefingText: *mut c_char;
    static mut DebriefingSong: [c_char; 500];
}

const MISSION_COMPLETE_BONUS: f32 = 1000.;

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
