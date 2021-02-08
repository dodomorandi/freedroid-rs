#[cfg(not(feature = "gcw0"))]
use crate::input::{key_cmds, wait_for_all_keys_released, wait_for_key_pressed};

#[cfg(feature = "gcw0")]
use crate::{
    defs::{gcw0_a_pressed, gcw0_any_button_pressed, gcw0_any_button_pressed_r},
    input::SDL_Delay,
};

use crate::{
    b_font::{
        CenteredPutString, CharWidth, FontHeight, GetCurrentFont, PrintStringFont, PutString,
        SetCurrentFont, TextWidth,
    },
    defs::{
        self, AssembleCombatWindowFlags, Cmds, DisplayBannerFlags, DownPressed, Droid, FirePressed,
        LeftPressed, MenuAction, ReturnPressedR, RightPressed, Status, UpPressed,
        CREDITS_PIC_FILE_C, GRAPHICS_DIR_C,
    },
    global::{
        curShip, quit_LevelEditor, quit_Menu, show_all_droids, sound_on, stop_influencer,
        AllEnemys, Block_Rect, CurLevel, CurrentCombatScaleFactor, Druidmap, Font0_BFont,
        Font1_BFont, Font2_BFont, Full_User_Rect, InvincibleMode, Me, Menu_BFont, NumEnemys,
        Number_Of_Droid_Types, Screen_Rect, User_Rect,
    },
    graphics::{
        ne_screen, BannerIsDestroyed, ClearGraphMem, DisplayImage, MakeGridOnScreen,
        SetCombatScaleTo,
    },
    highscore::ShowHighscores,
    input::{
        any_key_is_pressedR, cmd_is_activeR, cmd_strings, keystr, update_input, KeyIsPressed,
        KeyIsPressedR, SDL_Delay, WheelDownPressed, WheelUpPressed,
    },
    map::SaveShip,
    misc::{find_file, Activate_Conservative_Frame_Computation, Armageddon, Teleport, Terminate},
    ship::ShowDeckMap,
    sound::{MenuItemSelectedSound, MoveMenuPositionSound},
    text::{getchar_raw, printf_SDL, GetString},
    vars::InfluenceModeNames,
    view::{Assemble_Combat_Picture, DisplayBanner, PutInfluence},
};

use cstr::cstr;
use defs::{get_user_center, Criticality, Themed};
use sdl::{
    keysym::{SDLK_BACKSPACE, SDLK_DOWN, SDLK_ESCAPE, SDLK_LEFT, SDLK_RIGHT, SDLK_UP},
    mouse::ll::{SDL_ShowCursor, SDL_DISABLE, SDL_ENABLE},
    sdl::ll::SDL_GetTicks,
    video::ll::{
        SDL_DisplayFormat, SDL_Flip, SDL_FreeSurface, SDL_SetClipRect, SDL_Surface, SDL_UpperBlit,
    },
};
use std::{
    convert::{TryFrom, TryInto},
    ffi::CStr,
    os::raw::{c_char, c_int, c_void},
    ptr::null_mut,
};

extern "C" {
    pub static mut fheight: c_int;
    pub static mut Menu_Background: *mut SDL_Surface;
    pub static LevelEditorMenu: *const MenuEntry;

    #[cfg(target = "android")]
    pub static MainMenu: [MenuEntry; 8];

    #[cfg(not(target = "android"))]
    pub static MainMenu: [MenuEntry; 10];
}

const FILENAME_LEN: u8 = 128;
const SHIP_EXT: &CStr = cstr!(".shp");
const ELEVEXT: &CStr = cstr!(".elv");
const CREWEXT: &CStr = cstr!(".crw");

#[repr(C)]
pub struct MenuEntry {
    name: *const c_char,
    handler: Option<unsafe extern "C" fn(MenuAction) -> *const c_char>,
    submenu: *const MenuEntry,
}

#[no_mangle]
pub unsafe extern "C" fn handle_QuitGame(action: MenuAction) -> *const c_char {
    if action != MenuAction::CLICK {
        return null_mut();
    }

    MenuItemSelectedSound();
    InitiateMenu(false);

    #[cfg(feature = "gcw0")]
    const QUIT_STRING: &CStr = cstr!("Press A to quit");

    #[cfg(not(feature = "gcw0"))]
    const QUIT_STRING: &CStr = cstr!("Hit 'y' or press Fire to quit");

    let text_width = TextWidth(QUIT_STRING.as_ptr());
    let text_x = i32::from(User_Rect.x) + (i32::from(User_Rect.w) - text_width) / 2;
    let text_y = i32::from(User_Rect.y) + (i32::from(User_Rect.h) - fheight) / 2;
    PutString(ne_screen, text_x, text_y, QUIT_STRING.as_ptr());
    SDL_Flip(ne_screen);

    #[cfg(feature = "gcw0")]
    {
        while !gcw0_any_button_pressed() {
            SDL_Delay(1);
        }

        if gcw0_a_pressed() {
            while !gcw0_any_button_pressed_r() {
                // In case FirePressed && !Gcw0APressed() -> would cause a loop otherwise in the menu...
                SDL_Delay(1);
            }
            Terminate(defs::OK.into());
        }
    }

    #[cfg(not(feature = "gcw0"))]
    {
        wait_for_all_keys_released();
        let key = wait_for_key_pressed();
        if key == b'y'.into()
            || key == key_cmds[Cmds::Fire as usize][0]
            || key == key_cmds[Cmds::Fire as usize][1]
            || key == key_cmds[Cmds::Fire as usize][2]
        {
            Terminate(defs::OK.into());
        }
    }

    null_mut()
}

