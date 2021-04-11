#[cfg(not(feature = "gcw0"))]
use crate::input::{wait_for_all_keys_released, wait_for_key_pressed, KEY_CMDS};

#[cfg(feature = "gcw0")]
use crate::{
    defs::{gcw0_a_pressed, gcw0_any_button_pressed, gcw0_any_button_pressed_r},
    input::SDL_Delay,
};

use crate::{
    b_font::{
        centered_put_string, char_width, font_height, get_current_font, print_string_font,
        put_string, set_current_font, text_width,
    },
    defs::{
        self, down_pressed, fire_pressed, get_user_center, left_pressed, return_pressed_r,
        right_pressed, up_pressed, AssembleCombatWindowFlags, Cmds, Criticality,
        DisplayBannerFlags, Droid, MapTile, MenuAction, Status, Themed, BYCOLOR,
        CREDITS_PIC_FILE_C, GRAPHICS_DIR_C, MAX_MAP_COLS, MAX_MAP_ROWS,
    },
    global::{
        CURRENT_COMBAT_SCALE_FACTOR, FONT0_B_FONT, FONT1_B_FONT, FONT2_B_FONT, GAME_CONFIG,
        INFLUENCE_MODE_NAMES, MENU_B_FONT,
    },
    graphics::{
        clear_graph_mem, display_image, init_pictures, make_grid_on_screen, set_combat_scale_to,
        toggle_fullscreen, ALL_THEMES, BANNER_IS_DESTROYED, CLASSIC_THEME_INDEX, NE_SCREEN,
    },
    highscore::show_highscores,
    input::{
        any_key_is_pressed_r, cmd_is_active_r, key_is_pressed, key_is_pressed_r, update_input,
        wheel_down_pressed, wheel_up_pressed, SDL_Delay, CMD_STRINGS, KEYSTR,
    },
    level_editor::level_editor,
    map::{save_ship, COLOR_NAMES},
    misc::{
        activate_conservative_frame_computation, armageddon, find_file, my_malloc, teleport,
        terminate,
    },
    ship::show_deck_map,
    sound::{
        menu_item_selected_sound, move_lift_sound, move_menu_position_sound, set_bg_music_volume,
        set_sound_f_x_volume, switch_background_music_to,
    },
    text::{display_text, get_string, getchar_raw, printf_sdl},
    vars::{
        BLOCK_RECT, CLASSIC_USER_RECT, DRUIDMAP, FULL_USER_RECT, ME, MENU_RECT, SCREEN_RECT,
        USER_RECT,
    },
    view::{assemble_combat_picture, display_banner, put_influence},
    ALL_ENEMYS, CUR_LEVEL, CUR_SHIP, INVINCIBLE_MODE, NUMBER_OF_DROID_TYPES, NUM_ENEMYS,
    SHOW_ALL_DROIDS, SOUND_ON, STOP_INFLUENCER,
};

use cstr::cstr;
use log::error;
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
    io::Cursor,
    ops::{AddAssign, Not, SubAssign},
    os::raw::{c_char, c_float, c_int, c_void},
    ptr::null_mut,
};

static mut FONT_HEIGHT: i32 = 0;
static mut MENU_BACKGROUND: *mut SDL_Surface = null_mut();
static mut QUIT_MENU: bool = false;

pub static mut QUIT_LEVEL_EDITOR: bool = false;

// const FILENAME_LEN: u8 = 128;
const SHIP_EXT_C: &CStr = cstr!(".shp");
pub const SHIP_EXT: &str = ".shp";
// const ELEVEXT: &CStr = cstr!(".elv");
// const CREWEXT: &CStr = cstr!(".crw");

macro_rules! menu_entry {
    () => {
        MenuEntry {
            name: null_mut(),
            handler: None,
            submenu: null_mut(),
        }
    };
    ($name:tt) => {
        MenuEntry {
            name: cstr!($name).as_ptr(),
            handler: None,
            submenu: null_mut(),
        }
    };
    ($name:tt, $handler:expr) => {
        MenuEntry {
            name: cstr!($name).as_ptr(),
            handler: Some($handler),
            submenu: null_mut(),
        }
    };
    ($name:tt, None, $submenu:expr) => {
        MenuEntry {
            name: cstr!($name).as_ptr(),
            handler: None,
            submenu: $submenu.as_ptr(),
        }
    };
    ($name:tt, $handler:expr, $submenu:expr) => {
        MenuEntry {
            name: cstr!($name).as_ptr(),
            handler: Some($handler),
            submenu: $submenu.as_ptr(),
        }
    };
}

#[cfg(target_os = "android")]
const LEGACY_MENU: [MenuEntry; 9] = [
    menu_entry! { "Back" },
    menu_entry! { "Set Strictly Classic", handle_strictly_classic},
    menu_entry! { "Combat Window: ", handle_window_type},
    menu_entry! { "Graphics Theme: ", handle_theme},
    menu_entry! { "Droid Talk: ", handle_droid_talk},
    menu_entry! { "Show Decals: ", handle_show_decals},
    menu_entry! { "All Map Visible: ", handle_all_map_visible},
    menu_entry! { "Empty Level Speedup: ", handle_empty_level_speedup},
    menu_entry! {},
];

#[cfg(not(target_os = "android"))]
const LEGACY_MENU: [MenuEntry; 11] = [
    menu_entry! { "Back"},
    menu_entry! { "Set Strictly Classic", handle_strictly_classic},
    menu_entry! { "Combat Window: ", handle_window_type},
    menu_entry! { "Graphics Theme: ", handle_theme},
    menu_entry! { "Droid Talk: ", handle_droid_talk},
    menu_entry! { "Show Decals: ", handle_show_decals},
    menu_entry! { "All Map Visible: ", handle_all_map_visible},
    menu_entry! { "Transfer = Activate: ", handle_transfer_is_activate},
    menu_entry! { "Hold Fire to Transfer: ", handle_fire_is_transfer},
    menu_entry! { "Empty Level Speedup: ", handle_empty_level_speedup},
    menu_entry! {},
];

const GRAPHICS_SOUND_MENU: [MenuEntry; 5] = [
    menu_entry! { "Back"},
    menu_entry! { "Music Volume: ", handle_music_volume},
    menu_entry! { "Sound Volume: ", handle_sound_volume},
    menu_entry! { "Fullscreen Mode: ", handle_fullscreen},
    menu_entry! {},
];

const HUD_MENU: [MenuEntry; 5] = [
    menu_entry! { "Back"},
    menu_entry! { "Show Position: ", handle_show_position},
    menu_entry! { "Show Framerate: ", handle_show_framerate},
    menu_entry! { "Show Energy: ", handle_show_energy},
    menu_entry! {},
];

const LEVEL_EDITOR_MENU: [MenuEntry; 8] = [
    menu_entry! { "Exit Level Editor", 	handle_le_exit},
    menu_entry! { "Current Level: ", handle_le_level_number},
    menu_entry! { "Level Color: ", handle_le_color},
    menu_entry! { "Levelsize X: ", handle_le_size_x},
    menu_entry! { "Levelsize Y: ", handle_le_size_y},
    menu_entry! { "Level Name: ", handle_le_name},
    menu_entry! { "Save ship: ", handle_le_save_ship},
    menu_entry! {},
];

