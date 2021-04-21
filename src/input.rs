#[cfg(target_os = "android")]
use crate::graphics::NE_SCREEN;
use crate::{
    defs::{
        self, alt_pressed, ctrl_pressed, down_pressed, get_user_center, left_pressed,
        right_pressed, shift_pressed, up_pressed, Cmds, MenuAction, PointerStates,
    },
    graphics::{take_screenshot, toggle_fullscreen},
    menu::{cheatmenu, handle_quit_game, show_main_menu},
    misc::terminate,
    structs::Point,
    Data,
};

use cstr::cstr;
use log::info;
#[cfg(feature = "gcw0")]
use sdl::event::ll::{SDLK_BACKSPACE, SDLK_LALT, SDLK_LCTRL, SDLK_TAB};
#[cfg(not(feature = "gcw0"))]
use sdl::event::ll::{SDLK_F12, SDLK_PAUSE, SDLK_RSHIFT};
use sdl::{
    event::{
        ll::{
            SDLMod, SDL_Event, SDL_PollEvent, SDLK_DOWN, SDLK_ESCAPE, SDLK_LEFT, SDLK_RETURN,
            SDLK_RIGHT, SDLK_SPACE, SDLK_UP, SDL_ENABLE, SDL_JOYAXISMOTION, SDL_JOYBUTTONDOWN,
            SDL_JOYBUTTONUP, SDL_KEYDOWN, SDL_KEYUP, SDL_MOUSEBUTTONDOWN, SDL_MOUSEBUTTONUP,
            SDL_MOUSEMOTION, SDL_QUIT,
        },
        MouseState,
    },
    joy::{
        get_joystick_name,
        ll::{
            SDL_Joystick, SDL_JoystickEventState, SDL_JoystickNumAxes, SDL_JoystickNumButtons,
            SDL_JoystickOpen, SDL_NumJoysticks,
        },
    },
    ll::{SDL_GetTicks, SDL_InitSubSystem, SDL_INIT_JOYSTICK},
};
use std::{
    convert::{identity, TryFrom},
    os::raw::{c_char, c_int},
    ptr::{null, null_mut},
};

#[cfg(target_os = "android")]
use sdl::video::ll::SDL_Flip;

extern "C" {
    pub fn SDL_Delay(ms: u32);
}

pub static mut SHOW_CURSOR: bool = false;
pub static mut WHEEL_UP_EVENTS: c_int = 0;
pub static mut WHEEL_DOWN_EVENTS: c_int = 0;
pub static mut LAST_MOUSE_EVENT: u32 = 0;
pub static mut CURRENT_MODIFIERS: SDLMod = 0;
pub static mut INPUT_STATE: [c_int; PointerStates::Last as usize] =
    [0; PointerStates::Last as usize];
pub static mut EVENT: SDL_Event = SDL_Event { data: [0; 24] };
pub static mut JOY_SENSITIVITY: c_int = 0;
pub static mut INPUT_AXIS: Point = Point { x: 0, y: 0 }; /* joystick (and mouse) axis values */
pub static mut JOY: *mut SDL_Joystick = null_mut();
pub static mut JOY_NUM_AXES: i32 = 0; /* number of joystick axes */
pub static mut AXIS_IS_ACTIVE: i32 = 0; /* is firing to use axis-values or not */

#[cfg(feature = "gcw0")]
pub static mut key_cmds: [[c_int; 3]; Cmds::Last as usize] = [
    [SDLK_UP as c_int, PointerStates::JoyUp as c_int, 0], // CMD_UP
    [SDLK_DOWN as c_int, PointerStates::JoyDown as c_int, 0], // CMD_DOWN
    [SDLK_LEFT as c_int, PointerStates::JoyLeft as c_int, 0], // CMD_LEFT
    [SDLK_RIGHT as c_int, PointerStates::JoyRight as c_int, 0], // CMD_RIGHT
    [SDLK_SPACE as c_int, SDLK_LCTRL as c_int, 0],        // CMD_FIRE
    [SDLK_LALT as c_int, PointerStates::JoyButton2 as c_int, 0], // CMD_ACTIVATE
    [SDLK_BACKSPACE as c_int, SDLK_TAB as c_int, 0],      // CMD_TAKEOVER
    [0, 0, 0],                                            // CMD_QUIT,
    [SDLK_RETURN as c_int, 0, 0],                         // CMD_PAUSE,
    [0, 0, 0],                                            // CMD_SCREENSHOT
    [0, 0, 0],                                            // CMD_FULLSCREEN,
    [SDLK_ESCAPE as c_int, PointerStates::JoyButton4 as c_int, 0], // CMD_MENU,
    [
        SDLK_ESCAPE as c_int,
        PointerStates::JoyButton2 as c_int,
        PointerStates::MouseButton2 as c_int,
    ], // CMD_BACK
];

