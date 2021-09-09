use crate::{
    defs::{
        self, scale_point, scale_rect, Cmds, Criticality, DisplayBannerFlags, Droid, SoundType,
        Themed, BANNER_BLOCK_FILE_C, BLAST_BLOCK_FILE_C, BULLET_BLOCK_FILE_C,
        CONSOLE_BG_PIC1_FILE_C, CONSOLE_BG_PIC2_FILE_C, CONSOLE_PIC_FILE_C, DIGITNUMBER,
        DIGIT_BLOCK_FILE_C, DROID_BLOCK_FILE_C, ENEMYPHASES, FONT0_FILE, FONT0_FILE_C, FONT1_FILE,
        FONT1_FILE_C, FONT2_FILE, FONT2_FILE_C, FREE_ONLY, GRAPHICS_DIR_C, ICON_FILE, ICON_FILE_C,
        INIT_ONLY, MAP_BLOCK_FILE_C, MAXBULLETS, MAX_THEMES, NUM_COLORS, NUM_DECAL_PICS,
        NUM_MAP_BLOCKS, PARA_FONT_FILE, PARA_FONT_FILE_C, SHIP_OFF_PIC_FILE_C, SHIP_ON_PIC_FILE_C,
        TAKEOVER_BG_PIC_FILE_C,
    },
    misc::read_value_from_string,
    structs::ThemeList,
    takeover::TO_BLOCK_FILE_C,
    vars::{ORIG_BLOCK_RECT, ORIG_DIGIT_RECT},
    Data,
};

use array_init::array_init;
use cstr::cstr;
use log::{error, info, trace, warn};
use sdl::{FrameBuffer, Surface};
use sdl_sys::{
    zoomSurface, IMG_Load, SDL_CreateCursor, SDL_CreateRGBSurface, SDL_Cursor, SDL_Delay,
    SDL_DisplayFormat, SDL_DisplayFormatAlpha, SDL_FillRect, SDL_Flip, SDL_FreeCursor,
    SDL_FreeSurface, SDL_GetClipRect, SDL_GetError, SDL_GetTicks, SDL_GetVideoInfo, SDL_Init,
    SDL_InitSubSystem, SDL_MapRGB, SDL_MapRGBA, SDL_Quit, SDL_RWFromFile, SDL_RWFromMem, SDL_RWops,
    SDL_Rect, SDL_SaveBMP_RW, SDL_SetAlpha, SDL_SetGamma, SDL_SetVideoMode, SDL_UpdateRect,
    SDL_UpperBlit, SDL_VideoDriverName, SDL_VideoInfo, SDL_WM_SetCaption, SDL_WM_SetIcon,
    SDL_FULLSCREEN, SDL_INIT_TIMER, SDL_INIT_VIDEO, SDL_RLEACCEL, SDL_SRCALPHA,
};
use std::{
    cell::RefCell,
    convert::{TryFrom, TryInto},
    ffi::CStr,
    ops::Not,
    os::raw::{c_char, c_float, c_int, c_short, c_void},
    ptr::{null_mut, NonNull},
    rc::Rc,
};

#[derive(Debug)]
pub struct Graphics {
    vid_info: *const SDL_VideoInfo,
    pub vid_bpp: c_int,
    portrait_raw_mem: [Option<Box<[u8]>>; Droid::NumDroids as usize],
    fonts_loaded: c_int,
    // A pointer to the surfaces containing the map-pics, which may be rescaled with respect to
    pub map_block_surface_pointer: [[Option<Rc<RefCell<Surface>>>; NUM_MAP_BLOCKS]; NUM_COLORS],
    // A pointer to the surfaces containing the original map-pics as read from disk
    orig_map_block_surface_pointer: [[Option<Rc<RefCell<Surface>>>; NUM_MAP_BLOCKS]; NUM_COLORS],
    // a block for temporary pic-construction
    pub build_block: Option<Surface>,
    pub banner_is_destroyed: i32,
    /* the banner pic */
    pub banner_pic: Option<Surface>,
    pub pic999: Option<Surface>,
    pub packed_portraits: [*mut SDL_RWops; Droid::NumDroids as usize],
    pub decal_pics: [Option<Surface>; NUM_DECAL_PICS],
    pub takeover_bg_pic: Option<Surface>,
    pub console_pic: Option<Surface>,
    pub console_bg_pic1: Option<Surface>,
    pub console_bg_pic2: Option<Surface>,
    pub arrow_up: Option<Surface>,
    pub arrow_down: Option<Surface>,
    pub arrow_right: Option<Surface>,
    pub arrow_left: Option<Surface>,
    // Side-view of ship: lights off
    pub ship_off_pic: Option<Surface>,
    // Side-view of ship: lights on
    pub ship_on_pic: Option<Surface>,
    pub progress_meter_pic: Option<Surface>,
    pub progress_filler_pic: Option<Surface>,
    /* the graphics display */
    pub ne_screen: Option<sdl::FrameBuffer>,
    pub enemy_surface_pointer: [Option<Surface>; ENEMYPHASES as usize],
    pub influencer_surface_pointer: [Option<Surface>; ENEMYPHASES as usize],
    pub influ_digit_surface_pointer: [Option<Surface>; DIGITNUMBER],
    pub enemy_digit_surface_pointer: [Option<Surface>; DIGITNUMBER],
    pub crosshair_cursor: *mut SDL_Cursor,
    pub arrow_cursor: *mut SDL_Cursor,
    pub number_of_bullet_types: i32,
    pub all_themes: ThemeList,
    pub classic_theme_index: i32,
    number_of_screenshot: u32,
    pic: Option<Surface>,
}

impl Default for Graphics {
    fn default() -> Self {
        Self {
            vid_info: null_mut(),
            vid_bpp: 0,
            portrait_raw_mem: array_init(|_| None),
            fonts_loaded: 0,
            map_block_surface_pointer: array_init(|_| array_init(|_| None)),
            orig_map_block_surface_pointer: array_init(|_| array_init(|_| None)),
            build_block: None,
            banner_is_destroyed: 0,
            banner_pic: None,
            pic999: None,
            packed_portraits: [null_mut(); Droid::NumDroids as usize],
            decal_pics: array_init(|_| None),
            takeover_bg_pic: None,
            console_pic: None,
            console_bg_pic1: None,
            console_bg_pic2: None,
            arrow_up: None,
            arrow_down: None,
            arrow_right: None,
            arrow_left: None,
            ship_off_pic: None,
            ship_on_pic: None,
            progress_meter_pic: None,
            progress_filler_pic: None,
            ne_screen: None,
            enemy_surface_pointer: array_init(|_| None),
            influencer_surface_pointer: array_init(|_| None),
            influ_digit_surface_pointer: array_init(|_| None),
            enemy_digit_surface_pointer: array_init(|_| None),
            crosshair_cursor: null_mut(),
            arrow_cursor: null_mut(),
            number_of_bullet_types: 0,
            all_themes: ThemeList {
                num_themes: 0,
                cur_tnum: 0,
                theme_name: [null_mut(); MAX_THEMES],
            },
            classic_theme_index: 0,
            number_of_screenshot: 0,
            pic: None,
        }
    }
}

