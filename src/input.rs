#[cfg(target_os = "android")]
use crate::global::ne_screen;
use crate::{
    defs::{get_user_center, Cmds, PointerStates},
    global::{axis_is_active, input_axis, joy_num_axes, joy_sensitivity, last_mouse_event},
    misc::Terminate,
};

use log::info;
use sdl::{
    event::{
        ll::{
            SDLMod, SDL_Event, SDL_PollEvent, SDL_JOYAXISMOTION, SDL_JOYBUTTONDOWN,
            SDL_JOYBUTTONUP, SDL_KEYDOWN, SDL_KEYUP, SDL_MOUSEBUTTONDOWN, SDL_MOUSEBUTTONUP,
            SDL_MOUSEMOTION, SDL_QUIT,
        },
        MouseState,
    },
    ll::SDL_GetTicks,
};
use std::{convert::TryFrom, os::raw::c_int};

extern "C" {
    pub fn SDL_Delay(ms: u32);
    pub static mut input_state: [c_int; PointerStates::Last as usize];
    pub fn KeyIsPressedR(key: c_int) -> bool;
    pub fn cmd_is_activeR(command: Cmds) -> bool;
    pub fn cmd_is_active(command: Cmds) -> bool;
    pub fn wait_for_all_keys_released();
    pub static mut key_cmds: [[c_int; 3]; Cmds::Last as usize];
    pub fn WheelUpPressed() -> bool;
    pub fn WheelDownPressed() -> bool;
    pub static mut show_cursor: bool;
    pub static mut event: SDL_Event;
    pub static mut current_modifiers: SDLMod;
    pub static mut WheelUpEvents: c_int;
    pub static mut WheelDownEvents: c_int;
}

pub const CURSOR_KEEP_VISIBLE: u32 = 3000; // ticks to keep mouse-cursor visible without mouse-input

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
const OLD_BIT: c_int = 0x01 << 9;
const LONG_PRESSED: c_int = true as c_int | OLD_BIT;
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
