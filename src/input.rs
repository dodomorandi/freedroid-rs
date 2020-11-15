#[cfg(target_os = "android")]
use crate::global::ne_screen;
use crate::{
    defs::{
        self, get_user_center, AltPressed, Cmds, CtrlPressed, DownPressed, LeftPressed, MenuAction,
        PointerStates, RightPressed, ShiftPressed, UpPressed,
    },
    global::{axis_is_active, input_axis, joy, joy_num_axes, joy_sensitivity, last_mouse_event},
    graphics::{toggle_fullscreen, TakeScreenshot},
    menu::{handle_QuitGame, showMainMenu, Cheatmenu},
    misc::{Pause, Terminate},
};

use cstr::cstr;
use log::info;
use sdl::{
    event::{
        ll::{
            SDLMod, SDL_Event, SDL_PollEvent, SDL_ENABLE, SDL_JOYAXISMOTION, SDL_JOYBUTTONDOWN,
            SDL_JOYBUTTONUP, SDL_KEYDOWN, SDL_KEYUP, SDL_MOUSEBUTTONDOWN, SDL_MOUSEBUTTONUP,
            SDL_MOUSEMOTION, SDL_QUIT,
        },
        MouseState,
    },
    joy::{
        get_joystick_name,
        ll::{
            SDL_JoystickEventState, SDL_JoystickNumAxes, SDL_JoystickNumButtons, SDL_JoystickOpen,
            SDL_NumJoysticks,
        },
    },
    ll::{SDL_GetTicks, SDL_InitSubSystem, SDL_INIT_JOYSTICK},
};
use std::{
    convert::{identity, TryFrom},
    os::raw::{c_char, c_int},
    ptr::{null, null_mut},
};

extern "C" {
    pub fn SDL_Delay(ms: u32);
    pub static mut input_state: [c_int; PointerStates::Last as usize];
    pub static mut key_cmds: [[c_int; 3]; Cmds::Last as usize];
    pub static mut show_cursor: bool;
    pub static mut event: SDL_Event;
    pub static mut current_modifiers: SDLMod;
    pub static mut WheelUpEvents: c_int;
    pub static mut WheelDownEvents: c_int;
}

pub const CURSOR_KEEP_VISIBLE: u32 = 3000; // ticks to keep mouse-cursor visible without mouse-input

#[no_mangle]
pub static mut keystr: [*const c_char; PointerStates::Last as usize] =
    [null(); PointerStates::Last as usize];

/// Check if any keys have been 'freshly' pressed. If yes, return key-code, otherwise 0.
#[no_mangle]
pub extern "C" fn wait_for_key_pressed() -> c_int {
    loop {
        match any_key_just_pressed() {
            0 => unsafe { SDL_Delay(1) },
            key => break key,
        }
    }
}

