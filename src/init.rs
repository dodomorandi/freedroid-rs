use crate::{
    b_font::{Para_BFont, SetCurrentFont},
    bullet::{ExplodeBlasts, MoveBullets},
    defs::{
        self, scale_rect, AssembleCombatWindowFlags, Criticality, DisplayBannerFlags, Status,
        Themed, GRAPHICS_DIR_C, TITLE_PIC_FILE_C, WAIT_AFTER_KILLED,
    },
    global::{
        num_highscores, Blastmap, Bulletmap, CurrentCombatScaleFactor, Druidmap, GameConfig,
        Highscores, SkipAFewFrames,
    },
    graphics::{
        ne_screen, DisplayImage, InitPictures, Init_Video, Load_Fonts, MakeGridOnScreen,
        Number_Of_Bullet_Types,
    },
    highscore::InitHighscores,
    input::{wait_for_all_keys_released, wait_for_key_pressed, Init_Joy},
    misc::{find_file, init_progress, update_progress, Terminate},
    sound::{Init_Audio, Switch_Background_Music_To},
    text::{DisplayText, ScrollText},
    vars::{Classic_User_Rect, Full_User_Rect, Screen_Rect, User_Rect},
    view::{Assemble_Combat_Picture, DisplayBanner},
    AllBullets, AllEnemys, GameOver, Me, NumEnemys, Number_Of_Droid_Types, RealScore, ShowScore,
};

use cstr::cstr;
use log::error;
use sdl::{
    event::ll::SDL_DISABLE,
    ll::SDL_GetTicks,
    mouse::ll::SDL_ShowCursor,
    video::ll::{SDL_Flip, SDL_FreeSurface, SDL_SetClipRect},
};
use std::{
    convert::{TryFrom, TryInto},
    ops::Not,
    os::raw::{c_char, c_int, c_long, c_uint, c_void},
    ptr::null_mut,
};

extern "C" {
    pub fn InitNewMission(mission_name: *mut c_char);
    pub fn LoadGameConfig();
    pub fn parse_command_line(argc: c_int, argv: *mut *const c_char);
    pub fn FindAllThemes();
    pub fn Init_Game_Data(data_filename: *mut c_char);

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
    parse_command_line(argc, argv);

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