/// simple wrapper to ShowMenu() to provide the external entry point into the main menu
#[no_mangle]
pub unsafe extern "C" fn showMainMenu() {
    ShowMenu(MainMenu.as_ptr());
}

#[no_mangle]
pub unsafe extern "C" fn FreeMenuData() {
    SDL_FreeSurface(Menu_Background);
}

#[no_mangle]
pub unsafe extern "C" fn InitiateMenu(with_droids: bool) {
    // Here comes the standard initializer for all the menus and submenus
    // of the big escape menu.  This prepares the screen, so that we can
    // write on it further down.
    Activate_Conservative_Frame_Computation();

    SDL_SetClipRect(ne_screen, null_mut());
    Me.status = Status::Menu as i32;
    ClearGraphMem();
    DisplayBanner(
        null_mut(),
        null_mut(),
        (DisplayBannerFlags::NO_SDL_UPDATE | DisplayBannerFlags::FORCE_UPDATE)
            .bits()
            .into(),
    );
    if with_droids {
        Assemble_Combat_Picture(0);
    } else {
        Assemble_Combat_Picture(AssembleCombatWindowFlags::ONLY_SHOW_MAP.bits().into());
    }

    SDL_SetClipRect(ne_screen, null_mut());
    MakeGridOnScreen(None);

    if !Menu_Background.is_null() {
        SDL_FreeSurface(Menu_Background);
    }
    Menu_Background = SDL_DisplayFormat(ne_screen); // keep a global copy of background

    SDL_ShowCursor(SDL_DISABLE); // deactivate mouse-cursor in menus
    SetCurrentFont(Menu_BFont);
    fheight = FontHeight(&*GetCurrentFont()) + 2;
}

