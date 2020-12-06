use crate::{
    b_font::{BFontInfo, GetCurrentFont, LoadFont, PutPixel, SetCurrentFont},
    defs::{
        self, free_if_unused, scale_point, scale_rect, Cmds, Criticality, DisplayBannerFlags,
        Droid, Sound, Themed, BANNER_BLOCK_FILE_C, BLAST_BLOCK_FILE_C, BULLET_BLOCK_FILE_C,
        CONSOLE_BG_PIC1_FILE_C, CONSOLE_BG_PIC2_FILE_C, CONSOLE_PIC_FILE_C, DIGIT_BLOCK_FILE_C,
        DROID_BLOCK_FILE_C, FONT0_FILE, FONT0_FILE_C, FONT1_FILE, FONT1_FILE_C, FONT2_FILE,
        FONT2_FILE_C, FREE_ONLY, GRAPHICS_DIR_C, ICON_FILE, ICON_FILE_C, INIT_ONLY,
        MAP_BLOCK_FILE_C, MAXBULLETS, PARA_FONT_FILE, PARA_FONT_FILE_C, SHIP_OFF_PIC_FILE_C,
        SHIP_ON_PIC_FILE_C, TAKEOVER_BG_PIC_FILE_C,
    },
    global::{
        arrow_cursor, arrow_down, arrow_left, arrow_right, arrow_up, banner_pic, console_bg_pic1,
        console_bg_pic2, console_pic, crosshair_cursor, ne_screen, packed_portraits, pic999,
        progress_filler_pic, progress_meter_pic, ship_off_pic, ship_on_pic, takeover_bg_pic,
        to_blocks, AllBullets, BannerIsDestroyed, Banner_Rect, Blastmap, Block_Rect, BuildBlock,
        Bulletmap, CapsuleBlocks, Classic_User_Rect, ConsMenuItem_Rect, Cons_Droid_Rect,
        Cons_Header_Rect, Cons_Menu_Rect, Cons_Menu_Rects, Cons_Text_Rect, CurCapsuleStart,
        Decal_pics, Digit_Rect, DruidStart, Druidmap, EnemyDigitSurfacePointer,
        EnemySurfacePointer, FillBlocks, FirstDigit_Rect, Font0_BFont, Font1_BFont, Font2_BFont,
        Full_User_Rect, GameConfig, Highscore_BFont, InfluDigitSurfacePointer,
        InfluencerSurfacePointer, LeftCapsulesStart, LeftInfo_Rect, MapBlockSurfacePointer,
        Menu_BFont, Menu_Rect, Number_Of_Bullet_Types, OptionsMenu_Rect, OrigBlock_Rect,
        OrigDigit_Rect, OrigMapBlockSurfacePointer, Para_BFont, PlaygroundStart, Portrait_Rect,
        RightInfo_Rect, Screen_Rect, SecondDigit_Rect, TO_CapsuleRect, TO_ColumnRect,
        TO_ColumnStart, TO_ElementRect, TO_FillBlock, TO_GroundRect, TO_LeaderBlockStart,
        TO_LeaderLed, TO_LeftGroundStart, TO_RightGroundStart, ThirdDigit_Rect, ToColumnBlock,
        ToGameBlocks, ToGroundBlocks, ToLeaderBlock, User_Rect,
    },
    input::{any_key_just_pressed, cmd_is_active, wait_for_all_keys_released, SDL_Delay},
    misc::{
        find_file, init_progress, update_progress, Activate_Conservative_Frame_Computation,
        MyMalloc, Terminate,
    },
    sound::Play_Sound,
    takeover::{set_takeover_rects, TO_BLOCK_FILE_C},
    text::printf_SDL,
    view::DisplayBanner,
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
    convert::{TryFrom, TryInto},
    ffi::CStr,
    os::raw::{c_char, c_double, c_float, c_int, c_void},
    ptr::null_mut,
};