#[cfg(not(feature = "gcw0"))]
pub static mut KEY_CMDS: [[c_int; 3]; Cmds::Last as usize] = [
    [
        SDLK_UP as c_int,
        PointerStates::JoyUp as c_int,
        b'w' as c_int,
    ], // CMD_UP
    [
        SDLK_DOWN as c_int,
        PointerStates::JoyDown as c_int,
        b's' as c_int,
    ], // CMD_DOWN
    [
        SDLK_LEFT as c_int,
        PointerStates::JoyLeft as c_int,
        b'a' as c_int,
    ], // CMD_LEFT
    [
        SDLK_RIGHT as c_int,
        PointerStates::JoyRight as c_int,
        b'd' as c_int,
    ], // CMD_RIGHT
    [
        SDLK_SPACE as c_int,
        PointerStates::JoyButton1 as c_int,
        PointerStates::MouseButton1 as c_int,
    ], // CMD_FIRE
    [SDLK_RETURN as c_int, SDLK_RSHIFT as c_int, b'e' as c_int], // CMD_ACTIVATE
    [
        SDLK_SPACE as c_int,
        PointerStates::JoyButton2 as c_int,
        PointerStates::MouseButton2 as c_int,
    ], // CMD_TAKEOVER
    [b'q' as c_int, 0, 0],                                       // CMD_QUIT,
    [SDLK_PAUSE as c_int, b'p' as c_int, 0],                     // CMD_PAUSE,
    [SDLK_F12 as c_int, 0, 0],                                   // CMD_SCREENSHOT
    [b'f' as c_int, 0, 0],                                       // CMD_FULLSCREEN,
    [SDLK_ESCAPE as c_int, PointerStates::JoyButton4 as c_int, 0], // CMD_MENU,
    [
        SDLK_ESCAPE as c_int,
        PointerStates::JoyButton2 as c_int,
        PointerStates::MouseButton2 as c_int,
    ], // CMD_BACK
];

pub static mut CMD_STRINGS: [*const c_char; Cmds::Last as usize] = [
    cstr!("UP").as_ptr(),
    cstr!("DOWN").as_ptr(),
    cstr!("LEFT").as_ptr(),
    cstr!("RIGHT").as_ptr(),
    cstr!("FIRE").as_ptr(),
    cstr!("ACTIVATE").as_ptr(),
    cstr!("TAKEOVER").as_ptr(),
    cstr!("QUIT").as_ptr(),
    cstr!("PAUSE").as_ptr(),
    cstr!("SCREENSHOT").as_ptr(),
    cstr!("FULLSCREEN").as_ptr(),
    cstr!("MENU").as_ptr(),
    cstr!("BACK").as_ptr(),
];

pub const CURSOR_KEEP_VISIBLE: u32 = 3000; // ticks to keep mouse-cursor visible without mouse-input
pub static mut KEYSTR: [*const c_char; PointerStates::Last as usize] =
    [null(); PointerStates::Last as usize];

/// Check if any keys have been 'freshly' pressed. If yes, return key-code, otherwise 0.
pub fn wait_for_key_pressed() -> c_int {
    loop {
        match any_key_just_pressed() {
            0 => unsafe { SDL_Delay(1) },
            key => break key,
        }
    }
}

pub fn any_key_just_pressed() -> c_int {
    #[cfg(target_os = "android")]
    unsafe {
        SDL_Flip(NE_SCREEN)
    };

    unsafe {
        update_input();
    }

    let pressed_key = (0..PointerStates::Last as c_int)
        .map(|key| (key, unsafe { &mut INPUT_STATE[key as usize] }))
        .find(|(_, key_flags)| is_just_pressed(**key_flags));

    match pressed_key {
        Some((key, key_flags)) => {
            clear_fresh(key_flags);
            key
        }
        None => 0,
    }
}

const FRESH_BIT: c_int = 0x01 << 8;
const PRESSED: c_int = true as c_int | FRESH_BIT;
const RELEASED: c_int = false as c_int | FRESH_BIT;

const fn is_just_pressed(key_flags: c_int) -> bool {
    key_flags & PRESSED == PRESSED
}

fn clear_fresh(key_flags: &mut c_int) {
    *key_flags &= !FRESH_BIT;
}