#[no_mangle]
pub unsafe extern "C" fn Cheatmenu() {
    // Prevent distortion of framerate by the delay coming from
    // the time spend in the menu.
    Activate_Conservative_Frame_Computation();

    let font = Font0_BFont;

    SetCurrentFont(font); /* not the ideal one, but there's currently */
    /* no other it seems.. */
    const X0: i32 = 50;
    const Y0: i32 = 20;

    let cur_level = &mut *CurLevel;
    let droid_map = std::slice::from_raw_parts(Druidmap, Droid::NumDroids as usize);
    let mut resume = false;
    while !resume {
        ClearGraphMem();
        printf_SDL(
            ne_screen,
            X0,
            Y0,
            cstr!("Current position: Level=%d, X=%d, Y=%d\n").as_ptr() as *mut c_char,
            cur_level.levelnum,
            Me.pos.x as c_int,
            Me.pos.y as c_int,
        );
        printf_SDL(
            ne_screen,
            -1,
            -1,
            cstr!(" a. Armageddon (alle Robots sprengen)\n").as_ptr() as *mut c_char,
        );
        printf_SDL(
            ne_screen,
            -1,
            -1,
            cstr!(" l. robot list of current level\n").as_ptr() as *mut c_char,
        );
        printf_SDL(
            ne_screen,
            -1,
            -1,
            cstr!(" g. complete robot list\n").as_ptr() as *mut c_char,
        );
        printf_SDL(
            ne_screen,
            -1,
            -1,
            cstr!(" d. destroy robots on current level\n").as_ptr() as *mut c_char,
        );
        printf_SDL(
            ne_screen,
            -1,
            -1,
            cstr!(" t. Teleportation\n").as_ptr() as *mut c_char,
        );
        printf_SDL(
            ne_screen,
            -1,
            -1,
            cstr!(" r. change to new robot type\n").as_ptr() as *mut c_char,
        );
        printf_SDL(
            ne_screen,
            -1,
            -1,
            cstr!(" i. Invinciblemode: %s").as_ptr() as *mut c_char,
            if InvincibleMode != 0 {
                cstr!("ON\n").as_ptr() as *mut c_char
            } else {
                cstr!("OFF\n").as_ptr() as *mut c_char
            },
        );
        printf_SDL(
            ne_screen,
            -1,
            -1,
            cstr!(" e. set energy\n").as_ptr() as *mut c_char,
        );
        printf_SDL(
            ne_screen,
            -1,
            -1,
            cstr!(" n. No hidden droids: %s").as_ptr() as *mut c_char,
            if show_all_droids != 0 {
                cstr!("ON\n").as_ptr() as *mut c_char
            } else {
                cstr!("OFF\n").as_ptr() as *mut c_char
            },
        );
        printf_SDL(
            ne_screen,
            -1,
            -1,
            cstr!(" m. Map of Deck xy\n").as_ptr() as *mut c_char,
        );
        printf_SDL(
            ne_screen,
            -1,
            -1,
            cstr!(" s. Sound: %s").as_ptr() as *mut c_char,
            if sound_on != 0 {
                cstr!("ON\n").as_ptr() as *mut c_char
            } else {
                cstr!("OFF\n").as_ptr() as *mut c_char
            },
        );
        printf_SDL(
            ne_screen,
            -1,
            -1,
            cstr!(" w. Print current waypoints\n").as_ptr() as *mut c_char,
        );
        printf_SDL(
            ne_screen,
            -1,
            -1,
            cstr!(" z. change Zoom factor\n").as_ptr() as *mut c_char,
        );
        printf_SDL(
            ne_screen,
            -1,
            -1,
            cstr!(" f. Freeze on this positon: %s").as_ptr() as *mut c_char,
            if stop_influencer != 0 {
                cstr!("ON\n").as_ptr() as *mut c_char
            } else {
                cstr!("OFF\n").as_ptr() as *mut c_char
            },
        );
        printf_SDL(
            ne_screen,
            -1,
            -1,
            cstr!(" q. RESUME game\n").as_ptr() as *mut c_char,
        );

        match u8::try_from(getchar_raw()).ok() {
            Some(b'f') => {
                stop_influencer = !stop_influencer;
            }

            Some(b'z') => {
                ClearGraphMem();
                printf_SDL(
                    ne_screen,
                    X0,
                    Y0,
                    cstr!("Current Zoom factor: %f\n").as_ptr() as *mut c_char,
                    CurrentCombatScaleFactor as f64,
                );
                printf_SDL(
                    ne_screen,
                    -1,
                    -1,
                    cstr!("New zoom factor: ").as_ptr() as *mut c_char,
                );
                let input = GetString(40, 2);
                libc::sscanf(
                    input,
                    cstr!("%f").as_ptr() as *mut c_char,
                    &mut CurrentCombatScaleFactor,
                );
                libc::free(input as *mut c_void);
                SetCombatScaleTo(CurrentCombatScaleFactor);
            }

            Some(b'a') => {
                /* armageddon */
                resume = true;
                Armageddon();
            }

            Some(b'l') => {
                /* robot list of this deck */
                let mut l = 0; /* line counter for enemy output */
                for i in 0..usize::try_from(NumEnemys).unwrap() {
                    if AllEnemys[i].levelnum == cur_level.levelnum {
                        if l != 0 && l % 20 == 0 {
                            printf_SDL(
                                ne_screen,
                                -1,
                                -1,
                                cstr!(" --- MORE --- \n").as_ptr() as *mut c_char,
                            );
                            if getchar_raw() == b'q'.into() {
                                break;
                            }
                        }
                        if l % 20 == 0 {
                            ClearGraphMem();
                            printf_SDL(
                                ne_screen,
                                X0,
                                Y0,
                                cstr!(" NR.   ID  X    Y   ENERGY   Status\n").as_ptr()
                                    as *mut c_char,
                            );
                            printf_SDL(
                                ne_screen,
                                -1,
                                -1,
                                cstr!("---------------------------------------------\n").as_ptr()
                                    as *mut c_char,
                            );
                        }

                        l += 1;
                        let status = if AllEnemys[i].status == Status::Out as i32 {
                            cstr!("OUT").as_ptr() as *mut c_char
                        } else if AllEnemys[i].status == Status::Terminated as i32 {
                            cstr!("DEAD").as_ptr() as *mut c_char
                        } else {
                            cstr!("ACTIVE").as_ptr() as *mut c_char
                        };

                        printf_SDL(
                            ne_screen,
                            -1,
                            -1,
                            cstr!("%d.   %s   %d   %d   %d    %s.\n").as_ptr() as *mut c_char,
                            i as i32,
                            droid_map[usize::try_from(AllEnemys[i].ty).unwrap()]
                                .druidname
                                .as_ptr(),
                            AllEnemys[i].pos.x as c_int,
                            AllEnemys[i].pos.y as c_int,
                            AllEnemys[i].energy as c_int,
                            status,
                        );
                    }
                }

                printf_SDL(
                    ne_screen,
                    -1,
                    -1,
                    cstr!(" --- END --- \n").as_ptr() as *mut c_char,
                );
                getchar_raw();
            }

            Some(b'g') => {
                /* complete robot list of this ship */
                for i in 0..usize::try_from(NumEnemys).unwrap() {
                    if AllEnemys[i].ty == -1 {
                        continue;
                    }

                    if i != 0 && !i % 13 == 0 {
                        printf_SDL(
                            ne_screen,
                            -1,
                            -1,
                            cstr!(" --- MORE --- ('q' to quit)\n").as_ptr() as *mut c_char,
                        );
                        if getchar_raw() == b'q'.into() {
                            break;
                        }
                    }
                    if i % 13 == 0 {
                        ClearGraphMem();
                        printf_SDL(
                            ne_screen,
                            X0,
                            Y0,
                            cstr!("Nr.  Lev. ID  Energy  Status.\n").as_ptr() as *mut c_char,
                        );
                        printf_SDL(
                            ne_screen,
                            -1,
                            -1,
                            cstr!("------------------------------\n").as_ptr() as *mut c_char,
                        );
                    }

                    printf_SDL(
                        ne_screen,
                        -1,
                        -1,
                        cstr!("%d  %d  %s  %d  %s\n").as_ptr() as *mut c_char,
                        i as i32,
                        AllEnemys[i].levelnum,
                        droid_map[usize::try_from(AllEnemys[i].ty).unwrap()]
                            .druidname
                            .as_ptr(),
                        AllEnemys[i].energy as c_int,
                        InfluenceModeNames[usize::try_from(AllEnemys[i].status).unwrap()],
                    );
                }

                printf_SDL(
                    ne_screen,
                    -1,
                    -1,
                    cstr!(" --- END ---\n").as_ptr() as *mut c_char,
                );
                getchar_raw();
            }

            Some(b'd') => {
                /* destroy all robots on this level, haha */
                for enemy in &mut AllEnemys {
                    if enemy.levelnum == cur_level.levelnum {
                        enemy.energy = -100.;
                    }
                }
                printf_SDL(
                    ne_screen,
                    -1,
                    -1,
                    cstr!("All robots on this deck killed!\n").as_ptr() as *mut c_char,
                );
                getchar_raw();
            }

            Some(b't') => {
                /* Teleportation */
                ClearGraphMem();
                printf_SDL(
                    ne_screen,
                    X0,
                    Y0,
                    cstr!("Enter Level, X, Y: ").as_ptr() as *mut c_char,
                );
                let input = GetString(40, 2);
                let mut l_num = 0;
                let mut x = 0;
                let mut y = 0;

                libc::sscanf(
                    input,
                    cstr!("%d, %d, %d\n").as_ptr() as *mut c_char,
                    &mut l_num,
                    &mut x,
                    &mut y,
                );
                libc::free(input as *mut c_void);
                Teleport(l_num, x, y);
            }

            Some(b'r') => {
                /* change to new robot type */
                ClearGraphMem();
                printf_SDL(
                    ne_screen,
                    X0,
                    Y0,
                    cstr!("Type number of new robot: ").as_ptr() as *mut c_char,
                );
                let input = GetString(40, 2);
                let mut i = 0;
                for _ in 0..u32::try_from(Number_Of_Droid_Types).unwrap() {
                    if libc::strcmp(droid_map[i].druidname.as_ptr(), input) != 0 {
                        break;
                    }
                    i += 1;
                }

                if i == usize::try_from(Number_Of_Droid_Types).unwrap() {
                    printf_SDL(
                        ne_screen,
                        X0,
                        Y0 + 20,
                        cstr!("Unrecognized robot-type: %s").as_ptr() as *mut c_char,
                        input,
                    );
                    getchar_raw();
                    ClearGraphMem();
                } else {
                    Me.ty = i.try_into().unwrap();
                    Me.energy = droid_map[usize::try_from(Me.ty).unwrap()].maxenergy;
                    Me.health = Me.energy;
                    printf_SDL(
                        ne_screen,
                        X0,
                        Y0 + 20,
                        cstr!("You are now a %s. Have fun!\n").as_ptr() as *mut c_char,
                        input,
                    );
                    getchar_raw();
                }
                libc::free(input as *mut c_void);
            }

            Some(b'i') => {
                /* togge Invincible mode */
                InvincibleMode = !InvincibleMode;
            }

            Some(b'e') => {
                /* complete heal */
                ClearGraphMem();
                printf_SDL(
                    ne_screen,
                    X0,
                    Y0,
                    cstr!("Current energy: %f\n").as_ptr() as *mut c_char,
                    Me.energy as f64,
                );
                printf_SDL(
                    ne_screen,
                    -1,
                    -1,
                    cstr!("Enter your new energy: ").as_ptr() as *mut c_char,
                );
                let input = GetString(40, 2);
                let mut num = 0;
                libc::sscanf(input, cstr!("%d").as_ptr() as *mut c_char, &mut num);
                libc::free(input as *mut c_void);
                Me.energy = num as f32;
                if Me.energy > Me.health {
                    Me.health = Me.energy;
                }
            }

            Some(b'n') => {
                /* toggle display of all droids */
                show_all_droids = !show_all_droids;
            }

            Some(b's') => {
                /* toggle sound on/off */
                sound_on = !sound_on;
            }

            Some(b'm') => {
                /* Show deck map in Concept view */
                printf_SDL(
                    ne_screen,
                    -1,
                    -1,
                    cstr!("\nLevelnum: ").as_ptr() as *mut c_char,
                );
                let input = GetString(40, 2);
                let mut l_num = 0;
                libc::sscanf(input, cstr!("%d").as_ptr() as *mut c_char, &mut l_num);
                libc::free(input as *mut c_void);
                ShowDeckMap(*curShip.AllLevels[usize::try_from(l_num).unwrap()]);
                getchar_raw();
            }

            Some(b'w') => {
                /* print waypoint info of current level */
                for (i, waypoint) in cur_level.AllWaypoints.iter_mut().enumerate() {
                    if i != 0 && i % 20 == 0 {
                        printf_SDL(
                            ne_screen,
                            -1,
                            -1,
                            cstr!(" ---- MORE -----\n").as_ptr() as *mut c_char,
                        );
                        if getchar_raw() == b'q'.into() {
                            break;
                        }
                    }
                    if i % 20 == 0 {
                        ClearGraphMem();
                        printf_SDL(
                            ne_screen,
                            X0,
                            Y0,
                            cstr!("Nr.   X   Y      C1  C2  C3  C4\n").as_ptr() as *mut c_char,
                        );
                        printf_SDL(
                            ne_screen,
                            -1,
                            -1,
                            cstr!("------------------------------------\n").as_ptr() as *mut c_char,
                        );
                    }
                    printf_SDL(
                        ne_screen,
                        -1,
                        -1,
                        cstr!("%2d   %2d  %2d      %2d  %2d  %2d  %2d\n").as_ptr() as *mut c_char,
                        i as i32,
                        i32::from(waypoint.x),
                        i32::from(waypoint.y),
                        waypoint.connections[0],
                        waypoint.connections[1],
                        waypoint.connections[2],
                        waypoint.connections[3],
                    );
                }
                printf_SDL(
                    ne_screen,
                    -1,
                    -1,
                    cstr!(" --- END ---\n").as_ptr() as *mut c_char,
                );
                getchar_raw();
            }

            Some(b' ') | Some(b'q') => {
                resume = true;
            }

            _ => {}
        }
    }

    ClearGraphMem();

    update_input(); /* treat all pending keyboard events */
}