#[no_mangle]
pub extern "C" fn any_key_just_pressed() -> c_int {
    #[cfg(target_os = "android")]
    unsafe {
        SDL_Flip(ne_screen)
    };

    unsafe {
        update_input();
    }

    let pressed_key = (0..PointerStates::Last as c_int)
        .map(|key| (key, unsafe { &mut input_state[key as usize] }))
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

#[no_mangle]
pub unsafe extern "C" fn update_input() -> c_int {
    // switch mouse-cursor visibility as a function of time of last activity
    if SDL_GetTicks() - last_mouse_event > CURSOR_KEEP_VISIBLE {
        show_cursor = false;
    } else {
        show_cursor = true;
    }

    while SDL_PollEvent(&mut event) != 0 {
        match (*event._type()).into() {
            SDL_QUIT => {
                info!("User requested termination, terminating.");
                Terminate(0);
            }

            SDL_KEYDOWN => {
                let key = &*event.key();
                current_modifiers = key.keysym._mod;
                input_state[usize::try_from(key.keysym.sym).unwrap()] = PRESSED;
                #[cfg(feature = "gcw0")]
                if (input_axis.x || input_axis.y) {
                    axis_is_active = TRUE; // 4 GCW-0 ; breaks cursor keys after axis has been active...
                }
            }
            SDL_KEYUP => {
                let key = &*event.key();
                current_modifiers = key.keysym._mod;
                input_state[usize::try_from(key.keysym.sym).unwrap()] = RELEASED;
                #[cfg(feature = "gcw0")]
                {
                    axis_is_active = FALSE;
                }
            }

            SDL_JOYAXISMOTION => {
                let jaxis = &*event.jaxis();
                let axis = jaxis.axis;
                if axis == 0 || ((joy_num_axes >= 5) && (axis == 3))
                /* x-axis */
                {
                    input_axis.x = jaxis.value.into();

                    // this is a bit tricky, because we want to allow direction keys
                    // to be soft-released. When mapping the joystick->keyboard, we
                    // therefore have to make sure that this mapping only occurs when
                    // and actual _change_ of the joystick-direction ('digital') occurs
                    // so that it behaves like "set"/"release"
                    if joy_sensitivity * i32::from(jaxis.value) > 10000 {
                        /* about half tilted */
                        input_state[PointerStates::JoyRight as usize] = PRESSED;
                        input_state[PointerStates::JoyLeft as usize] = false.into();
                    } else if joy_sensitivity * i32::from(jaxis.value) < -10000 {
                        input_state[PointerStates::JoyLeft as usize] = PRESSED;
                        input_state[PointerStates::JoyRight as usize] = false.into();
                    } else {
                        input_state[PointerStates::JoyLeft as usize] = false.into();
                        input_state[PointerStates::JoyRight as usize] = false.into();
                    }
                } else if (axis == 1) || ((joy_num_axes >= 5) && (axis == 4)) {
                    /* y-axis */
                    input_axis.y = jaxis.value.into();

                    if joy_sensitivity * i32::from(jaxis.value) > 10000 {
                        input_state[PointerStates::JoyDown as usize] = PRESSED;
                        input_state[PointerStates::JoyUp as usize] = false.into();
                    } else if joy_sensitivity * i32::from(jaxis.value) < -10000 {
                        input_state[PointerStates::JoyUp as usize] = PRESSED;
                        input_state[PointerStates::JoyDown as usize] = false.into();
                    } else {
                        input_state[PointerStates::JoyUp as usize] = false.into();
                        input_state[PointerStates::JoyDown as usize] = false.into();
                    }
                }
            }

            SDL_JOYBUTTONDOWN => {
                let jbutton = &*event.jbutton();
                // first button
                if jbutton.button == 0 {
                    input_state[PointerStates::JoyButton1 as usize] = PRESSED;
                }
                // second button
                else if jbutton.button == 1 {
                    input_state[PointerStates::JoyButton2 as usize] = PRESSED;
                }
                // and third button
                else if jbutton.button == 2 {
                    input_state[PointerStates::JoyButton3 as usize] = PRESSED;
                }
                // and fourth button
                else if jbutton.button == 3 {
                    input_state[PointerStates::JoyButton4 as usize] = PRESSED;
                }

                axis_is_active = true.into();
            }

            SDL_JOYBUTTONUP => {
                let jbutton = &*event.jbutton();
                // first button
                if jbutton.button == 0 {
                    input_state[PointerStates::JoyButton1 as usize] = false.into();
                }
                // second button
                else if jbutton.button == 1 {
                    input_state[PointerStates::JoyButton2 as usize] = false.into();
                }
                // and third button
                else if jbutton.button == 2 {
                    input_state[PointerStates::JoyButton3 as usize] = false.into();
                }
                // and fourth button
                else if jbutton.button == 3 {
                    input_state[PointerStates::JoyButton4 as usize] = false.into();
                }

                axis_is_active = false.into();
            }

            SDL_MOUSEMOTION => {
                let button = &*event.button();
                let user_center = get_user_center();
                input_axis.x = i32::from(button.x) - i32::from(user_center.x) + 16;
                input_axis.y = i32::from(button.y) - i32::from(user_center.y) + 16;

                last_mouse_event = SDL_GetTicks();
            }

            /* Mouse control */
            SDL_MOUSEBUTTONDOWN => {
                let button = &*event.button();
                if button.button == MouseState::Left as u8 {
                    input_state[PointerStates::MouseButton1 as usize] = PRESSED;
                    axis_is_active = true.into();
                }

                if button.button == MouseState::Right as u8 {
                    input_state[PointerStates::MouseButton2 as usize] = PRESSED;
                }

                if button.button == MouseState::Middle as u8 {
                    input_state[PointerStates::MouseButton3 as usize] = PRESSED;
                }

                // wheel events are immediately released, so we rather
                // count the number of not yet read-out events
                if button.button == MouseState::WheelUp as u8 {
                    WheelUpEvents += 1;
                }

                if button.button == MouseState::WheelDown as u8 {
                    WheelDownEvents += 1;
                }

                last_mouse_event = SDL_GetTicks();
            }

            SDL_MOUSEBUTTONUP => {
                let button = &*event.button();
                if button.button == MouseState::Left as u8 {
                    input_state[PointerStates::MouseButton1 as usize] = false.into();
                    axis_is_active = false.into();
                }

                if button.button == MouseState::Right as u8 {
                    input_state[PointerStates::MouseButton2 as usize] = false.into();
                }

                if button.button == MouseState::Middle as u8 {
                    input_state[PointerStates::MouseButton3 as usize] = false.into();
                }
            }

            _ => break,
        }
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn KeyIsPressed(key: c_int) -> bool {
    update_input();

    (input_state[usize::try_from(key).unwrap()] & PRESSED) == PRESSED
}

/// Does the same as KeyIsPressed, but automatically releases the key as well..
#[no_mangle]
pub unsafe extern "C" fn KeyIsPressedR(key: c_int) -> bool {
    let ret = KeyIsPressed(key);

    ReleaseKey(key);
    ret
}

#[no_mangle]
pub unsafe extern "C" fn ReleaseKey(key: c_int) {
    input_state[usize::try_from(key).unwrap()] = false.into();
}

#[no_mangle]
pub unsafe extern "C" fn WheelUpPressed() -> bool {
    update_input();
    if WheelUpEvents != 0 {
        WheelUpEvents -= 1;
        true
    } else {
        false
    }
}

#[no_mangle]
pub unsafe extern "C" fn WheelDownPressed() -> bool {
    update_input();
    if WheelDownEvents != 0 {
        WheelDownEvents -= 1;
        true
    } else {
        false
    }
}

#[no_mangle]
pub unsafe extern "C" fn cmd_is_active(cmd: Cmds) -> bool {
    let cmd = cmd as usize;
    KeyIsPressed(key_cmds[cmd][0])
        || KeyIsPressed(key_cmds[cmd][1])
        || KeyIsPressed(key_cmds[cmd][2])
}

/// the same but release the keys: use only for menus!
#[no_mangle]
pub unsafe extern "C" fn cmd_is_activeR(cmd: Cmds) -> bool {
    let cmd = cmd as usize;

    let c1 = KeyIsPressedR(key_cmds[cmd][0]);
    let c2 = KeyIsPressedR(key_cmds[cmd][1]);
    let c3 = KeyIsPressedR(key_cmds[cmd][2]);

    c1 || c2 || c3
}

#[no_mangle]
pub unsafe extern "C" fn wait_for_all_keys_released() {
    while any_key_is_pressedR() {
        SDL_Delay(1);
    }
    ResetMouseWheel();
}

#[no_mangle]
pub unsafe extern "C" fn any_key_is_pressedR() -> bool {
    #[cfg(target_os = "android")]
    SDL_Flip(ne_screen); // make sure we keep updating screen to read out Android inputs

    #[cfg(not(target_os = "android"))]
    update_input();

    for state in &mut input_state {
        if (*state & PRESSED) != 0 {
            *state = 0;
            return true;
        }
    }
    false
}

// forget the wheel-counters
#[no_mangle]
pub unsafe extern "C" fn ResetMouseWheel() {
    WheelUpEvents = 0;
    WheelDownEvents = 0;
}

#[no_mangle]
pub unsafe extern "C" fn ModIsPressed(sdl_mod: SDLMod) -> bool {
    update_input();
    (current_modifiers & sdl_mod) != 0
}

#[no_mangle]
pub unsafe extern "C" fn NoDirectionPressed() -> bool {
    !((axis_is_active != 0 && (input_axis.x != 0 || input_axis.y != 0))
        || DownPressed()
        || UpPressed()
        || LeftPressed()
        || RightPressed())
}

#[no_mangle]
pub unsafe extern "C" fn JoyAxisMotion() -> c_int {
    update_input();
    (input_state[PointerStates::JoyUp as usize] != 0
        || input_state[PointerStates::JoyDown as usize] != 0
        || input_state[PointerStates::JoyLeft as usize] != 0
        || input_state[PointerStates::JoyRight as usize] != 0)
        .into()
}

#[no_mangle]
pub unsafe extern "C" fn ReactToSpecialKeys() {
    if cmd_is_activeR(Cmds::Quit) {
        handle_QuitGame(MenuAction::Click);
    }

    if cmd_is_activeR(Cmds::Pause) {
        Pause();
    }

    if cmd_is_active(Cmds::Screenshot) {
        TakeScreenshot();
    }

    if cmd_is_activeR(Cmds::Fullscreen) {
        toggle_fullscreen();
    }

    if cmd_is_activeR(Cmds::Menu) {
        showMainMenu();
    }

    // this stuff remains hardcoded to keys
    if KeyIsPressedR(b'c'.into()) && AltPressed() && CtrlPressed() && ShiftPressed() {
        Cheatmenu();
    }
}

#[no_mangle]
pub unsafe extern "C" fn Init_Joy() {
    if SDL_InitSubSystem(SDL_INIT_JOYSTICK) == -1 {
        eprintln!("Couldn't initialize SDL-Joystick: {}", sdl::get_error(),);
        Terminate(defs::ERR.into());
    } else {
        info!("SDL Joystick initialisation successful.");
    }

    let num_joy = SDL_NumJoysticks();
    info!("{} Joysticks found!\n", num_joy);

    if num_joy > 0 {
        joy = SDL_JoystickOpen(0);
    }

    if !joy.is_null() {
        info!(
            "Identifier: {}",
            get_joystick_name(0).unwrap_or_else(identity)
        );

        joy_num_axes = SDL_JoystickNumAxes(joy);
        info!("Number of Axes: {}", joy_num_axes);
        info!("Number of Buttons: {}", SDL_JoystickNumButtons(joy));

        /* aktivate Joystick event handling */
        SDL_JoystickEventState(SDL_ENABLE);
    } else {
        joy = null_mut(); /* signals that no yoystick is present */
    }
}

#[no_mangle]
pub unsafe extern "C" fn init_keystr() {
    use sdl::keysym::*;

    keystr[0] = cstr!("NONE").as_ptr(); // Empty bind will otherwise crash on some platforms - also, we choose "NONE" as a placeholder...
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
        keystr[SDLK_BACKSPACE as usize] = cstr!("BS").as_ptr();
        keystr[SDLK_TAB as usize] = cstr!("Tab").as_ptr();
        keystr[SDLK_RETURN as usize] = cstr!("Return").as_ptr();
        keystr[SDLK_SPACE as usize] = cstr!("Space").as_ptr();
        keystr[SDLK_ESCAPE as usize] = cstr!("Esc").as_ptr();
    }

    keystr[SDLK_CLEAR as usize] = cstr!("Clear").as_ptr();
    keystr[SDLK_PAUSE as usize] = cstr!("Pause").as_ptr();
    keystr[SDLK_EXCLAIM as usize] = cstr!("!").as_ptr();
    keystr[SDLK_QUOTEDBL as usize] = cstr!("\"").as_ptr();
    keystr[SDLK_HASH as usize] = cstr!("#").as_ptr();
    keystr[SDLK_DOLLAR as usize] = cstr!("$").as_ptr();
    keystr[SDLK_AMPERSAND as usize] = cstr!("&").as_ptr();
    keystr[SDLK_QUOTE as usize] = cstr!("'").as_ptr();
    keystr[SDLK_LEFTPAREN as usize] = cstr!("(").as_ptr();
    keystr[SDLK_RIGHTPAREN as usize] = cstr!(")").as_ptr();
    keystr[SDLK_ASTERISK as usize] = cstr!("*").as_ptr();
    keystr[SDLK_PLUS as usize] = cstr!("+").as_ptr();
    keystr[SDLK_COMMA as usize] = cstr!(",").as_ptr();
    keystr[SDLK_MINUS as usize] = cstr!("-").as_ptr();
    keystr[SDLK_PERIOD as usize] = cstr!(".").as_ptr();
    keystr[SDLK_SLASH as usize] = cstr!("/").as_ptr();
    keystr[SDLK_0 as usize] = cstr!("0").as_ptr();
    keystr[SDLK_1 as usize] = cstr!("1").as_ptr();
    keystr[SDLK_2 as usize] = cstr!("2").as_ptr();
    keystr[SDLK_3 as usize] = cstr!("3").as_ptr();
    keystr[SDLK_4 as usize] = cstr!("4").as_ptr();
    keystr[SDLK_5 as usize] = cstr!("5").as_ptr();
    keystr[SDLK_6 as usize] = cstr!("6").as_ptr();
    keystr[SDLK_7 as usize] = cstr!("7").as_ptr();
    keystr[SDLK_8 as usize] = cstr!("8").as_ptr();
    keystr[SDLK_9 as usize] = cstr!("9").as_ptr();
    keystr[SDLK_COLON as usize] = cstr!(":").as_ptr();
    keystr[SDLK_SEMICOLON as usize] = cstr!(";").as_ptr();
    keystr[SDLK_LESS as usize] = cstr!("<").as_ptr();
    keystr[SDLK_EQUALS as usize] = cstr!("=").as_ptr();
    keystr[SDLK_GREATER as usize] = cstr!(">").as_ptr();
    keystr[SDLK_QUESTION as usize] = cstr!("?").as_ptr();
    keystr[SDLK_AT as usize] = cstr!("@").as_ptr();
    keystr[SDLK_LEFTBRACKET as usize] = cstr!("[").as_ptr();
    keystr[SDLK_BACKSLASH as usize] = cstr!("\\").as_ptr();
    keystr[SDLK_RIGHTBRACKET as usize] = cstr!(" as usize]").as_ptr();
    keystr[SDLK_CARET as usize] = cstr!("^").as_ptr();
    keystr[SDLK_UNDERSCORE as usize] = cstr!("_").as_ptr();
    keystr[SDLK_BACKQUOTE as usize] = cstr!("`").as_ptr();
    keystr[SDLK_a as usize] = cstr!("a").as_ptr();
    keystr[SDLK_b as usize] = cstr!("b").as_ptr();
    keystr[SDLK_c as usize] = cstr!("c").as_ptr();
    keystr[SDLK_d as usize] = cstr!("d").as_ptr();
    keystr[SDLK_e as usize] = cstr!("e").as_ptr();
    keystr[SDLK_f as usize] = cstr!("f").as_ptr();
    keystr[SDLK_g as usize] = cstr!("g").as_ptr();
    keystr[SDLK_h as usize] = cstr!("h").as_ptr();
    keystr[SDLK_i as usize] = cstr!("i").as_ptr();
    keystr[SDLK_j as usize] = cstr!("j").as_ptr();
    keystr[SDLK_k as usize] = cstr!("k").as_ptr();
    keystr[SDLK_l as usize] = cstr!("l").as_ptr();
    keystr[SDLK_m as usize] = cstr!("m").as_ptr();
    keystr[SDLK_n as usize] = cstr!("n").as_ptr();
    keystr[SDLK_o as usize] = cstr!("o").as_ptr();
    keystr[SDLK_p as usize] = cstr!("p").as_ptr();
    keystr[SDLK_q as usize] = cstr!("q").as_ptr();
    keystr[SDLK_r as usize] = cstr!("r").as_ptr();
    keystr[SDLK_s as usize] = cstr!("s").as_ptr();
    keystr[SDLK_t as usize] = cstr!("t").as_ptr();
    keystr[SDLK_u as usize] = cstr!("u").as_ptr();
    keystr[SDLK_v as usize] = cstr!("v").as_ptr();
    keystr[SDLK_w as usize] = cstr!("w").as_ptr();
    keystr[SDLK_x as usize] = cstr!("x").as_ptr();
    keystr[SDLK_y as usize] = cstr!("y").as_ptr();
    keystr[SDLK_z as usize] = cstr!("z").as_ptr();
    keystr[SDLK_DELETE as usize] = cstr!("Del").as_ptr();

    /* Numeric keypad */
    keystr[SDLK_KP0 as usize] = cstr!("Num[0 as usize]").as_ptr();
    keystr[SDLK_KP1 as usize] = cstr!("Num[1 as usize]").as_ptr();
    keystr[SDLK_KP2 as usize] = cstr!("Num[2 as usize]").as_ptr();
    keystr[SDLK_KP3 as usize] = cstr!("Num[3 as usize]").as_ptr();
    keystr[SDLK_KP4 as usize] = cstr!("Num[4 as usize]").as_ptr();
    keystr[SDLK_KP5 as usize] = cstr!("Num[5 as usize]").as_ptr();
    keystr[SDLK_KP6 as usize] = cstr!("Num[6 as usize]").as_ptr();
    keystr[SDLK_KP7 as usize] = cstr!("Num[7 as usize]").as_ptr();
    keystr[SDLK_KP8 as usize] = cstr!("Num[8 as usize]").as_ptr();
    keystr[SDLK_KP9 as usize] = cstr!("Num[9 as usize]").as_ptr();
    keystr[SDLK_KP_PERIOD as usize] = cstr!("Num[. as usize]").as_ptr();
    keystr[SDLK_KP_DIVIDE as usize] = cstr!("Num[/ as usize]").as_ptr();
    keystr[SDLK_KP_MULTIPLY as usize] = cstr!("Num[* as usize]").as_ptr();
    keystr[SDLK_KP_MINUS as usize] = cstr!("Num[- as usize]").as_ptr();
    keystr[SDLK_KP_PLUS as usize] = cstr!("Num[+ as usize]").as_ptr();
    keystr[SDLK_KP_ENTER as usize] = cstr!("Num[Enter as usize]").as_ptr();
    keystr[SDLK_KP_EQUALS as usize] = cstr!("Num[= as usize]").as_ptr();

    /* Arrows + Home/End pad */
    keystr[SDLK_UP as usize] = cstr!("Up").as_ptr();
    keystr[SDLK_DOWN as usize] = cstr!("Down").as_ptr();
    keystr[SDLK_RIGHT as usize] = cstr!("Right").as_ptr();
    keystr[SDLK_LEFT as usize] = cstr!("Left").as_ptr();
    keystr[SDLK_INSERT as usize] = cstr!("Insert").as_ptr();
    keystr[SDLK_HOME as usize] = cstr!("Home").as_ptr();
    keystr[SDLK_END as usize] = cstr!("End").as_ptr();
    keystr[SDLK_PAGEUP as usize] = cstr!("PageUp").as_ptr();
    keystr[SDLK_PAGEDOWN as usize] = cstr!("PageDown").as_ptr();

    /* Function keys */
    keystr[SDLK_F1 as usize] = cstr!("F1").as_ptr();
    keystr[SDLK_F2 as usize] = cstr!("F2").as_ptr();
    keystr[SDLK_F3 as usize] = cstr!("F3").as_ptr();
    keystr[SDLK_F4 as usize] = cstr!("F4").as_ptr();
    keystr[SDLK_F5 as usize] = cstr!("F5").as_ptr();
    keystr[SDLK_F6 as usize] = cstr!("F6").as_ptr();
    keystr[SDLK_F7 as usize] = cstr!("F7").as_ptr();
    keystr[SDLK_F8 as usize] = cstr!("F8").as_ptr();
    keystr[SDLK_F9 as usize] = cstr!("F9").as_ptr();
    keystr[SDLK_F10 as usize] = cstr!("F10").as_ptr();
    keystr[SDLK_F11 as usize] = cstr!("F11").as_ptr();
    keystr[SDLK_F12 as usize] = cstr!("F12").as_ptr();
    keystr[SDLK_F13 as usize] = cstr!("F13").as_ptr();
    keystr[SDLK_F14 as usize] = cstr!("F14").as_ptr();
    keystr[SDLK_F15 as usize] = cstr!("F15").as_ptr();

    /* Key state modifier keys */
    keystr[SDLK_NUMLOCK as usize] = cstr!("NumLock").as_ptr();
    keystr[SDLK_CAPSLOCK as usize] = cstr!("CapsLock").as_ptr();
    keystr[SDLK_SCROLLOCK as usize] = cstr!("ScrlLock").as_ptr();
    #[cfg(feature = "gcw0")]
    {
        keystr[SDLK_LSHIFT as usize] = cstr!("X").as_ptr();
        keystr[SDLK_LCTRL as usize] = cstr!("A").as_ptr();
        keystr[SDLK_LALT as usize] = cstr!("B").as_ptr();
    }

    #[cfg(not(feature = "gcw0"))]
    {
        keystr[SDLK_LSHIFT as usize] = cstr!("LShift").as_ptr();
        keystr[SDLK_LCTRL as usize] = cstr!("LCtrl").as_ptr();
        keystr[SDLK_LALT as usize] = cstr!("LAlt").as_ptr();
    }

    keystr[SDLK_RSHIFT as usize] = cstr!("RShift").as_ptr();
    keystr[SDLK_RCTRL as usize] = cstr!("RCtrl").as_ptr();
    keystr[SDLK_RALT as usize] = cstr!("RAlt").as_ptr();
    keystr[SDLK_RMETA as usize] = cstr!("RMeta").as_ptr();
    keystr[SDLK_LMETA as usize] = cstr!("LMeta").as_ptr();
    keystr[SDLK_LSUPER as usize] = cstr!("LSuper").as_ptr();
    keystr[SDLK_RSUPER as usize] = cstr!("RSuper").as_ptr();
    keystr[SDLK_MODE as usize] = cstr!("Mode").as_ptr();
    keystr[SDLK_COMPOSE as usize] = cstr!("Compose").as_ptr();

    /* Miscellaneous function keys */
    keystr[SDLK_HELP as usize] = cstr!("Help").as_ptr();
    keystr[SDLK_PRINT as usize] = cstr!("Print").as_ptr();
    keystr[SDLK_SYSREQ as usize] = cstr!("SysReq").as_ptr();
    keystr[SDLK_BREAK as usize] = cstr!("Break").as_ptr();
    keystr[SDLK_MENU as usize] = cstr!("Menu").as_ptr();
    keystr[SDLK_POWER as usize] = cstr!("Power").as_ptr();
    keystr[SDLK_EURO as usize] = cstr!("Euro").as_ptr();
    keystr[SDLK_UNDO as usize] = cstr!("Undo").as_ptr();

    /* Mouse und Joy buttons */
    keystr[PointerStates::MouseButton1 as usize] = cstr!("Mouse1").as_ptr();
    keystr[PointerStates::MouseButton2 as usize] = cstr!("Mouse2").as_ptr();
    keystr[PointerStates::MouseButton3 as usize] = cstr!("Mouse3").as_ptr();
    keystr[PointerStates::MouseWheelup as usize] = cstr!("WheelUp").as_ptr();
    keystr[PointerStates::MouseWheeldown as usize] = cstr!("WheelDown").as_ptr();

    keystr[PointerStates::JoyUp as usize] = cstr!("JoyUp").as_ptr();
    keystr[PointerStates::JoyDown as usize] = cstr!("JoyDown").as_ptr();
    keystr[PointerStates::JoyLeft as usize] = cstr!("JoyLeft").as_ptr();
    keystr[PointerStates::JoyRight as usize] = cstr!("JoyRight").as_ptr();
    keystr[PointerStates::JoyButton1 as usize] = cstr!("Joy-A").as_ptr();
    keystr[PointerStates::JoyButton2 as usize] = cstr!("Joy-B").as_ptr();
    keystr[PointerStates::JoyButton3 as usize] = cstr!("Joy-X").as_ptr();
    keystr[PointerStates::JoyButton4 as usize] = cstr!("Joy-Y").as_ptr();
}
