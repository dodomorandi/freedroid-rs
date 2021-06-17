use crate::{
    b_font::{put_pixel, BFontInfo},
    defs::{
        self, free_if_unused, scale_point, scale_rect, Cmds, Criticality, DisplayBannerFlags,
        Droid, SoundType, Themed, BANNER_BLOCK_FILE_C, BLAST_BLOCK_FILE_C, BULLET_BLOCK_FILE_C,
        CONSOLE_BG_PIC1_FILE_C, CONSOLE_BG_PIC2_FILE_C, CONSOLE_PIC_FILE_C, DIGITNUMBER,
        DIGIT_BLOCK_FILE_C, DROID_BLOCK_FILE_C, ENEMYPHASES, FONT0_FILE, FONT0_FILE_C, FONT1_FILE,
        FONT1_FILE_C, FONT2_FILE, FONT2_FILE_C, FREE_ONLY, GRAPHICS_DIR_C, ICON_FILE, ICON_FILE_C,
        INIT_ONLY, MAP_BLOCK_FILE_C, MAXBULLETS, MAX_THEMES, NUM_COLORS, NUM_DECAL_PICS,
        NUM_MAP_BLOCKS, PARA_FONT_FILE, PARA_FONT_FILE_C, SHIP_OFF_PIC_FILE_C, SHIP_ON_PIC_FILE_C,
        TAKEOVER_BG_PIC_FILE_C,
    },
    input::SDL_Delay,
    misc::read_value_from_string,
    structs::ThemeList,
    takeover::{
        set_takeover_rects, CAPSULE_BLOCKS, CAPSULE_RECT, COLUMN_BLOCK, COLUMN_RECT, COLUMN_START,
        CUR_CAPSULE_STARTS, DROID_STARTS, ELEMENT_RECT, FILL_BLOCK, FILL_BLOCKS, GROUND_RECT,
        LEADER_BLOCK, LEADER_BLOCK_START, LEADER_LED, LEFT_CAPSULE_STARTS, LEFT_GROUND_START,
        PLAYGROUND_STARTS, RIGHT_GROUND_START, TO_BLOCKS, TO_BLOCK_FILE_C, TO_GAME_BLOCKS,
        TO_GROUND_BLOCKS,
    },
    vars::{BLASTMAP, BULLETMAP, DRUIDMAP, ME, ORIG_BLOCK_RECT, ORIG_DIGIT_RECT},
    Data, ALL_BULLETS, FIRST_DIGIT_RECT, SECOND_DIGIT_RECT, THIRD_DIGIT_RECT,
};

use array_init::array_init;
use cstr::cstr;
use log::{error, info, trace, warn};
use sdl::{
    mouse::ll::{SDL_CreateCursor, SDL_Cursor, SDL_FreeCursor},
    sdl::{
        get_error,
        ll::{SDL_GetTicks, SDL_Init, SDL_InitSubSystem, SDL_Quit, SDL_INIT_TIMER, SDL_INIT_VIDEO},
        Rect,
    },
    video::SurfaceFlag,
    video::{
        ll::{
            SDL_ConvertSurface, SDL_CreateRGBSurface, SDL_DisplayFormat, SDL_DisplayFormatAlpha,
            SDL_FillRect, SDL_Flip, SDL_FreeSurface, SDL_GetRGBA, SDL_GetVideoInfo,
            SDL_LockSurface, SDL_MapRGB, SDL_MapRGBA, SDL_RWFromFile, SDL_RWops, SDL_Rect,
            SDL_SaveBMP_RW, SDL_SetAlpha, SDL_SetClipRect, SDL_SetGamma, SDL_SetVideoMode,
            SDL_Surface, SDL_UnlockSurface, SDL_UpdateRect, SDL_UpperBlit, SDL_VideoInfo,
        },
        VideoFlag, VideoInfoFlag,
    },
    wm::ll::{SDL_WM_SetCaption, SDL_WM_SetIcon},
};
use std::{
    alloc::{alloc_zeroed, dealloc, Layout},
    convert::{TryFrom, TryInto},
    ffi::CStr,
    os::raw::{c_char, c_double, c_float, c_int, c_short, c_void},
    ptr::null_mut,
};

pub static mut VID_INFO: *const SDL_VideoInfo = null_mut();
pub static mut VID_BPP: c_int = 0;
pub static mut PORTRAIT_RAW_MEM: once_cell::sync::Lazy<
    [Option<Box<[u8]>>; Droid::NumDroids as usize],
> = once_cell::sync::Lazy::new(|| array_init::array_init(|_| None));
pub static mut FONTS_LOADED: c_int = 0;
pub static mut MAP_BLOCK_SURFACE_POINTER: [[*mut SDL_Surface; NUM_MAP_BLOCKS]; NUM_COLORS] =
    [[null_mut(); NUM_MAP_BLOCKS]; NUM_COLORS]; // A pointer to the surfaces containing the map-pics, which may be rescaled with respect to
pub static mut ORIG_MAP_BLOCK_SURFACE_POINTER: [[*mut SDL_Surface; NUM_MAP_BLOCKS]; NUM_COLORS] =
    [[null_mut(); NUM_MAP_BLOCKS]; NUM_COLORS]; // A pointer to the surfaces containing the original map-pics as read from disk
pub static mut BUILD_BLOCK: *mut SDL_Surface = null_mut(); // a block for temporary pic-construction
pub static mut BANNER_IS_DESTROYED: i32 = 0;
pub static mut BANNER_PIC: *mut SDL_Surface = null_mut(); /* the banner pic */
pub static mut PIC999: *mut SDL_Surface = null_mut();
pub static mut PACKED_PORTRAITS: [*mut SDL_RWops; Droid::NumDroids as usize] =
    [null_mut(); Droid::NumDroids as usize];
pub static mut DECAL_PICS: [*mut SDL_Surface; NUM_DECAL_PICS] = [null_mut(); NUM_DECAL_PICS];
pub static mut TAKEOVER_BG_PIC: *mut SDL_Surface = null_mut();
pub static mut CONSOLE_PIC: *mut SDL_Surface = null_mut();
pub static mut CONSOLE_BG_PIC1: *mut SDL_Surface = null_mut();
pub static mut CONSOLE_BG_PIC2: *mut SDL_Surface = null_mut();
pub static mut ARROW_UP: *mut SDL_Surface = null_mut();
pub static mut ARROW_DOWN: *mut SDL_Surface = null_mut();
pub static mut ARROW_RIGHT: *mut SDL_Surface = null_mut();
pub static mut ARROW_LEFT: *mut SDL_Surface = null_mut();
pub static mut SHIP_OFF_PIC: *mut SDL_Surface = null_mut(); /* Side-view of ship: lights off */
pub static mut SHIP_ON_PIC: *mut SDL_Surface = null_mut(); /* Side-view of ship: lights on */
pub static mut PROGRESS_METER_PIC: *mut SDL_Surface = null_mut();
pub static mut PROGRESS_FILLER_PIC: *mut SDL_Surface = null_mut();
pub static mut NE_SCREEN: *mut SDL_Surface = null_mut(); /* the graphics display */
pub static mut ENEMY_SURFACE_POINTER: [*mut SDL_Surface; ENEMYPHASES as usize] =
    [null_mut(); ENEMYPHASES as usize]; // A pointer to the surfaces containing the pictures of the
pub static mut INFLUENCER_SURFACE_POINTER: [*mut SDL_Surface; ENEMYPHASES as usize] =
    [null_mut(); ENEMYPHASES as usize]; // A pointer to the surfaces containing the pictures of the
pub static mut INFLU_DIGIT_SURFACE_POINTER: [*mut SDL_Surface; DIGITNUMBER] =
    [null_mut(); DIGITNUMBER]; // A pointer to the surfaces containing the pictures of the
pub static mut ENEMY_DIGIT_SURFACE_POINTER: [*mut SDL_Surface; DIGITNUMBER] =
    [null_mut(); DIGITNUMBER]; // A pointer to the surfaces containing the pictures of the
pub static mut CROSSHAIR_CURSOR: *mut SDL_Cursor = null_mut();
pub static mut ARROW_CURSOR: *mut SDL_Cursor = null_mut();
pub static mut NUMBER_OF_BULLET_TYPES: i32 = 0;
pub static mut ALL_THEMES: ThemeList = ThemeList {
    num_themes: 0,
    cur_tnum: 0,
    theme_name: [null_mut(); MAX_THEMES],
};
pub static mut CLASSIC_THEME_INDEX: i32 = 0;