/// get menu input actions
///
/// NOTE: built-in time delay to ensure spurious key-repetitions
/// such as from touchpad 'wheel' or android joystic emulation
/// don't create unexpected menu movements:
/// ==> ignore all movement commands withing delay_ms milliseconds of each other
#[no_mangle]
pub unsafe extern "C" fn getMenuAction(wait_repeat_ticks: u32) -> MenuAction {
    let mut action = MenuAction::empty();

    // 'normal' menu action keys get released
    if KeyIsPressedR(SDLK_BACKSPACE as c_int) {
        {
            action = MenuAction::DELETE;
        }
    }
    if cmd_is_activeR(Cmds::Back) || KeyIsPressedR(SDLK_ESCAPE as c_int) {
        {
            action = MenuAction::BACK;
        }
    }

    if FirePressed() || ReturnPressedR() {
        {
            action = MenuAction::CLICK;
        }
    }

    // ----- up/down motion: allow for key-repeat, but carefully control repeat rate (modelled on takeover game)
    static mut LAST_MOVEKEY_TIME: u32 = 0;

    static mut UP: bool = false;
    static mut DOWN: bool = false;
    static mut LEFT: bool = false;
    static mut RIGHT: bool = false;

    // we register if there have been key-press events in the "waiting period" between move-ticks
    if !UP && (UpPressed() || KeyIsPressed(SDLK_UP as c_int)) {
        UP = true;
        LAST_MOVEKEY_TIME = SDL_GetTicks();
        action |= MenuAction::UP;
    }
    if !DOWN && (DownPressed() || KeyIsPressed(SDLK_DOWN as c_int)) {
        DOWN = true;
        LAST_MOVEKEY_TIME = SDL_GetTicks();
        action |= MenuAction::DOWN;
    }
    if !LEFT && (LeftPressed() || KeyIsPressed(SDLK_LEFT as c_int)) {
        LEFT = true;
        LAST_MOVEKEY_TIME = SDL_GetTicks();
        action |= MenuAction::LEFT;
    }
    if !RIGHT && (RightPressed() || KeyIsPressed(SDLK_RIGHT as c_int)) {
        RIGHT = true;
        LAST_MOVEKEY_TIME = SDL_GetTicks();
        action |= MenuAction::RIGHT;
    }

    if !(UpPressed() || KeyIsPressed(SDLK_UP as c_int)) {
        UP = false;
    }
    if !(DownPressed() || KeyIsPressed(SDLK_DOWN as c_int)) {
        DOWN = false;
    }
    if !(LeftPressed() || KeyIsPressed(SDLK_LEFT as c_int)) {
        LEFT = false;
    }
    if !(RightPressed() || KeyIsPressed(SDLK_RIGHT as c_int)) {
        RIGHT = false;
    }

    // check if enough time since we registered last new move-action
    if SDL_GetTicks() - LAST_MOVEKEY_TIME > wait_repeat_ticks {
        if UP {
            action |= MenuAction::UP;
        }
        if DOWN {
            action |= MenuAction::DOWN;
        }
        if LEFT {
            action |= MenuAction::LEFT;
        }
        if RIGHT {
            action |= MenuAction::RIGHT;
        }
    }
    // special handling of mouse wheel: register every event, no need for key-repeat delays
    if WheelUpPressed() {
        action |= MenuAction::UP_WHEEL;
    }
    if WheelDownPressed() {
        action |= MenuAction::DOWN_WHEEL;
    }

    action
}

