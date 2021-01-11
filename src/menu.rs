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
    b_font::{FontHeight, GetCurrentFont, PutString, SetCurrentFont, TextWidth},
    defs::{self, AssembleCombatWindowFlags, DisplayBannerFlags, MenuAction, Status},
    global::{Me, Menu_BFont, User_Rect},
    graphics::{ne_screen, ClearGraphMem, MakeGridOnScreen},
    misc::{Activate_Conservative_Frame_Computation, Terminate},
    sound::MenuItemSelectedSound,
    view::{Assemble_Combat_Picture, DisplayBanner},
};

use cstr::cstr;
use sdl::{
    mouse::ll::{SDL_ShowCursor, SDL_DISABLE},
    video::ll::{SDL_DisplayFormat, SDL_Flip, SDL_FreeSurface, SDL_SetClipRect, SDL_Surface},
};
use std::{
    ffi::CStr,
    os::raw::{c_char, c_int},
    ptr::null_mut,
};

extern "C" {
    pub fn Cheatmenu();
    pub fn getMenuAction(wait_repeat_ticks: u32) -> MenuAction;
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