#[link(name = "SDL_image")]
extern "C" {
    pub fn IMG_Load(file: *const c_char) -> *mut SDL_Surface;
}

extern "C" {
    pub fn zoomSurface(
        src: *mut SDL_Surface,
        zoomx: c_double,
        zoomy: c_double,
        smooth: c_int,
    ) -> *mut SDL_Surface;
    pub fn SDL_GetClipRect(surface: *mut SDL_Surface, rect: *mut SDL_Rect);
    pub fn SDL_VideoDriverName(namebuf: *mut c_char, maxlen: c_int) -> *mut c_char;
    pub fn SDL_RWFromMem(mem: *mut c_void, size: c_int) -> *mut SDL_RWops;
}

impl Data {
    /// This function draws a "grid" on the screen, that means every
    /// "second" pixel is blacked out, thereby generation a fading
    /// effect.  This function was created to fade the background of the
    /// Escape menu and its submenus.
    pub unsafe fn make_grid_on_screen(&self, grid_rectangle: Option<&SDL_Rect>) {
        let grid_rectangle = grid_rectangle.unwrap_or(&self.vars.user_rect);

        trace!("MakeGridOnScreen(...): real function call confirmed.");
        SDL_LockSurface(NE_SCREEN);
        let rect_x = i32::from(grid_rectangle.x);
        let rect_y = i32::from(grid_rectangle.y);
        (rect_y..(rect_y + i32::from(grid_rectangle.y)))
            .flat_map(|y| (rect_x..(rect_x + i32::from(grid_rectangle.w))).map(move |x| (x, y)))
            .filter(|(x, y)| (x + y) % 2 == 0)
            .for_each(|(x, y)| putpixel(NE_SCREEN, x, y, 0));

        SDL_UnlockSurface(NE_SCREEN);
        trace!("MakeGridOnScreen(...): end of function reached.");
    }
}