/// Generic menu handler
#[no_mangle]
pub unsafe extern "C" fn ShowMenu(menu_entries: *const MenuEntry) {
    use std::ops::Not;

    InitiateMenu(false);
    wait_for_all_keys_released();

    // figure out menu-start point to make it centered
    let mut num_entries = 0;
    let mut menu_width = None::<i32>;
    loop {
        let entry = &*menu_entries.add(num_entries);
        if entry.name.is_null() {
            break;
        }

        let width = TextWidth(entry.name);
        menu_width = Some(
            menu_width
                .map(|menu_width| menu_width.max(width))
                .unwrap_or(width),
        );

        num_entries += 1;
    }
    let menu_entries = std::slice::from_raw_parts(menu_entries, num_entries);
    let menu_width = menu_width.unwrap();

    let menu_height = i32::try_from(num_entries).unwrap() * fheight;
    let menu_x = i32::from(Full_User_Rect.x) + (i32::from(Full_User_Rect.w) - menu_width) / 2;
    let menu_y = i32::from(Full_User_Rect.y) + (i32::from(Full_User_Rect.h) - menu_height) / 2;
    let influ_x = menu_x - i32::from(Block_Rect.w) - fheight;

    let mut menu_pos = 0;

    let wait_move_ticks: u32 = 100;
    static mut LAST_MOVE_TICK: u32 = 0;
    let mut finished = false;
    quit_Menu = false;
    let mut need_update = true;
    while !finished {
        let handler = menu_entries[menu_pos].handler;
        let submenu = menu_entries[menu_pos].submenu;

        if need_update {
            SDL_UpperBlit(Menu_Background, null_mut(), ne_screen, null_mut());
            // print menu
            menu_entries.iter().enumerate().for_each(|(i, entry)| {
                let arg = entry
                    .handler
                    .map(|handler| (handler)(MenuAction::INFO))
                    .unwrap_or(null_mut());

                let arg = if arg.is_null() {
                    cstr!("").as_ptr()
                } else {
                    arg
                };

                let mut full_name: [c_char; 256] = [0; 256];
                libc::sprintf(
                    full_name.as_mut_ptr(),
                    cstr!("%s%s").as_ptr(),
                    entry.name,
                    arg,
                );
                PutString(
                    ne_screen,
                    menu_x,
                    menu_y + i32::try_from(i).unwrap() * fheight,
                    full_name.as_ptr(),
                );
            });
            PutInfluence(
                influ_x,
                menu_y + ((menu_pos as f64 - 0.5) * f64::from(fheight)) as c_int,
            );

            #[cfg(not(target_os = "android"))]
            SDL_Flip(ne_screen);

            need_update = false;
        }

        #[cfg(target_os = "android")]
        SDL_Flip(ne_screen); // for responsive input on Android, we need to run this every cycle

        let action = getMenuAction(250);

        let time_for_move = SDL_GetTicks() - LAST_MOVE_TICK > wait_move_ticks;
        match action {
            MenuAction::BACK => {
                finished = true;
                wait_for_all_keys_released();
            }

            MenuAction::CLICK => {
                if handler.is_none() && submenu.is_null() {
                    MenuItemSelectedSound();
                    finished = true;
                } else {
                    if let Some(handler) = handler {
                        wait_for_all_keys_released();
                        (handler)(action);
                    }

                    if submenu.is_null().not() {
                        MenuItemSelectedSound();
                        wait_for_all_keys_released();
                        ShowMenu(submenu);
                        InitiateMenu(false);
                    }
                    need_update = true;
                }
            }

            MenuAction::RIGHT | MenuAction::LEFT => {
                if !time_for_move {
                    continue;
                }

                if let Some(handler) = handler {
                    (handler)(action);
                }
                LAST_MOVE_TICK = SDL_GetTicks();
                need_update = true;
            }

            MenuAction::UP | MenuAction::UP_WHEEL => {
                if action == MenuAction::UP && !time_for_move {
                    continue;
                }

                MoveMenuPositionSound();
                if menu_pos > 0 {
                    menu_pos -= 1;
                } else {
                    menu_pos = num_entries - 1;
                }
                LAST_MOVE_TICK = SDL_GetTicks();
                need_update = true;
            }

            MenuAction::DOWN | MenuAction::DOWN_WHEEL => {
                if action == MenuAction::DOWN && !time_for_move {
                    continue;
                }

                MoveMenuPositionSound();
                if menu_pos < num_entries - 1 {
                    menu_pos += 1;
                } else {
                    menu_pos = 0;
                }
                LAST_MOVE_TICK = SDL_GetTicks();
                need_update = true;
            }

            _ => {}
        }

        if quit_Menu {
            finished = true;
        }

        SDL_Delay(1); // don't hog CPU
    }

    ClearGraphMem();
    SDL_ShowCursor(SDL_ENABLE); // reactivate mouse-cursor for game
                                // Since we've faded out the whole scren, it can't hurt
                                // to have the top status bar redrawn...
    BannerIsDestroyed = true.into();
    Me.status = Status::Mobile as i32;

    while any_key_is_pressedR()
    // wait for all key/controller-release
    {
        SDL_Delay(1);
    }
}