pub unsafe fn update_input() -> c_int {
    // switch mouse-cursor visibility as a function of time of last activity
    if SDL_GetTicks() - LAST_MOUSE_EVENT > CURSOR_KEEP_VISIBLE {
        SHOW_CURSOR = false;
    } else {
        SHOW_CURSOR = true;
    }

    while SDL_PollEvent(&mut EVENT) != 0 {
        match (*EVENT._type()).into() {
            SDL_QUIT => {
                info!("User requested termination, terminating.");
                terminate(0);
            }

            SDL_KEYDOWN => {
                let key = &*EVENT.key();
                CURRENT_MODIFIERS = key.keysym._mod;
                INPUT_STATE[usize::try_from(key.keysym.sym).unwrap()] = PRESSED;
                #[cfg(feature = "gcw0")]
                if input_axis.x != 0 || input_axis.y != 0 {
                    axis_is_active = true.into(); // 4 GCW-0 ; breaks cursor keys after axis has been active...
                }
            }
            SDL_KEYUP => {
                let key = &*EVENT.key();
                CURRENT_MODIFIERS = key.keysym._mod;
                INPUT_STATE[usize::try_from(key.keysym.sym).unwrap()] = RELEASED;
                #[cfg(feature = "gcw0")]
                {
                    axis_is_active = false.into();
                }
            }

            SDL_JOYAXISMOTION => {
                let jaxis = &*EVENT.jaxis();
                let axis = jaxis.axis;
                if axis == 0 || ((JOY_NUM_AXES >= 5) && (axis == 3))
                /* x-axis */
                {
                    INPUT_AXIS.x = jaxis.value.into();

                    // this is a bit tricky, because we want to allow direction keys
                    // to be soft-released. When mapping the joystick->keyboard, we
                    // therefore have to make sure that this mapping only occurs when
                    // and actual _change_ of the joystick-direction ('digital') occurs
                    // so that it behaves like "set"/"release"
                    if JOY_SENSITIVITY * i32::from(jaxis.value) > 10000 {
                        /* about half tilted */
                        INPUT_STATE[PointerStates::JoyRight as usize] = PRESSED;
                        INPUT_STATE[PointerStates::JoyLeft as usize] = false.into();
                    } else if JOY_SENSITIVITY * i32::from(jaxis.value) < -10000 {
                        INPUT_STATE[PointerStates::JoyLeft as usize] = PRESSED;
                        INPUT_STATE[PointerStates::JoyRight as usize] = false.into();
                    } else {
                        INPUT_STATE[PointerStates::JoyLeft as usize] = false.into();
                        INPUT_STATE[PointerStates::JoyRight as usize] = false.into();
                    }
                } else if (axis == 1) || ((JOY_NUM_AXES >= 5) && (axis == 4)) {
                    /* y-axis */
                    INPUT_AXIS.y = jaxis.value.into();

                    if JOY_SENSITIVITY * i32::from(jaxis.value) > 10000 {
                        INPUT_STATE[PointerStates::JoyDown as usize] = PRESSED;
                        INPUT_STATE[PointerStates::JoyUp as usize] = false.into();
                    } else if JOY_SENSITIVITY * i32::from(jaxis.value) < -10000 {
                        INPUT_STATE[PointerStates::JoyUp as usize] = PRESSED;
                        INPUT_STATE[PointerStates::JoyDown as usize] = false.into();
                    } else {
                        INPUT_STATE[PointerStates::JoyUp as usize] = false.into();
                        INPUT_STATE[PointerStates::JoyDown as usize] = false.into();
                    }
                }
            }

            SDL_JOYBUTTONDOWN => {
                let jbutton = &*EVENT.jbutton();
                // first button
                if jbutton.button == 0 {
                    INPUT_STATE[PointerStates::JoyButton1 as usize] = PRESSED;
                }
                // second button
                else if jbutton.button == 1 {
                    INPUT_STATE[PointerStates::JoyButton2 as usize] = PRESSED;
                }
                // and third button
                else if jbutton.button == 2 {
                    INPUT_STATE[PointerStates::JoyButton3 as usize] = PRESSED;
                }
                // and fourth button
                else if jbutton.button == 3 {
                    INPUT_STATE[PointerStates::JoyButton4 as usize] = PRESSED;
                }

                AXIS_IS_ACTIVE = true.into();
            }

            SDL_JOYBUTTONUP => {
                let jbutton = &*EVENT.jbutton();
                // first button
                if jbutton.button == 0 {
                    INPUT_STATE[PointerStates::JoyButton1 as usize] = false.into();
                }
                // second button
                else if jbutton.button == 1 {
                    INPUT_STATE[PointerStates::JoyButton2 as usize] = false.into();
                }
                // and third button
                else if jbutton.button == 2 {
                    INPUT_STATE[PointerStates::JoyButton3 as usize] = false.into();
                }
                // and fourth button
                else if jbutton.button == 3 {
                    INPUT_STATE[PointerStates::JoyButton4 as usize] = false.into();
                }

                AXIS_IS_ACTIVE = false.into();
            }

            SDL_MOUSEMOTION => {
                let button = &*EVENT.button();
                let user_center = get_user_center();
                INPUT_AXIS.x = i32::from(button.x) - i32::from(user_center.x) + 16;
                INPUT_AXIS.y = i32::from(button.y) - i32::from(user_center.y) + 16;

                LAST_MOUSE_EVENT = SDL_GetTicks();
            }

            /* Mouse control */
            SDL_MOUSEBUTTONDOWN => {
                let button = &*EVENT.button();
                if button.button == MouseState::Left as u8 {
                    INPUT_STATE[PointerStates::MouseButton1 as usize] = PRESSED;
                    AXIS_IS_ACTIVE = true.into();
                }

                if button.button == MouseState::Right as u8 {
                    INPUT_STATE[PointerStates::MouseButton2 as usize] = PRESSED;
                }

                if button.button == MouseState::Middle as u8 {
                    INPUT_STATE[PointerStates::MouseButton3 as usize] = PRESSED;
                }

                // wheel events are immediately released, so we rather
                // count the number of not yet read-out events
                if button.button == MouseState::WheelUp as u8 {
                    WHEEL_UP_EVENTS += 1;
                }

                if button.button == MouseState::WheelDown as u8 {
                    WHEEL_DOWN_EVENTS += 1;
                }

                LAST_MOUSE_EVENT = SDL_GetTicks();
            }

            SDL_MOUSEBUTTONUP => {
                let button = &*EVENT.button();
                if button.button == MouseState::Left as u8 {
                    INPUT_STATE[PointerStates::MouseButton1 as usize] = false.into();
                    AXIS_IS_ACTIVE = false.into();
                }

                if button.button == MouseState::Right as u8 {
                    INPUT_STATE[PointerStates::MouseButton2 as usize] = false.into();
                }

                if button.button == MouseState::Middle as u8 {
                    INPUT_STATE[PointerStates::MouseButton3 as usize] = false.into();
                }
            }

            _ => break,
        }
    }

    0
}

