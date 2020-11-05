use crate::defs::PointerStates;
#[cfg(target_os = "android")]
use crate::global::ne_screen;

use std::os::raw::c_int;

extern "C" {
    #[no_mangle]
    pub fn SDL_Delay(ms: u32);

    #[no_mangle]
    pub fn update_input() -> c_int;

    #[no_mangle]
    pub static mut input_state: [c_int; PointerStates::Last as usize];
}

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