/// subroutine to display the current key-config and highlight current selection
#[no_mangle]
pub unsafe extern "C" fn Display_Key_Config(selx: c_int, sely: c_int) {
    let startx = i32::from(Full_User_Rect.x) + (1.2 * f32::from(Block_Rect.w)) as i32;
    let starty = i32::from(Full_User_Rect.y) + FontHeight(&*GetCurrentFont());
    let col1 = startx + (7.5 * f64::from(CharWidth(&*GetCurrentFont(), b'O'.into()))) as i32;
    let col2 = col1 + (6.5 * f64::from(CharWidth(&*GetCurrentFont(), b'O'.into()))) as i32;
    let col3 = col2 + (6.5 * f64::from(CharWidth(&*GetCurrentFont(), b'O'.into()))) as i32;
    let lheight = FontHeight(&*Font0_BFont) + 2;

    SDL_UpperBlit(Menu_Background, null_mut(), ne_screen, null_mut());

    #[cfg(feature = "gcw0")]
    PrintStringFont(
        ne_screen,
        Font0_BFont,
        col1,
        starty,
        cstr!("(RShldr to clear an entry)").as_ptr() as *mut c_char,
    );

    #[cfg(not(feature = "gcw0"))]
    {
        PrintStringFont(
            ne_screen,
            Font0_BFont,
            col1,
            starty,
            cstr!("(RShldr to clear an entry)").as_ptr() as *mut c_char,
        );
        PrintStringFont(
            ne_screen,
            Font0_BFont,
            col1,
            starty,
            cstr!("(Backspace to clear an entry)").as_ptr() as *mut c_char,
        );
    }

    let mut posy = 1;
    PrintStringFont(
        ne_screen,
        Font0_BFont,
        startx,
        starty + (posy) * lheight,
        cstr!("Command").as_ptr() as *mut c_char,
    );
    PrintStringFont(
        ne_screen,
        Font0_BFont,
        col1,
        starty + (posy) * lheight,
        cstr!("Key1").as_ptr() as *mut c_char,
    );
    PrintStringFont(
        ne_screen,
        Font0_BFont,
        col2,
        starty + (posy) * lheight,
        cstr!("Key2").as_ptr() as *mut c_char,
    );
    PrintStringFont(
        ne_screen,
        Font0_BFont,
        col3,
        starty + (posy) * lheight,
        cstr!("Key3").as_ptr() as *mut c_char,
    );
    posy += 1;

    for i in 0..Cmds::Last as usize {
        let pos_font = |x, y| {
            if x != selx || i32::try_from(y).unwrap() != sely {
                Font1_BFont
            } else {
                Font2_BFont
            }
        };

        PrintStringFont(
            ne_screen,
            Font0_BFont,
            startx,
            starty + (posy) * lheight,
            cmd_strings[i] as *mut c_char,
        );
        PrintStringFont(
            ne_screen,
            pos_font(1, 1 + i),
            col1,
            starty + (posy) * lheight,
            keystr[usize::try_from(key_cmds[i][0]).unwrap()] as *mut c_char,
        );
        PrintStringFont(
            ne_screen,
            pos_font(2, 1 + i),
            col2,
            starty + (posy) * lheight,
            keystr[usize::try_from(key_cmds[i][1]).unwrap()] as *mut c_char,
        );
        PrintStringFont(
            ne_screen,
            pos_font(3, 1 + i),
            col3,
            starty + (posy) * lheight,
            keystr[usize::try_from(key_cmds[i][2]).unwrap()] as *mut c_char,
        );
        posy += 1;
    }

    SDL_Flip(ne_screen);
}