impl Data {
    /// This function draws a "grid" on the screen, that means every
    /// "second" pixel is blacked out, thereby generation a fading
    /// effect.  This function was created to fade the background of the
    /// Escape menu and its submenus.
    pub fn make_grid_on_screen(&mut self, grid_rectangle: Option<&SDL_Rect>) {
        let grid_rectangle = grid_rectangle.unwrap_or(&self.vars.user_rect);

        trace!("MakeGridOnScreen(...): real function call confirmed.");
        let ne_screen = self.graphics.ne_screen.as_mut().unwrap();
        let rect_x = u16::try_from(grid_rectangle.x).unwrap();
        let rect_y = u16::try_from(grid_rectangle.y).unwrap();
        let mut ne_screen = ne_screen.lock().unwrap();
        (rect_y..(rect_y + grid_rectangle.h))
            .flat_map(|y| (rect_x..(rect_x + grid_rectangle.w)).map(move |x| (x, y)))
            .filter(|(x, y)| (x + y) % 2 == 0)
            .for_each(|(x, y)| ne_screen.pixels().set(x, y, 0).unwrap());
        trace!("MakeGridOnScreen(...): end of function reached.");
    }
}

pub unsafe fn apply_filter(
    surface: &mut Surface,
    fred: c_float,
    fgreen: c_float,
    fblue: c_float,
) -> c_int {
    let w = surface.width();
    (0..surface.height())
        .flat_map(move |y| (0..w).map(move |x| (x, y)))
        .for_each(|(x, y)| {
            let [mut red, mut green, mut blue, alpha] =
                surface.lock().unwrap().pixels().get(x, y).unwrap().rgba();
            if alpha == 0 {
                return;
            }

            red = (red as c_float * fred) as u8;
            green = (green as c_float * fgreen) as u8;
            blue = (blue as c_float * fblue) as u8;

            let pixel_value = SDL_MapRGBA(surface.raw().format(), red, green, blue, alpha);
            let mut surface = surface.lock().unwrap();
            surface.pixels().set(x, y, pixel_value).unwrap();
        });

    defs::OK.into()
}