pub unsafe fn key_is_pressed(key: c_int) -> bool {
    update_input();

    (INPUT_STATE[usize::try_from(key).unwrap()] & PRESSED) == PRESSED
}

/// Does the same as KeyIsPressed, but automatically releases the key as well..
pub unsafe fn key_is_pressed_r(key: c_int) -> bool {
    let ret = key_is_pressed(key);

    release_key(key);
    ret
}

pub unsafe fn release_key(key: c_int) {
    INPUT_STATE[usize::try_from(key).unwrap()] = false.into();
}

pub unsafe fn wheel_up_pressed() -> bool {
    update_input();
    if WHEEL_UP_EVENTS != 0 {
        WHEEL_UP_EVENTS -= 1;
        true
    } else {
        false
    }
}

pub unsafe fn wheel_down_pressed() -> bool {
    update_input();
    if WHEEL_DOWN_EVENTS != 0 {
        WHEEL_DOWN_EVENTS -= 1;
        true
    } else {
        false
    }
}

pub unsafe fn cmd_is_active(cmd: Cmds) -> bool {
    let cmd = cmd as usize;
    key_is_pressed(KEY_CMDS[cmd][0])
        || key_is_pressed(KEY_CMDS[cmd][1])
        || key_is_pressed(KEY_CMDS[cmd][2])
}

/// the same but release the keys: use only for menus!
pub unsafe fn cmd_is_active_r(cmd: Cmds) -> bool {
    let cmd = cmd as usize;

    let c1 = key_is_pressed_r(KEY_CMDS[cmd][0]);
    let c2 = key_is_pressed_r(KEY_CMDS[cmd][1]);
    let c3 = key_is_pressed_r(KEY_CMDS[cmd][2]);

    c1 || c2 || c3
}

pub unsafe fn wait_for_all_keys_released() {
    while any_key_is_pressed_r() {
        SDL_Delay(1);
    }
    reset_mouse_wheel();
}

pub unsafe fn any_key_is_pressed_r() -> bool {
    #[cfg(target_os = "android")]
    SDL_Flip(NE_SCREEN); // make sure we keep updating screen to read out Android inputs

    #[cfg(not(target_os = "android"))]
    update_input();

    for state in &mut INPUT_STATE {
        if (*state & PRESSED) != 0 {
            *state = 0;
            return true;
        }
    }
    false
}