#[no_mangle]
pub unsafe extern "C" fn Key_Config_Menu() {
    let mut selx = 1;
    let mut sely = 1; // currently selected menu-position
    const WAIT_MOVE_TICKS: u32 = 100;
    static mut LAST_MOVE_TICK: u32 = 0;

    let mut finished = false;
    while !finished {
        Display_Key_Config(i32::try_from(selx).unwrap(), i32::try_from(sely).unwrap());

        let action = getMenuAction(250);
        let time_for_move = SDL_GetTicks() - LAST_MOVE_TICK > WAIT_MOVE_TICKS;

        match action {
            MenuAction::BACK => {
                finished = true;
                wait_for_all_keys_released();
            }

            MenuAction::CLICK => {
                MenuItemSelectedSound();

                key_cmds[sely - 1][selx - 1] = b'_'.into();
                Display_Key_Config(i32::try_from(selx).unwrap(), i32::try_from(sely).unwrap());
                key_cmds[sely - 1][selx - 1] = getchar_raw(); // includes joystick input!;
                wait_for_all_keys_released();
                LAST_MOVE_TICK = SDL_GetTicks();
            }

            MenuAction::UP | MenuAction::UP_WHEEL => {
                if action == MenuAction::UP && !time_for_move {
                    continue;
                }
                if sely > 1 {
                    sely -= 1;
                } else {
                    sely = Cmds::Last as usize;
                }
                MoveMenuPositionSound();
                LAST_MOVE_TICK = SDL_GetTicks();
            }

            MenuAction::DOWN | MenuAction::DOWN_WHEEL => {
                if action == MenuAction::DOWN && !time_for_move {
                    continue;
                }
                if sely < Cmds::Last as usize {
                    sely += 1;
                } else {
                    sely = 1;
                }
                MoveMenuPositionSound();
                LAST_MOVE_TICK = SDL_GetTicks();
            }

            MenuAction::RIGHT => {
                if !time_for_move {
                    continue;
                }

                if selx < 3 {
                    selx += 1;
                } else {
                    selx = 1;
                }
                MoveMenuPositionSound();
                LAST_MOVE_TICK = SDL_GetTicks();
            }

            MenuAction::LEFT => {
                if !time_for_move {
                    continue;
                }

                if selx > 1 {
                    selx -= 1;
                } else {
                    selx = 3;
                }
                MoveMenuPositionSound();
                LAST_MOVE_TICK = SDL_GetTicks();
            }

            MenuAction::DELETE => {
                key_cmds[sely - 1][selx - 1] = 0;
                MenuItemSelectedSound();
            }
            _ => {}
        }

        SDL_Delay(1);
    }
}

