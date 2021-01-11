#[cfg(not(feature = "gcw0"))]
use crate::{
    defs::Cmds,
    input::{key_cmds, wait_for_all_keys_released, wait_for_key_pressed},
};

#[cfg(feature = "gcw0")]
use crate::{
    defs::{gcw0_a_pressed, gcw0_any_button_pressed, gcw0_any_button_pressed_r},
    input::SDL_Delay,
};

use crate::{
    b_font::{PutString, TextWidth},
    defs::{self, MenuAction},
    global::User_Rect,
    graphics::ne_screen,
    misc::Terminate,
    sound::MenuItemSelectedSound,
};

use cstr::cstr;
use sdl::video::ll::SDL_Flip;
use std::{
    ffi::CStr,
    os::raw::{c_char, c_int},
    ptr::null_mut,
};

extern "C" {
    pub fn showMainMenu();
    pub fn Cheatmenu();
    pub fn FreeMenuData();
    pub fn getMenuAction(wait_repeat_ticks: u32) -> MenuAction;
    pub fn InitiateMenu(with_droids: bool);
    pub static mut fheight: c_int;
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
