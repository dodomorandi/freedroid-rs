#[cfg(feature = "gcw0")]
use crate::{
    defs::{gcw0_a_pressed, gcw0_any_button_pressed, gcw0_any_button_pressed_r},
    input::SDL_Delay,
};

use crate::{
    b_font::{char_width, font_height, print_string_font},
    defs::{
        AssembleCombatWindowFlags, Cmds, Criticality, DisplayBannerFlags, Droid, MapTile,
        MenuAction, Status, Themed, BYCOLOR, CREDITS_PIC_FILE_C, GRAPHICS_DIR_C, MAX_MAP_COLS,
        MAX_MAP_ROWS,
    },
    global::INFLUENCE_MODE_NAMES,
    input::{SDL_Delay, CMD_STRINGS},
    map::COLOR_NAMES,
    misc::dealloc_c_string,
    Data,
};

use cstr::cstr;
use sdl::{
    keysym::{SDLK_BACKSPACE, SDLK_DOWN, SDLK_ESCAPE, SDLK_LEFT, SDLK_RIGHT, SDLK_UP},
    mouse::ll::{SDL_ShowCursor, SDL_DISABLE, SDL_ENABLE},
    sdl::ll::SDL_GetTicks,
    video::ll::{
        SDL_DisplayFormat, SDL_Flip, SDL_FreeSurface, SDL_SetClipRect, SDL_Surface, SDL_UpperBlit,
    },
};
use std::{
    alloc::{alloc_zeroed, dealloc, realloc, Layout},
    convert::{TryFrom, TryInto},
    ffi::CStr,
    io::Cursor,
    ops::{AddAssign, Not, SubAssign},
    os::raw::{c_char, c_float, c_int},
    ptr::null_mut,
    sync::atomic::AtomicBool,
};

#[derive(Debug)]
pub struct Menu {
    font_height: i32,
    menu_background: *mut SDL_Surface,
    quit_menu: bool,
    pub quit_level_editor: bool,
    last_movekey_time: u32,
    menu_action_directions: MenuActionDirections,
    show_menu_last_move_tick: u32,
    key_config_menu_last_move_tick: u32,
    fname: [c_char; 255],
    le_level_number_buf: [c_char; 256],
    le_size_x_buf: [c_char; 256],
    le_size_y_buf: [c_char; 256],
    empty_level_speedup_buf: [c_char; 256],
    music_volume_buf: [c_char; 256],
    sound_volume_buf: [c_char; 256],
}

#[derive(Debug, Default)]
struct MenuActionDirections {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

impl Default for Menu {
    fn default() -> Self {
        Self {
            font_height: 0,
            menu_background: null_mut(),
            quit_menu: false,
            quit_level_editor: false,
            last_movekey_time: 0,
            menu_action_directions: Default::default(),
            show_menu_last_move_tick: 0,
            key_config_menu_last_move_tick: 0,
            fname: [0; 255],
            le_level_number_buf: [0; 256],
            le_size_x_buf: [0; 256],
            le_size_y_buf: [0; 256],
            empty_level_speedup_buf: [0; 256],
            music_volume_buf: [0; 256],
            sound_volume_buf: [0; 256],
        }
    }
}

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
    menu_entry! { "Set Strictly Classic", Data::handle_strictly_classic},
    menu_entry! { "Combat Window: ", Data::handle_window_type},
    menu_entry! { "Graphics Theme: ", Data::handle_theme},
    menu_entry! { "Droid Talk: ", Data::handle_droid_talk},
    menu_entry! { "Show Decals: ", Data::handle_show_decals},
    menu_entry! { "All Map Visible: ", Data::handle_all_map_visible},
    menu_entry! { "Empty Level Speedup: ", Data::handle_empty_level_speedup},
    menu_entry! {},
];

#[cfg(not(target_os = "android"))]
const LEGACY_MENU: [MenuEntry; 11] = [
    menu_entry! { "Back"},
    menu_entry! { "Set Strictly Classic", Data::handle_strictly_classic},
    menu_entry! { "Combat Window: ", Data::handle_window_type},
    menu_entry! { "Graphics Theme: ", Data::handle_theme},
    menu_entry! { "Droid Talk: ", Data::handle_droid_talk},
    menu_entry! { "Show Decals: ", Data::handle_show_decals},
    menu_entry! { "All Map Visible: ", Data::handle_all_map_visible},
    menu_entry! { "Transfer = Activate: ", Data::handle_transfer_is_activate},
    menu_entry! { "Hold Fire to Transfer: ", Data::handle_fire_is_transfer},
    menu_entry! { "Empty Level Speedup: ", Data::handle_empty_level_speedup},
    menu_entry! {},
];

const GRAPHICS_SOUND_MENU: [MenuEntry; 5] = [
    menu_entry! { "Back"},
    menu_entry! { "Music Volume: ", Data::handle_music_volume},
    menu_entry! { "Sound Volume: ", Data::handle_sound_volume},
    menu_entry! { "Fullscreen Mode: ", Data::handle_fullscreen},
    menu_entry! {},
];

const HUD_MENU: [MenuEntry; 5] = [
    menu_entry! { "Back"},
    menu_entry! { "Show Position: ", Data::handle_show_position},
    menu_entry! { "Show Framerate: ", Data::handle_show_framerate},
    menu_entry! { "Show Energy: ", Data::handle_show_energy},
    menu_entry! {},
];

const LEVEL_EDITOR_MENU: [MenuEntry; 8] = [
    menu_entry! { "Exit Level Editor", 	Data::handle_le_exit},
    menu_entry! { "Current Level: ", Data::handle_le_level_number},
    menu_entry! { "Level Color: ", Data::handle_le_color},
    menu_entry! { "Levelsize X: ", Data::handle_le_size_x},
    menu_entry! { "Levelsize Y: ", Data::handle_le_size_y},
    menu_entry! { "Level Name: ", Data::handle_le_name},
    menu_entry! { "Save ship: ", Data::handle_le_save_ship},
    menu_entry! {},
];

#[cfg(target_os = "android")]
const MAIN_MENU: [MenuEntry; 8] = [
    menu_entry! { "Back to Game"},
    menu_entry! { "Graphics & Sound", None, GRAPHICS_SOUND_MENU },
    menu_entry! { "Legacy Options", None, LEGACY_MENU },
    menu_entry! { "HUD Settings", None, HUD_MENU },
    menu_entry! { "Highscores", Data::handle_highscores},
    menu_entry! { "Credits", Data::handle_credits},
    menu_entry! { "Quit Game", Data::handle_quit_game},
    menu_entry! {},
];

#[cfg(not(target_os = "android"))]
const MAIN_MENU: [MenuEntry; 10] = [
    menu_entry! { "Back to Game"},
    menu_entry! { "Graphics & Sound", None, GRAPHICS_SOUND_MENU },
    menu_entry! { "Legacy Options", None, LEGACY_MENU },
    menu_entry! { "HUD Settings", None, HUD_MENU },
    menu_entry! { "Level Editor", Data::handle_open_level_editor},
    menu_entry! { "Highscores", Data::handle_highscores},
    menu_entry! { "Credits", Data::handle_credits},
    menu_entry! { "Configure Keys", Data::handle_configure_keys},
    menu_entry! { "Quit Game", Data::handle_quit_game},
    menu_entry! {},
];

pub struct MenuEntry {
    name: *const c_char,
    handler: Option<unsafe fn(&mut Data, MenuAction) -> *const c_char>,
    submenu: *const MenuEntry,
}