extern "C" {
    pub static mut vid_info: *const SDL_VideoInfo;
    pub static mut vid_bpp: c_int;
    pub static mut portrait_raw_mem: [*mut c_char; Droid::NumDroids as usize];
    pub fn IMG_Load(file: *const c_char) -> *mut SDL_Surface;
    pub fn zoomSurface(
        src: *mut SDL_Surface,
        zoomx: c_double,
        zoomy: c_double,
        smooth: c_int,
    ) -> *mut SDL_Surface;
    pub fn SDL_GetClipRect(surface: *mut SDL_Surface, rect: *mut SDL_Rect);
    pub fn SDL_VideoDriverName(namebuf: *mut c_char, maxlen: c_int) -> *mut c_char;
    pub fn SDL_RWFromMem(mem: *mut c_void, size: c_int) -> *mut SDL_RWops;
    pub static mut fonts_loaded: c_int;
    pub fn LoadThemeConfigurationFile();
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

#[no_mangle]
pub unsafe extern "C" fn ScalePic(pic: &mut *mut SDL_Surface, scale: c_float) {
    if (scale - 1.0).abs() <= f32::EPSILON {
        return;
    }
    let scale = scale.into();

    let tmp = *pic;
    *pic = zoomSurface(tmp, scale, scale, 0);
    if pic.is_null() {
        error!("zoomSurface() failed for scale = {}.", scale);
        Terminate(defs::ERR.into());
    }
    SDL_FreeSurface(tmp);
}

#[no_mangle]
pub unsafe extern "C" fn ScaleGraphics(scale: c_float) {
    static INIT: std::sync::Once = std::sync::Once::new();

    /* For some reason we need to SetAlpha every time on OS X */
    /* Digits are only used in _internal_ blits ==> clear per-surf alpha */
    for &surface in &InfluDigitSurfacePointer {
        SDL_SetAlpha(surface, 0, 0);
    }
    for &surface in &EnemyDigitSurfacePointer {
        SDL_SetAlpha(surface, 0, 0);
    }
    if (scale - 1.).abs() <= f32::EPSILON {
        return;
    }

    // these are reset in a theme-change by the theme-config-file
    // therefore we need to rescale them each time again
    scale_rect(&mut FirstDigit_Rect, scale);
    scale_rect(&mut SecondDigit_Rect, scale);
    scale_rect(&mut ThirdDigit_Rect, scale);

    // note: only rescale these rects the first time!!
    let mut init = false;
    INIT.call_once(|| {
        init = true;
        ScaleStatRects(scale);
    });

    //---------- rescale Map blocks
    OrigMapBlockSurfacePointer
        .iter_mut()
        .flat_map(|surfaces| surfaces.iter_mut())
        .zip(
            MapBlockSurfacePointer
                .iter_mut()
                .flat_map(|surfaces| surfaces.iter_mut()),
        )
        .for_each(|(orig_surface, map_surface)| {
            ScalePic(orig_surface, scale);
            *map_surface = *orig_surface;
        });

    //---------- rescale Droid-model  blocks
    /* Droid pics are only used in _internal_ blits ==> clear per-surf alpha */
    for surface in &mut InfluencerSurfacePointer {
        ScalePic(surface, scale);
        SDL_SetAlpha(*surface, 0, 0);
    }
    for surface in &mut EnemySurfacePointer {
        ScalePic(surface, scale);
        SDL_SetAlpha(*surface, 0, 0);
    }

    //---------- rescale Bullet blocks
    let bulletmap =
        std::slice::from_raw_parts_mut(Bulletmap, usize::try_from(Number_Of_Bullet_Types).unwrap());
    bulletmap
        .iter_mut()
        .flat_map(|bullet| bullet.SurfacePointer.iter_mut())
        .for_each(|surface| ScalePic(surface, scale));

    //---------- rescale Blast blocks
    Blastmap
        .iter_mut()
        .flat_map(|blast| blast.SurfacePointer.iter_mut())
        .for_each(|surface| ScalePic(surface, scale));

    //---------- rescale Digit blocks
    for surface in &mut InfluDigitSurfacePointer {
        ScalePic(surface, scale);
        SDL_SetAlpha(*surface, 0, 0);
    }
    for surface in &mut EnemyDigitSurfacePointer {
        ScalePic(surface, scale);
        SDL_SetAlpha(*surface, 0, 0);
    }

    //---------- rescale Takeover pics
    ScalePic(&mut to_blocks, scale);
    //  printf_SDL (ne_screen, -1, -1, ".");

    ScalePic(&mut ship_on_pic, scale);
    ScalePic(&mut ship_off_pic, scale);

    // the following are not theme-specific and are therefore only loaded once!
    if init {
        //  create a new tmp block-build storage
        free_if_unused(BuildBlock);
        let tmp = SDL_CreateRGBSurface(
            0,
            Block_Rect.w.into(),
            Block_Rect.h.into(),
            vid_bpp,
            0,
            0,
            0,
            0,
        );
        BuildBlock = SDL_DisplayFormatAlpha(tmp);
        SDL_FreeSurface(tmp);

        // takeover pics
        ScalePic(&mut takeover_bg_pic, scale);

        //---------- Console pictures
        ScalePic(&mut console_pic, scale);
        ScalePic(&mut console_bg_pic1, scale);
        ScalePic(&mut console_bg_pic2, scale);
        ScalePic(&mut arrow_up, scale);
        ScalePic(&mut arrow_down, scale);
        ScalePic(&mut arrow_right, scale);
        ScalePic(&mut arrow_left, scale);
        //---------- Banner
        ScalePic(&mut banner_pic, scale);

        ScalePic(&mut pic999, scale);

        // get the Ashes pics
        if !Decal_pics[0].is_null() {
            ScalePic(&mut Decal_pics[0], scale);
        }
        if !Decal_pics[1].is_null() {
            ScalePic(&mut Decal_pics[1], scale);
        }
    }

    printf_SDL(ne_screen, -1, -1, cstr!(" ok\n").as_ptr() as *mut c_char);
}

/// display "white noise" effect in Rect.
/// algorith basically stolen from
/// Greg Knauss's "xteevee" hack in xscreensavers.
///
/// timeout is in ms
#[no_mangle]
pub unsafe extern "C" fn white_noise(bitmap: *mut SDL_Surface, rect: &mut Rect, timeout: c_int) {
    use rand::{
        seq::{IteratorRandom, SliceRandom},
        Rng,
    };
    const NOISE_COLORS: usize = 6;
    const NOISE_TILES: usize = 8;

    let signal_strengh = 60;

    let grey: [u32; NOISE_COLORS] = array_init(|index| {
        let color = (((index as f64 + 1.0) / (NOISE_COLORS as f64)) * 255.0) as u8;
        SDL_MapRGB((*ne_screen).format, color, color, color)
    });

    // produce the tiles
    let tmp = SDL_CreateRGBSurface(0, rect.w.into(), rect.h.into(), vid_bpp, 0, 0, 0, 0);
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
                    PutPixel(&*tile, x.into(), y.into(), *grey.choose(&mut rng).unwrap());
                }
            });
        tile
    });
    SDL_FreeSurface(tmp2);

    let mut used_tiles: [c_char; NOISE_TILES / 2 + 1] = [-1; NOISE_TILES / 2 + 1];
    // let's go
    Play_Sound(Sound::WhiteNoise as c_int);

    let now = SDL_GetTicks();

    wait_for_all_keys_released();
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
        SDL_GetClipRect(ne_screen, &mut clip_rect);
        SDL_SetClipRect(ne_screen, null_mut());
        // set it
        SDL_UpperBlit(
            noise_tiles[usize::try_from(next_tile).unwrap()],
            null_mut(),
            ne_screen,
            rect,
        );
        SDL_UpdateRect(
            ne_screen,
            rect.x.into(),
            rect.y.into(),
            rect.w.into(),
            rect.h.into(),
        );
        SDL_Delay(25);

        if timeout != 0 && SDL_GetTicks() - now > timeout.try_into().unwrap() {
            break;
        }

        if any_key_just_pressed() != 0 {
            break;
        }
    }

    //restore previous clip-rectange
    SDL_SetClipRect(ne_screen, &clip_rect);

    for &tile in &noise_tiles {
        SDL_FreeSurface(tile);
    }
}