#[no_mangle]
pub unsafe extern "C" fn ShowCredits() {
    let col2 = 2 * i32::from(User_Rect.w) / 3;

    let h = FontHeight(&*Menu_BFont);
    let em = CharWidth(&*Menu_BFont, b'm'.into());

    let screen = Screen_Rect;
    SDL_SetClipRect(ne_screen, null_mut());
    DisplayImage(find_file(
        CREDITS_PIC_FILE_C.as_ptr() as *mut c_char,
        GRAPHICS_DIR_C.as_ptr() as *mut c_char,
        Themed::NoTheme as i32,
        Criticality::Critical as i32,
    ));
    MakeGridOnScreen(Some(&screen));

    let oldfont = GetCurrentFont();
    SetCurrentFont(Font1_BFont);

    printf_SDL(
        ne_screen,
        i32::from(get_user_center().x) - 2 * em,
        h,
        cstr!("CREDITS\n").as_ptr() as *mut c_char,
    );

    printf_SDL(
        ne_screen,
        em,
        -1,
        cstr!("PROGRAMMING:").as_ptr() as *mut c_char,
    );
    printf_SDL(
        ne_screen,
        col2,
        -1,
        cstr!("Johannes Prix\n").as_ptr() as *mut c_char,
    );
    printf_SDL(
        ne_screen,
        -1,
        -1,
        cstr!("Reinhard Prix\n").as_ptr() as *mut c_char,
    );
    printf_SDL(ne_screen, -1, -1, cstr!("\n").as_ptr() as *mut c_char);

    printf_SDL(ne_screen, em, -1, cstr!("ARTWORK:").as_ptr() as *mut c_char);
    printf_SDL(
        ne_screen,
        col2,
        -1,
        cstr!("Bastian Salmela\n").as_ptr() as *mut c_char,
    );
    printf_SDL(ne_screen, -1, -1, cstr!("\n").as_ptr() as *mut c_char);
    printf_SDL(
        ne_screen,
        em,
        -1,
        cstr!("ADDITIONAL THEMES:\n").as_ptr() as *mut c_char,
    );
    printf_SDL(
        ne_screen,
        2 * em,
        -1,
        cstr!("Lanzz-theme").as_ptr() as *mut c_char,
    );
    printf_SDL(
        ne_screen,
        col2,
        -1,
        cstr!("Lanzz\n").as_ptr() as *mut c_char,
    );
    printf_SDL(
        ne_screen,
        2 * em,
        -1,
        cstr!("Para90-theme").as_ptr() as *mut c_char,
    );
    printf_SDL(
        ne_screen,
        col2,
        -1,
        cstr!("Andreas Wedemeyer\n").as_ptr() as *mut c_char,
    );

    printf_SDL(ne_screen, -1, -1, cstr!("\n").as_ptr() as *mut c_char);
    printf_SDL(
        ne_screen,
        em,
        -1,
        cstr!("C64 LEGACY MODS:\n").as_ptr() as *mut c_char,
    );

    printf_SDL(
        ne_screen,
        2 * em,
        -1,
        cstr!("Green Beret, Sanxion, Uridium2").as_ptr() as *mut c_char,
    );
    printf_SDL(
        ne_screen,
        col2,
        -1,
        cstr!("#dreamfish/trsi\n").as_ptr() as *mut c_char,
    );

    printf_SDL(
        ne_screen,
        2 * em,
        -1,
        cstr!("The last V8, Anarchy").as_ptr() as *mut c_char,
    );
    printf_SDL(
        ne_screen,
        col2,
        -1,
        cstr!("4-mat\n").as_ptr() as *mut c_char,
    );

    printf_SDL(ne_screen, 2 * em, -1, cstr!("Tron").as_ptr() as *mut c_char);
    printf_SDL(
        ne_screen,
        col2,
        -1,
        cstr!("Kollaps\n").as_ptr() as *mut c_char,
    );

    printf_SDL(
        ne_screen,
        2 * em,
        -1,
        cstr!("Starpaws").as_ptr() as *mut c_char,
    );
    printf_SDL(
        ne_screen,
        col2,
        -1,
        cstr!("Nashua\n").as_ptr() as *mut c_char,
    );

    printf_SDL(
        ne_screen,
        2 * em,
        -1,
        cstr!("Commando").as_ptr() as *mut c_char,
    );
    printf_SDL(
        ne_screen,
        col2,
        -1,
        cstr!("Android").as_ptr() as *mut c_char,
    );

    SDL_Flip(ne_screen);
    wait_for_key_pressed();
    SetCurrentFont(oldfont);
}

/// simple wrapper to ShowMenu() to provide the external entry point into the Level Editor menu
#[no_mangle]
pub unsafe extern "C" fn showLevelEditorMenu() {
    quit_LevelEditor = false;
    ShowMenu(LevelEditorMenu);
}

#[no_mangle]
pub unsafe extern "C" fn handle_ConfigureKeys(action: MenuAction) -> *const c_char {
    if action == MenuAction::CLICK {
        MenuItemSelectedSound();
        Key_Config_Menu();
    }

    null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn handle_Highscores(action: MenuAction) -> *const c_char {
    if action == MenuAction::CLICK {
        MenuItemSelectedSound();
        ShowHighscores();
    }
    null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn handle_Credits(action: MenuAction) -> *const c_char {
    if action == MenuAction::CLICK {
        MenuItemSelectedSound();
        ShowCredits();
    }

    null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn handle_LE_SaveShip(action: MenuAction) -> *const c_char {
    const SHIPNAME: &CStr = cstr!("Testship");
    static mut FNAME: [c_char; 255] = [0; 255];
    libc::snprintf(
        FNAME.as_mut_ptr(),
        FNAME.len() - 1,
        cstr!("%s%s").as_ptr() as *mut c_char,
        SHIPNAME.as_ptr() as *mut c_char,
        SHIP_EXT.as_ptr() as *mut c_char,
    );

    if action == MenuAction::INFO {
        return FNAME.as_ptr();
    }

    if action == MenuAction::CLICK {
        SaveShip(SHIPNAME.as_ptr());
        let mut output = [0; 255];
        libc::snprintf(
            output.as_mut_ptr(),
            output.len() - 1,
            cstr!("Ship saved as '%s'").as_ptr() as *mut c_char,
            FNAME,
        );
        CenteredPutString(ne_screen, 3 * FontHeight(&*Menu_BFont), output.as_mut_ptr());
        SDL_Flip(ne_screen);
        wait_for_key_pressed();
        InitiateMenu(false);
    }

    null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn handle_LE_Comment(action: MenuAction) -> *const c_char {
    if action == MenuAction::INFO {
        (*CurLevel).Level_Enter_Comment
    } else {
        null_mut()
    }
}