#[cfg(target_os = "android")]
const MAIN_MENU: [MenuEntry; 8] = [
    menu_entry! { "Back to Game"},
    menu_entry! { "Graphics & Sound", None, GRAPHICS_SOUND_MENU },
    menu_entry! { "Legacy Options", None, LEGACY_MENU },
    menu_entry! { "HUD Settings", None, HUD_MENU },
    menu_entry! { "Highscores", handle_highscores},
    menu_entry! { "Credits", handle_credits},
    menu_entry! { "Quit Game", handle_quit_game},
    menu_entry! {},
];

#[cfg(not(target_os = "android"))]
const MAIN_MENU: [MenuEntry; 10] = [
    menu_entry! { "Back to Game"},
    menu_entry! { "Graphics & Sound", None, GRAPHICS_SOUND_MENU },
    menu_entry! { "Legacy Options", None, LEGACY_MENU },
    menu_entry! { "HUD Settings", None, HUD_MENU },
    menu_entry! { "Level Editor", handle_open_level_editor},
    menu_entry! { "Highscores", handle_highscores},
    menu_entry! { "Credits", handle_credits},
    menu_entry! { "Configure Keys", handle_configure_keys},
    menu_entry! { "Quit Game", handle_quit_game},
    menu_entry! {},
];

pub struct MenuEntry {
    name: *const c_char,
    handler: Option<unsafe fn(MenuAction) -> *const c_char>,
    submenu: *const MenuEntry,
}

pub unsafe fn handle_quit_game(action: MenuAction) -> *const c_char {
    if action != MenuAction::CLICK {
        return null_mut();
    }

    menu_item_selected_sound();
    initiate_menu(false);

    #[cfg(feature = "gcw0")]
    const QUIT_STRING: &CStr = cstr!("Press A to quit");

    #[cfg(not(feature = "gcw0"))]
    const QUIT_STRING: &[u8] = b"Hit 'y' or press Fire to quit";

    let text_width = text_width(QUIT_STRING);
    let text_x = i32::from(USER_RECT.x) + (i32::from(USER_RECT.w) - text_width) / 2;
    let text_y = i32::from(USER_RECT.y) + (i32::from(USER_RECT.h) - FONT_HEIGHT) / 2;
    put_string(NE_SCREEN, text_x, text_y, QUIT_STRING);
    SDL_Flip(NE_SCREEN);

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
            || key == KEY_CMDS[Cmds::Fire as usize][0]
            || key == KEY_CMDS[Cmds::Fire as usize][1]
            || key == KEY_CMDS[Cmds::Fire as usize][2]
        {
            terminate(defs::OK.into());
        }
    }

    null_mut()
}

/// simple wrapper to ShowMenu() to provide the external entry point into the main menu
pub unsafe fn show_main_menu() {
    show_menu(MAIN_MENU.as_ptr());
}

pub unsafe fn free_menu_data() {
    SDL_FreeSurface(MENU_BACKGROUND);
}

pub unsafe fn initiate_menu(with_droids: bool) {
    // Here comes the standard initializer for all the menus and submenus
    // of the big escape menu.  This prepares the screen, so that we can
    // write on it further down.
    activate_conservative_frame_computation();

    SDL_SetClipRect(NE_SCREEN, null_mut());
    ME.status = Status::Menu as i32;
    clear_graph_mem();
    display_banner(
        null_mut(),
        null_mut(),
        (DisplayBannerFlags::NO_SDL_UPDATE | DisplayBannerFlags::FORCE_UPDATE)
            .bits()
            .into(),
    );
    if with_droids {
        assemble_combat_picture(0);
    } else {
        assemble_combat_picture(AssembleCombatWindowFlags::ONLY_SHOW_MAP.bits().into());
    }

    SDL_SetClipRect(NE_SCREEN, null_mut());
    make_grid_on_screen(None);

    if !MENU_BACKGROUND.is_null() {
        SDL_FreeSurface(MENU_BACKGROUND);
    }
    MENU_BACKGROUND = SDL_DisplayFormat(NE_SCREEN); // keep a global copy of background

    SDL_ShowCursor(SDL_DISABLE); // deactivate mouse-cursor in menus
    set_current_font(MENU_B_FONT);
    FONT_HEIGHT = font_height(&*get_current_font()) + 2;
}