impl Data {
    pub unsafe fn toggle_fullscreen(&mut self) {
        let ne_screen = self.graphics.ne_screen.as_mut().unwrap();
        let mut vid_flags = ne_screen.flags();

        if self.global.game_config.use_fullscreen != 0 {
            vid_flags &= !(SDL_FULLSCREEN as u32);
        } else {
            vid_flags |= SDL_FULLSCREEN as u32;
        }

        let new_ne_screen = SDL_SetVideoMode(
            self.vars.screen_rect.w.into(),
            self.vars.screen_rect.h.into(),
            0,
            vid_flags,
        );
        let new_ne_screen = match NonNull::new(new_ne_screen) {
            Some(ptr) => ptr,
            None => {
                error!(
                    "unable to toggle windowed/fullscreen {} x {} video mode.",
                    self.vars.screen_rect.w, self.vars.screen_rect.h,
                );
                panic!(
                    "SDL-Error: {}",
                    CStr::from_ptr(SDL_GetError()).to_string_lossy()
                );
            }
        };

        *ne_screen = sdl::FrameBuffer::from_ptr(new_ne_screen);

        if ne_screen.flags() != vid_flags {
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
        self.activate_conservative_frame_computation();

        let screenshot_filename =
            format!("Screenshot_{}.bmp\0", self.graphics.number_of_screenshot);
        SDL_SaveBMP_RW(
            self.graphics.ne_screen.as_mut().unwrap().as_mut_ptr(),
            SDL_RWFromFile(
                screenshot_filename.as_ptr() as *const c_char,
                cstr!("wb").as_ptr(),
            ),
            1,
        );
        self.graphics.number_of_screenshot = self.graphics.number_of_screenshot.wrapping_add(1);
        self.display_banner(
            cstr!("Screenshot").as_ptr(),
            null_mut(),
            (DisplayBannerFlags::NO_SDL_UPDATE | DisplayBannerFlags::FORCE_UPDATE)
                .bits()
                .into(),
        );
        self.make_grid_on_screen(None);
        SDL_Flip(self.graphics.ne_screen.as_mut().unwrap().as_mut_ptr());
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

impl Data {
    pub unsafe fn free_graphics(&mut self) {
        // free RWops structures
        self.graphics
            .packed_portraits
            .iter()
            .filter(|packed_portrait| !packed_portrait.is_null())
            .for_each(|&packed_portrait| {
                let close: unsafe fn(context: *mut SDL_RWops) -> c_int =
                    std::mem::transmute((*packed_portrait).close);
                close(packed_portrait);
            });

        self.graphics
            .portrait_raw_mem
            .iter_mut()
            .for_each(|mem| drop(mem.take()));

        self.graphics.enemy_surface_pointer = array_init(|_| None);
        self.graphics.influencer_surface_pointer = array_init(|_| None);
        self.graphics.influ_digit_surface_pointer = array_init(|_| None);
        self.graphics.enemy_digit_surface_pointer = array_init(|_| None);
        self.graphics.decal_pics = array_init(|_| None);

        self.graphics
            .orig_map_block_surface_pointer
            .iter_mut()
            .flat_map(|arr| arr.iter_mut())
            .for_each(|surface| *surface = None);

        self.graphics.build_block = None;
        self.graphics.banner_pic = None;
        self.graphics.pic999 = None;
        // SDL_RWops *packed_portraits[NUM_DROIDS];
        self.graphics.takeover_bg_pic = None;
        self.graphics.console_pic = None;
        self.graphics.console_bg_pic1 = None;
        self.graphics.console_bg_pic2 = None;

        self.graphics.arrow_up = None;
        self.graphics.arrow_down = None;
        self.graphics.arrow_right = None;
        self.graphics.arrow_left = None;

        self.graphics.ship_off_pic = None;
        self.graphics.ship_on_pic = None;
        self.graphics.progress_meter_pic = None;
        self.graphics.progress_filler_pic = None;
        self.takeover.to_blocks = None;

        // free fonts
        let fonts = [
            self.global.menu_b_font,
            self.global.para_b_font,
            self.global.highscore_b_font,
            self.global.font0_b_font,
            self.global.font1_b_font,
            self.global.font2_b_font,
        ];
        for (index, font) in fonts.iter().copied().enumerate() {
            if font.is_null() || fonts[..index].contains(&font) {
                continue;
            }
            drop(Box::from_raw(font));
        }

        // free Load_Block()-internal buffer
        self.graphics
            .load_block(null_mut(), 0, 0, null_mut(), FREE_ONLY as i32);

        // free cursors
        SDL_FreeCursor(self.graphics.crosshair_cursor);
        SDL_FreeCursor(self.graphics.arrow_cursor);
    }
}

impl Graphics {
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
        &mut self,
        fpath: *mut c_char,
        line: c_int,
        col: c_int,
        block: *const SDL_Rect,
        flags: c_int,
    ) -> Option<Surface> {
        Self::load_block_vid_bpp_pic(self.vid_bpp, &mut self.pic, fpath, line, col, block, flags)
    }

    pub unsafe fn load_block_vid_bpp_pic(
        vid_bpp: i32,
        pic: &mut Option<Surface>,
        fpath: *mut c_char,
        line: c_int,
        col: c_int,
        block: *const SDL_Rect,
        flags: c_int,
    ) -> Option<Surface> {
        if fpath.is_null() && pic.is_none() {
            /* we need some info.. */
            return None;
        }

        if pic.is_some() && flags == FREE_ONLY as c_int {
            *pic = None;
            return None;
        }

        if !fpath.is_null() {
            // initialize: read & malloc new pic, dont' return a copy!!
            *pic = Some(Surface::from_ptr(NonNull::new(IMG_Load(fpath)).unwrap()));
        }

        if (flags & INIT_ONLY as c_int) != 0 {
            return None; // that's it guys, only initialzing...
        }

        let pic = pic.as_mut().unwrap();
        let dim = if block.is_null() {
            rect!(0, 0, pic.width(), pic.height())
        } else {
            let block = &*block;
            rect!(0, 0, block.w, block.h)
        };

        let raw_format = pic.raw().format();
        assert!(raw_format.is_null().not());
        let usealpha = (*raw_format).Amask != 0;

        if usealpha {
            SDL_SetAlpha(pic.as_mut_ptr(), 0, 0); /* clear per-surf alpha for internal blit */
        }
        let tmp = SDL_CreateRGBSurface(0, dim.w.into(), dim.h.into(), vid_bpp, 0, 0, 0, 0);
        let mut ret = if usealpha {
            Surface::from_ptr(NonNull::new(SDL_DisplayFormatAlpha(tmp)).unwrap())
        } else {
            Surface::from_ptr(NonNull::new(SDL_DisplayFormat(tmp)).unwrap())
        };
        SDL_FreeSurface(tmp);

        let mut src = rect!(
            i16::try_from(col).unwrap() * i16::try_from(dim.w + 2).unwrap(),
            i16::try_from(line).unwrap() * i16::try_from(dim.h + 2).unwrap(),
            dim.w,
            dim.h,
        );
        SDL_UpperBlit(pic.as_mut_ptr(), &mut src, ret.as_mut_ptr(), null_mut());
        if usealpha {
            SDL_SetAlpha(
                ret.as_mut_ptr(),
                SDL_SRCALPHA as u32 | SDL_RLEACCEL as u32,
                255,
            );
        }

        Some(ret)
    }
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
            ($point:expr) => {
                scale_point(&mut $point, scale);
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

        for block in &mut self.takeover.fill_blocks {
            scale_rect(block, scale);
        }

        for block in &mut self.takeover.capsule_blocks {
            scale_rect(block, scale);
        }

        for block in &mut self.takeover.to_game_blocks {
            scale_rect(block, scale);
        }

        for block in &mut self.takeover.to_ground_blocks {
            scale_rect(block, scale);
        }

        scale!(self.takeover.column_block);
        scale!(self.takeover.leader_block);

        for point in &mut self.takeover.left_capsule_starts {
            scale_point(point, scale);
        }
        for point in &mut self.takeover.cur_capsule_starts {
            scale_point(point, scale);
        }
        for point in &mut self.takeover.playground_starts {
            scale_point(point, scale);
        }
        for point in &mut self.takeover.droid_starts {
            scale_point(point, scale);
        }
        scale_point!(self.takeover.left_ground_start);
        scale_point!(self.takeover.left_ground_start);
        scale_point!(self.takeover.column_start);
        scale_point!(self.takeover.right_ground_start);
        scale_point!(self.takeover.leader_block_start);

        scale!(self.takeover.fill_block);
        scale!(self.takeover.element_rect);
        scale!(self.takeover.capsule_rect);
        scale!(self.takeover.leader_led);
        scale!(self.takeover.ground_rect);
        scale!(self.takeover.column_rect);
    }
}

pub fn scale_pic(pic: &mut Surface, scale: c_float) {
    if (scale - 1.0).abs() <= f32::EPSILON {
        return;
    }
    let scale = scale.into();

    // # Safety
    // [`zoomSurface`] creates a new SDL_Surface or returns null, therefore no aliasing can occur.
    unsafe {
        *pic = Surface::from_ptr(
            NonNull::new(zoomSurface(pic.as_mut_ptr(), scale, scale, 0))
                .unwrap_or_else(|| panic!("zoomSurface() failed for scale = {}.", scale)),
        );
    }
}

pub fn scale_pic_surface(pic: &mut Surface, scale: c_float) {
    if (scale - 1.0).abs() <= f32::EPSILON {
        return;
    }
    let scale = scale.into();

    let new_pic = NonNull::new(unsafe { zoomSurface(pic.as_mut_ptr(), scale, scale, 0) });
    match new_pic {
        Some(new_pic) => *pic = unsafe { Surface::from_ptr(new_pic) },
        None => panic!("zoomSurface() failed for scale = {}.", scale),
    }
}

impl Data {
    pub unsafe fn scale_graphics(&mut self, scale: c_float) {
        static INIT: std::sync::Once = std::sync::Once::new();

        /* For some reason we need to SetAlpha every time on OS X */
        /* Digits are only used in _internal_ blits ==> clear per-surf alpha */
        self.graphics
            .influ_digit_surface_pointer
            .iter_mut()
            .flatten()
            .for_each(|surface| {
                SDL_SetAlpha(surface.as_mut_ptr(), 0, 0);
            });
        self.graphics
            .enemy_digit_surface_pointer
            .iter_mut()
            .flatten()
            .for_each(|surface| {
                SDL_SetAlpha(surface.as_mut_ptr(), 0, 0);
            });
        if (scale - 1.).abs() <= f32::EPSILON {
            return;
        }

        // these are reset in a theme-change by the theme-config-file
        // therefore we need to rescale them each time again
        scale_rect(&mut self.main.first_digit_rect, scale);
        scale_rect(&mut self.main.second_digit_rect, scale);
        scale_rect(&mut self.main.third_digit_rect, scale);

        // note: only rescale these rects the first time!!
        let mut init = false;
        INIT.call_once(|| {
            init = true;
            self.scale_stat_rects(scale);
        });

        //---------- rescale Map blocks
        self.graphics
            .orig_map_block_surface_pointer
            .iter_mut()
            .flat_map(|surfaces| surfaces.iter_mut())
            .zip(
                self.graphics
                    .map_block_surface_pointer
                    .iter_mut()
                    .flat_map(|surfaces| surfaces.iter_mut()),
            )
            .for_each(|(orig_surface, map_surface)| {
                let orig_surface = orig_surface.as_mut().unwrap();
                scale_pic(&mut *orig_surface.borrow_mut(), scale);
                *map_surface = Some(Rc::clone(orig_surface));
            });

        //---------- rescale Droid-model  blocks
        /* Droid pics are only used in _internal_ blits ==> clear per-surf alpha */
        for surface in &mut self.graphics.influencer_surface_pointer {
            let surface = surface.as_mut().unwrap();
            scale_pic(surface, scale);
            SDL_SetAlpha(surface.as_mut_ptr(), 0, 0);
        }
        for surface in &mut self.graphics.enemy_surface_pointer {
            let surface = surface.as_mut().unwrap();
            scale_pic(surface, scale);
            SDL_SetAlpha(surface.as_mut_ptr(), 0, 0);
        }

        //---------- rescale Bullet blocks
        let bulletmap = std::slice::from_raw_parts_mut(
            self.vars.bulletmap,
            usize::try_from(self.graphics.number_of_bullet_types).unwrap(),
        );
        bulletmap
            .iter_mut()
            .flat_map(|bullet| bullet.surfaces.iter_mut())
            .for_each(|surface| scale_pic(surface.as_mut().unwrap(), scale));

        //---------- rescale Blast blocks
        self.vars
            .blastmap
            .iter_mut()
            .flat_map(|blast| blast.surfaces.iter_mut())
            .for_each(|surface| scale_pic(surface.as_mut().unwrap(), scale));

        //---------- rescale Digit blocks
        for surface in &mut self.graphics.influ_digit_surface_pointer {
            let surface = surface.as_mut().unwrap();
            scale_pic(surface, scale);
            SDL_SetAlpha(surface.as_mut_ptr(), 0, 0);
        }
        for surface in &mut self.graphics.enemy_digit_surface_pointer {
            let surface = surface.as_mut().unwrap();
            scale_pic(surface, scale);
            SDL_SetAlpha(surface.as_mut_ptr(), 0, 0);
        }

        //---------- rescale Takeover pics
        scale_pic(self.takeover.to_blocks.as_mut().unwrap(), scale);

        scale_pic(self.graphics.ship_on_pic.as_mut().unwrap(), scale);
        scale_pic(self.graphics.ship_off_pic.as_mut().unwrap(), scale);

        // the following are not theme-specific and are therefore only loaded once!
        if init {
            //  create a new tmp block-build storage
            let tmp = SDL_CreateRGBSurface(
                0,
                self.vars.block_rect.w.into(),
                self.vars.block_rect.h.into(),
                self.graphics.vid_bpp,
                0,
                0,
                0,
                0,
            );
            self.graphics.build_block = Some(Surface::from_ptr(
                NonNull::new(SDL_DisplayFormatAlpha(tmp)).unwrap(),
            ));
            SDL_FreeSurface(tmp);

            // takeover pics
            scale_pic(self.graphics.takeover_bg_pic.as_mut().unwrap(), scale);

            //---------- Console pictures
            scale_pic(self.graphics.console_pic.as_mut().unwrap(), scale);
            scale_pic(self.graphics.console_bg_pic1.as_mut().unwrap(), scale);
            scale_pic(self.graphics.console_bg_pic2.as_mut().unwrap(), scale);
            scale_pic(self.graphics.arrow_up.as_mut().unwrap(), scale);
            scale_pic(self.graphics.arrow_down.as_mut().unwrap(), scale);
            scale_pic(self.graphics.arrow_right.as_mut().unwrap(), scale);
            scale_pic(self.graphics.arrow_left.as_mut().unwrap(), scale);
            //---------- Banner
            scale_pic(self.graphics.banner_pic.as_mut().unwrap(), scale);

            scale_pic(self.graphics.pic999.as_mut().unwrap(), scale);

            // get the Ashes pics
            if let Some(pic) = self.graphics.decal_pics[0].as_mut() {
                scale_pic(pic, scale);
            }
            if let Some(pic) = self.graphics.decal_pics[1].as_mut() {
                scale_pic(pic, scale);
            }
        }

        let mut ne_screen = self.graphics.ne_screen.take().unwrap();
        self.printf_sdl(&mut ne_screen, -1, -1, format_args!(" ok\n"));
        self.graphics.ne_screen = Some(ne_screen);
    }

    /// display "white noise" effect in SDL_Rect.
    /// algorith basically stolen from
    /// Greg Knauss's "xteevee" hack in xscreensavers.
    ///
    /// timeout is in ms
    pub unsafe fn white_noise(
        &mut self,
        frame_buffer: &mut FrameBuffer,
        rect: &mut SDL_Rect,
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
            SDL_MapRGB(frame_buffer.format().as_ptr(), color, color, color)
        });

        // produce the tiles
        let tmp = SDL_CreateRGBSurface(
            0,
            rect.w.into(),
            rect.h.into(),
            self.graphics.vid_bpp,
            0,
            0,
            0,
            0,
        );
        let tmp2 = SDL_DisplayFormat(tmp);
        SDL_FreeSurface(tmp);
        SDL_UpperBlit(frame_buffer.as_mut_ptr(), rect, tmp2, null_mut());

        let mut rng = rand::thread_rng();
        let mut noise_tiles: [Surface; NOISE_TILES] = array_init(|_| {
            let mut tile = Surface::from_ptr(NonNull::new(SDL_DisplayFormat(tmp2)).unwrap());
            let mut lock = tile.lock().unwrap();
            (0..u16::try_from(rect.x).unwrap())
                .flat_map(|x| (0..rect.h).map(move |y| (x, y)))
                .for_each(|(x, y)| {
                    if rng.gen_range(0, 100) > signal_strengh {
                        lock.pixels()
                            .set(x, y, *grey.choose(&mut rng).unwrap())
                            .unwrap();
                    }
                });
            drop(lock);
            tile
        });
        SDL_FreeSurface(tmp2);

        let mut used_tiles: [c_char; NOISE_TILES / 2 + 1] = [-1; NOISE_TILES / 2 + 1];
        // let's go
        self.play_sound(SoundType::WhiteNoise as c_int);

        let now = SDL_GetTicks();

        self.wait_for_all_keys_released();
        let mut clip_rect = rect!();
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
            SDL_GetClipRect(frame_buffer.as_mut_ptr(), &mut clip_rect);
            frame_buffer.clear_clip_rect();
            // set it
            SDL_UpperBlit(
                noise_tiles[usize::try_from(next_tile).unwrap()].as_mut_ptr(),
                null_mut(),
                frame_buffer.as_mut_ptr(),
                rect,
            );
            SDL_UpdateRect(
                frame_buffer.as_mut_ptr(),
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
        frame_buffer.set_clip_rect(&clip_rect);
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

        self.global.menu_b_font = self.global.para_b_font;
        self.global.highscore_b_font = self.global.para_b_font;

        self.graphics.fonts_loaded = true.into();

        defs::OK.into()
    }

    pub unsafe fn clear_graph_mem(&mut self) {
        // One this function is done, the rahmen at the
        // top of the screen surely is destroyed.  We inform the
        // DisplayBanner function of the matter...
        self.graphics.banner_is_destroyed = true.into();

        let ne_screen = self.graphics.ne_screen.as_mut().unwrap();
        ne_screen.clear_clip_rect();

        // Now we fill the screen with black color...
        SDL_FillRect(ne_screen.as_mut_ptr(), null_mut(), 0);
        SDL_Flip(ne_screen.as_mut_ptr());
    }

    /// Initialise the Video display and graphics engine
    pub unsafe fn init_video(&mut self) {
        const YN: [&str; 2] = ["no", "yes"];

        /* Initialize the SDL library */
        // if ( SDL_Init (SDL_INIT_VIDEO | SDL_INIT_TIMER) == -1 )

        if SDL_Init(SDL_INIT_VIDEO as u32) == -1 {
            panic!(
                "Couldn't initialize SDL: {}",
                CStr::from_ptr(SDL_GetError()).to_string_lossy()
            );
        } else {
            info!("SDL Video initialisation successful.");
        }

        // Now SDL_TIMER is initialized here:

        if SDL_InitSubSystem(SDL_INIT_TIMER as u32) == -1 {
            panic!(
                "Couldn't initialize SDL: {}",
                CStr::from_ptr(SDL_GetError()).to_string_lossy()
            );
        } else {
            info!("SDL Timer initialisation successful.");
        }

        /* clean up on exit */
        libc::atexit(std::mem::transmute(SDL_Quit as unsafe extern "C" fn()));

        self.graphics.vid_info = SDL_GetVideoInfo(); /* just curious */
        let mut vid_driver: [c_char; 81] = [0; 81];
        SDL_VideoDriverName(vid_driver.as_mut_ptr(), 80);

        let vid_info_ref = *self.graphics.vid_info;
        if cfg!(os_target = "android") {
            self.graphics.vid_bpp = 16; // Hardcoded Android default
        } else {
            self.graphics.vid_bpp = (*vid_info_ref.vfmt).BitsPerPixel.into();
        }

        macro_rules! flag {
            ($flag:ident) => {
                (vid_info_ref.$flag()) != 0
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
            flag_yn!(hw_available)
        );
        info!(
            "Is there a window manager available: {}",
            flag_yn!(wm_available)
        );
        info!(
            "Are hardware to hardware blits accelerated: {}",
            flag_yn!(blit_hw)
        );
        info!(
            "Are hardware to hardware colorkey blits accelerated: {}",
            flag_yn!(blit_hw_CC)
        );
        info!(
            "Are hardware to hardware alpha blits accelerated: {}",
            flag_yn!(blit_hw_A)
        );
        info!(
            "Are software to hardware blits accelerated: {}",
            flag_yn!(blit_sw)
        );
        info!(
            "Are software to hardware colorkey blits accelerated: {}",
            flag_yn!(blit_sw_CC)
        );
        info!(
            "Are software to hardware alpha blits accelerated: {}",
            flag_yn!(blit_sw_A)
        );
        info!("Are color fills accelerated: {}", flag_yn!(blit_fill));
        info!(
            "Total amount of video memory in Kilobytes: {}",
            vid_info_ref.video_mem
        );
        info!(
            "Pixel format of the video device: bpp = {}, bytes/pixel = {}",
            self.graphics.vid_bpp,
            (*vid_info_ref.vfmt).BytesPerPixel
        );
        info!(
            "Video Driver Name: {}",
            CStr::from_ptr(vid_driver.as_ptr()).to_string_lossy()
        );
        info!("----------------------------------------------------------------------");

        let vid_flags = if self.global.game_config.use_fullscreen != 0 {
            SDL_FULLSCREEN as u32
        } else {
            0
        };

        if flag!(wm_available) {
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

        let ne_screen = SDL_SetVideoMode(
            self.vars.screen_rect.w.into(),
            self.vars.screen_rect.h.into(),
            0,
            vid_flags,
        );
        let ne_screen = match NonNull::new(ne_screen) {
            Some(ne_screen) => ne_screen,
            None => {
                error!(
                    "Couldn't set {} x {} video mode. SDL: {}",
                    self.vars.screen_rect.w,
                    self.vars.screen_rect.h,
                    CStr::from_ptr(SDL_GetError()).to_string_lossy(),
                );
                std::process::exit(-1);
            }
        };
        self.graphics.ne_screen = Some(sdl::FrameBuffer::from_ptr(ne_screen));

        self.graphics.vid_info = SDL_GetVideoInfo(); /* info about current video mode */

        info!("Got video mode: ");

        SDL_SetGamma(1., 1., 1.);
        self.global.game_config.current_gamma_correction = 1.;
    }

    /// load a pic into memory and return the SDL_RWops pointer to it
    pub unsafe fn load_raw_pic(
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

        if self.graphics.fonts_loaded == 0 {
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
        self.graphics
            .load_block(fpath, 0, 0, null_mut(), INIT_ONLY as i32); /* init function */
        let Self {
            graphics:
                Graphics {
                    map_block_surface_pointer,
                    vid_bpp,
                    orig_map_block_surface_pointer,
                    pic,
                    ..
                },
            ..
        } = self;
        orig_map_block_surface_pointer
            .iter_mut()
            .enumerate()
            .zip(map_block_surface_pointer.iter_mut())
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
                *orig_surface = Graphics::load_block_vid_bpp_pic(
                    *vid_bpp,
                    pic,
                    null_mut(),
                    color_index.try_into().unwrap(),
                    block_index.try_into().unwrap(),
                    &ORIG_BLOCK_RECT,
                    0,
                )
                .map(|surface| Rc::new(RefCell::new(surface)));
                *surface = orig_surface.as_ref().map(Rc::clone);
            });

        self.update_progress(20);
        //---------- get Droid-model  blocks
        let fpath = self.find_file(
            DROID_BLOCK_FILE_C.as_ptr() as *mut c_char,
            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
            Themed::UseTheme as c_int,
            Criticality::Critical as c_int,
        );
        self.graphics
            .load_block(fpath, 0, 0, null_mut(), INIT_ONLY as c_int);

        let Self {
            graphics:
                Graphics {
                    vid_bpp,
                    pic,
                    influencer_surface_pointer,
                    enemy_surface_pointer,
                    ..
                },
            ..
        } = self;

        influencer_surface_pointer.iter_mut().enumerate().for_each(
            |(index, influencer_surface)| {
                *influencer_surface = Graphics::load_block_vid_bpp_pic(
                    *vid_bpp,
                    pic,
                    null_mut(),
                    0,
                    index.try_into().unwrap(),
                    &ORIG_BLOCK_RECT,
                    0,
                );

                /* Droid pics are only used in _internal_ blits ==> clear per-surf alpha */
                SDL_SetAlpha(influencer_surface.as_mut().unwrap().as_mut_ptr(), 0, 0);
            },
        );

        enemy_surface_pointer
            .iter_mut()
            .enumerate()
            .for_each(|(index, enemy_surface)| {
                *enemy_surface = Graphics::load_block_vid_bpp_pic(
                    *vid_bpp,
                    pic,
                    null_mut(),
                    1,
                    index.try_into().unwrap(),
                    &ORIG_BLOCK_RECT,
                    0,
                );

                /* Droid pics are only used in _internal_ blits ==> clear per-surf alpha */
                SDL_SetAlpha(enemy_surface.as_mut().unwrap().as_mut_ptr(), 0, 0);
            });

        self.update_progress(30);
        //---------- get Bullet blocks
        let fpath = self.find_file(
            BULLET_BLOCK_FILE_C.as_ptr() as *mut c_char,
            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
            Themed::UseTheme as c_int,
            Criticality::Critical as c_int,
        );
        self.graphics
            .load_block(fpath, 0, 0, null_mut(), INIT_ONLY as c_int);
        std::slice::from_raw_parts_mut(
            self.vars.bulletmap,
            self.graphics.number_of_bullet_types.try_into().unwrap(),
        )
        .iter_mut()
        .enumerate()
        .flat_map(|(bullet_type_index, bullet)| {
            bullet
                .surfaces
                .iter_mut()
                .enumerate()
                .map(move |(phase_index, surface)| (bullet_type_index, phase_index, surface))
        })
        .for_each(|(bullet_type_index, phase_index, surface)| {
            *surface = self.graphics.load_block(
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
        self.graphics
            .load_block(fpath, 0, 0, null_mut(), INIT_ONLY as c_int);

        let Self { vars, graphics, .. } = self;
        vars.blastmap
            .iter_mut()
            .enumerate()
            .flat_map(|(blast_type_index, blast)| {
                blast
                    .surfaces
                    .iter_mut()
                    .enumerate()
                    .map(move |(surface_index, surface)| (blast_type_index, surface_index, surface))
            })
            .for_each(|(blast_type_index, surface_index, surface)| {
                *surface = graphics.load_block(
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
        self.graphics
            .load_block(fpath, 0, 0, null_mut(), INIT_ONLY as c_int);
        let Self {
            graphics:
                Graphics {
                    vid_bpp,
                    pic,
                    influ_digit_surface_pointer,
                    enemy_digit_surface_pointer,
                    ..
                },
            ..
        } = self;
        influ_digit_surface_pointer
            .iter_mut()
            .enumerate()
            .for_each(|(index, surface)| {
                *surface = Graphics::load_block_vid_bpp_pic(
                    *vid_bpp,
                    pic,
                    null_mut(),
                    0,
                    index.try_into().unwrap(),
                    &ORIG_DIGIT_RECT,
                    0,
                );
            });
        enemy_digit_surface_pointer
            .iter_mut()
            .enumerate()
            .for_each(|(index, surface)| {
                *surface = Graphics::load_block_vid_bpp_pic(
                    *vid_bpp,
                    pic,
                    null_mut(),
                    0,
                    (index + 10).try_into().unwrap(),
                    &ORIG_DIGIT_RECT,
                    0,
                );
            });

        self.update_progress(50);

        //---------- get Takeover pics
        let fpath = self.find_file(
            TO_BLOCK_FILE_C.as_ptr() as *mut c_char,
            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
            Themed::UseTheme as c_int,
            Criticality::Critical as c_int,
        );
        self.takeover.to_blocks = self.graphics.load_block(fpath, 0, 0, null_mut(), 0);

        self.update_progress(60);

        self.graphics.ship_on_pic = Some(Surface::from_ptr(
            NonNull::new(IMG_Load(self.find_file(
                SHIP_ON_PIC_FILE_C.as_ptr() as *mut c_char,
                GRAPHICS_DIR_C.as_ptr() as *mut c_char,
                Themed::UseTheme as c_int,
                Criticality::Critical as c_int,
            )))
            .unwrap(),
        ));
        self.graphics.ship_off_pic = Some(Surface::from_ptr(
            NonNull::new(IMG_Load(self.find_file(
                SHIP_OFF_PIC_FILE_C.as_ptr() as *mut c_char,
                GRAPHICS_DIR_C.as_ptr() as *mut c_char,
                Themed::UseTheme as c_int,
                Criticality::Critical as c_int,
            )))
            .unwrap(),
        ));

        // the following are not theme-specific and are therefore only loaded once!
        DO_ONCE.call_once(|| {
            //  create the tmp block-build storage
            let tmp = SDL_CreateRGBSurface(
                0,
                self.vars.block_rect.w.into(),
                self.vars.block_rect.h.into(),
                self.graphics.vid_bpp,
                0,
                0,
                0,
                0,
            );
            self.graphics.build_block = Some(Surface::from_ptr(
                NonNull::new(SDL_DisplayFormatAlpha(tmp)).unwrap(),
            ));
            SDL_FreeSurface(tmp);

            // takeover background pics
            let fpath = self.find_file(
                TAKEOVER_BG_PIC_FILE_C.as_ptr() as *mut c_char,
                GRAPHICS_DIR_C.as_ptr() as *mut c_char,
                Themed::NoTheme as c_int,
                Criticality::Critical as c_int,
            );
            self.graphics.takeover_bg_pic = self.graphics.load_block(fpath, 0, 0, null_mut(), 0);
            self.set_takeover_rects(); // setup takeover rectangles

            // cursor shapes
            self.graphics.arrow_cursor = init_system_cursor(&ARROW_XPM);
            self.graphics.crosshair_cursor = init_system_cursor(&CROSSHAIR_XPM);
            //---------- get Console pictures
            let fpath = self.find_file(
                CONSOLE_PIC_FILE_C.as_ptr() as *mut c_char,
                GRAPHICS_DIR_C.as_ptr() as *mut c_char,
                Themed::NoTheme as c_int,
                Criticality::Critical as c_int,
            );
            self.graphics.console_pic = self.graphics.load_block(fpath, 0, 0, null_mut(), 0);
            let fpath = self.find_file(
                CONSOLE_BG_PIC1_FILE_C.as_ptr() as *mut c_char,
                GRAPHICS_DIR_C.as_ptr() as *mut c_char,
                Themed::NoTheme as c_int,
                Criticality::Critical as c_int,
            );
            self.graphics.console_bg_pic1 = self.graphics.load_block(fpath, 0, 0, null_mut(), 0);
            let fpath = self.find_file(
                CONSOLE_BG_PIC2_FILE_C.as_ptr() as *mut c_char,
                GRAPHICS_DIR_C.as_ptr() as *mut c_char,
                Themed::NoTheme as c_int,
                Criticality::Critical as c_int,
            );
            self.graphics.console_bg_pic2 = self.graphics.load_block(fpath, 0, 0, null_mut(), 0);

            self.update_progress(80);

            self.graphics.arrow_up = Some(Surface::from_ptr(
                NonNull::new(IMG_Load(self.find_file(
                    cstr!("arrow_up.png").as_ptr() as *mut c_char,
                    GRAPHICS_DIR_C.as_ptr() as *mut c_char,
                    Themed::NoTheme as c_int,
                    Criticality::Critical as c_int,
                )))
                .unwrap(),
            ));
            self.graphics.arrow_down = Some(Surface::from_ptr(
                NonNull::new(IMG_Load(self.find_file(
                    cstr!("arrow_down.png").as_ptr() as *mut c_char,
                    GRAPHICS_DIR_C.as_ptr() as *mut c_char,
                    Themed::NoTheme as c_int,
                    Criticality::Critical as c_int,
                )))
                .unwrap(),
            ));
            self.graphics.arrow_right = Some(Surface::from_ptr(
                NonNull::new(IMG_Load(self.find_file(
                    cstr!("arrow_right.png").as_ptr() as *mut c_char,
                    GRAPHICS_DIR_C.as_ptr() as *mut c_char,
                    Themed::NoTheme as c_int,
                    Criticality::Critical as c_int,
                )))
                .unwrap(),
            ));
            self.graphics.arrow_left = Some(Surface::from_ptr(
                NonNull::new(IMG_Load(self.find_file(
                    cstr!("arrow_left.png").as_ptr() as *mut c_char,
                    GRAPHICS_DIR_C.as_ptr() as *mut c_char,
                    Themed::NoTheme as c_int,
                    Criticality::Critical as c_int,
                )))
                .unwrap(),
            ));
            //---------- get Banner
            let fpath = self.find_file(
                BANNER_BLOCK_FILE_C.as_ptr() as *mut c_char,
                GRAPHICS_DIR_C.as_ptr() as *mut c_char,
                Themed::NoTheme as c_int,
                Criticality::Critical as c_int,
            );
            self.graphics.banner_pic = self.graphics.load_block(fpath, 0, 0, null_mut(), 0);

            self.update_progress(90);
            //---------- get Droid images ----------
            let droids = std::slice::from_raw_parts(self.vars.droidmap, Droid::NumDroids as usize);
            let Self {
                graphics,
                global,
                misc,
                ..
            } = self;
            droids
                .iter()
                .zip(graphics.packed_portraits.iter_mut())
                .zip(graphics.portrait_raw_mem.iter_mut())
                .for_each(|((droid, packed_portrait), raw_portrait)| {
                    // first check if we find a file with rotation-frames: first try .jpg
                    libc::strcpy(fname.as_mut_ptr(), droid.druidname.as_ptr());
                    libc::strcat(fname.as_mut_ptr(), cstr!(".jpg").as_ptr());
                    let mut fpath = Self::find_file_static(
                        global,
                        misc,
                        fname.as_mut_ptr(),
                        GRAPHICS_DIR_C.as_ptr() as *mut c_char,
                        Themed::NoTheme as c_int,
                        Criticality::Ignore as c_int,
                    );
                    // then try with .png
                    if fpath.is_null() {
                        libc::strcpy(fname.as_mut_ptr(), droid.druidname.as_ptr());
                        libc::strcat(fname.as_mut_ptr(), cstr!(".png").as_ptr());
                        fpath = Self::find_file_static(
                            global,
                            misc,
                            fname.as_mut_ptr(),
                            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
                            Themed::NoTheme as c_int,
                            Criticality::Critical as c_int,
                        );
                    }

                    *packed_portrait = Self::load_raw_pic(fpath, raw_portrait);
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
            self.graphics.pic999 = self.graphics.load_block(fpath, 0, 0, null_mut(), 0);

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
                self.graphics
                    .load_block(fpath, 0, 0, null_mut(), INIT_ONLY as c_int);
                self.graphics.decal_pics[0] =
                    self.graphics
                        .load_block(null_mut(), 0, 0, &ORIG_BLOCK_RECT, 0);
                self.graphics.decal_pics[1] =
                    self.graphics
                        .load_block(null_mut(), 0, 1, &ORIG_BLOCK_RECT, 0);
            }
        });

        self.update_progress(96);
        // if scale != 1 then we need to rescale everything now
        self.scale_graphics(self.global.game_config.scale);

        self.update_progress(98);

        // make sure bullet-surfaces get re-generated!
        self.main
            .all_bullets
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

    let last_line = std::str::from_utf8(image[4 + 32]).unwrap();
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
            &mut self.vars.blastmap[0].phases as *mut c_int as *mut c_void,
        );

        read_value_from_string(
            data.as_ptr() as *mut c_char,
            BLAST_TWO_NUMBER_OF_PHASES_STRING.as_ptr() as *mut c_char,
            cstr!("%d").as_ptr() as *mut c_char,
            &mut self.vars.blastmap[1].phases as *mut c_int as *mut c_void,
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
            if bullet_index >= self.graphics.number_of_bullet_types {
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
                &mut (*self.vars.bulletmap.offset(bullet_index.try_into().unwrap())).phases
                    as *mut c_int as *mut c_void,
            );
            read_value_from_string(
                read.as_ptr() as *mut c_char,
                cstr!("and number of phase changes per second=").as_ptr() as *mut c_char,
                cstr!("%f").as_ptr() as *mut c_char,
                &mut (*self.vars.bulletmap.offset(bullet_index.try_into().unwrap()))
                    .phase_changes_per_second as *mut c_float as *mut c_void,
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
            &mut self.main.first_digit_rect.x as *mut c_short as *mut c_void,
        );
        read_value_from_string(
            data.as_ptr() as *mut c_char,
            DIGIT_ONE_POSITION_Y_STRING.as_ptr() as *mut c_char,
            cstr!("%hd").as_ptr() as *mut c_char,
            &mut self.main.first_digit_rect.y as *mut c_short as *mut c_void,
        );

        read_value_from_string(
            data.as_ptr() as *mut c_char,
            DIGIT_TWO_POSITION_X_STRING.as_ptr() as *mut c_char,
            cstr!("%hd").as_ptr() as *mut c_char,
            &mut self.main.second_digit_rect.x as *mut c_short as *mut c_void,
        );
        read_value_from_string(
            data.as_ptr() as *mut c_char,
            DIGIT_TWO_POSITION_Y_STRING.as_ptr() as *mut c_char,
            cstr!("%hd").as_ptr() as *mut c_char,
            &mut self.main.second_digit_rect.y as *mut c_short as *mut c_void,
        );

        read_value_from_string(
            data.as_ptr() as *mut c_char,
            DIGIT_THREE_POSITION_X_STRING.as_ptr() as *mut c_char,
            cstr!("%hd").as_ptr() as *mut c_char,
            &mut self.main.third_digit_rect.x as *mut i16 as *mut c_void,
        );
        read_value_from_string(
            data.as_ptr() as *mut c_char,
            DIGIT_THREE_POSITION_Y_STRING.as_ptr() as *mut c_char,
            cstr!("%hd").as_ptr() as *mut c_char,
            &mut self.main.third_digit_rect.y as *mut c_short as *mut c_void,
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

        self.graphics
            .map_block_surface_pointer
            .iter_mut()
            .zip(self.graphics.orig_map_block_surface_pointer.iter_mut())
            .flat_map(|(map_block, orig_map_block)| {
                map_block.iter_mut().zip(orig_map_block.iter_mut())
            })
            .for_each(|(surface, orig_surface)| {
                let mut orig_surface = orig_surface.as_mut().unwrap().borrow_mut();
                let tmp = zoomSurface(orig_surface.as_mut_ptr(), scale.into(), scale.into(), 0);
                if tmp.is_null() {
                    panic!("zoomSurface() failed for scale = {}.", scale);
                }
                // and optimize
                *surface = Some(Rc::new(RefCell::new(Surface::from_ptr(
                    NonNull::new(SDL_DisplayFormat(tmp)).unwrap(),
                ))));
                SDL_FreeSurface(tmp); // free the old surface
            });

        static ORIG_BLOCK: OnceCell<SDL_Rect> = OnceCell::new();
        let orig_block = ORIG_BLOCK.get_or_init(|| self.vars.block_rect);

        self.vars.block_rect = *orig_block;
        scale_rect(&mut self.vars.block_rect, scale);
    }

    /// This function load an image and displays it directly to the self.graphics.ne_screen
    /// but without updating it.
    /// This might be very handy, especially in the Title() function to
    /// display the title image and perhaps also for displaying the ship
    /// and that.
    pub unsafe fn display_image(&mut self, datafile: *mut c_char) {
        let mut image = Surface::from_ptr(NonNull::new(IMG_Load(datafile)).unwrap_or_else(|| {
            panic!(
                "couldn't load image {}: {}",
                CStr::from_ptr(datafile).to_string_lossy(),
                CStr::from_ptr(SDL_GetError()).to_string_lossy()
            )
        }));

        if (self.global.game_config.scale - 1.).abs() > c_float::EPSILON {
            scale_pic(&mut image, self.global.game_config.scale);
        }

        SDL_UpperBlit(
            image.as_mut_ptr(),
            null_mut(),
            self.graphics.ne_screen.as_mut().unwrap().as_mut_ptr(),
            null_mut(),
        );
    }

    pub unsafe fn draw_line_between_tiles(
        &mut self,
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
                    - f32::from(self.vars.block_rect.w) * (self.vars.me.pos.x - x1);
                let user_center = self.vars.get_user_center();
                let pixy = f32::from(user_center.y)
                    - f32::from(self.vars.block_rect.h) * (self.vars.me.pos.y - y1)
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
                let ne_screen = self.graphics.ne_screen.as_mut().unwrap();
                let mut ne_screen = ne_screen.lock().unwrap();
                ne_screen
                    .pixels()
                    .set(pixx as u16, pixy as u16, color.try_into().unwrap())
                    .unwrap();
                ne_screen
                    .pixels()
                    .set(pixx as u16 - 1, pixy as u16, color.try_into().unwrap())
                    .unwrap();

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
                - f32::from(self.vars.block_rect.w) * (self.vars.me.pos.x - x1)
                + i;
            let user_center = self.vars.get_user_center();
            let pixy = f32::from(user_center.y)
                - f32::from(self.vars.block_rect.h) * (self.vars.me.pos.y - y1)
                + i * slope;
            if pixx <= f32::from(self.vars.user_rect.x)
                || pixx >= f32::from(self.vars.user_rect.x) + f32::from(self.vars.user_rect.w) - 1.
                || pixy <= f32::from(self.vars.user_rect.y)
                || pixy >= f32::from(self.vars.user_rect.y) + f32::from(self.vars.user_rect.h) - 1.
            {
                i += 1.;
                continue;
            }
            let ne_screen = self.graphics.ne_screen.as_mut().unwrap();
            let mut ne_screen = ne_screen.lock().unwrap();
            ne_screen
                .pixels()
                .set(pixx as u16, pixy as u16, color.try_into().unwrap())
                .unwrap();
            ne_screen
                .pixels()
                .set(pixx as u16, pixy as u16 - 1, color.try_into().unwrap())
                .unwrap();
            i += 1.;
        }
    }
}
