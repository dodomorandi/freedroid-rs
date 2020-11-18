use crate::{
    defs,
    global::{ne_screen, User_Rect},
};

use log::trace;
use sdl::video::ll::{SDL_LockSurface, SDL_MapRGBA, SDL_Rect, SDL_Surface, SDL_UnlockSurface};
use std::os::raw::{c_float, c_int};

extern "C" {
    pub static mut vid_bpp: c_int;
    pub fn toggle_fullscreen();
    pub fn TakeScreenshot();
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
