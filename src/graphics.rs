use crate::{
    defs::{
        self, scale_point, scale_rect, Cmds, DisplayBannerFlags, Droid, Sound, FREE_ONLY, INIT_ONLY,
    },
    global::{
        arrow_cursor, arrow_down, arrow_left, arrow_right, arrow_up, banner_pic, console_bg_pic1,
        console_bg_pic2, console_pic, crosshair_cursor, ne_screen, packed_portraits, pic999,
        progress_filler_pic, progress_meter_pic, ship_off_pic, ship_on_pic, takeover_bg_pic,
        to_blocks, Banner_Rect, Block_Rect, BuildBlock, CapsuleBlocks, Classic_User_Rect,
        ConsMenuItem_Rect, Cons_Droid_Rect, Cons_Header_Rect, Cons_Menu_Rect, Cons_Menu_Rects,
        Cons_Text_Rect, CurCapsuleStart, Decal_pics, Digit_Rect, DruidStart,
        EnemyDigitSurfacePointer, EnemySurfacePointer, FillBlocks, Font0_BFont, Font1_BFont,
        Font2_BFont, Full_User_Rect, GameConfig, Highscore_BFont, InfluDigitSurfacePointer,
        InfluencerSurfacePointer, LeftCapsulesStart, LeftInfo_Rect, Menu_BFont, Menu_Rect,
        OptionsMenu_Rect, OrigMapBlockSurfacePointer, Para_BFont, PlaygroundStart, Portrait_Rect,
        RightInfo_Rect, Screen_Rect, TO_CapsuleRect, TO_ColumnRect, TO_ColumnStart, TO_ElementRect,
        TO_FillBlock, TO_GroundRect, TO_LeaderBlockStart, TO_LeaderLed, TO_LeftGroundStart,
        TO_RightGroundStart, ToColumnBlock, ToGameBlocks, ToGroundBlocks, ToLeaderBlock, User_Rect,
    },
    input::{cmd_is_active, SDL_Delay},
    misc::{Activate_Conservative_Frame_Computation, Terminate},
    sound::Play_Sound,
    view::DisplayBanner,
};

use cstr::cstr;
use log::{error, trace, warn};
use sdl::{
    mouse::ll::SDL_FreeCursor,
    sdl::{get_error, Rect},
    video::SurfaceFlag,
    video::{
        ll::{
            SDL_CreateRGBSurface, SDL_DisplayFormat, SDL_DisplayFormatAlpha, SDL_Flip,
            SDL_FreeSurface, SDL_GetRGBA, SDL_LockSurface, SDL_MapRGBA, SDL_RWFromFile, SDL_RWops,
            SDL_Rect, SDL_SaveBMP_RW, SDL_SetAlpha, SDL_SetVideoMode, SDL_Surface,
            SDL_UnlockSurface, SDL_UpperBlit,
        },
        VideoFlag,
    },
};
use std::{
    convert::{TryFrom, TryInto},
    os::raw::{c_char, c_float, c_int, c_void},
    ptr::null_mut,
};