// forget the wheel-counters

pub unsafe fn reset_mouse_wheel() {
    WHEEL_UP_EVENTS = 0;
    WHEEL_DOWN_EVENTS = 0;
}

pub unsafe fn mod_is_pressed(sdl_mod: SDLMod) -> bool {
    update_input();
    (CURRENT_MODIFIERS & sdl_mod) != 0
}

pub unsafe fn no_direction_pressed() -> bool {
    !((AXIS_IS_ACTIVE != 0 && (INPUT_AXIS.x != 0 || INPUT_AXIS.y != 0))
        || down_pressed()
        || up_pressed()
        || left_pressed()
        || right_pressed())
}

impl Data {
    pub unsafe fn react_to_special_keys(&mut self) {
        if cmd_is_active_r(Cmds::Quit) {
            handle_quit_game(MenuAction::CLICK);
        }

        if cmd_is_active_r(Cmds::Pause) {
            self.pause();
        }

        if cmd_is_active(Cmds::Screenshot) {
            take_screenshot();
        }

        if cmd_is_active_r(Cmds::Fullscreen) {
            toggle_fullscreen();
        }

        if cmd_is_active_r(Cmds::Menu) {
            show_main_menu();
        }

        // this stuff remains hardcoded to keys
        if key_is_pressed_r(b'c'.into()) && alt_pressed() && ctrl_pressed() && shift_pressed() {
            cheatmenu();
        }
    }
}

pub unsafe fn init_joy() {
    if SDL_InitSubSystem(SDL_INIT_JOYSTICK) == -1 {
        eprintln!("Couldn't initialize SDL-Joystick: {}", sdl::get_error(),);
        terminate(defs::ERR.into());
    } else {
        info!("SDL Joystick initialisation successful.");
    }

    let num_joy = SDL_NumJoysticks();
    info!("{} Joysticks found!\n", num_joy);

    if num_joy > 0 {
        JOY = SDL_JoystickOpen(0);
    }

    if !JOY.is_null() {
        info!(
            "Identifier: {}",
            get_joystick_name(0).unwrap_or_else(identity)
        );

        JOY_NUM_AXES = SDL_JoystickNumAxes(JOY);
        info!("Number of Axes: {}", JOY_NUM_AXES);
        info!("Number of Buttons: {}", SDL_JoystickNumButtons(JOY));

        /* aktivate Joystick event handling */
        SDL_JoystickEventState(SDL_ENABLE);
    } else {
        JOY = null_mut(); /* signals that no yoystick is present */
    }
}