#[no_mangle]
pub unsafe extern "C" fn Duplicate_Font(in_font: &BFontInfo) -> *mut BFontInfo {
    let out_font = MyMalloc(std::mem::size_of::<BFontInfo>().try_into().unwrap()) as *mut BFontInfo;

    std::ptr::copy_nonoverlapping(in_font, out_font, 1);
    (*out_font).surface = SDL_ConvertSurface(
        in_font.surface,
        (*in_font.surface).format,
        (*in_font.surface).flags,
    );
    if (*out_font).surface.is_null() {
        error!("Duplicate_Font: failed to copy SDL_Surface using SDL_ConvertSurface()");
        Terminate(defs::ERR.into());
    }

    out_font
}

#[no_mangle]
pub unsafe extern "C" fn Load_Fonts() -> c_int {
    let mut fpath = find_file(
        PARA_FONT_FILE_C.as_ptr(),
        GRAPHICS_DIR_C.as_ptr() as *mut c_char,
        Themed::NoTheme as c_int,
        Criticality::Critical as c_int,
    );
    Para_BFont = LoadFont(fpath, GameConfig.scale);
    if Para_BFont.is_null() {
        error!("font file named {} was not found.", PARA_FONT_FILE);
        Terminate(defs::ERR.into());
    }

    fpath = find_file(
        FONT0_FILE_C.as_ptr(),
        GRAPHICS_DIR_C.as_ptr() as *mut c_char,
        Themed::NoTheme as c_int,
        Criticality::Critical as c_int,
    );
    Font0_BFont = LoadFont(fpath, GameConfig.scale);
    if Font0_BFont.is_null() {
        error!("font file named {} was not found.\n", FONT0_FILE);
        Terminate(defs::ERR.into());
    }

    fpath = find_file(
        FONT1_FILE_C.as_ptr(),
        GRAPHICS_DIR_C.as_ptr() as *mut c_char,
        Themed::NoTheme as c_int,
        Criticality::Critical as c_int,
    );
    Font1_BFont = LoadFont(fpath, GameConfig.scale);
    if Font1_BFont.is_null() {
        error!("font file named {} was not found.", FONT1_FILE);
        Terminate(defs::ERR.into());
    }

    fpath = find_file(
        FONT2_FILE_C.as_ptr(),
        GRAPHICS_DIR_C.as_ptr() as *mut c_char,
        Themed::NoTheme as c_int,
        Criticality::Critical as c_int,
    );
    Font2_BFont = LoadFont(fpath, GameConfig.scale);
    if Font2_BFont.is_null() {
        error!("font file named {} was not found.", FONT2_FILE);
        Terminate(defs::ERR.into());
    }

    Menu_BFont = Duplicate_Font(&*Para_BFont);
    Highscore_BFont = Duplicate_Font(&*Para_BFont);

    fonts_loaded = true.into();

    defs::OK.into()
}

