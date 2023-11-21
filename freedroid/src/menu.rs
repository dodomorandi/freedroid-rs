#[cfg(feature = "gcw0")]
use crate::defs::{gcw0_a_pressed, gcw0_any_button_pressed, gcw0_any_button_pressed_r};

use crate::{
    array_c_string::ArrayCString,
    b_font::{char_width, font_height, print_string_font},
    cur_level,
    defs::{
        AssembleCombatWindowFlags, Cmds, Criticality, DisplayBannerFlags, MapTile, MenuAction,
        Status, Themed, BYCOLOR, CREDITS_PIC_FILE, GRAPHICS_DIR_C, MAX_MAP_COLS, MAX_MAP_ROWS,
    },
    global::{Global, INFLUENCE_MODE_NAMES},
    graphics::Graphics,
    input::{CMD_STRINGS, KEY_STRINGS},
    map::COLOR_NAMES,
    sound::Sound,
    Data, Sdl,
};

use cstr::cstr;
use nom::Finish;
use sdl::Surface;
use sdl_sys::{
    SDLKey_SDLK_BACKSPACE, SDLKey_SDLK_DOWN, SDLKey_SDLK_ESCAPE, SDLKey_SDLK_LEFT,
    SDLKey_SDLK_RIGHT, SDLKey_SDLK_UP,
};
use std::{
    ffi::CStr,
    io::Cursor,
    ops::{AddAssign, SubAssign},
    os::raw::{c_float, c_int},
};

#[derive(Debug, Default)]
pub struct Menu<'sdl> {
    font_height: i32,
    menu_background: Option<Surface<'sdl>>,
    quit_menu: bool,
    pub quit_level_editor: bool,
    last_movekey_time: u32,
    menu_action_directions: MenuActionDirections,
    show_menu_last_move_tick: u32,
    key_config_menu_last_move_tick: u32,
    fname: ArrayCString<256>,
    le_level_number_buf: ArrayCString<256>,
    le_size_x_buf: ArrayCString<256>,
    le_size_y_buf: ArrayCString<256>,
    empty_level_speedup_buf: ArrayCString<256>,
    music_volume_buf: ArrayCString<256>,
    sound_volume_buf: ArrayCString<256>,
}

#[derive(Debug, Default)]
struct MenuActionDirections {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

// const FILENAME_LEN: u8 = 128;
const SHIP_EXT_C: &CStr = cstr!(".shp");
pub const SHIP_EXT: &str = ".shp";
// const ELEVEXT: &CStr = cstr!(".elv");
// const CREWEXT: &CStr = cstr!(".crw");

macro_rules! menu_entry {
    () => {
        MenuEntry {
            name: None,
            handler: None,
            submenu: None,
        }
    };
    ($name:tt) => {
        MenuEntry {
            name: Some($name),
            handler: None,
            submenu: None,
        }
    };
    ($name:tt, $handler:expr) => {
        MenuEntry {
            name: Some($name),
            handler: Some($handler),
            submenu: None,
        }
    };
    ($name:tt, None, $submenu:expr) => {
        MenuEntry {
            name: Some($name),
            handler: None,
            submenu: Some(&$submenu),
        }
    };
    ($name:tt, $handler:expr, $submenu:expr) => {
        MenuEntry {
            name: Some($name),
            handler: Some($handler),
            submenu: Some(&$submenu),
        }
    };
}

pub struct MenuEntry<'sdl> {
    name: Option<&'static str>,
    handler: Option<for<'a> fn(&'a mut Data<'sdl>, MenuAction) -> Option<&'a CStr>>,
    submenu: Option<&'sdl [MenuEntry<'sdl>]>,
}