impl Data {
    pub unsafe fn handle_quit_game(&mut self, action: MenuAction) -> *const c_char {
        if action != MenuAction::CLICK {
            return null_mut();
        }

        self.menu_item_selected_sound();
        self.initiate_menu(false);

        #[cfg(feature = "gcw0")]
        const QUIT_STRING: &CStr = cstr!("Press A to quit");

        #[cfg(not(feature = "gcw0"))]
        const QUIT_STRING: &[u8] = b"Hit 'y' or press Fire to quit";

        let text_width = self.text_width(QUIT_STRING);
        let text_x =
            i32::from(self.vars.user_rect.x) + (i32::from(self.vars.user_rect.w) - text_width) / 2;
        let text_y = i32::from(self.vars.user_rect.y)
            + (i32::from(self.vars.user_rect.h) - self.menu.font_height) / 2;
        self.put_string(self.graphics.ne_screen, text_x, text_y, QUIT_STRING);
        SDL_Flip(self.graphics.ne_screen);

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
            self.wait_for_all_keys_released();
            let key = self.wait_for_key_pressed();
            if key == b'y'.into()
                || key == self.input.key_cmds[Cmds::Fire as usize][0]
                || key == self.input.key_cmds[Cmds::Fire as usize][1]
                || key == self.input.key_cmds[Cmds::Fire as usize][2]
            {
                self.quit_successfully();
            }
        }

