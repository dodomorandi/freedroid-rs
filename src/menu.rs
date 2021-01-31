#[cfg(not(feature = "gcw0"))]
use crate::input::{key_cmds, wait_for_all_keys_released, wait_for_key_pressed};

#[cfg(feature = "gcw0")]
use crate::{
    defs::{gcw0_a_pressed, gcw0_any_button_pressed, gcw0_any_button_pressed_r},
    input::SDL_Delay,
};

use crate::{
    b_font::{FontHeight, GetCurrentFont, PutString, SetCurrentFont, TextWidth},
    defs::{
        self, AssembleCombatWindowFlags, Cmds, DisplayBannerFlags, DownPressed, Droid, FirePressed,
        LeftPressed, MenuAction, ReturnPressedR, RightPressed, Status, UpPressed,
    },
    global::{
        curShip, show_all_droids, sound_on, stop_influencer, AllEnemys, CurLevel,
        CurrentCombatScaleFactor, Druidmap, Font0_BFont, InvincibleMode, Me, Menu_BFont, NumEnemys,
        Number_Of_Droid_Types, User_Rect,
    },
    graphics::{ne_screen, ClearGraphMem, MakeGridOnScreen, SetCombatScaleTo},
    input::{
        cmd_is_activeR, update_input, KeyIsPressed, KeyIsPressedR, WheelDownPressed, WheelUpPressed,
    },
    misc::{Activate_Conservative_Frame_Computation, Armageddon, Teleport, Terminate},
    ship::ShowDeckMap,
    sound::MenuItemSelectedSound,
    text::{getchar_raw, printf_SDL, GetString},
    vars::InfluenceModeNames,
    view::{Assemble_Combat_Picture, DisplayBanner},
};

use cstr::cstr;
use sdl::{
    keysym::{SDLK_BACKSPACE, SDLK_DOWN, SDLK_ESCAPE, SDLK_LEFT, SDLK_RIGHT, SDLK_UP},
    mouse::ll::{SDL_ShowCursor, SDL_DISABLE},
    sdl::ll::SDL_GetTicks,
    video::ll::{SDL_DisplayFormat, SDL_Flip, SDL_FreeSurface, SDL_SetClipRect, SDL_Surface},
};
use std::{
    convert::{TryFrom, TryInto},
    ffi::CStr,
    os::raw::{c_char, c_int, c_void},
    ptr::null_mut,
};

extern "C" {
    pub fn ShowMenu(menu_entries: *const MenuEntry);
    pub static mut fheight: c_int;
    pub static mut Menu_Background: *mut SDL_Surface;

    #[cfg(target = "android")]
    pub static MainMenu: [MenuEntry; 8];

    #[cfg(not(target = "android"))]
    pub static MainMenu: [MenuEntry; 10];
}

#[repr(C)]
pub struct MenuEntry {
    name: *const c_char,
    handler: unsafe extern "C" fn() -> *const c_char,
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
    static mut last_movekey_time: u32 = 0;

    static mut up: bool = false;
    static mut down: bool = false;
    static mut left: bool = false;
    static mut right: bool = false;

    // we register if there have been key-press events in the "waiting period" between move-ticks
    if !up && (UpPressed() || KeyIsPressed(SDLK_UP as c_int)) {
        up = true;
        last_movekey_time = SDL_GetTicks();
        action |= MenuAction::UP;
    }
    if !down && (DownPressed() || KeyIsPressed(SDLK_DOWN as c_int)) {
        down = true;
        last_movekey_time = SDL_GetTicks();
        action |= MenuAction::DOWN;
    }
    if !left && (LeftPressed() || KeyIsPressed(SDLK_LEFT as c_int)) {
        left = true;
        last_movekey_time = SDL_GetTicks();
        action |= MenuAction::LEFT;
    }
    if !right && (RightPressed() || KeyIsPressed(SDLK_RIGHT as c_int)) {
        right = true;
        last_movekey_time = SDL_GetTicks();
        action |= MenuAction::RIGHT;
    }

    if !(UpPressed() || KeyIsPressed(SDLK_UP as c_int)) {
        up = false;
    }
    if !(DownPressed() || KeyIsPressed(SDLK_DOWN as c_int)) {
        down = false;
    }
    if !(LeftPressed() || KeyIsPressed(SDLK_LEFT as c_int)) {
        left = false;
    }
    if !(RightPressed() || KeyIsPressed(SDLK_RIGHT as c_int)) {
        right = false;
    }

    // check if enough time since we registered last new move-action
    if SDL_GetTicks() - last_movekey_time > wait_repeat_ticks {
        if up {
            action |= MenuAction::UP;
        }
        if down {
            action |= MenuAction::DOWN;
        }
        if left {
            action |= MenuAction::LEFT;
        }
        if right {
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