pub unsafe fn cheatmenu() {
    // Prevent distortion of framerate by the delay coming from
    // the time spend in the menu.
    activate_conservative_frame_computation();

    let font = FONT0_B_FONT;

    set_current_font(font); /* not the ideal one, but there's currently */
    /* no other it seems.. */
    const X0: i32 = 50;
    const Y0: i32 = 20;

    let cur_level = &mut *CUR_LEVEL;
    let droid_map = std::slice::from_raw_parts(DRUIDMAP, Droid::NumDroids as usize);
    let mut resume = false;
    while !resume {
        clear_graph_mem();
        printf_sdl(
            NE_SCREEN,
            X0,
            Y0,
            format_args!(
                "Current position: Level={}, X={:.0}, Y={:.0}\n",
                cur_level.levelnum, ME.pos.x, ME.pos.y,
            ),
        );
        printf_sdl(
            NE_SCREEN,
            -1,
            -1,
            format_args!(" a. Armageddon (alle Robots sprengen)\n"),
        );
        printf_sdl(
            NE_SCREEN,
            -1,
            -1,
            format_args!(" l. robot list of current level\n"),
        );
        printf_sdl(NE_SCREEN, -1, -1, format_args!(" g. complete robot list\n"));
        printf_sdl(
            NE_SCREEN,
            -1,
            -1,
            format_args!(" d. destroy robots on current level\n"),
        );
        printf_sdl(NE_SCREEN, -1, -1, format_args!(" t. Teleportation\n"));
        printf_sdl(
            NE_SCREEN,
            -1,
            -1,
            format_args!(" r. change to new robot type\n"),
        );
        printf_sdl(
            NE_SCREEN,
            -1,
            -1,
            format_args!(
                " i. Invinciblemode: {}\n",
                if INVINCIBLE_MODE != 0 { "ON" } else { "OFF" },
            ),
        );
        printf_sdl(NE_SCREEN, -1, -1, format_args!(" e. set energy\n"));
        printf_sdl(
            NE_SCREEN,
            -1,
            -1,
            format_args!(
                " n. No hidden droids: {}\n",
                if SHOW_ALL_DROIDS != 0 { "ON" } else { "OFF" },
            ),
        );
        printf_sdl(NE_SCREEN, -1, -1, format_args!(" m. Map of Deck xy\n"));
        printf_sdl(
            NE_SCREEN,
            -1,
            -1,
            format_args!(" s. Sound: {}\n", if SOUND_ON != 0 { "ON" } else { "OFF" }),
        );
        printf_sdl(
            NE_SCREEN,
            -1,
            -1,
            format_args!(" w. Print current waypoints\n"),
        );
        printf_sdl(NE_SCREEN, -1, -1, format_args!(" z. change Zoom factor\n"));
        printf_sdl(
            NE_SCREEN,
            -1,
            -1,
            format_args!(
                " f. Freeze on this positon: {}\n",
                if STOP_INFLUENCER != 0 { "ON" } else { "OFF" },
            ),
        );
        printf_sdl(NE_SCREEN, -1, -1, format_args!(" q. RESUME game\n"));

        match u8::try_from(getchar_raw()).ok() {
            Some(b'f') => {
                STOP_INFLUENCER = !STOP_INFLUENCER;
            }

            Some(b'z') => {
                clear_graph_mem();
                printf_sdl(
                    NE_SCREEN,
                    X0,
                    Y0,
                    format_args!("Current Zoom factor: {}\n", CURRENT_COMBAT_SCALE_FACTOR),
                );
                printf_sdl(NE_SCREEN, -1, -1, format_args!("New zoom factor: "));
                let input = get_string(40, 2);
                libc::sscanf(
                    input,
                    cstr!("%f").as_ptr() as *mut c_char,
                    &mut CURRENT_COMBAT_SCALE_FACTOR,
                );
                libc::free(input as *mut c_void);
                set_combat_scale_to(CURRENT_COMBAT_SCALE_FACTOR);
            }

            Some(b'a') => {
                /* armageddon */
                resume = true;
                armageddon();
            }

            Some(b'l') => {
                /* robot list of this deck */
                let mut l = 0; /* line counter for enemy output */
                for i in 0..usize::try_from(NUM_ENEMYS).unwrap() {
                    if ALL_ENEMYS[i].levelnum == cur_level.levelnum {
                        if l != 0 && l % 20 == 0 {
                            printf_sdl(NE_SCREEN, -1, -1, format_args!(" --- MORE --- \n"));
                            if getchar_raw() == b'q'.into() {
                                break;
                            }
                        }
                        if l % 20 == 0 {
                            clear_graph_mem();
                            printf_sdl(
                                NE_SCREEN,
                                X0,
                                Y0,
                                format_args!(" NR.   ID  X    Y   ENERGY   Status\n"),
                            );
                            printf_sdl(
                                NE_SCREEN,
                                -1,
                                -1,
                                format_args!("---------------------------------------------\n"),
                            );
                        }

                        l += 1;
                        let status = if ALL_ENEMYS[i].status == Status::Out as i32 {
                            "OUT"
                        } else if ALL_ENEMYS[i].status == Status::Terminated as i32 {
                            "DEAD"
                        } else {
                            "ACTIVE"
                        };

                        printf_sdl(
                            NE_SCREEN,
                            -1,
                            -1,
                            format_args!(
                                "{}.   {}   {:.0}   {:.0}   {:.0}    {}.\n",
                                i,
                                CStr::from_ptr(
                                    droid_map[usize::try_from(ALL_ENEMYS[i].ty).unwrap()]
                                        .druidname
                                        .as_ptr()
                                )
                                .to_str()
                                .unwrap(),
                                ALL_ENEMYS[i].pos.x,
                                ALL_ENEMYS[i].pos.y,
                                ALL_ENEMYS[i].energy,
                                status,
                            ),
                        );
                    }
                }

                printf_sdl(NE_SCREEN, -1, -1, format_args!(" --- END --- \n"));
                getchar_raw();
            }

            Some(b'g') => {
                /* complete robot list of this ship */
                for i in 0..usize::try_from(NUM_ENEMYS).unwrap() {
                    if ALL_ENEMYS[i].ty == -1 {
                        continue;
                    }

                    if i != 0 && !i % 13 == 0 {
                        printf_sdl(
                            NE_SCREEN,
                            -1,
                            -1,
                            format_args!(" --- MORE --- ('q' to quit)\n"),
                        );
                        if getchar_raw() == b'q'.into() {
                            break;
                        }
                    }
                    if i % 13 == 0 {
                        clear_graph_mem();
                        printf_sdl(
                            NE_SCREEN,
                            X0,
                            Y0,
                            format_args!("Nr.  Lev. ID  Energy  Status.\n"),
                        );
                        printf_sdl(
                            NE_SCREEN,
                            -1,
                            -1,
                            format_args!("------------------------------\n"),
                        );
                    }

                    printf_sdl(
                        NE_SCREEN,
                        -1,
                        -1,
                        format_args!(
                            "{}  {}  {}  {:.0}  {}\n",
                            i,
                            ALL_ENEMYS[i].levelnum,
                            CStr::from_ptr(
                                droid_map[usize::try_from(ALL_ENEMYS[i].ty).unwrap()]
                                    .druidname
                                    .as_ptr()
                            )
                            .to_str()
                            .unwrap(),
                            ALL_ENEMYS[i].energy,
                            INFLUENCE_MODE_NAMES[usize::try_from(ALL_ENEMYS[i].status).unwrap()]
                                .to_str()
                                .unwrap(),
                        ),
                    );
                }

                printf_sdl(NE_SCREEN, -1, -1, format_args!(" --- END ---\n"));
                getchar_raw();
            }

            Some(b'd') => {
                /* destroy all robots on this level, haha */
                for enemy in &mut ALL_ENEMYS {
                    if enemy.levelnum == cur_level.levelnum {
                        enemy.energy = -100.;
                    }
                }
                printf_sdl(
                    NE_SCREEN,
                    -1,
                    -1,
                    format_args!("All robots on this deck killed!\n"),
                );
                getchar_raw();
            }

            Some(b't') => {
                /* Teleportation */
                clear_graph_mem();
                printf_sdl(NE_SCREEN, X0, Y0, format_args!("Enter Level, X, Y: "));
                let input = get_string(40, 2);
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
                teleport(l_num, x, y);
            }

            Some(b'r') => {
                /* change to new robot type */
                clear_graph_mem();
                printf_sdl(
                    NE_SCREEN,
                    X0,
                    Y0,
                    format_args!("Type number of new robot: "),
                );
                let input = get_string(40, 2);
                let mut i = 0;
                for _ in 0..u32::try_from(NUMBER_OF_DROID_TYPES).unwrap() {
                    if libc::strcmp(droid_map[i].druidname.as_ptr(), input) != 0 {
                        break;
                    }
                    i += 1;
                }

                if i == usize::try_from(NUMBER_OF_DROID_TYPES).unwrap() {
                    printf_sdl(
                        NE_SCREEN,
                        X0,
                        Y0 + 20,
                        format_args!(
                            "Unrecognized robot-type: {}",
                            CStr::from_ptr(input).to_str().unwrap(),
                        ),
                    );
                    getchar_raw();
                    clear_graph_mem();
                } else {
                    ME.ty = i.try_into().unwrap();
                    ME.energy = droid_map[usize::try_from(ME.ty).unwrap()].maxenergy;
                    ME.health = ME.energy;
                    printf_sdl(
                        NE_SCREEN,
                        X0,
                        Y0 + 20,
                        format_args!(
                            "You are now a {}. Have fun!\n",
                            CStr::from_ptr(input).to_str().unwrap(),
                        ),
                    );
                    getchar_raw();
                }
                libc::free(input as *mut c_void);
            }

            Some(b'i') => {
                /* togge Invincible mode */
                INVINCIBLE_MODE = !INVINCIBLE_MODE;
            }

            Some(b'e') => {
                /* complete heal */
                clear_graph_mem();
                printf_sdl(
                    NE_SCREEN,
                    X0,
                    Y0,
                    format_args!("Current energy: {}\n", ME.energy,),
                );
                printf_sdl(NE_SCREEN, -1, -1, format_args!("Enter your new energy: "));
                let input = get_string(40, 2);
                let mut num = 0;
                libc::sscanf(input, cstr!("%d").as_ptr() as *mut c_char, &mut num);
                libc::free(input as *mut c_void);
                ME.energy = num as f32;
                if ME.energy > ME.health {
                    ME.health = ME.energy;
                }
            }

            Some(b'n') => {
                /* toggle display of all droids */
                SHOW_ALL_DROIDS = !SHOW_ALL_DROIDS;
            }

            Some(b's') => {
                /* toggle sound on/off */
                SOUND_ON = !SOUND_ON;
            }

            Some(b'm') => {
                /* Show deck map in Concept view */
                printf_sdl(NE_SCREEN, -1, -1, format_args!("\nLevelnum: "));
                let input = get_string(40, 2);
                let mut l_num = 0;
                libc::sscanf(input, cstr!("%d").as_ptr() as *mut c_char, &mut l_num);
                libc::free(input as *mut c_void);
                show_deck_map();
                getchar_raw();
            }

            Some(b'w') => {
                /* print waypoint info of current level */
                for (i, waypoint) in cur_level.all_waypoints.iter_mut().enumerate() {
                    if i != 0 && i % 20 == 0 {
                        printf_sdl(NE_SCREEN, -1, -1, format_args!(" ---- MORE -----\n"));
                        if getchar_raw() == b'q'.into() {
                            break;
                        }
                    }
                    if i % 20 == 0 {
                        clear_graph_mem();
                        printf_sdl(
                            NE_SCREEN,
                            X0,
                            Y0,
                            format_args!("Nr.   X   Y      C1  C2  C3  C4\n"),
                        );
                        printf_sdl(
                            NE_SCREEN,
                            -1,
                            -1,
                            format_args!("------------------------------------\n"),
                        );
                    }
                    printf_sdl(
                        NE_SCREEN,
                        -1,
                        -1,
                        format_args!(
                            "{:2}   {:2}  {:2}      {:2}  {:2}  {:2}  {:2}\n",
                            i,
                            waypoint.x,
                            waypoint.y,
                            waypoint.connections[0],
                            waypoint.connections[1],
                            waypoint.connections[2],
                            waypoint.connections[3],
                        ),
                    );
                }
                printf_sdl(NE_SCREEN, -1, -1, format_args!(" --- END ---\n"));
                getchar_raw();
            }

            Some(b' ') | Some(b'q') => {
                resume = true;
            }

            _ => {}
        }
    }

    clear_graph_mem();

    update_input(); /* treat all pending keyboard events */
}