pub unsafe fn apply_filter(
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

            get_rgba(surface, x, y, &mut red, &mut green, &mut blue, &mut alpha);
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

impl Data {
    pub unsafe fn toggle_fullscreen(&mut self) {
        let mut vid_flags = (*NE_SCREEN).flags;

        if self.global.game_config.use_fullscreen != 0 {
            vid_flags &= !(VideoFlag::Fullscreen as u32);
        } else {
            vid_flags |= VideoFlag::Fullscreen as u32;
        }

        NE_SCREEN = SDL_SetVideoMode(
            self.vars.screen_rect.w.into(),
            self.vars.screen_rect.h.into(),
            0,
            vid_flags,
        );
        if NE_SCREEN.is_null() {
            error!(
                "unable to toggle windowed/fullscreen {} x {} video mode.",
                self.vars.screen_rect.w, self.vars.screen_rect.h,
            );
            panic!("SDL-Error: {}", get_error());
        }

        if (*NE_SCREEN).flags != vid_flags {
            warn!("Failed to toggle windowed/fullscreen mode!");
        } else {
            self.global.game_config.use_fullscreen = !self.global.game_config.use_fullscreen;
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
    pub unsafe fn take_screenshot(&mut self) {
        static mut NUMBER_OF_SCREENSHOT: u32 = 0;

        self.activate_conservative_frame_computation();

        let screenshot_filename = format!("Screenshot_{}.bmp\0", NUMBER_OF_SCREENSHOT);
        SDL_SaveBMP_RW(
            NE_SCREEN,
            SDL_RWFromFile(
                screenshot_filename.as_ptr() as *const c_char,
                cstr!("wb").as_ptr(),
            ),
            1,
        );
        NUMBER_OF_SCREENSHOT = NUMBER_OF_SCREENSHOT.wrapping_add(1);
        self.display_banner(
            cstr!("Screenshot").as_ptr(),
            null_mut(),
            (DisplayBannerFlags::NO_SDL_UPDATE | DisplayBannerFlags::FORCE_UPDATE)
                .bits()
                .into(),
        );
        self.make_grid_on_screen(None);
        SDL_Flip(NE_SCREEN);
        self.play_sound(SoundType::Screenshot as i32);

        while self.cmd_is_active(Cmds::Screenshot) {
            SDL_Delay(1);
        }

        self.display_banner(
            null_mut(),
            null_mut(),
            DisplayBannerFlags::FORCE_UPDATE.bits().into(),
        );
    }
}

#[inline]
unsafe fn free_surface_array(surfaces: &[*mut SDL_Surface]) {
    surfaces
        .iter()
        .for_each(|&surface| SDL_FreeSurface(surface));
}

impl Data {
    pub unsafe fn free_graphics(&self) {
        // free RWops structures
        PACKED_PORTRAITS
            .iter()
            .filter(|packed_portrait| !packed_portrait.is_null())
            .for_each(|&packed_portrait| {
                let close: unsafe fn(context: *mut SDL_RWops) -> c_int =
                    std::mem::transmute((*packed_portrait).close);
                close(packed_portrait);
            });

        PORTRAIT_RAW_MEM.iter_mut().for_each(|mem| drop(mem.take()));

        SDL_FreeSurface(NE_SCREEN);

        free_surface_array(&ENEMY_SURFACE_POINTER);
        free_surface_array(&INFLUENCER_SURFACE_POINTER);
        free_surface_array(&INFLU_DIGIT_SURFACE_POINTER);
        free_surface_array(&ENEMY_DIGIT_SURFACE_POINTER);
        free_surface_array(&DECAL_PICS);

        ORIG_MAP_BLOCK_SURFACE_POINTER
            .iter()
            .flat_map(|arr| arr.iter())
            .for_each(|&surface| SDL_FreeSurface(surface));

        SDL_FreeSurface(BUILD_BLOCK);
        SDL_FreeSurface(BANNER_PIC);
        SDL_FreeSurface(PIC999);
        // SDL_RWops *packed_portraits[NUM_DROIDS];
        SDL_FreeSurface(TAKEOVER_BG_PIC);
        SDL_FreeSurface(CONSOLE_PIC);
        SDL_FreeSurface(CONSOLE_BG_PIC1);
        SDL_FreeSurface(CONSOLE_BG_PIC2);

        SDL_FreeSurface(ARROW_UP);
        SDL_FreeSurface(ARROW_DOWN);
        SDL_FreeSurface(ARROW_RIGHT);
        SDL_FreeSurface(ARROW_LEFT);

        SDL_FreeSurface(SHIP_OFF_PIC);
        SDL_FreeSurface(SHIP_ON_PIC);
        SDL_FreeSurface(PROGRESS_METER_PIC);
        SDL_FreeSurface(PROGRESS_FILLER_PIC);
        SDL_FreeSurface(TO_BLOCKS);

        // free fonts
        [
            self.global.menu_b_font,
            self.global.para_b_font,
            self.global.highscore_b_font,
            self.global.font0_b_font,
            self.global.font1_b_font,
            self.global.font2_b_font,
        ]
        .iter()
        .filter(|font| !font.is_null())
        .for_each(|&font| {
            SDL_FreeSurface((*font).surface);
            dealloc(font as *mut u8, Layout::new::<BFontInfo>());
        });

        // free Load_Block()-internal buffer
        load_block(null_mut(), 0, 0, null_mut(), FREE_ONLY as i32);

        // free cursors
        SDL_FreeCursor(CROSSHAIR_CURSOR);
        SDL_FreeCursor(ARROW_CURSOR);
    }
}

/// Set the pixel at (x, y) to the given value
/// NOTE: The surface must be locked before calling this!
pub unsafe fn putpixel(surface: *const SDL_Surface, x: c_int, y: c_int, pixel: u32) {
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
pub unsafe fn get_rgba(
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
pub unsafe fn load_block(
    fpath: *mut c_char,
    line: c_int,
    col: c_int,
    block: *const SDL_Rect,
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
    let tmp = SDL_CreateRGBSurface(0, dim.w.into(), dim.h.into(), VID_BPP, 0, 0, 0, 0);
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

impl Data {
    /// scale all "static" rectangles, which are theme-independent
    pub unsafe fn scale_stat_rects(&mut self, scale: c_float) {
        macro_rules! scale {
            ($rect:expr) => {
                scale_rect(&mut $rect, scale);
            };
        }

        macro_rules! scale_point {
            ($rect:ident) => {
                scale_point(&mut $rect, scale);
            };
        }

        scale!(self.vars.block_rect);
        scale!(self.vars.user_rect);
        scale!(self.vars.classic_user_rect);
        scale!(self.vars.full_user_rect);
        scale!(self.vars.banner_rect);
        scale!(self.vars.portrait_rect);
        scale!(self.vars.cons_droid_rect);
        scale!(self.vars.menu_rect);
        scale!(self.vars.options_menu_rect);
        scale!(self.vars.digit_rect);
        scale!(self.vars.cons_header_rect);
        scale!(self.vars.cons_menu_rect);
        scale!(self.vars.cons_text_rect);

        for block in &mut self.vars.cons_menu_rects {
            scale_rect(block, scale);
        }

        scale!(self.vars.cons_menu_item_rect);

        scale!(self.vars.left_info_rect);
        scale!(self.vars.right_info_rect);

        for block in &mut FILL_BLOCKS {
            scale_rect(block, scale);
        }

        for block in &mut CAPSULE_BLOCKS {
            scale_rect(block, scale);
        }

        for block in &mut *TO_GAME_BLOCKS.lock().unwrap() {
            scale_rect(block, scale);
        }

        for block in &mut *TO_GROUND_BLOCKS.lock().unwrap() {
            scale_rect(block, scale);
        }

        scale!(COLUMN_BLOCK);
        scale!(LEADER_BLOCK);

        for point in &mut LEFT_CAPSULE_STARTS {
            scale_point(point, scale);
        }
        for point in &mut CUR_CAPSULE_STARTS {
            scale_point(point, scale);
        }
        for point in &mut PLAYGROUND_STARTS {
            scale_point(point, scale);
        }
        for point in &mut DROID_STARTS {
            scale_point(point, scale);
        }
        scale_point!(LEFT_GROUND_START);
        scale_point!(COLUMN_START);
        scale_point!(RIGHT_GROUND_START);
        scale_point!(LEADER_BLOCK_START);

        scale!(FILL_BLOCK);
        scale!(ELEMENT_RECT);
        scale!(CAPSULE_RECT);
        scale!(LEADER_LED);
        scale!(GROUND_RECT);
        scale!(COLUMN_RECT);
    }
}

pub unsafe fn scale_pic(pic: &mut *mut SDL_Surface, scale: c_float) {
    if (scale - 1.0).abs() <= f32::EPSILON {
        return;
    }
    let scale = scale.into();

    let tmp = *pic;
    *pic = zoomSurface(tmp, scale, scale, 0);
    if pic.is_null() {
        panic!("zoomSurface() failed for scale = {}.", scale);
    }
    SDL_FreeSurface(tmp);
}

impl Data {
    pub unsafe fn scale_graphics(&mut self, scale: c_float) {
        static INIT: std::sync::Once = std::sync::Once::new();

        /* For some reason we need to SetAlpha every time on OS X */
        /* Digits are only used in _internal_ blits ==> clear per-surf alpha */
        for &surface in &INFLU_DIGIT_SURFACE_POINTER {
            SDL_SetAlpha(surface, 0, 0);
        }
        for &surface in &ENEMY_DIGIT_SURFACE_POINTER {
            SDL_SetAlpha(surface, 0, 0);
        }
        if (scale - 1.).abs() <= f32::EPSILON {
            return;
        }

        // these are reset in a theme-change by the theme-config-file
        // therefore we need to rescale them each time again
        scale_rect(&mut FIRST_DIGIT_RECT, scale);
        scale_rect(&mut SECOND_DIGIT_RECT, scale);
        scale_rect(&mut THIRD_DIGIT_RECT, scale);

        // note: only rescale these rects the first time!!
        let mut init = false;
        INIT.call_once(|| {
            init = true;
            self.scale_stat_rects(scale);
        });

        //---------- rescale Map blocks
        ORIG_MAP_BLOCK_SURFACE_POINTER
            .iter_mut()
            .flat_map(|surfaces| surfaces.iter_mut())
            .zip(
                MAP_BLOCK_SURFACE_POINTER
                    .iter_mut()
                    .flat_map(|surfaces| surfaces.iter_mut()),
            )
            .for_each(|(orig_surface, map_surface)| {
                scale_pic(orig_surface, scale);
                *map_surface = *orig_surface;
            });

        //---------- rescale Droid-model  blocks
        /* Droid pics are only used in _internal_ blits ==> clear per-surf alpha */
        for surface in &mut INFLUENCER_SURFACE_POINTER {
            scale_pic(surface, scale);
            SDL_SetAlpha(*surface, 0, 0);
        }
        for surface in &mut ENEMY_SURFACE_POINTER {
            scale_pic(surface, scale);
            SDL_SetAlpha(*surface, 0, 0);
        }

        //---------- rescale Bullet blocks
        let bulletmap = std::slice::from_raw_parts_mut(
            BULLETMAP,
            usize::try_from(NUMBER_OF_BULLET_TYPES).unwrap(),
        );
        bulletmap
            .iter_mut()
            .flat_map(|bullet| bullet.surface_pointer.iter_mut())
            .for_each(|surface| scale_pic(surface, scale));

        //---------- rescale Blast blocks
        BLASTMAP
            .iter_mut()
            .flat_map(|blast| blast.surface_pointer.iter_mut())
            .for_each(|surface| scale_pic(surface, scale));

        //---------- rescale Digit blocks
        for surface in &mut INFLU_DIGIT_SURFACE_POINTER {
            scale_pic(surface, scale);
            SDL_SetAlpha(*surface, 0, 0);
        }
        for surface in &mut ENEMY_DIGIT_SURFACE_POINTER {
            scale_pic(surface, scale);
            SDL_SetAlpha(*surface, 0, 0);
        }

        //---------- rescale Takeover pics
        scale_pic(&mut TO_BLOCKS, scale);

        scale_pic(&mut SHIP_ON_PIC, scale);
        scale_pic(&mut SHIP_OFF_PIC, scale);

        // the following are not theme-specific and are therefore only loaded once!
        if init {
            //  create a new tmp block-build storage
            free_if_unused(BUILD_BLOCK);
            let tmp = SDL_CreateRGBSurface(
                0,
                self.vars.block_rect.w.into(),
                self.vars.block_rect.h.into(),
                VID_BPP,
                0,
                0,
                0,
                0,
            );
            BUILD_BLOCK = SDL_DisplayFormatAlpha(tmp);
            SDL_FreeSurface(tmp);

            // takeover pics
            scale_pic(&mut TAKEOVER_BG_PIC, scale);

            //---------- Console pictures
            scale_pic(&mut CONSOLE_PIC, scale);
            scale_pic(&mut CONSOLE_BG_PIC1, scale);
            scale_pic(&mut CONSOLE_BG_PIC2, scale);
            scale_pic(&mut ARROW_UP, scale);
            scale_pic(&mut ARROW_DOWN, scale);
            scale_pic(&mut ARROW_RIGHT, scale);
            scale_pic(&mut ARROW_LEFT, scale);
            //---------- Banner
            scale_pic(&mut BANNER_PIC, scale);

            scale_pic(&mut PIC999, scale);

            // get the Ashes pics
            if !DECAL_PICS[0].is_null() {
                scale_pic(&mut DECAL_PICS[0], scale);
            }
            if !DECAL_PICS[1].is_null() {
                scale_pic(&mut DECAL_PICS[1], scale);
            }
        }

        self.printf_sdl(NE_SCREEN, -1, -1, format_args!(" ok\n"));
    }

    /// display "white noise" effect in Rect.
    /// algorith basically stolen from
    /// Greg Knauss's "xteevee" hack in xscreensavers.
    ///
    /// timeout is in ms
    pub unsafe fn white_noise(
        &mut self,
        bitmap: *mut SDL_Surface,
        rect: &mut Rect,
        timeout: c_int,
    ) {
        use rand::{
            seq::{IteratorRandom, SliceRandom},
            Rng,
        };
        const NOISE_COLORS: usize = 6;
        const NOISE_TILES: usize = 8;

        let signal_strengh = 60;

        let grey: [u32; NOISE_COLORS] = array_init(|index| {
            let color = (((index as f64 + 1.0) / (NOISE_COLORS as f64)) * 255.0) as u8;
            SDL_MapRGB((*NE_SCREEN).format, color, color, color)
        });

        // produce the tiles
        let tmp = SDL_CreateRGBSurface(0, rect.w.into(), rect.h.into(), VID_BPP, 0, 0, 0, 0);
        let tmp2 = SDL_DisplayFormat(tmp);
        SDL_FreeSurface(tmp);
        SDL_UpperBlit(bitmap, rect, tmp2, null_mut());

        let mut rng = rand::thread_rng();
        let noise_tiles: [*mut SDL_Surface; NOISE_TILES] = array_init(|_| {
            let tile = SDL_DisplayFormat(tmp2);
            (0..rect.x)
                .flat_map(|x| (0..rect.h).map(move |y| (x, y)))
                .for_each(|(x, y)| {
                    if rng.gen_range(0, 100) > signal_strengh {
                        put_pixel(&*tile, x.into(), y.into(), *grey.choose(&mut rng).unwrap());
                    }
                });
            tile
        });
        SDL_FreeSurface(tmp2);

        let mut used_tiles: [c_char; NOISE_TILES / 2 + 1] = [-1; NOISE_TILES / 2 + 1];
        // let's go
        self.play_sound(SoundType::WhiteNoise as c_int);

        let now = SDL_GetTicks();

        self.wait_for_all_keys_released();
        let mut clip_rect = Rect::new(0, 0, 0, 0);
        loop {
            // pick an old enough tile
            let mut next_tile;
            loop {
                next_tile = (0..NOISE_TILES as i8).choose(&mut rng).unwrap();
                for &used_tile in &used_tiles {
                    if next_tile == used_tile {
                        next_tile = -1;
                        break;
                    }
                }

                if next_tile != -1 {
                    break;
                }
            }
            used_tiles.copy_within(1.., 0);
            *used_tiles.last_mut().unwrap() = next_tile;

            // make sure we can blit the full rect without clipping! (would change *rect!)
            SDL_GetClipRect(NE_SCREEN, &mut clip_rect);
            SDL_SetClipRect(NE_SCREEN, null_mut());
            // set it
            SDL_UpperBlit(
                noise_tiles[usize::try_from(next_tile).unwrap()],
                null_mut(),
                NE_SCREEN,
                rect,
            );
            SDL_UpdateRect(
                NE_SCREEN,
                rect.x.into(),
                rect.y.into(),
                rect.w.into(),
                rect.h.into(),
            );
            SDL_Delay(25);

            if timeout != 0 && SDL_GetTicks() - now > timeout.try_into().unwrap() {
                break;
            }

            if self.any_key_just_pressed() != 0 {
                break;
            }
        }

        //restore previous clip-rectange
        SDL_SetClipRect(NE_SCREEN, &clip_rect);

        for &tile in &noise_tiles {
            SDL_FreeSurface(tile);
        }
    }

    pub unsafe fn duplicate_font(&mut self, in_font: &BFontInfo) -> *mut BFontInfo {
        let out_font = alloc_zeroed(Layout::new::<BFontInfo>()) as *mut BFontInfo;

        std::ptr::copy_nonoverlapping(in_font, out_font, 1);
        (*out_font).surface = SDL_ConvertSurface(
            in_font.surface,
            (*in_font.surface).format,
            (*in_font.surface).flags,
        );
        if (*out_font).surface.is_null() {
            panic!("Duplicate_Font: failed to copy SDL_Surface using SDL_ConvertSurface()");
        }

        out_font
    }

    pub unsafe fn load_fonts(&mut self) -> c_int {
        let mut fpath = self.find_file(
            PARA_FONT_FILE_C.as_ptr(),
            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
            Themed::NoTheme as c_int,
            Criticality::Critical as c_int,
        );
        self.global.para_b_font = self.load_font(fpath, self.global.game_config.scale);
        if self.global.para_b_font.is_null() {
            panic!("font file named {} was not found.", PARA_FONT_FILE);
        }

        fpath = self.find_file(
            FONT0_FILE_C.as_ptr(),
            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
            Themed::NoTheme as c_int,
            Criticality::Critical as c_int,
        );
        self.global.font0_b_font = self.load_font(fpath, self.global.game_config.scale);
        if self.global.font0_b_font.is_null() {
            panic!("font file named {} was not found.\n", FONT0_FILE);
        }

        fpath = self.find_file(
            FONT1_FILE_C.as_ptr(),
            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
            Themed::NoTheme as c_int,
            Criticality::Critical as c_int,
        );
        self.global.font1_b_font = self.load_font(fpath, self.global.game_config.scale);
        if self.global.font1_b_font.is_null() {
            panic!("font file named {} was not found.", FONT1_FILE);
        }

        fpath = self.find_file(
            FONT2_FILE_C.as_ptr(),
            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
            Themed::NoTheme as c_int,
            Criticality::Critical as c_int,
        );
        self.global.font2_b_font = self.load_font(fpath, self.global.game_config.scale);
        if self.global.font2_b_font.is_null() {
            panic!("font file named {} was not found.", FONT2_FILE);
        }

        self.global.menu_b_font = self.duplicate_font(&*self.global.para_b_font);
        self.global.highscore_b_font = self.duplicate_font(&*self.global.para_b_font);

        FONTS_LOADED = true.into();

        defs::OK.into()
    }
}

pub unsafe fn clear_graph_mem() {
    // One this function is done, the rahmen at the
    // top of the screen surely is destroyed.  We inform the
    // DisplayBanner function of the matter...
    BANNER_IS_DESTROYED = true.into();

    SDL_SetClipRect(NE_SCREEN, null_mut());

    // Now we fill the screen with black color...
    SDL_FillRect(NE_SCREEN, null_mut(), 0);
    SDL_Flip(NE_SCREEN);
}

impl Data {
    /// Initialise the Video display and graphics engine
    pub unsafe fn init_video(&mut self) {
        const YN: [&str; 2] = ["no", "yes"];

        /* Initialize the SDL library */
        // if ( SDL_Init (SDL_INIT_VIDEO | SDL_INIT_TIMER) == -1 )

        if SDL_Init(SDL_INIT_VIDEO) == -1 {
            panic!("Couldn't initialize SDL: {}", get_error());
        } else {
            info!("SDL Video initialisation successful.");
        }

        // Now SDL_TIMER is initialized here:

        if SDL_InitSubSystem(SDL_INIT_TIMER) == -1 {
            panic!("Couldn't initialize SDL: {}", get_error());
        } else {
            info!("SDL Timer initialisation successful.");
        }

        /* clean up on exit */
        libc::atexit(std::mem::transmute(SDL_Quit as unsafe extern "C" fn()));

        VID_INFO = SDL_GetVideoInfo(); /* just curious */
        let mut vid_driver: [c_char; 81] = [0; 81];
        SDL_VideoDriverName(vid_driver.as_mut_ptr(), 80);

        let vid_info_ref = *VID_INFO;
        if cfg!(os_target = "android") {
            VID_BPP = 16; // Hardcoded Android default
        } else {
            VID_BPP = (*vid_info_ref.vfmt).BitsPerPixel.into();
        }

        macro_rules! flag {
            ($flag:ident) => {
                (vid_info_ref.flags & VideoInfoFlag::$flag as u32) != 0
            };
        }
        macro_rules! flag_yn {
            ($flag:ident) => {
                YN[usize::from(flag!($flag))]
            };
        }

        info!("Video info summary from SDL:");
        info!("----------------------------------------------------------------------");
        info!(
            "Is it possible to create hardware surfaces: {}",
            flag_yn!(HWAvailable)
        );
        info!(
            "Is there a window manager available: {}",
            flag_yn!(WMAvailable)
        );
        info!(
            "Are hardware to hardware blits accelerated: {}",
            flag_yn!(BlitHW)
        );
        info!(
            "Are hardware to hardware colorkey blits accelerated: {}",
            flag_yn!(BlitHWColorkey)
        );
        info!(
            "Are hardware to hardware alpha blits accelerated: {}",
            flag_yn!(BlitHWAlpha)
        );
        info!(
            "Are software to hardware blits accelerated: {}",
            flag_yn!(BlitSW)
        );
        info!(
            "Are software to hardware colorkey blits accelerated: {}",
            flag_yn!(BlitSWColorkey)
        );
        info!(
            "Are software to hardware alpha blits accelerated: {}",
            flag_yn!(BlitSWAlpha)
        );
        info!("Are color fills accelerated: {}", flag_yn!(BlitFill));
        info!(
            "Total amount of video memory in Kilobytes: {}",
            vid_info_ref.video_mem
        );
        info!(
            "Pixel format of the video device: bpp = {}, bytes/pixel = {}",
            VID_BPP,
            (*vid_info_ref.vfmt).BytesPerPixel
        );
        info!(
            "Video Driver Name: {}",
            CStr::from_ptr(vid_driver.as_ptr()).to_string_lossy()
        );
        info!("----------------------------------------------------------------------");

        let vid_flags = if self.global.game_config.use_fullscreen != 0 {
            VideoFlag::Fullscreen as u32
        } else {
            0
        };

        if flag!(WMAvailable) {
            /* if there's a window-manager */
            SDL_WM_SetCaption(cstr!("Freedroid").as_ptr(), cstr!("").as_ptr());
            let fpath = self.find_file(
                ICON_FILE_C.as_ptr() as *mut c_char,
                GRAPHICS_DIR_C.as_ptr() as *mut c_char,
                Themed::NoTheme as c_int,
                Criticality::WarnOnly as c_int,
            );
            if fpath.is_null() {
                warn!("Could not find icon file '{}'", ICON_FILE);
            } else {
                let img = IMG_Load(fpath);
                if img.is_null() {
                    warn!(
                        "IMG_Load failed for icon file '{}'\n",
                        CStr::from_ptr(fpath).to_string_lossy()
                    );
                } else {
                    SDL_WM_SetIcon(img, null_mut());
                    SDL_FreeSurface(img);
                }
            }
        }

        NE_SCREEN = SDL_SetVideoMode(
            self.vars.screen_rect.w.into(),
            self.vars.screen_rect.h.into(),
            0,
            vid_flags,
        );
        if NE_SCREEN.is_null() {
            error!(
                "Couldn't set {} x {} video mode. SDL: {}",
                self.vars.screen_rect.w,
                self.vars.screen_rect.h,
                get_error(),
            );
            std::process::exit(-1);
        }

        VID_INFO = SDL_GetVideoInfo(); /* info about current video mode */

        info!("Got video mode: ");

        SDL_SetGamma(1., 1., 1.);
        self.global.game_config.current_gamma_correction = 1.;
    }

    /// load a pic into memory and return the SDL_RWops pointer to it
    pub unsafe fn load_raw_pic(
        &mut self,
        fpath: *const c_char,
        raw_mem: &mut Option<Box<[u8]>>,
    ) -> *mut SDL_RWops {
        use std::{fs::File, io::Read, path::Path};

        // sanity check
        if fpath.is_null() {
            panic!("load_raw_pic() called with NULL argument!");
        }

        let fpath = match CStr::from_ptr(fpath).to_str() {
            Ok(fpath) => fpath,
            Err(err) => {
                panic!("unable to convert path with invalid UTF-8 data: {}", err);
            }
        };
        let fpath = Path::new(&fpath);
        let mut file = match File::open(fpath) {
            Ok(file) => file,
            Err(_) => {
                panic!("could not open file {}. Giving up", fpath.display());
            }
        };

        let metadata = match file.metadata() {
            Ok(metadata) => metadata,
            Err(err) => {
                panic!("unable to get file metadata: {}", err);
            }
        };

        let len = metadata.len().try_into().unwrap();
        let mut buf = vec![0; len].into_boxed_slice();
        if file.read_exact(&mut *buf).is_err() {
            panic!("cannot reading file {}. Giving up...", fpath.display());
        }
        drop(file);

        let ops = SDL_RWFromMem(buf.as_mut_ptr() as *mut c_void, len.try_into().unwrap());
        *raw_mem = Some(buf);
        ops
    }

    /// Get the pics for: druids, bullets, blasts
    ///
    /// reads all blocks and puts the right pointers into
    /// the various structs
    ///
    /// Returns true/false
    pub unsafe fn init_pictures(&mut self) -> c_int {
        use std::sync::Once;

        static DO_ONCE: Once = Once::new();
        let mut fname: [c_char; 500] = [0; 500];

        // Loading all these pictures might take a while...
        // and we do not want do deal with huge frametimes, which
        // could box the influencer out of the ship....
        self.activate_conservative_frame_computation();

        let oldfont = self.b_font.current_font;

        if FONTS_LOADED == 0 {
            self.load_fonts();
        }

        self.b_font.current_font = self.global.font0_b_font;

        self.init_progress(cstr!("Loading pictures").as_ptr() as *mut c_char);

        self.load_theme_configuration_file();

        self.update_progress(15);

        //---------- get Map blocks
        let fpath = self.find_file(
            MAP_BLOCK_FILE_C.as_ptr(),
            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
            Themed::UseTheme as c_int,
            Criticality::Critical as c_int,
        );
        load_block(fpath, 0, 0, null_mut(), INIT_ONLY as i32); /* init function */
        ORIG_MAP_BLOCK_SURFACE_POINTER
            .iter_mut()
            .enumerate()
            .zip(MAP_BLOCK_SURFACE_POINTER.iter_mut())
            .flat_map(|((color_index, orig_color_map), color_map)| {
                orig_color_map
                    .iter_mut()
                    .enumerate()
                    .map(move |(block_index, orig_surface)| {
                        (color_index, block_index, orig_surface)
                    })
                    .zip(color_map.iter_mut())
            })
            .for_each(|((color_index, block_index, orig_surface), surface)| {
                free_if_unused(*orig_surface);
                *orig_surface = load_block(
                    null_mut(),
                    color_index.try_into().unwrap(),
                    block_index.try_into().unwrap(),
                    &ORIG_BLOCK_RECT,
                    0,
                );
                *surface = *orig_surface;
            });

        self.update_progress(20);
        //---------- get Droid-model  blocks
        let fpath = self.find_file(
            DROID_BLOCK_FILE_C.as_ptr() as *mut c_char,
            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
            Themed::UseTheme as c_int,
            Criticality::Critical as c_int,
        );
        load_block(fpath, 0, 0, null_mut(), INIT_ONLY as c_int);
        INFLUENCER_SURFACE_POINTER.iter_mut().enumerate().for_each(
            |(index, influencer_surface)| {
                free_if_unused(*influencer_surface);
                *influencer_surface = load_block(
                    null_mut(),
                    0,
                    index.try_into().unwrap(),
                    &ORIG_BLOCK_RECT,
                    0,
                );

                /* Droid pics are only used in _internal_ blits ==> clear per-surf alpha */
                SDL_SetAlpha(*influencer_surface, 0, 0);
            },
        );

        ENEMY_SURFACE_POINTER
            .iter_mut()
            .enumerate()
            .for_each(|(index, enemy_surface)| {
                free_if_unused(*enemy_surface);
                *enemy_surface = load_block(
                    null_mut(),
                    1,
                    index.try_into().unwrap(),
                    &ORIG_BLOCK_RECT,
                    0,
                );

                /* Droid pics are only used in _internal_ blits ==> clear per-surf alpha */
                SDL_SetAlpha(*enemy_surface, 0, 0);
            });

        self.update_progress(30);
        //---------- get Bullet blocks
        let fpath = self.find_file(
            BULLET_BLOCK_FILE_C.as_ptr() as *mut c_char,
            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
            Themed::UseTheme as c_int,
            Criticality::Critical as c_int,
        );
        load_block(fpath, 0, 0, null_mut(), INIT_ONLY as c_int);
        std::slice::from_raw_parts_mut(BULLETMAP, NUMBER_OF_BULLET_TYPES.try_into().unwrap())
            .iter_mut()
            .enumerate()
            .flat_map(|(bullet_type_index, bullet)| {
                bullet
                    .surface_pointer
                    .iter_mut()
                    .enumerate()
                    .map(move |(phase_index, surface)| (bullet_type_index, phase_index, surface))
            })
            .for_each(|(bullet_type_index, phase_index, surface)| {
                free_if_unused(*surface);
                *surface = load_block(
                    null_mut(),
                    bullet_type_index.try_into().unwrap(),
                    phase_index.try_into().unwrap(),
                    &ORIG_BLOCK_RECT,
                    0,
                );
            });

        self.update_progress(35);

        //---------- get Blast blocks
        let fpath = self.find_file(
            BLAST_BLOCK_FILE_C.as_ptr() as *mut c_char,
            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
            Themed::UseTheme as c_int,
            Criticality::Critical as c_int,
        );
        load_block(fpath, 0, 0, null_mut(), INIT_ONLY as c_int);
        BLASTMAP
            .iter_mut()
            .enumerate()
            .flat_map(|(blast_type_index, blast)| {
                blast
                    .surface_pointer
                    .iter_mut()
                    .enumerate()
                    .map(move |(surface_index, surface)| (blast_type_index, surface_index, surface))
            })
            .for_each(|(blast_type_index, surface_index, surface)| {
                free_if_unused(*surface);
                *surface = load_block(
                    null_mut(),
                    blast_type_index.try_into().unwrap(),
                    surface_index.try_into().unwrap(),
                    &ORIG_BLOCK_RECT,
                    0,
                );
            });

        self.update_progress(45);

        //---------- get Digit blocks
        let fpath = self.find_file(
            DIGIT_BLOCK_FILE_C.as_ptr() as *mut c_char,
            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
            Themed::UseTheme as c_int,
            Criticality::Critical as c_int,
        );
        load_block(fpath, 0, 0, null_mut(), INIT_ONLY as c_int);
        INFLU_DIGIT_SURFACE_POINTER
            .iter_mut()
            .enumerate()
            .for_each(|(index, surface)| {
                free_if_unused(*surface);
                *surface = load_block(
                    null_mut(),
                    0,
                    index.try_into().unwrap(),
                    &ORIG_DIGIT_RECT,
                    0,
                );
            });
        ENEMY_DIGIT_SURFACE_POINTER
            .iter_mut()
            .enumerate()
            .for_each(|(index, surface)| {
                free_if_unused(*surface);
                *surface = load_block(
                    null_mut(),
                    0,
                    (index + 10).try_into().unwrap(),
                    &ORIG_DIGIT_RECT,
                    0,
                );
            });

        self.update_progress(50);

        //---------- get Takeover pics
        free_if_unused(TO_BLOCKS); /* this happens when we do theme-switching */
        let fpath = self.find_file(
            TO_BLOCK_FILE_C.as_ptr() as *mut c_char,
            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
            Themed::UseTheme as c_int,
            Criticality::Critical as c_int,
        );
        TO_BLOCKS = load_block(fpath, 0, 0, null_mut(), 0);

        self.update_progress(60);

        free_if_unused(SHIP_ON_PIC);
        SHIP_ON_PIC = IMG_Load(self.find_file(
            SHIP_ON_PIC_FILE_C.as_ptr() as *mut c_char,
            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
            Themed::UseTheme as c_int,
            Criticality::Critical as c_int,
        ));
        free_if_unused(SHIP_OFF_PIC);
        SHIP_OFF_PIC = IMG_Load(self.find_file(
            SHIP_OFF_PIC_FILE_C.as_ptr() as *mut c_char,
            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
            Themed::UseTheme as c_int,
            Criticality::Critical as c_int,
        ));

        // the following are not theme-specific and are therefore only loaded once!
        DO_ONCE.call_once(|| {
            //  create the tmp block-build storage
            let tmp = SDL_CreateRGBSurface(
                0,
                self.vars.block_rect.w.into(),
                self.vars.block_rect.h.into(),
                VID_BPP,
                0,
                0,
                0,
                0,
            );
            BUILD_BLOCK = SDL_DisplayFormatAlpha(tmp);
            SDL_FreeSurface(tmp);

            // takeover background pics
            let fpath = self.find_file(
                TAKEOVER_BG_PIC_FILE_C.as_ptr() as *mut c_char,
                GRAPHICS_DIR_C.as_ptr() as *mut c_char,
                Themed::NoTheme as c_int,
                Criticality::Critical as c_int,
            );
            TAKEOVER_BG_PIC = load_block(fpath, 0, 0, null_mut(), 0);
            set_takeover_rects(); // setup takeover rectangles

            // cursor shapes
            ARROW_CURSOR = init_system_cursor(&ARROW_XPM);
            CROSSHAIR_CURSOR = init_system_cursor(&CROSSHAIR_XPM);
            //---------- get Console pictures
            let fpath = self.find_file(
                CONSOLE_PIC_FILE_C.as_ptr() as *mut c_char,
                GRAPHICS_DIR_C.as_ptr() as *mut c_char,
                Themed::NoTheme as c_int,
                Criticality::Critical as c_int,
            );
            CONSOLE_PIC = load_block(fpath, 0, 0, null_mut(), 0);
            let fpath = self.find_file(
                CONSOLE_BG_PIC1_FILE_C.as_ptr() as *mut c_char,
                GRAPHICS_DIR_C.as_ptr() as *mut c_char,
                Themed::NoTheme as c_int,
                Criticality::Critical as c_int,
            );
            CONSOLE_BG_PIC1 = load_block(fpath, 0, 0, null_mut(), 0);
            let fpath = self.find_file(
                CONSOLE_BG_PIC2_FILE_C.as_ptr() as *mut c_char,
                GRAPHICS_DIR_C.as_ptr() as *mut c_char,
                Themed::NoTheme as c_int,
                Criticality::Critical as c_int,
            );
            CONSOLE_BG_PIC2 = load_block(fpath, 0, 0, null_mut(), 0);

            self.update_progress(80);

            ARROW_UP = IMG_Load(self.find_file(
                cstr!("arrow_up.png").as_ptr() as *mut c_char,
                GRAPHICS_DIR_C.as_ptr() as *mut c_char,
                Themed::NoTheme as c_int,
                Criticality::Critical as c_int,
            ));
            ARROW_DOWN = IMG_Load(self.find_file(
                cstr!("arrow_down.png").as_ptr() as *mut c_char,
                GRAPHICS_DIR_C.as_ptr() as *mut c_char,
                Themed::NoTheme as c_int,
                Criticality::Critical as c_int,
            ));
            ARROW_RIGHT = IMG_Load(self.find_file(
                cstr!("arrow_right.png").as_ptr() as *mut c_char,
                GRAPHICS_DIR_C.as_ptr() as *mut c_char,
                Themed::NoTheme as c_int,
                Criticality::Critical as c_int,
            ));
            ARROW_LEFT = IMG_Load(self.find_file(
                cstr!("arrow_left.png").as_ptr() as *mut c_char,
                GRAPHICS_DIR_C.as_ptr() as *mut c_char,
                Themed::NoTheme as c_int,
                Criticality::Critical as c_int,
            ));
            //---------- get Banner
            let fpath = self.find_file(
                BANNER_BLOCK_FILE_C.as_ptr() as *mut c_char,
                GRAPHICS_DIR_C.as_ptr() as *mut c_char,
                Themed::NoTheme as c_int,
                Criticality::Critical as c_int,
            );
            BANNER_PIC = load_block(fpath, 0, 0, null_mut(), 0);

            self.update_progress(90);
            //---------- get Droid images ----------
            let droids = std::slice::from_raw_parts(DRUIDMAP, Droid::NumDroids as usize);
            droids
                .iter()
                .zip(PACKED_PORTRAITS.iter_mut())
                .zip(PORTRAIT_RAW_MEM.iter_mut())
                .for_each(|((droid, packed_portrait), raw_portrait)| {
                    // first check if we find a file with rotation-frames: first try .jpg
                    libc::strcpy(fname.as_mut_ptr(), droid.druidname.as_ptr());
                    libc::strcat(fname.as_mut_ptr(), cstr!(".jpg").as_ptr());
                    let mut fpath = self.find_file(
                        fname.as_mut_ptr(),
                        GRAPHICS_DIR_C.as_ptr() as *mut c_char,
                        Themed::NoTheme as c_int,
                        Criticality::Ignore as c_int,
                    );
                    // then try with .png
                    if fpath.is_null() {
                        libc::strcpy(fname.as_mut_ptr(), droid.druidname.as_ptr());
                        libc::strcat(fname.as_mut_ptr(), cstr!(".png").as_ptr());
                        fpath = self.find_file(
                            fname.as_mut_ptr(),
                            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
                            Themed::NoTheme as c_int,
                            Criticality::Critical as c_int,
                        );
                    }

                    *packed_portrait = self.load_raw_pic(fpath, raw_portrait);
                });

            self.update_progress(95);
            // we need the 999.png in any case for transparency!
            libc::strcpy(
                fname.as_mut_ptr(),
                droids[Droid::Droid999 as usize].druidname.as_ptr(),
            );
            libc::strcat(fname.as_mut_ptr(), cstr!(".png").as_ptr());
            let fpath = self.find_file(
                fname.as_mut_ptr(),
                GRAPHICS_DIR_C.as_ptr() as *mut c_char,
                Themed::NoTheme as c_int,
                Criticality::Critical as c_int,
            );
            PIC999 = load_block(fpath, 0, 0, null_mut(), 0);

            // get the Ashes pics
            libc::strcpy(fname.as_mut_ptr(), cstr!("Ashes.png").as_ptr());
            let fpath = self.find_file(
                fname.as_mut_ptr(),
                GRAPHICS_DIR_C.as_ptr() as *mut c_char,
                Themed::NoTheme as c_int,
                Criticality::WarnOnly as c_int,
            );
            if fpath.is_null() {
                warn!("deactivated display of droid-decals");
                self.global.game_config.show_decals = false.into();
            } else {
                load_block(fpath, 0, 0, null_mut(), INIT_ONLY as c_int);
                DECAL_PICS[0] = load_block(null_mut(), 0, 0, &ORIG_BLOCK_RECT, 0);
                DECAL_PICS[1] = load_block(null_mut(), 0, 1, &ORIG_BLOCK_RECT, 0);
            }
        });

        self.update_progress(96);
        // if scale != 1 then we need to rescale everything now
        self.scale_graphics(self.global.game_config.scale);

        self.update_progress(98);

        // make sure bullet-surfaces get re-generated!
        ALL_BULLETS
            .iter_mut()
            .take(MAXBULLETS)
            .for_each(|bullet| bullet.surfaces_were_generated = false.into());

        self.b_font.current_font = oldfont;

        true.into()
    }
}

const CROSSHAIR_XPM: [&[u8]; 37] = [
    /* width height num_colors chars_per_pixel */
    &*b"    32    32        3            1",
    /* colors */
    b"X c #000000",
    b". c #ffffff",
    b"  c None",
    /* pixels */
    b"                                ",
    b"                                ",
    b"               XXXX             ",
    b"               X..X             ",
    b"               X..X             ",
    b"               X..X             ",
    b"               X..X             ",
    b"               X..X             ",
    b"               X..X             ",
    b"               X..X             ",
    b"               X..X             ",
    b"               XXXX             ",
    b"                                ",
    b"   XXXXXXXXXXX      XXXXXXXXXX  ",
    b"   X.........X      X........X  ",
    b"   X.........X      X........X  ",
    b"   XXXXXXXXXXX      XXXXXXXXXX  ",
    b"                                ",
    b"               XXXX             ",
    b"               X..X             ",
    b"               X..X             ",
    b"               X..X             ",
    b"               X..X             ",
    b"               X..X             ",
    b"               X..X             ",
    b"               X..X             ",
    b"               X..X             ",
    b"               X..X             ",
    b"               X..X             ",
    b"               XXXX             ",
    b"                                ",
    b"                                ",
    b"0,0",
];

const ARROW_XPM: [&[u8]; 37] = [
    /* width height num_colors chars_per_pixel */
    &*b"    32    32        3            1",
    /* colors */
    b"X c #000000",
    b". c #ffffff",
    b"  c None",
    /* pixels */
    b"X                               ",
    b"XX                              ",
    b"X.X                             ",
    b"X..X                            ",
    b"X...X                           ",
    b"X....X                          ",
    b"X.....X                         ",
    b"X......X                        ",
    b"X.......X                       ",
    b"X........X                      ",
    b"X.....XXXXX                     ",
    b"X..X..X                         ",
    b"X.X X..X                        ",
    b"XX  X..X                        ",
    b"X    X..X                       ",
    b"     X..X                       ",
    b"      X..X                      ",
    b"      X..X                      ",
    b"       XX                       ",
    b"                                ",
    b"                                ",
    b"                                ",
    b"                                ",
    b"                                ",
    b"                                ",
    b"                                ",
    b"                                ",
    b"                                ",
    b"                                ",
    b"                                ",
    b"                                ",
    b"                                ",
    b"0,0",
];

/// This function was taken directly from the example in the SDL docu.
/// Even there they say they have stolen if from the mailing list.
/// Anyway it should create a new mouse cursor from an XPM.
/// The XPM is defined above and not read in from disk or something.
fn init_system_cursor(image: &[&[u8]]) -> *mut SDL_Cursor {
    let mut data = [0u8; 4 * 32];
    let mut mask = [0u8; 4 * 32];

    let mut i: isize = -1;
    for row in 0..32 {
        for col in 0..32 {
            if col % 8 != 0 {
                data[i as usize] <<= 1;
                mask[i as usize] <<= 1;
            } else {
                i += 1;
                data[i as usize] = 0;
                mask[i as usize] = 0;
            }

            match image[4 + row][col] {
                b'X' => {
                    data[i as usize] |= 0x01;
                    mask[i as usize] |= 0x01;
                }
                b'.' => {
                    mask[i as usize] |= 0x01;
                }
                b' ' => {}
                _ => panic!("invalid XPM charater"),
            }
        }
    }

    let last_line = std::str::from_utf8(&image[4 + 32]).unwrap();
    let mut hots = last_line.splitn(2, ',').map(|x| x.parse().unwrap());
    let hot_x = hots.next().unwrap();
    let hot_y = hots.next().unwrap();
    unsafe { SDL_CreateCursor(data.as_mut_ptr(), mask.as_mut_ptr(), 32, 32, hot_x, hot_y) }
}

impl Data {
    pub unsafe fn load_theme_configuration_file(&mut self) {
        use bstr::ByteSlice;

        const END_OF_THEME_DATA_STRING: &CStr = cstr!("**** End of theme data section ****");

        let fpath = self.find_file(
            cstr!("config.theme").as_ptr() as *mut c_char,
            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
            Themed::UseTheme as c_int,
            Criticality::Critical as c_int,
        );

        let data = self.read_and_malloc_and_terminate_file(
            fpath,
            END_OF_THEME_DATA_STRING.as_ptr() as *mut c_char,
        );

        //--------------------
        // Now the file is read in entirely and
        // we can start to analyze its content,
        //
        const BLAST_ONE_NUMBER_OF_PHASES_STRING: &CStr = cstr!("How many phases in Blast one :");
        const BLAST_TWO_NUMBER_OF_PHASES_STRING: &CStr = cstr!("How many phases in Blast two :");

        read_value_from_string(
            data.as_ptr() as *mut c_char,
            BLAST_ONE_NUMBER_OF_PHASES_STRING.as_ptr() as *mut c_char,
            cstr!("%d").as_ptr() as *mut c_char,
            &mut BLASTMAP[0].phases as *mut c_int as *mut c_void,
        );

        read_value_from_string(
            data.as_ptr() as *mut c_char,
            BLAST_TWO_NUMBER_OF_PHASES_STRING.as_ptr() as *mut c_char,
            cstr!("%d").as_ptr() as *mut c_char,
            &mut BLASTMAP[1].phases as *mut c_int as *mut c_void,
        );

        // Next we read in the number of phases that are to be used for each bullet type
        let mut reader = std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len());
        while let Some(read_start) = reader.find(b"For Bullettype Nr.=") {
            let read = &reader[read_start..];
            let mut bullet_index: c_int = 0;
            read_value_from_string(
                read.as_ptr() as *mut c_char,
                cstr!("For Bullettype Nr.=").as_ptr() as *mut c_char,
                cstr!("%d").as_ptr() as *mut c_char,
                &mut bullet_index as *mut c_int as *mut c_void,
            );
            if bullet_index >= NUMBER_OF_BULLET_TYPES {
                panic!(
                    "----------------------------------------------------------------------\n\
                 Freedroid has encountered a problem:\n\
                 In function 'char* LoadThemeConfigurationFile ( ... ):\n\
                 \n\
                 There was a specification for the number of phases in a bullet type\n\
                 that does not at all exist in the ruleset.\n\
                 \n\
                 This might indicate that either the ruleset file is corrupt or the \n\
                 theme.config configuration file is corrupt or (less likely) that there\n\
                 is a severe bug in the reading function.\n\
                 \n\
                 Please check that your theme and ruleset files are properly set up.\n\
                 \n\
                 Please also don't forget, that you might have to run 'make install'\n\
                 again after you've made modifications to the data files in the source tree.\n\
                 \n\
                 Freedroid will terminate now to draw attention to the data problem it could\n\
                 not resolve.... Sorry, if that interrupts a major game of yours.....\n\
                 ----------------------------------------------------------------------\n"
                );
            }
            read_value_from_string(
                read.as_ptr() as *mut c_char,
                cstr!("we will use number of phases=").as_ptr() as *mut c_char,
                cstr!("%d").as_ptr() as *mut c_char,
                &mut (*BULLETMAP.offset(bullet_index.try_into().unwrap())).phases as *mut c_int
                    as *mut c_void,
            );
            read_value_from_string(
                read.as_ptr() as *mut c_char,
                cstr!("and number of phase changes per second=").as_ptr() as *mut c_char,
                cstr!("%f").as_ptr() as *mut c_char,
                &mut (*BULLETMAP.offset(bullet_index.try_into().unwrap())).phase_changes_per_second
                    as *mut c_float as *mut c_void,
            );
            reader = &reader[read_start + 1..];
        }

        // --------------------
        // Also decidable from the theme is where in the robot to
        // display the digits.  This must also be read from the configuration
        // file of the theme
        //
        const DIGIT_ONE_POSITION_X_STRING: &CStr = cstr!("First digit x :");
        const DIGIT_ONE_POSITION_Y_STRING: &CStr = cstr!("First digit y :");
        const DIGIT_TWO_POSITION_X_STRING: &CStr = cstr!("Second digit x :");
        const DIGIT_TWO_POSITION_Y_STRING: &CStr = cstr!("Second digit y :");
        const DIGIT_THREE_POSITION_X_STRING: &CStr = cstr!("Third digit x :");
        const DIGIT_THREE_POSITION_Y_STRING: &CStr = cstr!("Third digit y :");

        read_value_from_string(
            data.as_ptr() as *mut c_char,
            DIGIT_ONE_POSITION_X_STRING.as_ptr() as *mut c_char,
            cstr!("%hd").as_ptr() as *mut c_char,
            &mut FIRST_DIGIT_RECT.x as *mut c_short as *mut c_void,
        );
        read_value_from_string(
            data.as_ptr() as *mut c_char,
            DIGIT_ONE_POSITION_Y_STRING.as_ptr() as *mut c_char,
            cstr!("%hd").as_ptr() as *mut c_char,
            &mut FIRST_DIGIT_RECT.y as *mut c_short as *mut c_void,
        );

        read_value_from_string(
            data.as_ptr() as *mut c_char,
            DIGIT_TWO_POSITION_X_STRING.as_ptr() as *mut c_char,
            cstr!("%hd").as_ptr() as *mut c_char,
            &mut SECOND_DIGIT_RECT.x as *mut c_short as *mut c_void,
        );
        read_value_from_string(
            data.as_ptr() as *mut c_char,
            DIGIT_TWO_POSITION_Y_STRING.as_ptr() as *mut c_char,
            cstr!("%hd").as_ptr() as *mut c_char,
            &mut SECOND_DIGIT_RECT.y as *mut c_short as *mut c_void,
        );

        read_value_from_string(
            data.as_ptr() as *mut c_char,
            DIGIT_THREE_POSITION_X_STRING.as_ptr() as *mut c_char,
            cstr!("%hd").as_ptr() as *mut c_char,
            &mut THIRD_DIGIT_RECT.x as *mut i16 as *mut c_void,
        );
        read_value_from_string(
            data.as_ptr() as *mut c_char,
            DIGIT_THREE_POSITION_Y_STRING.as_ptr() as *mut c_char,
            cstr!("%hd").as_ptr() as *mut c_char,
            &mut THIRD_DIGIT_RECT.y as *mut c_short as *mut c_void,
        );
    }

    /// This function resizes all blocks and structures involved in assembling
    /// the combat picture to a new scale.  The new scale is relative to the
    /// standard scale with means scale=1 is 64x64 tile size.
    ///
    /// in the first call we assume the Block_Rect to be the original game-size
    /// and store this value for future rescalings
    pub unsafe fn set_combat_scale_to(&mut self, scale: c_float) {
        use once_cell::sync::OnceCell;

        MAP_BLOCK_SURFACE_POINTER
            .iter_mut()
            .zip(ORIG_MAP_BLOCK_SURFACE_POINTER.iter())
            .flat_map(|(map_block, orig_map_block)| map_block.iter_mut().zip(orig_map_block.iter()))
            .for_each(|(surface, &orig_surface)| {
                // if there's already a rescaled version, free it
                if *surface != orig_surface {
                    SDL_FreeSurface(*surface);
                }
                // then zoom..
                let tmp = zoomSurface(orig_surface, scale.into(), scale.into(), 0);
                if tmp.is_null() {
                    panic!("zoomSurface() failed for scale = {}.", scale);
                }
                // and optimize
                *surface = SDL_DisplayFormat(tmp);
                SDL_FreeSurface(tmp); // free the old surface
            });

        static ORIG_BLOCK: OnceCell<Rect> = OnceCell::new();
        let orig_block = ORIG_BLOCK.get_or_init(|| self.vars.block_rect);

        self.vars.block_rect = *orig_block;
        scale_rect(&mut self.vars.block_rect, scale);
    }

    /// This function load an image and displays it directly to the NE_SCREEN
    /// but without updating it.
    /// This might be very handy, especially in the Title() function to
    /// display the title image and perhaps also for displaying the ship
    /// and that.
    pub unsafe fn display_image(&mut self, datafile: *mut c_char) {
        let mut image = IMG_Load(datafile);
        if image.is_null() {
            panic!(
                "couldn't load image {}: {}",
                CStr::from_ptr(datafile).to_string_lossy(),
                get_error()
            );
        }

        if (self.global.game_config.scale - 1.).abs() > c_float::EPSILON {
            scale_pic(&mut image, self.global.game_config.scale);
        }

        SDL_UpperBlit(image, null_mut(), NE_SCREEN, null_mut());

        SDL_FreeSurface(image);
    }

    pub unsafe fn draw_line_between_tiles(
        &self,
        mut x1: c_float,
        mut y1: c_float,
        mut x2: c_float,
        mut y2: c_float,
        color: c_int,
    ) {
        if (x1 - x2).abs() <= f32::EPSILON && (y1 - y2).abs() <= f32::EPSILON {
            return;
        }

        if (x1 - x2).abs() <= f32::EPSILON
        // infinite slope!! special case, that must be caught!
        {
            if y1 > y2
            // in this case, just interchange 1 and 2
            {
                std::mem::swap(&mut y1, &mut y2);
            }

            let mut i = 0.;
            let max = (y2 - y1) * f32::from(self.vars.block_rect.w);
            while i < max {
                let pixx = f32::from(self.vars.user_rect.x) + f32::from(self.vars.user_rect.w / 2)
                    - f32::from(self.vars.block_rect.w) * (ME.pos.x - x1);
                let user_center = self.get_user_center();
                let pixy = f32::from(user_center.y)
                    - f32::from(self.vars.block_rect.h) * (ME.pos.y - y1)
                    + i;
                if pixx <= self.vars.user_rect.x.into()
                    || pixx
                        >= f32::from(self.vars.user_rect.x) + f32::from(self.vars.user_rect.w) - 1.
                    || pixy <= f32::from(self.vars.user_rect.y)
                    || pixy
                        >= f32::from(self.vars.user_rect.y) + f32::from(self.vars.user_rect.h) - 1.
                {
                    i += 1.;
                    continue;
                }
                putpixel(
                    NE_SCREEN,
                    pixx as c_int,
                    pixy as c_int,
                    color.try_into().unwrap(),
                );
                putpixel(
                    NE_SCREEN,
                    pixx as c_int - 1,
                    pixy as c_int,
                    color.try_into().unwrap(),
                );

                i += 1.;
            }
            return;
        }

        if x1 > x2
        // in this case, just interchange 1 and 2
        {
            std::mem::swap(&mut x1, &mut x2);
            std::mem::swap(&mut y1, &mut y2);
        }

        //--------------------
        // Now we start the drawing process
        //

        let slope = (y2 - y1) / (x2 - x1);
        let mut i = 0.;
        let max = (x2 - x1) * f32::from(self.vars.block_rect.w);
        while i < max {
            let pixx = f32::from(self.vars.user_rect.x) + f32::from(self.vars.user_rect.w / 2)
                - f32::from(self.vars.block_rect.w) * (ME.pos.x - x1)
                + i;
            let user_center = self.get_user_center();
            let pixy = f32::from(user_center.y)
                - f32::from(self.vars.block_rect.h) * (ME.pos.y - y1)
                + i * slope;
            if pixx <= f32::from(self.vars.user_rect.x)
                || pixx >= f32::from(self.vars.user_rect.x) + f32::from(self.vars.user_rect.w) - 1.
                || pixy <= f32::from(self.vars.user_rect.y)
                || pixy >= f32::from(self.vars.user_rect.y) + f32::from(self.vars.user_rect.h) - 1.
            {
                i += 1.;
                continue;
            }
            putpixel(
                NE_SCREEN,
                pixx as c_int,
                pixy as c_int,
                color.try_into().unwrap(),
            );
            putpixel(
                NE_SCREEN,
                pixx as c_int,
                pixy as c_int - 1,
                color.try_into().unwrap(),
            );
            i += 1.;
        }
    }
}
