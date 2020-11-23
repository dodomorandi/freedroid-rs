use crate::{
    defs::{self, Cmds, DisplayBannerFlags, Sound},
    global::{ne_screen, GameConfig, Screen_Rect, User_Rect},
    input::{cmd_is_active, SDL_Delay},
    misc::{Activate_Conservative_Frame_Computation, Terminate},
    sound::Play_Sound,
    view::DisplayBanner,
};

use cstr::cstr;
use log::{error, trace, warn};
use sdl::{
    sdl::get_error,
    video::{
        ll::{
            SDL_Flip, SDL_LockSurface, SDL_MapRGBA, SDL_RWFromFile, SDL_Rect, SDL_SaveBMP_RW,
            SDL_SetVideoMode, SDL_Surface, SDL_UnlockSurface,
        },
        VideoFlag,
    },
};
use std::{
    os::raw::{c_char, c_float, c_int},
    ptr::null_mut,
};

extern "C" {
    pub static mut vid_bpp: c_int;
    pub fn FreeGraphics();
    pub fn putpixel(surface: *mut SDL_Surface, x: c_int, y: c_int, pixel: u32);
    pub fn GetRGBA(
        surface: &SDL_Surface,
        x: c_int,
        y: c_int,
        r: &mut u8,
        g: &mut u8,
        b: &mut u8,
        a: &mut u8,
    );

}

/// This function draws a "grid" on the screen, that means every
/// "second" pixel is blacked out, thereby generation a fading
/// effect.  This function was created to fade the background of the
/// Escape menu and its submenus.
#[no_mangle]
pub unsafe extern "C" fn MakeGridOnScreen(grid_rectangle: Option<&SDL_Rect>) {
    let grid_rectangle = grid_rectangle.unwrap_or(&User_Rect);

    trace!("MakeGridOnScreen(...): real function call confirmed.");
    SDL_LockSurface(ne_screen);
    let rect_x = i32::from(grid_rectangle.x);
    let rect_y = i32::from(grid_rectangle.y);
    (rect_y..(rect_y + i32::from(grid_rectangle.y)))
        .flat_map(|y| (rect_x..(rect_x + i32::from(grid_rectangle.w))).map(move |x| (x, y)))
        .filter(|(x, y)| (x + y) % 2 == 0)
        .for_each(|(x, y)| putpixel(ne_screen, x, y, 0));

    SDL_UnlockSurface(ne_screen);
    trace!("MakeGridOnScreen(...): end of function reached.");
}

#[no_mangle]
pub unsafe extern "C" fn ApplyFilter(
    surface: &mut SDL_Surface,
    fred: c_float,
    fgreen: c_float,
    fblue: c_float,
) -> c_int {
    let w = surface.w;
    (0..surface.h)
        .flat_map(move |y| (0..w).map(move |x| (x, y)))
        .for_each(|(x, y)| {
            let mut red = 0;
            let mut green = 0;
            let mut blue = 0;
            let mut alpha = 0;

            GetRGBA(surface, x, y, &mut red, &mut green, &mut blue, &mut alpha);
            if alpha == 0 {
                return;
            }

            red = (red as c_float * fred) as u8;
            green = (green as c_float * fgreen) as u8;
            blue = (blue as c_float * fblue) as u8;

            putpixel(
                surface,
                x,
                y,
                SDL_MapRGBA(surface.format, red, green, blue, alpha),
            );
        });

    defs::OK.into()
}

#[no_mangle]
pub unsafe extern "C" fn toggle_fullscreen() {
    let mut vid_flags = (*ne_screen).flags;

    if GameConfig.UseFullscreen != 0 {
        vid_flags &= !(VideoFlag::Fullscreen as u32);
    } else {
        vid_flags |= VideoFlag::Fullscreen as u32;
    }

    ne_screen = SDL_SetVideoMode(Screen_Rect.w.into(), Screen_Rect.h.into(), 0, vid_flags);
    if ne_screen.is_null() {
        error!(
            "unable to toggle windowed/fullscreen {} x {} video mode.",
            Screen_Rect.w, Screen_Rect.h,
        );
        error!("SDL-Error: {}", get_error());
        Terminate(defs::ERR.into());
    }

    if (*ne_screen).flags != vid_flags {
        warn!("Failed to toggle windowed/fullscreen mode!");
    } else {
        GameConfig.UseFullscreen = !GameConfig.UseFullscreen;
    }
}

/// This function saves a screenshot to disk.
///
/// The screenshots are names "Screenshot_XX.bmp" where XX is a
/// running number.
///
/// NOTE:  This function does NOT check for existing screenshots,
///        but will silently overwrite them.  No problem in most
///        cases I think.
#[no_mangle]
pub unsafe extern "C" fn TakeScreenshot() {
    static mut NUMBER_OF_SCREENSHOT: u32 = 0;

    Activate_Conservative_Frame_Computation();

    let screenshot_filename = format!("Screenshot_{}.bmp\0", NUMBER_OF_SCREENSHOT);
    SDL_SaveBMP_RW(
        ne_screen,
        SDL_RWFromFile(
            screenshot_filename.as_ptr() as *const c_char,
            cstr!("wb").as_ptr(),
        ),
        1,
    );
    NUMBER_OF_SCREENSHOT = NUMBER_OF_SCREENSHOT.wrapping_add(1);
    DisplayBanner(
        cstr!("Screenshot").as_ptr(),
        null_mut(),
        (DisplayBannerFlags::NO_SDL_UPDATE | DisplayBannerFlags::FORCE_UPDATE)
            .bits()
            .into(),
    );
    MakeGridOnScreen(None);
    SDL_Flip(ne_screen);
    Play_Sound(Sound::Screenshot as i32);

    while cmd_is_active(Cmds::Screenshot) {
        SDL_Delay(1);
    }

    DisplayBanner(
        null_mut(),
        null_mut(),
        DisplayBannerFlags::FORCE_UPDATE.bits().into(),
    );
}