extern "C" {
    pub static mut vid_bpp: c_int;
    pub static mut portrait_raw_mem: [*mut c_char; Droid::NumDroids as usize];
    pub fn IMG_Load(file: *const c_char) -> *mut SDL_Surface;

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

#[inline]
unsafe fn free_surface_array(surfaces: &[*mut SDL_Surface]) {
    surfaces
        .iter()
        .for_each(|&surface| SDL_FreeSurface(surface));
}

#[no_mangle]
pub unsafe extern "C" fn FreeGraphics() {
    // free RWops structures
    packed_portraits
        .iter()
        .filter(|packed_portrait| !packed_portrait.is_null())
        .for_each(|&packed_portrait| {
            let close: unsafe extern "C" fn(context: *mut SDL_RWops) -> c_int =
                std::mem::transmute((*packed_portrait).close);
            close(packed_portrait);
        });

    portrait_raw_mem
        .iter()
        .for_each(|&mem| libc::free(mem as *mut c_void));

    SDL_FreeSurface(ne_screen);

    free_surface_array(&EnemySurfacePointer);
    free_surface_array(&InfluencerSurfacePointer);
    free_surface_array(&InfluDigitSurfacePointer);
    free_surface_array(&EnemyDigitSurfacePointer);
    free_surface_array(&Decal_pics);

    OrigMapBlockSurfacePointer
        .iter()
        .flat_map(|arr| arr.iter())
        .for_each(|&surface| SDL_FreeSurface(surface));

    SDL_FreeSurface(BuildBlock);
    SDL_FreeSurface(banner_pic);
    SDL_FreeSurface(pic999);
    // SDL_RWops *packed_portraits[NUM_DROIDS];
    SDL_FreeSurface(takeover_bg_pic);
    SDL_FreeSurface(console_pic);
    SDL_FreeSurface(console_bg_pic1);
    SDL_FreeSurface(console_bg_pic2);

    SDL_FreeSurface(arrow_up);
    SDL_FreeSurface(arrow_down);
    SDL_FreeSurface(arrow_right);
    SDL_FreeSurface(arrow_left);

    SDL_FreeSurface(ship_off_pic);
    SDL_FreeSurface(ship_on_pic);
    SDL_FreeSurface(progress_meter_pic);
    SDL_FreeSurface(progress_filler_pic);
    SDL_FreeSurface(to_blocks);

    // free fonts
    [
        Menu_BFont,
        Para_BFont,
        Highscore_BFont,
        Font0_BFont,
        Font1_BFont,
        Font2_BFont,
    ]
    .iter()
    .filter(|font| !font.is_null())
    .for_each(|&font| {
        SDL_FreeSurface((*font).surface);
        libc::free(font as *mut c_void);
    });

    // free Load_Block()-internal buffer
    Load_Block(null_mut(), 0, 0, null_mut(), FREE_ONLY as i32);

    // free cursors
    SDL_FreeCursor(crosshair_cursor);
    SDL_FreeCursor(arrow_cursor);
}

/// Set the pixel at (x, y) to the given value
/// NOTE: The surface must be locked before calling this!
#[no_mangle]
pub unsafe extern "C" fn putpixel(surface: *const SDL_Surface, x: c_int, y: c_int, pixel: u32) {
    if surface.is_null() || x < 0 || y < 0 {
        return;
    }

    let surface = &*surface;
    if (x >= surface.w) || (y < 0) || (y >= surface.h) {
        return;
    }

    let bpp = (*surface.format).BytesPerPixel.into();
    let data = (surface.pixels as *mut u8).offset((y * i32::from(surface.pitch)) as isize);

    match bpp {
        1 => *data.offset(x as isize) = pixel as u8,
        2 => *(data as *mut u16).offset(x as isize) = pixel as u16,
        3 => {
            let offset = isize::try_from(x).unwrap() * 3;
            let p = std::slice::from_raw_parts_mut(data.offset(offset), 3);
            if cfg!(target_endian = "big") {
                p[0] = ((pixel >> 16) & 0xff) as u8;
                p[1] = ((pixel >> 8) & 0xff) as u8;
                p[2] = (pixel & 0xff) as u8;
            } else {
                p[0] = (pixel & 0xff) as u8;
                p[1] = ((pixel >> 8) & 0xff) as u8;
                p[2] = ((pixel >> 16) & 0xff) as u8;
            }
        }
        4 => *(data as *mut u32).offset(x as isize) = pixel,
        _ => unreachable!(),
    }
}

/// This function gives the green component of a pixel, using a value of
/// 255 for the most green pixel and 0 for the least green pixel.
#[no_mangle]
pub unsafe extern "C" fn GetRGBA(
    surface: &SDL_Surface,
    x: c_int,
    y: c_int,
    red: &mut u8,
    green: &mut u8,
    blue: &mut u8,
    alpha: &mut u8,
) {
    let fmt = surface.format;
    let pixel = *((surface.pixels as *const u32)
        .add(usize::try_from(x).unwrap())
        .add(usize::try_from(y).unwrap() * usize::try_from(surface.w).unwrap()));

    SDL_GetRGBA(pixel, fmt, red, green, blue, alpha);
}

/// General block-reading routine: get block from pic-file
///
/// fpath: full pathname of picture-file; if NULL: use previous SDL-surf
/// line, col: block-position in pic-file to read block from
/// block: dimension of blocks to consider: if NULL: copy whole pic
/// NOTE: only w and h of block are used!!
///
/// NOTE: to avoid memory-leaks, use (flags | INIT_ONLY) if you only
///       call this function to set up a new pic-file to be read.
///       This will avoid copying & mallocing a new pic, NULL will be returned
#[no_mangle]
pub unsafe extern "C" fn Load_Block(
    fpath: *mut c_char,
    line: c_int,
    col: c_int,
    block: *mut SDL_Rect,
    flags: c_int,
) -> *mut SDL_Surface {
    static mut PIC: *mut SDL_Surface = null_mut();

    if fpath.is_null() && PIC.is_null() {
        /* we need some info.. */
        return null_mut();
    }

    if !PIC.is_null() && flags == FREE_ONLY as c_int {
        SDL_FreeSurface(PIC);
        return null_mut();
    }

    if !fpath.is_null() {
        // initialize: read & malloc new PIC, dont' return a copy!!

        if !PIC.is_null() {
            // previous PIC?
            SDL_FreeSurface(PIC);
        }
        PIC = IMG_Load(fpath);
    }

    if (flags & INIT_ONLY as c_int) != 0 {
        return null_mut(); // that's it guys, only initialzing...
    }

    assert!(!PIC.is_null());
    let pic = &mut *PIC;
    let dim = if block.is_null() {
        Rect::new(0, 0, pic.w.try_into().unwrap(), pic.h.try_into().unwrap())
    } else {
        let block = &*block;
        Rect::new(0, 0, block.w, block.h)
    };

    let usealpha = (*pic.format).Amask != 0;

    if usealpha {
        SDL_SetAlpha(pic, 0, 0); /* clear per-surf alpha for internal blit */
    }
    let tmp = SDL_CreateRGBSurface(0, dim.w.into(), dim.h.into(), vid_bpp, 0, 0, 0, 0);
    let ret = if usealpha {
        SDL_DisplayFormatAlpha(tmp)
    } else {
        SDL_DisplayFormat(tmp)
    };
    SDL_FreeSurface(tmp);

    let mut src = Rect::new(
        i16::try_from(col).unwrap() * i16::try_from(dim.w + 2).unwrap(),
        i16::try_from(line).unwrap() * i16::try_from(dim.h + 2).unwrap(),
        dim.w,
        dim.h,
    );
    SDL_UpperBlit(pic, &mut src, ret, null_mut());
    if usealpha {
        SDL_SetAlpha(
            ret,
            SurfaceFlag::SrcAlpha as u32 | SurfaceFlag::RLEAccel as u32,
            255,
        );
    }

    ret
}

/// scale all "static" rectangles, which are theme-independent
#[no_mangle]
pub unsafe extern "C" fn ScaleStatRects(scale: c_float) {
    macro_rules! scale {
        ($rect:ident) => {
            scale_rect(&mut $rect, scale);
        };
    }

    macro_rules! scale_point {
        ($rect:ident) => {
            scale_point(&mut $rect, scale);
        };
    }

    scale!(Block_Rect);
    scale!(User_Rect);
    scale!(Classic_User_Rect);
    scale!(Full_User_Rect);
    scale!(Banner_Rect);
    scale!(Portrait_Rect);
    scale!(Cons_Droid_Rect);
    scale!(Menu_Rect);
    scale!(OptionsMenu_Rect);
    scale!(Digit_Rect);
    scale!(Cons_Header_Rect);
    scale!(Cons_Menu_Rect);
    scale!(Cons_Text_Rect);

    for block in &mut Cons_Menu_Rects {
        scale_rect(block, scale);
    }

    scale!(ConsMenuItem_Rect);

    scale!(LeftInfo_Rect);
    scale!(RightInfo_Rect);

    for block in &mut FillBlocks {
        scale_rect(block, scale);
    }

    for block in &mut CapsuleBlocks {
        scale_rect(block, scale);
    }

    for block in &mut ToGameBlocks {
        scale_rect(block, scale);
    }

    for block in &mut ToGroundBlocks {
        scale_rect(block, scale);
    }

    scale!(ToColumnBlock);
    scale!(ToLeaderBlock);

    for point in &mut LeftCapsulesStart {
        scale_point(point, scale);
    }
    for point in &mut CurCapsuleStart {
        scale_point(point, scale);
    }
    for point in &mut PlaygroundStart {
        scale_point(point, scale);
    }
    for point in &mut DruidStart {
        scale_point(point, scale);
    }
    scale_point!(TO_LeftGroundStart);
    scale_point!(TO_ColumnStart);
    scale_point!(TO_RightGroundStart);
    scale_point!(TO_LeaderBlockStart);

    scale!(TO_FillBlock);
    scale!(TO_ElementRect);
    scale!(TO_CapsuleRect);
    scale!(TO_LeaderLed);
    scale!(TO_GroundRect);
    scale!(TO_ColumnRect);
}