/// Return the pixel value at (x, y)
/// NOTE: The surface must be locked before calling this!
#[no_mangle]
pub unsafe extern "C" fn getpixel(surface: &SDL_Surface, x: c_int, y: c_int) -> u32 {
    let bpp = (*surface.format).BytesPerPixel;
    /* Here p is the address to the pixel we want to retrieve */
    let p = surface.pixels.offset(
        isize::try_from(y).unwrap() * isize::try_from(surface.pitch).unwrap()
            + isize::try_from(x).unwrap() * isize::try_from(bpp).unwrap(),
    );

    match bpp {
        1 => (*(p as *const u8)).into(),
        2 => (*(p as *const u16)).into(),
        3 => {
            let p = std::slice::from_raw_parts(p as *const u8, 3);
            if cfg!(target_endian = "big") {
                u32::from(p[0]) << 16 | u32::from(p[1]) << 8 | u32::from(p[2])
            } else {
                u32::from(p[0]) | u32::from(p[1]) << 8 | u32::from(p[2]) << 16
            }
        }
        4 => *(p as *const u32),
        _ => {
            unreachable!()
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn ClearGraphMem() {
    // One this function is done, the rahmen at the
    // top of the screen surely is destroyed.  We inform the
    // DisplayBanner function of the matter...
    BannerIsDestroyed = true.into();

    SDL_SetClipRect(ne_screen, null_mut());

    // Now we fill the screen with black color...
    SDL_FillRect(ne_screen, null_mut(), 0);
    SDL_Flip(ne_screen);
}

/// Initialise the Video display and graphics engine
#[no_mangle]
pub unsafe extern "C" fn Init_Video() {
    const YN: [&str; 2] = ["no", "yes"];

    /* Initialize the SDL library */
    // if ( SDL_Init (SDL_INIT_VIDEO | SDL_INIT_TIMER) == -1 )

    if SDL_Init(SDL_INIT_VIDEO) == -1 {
        eprintln!("Couldn't initialize SDL: {}", get_error());
        Terminate(defs::ERR.into());
    } else {
        info!("SDL Video initialisation successful.");
    }

    // Now SDL_TIMER is initialized here:

    if SDL_InitSubSystem(SDL_INIT_TIMER) == -1 {
        eprintln!("Couldn't initialize SDL: {}", get_error());
        Terminate(defs::ERR.into());
    } else {
        info!("SDL Timer initialisation successful.");
    }

    /* clean up on exit */
    libc::atexit(std::mem::transmute(SDL_Quit as unsafe extern "C" fn()));

    vid_info = SDL_GetVideoInfo(); /* just curious */
    let mut vid_driver: [c_char; 81] = [0; 81];
    SDL_VideoDriverName(vid_driver.as_mut_ptr(), 80);

    let vid_info_ref = *vid_info;
    if cfg!(os_target = "android") {
        vid_bpp = 16; // Hardcoded Android default
    } else {
        vid_bpp = (*vid_info_ref.vfmt).BitsPerPixel.into();
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
        vid_bpp,
        (*vid_info_ref.vfmt).BytesPerPixel
    );
    info!(
        "Video Driver Name: {}",
        CStr::from_ptr(vid_driver.as_ptr()).to_string_lossy()
    );
    info!("----------------------------------------------------------------------");

    let vid_flags = if GameConfig.UseFullscreen != 0 {
        VideoFlag::Fullscreen as u32
    } else {
        0
    };

    if flag!(WMAvailable) {
        /* if there's a window-manager */
        SDL_WM_SetCaption(cstr!("Freedroid").as_ptr(), cstr!("").as_ptr());
        let fpath = find_file(
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

    ne_screen = SDL_SetVideoMode(Screen_Rect.w.into(), Screen_Rect.h.into(), 0, vid_flags);
    if ne_screen.is_null() {
        error!(
            "Couldn't set {} x {} video mode. SDL: {}",
            Screen_Rect.w,
            Screen_Rect.h,
            get_error(),
        );
        std::process::exit(-1);
    }

    vid_info = SDL_GetVideoInfo(); /* info about current video mode */

    info!("Got video mode: ");

    SDL_SetGamma(1., 1., 1.);
    GameConfig.Current_Gamma_Correction = 1.;
}

/// load a pic into memory and return the SDL_RWops pointer to it
#[no_mangle]
pub unsafe extern "C" fn load_raw_pic(
    fpath: *const c_char,
    raw_mem: *mut *mut c_char,
) -> *mut SDL_RWops {
    use std::{fs::File, io::Read, path::Path};

    if raw_mem.is_null() || !(*raw_mem).is_null() {
        error!("Invalid input 'raw_mem': must be pointing to NULL pointer");
        Terminate(defs::ERR.into());
    }

    // sanity check
    if fpath.is_null() {
        error!("load_raw_pic() called with NULL argument!");
        Terminate(defs::ERR.into());
    }

    let fpath = match CStr::from_ptr(fpath).to_str() {
        Ok(fpath) => fpath,
        Err(err) => {
            error!("unable to convert path with invalid UTF-8 data: {}", err);
            Terminate(defs::ERR.into());
        }
    };
    let fpath = Path::new(&fpath);
    let mut file = match File::open(fpath) {
        Ok(file) => file,
        Err(_) => {
            error!("could not open file {}. Giving up", fpath.display());
            Terminate(defs::ERR.into());
        }
    };

    let metadata = match file.metadata() {
        Ok(metadata) => metadata,
        Err(err) => {
            error!("unable to get file metadata: {}", err);
            Terminate(defs::ERR.into());
        }
    };

    let len = metadata.len().try_into().unwrap();
    *raw_mem = MyMalloc(len) as *mut i8;
    let buf = std::slice::from_raw_parts_mut(*raw_mem as *mut u8, len.try_into().unwrap());
    if file.read_exact(buf).is_err() {
        error!("cannot reading file {}. Giving up...", fpath.display());
        Terminate(defs::ERR.into());
    }
    drop(file);

    SDL_RWFromMem((*raw_mem) as *mut c_void, len.try_into().unwrap())
}

/// Get the pics for: druids, bullets, blasts
///
/// reads all blocks and puts the right pointers into
/// the various structs
///
/// Returns true/false
#[no_mangle]
pub unsafe extern "C" fn InitPictures() -> c_int {
    use std::sync::Once;

    static DO_ONCE: Once = Once::new();
    let mut fname: [c_char; 500] = [0; 500];

    // Loading all these pictures might take a while...
    // and we do not want do deal with huge frametimes, which
    // could box the influencer out of the ship....
    Activate_Conservative_Frame_Computation();

    let oldfont = GetCurrentFont();

    if fonts_loaded == 0 {
        Load_Fonts();
    }

    SetCurrentFont(Font0_BFont);

    init_progress(cstr!("Loading pictures").as_ptr() as *mut c_char);

    LoadThemeConfigurationFile();

    update_progress(15);

    //---------- get Map blocks
    let fpath = find_file(
        MAP_BLOCK_FILE_C.as_ptr(),
        GRAPHICS_DIR_C.as_ptr() as *mut c_char,
        Themed::UseTheme as c_int,
        Criticality::Critical as c_int,
    );
    Load_Block(fpath, 0, 0, null_mut(), INIT_ONLY as i32); /* init function */
    OrigMapBlockSurfacePointer
        .iter_mut()
        .enumerate()
        .zip(MapBlockSurfacePointer.iter_mut())
        .flat_map(|((color_index, orig_color_map), color_map)| {
            orig_color_map
                .iter_mut()
                .enumerate()
                .map(move |(block_index, orig_surface)| (color_index, block_index, orig_surface))
                .zip(color_map.iter_mut())
        })
        .for_each(|((color_index, block_index, orig_surface), surface)| {
            free_if_unused(*orig_surface);
            *orig_surface = Load_Block(
                null_mut(),
                color_index.try_into().unwrap(),
                block_index.try_into().unwrap(),
                &mut OrigBlock_Rect,
                0,
            );
            *surface = *orig_surface;
        });

    update_progress(20);
    //---------- get Droid-model  blocks
    let fpath = find_file(
        DROID_BLOCK_FILE_C.as_ptr() as *mut c_char,
        GRAPHICS_DIR_C.as_ptr() as *mut c_char,
        Themed::UseTheme as c_int,
        Criticality::Critical as c_int,
    );
    Load_Block(fpath, 0, 0, null_mut(), INIT_ONLY as c_int);
    InfluencerSurfacePointer
        .iter_mut()
        .enumerate()
        .for_each(|(index, influencer_surface)| {
            free_if_unused(*influencer_surface);
            *influencer_surface = Load_Block(
                null_mut(),
                0,
                index.try_into().unwrap(),
                &mut OrigBlock_Rect,
                0,
            );

            /* Droid pics are only used in _internal_ blits ==> clear per-surf alpha */
            SDL_SetAlpha(*influencer_surface, 0, 0);
        });

    EnemySurfacePointer
        .iter_mut()
        .enumerate()
        .for_each(|(index, enemy_surface)| {
            free_if_unused(*enemy_surface);
            *enemy_surface = Load_Block(
                null_mut(),
                1,
                index.try_into().unwrap(),
                &mut OrigBlock_Rect,
                0,
            );

            /* Droid pics are only used in _internal_ blits ==> clear per-surf alpha */
            SDL_SetAlpha(*enemy_surface, 0, 0);
        });

    update_progress(30);
    //---------- get Bullet blocks
    let fpath = find_file(
        BULLET_BLOCK_FILE_C.as_ptr() as *mut c_char,
        GRAPHICS_DIR_C.as_ptr() as *mut c_char,
        Themed::UseTheme as c_int,
        Criticality::Critical as c_int,
    );
    Load_Block(fpath, 0, 0, null_mut(), INIT_ONLY as c_int);
    std::slice::from_raw_parts_mut(Bulletmap, Number_Of_Bullet_Types.try_into().unwrap())
        .iter_mut()
        .enumerate()
        .flat_map(|(bullet_type_index, bullet)| {
            bullet
                .SurfacePointer
                .iter_mut()
                .enumerate()
                .map(move |(phase_index, surface)| (bullet_type_index, phase_index, surface))
        })
        .for_each(|(bullet_type_index, phase_index, surface)| {
            free_if_unused(*surface);
            *surface = Load_Block(
                null_mut(),
                bullet_type_index.try_into().unwrap(),
                phase_index.try_into().unwrap(),
                &mut OrigBlock_Rect,
                0,
            );
        });

    update_progress(35);

    //---------- get Blast blocks
    let fpath = find_file(
        BLAST_BLOCK_FILE_C.as_ptr() as *mut c_char,
        GRAPHICS_DIR_C.as_ptr() as *mut c_char,
        Themed::UseTheme as c_int,
        Criticality::Critical as c_int,
    );
    Load_Block(fpath, 0, 0, null_mut(), INIT_ONLY as c_int);
    Blastmap
        .iter_mut()
        .enumerate()
        .flat_map(|(blast_type_index, blast)| {
            blast
                .SurfacePointer
                .iter_mut()
                .enumerate()
                .map(move |(surface_index, surface)| (blast_type_index, surface_index, surface))
        })
        .for_each(|(blast_type_index, surface_index, surface)| {
            free_if_unused(*surface);
            *surface = Load_Block(
                null_mut(),
                blast_type_index.try_into().unwrap(),
                surface_index.try_into().unwrap(),
                &mut OrigBlock_Rect,
                0,
            );
        });

    update_progress(45);

    //---------- get Digit blocks
    let fpath = find_file(
        DIGIT_BLOCK_FILE_C.as_ptr() as *mut c_char,
        GRAPHICS_DIR_C.as_ptr() as *mut c_char,
        Themed::UseTheme as c_int,
        Criticality::Critical as c_int,
    );
    Load_Block(fpath, 0, 0, null_mut(), INIT_ONLY as c_int);
    InfluDigitSurfacePointer
        .iter_mut()
        .enumerate()
        .for_each(|(index, surface)| {
            free_if_unused(*surface);
            *surface = Load_Block(
                null_mut(),
                0,
                index.try_into().unwrap(),
                &mut OrigDigit_Rect,
                0,
            );
        });
    EnemyDigitSurfacePointer
        .iter_mut()
        .enumerate()
        .for_each(|(index, surface)| {
            free_if_unused(*surface);
            *surface = Load_Block(
                null_mut(),
                0,
                (index + 10).try_into().unwrap(),
                &mut OrigDigit_Rect,
                0,
            );
        });

    update_progress(50);

    //---------- get Takeover pics
    free_if_unused(to_blocks); /* this happens when we do theme-switching */
    let fpath = find_file(
        TO_BLOCK_FILE_C.as_ptr() as *mut c_char,
        GRAPHICS_DIR_C.as_ptr() as *mut c_char,
        Themed::UseTheme as c_int,
        Criticality::Critical as c_int,
    );
    to_blocks = Load_Block(fpath, 0, 0, null_mut(), 0);

    update_progress(60);

    free_if_unused(ship_on_pic);
    ship_on_pic = IMG_Load(find_file(
        SHIP_ON_PIC_FILE_C.as_ptr() as *mut c_char,
        GRAPHICS_DIR_C.as_ptr() as *mut c_char,
        Themed::UseTheme as c_int,
        Criticality::Critical as c_int,
    ));
    free_if_unused(ship_off_pic);
    ship_off_pic = IMG_Load(find_file(
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
            Block_Rect.w.into(),
            Block_Rect.h.into(),
            vid_bpp,
            0,
            0,
            0,
            0,
        );
        BuildBlock = SDL_DisplayFormatAlpha(tmp);
        SDL_FreeSurface(tmp);

        // takeover background pics
        let fpath = find_file(
            TAKEOVER_BG_PIC_FILE_C.as_ptr() as *mut c_char,
            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
            Themed::NoTheme as c_int,
            Criticality::Critical as c_int,
        );
        takeover_bg_pic = Load_Block(fpath, 0, 0, null_mut(), 0);
        set_takeover_rects(); // setup takeover rectangles

        // cursor shapes
        arrow_cursor = init_system_cursor(&ARROW_XPM);
        crosshair_cursor = init_system_cursor(&CROSSHAIR_XPM);
        //---------- get Console pictures
        let fpath = find_file(
            CONSOLE_PIC_FILE_C.as_ptr() as *mut c_char,
            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
            Themed::NoTheme as c_int,
            Criticality::Critical as c_int,
        );
        console_pic = Load_Block(fpath, 0, 0, null_mut(), 0);
        let fpath = find_file(
            CONSOLE_BG_PIC1_FILE_C.as_ptr() as *mut c_char,
            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
            Themed::NoTheme as c_int,
            Criticality::Critical as c_int,
        );
        console_bg_pic1 = Load_Block(fpath, 0, 0, null_mut(), 0);
        let fpath = find_file(
            CONSOLE_BG_PIC2_FILE_C.as_ptr() as *mut c_char,
            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
            Themed::NoTheme as c_int,
            Criticality::Critical as c_int,
        );
        console_bg_pic2 = Load_Block(fpath, 0, 0, null_mut(), 0);

        update_progress(80);

        arrow_up = IMG_Load(find_file(
            cstr!("arrow_up.png").as_ptr() as *mut c_char,
            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
            Themed::NoTheme as c_int,
            Criticality::Critical as c_int,
        ));
        arrow_down = IMG_Load(find_file(
            cstr!("arrow_down.png").as_ptr() as *mut c_char,
            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
            Themed::NoTheme as c_int,
            Criticality::Critical as c_int,
        ));
        arrow_right = IMG_Load(find_file(
            cstr!("arrow_right.png").as_ptr() as *mut c_char,
            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
            Themed::NoTheme as c_int,
            Criticality::Critical as c_int,
        ));
        arrow_left = IMG_Load(find_file(
            cstr!("arrow_left.png").as_ptr() as *mut c_char,
            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
            Themed::NoTheme as c_int,
            Criticality::Critical as c_int,
        ));
        //---------- get Banner
        let fpath = find_file(
            BANNER_BLOCK_FILE_C.as_ptr() as *mut c_char,
            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
            Themed::NoTheme as c_int,
            Criticality::Critical as c_int,
        );
        banner_pic = Load_Block(fpath, 0, 0, null_mut(), 0);

        update_progress(90);
        //---------- get Droid images ----------
        let droids = std::slice::from_raw_parts(Druidmap, Droid::NumDroids as usize);
        droids
            .iter()
            .zip(packed_portraits.iter_mut())
            .zip(portrait_raw_mem.iter_mut())
            .for_each(|((droid, packed_portrait), raw_portrait)| {
                // first check if we find a file with rotation-frames: first try .jpg
                libc::strcpy(fname.as_mut_ptr(), droid.druidname.as_ptr());
                libc::strcat(fname.as_mut_ptr(), cstr!(".jpg").as_ptr());
                let mut fpath = find_file(
                    fname.as_mut_ptr(),
                    GRAPHICS_DIR_C.as_ptr() as *mut c_char,
                    Themed::NoTheme as c_int,
                    Criticality::Ignore as c_int,
                );
                // then try with .png
                if fpath.is_null() {
                    libc::strcpy(fname.as_mut_ptr(), droid.druidname.as_ptr());
                    libc::strcat(fname.as_mut_ptr(), cstr!(".png").as_ptr());
                    fpath = find_file(
                        fname.as_mut_ptr(),
                        GRAPHICS_DIR_C.as_ptr() as *mut c_char,
                        Themed::NoTheme as c_int,
                        Criticality::Critical as c_int,
                    );
                }

                *packed_portrait = load_raw_pic(fpath, raw_portrait);
            });

        update_progress(95);
        // we need the 999.png in any case for transparency!
        libc::strcpy(
            fname.as_mut_ptr(),
            droids[Droid::Droid999 as usize].druidname.as_ptr(),
        );
        libc::strcat(fname.as_mut_ptr(), cstr!(".png").as_ptr());
        let fpath = find_file(
            fname.as_mut_ptr(),
            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
            Themed::NoTheme as c_int,
            Criticality::Critical as c_int,
        );
        pic999 = Load_Block(fpath, 0, 0, null_mut(), 0);

        // get the Ashes pics
        libc::strcpy(fname.as_mut_ptr(), cstr!("Ashes.png").as_ptr());
        let fpath = find_file(
            fname.as_mut_ptr(),
            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
            Themed::NoTheme as c_int,
            Criticality::WarnOnly as c_int,
        );
        if fpath.is_null() {
            warn!("deactivated display of droid-decals");
            GameConfig.ShowDecals = false.into();
        } else {
            Load_Block(fpath, 0, 0, null_mut(), INIT_ONLY as c_int);
            Decal_pics[0] = Load_Block(null_mut(), 0, 0, &mut OrigBlock_Rect, 0);
            Decal_pics[1] = Load_Block(null_mut(), 0, 1, &mut OrigBlock_Rect, 0);
        }
    });

    update_progress(96);
    // if scale != 1 then we need to rescale everything now
    ScaleGraphics(GameConfig.scale);

    update_progress(98);

    // make sure bullet-surfaces get re-generated!
    AllBullets
        .iter_mut()
        .take(MAXBULLETS)
        .for_each(|bullet| bullet.Surfaces_were_generated = false.into());

    SetCurrentFont(oldfont);

    true.into()
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