/// get menu input actions
///
/// NOTE: built-in time delay to ensure spurious key-repetitions
/// such as from touchpad 'wheel' or android joystic emulation
/// don't create unexpected menu movements:
/// ==> ignore all movement commands withing delay_ms milliseconds of each other
pub unsafe fn get_menu_action(wait_repeat_ticks: u32) -> MenuAction {
    let mut action = MenuAction::empty();

    // 'normal' menu action keys get released
    if key_is_pressed_r(SDLK_BACKSPACE as c_int) {
        {
            action = MenuAction::DELETE;
        }
    }
    if cmd_is_active_r(Cmds::Back) || key_is_pressed_r(SDLK_ESCAPE as c_int) {
        {
            action = MenuAction::BACK;
        }
    }

    if fire_pressed() || return_pressed_r() {
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
    if !UP && (up_pressed() || key_is_pressed(SDLK_UP as c_int)) {
        UP = true;
        LAST_MOVEKEY_TIME = SDL_GetTicks();
        action |= MenuAction::UP;
    }
    if !DOWN && (down_pressed() || key_is_pressed(SDLK_DOWN as c_int)) {
        DOWN = true;
        LAST_MOVEKEY_TIME = SDL_GetTicks();
        action |= MenuAction::DOWN;
    }
    if !LEFT && (left_pressed() || key_is_pressed(SDLK_LEFT as c_int)) {
        LEFT = true;
        LAST_MOVEKEY_TIME = SDL_GetTicks();
        action |= MenuAction::LEFT;
    }
    if !RIGHT && (right_pressed() || key_is_pressed(SDLK_RIGHT as c_int)) {
        RIGHT = true;
        LAST_MOVEKEY_TIME = SDL_GetTicks();
        action |= MenuAction::RIGHT;
    }

    if !(up_pressed() || key_is_pressed(SDLK_UP as c_int)) {
        UP = false;
    }
    if !(down_pressed() || key_is_pressed(SDLK_DOWN as c_int)) {
        DOWN = false;
    }
    if !(left_pressed() || key_is_pressed(SDLK_LEFT as c_int)) {
        LEFT = false;
    }
    if !(right_pressed() || key_is_pressed(SDLK_RIGHT as c_int)) {
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
    if wheel_up_pressed() {
        action |= MenuAction::UP_WHEEL;
    }
    if wheel_down_pressed() {
        action |= MenuAction::DOWN_WHEEL;
    }

    action
}

/// Generic menu handler
pub unsafe fn show_menu(menu_entries: *const MenuEntry) {
    use std::io::Write;

    initiate_menu(false);
    wait_for_all_keys_released();

    // figure out menu-start point to make it centered
    let mut num_entries = 0;
    let mut menu_width = None::<i32>;
    loop {
        let entry = &*menu_entries.add(num_entries);
        if entry.name.is_null() {
            break;
        }

        let width = text_width(CStr::from_ptr(entry.name).to_bytes());
        menu_width = Some(
            menu_width
                .map(|menu_width| menu_width.max(width))
                .unwrap_or(width),
        );

        num_entries += 1;
    }
    let menu_entries = std::slice::from_raw_parts(menu_entries, num_entries);
    let menu_width = menu_width.unwrap();

    let menu_height = i32::try_from(num_entries).unwrap() * FONT_HEIGHT;
    let menu_x = i32::from(FULL_USER_RECT.x) + (i32::from(FULL_USER_RECT.w) - menu_width) / 2;
    let menu_y = i32::from(FULL_USER_RECT.y) + (i32::from(FULL_USER_RECT.h) - menu_height) / 2;
    let influ_x = menu_x - i32::from(BLOCK_RECT.w) - FONT_HEIGHT;

    let mut menu_pos = 0;

    let wait_move_ticks: u32 = 100;
    static mut LAST_MOVE_TICK: u32 = 0;
    let mut finished = false;
    QUIT_MENU = false;
    let mut need_update = true;
    while !finished {
        let handler = menu_entries[menu_pos].handler;
        let submenu = menu_entries[menu_pos].submenu;

        if need_update {
            SDL_UpperBlit(MENU_BACKGROUND, null_mut(), NE_SCREEN, null_mut());
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

                let mut full_name: [u8; 256] = [0; 256];
                let mut cursor = Cursor::new(full_name.as_mut());
                write!(
                    cursor,
                    "{}{}",
                    CStr::from_ptr(entry.name).to_str().unwrap(),
                    CStr::from_ptr(arg).to_str().unwrap()
                )
                .unwrap();
                let position = usize::try_from(cursor.position()).unwrap();
                put_string(
                    NE_SCREEN,
                    menu_x,
                    menu_y + i32::try_from(i).unwrap() * FONT_HEIGHT,
                    &full_name[..position],
                );
            });
            put_influence(
                influ_x,
                menu_y + ((menu_pos as f64 - 0.5) * f64::from(FONT_HEIGHT)) as c_int,
            );

            #[cfg(not(target_os = "android"))]
            SDL_Flip(NE_SCREEN);

            need_update = false;
        }

        #[cfg(target_os = "android")]
        SDL_Flip(NE_SCREEN); // for responsive input on Android, we need to run this every cycle

        let action = get_menu_action(250);

        let time_for_move = SDL_GetTicks() - LAST_MOVE_TICK > wait_move_ticks;
        match action {
            MenuAction::BACK => {
                finished = true;
                wait_for_all_keys_released();
            }

            MenuAction::CLICK => {
                if handler.is_none() && submenu.is_null() {
                    menu_item_selected_sound();
                    finished = true;
                } else {
                    if let Some(handler) = handler {
                        wait_for_all_keys_released();
                        (handler)(action);
                    }

                    if submenu.is_null().not() {
                        menu_item_selected_sound();
                        wait_for_all_keys_released();
                        show_menu(submenu);
                        initiate_menu(false);
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

                move_menu_position_sound();
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

                move_menu_position_sound();
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

        if QUIT_MENU {
            finished = true;
        }

        SDL_Delay(1); // don't hog CPU
    }

    clear_graph_mem();
    SDL_ShowCursor(SDL_ENABLE); // reactivate mouse-cursor for game
                                // Since we've faded out the whole scren, it can't hurt
                                // to have the top status bar redrawn...
    BANNER_IS_DESTROYED = true.into();
    ME.status = Status::Mobile as i32;

    while any_key_is_pressed_r()
    // wait for all key/controller-release
    {
        SDL_Delay(1);
    }
}

/// subroutine to display the current key-config and highlight current selection
pub unsafe fn display_key_config(selx: c_int, sely: c_int) {
    let startx = i32::from(FULL_USER_RECT.x) + (1.2 * f32::from(BLOCK_RECT.w)) as i32;
    let starty = i32::from(FULL_USER_RECT.y) + font_height(&*get_current_font());
    let col1 = startx + (7.5 * f64::from(char_width(&*get_current_font(), b'O'))) as i32;
    let col2 = col1 + (6.5 * f64::from(char_width(&*get_current_font(), b'O'))) as i32;
    let col3 = col2 + (6.5 * f64::from(char_width(&*get_current_font(), b'O'))) as i32;
    let lheight = font_height(&*FONT0_B_FONT) + 2;

    SDL_UpperBlit(MENU_BACKGROUND, null_mut(), NE_SCREEN, null_mut());

    #[cfg(feature = "gcw0")]
    PrintStringFont(
        NE_SCREEN,
        Font0_BFont,
        col1,
        starty,
        format_args!("(RShldr to clear an entry)"),
    );

    #[cfg(not(feature = "gcw0"))]
    {
        print_string_font(
            NE_SCREEN,
            FONT0_B_FONT,
            col1,
            starty,
            format_args!("(RShldr to clear an entry)"),
        );
        print_string_font(
            NE_SCREEN,
            FONT0_B_FONT,
            col1,
            starty,
            format_args!("(Backspace to clear an entry)"),
        );
    }

    let mut posy = 1;
    print_string_font(
        NE_SCREEN,
        FONT0_B_FONT,
        startx,
        starty + (posy) * lheight,
        format_args!("Command"),
    );
    print_string_font(
        NE_SCREEN,
        FONT0_B_FONT,
        col1,
        starty + (posy) * lheight,
        format_args!("Key1"),
    );
    print_string_font(
        NE_SCREEN,
        FONT0_B_FONT,
        col2,
        starty + (posy) * lheight,
        format_args!("Key2"),
    );
    print_string_font(
        NE_SCREEN,
        FONT0_B_FONT,
        col3,
        starty + (posy) * lheight,
        format_args!("Key3"),
    );
    posy += 1;

    for i in 0..Cmds::Last as usize {
        let pos_font = |x, y| {
            if x != selx || i32::try_from(y).unwrap() != sely {
                FONT1_B_FONT
            } else {
                FONT2_B_FONT
            }
        };

        print_string_font(
            NE_SCREEN,
            FONT0_B_FONT,
            startx,
            starty + (posy) * lheight,
            format_args!("{}", CStr::from_ptr(CMD_STRINGS[i]).to_str().unwrap()),
        );
        print_string_font(
            NE_SCREEN,
            pos_font(1, 1 + i),
            col1,
            starty + (posy) * lheight,
            format_args!(
                "{}",
                CStr::from_ptr(KEYSTR[usize::try_from(KEY_CMDS[i][0]).unwrap()])
                    .to_str()
                    .unwrap()
            ),
        );
        print_string_font(
            NE_SCREEN,
            pos_font(2, 1 + i),
            col2,
            starty + (posy) * lheight,
            format_args!(
                "{}",
                CStr::from_ptr(KEYSTR[usize::try_from(KEY_CMDS[i][1]).unwrap()])
                    .to_str()
                    .unwrap()
            ),
        );
        print_string_font(
            NE_SCREEN,
            pos_font(3, 1 + i),
            col3,
            starty + (posy) * lheight,
            format_args!(
                "{}",
                CStr::from_ptr(KEYSTR[usize::try_from(KEY_CMDS[i][2]).unwrap()])
                    .to_str()
                    .unwrap()
            ),
        );
        posy += 1;
    }

    SDL_Flip(NE_SCREEN);
}

pub unsafe fn key_config_menu() {
    let mut selx = 1;
    let mut sely = 1; // currently selected menu-position
    const WAIT_MOVE_TICKS: u32 = 100;
    static mut LAST_MOVE_TICK: u32 = 0;

    let mut finished = false;
    while !finished {
        display_key_config(i32::try_from(selx).unwrap(), i32::try_from(sely).unwrap());

        let action = get_menu_action(250);
        let time_for_move = SDL_GetTicks() - LAST_MOVE_TICK > WAIT_MOVE_TICKS;

        match action {
            MenuAction::BACK => {
                finished = true;
                wait_for_all_keys_released();
            }

            MenuAction::CLICK => {
                menu_item_selected_sound();

                KEY_CMDS[sely - 1][selx - 1] = b'_'.into();
                display_key_config(i32::try_from(selx).unwrap(), i32::try_from(sely).unwrap());
                KEY_CMDS[sely - 1][selx - 1] = getchar_raw(); // includes joystick input!;
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
                move_menu_position_sound();
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
                move_menu_position_sound();
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
                move_menu_position_sound();
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
                move_menu_position_sound();
                LAST_MOVE_TICK = SDL_GetTicks();
            }

            MenuAction::DELETE => {
                KEY_CMDS[sely - 1][selx - 1] = 0;
                menu_item_selected_sound();
            }
            _ => {}
        }

        SDL_Delay(1);
    }
}

pub unsafe fn show_credits() {
    let col2 = 2 * i32::from(USER_RECT.w) / 3;

    let h = font_height(&*MENU_B_FONT);
    let em = char_width(&*MENU_B_FONT, b'm');

    let screen = SCREEN_RECT;
    SDL_SetClipRect(NE_SCREEN, null_mut());
    display_image(find_file(
        CREDITS_PIC_FILE_C.as_ptr() as *mut c_char,
        GRAPHICS_DIR_C.as_ptr() as *mut c_char,
        Themed::NoTheme as i32,
        Criticality::Critical as i32,
    ));
    make_grid_on_screen(Some(&screen));

    let oldfont = get_current_font();
    set_current_font(FONT1_B_FONT);

    printf_sdl(
        NE_SCREEN,
        i32::from(get_user_center().x) - 2 * em,
        h,
        format_args!("CREDITS\n"),
    );

    printf_sdl(NE_SCREEN, em, -1, format_args!("PROGRAMMING:"));
    printf_sdl(NE_SCREEN, col2, -1, format_args!("Johannes Prix\n"));
    printf_sdl(NE_SCREEN, -1, -1, format_args!("Reinhard Prix\n"));
    printf_sdl(NE_SCREEN, -1, -1, format_args!("\n"));

    printf_sdl(NE_SCREEN, em, -1, format_args!("ARTWORK:"));
    printf_sdl(NE_SCREEN, col2, -1, format_args!("Bastian Salmela\n"));
    printf_sdl(NE_SCREEN, -1, -1, format_args!("\n"));
    printf_sdl(NE_SCREEN, em, -1, format_args!("ADDITIONAL THEMES:\n"));
    printf_sdl(NE_SCREEN, 2 * em, -1, format_args!("Lanzz-theme"));
    printf_sdl(NE_SCREEN, col2, -1, format_args!("Lanzz\n"));
    printf_sdl(NE_SCREEN, 2 * em, -1, format_args!("Para90-theme"));
    printf_sdl(NE_SCREEN, col2, -1, format_args!("Andreas Wedemeyer\n"));

    printf_sdl(NE_SCREEN, -1, -1, format_args!("\n"));
    printf_sdl(NE_SCREEN, em, -1, format_args!("C64 LEGACY MODS:\n"));

    printf_sdl(
        NE_SCREEN,
        2 * em,
        -1,
        format_args!("Green Beret, Sanxion, Uridium2"),
    );
    printf_sdl(NE_SCREEN, col2, -1, format_args!("#dreamfish/trsi\n"));

    printf_sdl(NE_SCREEN, 2 * em, -1, format_args!("The last V8, Anarchy"));
    printf_sdl(NE_SCREEN, col2, -1, format_args!("4-mat\n"));

    printf_sdl(NE_SCREEN, 2 * em, -1, format_args!("Tron"));
    printf_sdl(NE_SCREEN, col2, -1, format_args!("Kollaps\n"));

    printf_sdl(NE_SCREEN, 2 * em, -1, format_args!("Starpaws"));
    printf_sdl(NE_SCREEN, col2, -1, format_args!("Nashua\n"));

    printf_sdl(NE_SCREEN, 2 * em, -1, format_args!("Commando"));
    printf_sdl(NE_SCREEN, col2, -1, format_args!("Android"));

    SDL_Flip(NE_SCREEN);
    wait_for_key_pressed();
    set_current_font(oldfont);
}

/// simple wrapper to ShowMenu() to provide the external entry point into the Level Editor menu
pub unsafe fn show_level_editor_menu() {
    QUIT_LEVEL_EDITOR = false;
    show_menu(LEVEL_EDITOR_MENU.as_ptr());
}

pub unsafe fn handle_configure_keys(action: MenuAction) -> *const c_char {
    if action == MenuAction::CLICK {
        menu_item_selected_sound();
        key_config_menu();
    }

    null_mut()
}

pub unsafe fn handle_highscores(action: MenuAction) -> *const c_char {
    if action == MenuAction::CLICK {
        menu_item_selected_sound();
        show_highscores();
    }
    null_mut()
}

pub unsafe fn handle_credits(action: MenuAction) -> *const c_char {
    if action == MenuAction::CLICK {
        menu_item_selected_sound();
        show_credits();
    }

    null_mut()
}

pub unsafe fn handle_le_save_ship(action: MenuAction) -> *const c_char {
    use std::io::Write;

    const SHIPNAME: &CStr = cstr!("Testship");
    static mut FNAME: [c_char; 255] = [0; 255];
    libc::snprintf(
        FNAME.as_mut_ptr(),
        FNAME.len() - 1,
        cstr!("%s%s").as_ptr() as *mut c_char,
        SHIPNAME.as_ptr() as *mut c_char,
        SHIP_EXT_C.as_ptr() as *mut c_char,
    );

    if action == MenuAction::INFO {
        return FNAME.as_ptr();
    }

    if action == MenuAction::CLICK {
        save_ship(SHIPNAME.as_ptr());
        let mut output = [0; 255];
        let mut cursor = Cursor::new(output.as_mut());
        write!(
            cursor,
            "Ship saved as '{}'",
            CStr::from_ptr(FNAME.as_ptr()).to_str().unwrap()
        )
        .unwrap();
        let position = usize::try_from(cursor.position()).unwrap();
        centered_put_string(
            NE_SCREEN,
            3 * font_height(&*MENU_B_FONT),
            &output[..position],
        );
        SDL_Flip(NE_SCREEN);
        wait_for_key_pressed();
        initiate_menu(false);
    }

    null_mut()
}

pub unsafe fn handle_le_name(action: MenuAction) -> *const c_char {
    let cur_level = &mut *CUR_LEVEL;
    if action == MenuAction::INFO {
        return cur_level.levelname;
    }

    if action == MenuAction::CLICK {
        display_text(
            cstr!("New level name: ").as_ptr() as *mut c_char,
            i32::from(MENU_RECT.x) - 2 * FONT_HEIGHT,
            i32::from(MENU_RECT.y) - 3 * FONT_HEIGHT,
            &FULL_USER_RECT,
        );
        SDL_Flip(NE_SCREEN);
        libc::free(cur_level.levelname as *mut c_void);
        cur_level.levelname = get_string(15, 2);
        initiate_menu(false);
    }

    null_mut()
}

pub unsafe fn handle_open_level_editor(action: MenuAction) -> *const c_char {
    if action == MenuAction::CLICK {
        menu_item_selected_sound();
        level_editor();
    }
    null_mut()
}

pub unsafe fn handle_le_exit(action: MenuAction) -> *const c_char {
    if action == MenuAction::CLICK {
        menu_item_selected_sound();
        QUIT_LEVEL_EDITOR = true;
        QUIT_MENU = true;
    }
    null_mut()
}

pub unsafe fn handle_le_level_number(action: MenuAction) -> *const c_char {
    static mut BUF: [c_char; 256] = [0; 256];
    let cur_level = &*CUR_LEVEL;
    if action == MenuAction::INFO {
        libc::sprintf(
            BUF.as_mut_ptr(),
            cstr!("%d").as_ptr() as *mut c_char,
            cur_level.levelnum,
        );
        return BUF.as_ptr();
    }

    let mut curlevel = cur_level.levelnum;
    menu_change_int(action, &mut curlevel, 1, 0, CUR_SHIP.num_levels - 1);
    teleport(curlevel, 3, 3);
    switch_background_music_to(BYCOLOR.as_ptr());
    initiate_menu(false);

    null_mut()
}

pub unsafe fn handle_le_color(action: MenuAction) -> *const c_char {
    let cur_level = &mut *CUR_LEVEL;
    if action == MenuAction::INFO {
        return COLOR_NAMES[usize::try_from(cur_level.color).unwrap()].as_ptr();
    }
    menu_change_int(
        action,
        &mut cur_level.color,
        1,
        0,
        c_int::try_from(COLOR_NAMES.len()).unwrap() - 1,
    );
    switch_background_music_to(BYCOLOR.as_ptr());
    initiate_menu(false);

    null_mut()
}

pub unsafe fn handle_le_size_x(action: MenuAction) -> *const c_char {
    static mut BUF: [c_char; 256] = [0; 256];
    let cur_level = &mut *CUR_LEVEL;
    if action == MenuAction::INFO {
        libc::sprintf(
            BUF.as_mut_ptr(),
            cstr!("%d").as_ptr() as *mut c_char,
            cur_level.xlen,
        );
        return BUF.as_ptr();
    }

    let oldxlen = cur_level.xlen;
    menu_change_int(
        action,
        &mut cur_level.xlen,
        1,
        0,
        i32::try_from(MAX_MAP_COLS).unwrap() - 1,
    );
    let newmem = usize::try_from(cur_level.xlen).unwrap();
    // adjust memory sizes for new value
    for row in 0..usize::try_from(cur_level.ylen).unwrap() {
        cur_level.map[row] = libc::realloc(cur_level.map[row] as *mut c_void, newmem) as *mut i8;
        if cur_level.map[row].is_null() {
            error!(
                "Failed to re-allocate to {} bytes in map row {}",
                newmem, row,
            );
            terminate(defs::ERR.into());
        }
        if cur_level.xlen > oldxlen {
            // fill new map area with VOID
            *cur_level.map[row].add(usize::try_from(cur_level.xlen - 1).unwrap()) =
                MapTile::Void as i8;
        }
    }
    initiate_menu(false);
    null_mut()
}

pub unsafe fn handle_le_size_y(action: MenuAction) -> *const c_char {
    use std::cmp::Ordering;

    static mut BUF: [c_char; 256] = [0; 256];
    let cur_level = &mut *CUR_LEVEL;
    if action == MenuAction::INFO {
        libc::sprintf(
            BUF.as_mut_ptr(),
            cstr!("%d").as_ptr() as *mut c_char,
            cur_level.ylen,
        );
        return BUF.as_ptr();
    }

    let oldylen = cur_level.ylen;
    menu_change_int(
        action,
        &mut cur_level.ylen,
        1,
        0,
        i32::try_from(MAX_MAP_ROWS - 1).unwrap(),
    );
    match oldylen.cmp(&cur_level.ylen) {
        Ordering::Greater => {
            libc::free(cur_level.map[usize::try_from(oldylen - 1).unwrap()] as *mut c_void);
            cur_level.map[usize::try_from(oldylen - 1).unwrap()] = null_mut();
        }
        Ordering::Less => {
            cur_level.map[usize::try_from(cur_level.ylen - 1).unwrap()] =
                my_malloc(cur_level.xlen.into()) as *mut i8;
            std::ptr::write_bytes(
                cur_level.map[usize::try_from(cur_level.ylen - 1).unwrap()],
                MapTile::Void as u8,
                usize::try_from(cur_level.xlen).unwrap(),
            )
        }
        Ordering::Equal => {}
    }

    initiate_menu(false);
    null_mut()
}

pub unsafe fn handle_strictly_classic(action: MenuAction) -> *const c_char {
    if action == MenuAction::CLICK {
        menu_item_selected_sound();
        GAME_CONFIG.droid_talk = false.into();
        GAME_CONFIG.show_decals = false.into();
        GAME_CONFIG.takeover_activates = true.into();
        GAME_CONFIG.fire_hold_takeover = true.into();
        GAME_CONFIG.all_map_visible = true.into();
        GAME_CONFIG.empty_level_speedup = 1.0;

        // set window type
        GAME_CONFIG.full_user_rect = false.into();
        USER_RECT = CLASSIC_USER_RECT;
        // set theme
        set_theme(CLASSIC_THEME_INDEX);
        initiate_menu(false);
    }

    null_mut()
}

pub unsafe fn handle_window_type(action: MenuAction) -> *const c_char {
    if action == MenuAction::INFO {
        return if GAME_CONFIG.full_user_rect != 0 {
            cstr!("Full").as_ptr()
        } else {
            cstr!("Classic").as_ptr()
        };
    }

    if action == MenuAction::CLICK || action == MenuAction::LEFT || action == MenuAction::RIGHT {
        flip_toggle(&mut GAME_CONFIG.full_user_rect);
        if GAME_CONFIG.full_user_rect != 0 {
            USER_RECT = FULL_USER_RECT;
        } else {
            USER_RECT = CLASSIC_USER_RECT;
        }

        initiate_menu(false);
    }
    null_mut()
}

pub unsafe fn handle_theme(action: MenuAction) -> *const c_char {
    if action == MenuAction::INFO {
        return ALL_THEMES.theme_name[usize::try_from(ALL_THEMES.cur_tnum).unwrap()]
            as *const c_char;
    }

    if action == MenuAction::CLICK || action == MenuAction::LEFT || action == MenuAction::RIGHT {
        move_lift_sound();
        let mut tnum = ALL_THEMES.cur_tnum;
        if action == MenuAction::CLICK && action == MenuAction::RIGHT {
            tnum += 1;
        } else {
            tnum -= 1;
        }

        if tnum < 0 {
            tnum = ALL_THEMES.num_themes - 1;
        }
        if tnum > ALL_THEMES.num_themes - 1 {
            tnum = 0;
        }

        set_theme(tnum);
        initiate_menu(false);
    }

    null_mut()
}

pub unsafe fn handle_droid_talk(action: MenuAction) -> *const c_char {
    if action == MenuAction::INFO {
        return is_toggle_on(GAME_CONFIG.droid_talk);
    }
    if action == MenuAction::CLICK || action == MenuAction::LEFT || action == MenuAction::RIGHT {
        flip_toggle(&mut GAME_CONFIG.droid_talk);
    }
    null_mut()
}

pub unsafe fn handle_all_map_visible(action: MenuAction) -> *const c_char {
    if action == MenuAction::INFO {
        return is_toggle_on(GAME_CONFIG.all_map_visible);
    }
    if action == MenuAction::CLICK || action == MenuAction::LEFT || action == MenuAction::RIGHT {
        flip_toggle(&mut GAME_CONFIG.all_map_visible);
        initiate_menu(false);
    }
    null_mut()
}

pub unsafe fn handle_show_decals(action: MenuAction) -> *const c_char {
    if action == MenuAction::INFO {
        return is_toggle_on(GAME_CONFIG.show_decals);
    }
    if action == MenuAction::CLICK || action == MenuAction::LEFT || action == MenuAction::RIGHT {
        flip_toggle(&mut GAME_CONFIG.show_decals);
        initiate_menu(false);
    }
    null_mut()
}

pub unsafe fn handle_transfer_is_activate(action: MenuAction) -> *const c_char {
    if action == MenuAction::INFO {
        return is_toggle_on(GAME_CONFIG.takeover_activates);
    }
    if action == MenuAction::CLICK || action == MenuAction::LEFT || action == MenuAction::RIGHT {
        flip_toggle(&mut GAME_CONFIG.takeover_activates);
    }
    null_mut()
}

pub unsafe fn handle_fire_is_transfer(action: MenuAction) -> *const c_char {
    if action == MenuAction::INFO {
        return is_toggle_on(GAME_CONFIG.fire_hold_takeover);
    }
    if action == MenuAction::CLICK || action == MenuAction::LEFT || action == MenuAction::RIGHT {
        flip_toggle(&mut GAME_CONFIG.fire_hold_takeover);
    }
    null_mut()
}

pub unsafe fn handle_empty_level_speedup(action: MenuAction) -> *const c_char {
    static mut BUF: [c_char; 256] = [0; 256];
    if action == MenuAction::INFO {
        libc::sprintf(
            BUF.as_mut_ptr(),
            cstr!("%3.1f").as_ptr() as *mut c_char,
            f64::from(GAME_CONFIG.empty_level_speedup),
        );
        return BUF.as_ptr();
    }

    menu_change_float(action, &mut GAME_CONFIG.empty_level_speedup, 0.1, 0.5, 2.0);
    null_mut()
}

pub unsafe fn handle_music_volume(action: MenuAction) -> *const c_char {
    static mut BUF: [c_char; 256] = [0; 256];
    if action == MenuAction::INFO {
        libc::sprintf(
            BUF.as_mut_ptr(),
            cstr!("%4.2f").as_ptr() as *mut c_char,
            f64::from(GAME_CONFIG.current_bg_music_volume),
        );
        return BUF.as_ptr();
    }

    menu_change_float(
        action,
        &mut GAME_CONFIG.current_bg_music_volume,
        0.05,
        0.,
        1.,
    );
    set_bg_music_volume(GAME_CONFIG.current_bg_music_volume);
    null_mut()
}

pub unsafe fn handle_sound_volume(action: MenuAction) -> *const c_char {
    static mut BUF: [c_char; 256] = [0; 256];
    if action == MenuAction::INFO {
        libc::sprintf(
            BUF.as_mut_ptr(),
            cstr!("%4.2f").as_ptr() as *mut c_char,
            f64::from(GAME_CONFIG.current_sound_fx_volume),
        );
        return BUF.as_ptr();
    }

    menu_change_float(
        action,
        &mut GAME_CONFIG.current_sound_fx_volume,
        0.05,
        0.,
        1.,
    );
    set_sound_f_x_volume(GAME_CONFIG.current_sound_fx_volume);
    null_mut()
}

pub unsafe fn handle_fullscreen(action: MenuAction) -> *const c_char {
    if action == MenuAction::INFO {
        return is_toggle_on(GAME_CONFIG.use_fullscreen);
    }
    if action == MenuAction::CLICK || action == MenuAction::LEFT || action == MenuAction::RIGHT {
        toggle_fullscreen();
        menu_item_selected_sound();
    }
    null_mut()
}

pub unsafe fn handle_show_position(action: MenuAction) -> *const c_char {
    if action == MenuAction::INFO {
        return is_toggle_on(GAME_CONFIG.draw_position);
    }
    if action == MenuAction::CLICK || action == MenuAction::LEFT || action == MenuAction::RIGHT {
        flip_toggle(&mut GAME_CONFIG.draw_position);
        initiate_menu(false);
    }
    null_mut()
}

pub unsafe fn handle_show_framerate(action: MenuAction) -> *const c_char {
    if action == MenuAction::INFO {
        return is_toggle_on(GAME_CONFIG.draw_framerate);
    }
    if action == MenuAction::CLICK || action == MenuAction::LEFT || action == MenuAction::RIGHT {
        flip_toggle(&mut GAME_CONFIG.draw_framerate);
        initiate_menu(false);
    }
    null_mut()
}

pub unsafe fn handle_show_energy(action: MenuAction) -> *const c_char {
    if action == MenuAction::INFO {
        return is_toggle_on(GAME_CONFIG.draw_energy);
    }
    if action == MenuAction::CLICK || action == MenuAction::LEFT || action == MenuAction::RIGHT {
        flip_toggle(&mut GAME_CONFIG.draw_energy);
        initiate_menu(false);
    }
    null_mut()
}

unsafe fn menu_change<T>(action: MenuAction, val: &mut T, step: T, min_value: T, max_value: T)
where
    T: PartialOrd + AddAssign + SubAssign,
{
    if action == MenuAction::RIGHT && *val < max_value {
        move_lift_sound();
        *val += step;
        if *val > max_value {
            *val = max_value;
        }
    } else if action == MenuAction::LEFT && *val > min_value {
        move_lift_sound();
        *val -= step;
        if *val <= min_value {
            *val = min_value;
        }
    }
}

pub unsafe fn menu_change_float(
    action: MenuAction,
    val: &mut c_float,
    step: c_float,
    min_value: c_float,
    max_value: c_float,
) {
    menu_change(action, val, step, min_value, max_value)
}

pub unsafe fn menu_change_int(
    action: MenuAction,
    val: &mut c_int,
    step: c_int,
    min_value: c_int,
    max_value: c_int,
) {
    menu_change(action, val, step, min_value, max_value)
}

pub fn is_toggle_on(toggle: c_int) -> *const c_char {
    if toggle != 0 {
        cstr!("YES").as_ptr()
    } else {
        cstr!("NO").as_ptr()
    }
}

pub unsafe fn flip_toggle(toggle: *mut c_int) {
    if toggle.is_null().not() {
        menu_item_selected_sound();
        *toggle = !*toggle;
    }
}

pub unsafe fn set_theme(theme_index: c_int) {
    assert!(theme_index >= 0 && theme_index < ALL_THEMES.num_themes);

    ALL_THEMES.cur_tnum = theme_index;
    libc::strcpy(
        GAME_CONFIG.theme_name.as_mut_ptr(),
        ALL_THEMES.theme_name[usize::try_from(ALL_THEMES.cur_tnum).unwrap()] as *const c_char,
    );
    init_pictures();
}