        null_mut()
    }

    /// simple wrapper to ShowMenu() to provide the external entry point into the main menu
    pub unsafe fn show_main_menu(&mut self) {
        self.show_menu(MAIN_MENU.as_ptr());
    }

    pub unsafe fn free_menu_data(&self) {
        SDL_FreeSurface(self.menu.menu_background);
    }

    pub unsafe fn initiate_menu(&mut self, with_droids: bool) {
        // Here comes the standard initializer for all the menus and submenus
        // of the big escape menu.  This prepares the screen, so that we can
        // write on it further down.
        self.activate_conservative_frame_computation();

        SDL_SetClipRect(self.graphics.ne_screen, null_mut());
        self.vars.me.status = Status::Menu as i32;
        self.clear_graph_mem();
        self.display_banner(
            null_mut(),
            null_mut(),
            (DisplayBannerFlags::NO_SDL_UPDATE | DisplayBannerFlags::FORCE_UPDATE)
                .bits()
                .into(),
        );
        if with_droids {
            self.assemble_combat_picture(0);
        } else {
            self.assemble_combat_picture(AssembleCombatWindowFlags::ONLY_SHOW_MAP.bits().into());
        }

        SDL_SetClipRect(self.graphics.ne_screen, null_mut());
        self.make_grid_on_screen(None);

        if !self.menu.menu_background.is_null() {
            SDL_FreeSurface(self.menu.menu_background);
        }
        self.menu.menu_background = SDL_DisplayFormat(self.graphics.ne_screen); // keep a global copy of background

        SDL_ShowCursor(SDL_DISABLE); // deactivate mouse-cursor in menus
        self.b_font.current_font = self.global.menu_b_font;
        self.menu.font_height = font_height(&*self.b_font.current_font) + 2;
    }

    pub unsafe fn cheatmenu(&mut self) {
        // Prevent distortion of framerate by the delay coming from
        // the time spend in the menu.
        self.activate_conservative_frame_computation();

        let font = self.global.font0_b_font;

        self.b_font.current_font = font; /* not the ideal one, but there's currently */
        /* no other it seems.. */
        const X0: i32 = 50;
        const Y0: i32 = 20;

        let cur_level = &mut *self.main.cur_level;
        let droid_map = std::slice::from_raw_parts(self.vars.droidmap, Droid::NumDroids as usize);
        let mut resume = false;
        while !resume {
            self.clear_graph_mem();
            self.printf_sdl(
                self.graphics.ne_screen,
                X0,
                Y0,
                format_args!(
                    "Current position: Level={}, X={:.0}, Y={:.0}\n",
                    cur_level.levelnum,
                    self.vars.me.pos.x.clone(),
                    self.vars.me.pos.y.clone(),
                ),
            );
            self.printf_sdl(
                self.graphics.ne_screen,
                -1,
                -1,
                format_args!(" a. Armageddon (alle Robots sprengen)\n"),
            );
            self.printf_sdl(
                self.graphics.ne_screen,
                -1,
                -1,
                format_args!(" l. robot list of current level\n"),
            );
            self.printf_sdl(
                self.graphics.ne_screen,
                -1,
                -1,
                format_args!(" g. complete robot list\n"),
            );
            self.printf_sdl(
                self.graphics.ne_screen,
                -1,
                -1,
                format_args!(" d. destroy robots on current level\n"),
            );
            self.printf_sdl(
                self.graphics.ne_screen,
                -1,
                -1,
                format_args!(" t. Teleportation\n"),
            );
            self.printf_sdl(
                self.graphics.ne_screen,
                -1,
                -1,
                format_args!(" r. change to new robot type\n"),
            );
            self.printf_sdl(
                self.graphics.ne_screen,
                -1,
                -1,
                format_args!(
                    " i. Invinciblemode: {}\n",
                    if self.main.invincible_mode != 0 {
                        "ON"
                    } else {
                        "OFF"
                    },
                ),
            );
            self.printf_sdl(
                self.graphics.ne_screen,
                -1,
                -1,
                format_args!(" e. set energy\n"),
            );
            self.printf_sdl(
                self.graphics.ne_screen,
                -1,
                -1,
                format_args!(
                    " n. No hidden droids: {}\n",
                    if self.main.show_all_droids != 0 {
                        "ON"
                    } else {
                        "OFF"
                    },
                ),
            );
            self.printf_sdl(
                self.graphics.ne_screen,
                -1,
                -1,
                format_args!(" m. Map of Deck xy\n"),
            );
            self.printf_sdl(
                self.graphics.ne_screen,
                -1,
                -1,
                format_args!(
                    " s. Sound: {}\n",
                    if self.main.sound_on != 0 { "ON" } else { "OFF" }
                ),
            );
            self.printf_sdl(
                self.graphics.ne_screen,
                -1,
                -1,
                format_args!(" w. Print current waypoints\n"),
            );
            self.printf_sdl(
                self.graphics.ne_screen,
                -1,
                -1,
                format_args!(" z. change Zoom factor\n"),
            );
            self.printf_sdl(
                self.graphics.ne_screen,
                -1,
                -1,
                format_args!(
                    " f. Freeze on this positon: {}\n",
                    if self.main.stop_influencer != 0 {
                        "ON"
                    } else {
                        "OFF"
                    },
                ),
            );
            self.printf_sdl(
                self.graphics.ne_screen,
                -1,
                -1,
                format_args!(" q. RESUME game\n"),
            );

            match u8::try_from(self.getchar_raw()).ok() {
                Some(b'f') => {
                    self.main.stop_influencer = !self.main.stop_influencer;
                }

                Some(b'z') => {
                    self.clear_graph_mem();
                    self.printf_sdl(
                        self.graphics.ne_screen,
                        X0,
                        Y0,
                        format_args!(
                            "Current Zoom factor: {}\n",
                            self.global.current_combat_scale_factor.clone(),
                        ),
                    );
                    self.printf_sdl(
                        self.graphics.ne_screen,
                        -1,
                        -1,
                        format_args!("New zoom factor: "),
                    );
                    let input = self.get_string(40, 2);
                    libc::sscanf(
                        input,
                        cstr!("%f").as_ptr() as *mut c_char,
                        &mut self.global.current_combat_scale_factor,
                    );
                    drop(Vec::from_raw_parts(input as *mut i8, 45, 45));
                    self.set_combat_scale_to(self.global.current_combat_scale_factor);
                }

                Some(b'a') => {
                    /* armageddon */
                    resume = true;
                    self.armageddon();
                }

                Some(b'l') => {
                    /* robot list of this deck */
                    let mut l = 0; /* line counter for enemy output */
                    for i in 0..usize::try_from(self.main.num_enemys).unwrap() {
                        if self.main.all_enemys[i].levelnum == cur_level.levelnum {
                            if l != 0 && l % 20 == 0 {
                                self.printf_sdl(
                                    self.graphics.ne_screen,
                                    -1,
                                    -1,
                                    format_args!(" --- MORE --- \n"),
                                );
                                if self.getchar_raw() == b'q'.into() {
                                    break;
                                }
                            }
                            if l % 20 == 0 {
                                self.clear_graph_mem();
                                self.printf_sdl(
                                    self.graphics.ne_screen,
                                    X0,
                                    Y0,
                                    format_args!(" NR.   ID  X    Y   ENERGY   Status\n"),
                                );
                                self.printf_sdl(
                                    self.graphics.ne_screen,
                                    -1,
                                    -1,
                                    format_args!("---------------------------------------------\n"),
                                );
                            }

                            l += 1;
                            let status = if self.main.all_enemys[i].status == Status::Out as i32 {
                                "OUT"
                            } else if self.main.all_enemys[i].status == Status::Terminated as i32 {
                                "DEAD"
                            } else {
                                "ACTIVE"
                            };

                            self.printf_sdl(
                                self.graphics.ne_screen,
                                -1,
                                -1,
                                format_args!(
                                    "{}.   {}   {:.0}   {:.0}   {:.0}    {}.\n",
                                    i,
                                    CStr::from_ptr(
                                        droid_map
                                            [usize::try_from(self.main.all_enemys[i].ty).unwrap()]
                                        .druidname
                                        .as_ptr()
                                    )
                                    .to_str()
                                    .unwrap(),
                                    self.main.all_enemys[i].pos.x.clone(),
                                    self.main.all_enemys[i].pos.y.clone(),
                                    self.main.all_enemys[i].energy.clone(),
                                    status,
                                ),
                            );
                        }
                    }

                    self.printf_sdl(
                        self.graphics.ne_screen,
                        -1,
                        -1,
                        format_args!(" --- END --- \n"),
                    );
                    self.getchar_raw();
                }

                Some(b'g') => {
                    /* complete robot list of this ship */
                    for i in 0..usize::try_from(self.main.num_enemys).unwrap() {
                        if self.main.all_enemys[i].ty == -1 {
                            continue;
                        }

                        if i != 0 && !i % 13 == 0 {
                            self.printf_sdl(
                                self.graphics.ne_screen,
                                -1,
                                -1,
                                format_args!(" --- MORE --- ('q' to quit)\n"),
                            );
                            if self.getchar_raw() == b'q'.into() {
                                break;
                            }
                        }
                        if i % 13 == 0 {
                            self.clear_graph_mem();
                            self.printf_sdl(
                                self.graphics.ne_screen,
                                X0,
                                Y0,
                                format_args!("Nr.  Lev. ID  Energy  Status.\n"),
                            );
                            self.printf_sdl(
                                self.graphics.ne_screen,
                                -1,
                                -1,
                                format_args!("------------------------------\n"),
                            );
                        }

                        self.printf_sdl(
                            self.graphics.ne_screen,
                            -1,
                            -1,
                            format_args!(
                                "{}  {}  {}  {:.0}  {}\n",
                                i,
                                self.main.all_enemys[i].levelnum.clone(),
                                CStr::from_ptr(
                                    droid_map[usize::try_from(self.main.all_enemys[i].ty).unwrap()]
                                        .druidname
                                        .as_ptr()
                                )
                                .to_str()
                                .unwrap(),
                                self.main.all_enemys[i].energy.clone(),
                                INFLUENCE_MODE_NAMES
                                    [usize::try_from(self.main.all_enemys[i].status).unwrap()]
                                .to_str()
                                .unwrap(),
                            ),
                        );
                    }

                    self.printf_sdl(
                        self.graphics.ne_screen,
                        -1,
                        -1,
                        format_args!(" --- END ---\n"),
                    );
                    self.getchar_raw();
                }

                Some(b'd') => {
                    /* destroy all robots on this level, haha */
                    for enemy in &mut self.main.all_enemys {
                        if enemy.levelnum == cur_level.levelnum {
                            enemy.energy = -100.;
                        }
                    }
                    self.printf_sdl(
                        self.graphics.ne_screen,
                        -1,
                        -1,
                        format_args!("All robots on this deck killed!\n"),
                    );
                    self.getchar_raw();
                }

                Some(b't') => {
                    /* Teleportation */
                    self.clear_graph_mem();
                    self.printf_sdl(
                        self.graphics.ne_screen,
                        X0,
                        Y0,
                        format_args!("Enter Level, X, Y: "),
                    );
                    let input = self.get_string(40, 2);
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
                    drop(Vec::from_raw_parts(input as *mut i8, 45, 45));
                    self.teleport(l_num, x, y);
                }

                Some(b'r') => {
                    /* change to new robot type */
                    self.clear_graph_mem();
                    self.printf_sdl(
                        self.graphics.ne_screen,
                        X0,
                        Y0,
                        format_args!("Type number of new robot: "),
                    );
                    let input = self.get_string(40, 2);
                    let mut i = 0;
                    for _ in 0..u32::try_from(self.main.number_of_droid_types).unwrap() {
                        if libc::strcmp(droid_map[i].druidname.as_ptr(), input) != 0 {
                            break;
                        }
                        i += 1;
                    }

                    if i == usize::try_from(self.main.number_of_droid_types).unwrap() {
                        self.printf_sdl(
                            self.graphics.ne_screen,
                            X0,
                            Y0 + 20,
                            format_args!(
                                "Unrecognized robot-type: {}",
                                CStr::from_ptr(input).to_str().unwrap(),
                            ),
                        );
                        self.getchar_raw();
                        self.clear_graph_mem();
                    } else {
                        self.vars.me.ty = i.try_into().unwrap();
                        self.vars.me.energy =
                            droid_map[usize::try_from(self.vars.me.ty).unwrap()].maxenergy;
                        self.vars.me.health = self.vars.me.energy;
                        self.printf_sdl(
                            self.graphics.ne_screen,
                            X0,
                            Y0 + 20,
                            format_args!(
                                "You are now a {}. Have fun!\n",
                                CStr::from_ptr(input).to_str().unwrap(),
                            ),
                        );
                        self.getchar_raw();
                    }
                    drop(Vec::from_raw_parts(input as *mut i8, 45, 45));
                }

                Some(b'i') => {
                    /* togge Invincible mode */
                    self.main.invincible_mode = !self.main.invincible_mode;
                }

                Some(b'e') => {
                    /* complete heal */
                    self.clear_graph_mem();
                    self.printf_sdl(
                        self.graphics.ne_screen,
                        X0,
                        Y0,
                        format_args!("Current energy: {}\n", self.vars.me.energy.clone()),
                    );
                    self.printf_sdl(
                        self.graphics.ne_screen,
                        -1,
                        -1,
                        format_args!("Enter your new energy: "),
                    );
                    let input = self.get_string(40, 2);
                    let mut num = 0;
                    libc::sscanf(input, cstr!("%d").as_ptr() as *mut c_char, &mut num);
                    drop(Vec::from_raw_parts(input as *mut i8, 45, 45));
                    self.vars.me.energy = num as f32;
                    if self.vars.me.energy > self.vars.me.health {
                        self.vars.me.health = self.vars.me.energy;
                    }
                }

                Some(b'n') => {
                    /* toggle display of all droids */
                    self.main.show_all_droids = !self.main.show_all_droids;
                }

                Some(b's') => {
                    /* toggle sound on/off */
                    self.main.sound_on = !self.main.sound_on;
                }

                Some(b'm') => {
                    /* Show deck map in Concept view */
                    self.printf_sdl(
                        self.graphics.ne_screen,
                        -1,
                        -1,
                        format_args!("\nLevelnum: "),
                    );
                    let input = self.get_string(40, 2);
                    let mut l_num = 0;
                    libc::sscanf(input, cstr!("%d").as_ptr() as *mut c_char, &mut l_num);
                    drop(Vec::from_raw_parts(input as *mut i8, 45, 45));
                    self.show_deck_map();
                    self.getchar_raw();
                }

                Some(b'w') => {
                    /* print waypoint info of current level */
                    for (i, waypoint) in cur_level.all_waypoints.iter_mut().enumerate() {
                        if i != 0 && i % 20 == 0 {
                            self.printf_sdl(
                                self.graphics.ne_screen,
                                -1,
                                -1,
                                format_args!(" ---- MORE -----\n"),
                            );
                            if self.getchar_raw() == b'q'.into() {
                                break;
                            }
                        }
                        if i % 20 == 0 {
                            self.clear_graph_mem();
                            self.printf_sdl(
                                self.graphics.ne_screen,
                                X0,
                                Y0,
                                format_args!("Nr.   X   Y      C1  C2  C3  C4\n"),
                            );
                            self.printf_sdl(
                                self.graphics.ne_screen,
                                -1,
                                -1,
                                format_args!("------------------------------------\n"),
                            );
                        }
                        self.printf_sdl(
                            self.graphics.ne_screen,
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
                    self.printf_sdl(
                        self.graphics.ne_screen,
                        -1,
                        -1,
                        format_args!(" --- END ---\n"),
                    );
                    self.getchar_raw();
                }

                Some(b' ') | Some(b'q') => {
                    resume = true;
                }

                _ => {}
            }
        }

        self.clear_graph_mem();

        self.update_input(); /* treat all pending keyboard events */
    }

    /// get menu input actions
    ///
    /// NOTE: built-in time delay to ensure spurious key-repetitions
    /// such as from touchpad 'wheel' or android joystic emulation
    /// don't create unexpected menu movements:
    /// ==> ignore all movement commands withing delay_ms milliseconds of each other
    pub unsafe fn get_menu_action(&mut self, wait_repeat_ticks: u32) -> MenuAction {
        let mut action = MenuAction::empty();

        // 'normal' menu action keys get released
        if self.key_is_pressed_r(SDLK_BACKSPACE as c_int) {
            {
                action = MenuAction::DELETE;
            }
        }
        if self.cmd_is_active_r(Cmds::Back) || self.key_is_pressed_r(SDLK_ESCAPE as c_int) {
            {
                action = MenuAction::BACK;
            }
        }

        if self.fire_pressed() || self.return_pressed_r() {
            {
                action = MenuAction::CLICK;
            }
        }

        // we register if there have been key-press events in the "waiting period" between move-ticks
        if !self.menu.menu_action_directions.up
            && (self.up_pressed() || self.key_is_pressed(SDLK_UP as c_int))
        {
            self.menu.menu_action_directions.up = true;
            self.menu.last_movekey_time = SDL_GetTicks();
            action |= MenuAction::UP;
        }
        if !self.menu.menu_action_directions.down
            && (self.down_pressed() || self.key_is_pressed(SDLK_DOWN as c_int))
        {
            self.menu.menu_action_directions.down = true;
            self.menu.last_movekey_time = SDL_GetTicks();
            action |= MenuAction::DOWN;
        }
        if !self.menu.menu_action_directions.left
            && (self.left_pressed() || self.key_is_pressed(SDLK_LEFT as c_int))
        {
            self.menu.menu_action_directions.left = true;
            self.menu.last_movekey_time = SDL_GetTicks();
            action |= MenuAction::LEFT;
        }
        if !self.menu.menu_action_directions.right
            && (self.right_pressed() || self.key_is_pressed(SDLK_RIGHT as c_int))
        {
            self.menu.menu_action_directions.right = true;
            self.menu.last_movekey_time = SDL_GetTicks();
            action |= MenuAction::RIGHT;
        }

        if !(self.up_pressed() || self.key_is_pressed(SDLK_UP as c_int)) {
            self.menu.menu_action_directions.up = false;
        }
        if !(self.down_pressed() || self.key_is_pressed(SDLK_DOWN as c_int)) {
            self.menu.menu_action_directions.down = false;
        }
        if !(self.left_pressed() || self.key_is_pressed(SDLK_LEFT as c_int)) {
            self.menu.menu_action_directions.left = false;
        }
        if !(self.right_pressed() || self.key_is_pressed(SDLK_RIGHT as c_int)) {
            self.menu.menu_action_directions.right = false;
        }

        // check if enough time since we registered last new move-action
        if SDL_GetTicks() - self.menu.last_movekey_time > wait_repeat_ticks {
            if self.menu.menu_action_directions.up {
                action |= MenuAction::UP;
            }
            if self.menu.menu_action_directions.down {
                action |= MenuAction::DOWN;
            }
            if self.menu.menu_action_directions.left {
                action |= MenuAction::LEFT;
            }
            if self.menu.menu_action_directions.right {
                action |= MenuAction::RIGHT;
            }
        }
        // special handling of mouse wheel: register every event, no need for key-repeat delays
        if self.wheel_up_pressed() {
            action |= MenuAction::UP_WHEEL;
        }
        if self.wheel_down_pressed() {
            action |= MenuAction::DOWN_WHEEL;
        }

        action
    }

    /// Generic menu handler
    pub unsafe fn show_menu(&mut self, menu_entries: *const MenuEntry) {
        use std::io::Write;

        self.initiate_menu(false);
        self.wait_for_all_keys_released();

        // figure out menu-start point to make it centered
        let mut num_entries = 0;
        let mut menu_width = None::<i32>;
        loop {
            let entry = &*menu_entries.add(num_entries);
            if entry.name.is_null() {
                break;
            }

            let width = self.text_width(CStr::from_ptr(entry.name).to_bytes());
            menu_width = Some(
                menu_width
                    .map(|menu_width| menu_width.max(width))
                    .unwrap_or(width),
            );

            num_entries += 1;
        }
        let menu_entries = std::slice::from_raw_parts(menu_entries, num_entries);
        let menu_width = menu_width.unwrap();

        let menu_height = i32::try_from(num_entries).unwrap() * self.menu.font_height;
        let menu_x = i32::from(self.vars.full_user_rect.x)
            + (i32::from(self.vars.full_user_rect.w) - menu_width) / 2;
        let menu_y = i32::from(self.vars.full_user_rect.y)
            + (i32::from(self.vars.full_user_rect.h) - menu_height) / 2;
        let influ_x = menu_x - i32::from(self.vars.block_rect.w) - self.menu.font_height;

        let mut menu_pos = 0;

        let wait_move_ticks: u32 = 100;
        let mut finished = false;
        self.menu.quit_menu = false;
        let mut need_update = true;
        while !finished {
            let handler = menu_entries[menu_pos].handler;
            let submenu = menu_entries[menu_pos].submenu;

            if need_update {
                SDL_UpperBlit(
                    self.menu.menu_background,
                    null_mut(),
                    self.graphics.ne_screen,
                    null_mut(),
                );
                // print menu
                menu_entries.iter().enumerate().for_each(|(i, entry)| {
                    let arg = entry
                        .handler
                        .map(|handler| (handler)(self, MenuAction::INFO))
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
                    self.put_string(
                        self.graphics.ne_screen,
                        menu_x,
                        menu_y + i32::try_from(i).unwrap() * self.menu.font_height,
                        &full_name[..position],
                    );
                });
                self.put_influence(
                    influ_x,
                    menu_y + ((menu_pos as f64 - 0.5) * f64::from(self.menu.font_height)) as c_int,
                );

                #[cfg(not(target_os = "android"))]
                SDL_Flip(self.graphics.ne_screen);

                need_update = false;
            }

            #[cfg(target_os = "android")]
            SDL_Flip(self.graphics.ne_screen); // for responsive input on Android, we need to run this every cycle

            let action = self.get_menu_action(250);

            let time_for_move =
                SDL_GetTicks() - self.menu.show_menu_last_move_tick > wait_move_ticks;
            match action {
                MenuAction::BACK => {
                    finished = true;
                    self.wait_for_all_keys_released();
                }

                MenuAction::CLICK => {
                    if handler.is_none() && submenu.is_null() {
                        self.menu_item_selected_sound();
                        finished = true;
                    } else {
                        if let Some(handler) = handler {
                            self.wait_for_all_keys_released();
                            (handler)(self, action);
                        }

                        if submenu.is_null().not() {
                            self.menu_item_selected_sound();
                            self.wait_for_all_keys_released();
                            self.show_menu(submenu);
                            self.initiate_menu(false);
                        }
                        need_update = true;
                    }
                }

                MenuAction::RIGHT | MenuAction::LEFT => {
                    if !time_for_move {
                        continue;
                    }

                    if let Some(handler) = handler {
                        (handler)(self, action);
                    }
                    self.menu.show_menu_last_move_tick = SDL_GetTicks();
                    need_update = true;
                }

                MenuAction::UP | MenuAction::UP_WHEEL => {
                    if action == MenuAction::UP && !time_for_move {
                        continue;
                    }

                    self.move_menu_position_sound();
                    if menu_pos > 0 {
                        menu_pos -= 1;
                    } else {
                        menu_pos = num_entries - 1;
                    }
                    self.menu.show_menu_last_move_tick = SDL_GetTicks();
                    need_update = true;
                }

                MenuAction::DOWN | MenuAction::DOWN_WHEEL => {
                    if action == MenuAction::DOWN && !time_for_move {
                        continue;
                    }

                    self.move_menu_position_sound();
                    if menu_pos < num_entries - 1 {
                        menu_pos += 1;
                    } else {
                        menu_pos = 0;
                    }
                    self.menu.show_menu_last_move_tick = SDL_GetTicks();
                    need_update = true;
                }

                _ => {}
            }

            if self.menu.quit_menu {
                finished = true;
            }

            SDL_Delay(1); // don't hog CPU
        }

        self.clear_graph_mem();
        SDL_ShowCursor(SDL_ENABLE); // reactivate mouse-cursor for game
                                    // Since we've faded out the whole scren, it can't hurt
                                    // to have the top status bar redrawn...
        self.graphics.banner_is_destroyed = true.into();
        self.vars.me.status = Status::Mobile as i32;

        while self.any_key_is_pressed_r()
        // wait for all key/controller-release
        {
            SDL_Delay(1);
        }
    }

    /// subroutine to display the current key-config and highlight current selection
    pub unsafe fn display_key_config(&self, selx: c_int, sely: c_int) {
        let current_font = self.b_font.current_font;
        let startx = i32::from(self.vars.full_user_rect.x)
            + (1.2 * f32::from(self.vars.block_rect.w)) as i32;
        let starty = i32::from(self.vars.full_user_rect.y) + font_height(&*current_font);
        let col1 = startx + (7.5 * f64::from(char_width(&*current_font, b'O'))) as i32;
        let col2 = col1 + (6.5 * f64::from(char_width(&*current_font, b'O'))) as i32;
        let col3 = col2 + (6.5 * f64::from(char_width(&*current_font, b'O'))) as i32;
        let lheight = font_height(&*self.global.font0_b_font) + 2;

        SDL_UpperBlit(
            self.menu.menu_background,
            null_mut(),
            self.graphics.ne_screen,
            null_mut(),
        );

        #[cfg(feature = "gcw0")]
        PrintStringFont(
            self.graphics.ne_screen,
            Font0_BFont,
            col1,
            starty,
            format_args!("(RShldr to clear an entry)"),
        );

        #[cfg(not(feature = "gcw0"))]
        {
            print_string_font(
                self.graphics.ne_screen,
                self.global.font0_b_font,
                col1,
                starty,
                format_args!("(RShldr to clear an entry)"),
            );
            print_string_font(
                self.graphics.ne_screen,
                self.global.font0_b_font,
                col1,
                starty,
                format_args!("(Backspace to clear an entry)"),
            );
        }

        let mut posy = 1;
        print_string_font(
            self.graphics.ne_screen,
            self.global.font0_b_font,
            startx,
            starty + (posy) * lheight,
            format_args!("Command"),
        );
        print_string_font(
            self.graphics.ne_screen,
            self.global.font0_b_font,
            col1,
            starty + (posy) * lheight,
            format_args!("Key1"),
        );
        print_string_font(
            self.graphics.ne_screen,
            self.global.font0_b_font,
            col2,
            starty + (posy) * lheight,
            format_args!("Key2"),
        );
        print_string_font(
            self.graphics.ne_screen,
            self.global.font0_b_font,
            col3,
            starty + (posy) * lheight,
            format_args!("Key3"),
        );
        posy += 1;

        for (i, cmd_string) in CMD_STRINGS[0..Cmds::Last as usize]
            .iter()
            .copied()
            .enumerate()
        {
            let pos_font = |x, y| {
                if x != selx || i32::try_from(y).unwrap() != sely {
                    self.global.font1_b_font
                } else {
                    self.global.font2_b_font
                }
            };

            print_string_font(
                self.graphics.ne_screen,
                self.global.font0_b_font,
                startx,
                starty + (posy) * lheight,
                format_args!("{}", CStr::from_ptr(cmd_string).to_str().unwrap()),
            );
            print_string_font(
                self.graphics.ne_screen,
                pos_font(1, 1 + i),
                col1,
                starty + (posy) * lheight,
                format_args!(
                    "{}",
                    CStr::from_ptr(
                        self.input.keystr[usize::try_from(self.input.key_cmds[i][0]).unwrap()]
                    )
                    .to_str()
                    .unwrap()
                ),
            );
            print_string_font(
                self.graphics.ne_screen,
                pos_font(2, 1 + i),
                col2,
                starty + (posy) * lheight,
                format_args!(
                    "{}",
                    CStr::from_ptr(
                        self.input.keystr[usize::try_from(self.input.key_cmds[i][1]).unwrap()]
                    )
                    .to_str()
                    .unwrap()
                ),
            );
            print_string_font(
                self.graphics.ne_screen,
                pos_font(3, 1 + i),
                col3,
                starty + (posy) * lheight,
                format_args!(
                    "{}",
                    CStr::from_ptr(
                        self.input.keystr[usize::try_from(self.input.key_cmds[i][2]).unwrap()]
                    )
                    .to_str()
                    .unwrap()
                ),
            );
            posy += 1;
        }

        SDL_Flip(self.graphics.ne_screen);
    }

    pub unsafe fn key_config_menu(&mut self) {
        let mut selx = 1;
        let mut sely = 1; // currently selected menu-position
        const WAIT_MOVE_TICKS: u32 = 100;

        let mut finished = false;
        while !finished {
            self.display_key_config(i32::try_from(selx).unwrap(), i32::try_from(sely).unwrap());

            let action = self.get_menu_action(250);
            let time_for_move =
                SDL_GetTicks() - self.menu.key_config_menu_last_move_tick > WAIT_MOVE_TICKS;

            match action {
                MenuAction::BACK => {
                    finished = true;
                    self.wait_for_all_keys_released();
                }

                MenuAction::CLICK => {
                    self.menu_item_selected_sound();

                    self.input.key_cmds[sely - 1][selx - 1] = b'_'.into();
                    self.display_key_config(
                        i32::try_from(selx).unwrap(),
                        i32::try_from(sely).unwrap(),
                    );
                    self.input.key_cmds[sely - 1][selx - 1] = self.getchar_raw(); // includes joystick input!;
                    self.wait_for_all_keys_released();
                    self.menu.key_config_menu_last_move_tick = SDL_GetTicks();
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
                    self.move_menu_position_sound();
                    self.menu.key_config_menu_last_move_tick = SDL_GetTicks();
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
                    self.move_menu_position_sound();
                    self.menu.key_config_menu_last_move_tick = SDL_GetTicks();
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
                    self.move_menu_position_sound();
                    self.menu.key_config_menu_last_move_tick = SDL_GetTicks();
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
                    self.move_menu_position_sound();
                    self.menu.key_config_menu_last_move_tick = SDL_GetTicks();
                }

                MenuAction::DELETE => {
                    self.input.key_cmds[sely - 1][selx - 1] = 0;
                    self.menu_item_selected_sound();
                }
                _ => {}
            }

            SDL_Delay(1);
        }
    }

    pub unsafe fn show_credits(&mut self) {
        let col2 = 2 * i32::from(self.vars.user_rect.w) / 3;

        let h = font_height(&*self.global.menu_b_font);
        let em = char_width(&*self.global.menu_b_font, b'm');

        let screen = self.vars.screen_rect;
        SDL_SetClipRect(self.graphics.ne_screen, null_mut());
        let image = self.find_file(
            CREDITS_PIC_FILE_C.as_ptr() as *mut c_char,
            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
            Themed::NoTheme as i32,
            Criticality::Critical as i32,
        );
        self.display_image(image);
        self.make_grid_on_screen(Some(&screen));

        let oldfont = std::mem::replace(&mut self.b_font.current_font, self.global.font1_b_font);

        self.printf_sdl(
            self.graphics.ne_screen,
            i32::from(self.get_user_center().x) - 2 * em,
            h,
            format_args!("CREDITS\n"),
        );

        self.printf_sdl(
            self.graphics.ne_screen,
            em,
            -1,
            format_args!("PROGRAMMING:"),
        );
        self.printf_sdl(
            self.graphics.ne_screen,
            col2,
            -1,
            format_args!("Johannes Prix\n"),
        );
        self.printf_sdl(
            self.graphics.ne_screen,
            -1,
            -1,
            format_args!("Reinhard Prix\n"),
        );
        self.printf_sdl(self.graphics.ne_screen, -1, -1, format_args!("\n"));

        self.printf_sdl(self.graphics.ne_screen, em, -1, format_args!("ARTWORK:"));
        self.printf_sdl(
            self.graphics.ne_screen,
            col2,
            -1,
            format_args!("Bastian Salmela\n"),
        );
        self.printf_sdl(self.graphics.ne_screen, -1, -1, format_args!("\n"));
        self.printf_sdl(
            self.graphics.ne_screen,
            em,
            -1,
            format_args!("ADDITIONAL THEMES:\n"),
        );
        self.printf_sdl(
            self.graphics.ne_screen,
            2 * em,
            -1,
            format_args!("Lanzz-theme"),
        );
        self.printf_sdl(self.graphics.ne_screen, col2, -1, format_args!("Lanzz\n"));
        self.printf_sdl(
            self.graphics.ne_screen,
            2 * em,
            -1,
            format_args!("Para90-theme"),
        );
        self.printf_sdl(
            self.graphics.ne_screen,
            col2,
            -1,
            format_args!("Andreas Wedemeyer\n"),
        );

        self.printf_sdl(self.graphics.ne_screen, -1, -1, format_args!("\n"));
        self.printf_sdl(
            self.graphics.ne_screen,
            em,
            -1,
            format_args!("C64 LEGACY MODS:\n"),
        );

        self.printf_sdl(
            self.graphics.ne_screen,
            2 * em,
            -1,
            format_args!("Green Beret, Sanxion, Uridium2"),
        );
        self.printf_sdl(
            self.graphics.ne_screen,
            col2,
            -1,
            format_args!("#dreamfish/trsi\n"),
        );

        self.printf_sdl(
            self.graphics.ne_screen,
            2 * em,
            -1,
            format_args!("The last V8, Anarchy"),
        );
        self.printf_sdl(self.graphics.ne_screen, col2, -1, format_args!("4-mat\n"));

        self.printf_sdl(self.graphics.ne_screen, 2 * em, -1, format_args!("Tron"));
        self.printf_sdl(self.graphics.ne_screen, col2, -1, format_args!("Kollaps\n"));

        self.printf_sdl(
            self.graphics.ne_screen,
            2 * em,
            -1,
            format_args!("Starpaws"),
        );
        self.printf_sdl(self.graphics.ne_screen, col2, -1, format_args!("Nashua\n"));

        self.printf_sdl(
            self.graphics.ne_screen,
            2 * em,
            -1,
            format_args!("Commando"),
        );
        self.printf_sdl(self.graphics.ne_screen, col2, -1, format_args!("Android"));

        SDL_Flip(self.graphics.ne_screen);
        self.wait_for_key_pressed();
        self.b_font.current_font = oldfont;
    }

    /// simple wrapper to ShowMenu() to provide the external entry point into the Level Editor menu
    pub unsafe fn show_level_editor_menu(&mut self) {
        self.menu.quit_level_editor = false;
        self.show_menu(LEVEL_EDITOR_MENU.as_ptr());
    }

    pub unsafe fn handle_configure_keys(&mut self, action: MenuAction) -> *const c_char {
        if action == MenuAction::CLICK {
            self.menu_item_selected_sound();
            self.key_config_menu();
        }

        null_mut()
    }

    pub unsafe fn handle_highscores(&mut self, action: MenuAction) -> *const c_char {
        if action == MenuAction::CLICK {
            self.menu_item_selected_sound();
            self.show_highscores();
        }
        null_mut()
    }

    pub unsafe fn handle_credits(&mut self, action: MenuAction) -> *const c_char {
        if action == MenuAction::CLICK {
            self.menu_item_selected_sound();
            self.show_credits();
        }

        null_mut()
    }

    pub unsafe fn handle_le_save_ship(&mut self, action: MenuAction) -> *const c_char {
        use std::io::Write;

        const SHIPNAME: &CStr = cstr!("Testship");
        libc::snprintf(
            self.menu.fname.as_mut_ptr(),
            self.menu.fname.len() - 1,
            cstr!("%s%s").as_ptr() as *mut c_char,
            SHIPNAME.as_ptr() as *mut c_char,
            SHIP_EXT_C.as_ptr() as *mut c_char,
        );

        if action == MenuAction::INFO {
            return self.menu.fname.as_ptr();
        }

        if action == MenuAction::CLICK {
            self.save_ship(SHIPNAME.as_ptr());
            let mut output = [0; 255];
            let mut cursor = Cursor::new(output.as_mut());
            write!(
                cursor,
                "Ship saved as '{}'",
                CStr::from_ptr(self.menu.fname.as_ptr()).to_str().unwrap()
            )
            .unwrap();
            let position = usize::try_from(cursor.position()).unwrap();
            self.centered_put_string(
                self.graphics.ne_screen,
                3 * font_height(&*self.global.menu_b_font),
                &output[..position],
            );
            SDL_Flip(self.graphics.ne_screen);
            self.wait_for_key_pressed();
            self.initiate_menu(false);
        }

        null_mut()
    }

    pub unsafe fn handle_le_name(&mut self, action: MenuAction) -> *const c_char {
        use std::sync::atomic::Ordering;

        let cur_level = &mut *self.main.cur_level;
        if action == MenuAction::INFO {
            return cur_level.levelname;
        }

        if action == MenuAction::CLICK {
            self.display_text(
                cstr!("New level name: ").as_ptr() as *mut c_char,
                i32::from(self.vars.menu_rect.x) - 2 * self.menu.font_height,
                i32::from(self.vars.menu_rect.y) - 3 * self.menu.font_height,
                &self.vars.full_user_rect,
            );
            SDL_Flip(self.graphics.ne_screen);
            static ALREADY_FREED: AtomicBool = AtomicBool::new(false);
            match ALREADY_FREED.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire) {
                Ok(_) => dealloc_c_string(cur_level.levelname),
                Err(_) => drop(Vec::from_raw_parts(cur_level.levelname as *mut i8, 20, 20)),
            }

            cur_level.levelname = self.get_string(15, 2);
            self.initiate_menu(false);
        }

        null_mut()
    }

    pub unsafe fn handle_open_level_editor(&mut self, action: MenuAction) -> *const c_char {
        if action == MenuAction::CLICK {
            self.menu_item_selected_sound();
            self.level_editor();
        }
        null_mut()
    }

    pub unsafe fn handle_le_exit(&mut self, action: MenuAction) -> *const c_char {
        if action == MenuAction::CLICK {
            self.menu_item_selected_sound();
            self.menu.quit_level_editor = true;
            self.menu.quit_menu = true;
        }
        null_mut()
    }

    pub unsafe fn handle_le_level_number(&mut self, action: MenuAction) -> *const c_char {
        let cur_level = &*self.main.cur_level;
        if action == MenuAction::INFO {
            libc::sprintf(
                self.menu.le_level_number_buf.as_mut_ptr(),
                cstr!("%d").as_ptr() as *mut c_char,
                cur_level.levelnum,
            );
            return self.menu.le_level_number_buf.as_ptr();
        }

        let mut curlevel = cur_level.levelnum;
        self.menu_change_int(
            action,
            &mut curlevel,
            1,
            0,
            self.main.cur_ship.num_levels - 1,
        );
        self.teleport(curlevel, 3, 3);
        self.switch_background_music_to(BYCOLOR.as_ptr());
        self.initiate_menu(false);

        null_mut()
    }

    pub unsafe fn handle_le_color(&mut self, action: MenuAction) -> *const c_char {
        let cur_level = &mut *self.main.cur_level;
        if action == MenuAction::INFO {
            return COLOR_NAMES[usize::try_from(cur_level.color).unwrap()].as_ptr();
        }
        self.menu_change_int(
            action,
            &mut cur_level.color,
            1,
            0,
            c_int::try_from(COLOR_NAMES.len()).unwrap() - 1,
        );
        self.switch_background_music_to(BYCOLOR.as_ptr());
        self.initiate_menu(false);

        null_mut()
    }

    pub unsafe fn handle_le_size_x(&mut self, action: MenuAction) -> *const c_char {
        let cur_level = &mut *self.main.cur_level;
        if action == MenuAction::INFO {
            libc::sprintf(
                self.menu.le_size_x_buf.as_mut_ptr(),
                cstr!("%d").as_ptr() as *mut c_char,
                cur_level.xlen,
            );
            return self.menu.le_size_x_buf.as_ptr();
        }

        let oldxlen = cur_level.xlen;
        self.menu_change_int(
            action,
            &mut cur_level.xlen,
            1,
            0,
            i32::try_from(MAX_MAP_COLS).unwrap() - 1,
        );
        let newmem = usize::try_from(cur_level.xlen).unwrap();
        // adjust memory sizes for new value
        for row in 0..usize::try_from(cur_level.ylen).unwrap() {
            cur_level.map[row] = realloc(
                cur_level.map[row] as *mut u8,
                Layout::array::<i8>(usize::try_from(oldxlen).unwrap()).unwrap(),
                newmem,
            ) as *mut i8;
            if cur_level.map[row].is_null() {
                panic!(
                    "Failed to re-allocate to {} bytes in map row {}",
                    newmem, row,
                );
            }
            if cur_level.xlen > oldxlen {
                // fill new map area with VOID
                *cur_level.map[row].add(usize::try_from(cur_level.xlen - 1).unwrap()) =
                    MapTile::Void as i8;
            }
        }
        self.initiate_menu(false);
        null_mut()
    }

    pub unsafe fn handle_le_size_y(&mut self, action: MenuAction) -> *const c_char {
        use std::cmp::Ordering;

        let cur_level = &mut *self.main.cur_level;
        if action == MenuAction::INFO {
            libc::sprintf(
                self.menu.le_size_y_buf.as_mut_ptr(),
                cstr!("%d").as_ptr() as *mut c_char,
                cur_level.ylen,
            );
            return self.menu.le_size_y_buf.as_ptr();
        }

        let oldylen = cur_level.ylen;
        self.menu_change_int(
            action,
            &mut cur_level.ylen,
            1,
            0,
            i32::try_from(MAX_MAP_ROWS - 1).unwrap(),
        );
        let layout = Layout::array::<i8>(usize::try_from(cur_level.xlen).unwrap()).unwrap();
        match oldylen.cmp(&cur_level.ylen) {
            Ordering::Greater => {
                dealloc(
                    cur_level.map[usize::try_from(oldylen - 1).unwrap()] as *mut u8,
                    layout,
                );
                cur_level.map[usize::try_from(oldylen - 1).unwrap()] = null_mut();
            }
            Ordering::Less => {
                cur_level.map[usize::try_from(cur_level.ylen - 1).unwrap()] =
                    alloc_zeroed(layout) as *mut i8;
                std::ptr::write_bytes(
                    cur_level.map[usize::try_from(cur_level.ylen - 1).unwrap()],
                    MapTile::Void as u8,
                    usize::try_from(cur_level.xlen).unwrap(),
                )
            }
            Ordering::Equal => {}
        }

        self.initiate_menu(false);
        null_mut()
    }

    pub unsafe fn handle_strictly_classic(&mut self, action: MenuAction) -> *const c_char {
        if action == MenuAction::CLICK {
            self.menu_item_selected_sound();
            self.global.game_config.droid_talk = false.into();
            self.global.game_config.show_decals = false.into();
            self.global.game_config.takeover_activates = true.into();
            self.global.game_config.fire_hold_takeover = true.into();
            self.global.game_config.all_map_visible = true.into();
            self.global.game_config.empty_level_speedup = 1.0;

            // set window type
            self.global.game_config.full_user_rect = false.into();
            self.vars.user_rect = self.vars.classic_user_rect;
            // set theme
            self.set_theme(self.graphics.classic_theme_index);
            self.initiate_menu(false);
        }

        null_mut()
    }

    pub unsafe fn handle_window_type(&mut self, action: MenuAction) -> *const c_char {
        if action == MenuAction::INFO {
            return if self.global.game_config.full_user_rect != 0 {
                cstr!("Full").as_ptr()
            } else {
                cstr!("Classic").as_ptr()
            };
        }

        if action == MenuAction::CLICK || action == MenuAction::LEFT || action == MenuAction::RIGHT
        {
            let toggle = &mut self.global.game_config.full_user_rect as *mut i32;
            self.flip_toggle(toggle);
            if self.global.game_config.full_user_rect != 0 {
                self.vars.user_rect = self.vars.full_user_rect;
            } else {
                self.vars.user_rect = self.vars.classic_user_rect;
            }

            self.initiate_menu(false);
        }
        null_mut()
    }

    pub unsafe fn handle_theme(&mut self, action: MenuAction) -> *const c_char {
        if action == MenuAction::INFO {
            return self.graphics.all_themes.theme_name
                [usize::try_from(self.graphics.all_themes.cur_tnum).unwrap()]
                as *const c_char;
        }

        if action == MenuAction::CLICK || action == MenuAction::LEFT || action == MenuAction::RIGHT
        {
            self.move_lift_sound();
            let mut tnum = self.graphics.all_themes.cur_tnum;
            if action == MenuAction::CLICK && action == MenuAction::RIGHT {
                tnum += 1;
            } else {
                tnum -= 1;
            }

            if tnum < 0 {
                tnum = self.graphics.all_themes.num_themes - 1;
            }
            if tnum > self.graphics.all_themes.num_themes - 1 {
                tnum = 0;
            }

            self.set_theme(tnum);
            self.initiate_menu(false);
        }

        null_mut()
    }

    pub unsafe fn handle_droid_talk(&mut self, action: MenuAction) -> *const c_char {
        if action == MenuAction::INFO {
            return is_toggle_on(self.global.game_config.droid_talk);
        }
        if action == MenuAction::CLICK || action == MenuAction::LEFT || action == MenuAction::RIGHT
        {
            let toggle = &mut self.global.game_config.droid_talk as *mut i32;
            self.flip_toggle(toggle);
        }
        null_mut()
    }

    pub unsafe fn handle_all_map_visible(&mut self, action: MenuAction) -> *const c_char {
        if action == MenuAction::INFO {
            return is_toggle_on(self.global.game_config.all_map_visible);
        }
        if action == MenuAction::CLICK || action == MenuAction::LEFT || action == MenuAction::RIGHT
        {
            let toggle = &mut self.global.game_config.all_map_visible as *mut i32;
            self.flip_toggle(toggle);
            self.initiate_menu(false);
        }
        null_mut()
    }

    pub unsafe fn handle_show_decals(&mut self, action: MenuAction) -> *const c_char {
        if action == MenuAction::INFO {
            return is_toggle_on(self.global.game_config.show_decals);
        }
        if action == MenuAction::CLICK || action == MenuAction::LEFT || action == MenuAction::RIGHT
        {
            let toggle = &mut self.global.game_config.show_decals as *mut i32;
            self.flip_toggle(toggle);
            self.initiate_menu(false);
        }
        null_mut()
    }

    pub unsafe fn handle_transfer_is_activate(&mut self, action: MenuAction) -> *const c_char {
        if action == MenuAction::INFO {
            return is_toggle_on(self.global.game_config.takeover_activates);
        }
        if action == MenuAction::CLICK || action == MenuAction::LEFT || action == MenuAction::RIGHT
        {
            let toggle = &mut self.global.game_config.takeover_activates as *mut i32;
            self.flip_toggle(toggle);
        }
        null_mut()
    }

    pub unsafe fn handle_fire_is_transfer(&mut self, action: MenuAction) -> *const c_char {
        if action == MenuAction::INFO {
            return is_toggle_on(self.global.game_config.fire_hold_takeover);
        }
        if action == MenuAction::CLICK || action == MenuAction::LEFT || action == MenuAction::RIGHT
        {
            let toggle = &mut self.global.game_config.fire_hold_takeover as *mut i32;
            self.flip_toggle(toggle);
        }
        null_mut()
    }

    pub unsafe fn handle_empty_level_speedup(&mut self, action: MenuAction) -> *const c_char {
        if action == MenuAction::INFO {
            libc::sprintf(
                self.menu.empty_level_speedup_buf.as_mut_ptr(),
                cstr!("%3.1f").as_ptr() as *mut c_char,
                f64::from(self.global.game_config.empty_level_speedup),
            );
            return self.menu.empty_level_speedup_buf.as_ptr();
        }

        let mut f = self.global.game_config.empty_level_speedup;
        self.menu_change_float(action, &mut f, 0.1, 0.5, 2.0);
        self.global.game_config.empty_level_speedup = f;
        null_mut()
    }

    pub unsafe fn handle_music_volume(&mut self, action: MenuAction) -> *const c_char {
        if action == MenuAction::INFO {
            libc::sprintf(
                self.menu.music_volume_buf.as_mut_ptr(),
                cstr!("%4.2f").as_ptr() as *mut c_char,
                f64::from(self.global.game_config.current_bg_music_volume),
            );
            return self.menu.music_volume_buf.as_ptr();
        }

        let mut f = self.global.game_config.current_bg_music_volume;
        self.menu_change_float(action, &mut f, 0.05, 0., 1.);
        self.global.game_config.current_bg_music_volume = f;

        self.set_bg_music_volume(self.global.game_config.current_bg_music_volume);
        null_mut()
    }

    pub unsafe fn handle_sound_volume(&mut self, action: MenuAction) -> *const c_char {
        if action == MenuAction::INFO {
            libc::sprintf(
                self.menu.sound_volume_buf.as_mut_ptr(),
                cstr!("%4.2f").as_ptr() as *mut c_char,
                f64::from(self.global.game_config.current_sound_fx_volume),
            );
            return self.menu.sound_volume_buf.as_ptr();
        }

        let mut f = self.global.game_config.current_sound_fx_volume;
        self.menu_change_float(action, &mut f, 0.05, 0., 1.);
        self.global.game_config.current_sound_fx_volume = f;
        self.set_sound_f_x_volume(self.global.game_config.current_sound_fx_volume);
        null_mut()
    }

    pub unsafe fn handle_fullscreen(&mut self, action: MenuAction) -> *const c_char {
        if action == MenuAction::INFO {
            return is_toggle_on(self.global.game_config.use_fullscreen);
        }
        if action == MenuAction::CLICK || action == MenuAction::LEFT || action == MenuAction::RIGHT
        {
            self.toggle_fullscreen();
            self.menu_item_selected_sound();
        }
        null_mut()
    }

    pub unsafe fn handle_show_position(&mut self, action: MenuAction) -> *const c_char {
        if action == MenuAction::INFO {
            return is_toggle_on(self.global.game_config.draw_position);
        }
        if action == MenuAction::CLICK || action == MenuAction::LEFT || action == MenuAction::RIGHT
        {
            let mut f = self.global.game_config.draw_position;
            self.flip_toggle(&mut f);
            self.global.game_config.draw_position = f;
            self.initiate_menu(false);
        }
        null_mut()
    }

    pub unsafe fn handle_show_framerate(&mut self, action: MenuAction) -> *const c_char {
        if action == MenuAction::INFO {
            return is_toggle_on(self.global.game_config.draw_framerate);
        }
        if action == MenuAction::CLICK || action == MenuAction::LEFT || action == MenuAction::RIGHT
        {
            let mut f = self.global.game_config.draw_framerate;
            self.flip_toggle(&mut f);
            self.global.game_config.draw_framerate = f;
            self.initiate_menu(false);
        }
        null_mut()
    }

    pub unsafe fn handle_show_energy(&mut self, action: MenuAction) -> *const c_char {
        if action == MenuAction::INFO {
            return is_toggle_on(self.global.game_config.draw_energy);
        }
        if action == MenuAction::CLICK || action == MenuAction::LEFT || action == MenuAction::RIGHT
        {
            let mut f = self.global.game_config.draw_energy;
            self.flip_toggle(&mut f);
            self.global.game_config.draw_energy = f;
            self.initiate_menu(false);
        }
        null_mut()
    }

    unsafe fn menu_change<T>(
        &self,
        action: MenuAction,
        val: &mut T,
        step: T,
        min_value: T,
        max_value: T,
    ) where
        T: PartialOrd + AddAssign + SubAssign,
    {
        if action == MenuAction::RIGHT && *val < max_value {
            self.move_lift_sound();
            *val += step;
            if *val > max_value {
                *val = max_value;
            }
        } else if action == MenuAction::LEFT && *val > min_value {
            self.move_lift_sound();
            *val -= step;
            if *val <= min_value {
                *val = min_value;
            }
        }
    }

    pub unsafe fn menu_change_float(
        &self,
        action: MenuAction,
        val: &mut c_float,
        step: c_float,
        min_value: c_float,
        max_value: c_float,
    ) {
        self.menu_change(action, val, step, min_value, max_value)
    }

    pub unsafe fn menu_change_int(
        &self,
        action: MenuAction,
        val: &mut c_int,
        step: c_int,
        min_value: c_int,
        max_value: c_int,
    ) {
        self.menu_change(action, val, step, min_value, max_value)
    }

    pub unsafe fn flip_toggle(&self, toggle: *mut c_int) {
        if toggle.is_null().not() {
            self.menu_item_selected_sound();
            *toggle = !*toggle;
        }
    }

    pub unsafe fn set_theme(&mut self, theme_index: c_int) {
        assert!(theme_index >= 0 && theme_index < self.graphics.all_themes.num_themes);

        self.graphics.all_themes.cur_tnum = theme_index;
        libc::strcpy(
            self.global.game_config.theme_name.as_mut_ptr(),
            self.graphics.all_themes.theme_name
                [usize::try_from(self.graphics.all_themes.cur_tnum).unwrap()]
                as *const c_char,
        );
        self.init_pictures();
    }
}

pub fn is_toggle_on(toggle: c_int) -> *const c_char {
    if toggle != 0 {
        cstr!("YES").as_ptr()
    } else {
        cstr!("NO").as_ptr()
    }
}