pub unsafe fn init_keystr() {
    use sdl::keysym::*;

    KEYSTR[0] = cstr!("NONE").as_ptr(); // Empty bind will otherwise crash on some platforms - also, we choose "NONE" as a placeholder...
    #[cfg(feature = "gcw0")]
    {
        // The GCW0 may change to joystick input altogether in the future - which will make these ifdefs unnecessary, I hope...
        keystr[SDLK_BACKSPACE as usize] = cstr!("RSldr").as_ptr();
        keystr[SDLK_TAB as usize] = cstr!("LSldr").as_ptr();
        keystr[SDLK_RETURN as usize] = cstr!("Start").as_ptr();
        keystr[SDLK_SPACE as usize] = cstr!("Y").as_ptr();
        keystr[SDLK_ESCAPE as usize] = cstr!("Select").as_ptr();
    }

    #[cfg(not(feature = "gcw0"))]
    {
        KEYSTR[SDLK_BACKSPACE as usize] = cstr!("BS").as_ptr();
        KEYSTR[SDLK_TAB as usize] = cstr!("Tab").as_ptr();
        KEYSTR[SDLK_RETURN as usize] = cstr!("Return").as_ptr();
        KEYSTR[SDLK_SPACE as usize] = cstr!("Space").as_ptr();
        KEYSTR[SDLK_ESCAPE as usize] = cstr!("Esc").as_ptr();
    }

    KEYSTR[SDLK_CLEAR as usize] = cstr!("Clear").as_ptr();
    KEYSTR[SDLK_PAUSE as usize] = cstr!("Pause").as_ptr();
    KEYSTR[SDLK_EXCLAIM as usize] = cstr!("!").as_ptr();
    KEYSTR[SDLK_QUOTEDBL as usize] = cstr!("\"").as_ptr();
    KEYSTR[SDLK_HASH as usize] = cstr!("#").as_ptr();
    KEYSTR[SDLK_DOLLAR as usize] = cstr!("$").as_ptr();
    KEYSTR[SDLK_AMPERSAND as usize] = cstr!("&").as_ptr();
    KEYSTR[SDLK_QUOTE as usize] = cstr!("'").as_ptr();
    KEYSTR[SDLK_LEFTPAREN as usize] = cstr!("(").as_ptr();
    KEYSTR[SDLK_RIGHTPAREN as usize] = cstr!(")").as_ptr();
    KEYSTR[SDLK_ASTERISK as usize] = cstr!("*").as_ptr();
    KEYSTR[SDLK_PLUS as usize] = cstr!("+").as_ptr();
    KEYSTR[SDLK_COMMA as usize] = cstr!(",").as_ptr();
    KEYSTR[SDLK_MINUS as usize] = cstr!("-").as_ptr();
    KEYSTR[SDLK_PERIOD as usize] = cstr!(".").as_ptr();
    KEYSTR[SDLK_SLASH as usize] = cstr!("/").as_ptr();
    KEYSTR[SDLK_0 as usize] = cstr!("0").as_ptr();
    KEYSTR[SDLK_1 as usize] = cstr!("1").as_ptr();
    KEYSTR[SDLK_2 as usize] = cstr!("2").as_ptr();
    KEYSTR[SDLK_3 as usize] = cstr!("3").as_ptr();
    KEYSTR[SDLK_4 as usize] = cstr!("4").as_ptr();
    KEYSTR[SDLK_5 as usize] = cstr!("5").as_ptr();
    KEYSTR[SDLK_6 as usize] = cstr!("6").as_ptr();
    KEYSTR[SDLK_7 as usize] = cstr!("7").as_ptr();
    KEYSTR[SDLK_8 as usize] = cstr!("8").as_ptr();
    KEYSTR[SDLK_9 as usize] = cstr!("9").as_ptr();
    KEYSTR[SDLK_COLON as usize] = cstr!(":").as_ptr();
    KEYSTR[SDLK_SEMICOLON as usize] = cstr!(";").as_ptr();
    KEYSTR[SDLK_LESS as usize] = cstr!("<").as_ptr();
    KEYSTR[SDLK_EQUALS as usize] = cstr!("=").as_ptr();
    KEYSTR[SDLK_GREATER as usize] = cstr!(">").as_ptr();
    KEYSTR[SDLK_QUESTION as usize] = cstr!("?").as_ptr();
    KEYSTR[SDLK_AT as usize] = cstr!("@").as_ptr();
    KEYSTR[SDLK_LEFTBRACKET as usize] = cstr!("[").as_ptr();
    KEYSTR[SDLK_BACKSLASH as usize] = cstr!("\\").as_ptr();
    KEYSTR[SDLK_RIGHTBRACKET as usize] = cstr!(" as usize]").as_ptr();
    KEYSTR[SDLK_CARET as usize] = cstr!("^").as_ptr();
    KEYSTR[SDLK_UNDERSCORE as usize] = cstr!("_").as_ptr();
    KEYSTR[SDLK_BACKQUOTE as usize] = cstr!("`").as_ptr();
    KEYSTR[SDLK_a as usize] = cstr!("a").as_ptr();
    KEYSTR[SDLK_b as usize] = cstr!("b").as_ptr();
    KEYSTR[SDLK_c as usize] = cstr!("c").as_ptr();
    KEYSTR[SDLK_d as usize] = cstr!("d").as_ptr();
    KEYSTR[SDLK_e as usize] = cstr!("e").as_ptr();
    KEYSTR[SDLK_f as usize] = cstr!("f").as_ptr();
    KEYSTR[SDLK_g as usize] = cstr!("g").as_ptr();
    KEYSTR[SDLK_h as usize] = cstr!("h").as_ptr();
    KEYSTR[SDLK_i as usize] = cstr!("i").as_ptr();
    KEYSTR[SDLK_j as usize] = cstr!("j").as_ptr();
    KEYSTR[SDLK_k as usize] = cstr!("k").as_ptr();
    KEYSTR[SDLK_l as usize] = cstr!("l").as_ptr();
    KEYSTR[SDLK_m as usize] = cstr!("m").as_ptr();
    KEYSTR[SDLK_n as usize] = cstr!("n").as_ptr();
    KEYSTR[SDLK_o as usize] = cstr!("o").as_ptr();
    KEYSTR[SDLK_p as usize] = cstr!("p").as_ptr();
    KEYSTR[SDLK_q as usize] = cstr!("q").as_ptr();
    KEYSTR[SDLK_r as usize] = cstr!("r").as_ptr();
    KEYSTR[SDLK_s as usize] = cstr!("s").as_ptr();
    KEYSTR[SDLK_t as usize] = cstr!("t").as_ptr();
    KEYSTR[SDLK_u as usize] = cstr!("u").as_ptr();
    KEYSTR[SDLK_v as usize] = cstr!("v").as_ptr();
    KEYSTR[SDLK_w as usize] = cstr!("w").as_ptr();
    KEYSTR[SDLK_x as usize] = cstr!("x").as_ptr();
    KEYSTR[SDLK_y as usize] = cstr!("y").as_ptr();
    KEYSTR[SDLK_z as usize] = cstr!("z").as_ptr();
    KEYSTR[SDLK_DELETE as usize] = cstr!("Del").as_ptr();

    /* Numeric keypad */
    KEYSTR[SDLK_KP0 as usize] = cstr!("Num[0 as usize]").as_ptr();
    KEYSTR[SDLK_KP1 as usize] = cstr!("Num[1 as usize]").as_ptr();
    KEYSTR[SDLK_KP2 as usize] = cstr!("Num[2 as usize]").as_ptr();
    KEYSTR[SDLK_KP3 as usize] = cstr!("Num[3 as usize]").as_ptr();
    KEYSTR[SDLK_KP4 as usize] = cstr!("Num[4 as usize]").as_ptr();
    KEYSTR[SDLK_KP5 as usize] = cstr!("Num[5 as usize]").as_ptr();
    KEYSTR[SDLK_KP6 as usize] = cstr!("Num[6 as usize]").as_ptr();
    KEYSTR[SDLK_KP7 as usize] = cstr!("Num[7 as usize]").as_ptr();
    KEYSTR[SDLK_KP8 as usize] = cstr!("Num[8 as usize]").as_ptr();
    KEYSTR[SDLK_KP9 as usize] = cstr!("Num[9 as usize]").as_ptr();
    KEYSTR[SDLK_KP_PERIOD as usize] = cstr!("Num[. as usize]").as_ptr();
    KEYSTR[SDLK_KP_DIVIDE as usize] = cstr!("Num[/ as usize]").as_ptr();
    KEYSTR[SDLK_KP_MULTIPLY as usize] = cstr!("Num[* as usize]").as_ptr();
    KEYSTR[SDLK_KP_MINUS as usize] = cstr!("Num[- as usize]").as_ptr();
    KEYSTR[SDLK_KP_PLUS as usize] = cstr!("Num[+ as usize]").as_ptr();
    KEYSTR[SDLK_KP_ENTER as usize] = cstr!("Num[Enter as usize]").as_ptr();
    KEYSTR[SDLK_KP_EQUALS as usize] = cstr!("Num[= as usize]").as_ptr();

    /* Arrows + Home/End pad */
    KEYSTR[SDLK_UP as usize] = cstr!("Up").as_ptr();
    KEYSTR[SDLK_DOWN as usize] = cstr!("Down").as_ptr();
    KEYSTR[SDLK_RIGHT as usize] = cstr!("Right").as_ptr();
    KEYSTR[SDLK_LEFT as usize] = cstr!("Left").as_ptr();
    KEYSTR[SDLK_INSERT as usize] = cstr!("Insert").as_ptr();
    KEYSTR[SDLK_HOME as usize] = cstr!("Home").as_ptr();
    KEYSTR[SDLK_END as usize] = cstr!("End").as_ptr();
    KEYSTR[SDLK_PAGEUP as usize] = cstr!("PageUp").as_ptr();
    KEYSTR[SDLK_PAGEDOWN as usize] = cstr!("PageDown").as_ptr();

    /* Function keys */
    KEYSTR[SDLK_F1 as usize] = cstr!("F1").as_ptr();
    KEYSTR[SDLK_F2 as usize] = cstr!("F2").as_ptr();
    KEYSTR[SDLK_F3 as usize] = cstr!("F3").as_ptr();
    KEYSTR[SDLK_F4 as usize] = cstr!("F4").as_ptr();
    KEYSTR[SDLK_F5 as usize] = cstr!("F5").as_ptr();
    KEYSTR[SDLK_F6 as usize] = cstr!("F6").as_ptr();
    KEYSTR[SDLK_F7 as usize] = cstr!("F7").as_ptr();
    KEYSTR[SDLK_F8 as usize] = cstr!("F8").as_ptr();
    KEYSTR[SDLK_F9 as usize] = cstr!("F9").as_ptr();
    KEYSTR[SDLK_F10 as usize] = cstr!("F10").as_ptr();
    KEYSTR[SDLK_F11 as usize] = cstr!("F11").as_ptr();
    KEYSTR[SDLK_F12 as usize] = cstr!("F12").as_ptr();
    KEYSTR[SDLK_F13 as usize] = cstr!("F13").as_ptr();
    KEYSTR[SDLK_F14 as usize] = cstr!("F14").as_ptr();
    KEYSTR[SDLK_F15 as usize] = cstr!("F15").as_ptr();

    /* Key state modifier keys */
    KEYSTR[SDLK_NUMLOCK as usize] = cstr!("NumLock").as_ptr();
    KEYSTR[SDLK_CAPSLOCK as usize] = cstr!("CapsLock").as_ptr();
    KEYSTR[SDLK_SCROLLOCK as usize] = cstr!("ScrlLock").as_ptr();
    #[cfg(feature = "gcw0")]
    {
        keystr[SDLK_LSHIFT as usize] = cstr!("X").as_ptr();
        keystr[SDLK_LCTRL as usize] = cstr!("A").as_ptr();
        keystr[SDLK_LALT as usize] = cstr!("B").as_ptr();
    }

    #[cfg(not(feature = "gcw0"))]
    {
        KEYSTR[SDLK_LSHIFT as usize] = cstr!("LShift").as_ptr();
        KEYSTR[SDLK_LCTRL as usize] = cstr!("LCtrl").as_ptr();
        KEYSTR[SDLK_LALT as usize] = cstr!("LAlt").as_ptr();
    }

    KEYSTR[SDLK_RSHIFT as usize] = cstr!("RShift").as_ptr();
    KEYSTR[SDLK_RCTRL as usize] = cstr!("RCtrl").as_ptr();
    KEYSTR[SDLK_RALT as usize] = cstr!("RAlt").as_ptr();
    KEYSTR[SDLK_RMETA as usize] = cstr!("RMeta").as_ptr();
    KEYSTR[SDLK_LMETA as usize] = cstr!("LMeta").as_ptr();
    KEYSTR[SDLK_LSUPER as usize] = cstr!("LSuper").as_ptr();
    KEYSTR[SDLK_RSUPER as usize] = cstr!("RSuper").as_ptr();
    KEYSTR[SDLK_MODE as usize] = cstr!("Mode").as_ptr();
    KEYSTR[SDLK_COMPOSE as usize] = cstr!("Compose").as_ptr();

    /* Miscellaneous function keys */
    KEYSTR[SDLK_HELP as usize] = cstr!("Help").as_ptr();
    KEYSTR[SDLK_PRINT as usize] = cstr!("Print").as_ptr();
    KEYSTR[SDLK_SYSREQ as usize] = cstr!("SysReq").as_ptr();
    KEYSTR[SDLK_BREAK as usize] = cstr!("Break").as_ptr();
    KEYSTR[SDLK_MENU as usize] = cstr!("Menu").as_ptr();
    KEYSTR[SDLK_POWER as usize] = cstr!("Power").as_ptr();
    KEYSTR[SDLK_EURO as usize] = cstr!("Euro").as_ptr();
    KEYSTR[SDLK_UNDO as usize] = cstr!("Undo").as_ptr();

    /* Mouse und Joy buttons */
    KEYSTR[PointerStates::MouseButton1 as usize] = cstr!("Mouse1").as_ptr();
    KEYSTR[PointerStates::MouseButton2 as usize] = cstr!("Mouse2").as_ptr();
    KEYSTR[PointerStates::MouseButton3 as usize] = cstr!("Mouse3").as_ptr();
    KEYSTR[PointerStates::MouseWheelup as usize] = cstr!("WheelUp").as_ptr();
    KEYSTR[PointerStates::MouseWheeldown as usize] = cstr!("WheelDown").as_ptr();

    KEYSTR[PointerStates::JoyUp as usize] = cstr!("JoyUp").as_ptr();
    KEYSTR[PointerStates::JoyDown as usize] = cstr!("JoyDown").as_ptr();
    KEYSTR[PointerStates::JoyLeft as usize] = cstr!("JoyLeft").as_ptr();
    KEYSTR[PointerStates::JoyRight as usize] = cstr!("JoyRight").as_ptr();
    KEYSTR[PointerStates::JoyButton1 as usize] = cstr!("Joy-A").as_ptr();
    KEYSTR[PointerStates::JoyButton2 as usize] = cstr!("Joy-B").as_ptr();
    KEYSTR[PointerStates::JoyButton3 as usize] = cstr!("Joy-X").as_ptr();
    KEYSTR[PointerStates::JoyButton4 as usize] = cstr!("Joy-Y").as_ptr();
}