impl<'sdl> Data<'sdl> {
    #[cfg(target_os = "android")]
    const LEGACY_MENU: [MenuEntry<'sdl>; 9] = [
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
    const LEGACY_MENU: [MenuEntry<'sdl>; 11] = [
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

    const GRAPHICS_SOUND_MENU: [MenuEntry<'sdl>; 5] = [
        menu_entry! { "Back"},
        menu_entry! { "Music Volume: ", Data::handle_music_volume},
        menu_entry! { "Sound Volume: ", Data::handle_sound_volume},
        menu_entry! { "Fullscreen Mode: ", Data::handle_fullscreen},
        menu_entry! {},
    ];

    const HUD_MENU: [MenuEntry<'sdl>; 5] = [
        menu_entry! { "Back"},
        menu_entry! { "Show Position: ", Data::handle_show_position},
        menu_entry! { "Show Framerate: ", Data::handle_show_framerate},
        menu_entry! { "Show Energy: ", Data::handle_show_energy},
        menu_entry! {},
    ];

    const LEVEL_EDITOR_MENU: [MenuEntry<'sdl>; 8] = [
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
    const MAIN_MENU: [MenuEntry<'sdl>; 8] = [
        menu_entry! { "Back to Game"},
        menu_entry! { "Graphics & Sound", None, Self::GRAPHICS_SOUND_MENU },
        menu_entry! { "Legacy Options", None, Self::LEGACY_MENU },
        menu_entry! { "HUD Settings", None, Self::HUD_MENU },
        menu_entry! { "Highscores", Data::handle_highscores},
        menu_entry! { "Credits", Data::handle_credits},
        menu_entry! { "Quit Game", Data::handle_quit_game},
        menu_entry! {},
    ];

    #[cfg(not(target_os = "android"))]
    const MAIN_MENU: [MenuEntry<'sdl>; 10] = [
        menu_entry! { "Back to Game"},
        menu_entry! { "Graphics & Sound", None, Self::GRAPHICS_SOUND_MENU },
        menu_entry! { "Legacy Options", None, Self::LEGACY_MENU },
        menu_entry! { "HUD Settings", None, Self::HUD_MENU },
        menu_entry! { "Level Editor", Data::handle_open_level_editor},
        menu_entry! { "Highscores", Data::handle_highscores},
        menu_entry! { "Credits", Data::handle_credits},
        menu_entry! { "Configure Keys", Data::handle_configure_keys},
        menu_entry! { "Quit Game", Data::handle_quit_game},
        menu_entry! {},
    ];

    pub fn handle_quit_game(&mut self, action: MenuAction) -> Option<&CStr> {
        if action != MenuAction::CLICK {
            return None;
        }

        self.menu_item_selected_sound();
        self.initiate_menu(false);

        #[cfg(feature = "gcw0")]
        const QUIT_STRING: &CStr = cstr!("Press A to quit");

        #[cfg(not(feature = "gcw0"))]
        const QUIT_STRING: &[u8] = b"Hit 'y' or press Fire to quit";

        let text_width = self.text_width(QUIT_STRING);
        let text_x = i32::from(self.vars.user_rect.x())
            + (i32::from(self.vars.user_rect.width()) - text_width) / 2;
        let text_y = i32::from(self.vars.user_rect.y())
            + (i32::from(self.vars.user_rect.height()) - self.menu.font_height) / 2;
        let mut ne_screen = self.graphics.ne_screen.take().unwrap();
        self.put_string(&mut ne_screen, text_x, text_y, QUIT_STRING);
        assert!(ne_screen.flip());
        self.graphics.ne_screen = Some(ne_screen);

        #[cfg(feature = "gcw0")]
        {
            while !gcw0_any_button_pressed() {
                self.sdl.delay_ms(1);
            }

            if gcw0_a_pressed() {
                while !gcw0_any_button_pressed_r() {
                    // In case FirePressed && !Gcw0APressed() -> would cause a loop otherwise in the menu...
                    self.sdl.delay_ms(1);
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
                self.quit.set(true);
            }
        }

        None
    }

    /// simple wrapper to ShowMenu() to provide the external entry point into the main menu
    pub fn show_main_menu(&mut self) {
        self.show_menu(&Self::MAIN_MENU);
    }

    pub fn free_menu_data(&mut self) {
        self.menu.menu_background = None
    }

    pub fn initiate_menu(&mut self, with_droids: bool) {
        // Here comes the standard initializer for all the menus and submenus
        // of the big escape menu.  This prepares the screen, so that we can
        // write on it further down.
        self.activate_conservative_frame_computation();

        self.graphics.ne_screen.as_mut().unwrap().clear_clip_rect();
        self.vars.me.status = Status::Menu as i32;
        self.clear_graph_mem();
        self.display_banner(
            None,
            None,
            (DisplayBannerFlags::NO_SDL_UPDATE | DisplayBannerFlags::FORCE_UPDATE)
                .bits()
                .into(),
        );
        if with_droids {
            self.assemble_combat_picture(0);
        } else {
            self.assemble_combat_picture(AssembleCombatWindowFlags::ONLY_SHOW_MAP.bits().into());
        }

        self.graphics.ne_screen.as_mut().unwrap().clear_clip_rect();
        self.make_grid_on_screen(None);

        // keep a global copy of background
        self.menu.menu_background = Some(
            self.graphics
                .ne_screen
                .as_mut()
                .unwrap()
                .display_format()
                .unwrap(),
        );

        self.sdl.cursor().hide();
        self.b_font.current_font = self.global.menu_b_font.clone();
        self.menu.font_height = font_height(
            self.b_font
                .current_font
                .as_ref()
                .unwrap()
                .ro(&self.font_owner),
        ) + 2;
    }

    pub fn cheatmenu(&mut self) {
        // Prevent distortion of framerate by the delay coming from
        // the time spend in the menu.
        self.activate_conservative_frame_computation();

        let font = self.global.font0_b_font.clone();

        self.b_font.current_font = font; /* not the ideal one, but there's currently */
        /* no other it seems.. */
        const X0: i32 = 50;
        const Y0: i32 = 20;

        let mut resume = false;
        while !resume {
            self.clear_graph_mem();
            let mut ne_screen = self.graphics.ne_screen.take().unwrap();
            Self::printf_sdl_static(
                &mut self.text,
                &self.b_font,
                &mut self.font_owner,
                &mut ne_screen,
                X0,
                Y0,
                format_args!(
                    "Current position: Level={}, X={:.0}, Y={:.0}\n",
                    cur_level!(self.main).levelnum,
                    self.vars.me.pos.x.clone(),
                    self.vars.me.pos.y.clone(),
                ),
            );
            self.printf_sdl(
                &mut ne_screen,
                -1,
                -1,
                format_args!(" a. Armageddon (alle Robots sprengen)\n"),
            );
            self.printf_sdl(
                &mut ne_screen,
                -1,
                -1,
                format_args!(" l. robot list of current level\n"),
            );
            self.printf_sdl(
                &mut ne_screen,
                -1,
                -1,
                format_args!(" g. complete robot list\n"),
            );
            self.printf_sdl(
                &mut ne_screen,
                -1,
                -1,
                format_args!(" d. destroy robots on current level\n"),
            );
            self.printf_sdl(&mut ne_screen, -1, -1, format_args!(" t. Teleportation\n"));
            self.printf_sdl(
                &mut ne_screen,
                -1,
                -1,
                format_args!(" r. change to new robot type\n"),
            );
            self.printf_sdl(
                &mut ne_screen,
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
            self.printf_sdl(&mut ne_screen, -1, -1, format_args!(" e. set energy\n"));
            self.printf_sdl(
                &mut ne_screen,
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
            self.printf_sdl(&mut ne_screen, -1, -1, format_args!(" m. Map of Deck xy\n"));
            self.printf_sdl(
                &mut ne_screen,
                -1,
                -1,
                format_args!(
                    " s. Sound: {}\n",
                    if self.main.sound_on != 0 { "ON" } else { "OFF" }
                ),
            );
            self.printf_sdl(
                &mut ne_screen,
                -1,
                -1,
                format_args!(" w. Print current waypoints\n"),
            );
            self.printf_sdl(
                &mut ne_screen,
                -1,
                -1,
                format_args!(" z. change Zoom factor\n"),
            );
            self.printf_sdl(
                &mut ne_screen,
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
            self.printf_sdl(&mut ne_screen, -1, -1, format_args!(" q. RESUME game\n"));

            match u8::try_from(self.getchar_raw()).ok() {
                Some(b'f') => {
                    self.main.stop_influencer = !self.main.stop_influencer;
                }

                Some(b'z') => {
                    self.graphics.ne_screen = Some(ne_screen);
                    self.clear_graph_mem();
                    ne_screen = self.graphics.ne_screen.take().unwrap();
                    self.printf_sdl(
                        &mut ne_screen,
                        X0,
                        Y0,
                        format_args!(
                            "Current Zoom factor: {}\n",
                            self.global.current_combat_scale_factor.clone(),
                        ),
                    );
                    self.printf_sdl(&mut ne_screen, -1, -1, format_args!("New zoom factor: "));
                    self.graphics.ne_screen = Some(ne_screen);
                    let input = self.get_string(40, 2).unwrap();
                    ne_screen = self.graphics.ne_screen.take().unwrap();
                    use nom::{
                        character::complete::space0, number::complete::float, sequence::preceded,
                    };

                    self.global.current_combat_scale_factor =
                        preceded(space0::<_, ()>, float)(input.to_bytes())
                            .finish()
                            .unwrap()
                            .1;
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
                        if self.main.all_enemys[i].levelnum == cur_level!(self.main).levelnum {
                            if l != 0 && l % 20 == 0 {
                                Self::printf_sdl_static(
                                    &mut self.text,
                                    &self.b_font,
                                    &mut self.font_owner,
                                    &mut ne_screen,
                                    -1,
                                    -1,
                                    format_args!(" --- MORE --- \n"),
                                );
                                if self.getchar_raw() == b'q'.into() {
                                    break;
                                }
                            }
                            if l % 20 == 0 {
                                self.graphics.ne_screen = Some(ne_screen);
                                self.clear_graph_mem();
                                ne_screen = self.graphics.ne_screen.take().unwrap();
                                Self::printf_sdl_static(
                                    &mut self.text,
                                    &self.b_font,
                                    &mut self.font_owner,
                                    &mut ne_screen,
                                    X0,
                                    Y0,
                                    format_args!(" NR.   ID  X    Y   ENERGY   Status\n"),
                                );
                                Self::printf_sdl_static(
                                    &mut self.text,
                                    &self.b_font,
                                    &mut self.font_owner,
                                    &mut ne_screen,
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

                            Self::printf_sdl_static(
                                &mut self.text,
                                &self.b_font,
                                &mut self.font_owner,
                                &mut ne_screen,
                                -1,
                                -1,
                                format_args!(
                                    "{}.   {}   {:.0}   {:.0}   {:.0}    {}.\n",
                                    i,
                                    self.vars.droidmap
                                        [usize::try_from(self.main.all_enemys[i].ty).unwrap()]
                                    .druidname
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

                    self.printf_sdl(&mut ne_screen, -1, -1, format_args!(" --- END --- \n"));
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
                                &mut ne_screen,
                                -1,
                                -1,
                                format_args!(" --- MORE --- ('q' to quit)\n"),
                            );
                            if self.getchar_raw() == b'q'.into() {
                                break;
                            }
                        }
                        if i % 13 == 0 {
                            self.graphics.ne_screen = Some(ne_screen);
                            self.clear_graph_mem();
                            ne_screen = self.graphics.ne_screen.take().unwrap();
                            self.printf_sdl(
                                &mut ne_screen,
                                X0,
                                Y0,
                                format_args!("Nr.  Lev. ID  Energy  Status.\n"),
                            );
                            self.printf_sdl(
                                &mut ne_screen,
                                -1,
                                -1,
                                format_args!("------------------------------\n"),
                            );
                        }

                        Self::printf_sdl_static(
                            &mut self.text,
                            &self.b_font,
                            &mut self.font_owner,
                            &mut ne_screen,
                            -1,
                            -1,
                            format_args!(
                                "{}  {}  {}  {:.0}  {}\n",
                                i,
                                self.main.all_enemys[i].levelnum.clone(),
                                self.vars.droidmap
                                    [usize::try_from(self.main.all_enemys[i].ty).unwrap()]
                                .druidname
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

                    self.printf_sdl(&mut ne_screen, -1, -1, format_args!(" --- END ---\n"));
                    self.getchar_raw();
                }

                Some(b'd') => {
                    /* destroy all robots on this level, haha */
                    let cur_level = cur_level!(self.main);
                    for enemy in &mut self.main.all_enemys {
                        if enemy.levelnum == cur_level.levelnum {
                            enemy.energy = -100.;
                        }
                    }
                    self.printf_sdl(
                        &mut ne_screen,
                        -1,
                        -1,
                        format_args!("All robots on this deck killed!\n"),
                    );
                    self.getchar_raw();
                }

                Some(b't') => {
                    /* Teleportation */
                    self.graphics.ne_screen = Some(ne_screen);
                    self.clear_graph_mem();
                    ne_screen = self.graphics.ne_screen.take().unwrap();
                    self.printf_sdl(&mut ne_screen, X0, Y0, format_args!("Enter Level, X, Y: "));
                    self.graphics.ne_screen = Some(ne_screen);
                    let input = self.get_string(40, 2).unwrap();
                    ne_screen = self.graphics.ne_screen.take().unwrap();

                    use nom::{
                        bytes::complete::tag,
                        character::complete::{i32, space0},
                        sequence::{delimited, pair, preceded, tuple},
                    };

                    let (l_num, x, y) = tuple((
                        preceded(space0::<_, ()>, i32),
                        preceded(pair(tag(", "), space0), i32),
                        delimited(pair(tag(", "), space0), i32, tag("\n")),
                    ))(input.to_bytes())
                    .finish()
                    .unwrap()
                    .1;
                    self.teleport(l_num, x, y);
                }

                Some(b'r') => {
                    /* change to new robot type */
                    self.graphics.ne_screen = Some(ne_screen);
                    self.clear_graph_mem();
                    ne_screen = self.graphics.ne_screen.take().unwrap();
                    self.printf_sdl(
                        &mut ne_screen,
                        X0,
                        Y0,
                        format_args!("Type number of new robot: "),
                    );
                    self.graphics.ne_screen = Some(ne_screen);
                    let input = self.get_string(40, 2).unwrap();
                    ne_screen = self.graphics.ne_screen.take().unwrap();
                    let mut i = 0;
                    for _ in 0..u32::from(self.main.number_of_droid_types) {
                        if self.vars.droidmap[i].druidname != *input {
                            break;
                        }
                        i += 1;
                    }

                    if i == usize::from(self.main.number_of_droid_types) {
                        self.printf_sdl(
                            &mut ne_screen,
                            X0,
                            Y0 + 20,
                            format_args!("Unrecognized robot-type: {}", input.to_str().unwrap(),),
                        );
                        self.getchar_raw();
                        self.graphics.ne_screen = Some(ne_screen);
                        self.clear_graph_mem();
                        ne_screen = self.graphics.ne_screen.take().unwrap();
                    } else {
                        self.vars.me.ty = i.try_into().unwrap();
                        self.vars.me.energy =
                            self.vars.droidmap[usize::try_from(self.vars.me.ty).unwrap()].maxenergy;
                        self.vars.me.health = self.vars.me.energy;
                        self.printf_sdl(
                            &mut ne_screen,
                            X0,
                            Y0 + 20,
                            format_args!("You are now a {}. Have fun!\n", input.to_str().unwrap(),),
                        );
                        self.getchar_raw();
                    }
                }

                Some(b'i') => {
                    /* togge Invincible mode */
                    self.main.invincible_mode = !self.main.invincible_mode;
                }

                Some(b'e') => {
                    /* complete heal */
                    self.graphics.ne_screen = Some(ne_screen);
                    self.clear_graph_mem();
                    ne_screen = self.graphics.ne_screen.take().unwrap();
                    self.printf_sdl(
                        &mut ne_screen,
                        X0,
                        Y0,
                        format_args!("Current energy: {}\n", self.vars.me.energy.clone()),
                    );
                    self.printf_sdl(
                        &mut ne_screen,
                        -1,
                        -1,
                        format_args!("Enter your new energy: "),
                    );
                    self.graphics.ne_screen = Some(ne_screen);
                    let input = self.get_string(40, 2).unwrap();
                    ne_screen = self.graphics.ne_screen.take().unwrap();

                    use nom::{
                        character::complete::{i32, space0},
                        sequence::preceded,
                    };
                    let num = preceded(space0::<_, ()>, i32)(input.to_bytes())
                        .finish()
                        .unwrap()
                        .1;
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
                    self.printf_sdl(&mut ne_screen, -1, -1, format_args!("\nLevelnum: "));
                    self.graphics.ne_screen = Some(ne_screen);
                    self.show_deck_map();
                    ne_screen = self.graphics.ne_screen.take().unwrap();
                    self.getchar_raw();
                }

                Some(b'w') => {
                    /* print waypoint info of current level */
                    for i in 0..cur_level!(self.main).all_waypoints.len() {
                        if i != 0 && i % 20 == 0 {
                            self.printf_sdl(
                                &mut ne_screen,
                                -1,
                                -1,
                                format_args!(" ---- MORE -----\n"),
                            );
                            if self.getchar_raw() == b'q'.into() {
                                break;
                            }
                        }
                        if i % 20 == 0 {
                            self.graphics.ne_screen = Some(ne_screen);
                            self.clear_graph_mem();
                            ne_screen = self.graphics.ne_screen.take().unwrap();
                            self.printf_sdl(
                                &mut ne_screen,
                                X0,
                                Y0,
                                format_args!("Nr.   X   Y      C1  C2  C3  C4\n"),
                            );
                            self.printf_sdl(
                                &mut ne_screen,
                                -1,
                                -1,
                                format_args!("------------------------------------\n"),
                            );
                        }
                        let cur_level = cur_level!(self.main);
                        let waypoint = &cur_level.all_waypoints[i];
                        Self::printf_sdl_static(
                            &mut self.text,
                            &self.b_font,
                            &mut self.font_owner,
                            &mut ne_screen,
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
                    self.printf_sdl(&mut ne_screen, -1, -1, format_args!(" --- END ---\n"));
                    self.getchar_raw();
                }

                Some(b' ') | Some(b'q') => {
                    resume = true;
                }

                _ => {}
            }
            self.graphics.ne_screen = Some(ne_screen);
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
    pub fn get_menu_action(&mut self, wait_repeat_ticks: u32) -> MenuAction {
        let mut action = MenuAction::empty();

        // 'normal' menu action keys get released
        if self.key_is_pressed_r(SDLKey_SDLK_BACKSPACE as c_int) {
            {
                action = MenuAction::DELETE;
            }
        }
        if self.cmd_is_active_r(Cmds::Back) || self.key_is_pressed_r(SDLKey_SDLK_ESCAPE as c_int) {
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
            && (self.up_pressed() || self.key_is_pressed(SDLKey_SDLK_UP as c_int))
        {
            self.menu.menu_action_directions.up = true;
            self.menu.last_movekey_time = self.sdl.ticks_ms();
            action |= MenuAction::UP;
        }
        if !self.menu.menu_action_directions.down
            && (self.down_pressed() || self.key_is_pressed(SDLKey_SDLK_DOWN as c_int))
        {
            self.menu.menu_action_directions.down = true;
            self.menu.last_movekey_time = self.sdl.ticks_ms();
            action |= MenuAction::DOWN;
        }
        if !self.menu.menu_action_directions.left
            && (self.left_pressed() || self.key_is_pressed(SDLKey_SDLK_LEFT as c_int))
        {
            self.menu.menu_action_directions.left = true;
            self.menu.last_movekey_time = self.sdl.ticks_ms();
            action |= MenuAction::LEFT;
        }
        if !self.menu.menu_action_directions.right
            && (self.right_pressed() || self.key_is_pressed(SDLKey_SDLK_RIGHT as c_int))
        {
            self.menu.menu_action_directions.right = true;
            self.menu.last_movekey_time = self.sdl.ticks_ms();
            action |= MenuAction::RIGHT;
        }

        if !(self.up_pressed() || self.key_is_pressed(SDLKey_SDLK_UP as c_int)) {
            self.menu.menu_action_directions.up = false;
        }
        if !(self.down_pressed() || self.key_is_pressed(SDLKey_SDLK_DOWN as c_int)) {
            self.menu.menu_action_directions.down = false;
        }
        if !(self.left_pressed() || self.key_is_pressed(SDLKey_SDLK_LEFT as c_int)) {
            self.menu.menu_action_directions.left = false;
        }
        if !(self.right_pressed() || self.key_is_pressed(SDLKey_SDLK_RIGHT as c_int)) {
            self.menu.menu_action_directions.right = false;
        }

        // check if enough time since we registered last new move-action
        if self.sdl.ticks_ms() - self.menu.last_movekey_time > wait_repeat_ticks {
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
    pub fn show_menu(&mut self, menu_entries: &[MenuEntry<'sdl>]) {
        use std::io::Write;

        self.initiate_menu(false);
        self.wait_for_all_keys_released();

        // figure out menu-start point to make it centered
        let mut num_entries = 0;
        let mut menu_width = None::<i32>;
        loop {
            let entry = &menu_entries[num_entries];
            let name = match entry.name.as_ref() {
                Some(name) => name,
                None => break,
            };

            let width = self.text_width(name.as_bytes());
            menu_width = Some(
                menu_width
                    .map(|menu_width| menu_width.max(width))
                    .unwrap_or(width),
            );

            num_entries += 1;
        }
        let menu_entries = &menu_entries[..num_entries];
        let menu_width = menu_width.unwrap();

        let menu_height = i32::try_from(num_entries).unwrap() * self.menu.font_height;
        let menu_x = i32::from(self.vars.full_user_rect.x())
            + (i32::from(self.vars.full_user_rect.width()) - menu_width) / 2;
        let menu_y = i32::from(self.vars.full_user_rect.y())
            + (i32::from(self.vars.full_user_rect.height()) - menu_height) / 2;
        let influ_x = menu_x - i32::from(self.vars.block_rect.width()) - self.menu.font_height;

        let mut menu_pos = 0;

        let wait_move_ticks: u32 = 100;
        let mut finished = false;
        self.menu.quit_menu = false;
        let mut need_update = true;
        while !finished {
            let handler = menu_entries[menu_pos].handler;
            let submenu = menu_entries[menu_pos].submenu;

            if need_update {
                let Data { menu, graphics, .. } = self;

                menu.menu_background
                    .as_mut()
                    .unwrap()
                    .blit(graphics.ne_screen.as_mut().unwrap());
                // print menu
                menu_entries.iter().enumerate().for_each(|(i, entry)| {
                    let arg = entry
                        .handler
                        .and_then(|handler| (handler)(self, MenuAction::INFO))
                        .unwrap_or(cstr!(""));

                    let mut full_name: [u8; 256] = [0; 256];
                    let mut cursor = Cursor::new(full_name.as_mut());
                    write!(
                        cursor,
                        "{}{}",
                        entry.name.as_ref().unwrap(),
                        arg.to_str().unwrap()
                    )
                    .unwrap();
                    let position = usize::try_from(cursor.position()).unwrap();
                    let mut ne_screen = self.graphics.ne_screen.take().unwrap();
                    self.put_string(
                        &mut ne_screen,
                        menu_x,
                        menu_y + i32::try_from(i).unwrap() * self.menu.font_height,
                        &full_name[..position],
                    );
                    self.graphics.ne_screen = Some(ne_screen);
                });
                self.put_influence(
                    influ_x,
                    menu_y + ((menu_pos as f64 - 0.5) * f64::from(self.menu.font_height)) as c_int,
                );

                #[cfg(not(target_os = "android"))]
                assert!(self.graphics.ne_screen.as_mut().unwrap().flip());

                need_update = false;
            }

            #[cfg(target_os = "android")]
            assert!(self.graphics.ne_screen.as_mut().unwrap().flip()); // for responsive input on Android, we need to run this every cycle

            let action = self.get_menu_action(250);

            let time_for_move =
                self.sdl.ticks_ms() - self.menu.show_menu_last_move_tick > wait_move_ticks;
            match action {
                MenuAction::BACK => {
                    finished = true;
                    self.wait_for_all_keys_released();
                }

                MenuAction::CLICK => {
                    if handler.is_none() && submenu.is_none() {
                        self.menu_item_selected_sound();
                        finished = true;
                    } else {
                        if let Some(handler) = handler {
                            self.wait_for_all_keys_released();
                            (handler)(self, action);
                        }

                        if let Some(submenu) = submenu {
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
                    self.menu.show_menu_last_move_tick = self.sdl.ticks_ms();
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
                    self.menu.show_menu_last_move_tick = self.sdl.ticks_ms();
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
                    self.menu.show_menu_last_move_tick = self.sdl.ticks_ms();
                    need_update = true;
                }

                _ => {}
            }

            if self.quit.get() || self.menu.quit_menu {
                finished = true;
            }

            self.sdl.delay_ms(1); // don't hog CPU
        }

        self.clear_graph_mem();
        self.sdl.cursor().show(); // reactivate mouse-cursor for game
                                  // Since we've faded out the whole scren, it can't hurt
                                  // to have the top status bar redrawn...

        self.graphics.banner_is_destroyed = true.into();
        self.vars.me.status = Status::Mobile as i32;

        while self.any_key_is_pressed_r()
        // wait for all key/controller-release
        {
            self.sdl.delay_ms(1);
        }
    }

    /// subroutine to display the current key-config and highlight current selection
    pub fn display_key_config(&mut self, selx: c_int, sely: c_int) {
        let current_font = self
            .b_font
            .current_font
            .as_ref()
            .unwrap()
            .ro(&self.font_owner);
        let startx = i32::from(self.vars.full_user_rect.x())
            + (1.2 * f32::from(self.vars.block_rect.width())) as i32;
        let starty = i32::from(self.vars.full_user_rect.y()) + font_height(current_font);
        let col1 = startx + (7.5 * f64::from(char_width(current_font, b'O'))) as i32;
        let col2 = col1 + (6.5 * f64::from(char_width(current_font, b'O'))) as i32;
        let col3 = col2 + (6.5 * f64::from(char_width(current_font, b'O'))) as i32;
        let lheight = font_height(
            self.global
                .font0_b_font
                .as_ref()
                .unwrap()
                .ro(&self.font_owner),
        ) + 2;

        let Data { menu, graphics, .. } = self;
        menu.menu_background
            .as_mut()
            .unwrap()
            .blit(graphics.ne_screen.as_mut().unwrap());

        #[cfg(feature = "gcw0")]
        PrintStringFont(
            self.graphics.ne_screen,
            Font0_BFont,
            col1,
            starty,
            format_args!("(RShldr to clear an entry)"),
        );

        let font0_b_font = self
            .global
            .font0_b_font
            .as_ref()
            .unwrap()
            .rw(&mut self.font_owner);
        #[cfg(not(feature = "gcw0"))]
        {
            print_string_font(
                self.graphics.ne_screen.as_mut().unwrap(),
                font0_b_font,
                col1,
                starty,
                format_args!("(RShldr to clear an entry)"),
            );
            print_string_font(
                self.graphics.ne_screen.as_mut().unwrap(),
                font0_b_font,
                col1,
                starty,
                format_args!("(Backspace to clear an entry)"),
            );
        }

        let mut posy = 1;
        print_string_font(
            self.graphics.ne_screen.as_mut().unwrap(),
            font0_b_font,
            startx,
            starty + (posy) * lheight,
            format_args!("Command"),
        );
        print_string_font(
            self.graphics.ne_screen.as_mut().unwrap(),
            font0_b_font,
            col1,
            starty + (posy) * lheight,
            format_args!("Key1"),
        );
        print_string_font(
            self.graphics.ne_screen.as_mut().unwrap(),
            font0_b_font,
            col2,
            starty + (posy) * lheight,
            format_args!("Key2"),
        );
        print_string_font(
            self.graphics.ne_screen.as_mut().unwrap(),
            font0_b_font,
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
            let &mut Data {
                graphics: Graphics {
                    ref mut ne_screen, ..
                },
                global:
                    Global {
                        ref font0_b_font,
                        ref font1_b_font,
                        ref font2_b_font,
                        ..
                    },
                ..
            } = self;

            let pos_font = |x, y| {
                if x != selx || i32::try_from(y).unwrap() != sely {
                    font1_b_font
                } else {
                    font2_b_font
                }
            };

            print_string_font(
                ne_screen.as_mut().unwrap(),
                font0_b_font.as_ref().unwrap().rw(&mut self.font_owner),
                startx,
                starty + (posy) * lheight,
                format_args!("{}", cmd_string),
            );
            print_string_font(
                ne_screen.as_mut().unwrap(),
                pos_font(1, 1 + i)
                    .as_ref()
                    .unwrap()
                    .rw(&mut self.font_owner),
                col1,
                starty + (posy) * lheight,
                format_args!(
                    "{}",
                    KEY_STRINGS[usize::try_from(self.input.key_cmds[i][0]).unwrap()]
                        .unwrap()
                        .to_str()
                        .unwrap()
                ),
            );
            print_string_font(
                ne_screen.as_mut().unwrap(),
                pos_font(2, 1 + i)
                    .as_ref()
                    .unwrap()
                    .rw(&mut self.font_owner),
                col2,
                starty + (posy) * lheight,
                format_args!(
                    "{}",
                    KEY_STRINGS[usize::try_from(self.input.key_cmds[i][1]).unwrap()]
                        .unwrap()
                        .to_str()
                        .unwrap()
                ),
            );
            print_string_font(
                ne_screen.as_mut().unwrap(),
                pos_font(3, 1 + i)
                    .as_ref()
                    .unwrap()
                    .rw(&mut self.font_owner),
                col3,
                starty + (posy) * lheight,
                format_args!(
                    "{}",
                    KEY_STRINGS[usize::try_from(self.input.key_cmds[i][2]).unwrap()]
                        .unwrap()
                        .to_str()
                        .unwrap()
                ),
            );
            posy += 1;
        }

        assert!(self.graphics.ne_screen.as_mut().unwrap().flip());
    }

    pub fn key_config_menu(&mut self) {
        let mut selx = 1;
        let mut sely = 1; // currently selected menu-position
        const WAIT_MOVE_TICKS: u32 = 100;

        let mut finished = false;
        while !finished {
            self.display_key_config(i32::try_from(selx).unwrap(), i32::try_from(sely).unwrap());

            let action = self.get_menu_action(250);
            let time_for_move =
                self.sdl.ticks_ms() - self.menu.key_config_menu_last_move_tick > WAIT_MOVE_TICKS;

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
                    self.menu.key_config_menu_last_move_tick = self.sdl.ticks_ms();
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
                    self.menu.key_config_menu_last_move_tick = self.sdl.ticks_ms();
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
                    self.menu.key_config_menu_last_move_tick = self.sdl.ticks_ms();
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
                    self.menu.key_config_menu_last_move_tick = self.sdl.ticks_ms();
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
                    self.menu.key_config_menu_last_move_tick = self.sdl.ticks_ms();
                }

                MenuAction::DELETE => {
                    self.input.key_cmds[sely - 1][selx - 1] = 0;
                    self.menu_item_selected_sound();
                }
                _ => {}
            }

            self.sdl.delay_ms(1);
        }
    }

    pub fn show_credits(&mut self) {
        let col2 = 2 * i32::from(self.vars.user_rect.width()) / 3;

        let menu_b_font = self
            .global
            .menu_b_font
            .as_ref()
            .unwrap()
            .ro(&self.font_owner);
        let h = font_height(menu_b_font);
        let em = char_width(menu_b_font, b'm');

        let screen = self.vars.screen_rect;
        self.graphics.ne_screen.as_mut().unwrap().clear_clip_rect();
        let image = Self::find_file_static(
            &self.global,
            &mut self.misc,
            CREDITS_PIC_FILE,
            Some(GRAPHICS_DIR_C),
            Themed::NoTheme as i32,
            Criticality::Critical as i32,
        )
        .unwrap();
        Self::display_image(self.sdl, &self.global, &mut self.graphics, image);
        self.make_grid_on_screen(Some(&screen));

        let oldfont = std::mem::replace(
            &mut self.b_font.current_font,
            self.global.font1_b_font.clone(),
        );

        let mut ne_screen = self.graphics.ne_screen.take().unwrap();
        self.printf_sdl(
            &mut ne_screen,
            i32::from(self.vars.get_user_center().x()) - 2 * em,
            h,
            format_args!("CREDITS\n"),
        );

        self.printf_sdl(&mut ne_screen, em, -1, format_args!("PROGRAMMING:"));
        self.printf_sdl(&mut ne_screen, col2, -1, format_args!("Johannes Prix\n"));
        self.printf_sdl(&mut ne_screen, -1, -1, format_args!("Reinhard Prix\n"));
        self.printf_sdl(&mut ne_screen, -1, -1, format_args!("\n"));

        self.printf_sdl(&mut ne_screen, em, -1, format_args!("ARTWORK:"));
        self.printf_sdl(&mut ne_screen, col2, -1, format_args!("Bastian Salmela\n"));
        self.printf_sdl(&mut ne_screen, -1, -1, format_args!("\n"));
        self.printf_sdl(&mut ne_screen, em, -1, format_args!("ADDITIONAL THEMES:\n"));
        self.printf_sdl(&mut ne_screen, 2 * em, -1, format_args!("Lanzz-theme"));
        self.printf_sdl(&mut ne_screen, col2, -1, format_args!("Lanzz\n"));
        self.printf_sdl(&mut ne_screen, 2 * em, -1, format_args!("Para90-theme"));
        self.printf_sdl(
            &mut ne_screen,
            col2,
            -1,
            format_args!("Andreas Wedemeyer\n"),
        );

        self.printf_sdl(&mut ne_screen, -1, -1, format_args!("\n"));
        self.printf_sdl(&mut ne_screen, em, -1, format_args!("C64 LEGACY MODS:\n"));

        self.printf_sdl(
            &mut ne_screen,
            2 * em,
            -1,
            format_args!("Green Beret, Sanxion, Uridium2"),
        );
        self.printf_sdl(&mut ne_screen, col2, -1, format_args!("#dreamfish/trsi\n"));

        self.printf_sdl(
            &mut ne_screen,
            2 * em,
            -1,
            format_args!("The last V8, Anarchy"),
        );
        self.printf_sdl(&mut ne_screen, col2, -1, format_args!("4-mat\n"));

        self.printf_sdl(&mut ne_screen, 2 * em, -1, format_args!("Tron"));
        self.printf_sdl(&mut ne_screen, col2, -1, format_args!("Kollaps\n"));

        self.printf_sdl(&mut ne_screen, 2 * em, -1, format_args!("Starpaws"));
        self.printf_sdl(&mut ne_screen, col2, -1, format_args!("Nashua\n"));

        self.printf_sdl(&mut ne_screen, 2 * em, -1, format_args!("Commando"));
        self.printf_sdl(&mut ne_screen, col2, -1, format_args!("Android"));

        assert!(self.graphics.ne_screen.as_mut().unwrap().flip());
        self.graphics.ne_screen = Some(ne_screen);
        self.wait_for_key_pressed();
        self.b_font.current_font = oldfont;
    }

    /// simple wrapper to ShowMenu() to provide the external entry point into the Level Editor menu
    pub fn show_level_editor_menu(&mut self) {
        self.menu.quit_level_editor = false;
        self.show_menu(&Self::LEVEL_EDITOR_MENU);
    }

    pub fn handle_configure_keys(&mut self, action: MenuAction) -> Option<&CStr> {
        if action == MenuAction::CLICK {
            self.menu_item_selected_sound();
            self.key_config_menu();
        }

        None
    }

    pub fn handle_highscores(&mut self, action: MenuAction) -> Option<&CStr> {
        if action == MenuAction::CLICK {
            self.menu_item_selected_sound();
            self.show_highscores();
        }
        None
    }

    pub fn handle_credits(&mut self, action: MenuAction) -> Option<&CStr> {
        if action == MenuAction::CLICK {
            self.menu_item_selected_sound();
            self.show_credits();
        }

        None
    }

    pub fn handle_le_save_ship(&mut self, action: MenuAction) -> Option<&CStr> {
        use std::io::Write;

        const SHIPNAME: &str = "Testship";
        self.menu.fname.clear();
        self.menu.fname.push_str(SHIPNAME);
        self.menu.fname.push_cstr(SHIP_EXT_C);

        if action == MenuAction::INFO {
            return Some(self.menu.fname.as_ref());
        }

        if action == MenuAction::CLICK {
            self.save_ship(SHIPNAME);
            let mut output = [0; 255];
            let mut cursor = Cursor::new(output.as_mut());
            write!(
                cursor,
                "Ship saved as '{}'",
                self.menu.fname.to_str().unwrap()
            )
            .unwrap();
            let position = usize::try_from(cursor.position()).unwrap();
            let mut ne_screen = self.graphics.ne_screen.take().unwrap();
            self.centered_put_string(
                &mut ne_screen,
                3 * font_height(
                    self.global
                        .menu_b_font
                        .as_ref()
                        .unwrap()
                        .ro(&self.font_owner),
                ),
                &output[..position],
            );
            assert!(ne_screen.flip());
            self.graphics.ne_screen = Some(ne_screen);
            self.wait_for_key_pressed();
            self.initiate_menu(false);
        }

        None
    }

    pub fn handle_le_name(&mut self, action: MenuAction) -> Option<&CStr> {
        if action == MenuAction::INFO {
            return Some(&cur_level!(self.main).levelname);
        }

        if action == MenuAction::CLICK {
            self.display_text(
                b"New level name: ",
                i32::from(self.vars.menu_rect.x()) - 2 * self.menu.font_height,
                i32::from(self.vars.menu_rect.y()) - 3 * self.menu.font_height,
                Some(self.vars.full_user_rect),
            );
            assert!(self.graphics.ne_screen.as_mut().unwrap().flip());

            cur_level!(mut self.main).levelname = self.get_string(15, 2).unwrap();
            self.initiate_menu(false);
        }

        None
    }

    pub fn handle_open_level_editor(&mut self, action: MenuAction) -> Option<&CStr> {
        if action == MenuAction::CLICK {
            self.menu_item_selected_sound();
            self.level_editor();
        }
        None
    }

    pub fn handle_le_exit(&mut self, action: MenuAction) -> Option<&CStr> {
        if action == MenuAction::CLICK {
            self.menu_item_selected_sound();
            self.menu.quit_level_editor = true;
            self.menu.quit_menu = true;
        }
        None
    }

    pub fn handle_le_level_number(&mut self, action: MenuAction) -> Option<&CStr> {
        use std::fmt::Write;

        let cur_level = self.main.cur_level();
        if action == MenuAction::INFO {
            self.menu.le_level_number_buf.clear();
            write!(self.menu.le_level_number_buf, "{}", cur_level.levelnum).unwrap();
            return Some(self.menu.le_level_number_buf.as_ref());
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
        self.switch_background_music_to(Some(BYCOLOR));
        self.initiate_menu(false);

        None
    }

    pub fn handle_le_color(&mut self, action: MenuAction) -> Option<&CStr> {
        let cur_level = cur_level!(mut self.main);
        if action == MenuAction::INFO {
            return Some(COLOR_NAMES[usize::try_from(cur_level.color).unwrap()]);
        }
        MenuChange {
            sound_on: self.main.sound_on,
            sdl: self.sdl,
            sound: self.sound.as_ref().unwrap(),
            action,
            val: &mut cur_level.color,
            step: 1,
            min_value: 0,
            max_value: c_int::try_from(COLOR_NAMES.len()).unwrap() - 1,
        }
        .run();
        self.switch_background_music_to(Some(BYCOLOR));
        self.initiate_menu(false);

        None
    }

    pub fn handle_le_size_x(&mut self, action: MenuAction) -> Option<&CStr> {
        use std::fmt::Write;

        let cur_level = cur_level!(mut self.main);
        if action == MenuAction::INFO {
            self.menu.le_size_x_buf.clear();
            write!(self.menu.le_size_x_buf, "{}", cur_level.xlen).unwrap();
            return Some(self.menu.le_size_x_buf.as_ref());
        }

        let oldxlen = cur_level.xlen;
        MenuChange {
            sound_on: self.main.sound_on,
            sdl: self.sdl,
            sound: self.sound.as_ref().unwrap(),
            action,
            val: &mut cur_level.xlen,
            step: 1,
            min_value: 0,
            max_value: i32::try_from(MAX_MAP_COLS).unwrap() - 1,
        }
        .run();
        let newmem = usize::try_from(cur_level.xlen).unwrap();
        // adjust memory sizes for new value
        for row in 0..usize::try_from(cur_level.ylen).unwrap() {
            cur_level.map[row].resize(newmem, MapTile::Void);
            if cur_level.xlen > oldxlen {
                // fill new map area with VOID
                cur_level.map[row][usize::try_from(cur_level.xlen - 1).unwrap()] = MapTile::Void;
            }
        }
        self.initiate_menu(false);
        None
    }

    pub fn handle_le_size_y(&mut self, action: MenuAction) -> Option<&CStr> {
        use std::{cmp::Ordering, fmt::Write};

        let cur_level = cur_level!(mut self.main);
        if action == MenuAction::INFO {
            self.menu.le_size_y_buf.clear();
            write!(self.menu.le_size_y_buf, "{}", cur_level.ylen).unwrap();
            return Some(self.menu.le_size_y_buf.as_ref());
        }

        let oldylen = cur_level.ylen;
        MenuChange {
            sound_on: self.main.sound_on,
            sdl: self.sdl,
            sound: self.sound.as_ref().unwrap(),
            action,
            val: &mut cur_level.ylen,
            step: 1,
            min_value: 0,
            max_value: i32::try_from(MAX_MAP_ROWS - 1).unwrap(),
        }
        .run();
        match oldylen.cmp(&cur_level.ylen) {
            Ordering::Greater => cur_level.map[usize::try_from(oldylen - 1).unwrap()].clear(),
            Ordering::Less => cur_level.map[usize::try_from(cur_level.ylen - 1).unwrap()]
                .resize(usize::try_from(cur_level.xlen).unwrap(), MapTile::Void),
            Ordering::Equal => {}
        }

        self.initiate_menu(false);
        None
    }

    pub fn handle_strictly_classic(&mut self, action: MenuAction) -> Option<&CStr> {
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

        None
    }

    pub fn handle_window_type(&mut self, action: MenuAction) -> Option<&CStr> {
        if action == MenuAction::INFO {
            let s = if self.global.game_config.full_user_rect != 0 {
                cstr!("Full")
            } else {
                cstr!("Classic")
            };

            return Some(s);
        }

        if action == MenuAction::CLICK || action == MenuAction::LEFT || action == MenuAction::RIGHT
        {
            self.flip_toggle(|data| &mut data.global.game_config.full_user_rect);
            if self.global.game_config.full_user_rect != 0 {
                self.vars.user_rect = self.vars.full_user_rect;
            } else {
                self.vars.user_rect = self.vars.classic_user_rect;
            }

            self.initiate_menu(false);
        }
        None
    }

    pub fn handle_theme(&mut self, action: MenuAction) -> Option<&CStr> {
        if action == MenuAction::INFO {
            return Some(
                &*self.graphics.all_themes.theme_name
                    [usize::try_from(self.graphics.all_themes.cur_tnum).unwrap()],
            );
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

        None
    }

    pub fn handle_droid_talk(&mut self, action: MenuAction) -> Option<&CStr> {
        if action == MenuAction::INFO {
            return Some(is_toggle_on(self.global.game_config.droid_talk));
        }
        if action == MenuAction::CLICK || action == MenuAction::LEFT || action == MenuAction::RIGHT
        {
            self.flip_toggle(|data| &mut data.global.game_config.droid_talk);
        }
        None
    }

    pub fn handle_all_map_visible(&mut self, action: MenuAction) -> Option<&CStr> {
        if action == MenuAction::INFO {
            return Some(is_toggle_on(self.global.game_config.all_map_visible));
        }
        if action == MenuAction::CLICK || action == MenuAction::LEFT || action == MenuAction::RIGHT
        {
            self.flip_toggle(|data| &mut data.global.game_config.all_map_visible);
            self.initiate_menu(false);
        }
        None
    }

    pub fn handle_show_decals(&mut self, action: MenuAction) -> Option<&CStr> {
        if action == MenuAction::INFO {
            return Some(is_toggle_on(self.global.game_config.show_decals));
        }
        if action == MenuAction::CLICK || action == MenuAction::LEFT || action == MenuAction::RIGHT
        {
            self.flip_toggle(|data| &mut data.global.game_config.show_decals);
            self.initiate_menu(false);
        }
        None
    }

    pub fn handle_transfer_is_activate(&mut self, action: MenuAction) -> Option<&CStr> {
        if action == MenuAction::INFO {
            return Some(is_toggle_on(self.global.game_config.takeover_activates));
        }
        if action == MenuAction::CLICK || action == MenuAction::LEFT || action == MenuAction::RIGHT
        {
            self.flip_toggle(|data| &mut data.global.game_config.takeover_activates);
        }
        None
    }

    pub fn handle_fire_is_transfer(&mut self, action: MenuAction) -> Option<&CStr> {
        if action == MenuAction::INFO {
            return Some(is_toggle_on(self.global.game_config.fire_hold_takeover));
        }
        if action == MenuAction::CLICK || action == MenuAction::LEFT || action == MenuAction::RIGHT
        {
            self.flip_toggle(|data| &mut data.global.game_config.fire_hold_takeover);
        }
        None
    }

    pub fn handle_empty_level_speedup(&mut self, action: MenuAction) -> Option<&CStr> {
        use std::fmt::Write;

        if action == MenuAction::INFO {
            self.menu.empty_level_speedup_buf.clear();
            write!(
                self.menu.empty_level_speedup_buf,
                "{:3.1}",
                f64::from(self.global.game_config.empty_level_speedup)
            )
            .unwrap();
            return Some(self.menu.empty_level_speedup_buf.as_ref());
        }

        let mut f = self.global.game_config.empty_level_speedup;
        self.menu_change_float(action, &mut f, 0.1, 0.5, 2.0);
        self.global.game_config.empty_level_speedup = f;
        None
    }

    pub fn handle_music_volume(&mut self, action: MenuAction) -> Option<&CStr> {
        use std::fmt::Write;

        if action == MenuAction::INFO {
            self.menu.music_volume_buf.clear();
            write!(
                self.menu.music_volume_buf,
                "{:4.2}",
                f64::from(self.global.game_config.current_bg_music_volume)
            )
            .unwrap();
            return Some(self.menu.music_volume_buf.as_ref());
        }

        let mut f = self.global.game_config.current_bg_music_volume;
        self.menu_change_float(action, &mut f, 0.05, 0., 1.);
        self.global.game_config.current_bg_music_volume = f;

        self.set_bg_music_volume(self.global.game_config.current_bg_music_volume);
        None
    }

    pub fn handle_sound_volume(&mut self, action: MenuAction) -> Option<&CStr> {
        use std::fmt::Write;

        if action == MenuAction::INFO {
            self.menu.sound_volume_buf.clear();
            write!(
                self.menu.sound_volume_buf,
                "{:4.2}",
                f64::from(self.global.game_config.current_sound_fx_volume)
            )
            .unwrap();
            return Some(self.menu.sound_volume_buf.as_ref());
        }

        let mut f = self.global.game_config.current_sound_fx_volume;
        self.menu_change_float(action, &mut f, 0.05, 0., 1.);
        self.global.game_config.current_sound_fx_volume = f;
        let Self {
            sound,
            sdl,
            global,
            main,
            ..
        } = &*self;
        let sound = sound.as_ref().unwrap();
        let mixer = sdl.mixer.get().unwrap();
        sound.set_sound_f_x_volume(main, mixer, global.game_config.current_sound_fx_volume);
        None
    }

    pub fn handle_fullscreen(&mut self, action: MenuAction) -> Option<&CStr> {
        if action == MenuAction::INFO {
            return Some(is_toggle_on(self.global.game_config.use_fullscreen));
        }
        if action == MenuAction::CLICK || action == MenuAction::LEFT || action == MenuAction::RIGHT
        {
            self.toggle_fullscreen();
            self.menu_item_selected_sound();
        }
        None
    }

    pub fn handle_show_position(&mut self, action: MenuAction) -> Option<&CStr> {
        if action == MenuAction::INFO {
            return Some(is_toggle_on(self.global.game_config.draw_position));
        }
        if action == MenuAction::CLICK || action == MenuAction::LEFT || action == MenuAction::RIGHT
        {
            self.flip_toggle(|data| &mut data.global.game_config.draw_position);
            self.initiate_menu(false);
        }
        None
    }

    pub fn handle_show_framerate(&mut self, action: MenuAction) -> Option<&CStr> {
        if action == MenuAction::INFO {
            return Some(is_toggle_on(self.global.game_config.draw_framerate));
        }
        if action == MenuAction::CLICK || action == MenuAction::LEFT || action == MenuAction::RIGHT
        {
            self.flip_toggle(|data| &mut data.global.game_config.draw_framerate);
            self.initiate_menu(false);
        }
        None
    }

    pub fn handle_show_energy(&mut self, action: MenuAction) -> Option<&CStr> {
        if action == MenuAction::INFO {
            return Some(is_toggle_on(self.global.game_config.draw_energy));
        }
        if action == MenuAction::CLICK || action == MenuAction::LEFT || action == MenuAction::RIGHT
        {
            self.flip_toggle(|data| &mut data.global.game_config.draw_energy);
            self.initiate_menu(false);
        }
        None
    }

    #[inline]
    fn menu_change<T>(&self, action: MenuAction, val: &mut T, step: T, min_value: T, max_value: T)
    where
        T: PartialOrd + AddAssign + SubAssign,
    {
        MenuChange {
            sound_on: self.main.sound_on,
            sdl: self.sdl,
            sound: self.sound.as_ref().unwrap(),
            action,
            val,
            step,
            min_value,
            max_value,
        }
        .run()
    }

    pub fn menu_change_float(
        &self,
        action: MenuAction,
        val: &mut c_float,
        step: c_float,
        min_value: c_float,
        max_value: c_float,
    ) {
        self.menu_change(action, val, step, min_value, max_value)
    }

    pub fn menu_change_int(
        &self,
        action: MenuAction,
        val: &mut c_int,
        step: c_int,
        min_value: c_int,
        max_value: c_int,
    ) {
        self.menu_change(action, val, step, min_value, max_value)
    }

    pub fn flip_toggle<F>(&mut self, mut get_toggle: F)
    where
        F: for<'a> FnMut(&'a mut Data) -> &'a mut c_int,
    {
        self.menu_item_selected_sound();
        let toggle = get_toggle(self);
        *toggle = !*toggle;
    }

    pub fn set_theme(&mut self, theme_index: c_int) {
        assert!(theme_index >= 0 && theme_index < self.graphics.all_themes.num_themes);

        self.graphics.all_themes.cur_tnum = theme_index;
        self.global.game_config.theme_name.set(
            &self.graphics.all_themes.theme_name
                [usize::try_from(self.graphics.all_themes.cur_tnum).unwrap()],
        );
        self.init_pictures();
    }
}

pub fn is_toggle_on(toggle: c_int) -> &'static CStr {
    if toggle != 0 {
        cstr!("YES")
    } else {
        cstr!("NO")
    }
}

#[must_use]
struct MenuChange<'a, 'b, T> {
    sound_on: c_int,
    sdl: &'a Sdl,
    sound: &'a Sound<'b>,
    action: MenuAction,
    val: &'a mut T,
    step: T,
    min_value: T,
    max_value: T,
}

impl<T> MenuChange<'_, '_, T>
where
    T: PartialOrd + AddAssign + SubAssign,
{
    fn run(self) {
        let Self {
            sound_on,
            sdl,
            sound,
            action,
            val,
            step,
            min_value,
            max_value,
        } = self;

        if action == MenuAction::RIGHT && *val < max_value {
            Data::move_lift_sound_static(sound_on, sdl, sound);
            *val += step;
            if *val > max_value {
                *val = max_value;
            }
        } else if action == MenuAction::LEFT && *val > min_value {
            Data::move_lift_sound_static(sound_on, sdl, sound);
            *val -= step;
            if *val <= min_value {
                *val = min_value;
            }
        }
    }
}
