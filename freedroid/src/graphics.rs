#[cfg(not(target_os = "android"))]
use crate::vars::Vars;
use crate::{
    defs::{
        self, scale_point, Cmds, Criticality, DisplayBannerFlags, Droid, SoundType, Themed,
        BANNER_BLOCK_FILE, BLAST_BLOCK_FILE, BULLET_BLOCK_FILE, CONSOLE_BG_PIC1_FILE,
        CONSOLE_BG_PIC2_FILE, CONSOLE_PIC_FILE, DIGITNUMBER, DIGIT_BLOCK_FILE, DROID_BLOCK_FILE,
        ENEMYPHASES, FONT0_FILE, FONT1_FILE, FONT2_FILE, FREE_ONLY, GRAPHICS_DIR_C, ICON_FILE,
        INIT_ONLY, MAP_BLOCK_FILE, MAXBULLETS, NUM_COLORS, NUM_DECAL_PICS, NUM_MAP_BLOCKS,
        PARA_FONT_FILE, SHIP_OFF_PIC_FILE, SHIP_ON_PIC_FILE, TAKEOVER_BG_PIC_FILE,
    },
    global::Global,
    misc::{read_float_from_string, read_i16_from_string, read_i32_from_string},
    read_and_malloc_and_terminate_file,
    structs::ThemeList,
    takeover::TO_BLOCK_FILE,
    vars::{ORIG_BLOCK_RECT, ORIG_DIGIT_RECT},
    Sdl,
};

use array_init::array_init;
use cstr::cstr;
use log::{error, info, trace, warn};
use once_cell::sync::Lazy;
use sdl::{
    cursor,
    rwops::{self, RwOps},
    ColorKeyFlag, Cursor, FrameBuffer, Pixel, Rect, Rgba, RwOpsOwned, Surface, VideoModeFlags,
};
use std::{
    borrow::Cow,
    cell::RefCell,
    ffi::{CStr, CString},
    ops::Not,
    path::Path,
    pin::Pin,
    rc::Rc,
};
use tinyvec_string::ArrayString;

#[derive(Debug)]
pub struct Graphics<'sdl> {
    pub vid_bpp: i32,
    fonts_loaded: i32,
    // A pointer to the surfaces containing the map-pics, which may be rescaled with respect to
    pub map_block_surface_pointer:
        [[Option<Rc<RefCell<Surface<'sdl>>>>; NUM_MAP_BLOCKS]; NUM_COLORS],
    // A pointer to the surfaces containing the original map-pics as read from disk
    orig_map_block_surface_pointer:
        [[Option<Rc<RefCell<Surface<'sdl>>>>; NUM_MAP_BLOCKS]; NUM_COLORS],
    // a block for temporary pic-construction
    pub build_block: Option<Surface<'sdl>>,
    pub banner_is_destroyed: i32,
    /* the banner pic */
    pub banner_pic: Option<Surface<'sdl>>,
    pub pic999: Option<Surface<'sdl>>,
    pub packed_portraits: [Option<RwOpsOwned>; Droid::NumDroids as usize],
    pub decal_pics: [Option<Surface<'sdl>>; NUM_DECAL_PICS],
    pub takeover_bg_pic: Option<Surface<'sdl>>,
    pub console_pic: Option<Surface<'sdl>>,
    pub console_bg_pic1: Option<Surface<'sdl>>,
    pub console_bg_pic2: Option<Surface<'sdl>>,
    pub arrow_up: Option<Surface<'sdl>>,
    pub arrow_down: Option<Surface<'sdl>>,
    pub arrow_right: Option<Surface<'sdl>>,
    pub arrow_left: Option<Surface<'sdl>>,
    // Side-view of ship: lights off
    pub ship_off_pic: Option<Surface<'sdl>>,
    // Side-view of ship: lights on
    pub ship_on_pic: Option<Surface<'sdl>>,
    pub progress_meter_pic: Option<Surface<'sdl>>,
    pub progress_filler_pic: Option<Surface<'sdl>>,
    /* the graphics display */
    pub ne_screen: Option<sdl::FrameBuffer<'sdl>>,
    pub enemy_surface_pointer: [Option<Surface<'sdl>>; ENEMYPHASES as usize],
    pub influencer_surface_pointer: [Option<Surface<'sdl>>; ENEMYPHASES as usize],
    pub influ_digit_surface_pointer: [Option<Surface<'sdl>>; DIGITNUMBER],
    pub enemy_digit_surface_pointer: [Option<Surface<'sdl>>; DIGITNUMBER],
    pub crosshair_cursor: Option<Cursor<'sdl, 'static>>,
    pub arrow_cursor: Option<Cursor<'sdl, 'static>>,
    pub number_of_bullet_types: i32,
    pub all_themes: ThemeList,
    pub classic_theme_index: i32,
    number_of_screenshot: u32,
    pic: Option<Surface<'sdl>>,
}

impl Default for Graphics<'_> {
    fn default() -> Self {
        Self {
            vid_bpp: 0,
            fonts_loaded: 0,
            map_block_surface_pointer: array_init(|_| array_init(|_| None)),
            orig_map_block_surface_pointer: array_init(|_| array_init(|_| None)),
            build_block: None,
            banner_is_destroyed: 0,
            banner_pic: None,
            pic999: None,
            packed_portraits: array_init(|_| None),
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
            crosshair_cursor: None,
            arrow_cursor: None,
            number_of_bullet_types: 0,
            all_themes: ThemeList {
                num_themes: 0,
                cur_tnum: 0,
                theme_name: array_init(|_| CString::default()),
            },
            classic_theme_index: 0,
            number_of_screenshot: 0,
            pic: None,
        }
    }
}

pub fn apply_filter(surface: &mut Surface, fred: f32, fgreen: f32, fblue: f32) -> i32 {
    let w = surface.width();
    (0..surface.height())
        .flat_map(move |y| (0..w).map(move |x| (x, y)))
        .for_each(|(x, y)| {
            let [mut pixel_red, mut pixel_green, mut pixel_blue, alpha] =
                surface.lock().unwrap().pixels().get(x, y).unwrap().rgba();
            if alpha == 0 {
                return;
            }

            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            {
                pixel_red = (f32::from(pixel_red) * fred) as u8;
                pixel_green = (f32::from(pixel_green) * fgreen) as u8;
                pixel_blue = (f32::from(pixel_blue) * fblue) as u8;
            }

            let pixel_value = surface
                .format()
                .map_rgba(pixel_red, pixel_green, pixel_blue, alpha);
            let mut surface = surface.lock().unwrap();
            surface.pixels().set(x, y, pixel_value).unwrap();
        });

    defs::OK.into()
}

impl<'sdl> Graphics<'sdl> {
    /// General block-reading routine: get block from pic-file
    ///
    /// fpath: full pathname of picture-file; if NULL: use previous SDL-surf
    /// line, col: block-position in pic-file to read block from
    /// block: dimension of blocks to consider: if NULL: copy whole pic
    /// NOTE: only w and h of block are used!!
    ///
    /// NOTE: to avoid memory-leaks, use (flags | `INIT_ONLY`) if you only
    ///       call this function to set up a new pic-file to be read.
    ///       This will avoid copying & mallocing a new pic, NULL will be returned
    pub fn load_block(
        &mut self,
        fpath: Option<&CStr>,
        line: i32,
        col: i32,
        block: Option<Rect>,
        flags: i32,
        sdl: &'sdl Sdl,
    ) -> Option<Surface<'sdl>> {
        let &mut Self {
            vid_bpp,
            ref mut pic,
            ..
        } = self;

        LoadBlockVidBppPic {
            vid_bpp,
            pic,
            fpath,
            line,
            col,
            block,
            flags,
            sdl,
        }
        .run()
    }
}

pub fn scale_pic(pic: &mut Surface, scale: f32) {
    if (scale - 1.0).abs() <= f32::EPSILON {
        return;
    }
    let scale = scale.into();

    *pic = pic
        .zoom(scale, scale, false)
        .unwrap_or_else(|| panic!("surface.zoom() failed for scale = {scale}."));
}

static CROSSHAIR_CURSOR: Lazy<cursor::Data<32>> = Lazy::new(|| {
    const XPM: [[u8; 32]; 32] = [
        *b"                                ",
        *b"                                ",
        *b"               XXXX             ",
        *b"               X..X             ",
        *b"               X..X             ",
        *b"               X..X             ",
        *b"               X..X             ",
        *b"               X..X             ",
        *b"               X..X             ",
        *b"               X..X             ",
        *b"               X..X             ",
        *b"               XXXX             ",
        *b"                                ",
        *b"   XXXXXXXXXXX      XXXXXXXXXX  ",
        *b"   X.........X      X........X  ",
        *b"   X.........X      X........X  ",
        *b"   XXXXXXXXXXX      XXXXXXXXXX  ",
        *b"                                ",
        *b"               XXXX             ",
        *b"               X..X             ",
        *b"               X..X             ",
        *b"               X..X             ",
        *b"               X..X             ",
        *b"               X..X             ",
        *b"               X..X             ",
        *b"               X..X             ",
        *b"               X..X             ",
        *b"               X..X             ",
        *b"               X..X             ",
        *b"               XXXX             ",
        *b"                                ",
        *b"                                ",
    ];

    cursor::Data::from_draw(&XPM)
});

static ARROW_CURSOR: Lazy<cursor::Data<32>> = Lazy::new(|| {
    const XPM: [[u8; 32]; 32] = [
        *b"X                               ",
        *b"XX                              ",
        *b"X.X                             ",
        *b"X..X                            ",
        *b"X...X                           ",
        *b"X....X                          ",
        *b"X.....X                         ",
        *b"X......X                        ",
        *b"X.......X                       ",
        *b"X........X                      ",
        *b"X.....XXXXX                     ",
        *b"X..X..X                         ",
        *b"X.X X..X                        ",
        *b"XX  X..X                        ",
        *b"X    X..X                       ",
        *b"     X..X                       ",
        *b"      X..X                      ",
        *b"      X..X                      ",
        *b"       XX                       ",
        *b"                                ",
        *b"                                ",
        *b"                                ",
        *b"                                ",
        *b"                                ",
        *b"                                ",
        *b"                                ",
        *b"                                ",
        *b"                                ",
        *b"                                ",
        *b"                                ",
        *b"                                ",
        *b"                                ",
    ];

    cursor::Data::from_draw(&XPM)
});

impl crate::Data<'_> {
    /// This function draws a "grid" on the screen, that means every
    /// "second" pixel is blacked out, thereby generation a fading
    /// effect.  This function was created to fade the background of the
    /// Escape menu and its submenus.
    pub fn make_grid_on_screen(&mut self, grid_rectangle: Option<&Rect>) {
        let grid_rectangle = grid_rectangle.unwrap_or(&self.vars.user_rect);

        trace!("MakeGridOnScreen(...): real function call confirmed.");
        let ne_screen = self.graphics.ne_screen.as_mut().unwrap();
        let rect_x = u16::try_from(grid_rectangle.x()).unwrap();
        let rect_y = u16::try_from(grid_rectangle.y()).unwrap();
        let mut ne_screen = ne_screen.lock().unwrap();
        (rect_y..(rect_y + grid_rectangle.height()))
            .flat_map(|y| (rect_x..(rect_x + grid_rectangle.width())).map(move |x| (x, y)))
            .filter(|(x, y)| (x + y) % 2 == 0)
            .for_each(|(x, y)| ne_screen.pixels().set(x, y, Pixel::black()).unwrap());
        trace!("MakeGridOnScreen(...): end of function reached.");
    }

    pub fn toggle_fullscreen(&mut self) {
        let ne_screen = self.graphics.ne_screen.take().unwrap();
        let mut vid_flags = VideoModeFlags::from_bits(ne_screen.flags()).unwrap();

        vid_flags.set(
            VideoModeFlags::FULLSCREEN,
            self.global.game_config.use_fullscreen == 0,
        );

        drop(ne_screen);
        let Some(new_ne_screen) = self.sdl.video.set_video_mode(
            self.vars.screen_rect.width().into(),
            self.vars.screen_rect.height().into(),
            None,
            vid_flags,
        ) else {
            error!(
                "unable to toggle windowed/fullscreen {} x {} video mode.",
                self.vars.screen_rect.width(),
                self.vars.screen_rect.height(),
            );
            panic!("SDL-Error: {}", self.sdl.get_error().to_string_lossy());
        };

        if new_ne_screen.flags() == vid_flags.bits() {
            self.global.game_config.use_fullscreen = !self.global.game_config.use_fullscreen;
        } else {
            warn!("Failed to toggle windowed/fullscreen mode!");
        }

        self.graphics.ne_screen = Some(new_ne_screen);
    }

    /// This function saves a screenshot to disk.
    ///
    /// The screenshots are names `Screenshot_XX.bmp` where XX is a
    /// running number.
    ///
    /// NOTE:  This function does NOT check for existing screenshots,
    ///        but will silently overwrite them.  No problem in most
    ///        cases I think.
    pub fn take_screenshot(&mut self) {
        use rwops::{Mode, ReadWriteMode};

        self.activate_conservative_frame_computation();

        let screenshot_filename =
            format!("Screenshot_{}.bmp", self.graphics.number_of_screenshot).into();
        let Some(mut rw_ops) = RwOps::from_pathbuf(
            screenshot_filename,
            Mode::from(ReadWriteMode::Write).with_binary(true),
        ) else {
            error!("Unable to take screenshot, cannot write to file.");
            return;
        };
        if self
            .graphics
            .ne_screen
            .as_mut()
            .unwrap()
            .save_bmp_rw(&mut rw_ops)
            .not()
        {
            error!("Unable to take screenshot, cannot write to file.");
            return;
        }
        drop(rw_ops);
        self.graphics.number_of_screenshot = self.graphics.number_of_screenshot.wrapping_add(1);
        self.display_banner(
            Some(cstr!("Screenshot")),
            None,
            (DisplayBannerFlags::NO_SDL_UPDATE | DisplayBannerFlags::FORCE_UPDATE)
                .bits()
                .into(),
        );
        self.make_grid_on_screen(None);
        assert!(self.graphics.ne_screen.as_mut().unwrap().flip());
        self.play_sound(SoundType::Screenshot);

        while self.cmd_is_active(Cmds::Screenshot) {
            self.sdl.delay_ms(1);
        }

        self.display_banner(None, None, DisplayBannerFlags::FORCE_UPDATE.bits().into());
    }

    pub fn free_graphics(&mut self) {
        // free RWops structures
        self.graphics.packed_portraits.fill_with(|| None);

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
        self.global.menu_b_font = None;
        self.global.para_b_font = None;
        self.global.highscore_b_font = None;
        self.global.font0_b_font = None;
        self.global.font1_b_font = None;
        self.global.font2_b_font = None;

        // free Load_Block()-internal buffer
        self.graphics
            .load_block(None, 0, 0, None, i32::from(FREE_ONLY), self.sdl);

        // free cursors
        self.graphics.crosshair_cursor = None;
        self.graphics.arrow_cursor = None;
    }

    /// scale all "static" rectangles, which are theme-independent
    pub fn scale_stat_rects(&mut self, scale: f32) {
        macro_rules! scale {
            ($rect:expr) => {
                $rect.scale(scale);
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
            block.scale(scale);
        }

        scale!(self.vars.cons_menu_item_rect);

        scale!(self.vars.left_info_rect);
        scale!(self.vars.right_info_rect);

        for block in &mut self.takeover.fill_blocks {
            block.scale(scale);
        }

        for block in &mut self.takeover.capsule_blocks {
            block.scale(scale);
        }

        for block in &mut self.takeover.to_game_blocks {
            block.scale(scale);
        }

        for block in &mut self.takeover.to_ground_blocks {
            block.scale(scale);
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

    pub fn scale_graphics(&mut self, scale: f32) {
        static INIT: std::sync::Once = std::sync::Once::new();

        /* For some reason we need to SetAlpha every time on OS X */
        /* Digits are only used in _internal_ blits ==> clear per-surf alpha */
        self.graphics
            .influ_digit_surface_pointer
            .iter_mut()
            .chain(&mut self.graphics.enemy_digit_surface_pointer)
            .flatten()
            .for_each(|surface| {
                if surface.set_alpha(ColorKeyFlag::empty(), 0).not() {
                    error!("Cannot set alpha channel on surface");
                }
            });

        if (scale - 1.).abs() <= f32::EPSILON {
            return;
        }

        // these are reset in a theme-change by the theme-config-file
        // therefore we need to rescale them each time again
        self.main.first_digit_rect.scale(scale);
        self.main.second_digit_rect.scale(scale);
        self.main.third_digit_rect.scale(scale);

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
                scale_pic(&mut orig_surface.borrow_mut(), scale);
                *map_surface = Some(Rc::clone(orig_surface));
            });

        //---------- rescale Droid-model  blocks
        /* Droid pics are only used in _internal_ blits ==> clear per-surf alpha */
        self.graphics
            .influencer_surface_pointer
            .iter_mut()
            .chain(&mut self.graphics.enemy_surface_pointer)
            .for_each(|surface| {
                let surface = surface.as_mut().unwrap();
                scale_pic(surface, scale);
                if surface.set_alpha(ColorKeyFlag::empty(), 0).not() {
                    error!("Cannot set alpha channel on surface");
                }
            });

        //---------- rescale Bullet and Blast blocks
        let bulletmap = &mut self.vars.bulletmap;
        bulletmap
            .iter_mut()
            .flat_map(|bullet| bullet.surfaces.iter_mut())
            .chain(
                self.vars
                    .blastmap
                    .iter_mut()
                    .flat_map(|blast| blast.surfaces.iter_mut()),
            )
            .for_each(|surface| scale_pic(surface.as_mut().unwrap(), scale));

        //---------- rescale Digit blocks
        self.graphics
            .influ_digit_surface_pointer
            .iter_mut()
            .chain(&mut self.graphics.enemy_digit_surface_pointer)
            .for_each(|surface| {
                let surface = surface.as_mut().unwrap();
                scale_pic(surface, scale);
                if surface.set_alpha(ColorKeyFlag::empty(), 0).not() {
                    error!("Cannot set alpha channel on surface");
                }
            });

        //---------- rescale Takeover pics
        scale_pic(self.takeover.to_blocks.as_mut().unwrap(), scale);
        scale_pic(self.graphics.ship_on_pic.as_mut().unwrap(), scale);
        scale_pic(self.graphics.ship_off_pic.as_mut().unwrap(), scale);

        // the following are not theme-specific and are therefore only loaded once!
        if init {
            //  create a new tmp block-build storage
            let build_block = Surface::create_rgb(
                self.vars.block_rect.width().into(),
                self.vars.block_rect.height().into(),
                self.graphics.vid_bpp.max(0).try_into().unwrap_or(u8::MAX),
                Rgba::default(),
            )
            .unwrap()
            .display_format_alpha()
            .unwrap();
            self.graphics.build_block = Some(build_block);

            // takeover pics
            scale_pic(self.graphics.takeover_bg_pic.as_mut().unwrap(), scale);

            //---------- Console pictures
            self.scale_console_pictures(scale);
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

    fn scale_console_pictures(&mut self, scale: f32) {
        scale_pic(self.graphics.console_pic.as_mut().unwrap(), scale);
        scale_pic(self.graphics.console_bg_pic1.as_mut().unwrap(), scale);
        scale_pic(self.graphics.console_bg_pic2.as_mut().unwrap(), scale);
        scale_pic(self.graphics.arrow_up.as_mut().unwrap(), scale);
        scale_pic(self.graphics.arrow_down.as_mut().unwrap(), scale);
        scale_pic(self.graphics.arrow_right.as_mut().unwrap(), scale);
        scale_pic(self.graphics.arrow_left.as_mut().unwrap(), scale);
    }

    /// display "white noise" effect in Rect.
    /// algorith basically stolen from
    /// Greg Knauss's "xteevee" hack in xscreensavers.
    ///
    /// timeout is in ms
    pub fn white_noise(&mut self, frame_buffer: &mut FrameBuffer, rect: &mut Rect, timeout: i32) {
        use rand::{
            seq::{IteratorRandom, SliceRandom},
            Rng,
        };
        const NOISE_COLORS: u8 = 6;
        const NOISE_TILES: u8 = 8;

        let signal_strengh = 60;

        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_sign_loss,
            clippy::cast_possible_truncation
        )]
        let grey: [Pixel; NOISE_COLORS as usize] = array_init(|index| {
            let color = (((index as f64 + 1.0) / f64::from(NOISE_COLORS)) * 255.0) as u8;
            frame_buffer.format().map_rgb(color, color, color)
        });

        // produce the tiles
        //
        let mut tmp = Surface::create_rgb(
            rect.width().into(),
            rect.height().into(),
            self.graphics.vid_bpp.max(0).try_into().unwrap_or(u8::MAX),
            Rgba::default(),
        )
        .unwrap()
        .display_format()
        .unwrap();
        frame_buffer.blit_from(&*rect, &mut tmp);

        let mut rng = rand::thread_rng();
        let mut noise_tiles: [Surface; NOISE_TILES as usize] = array_init(|_| {
            let mut tile = tmp.display_format().unwrap();
            let mut lock = tile.lock().unwrap();
            (0..rect.width())
                .flat_map(|x| (0..rect.height()).map(move |y| (x, y)))
                .for_each(|(x, y)| {
                    if rng.gen_range(0i32..100) > signal_strengh {
                        lock.pixels()
                            .set(x, y, *grey.choose(&mut rng).unwrap())
                            .unwrap();
                    }
                });
            drop(lock);
            tile
        });
        drop(tmp);

        let mut used_tiles: [i8; NOISE_TILES as usize / 2 + 1] = [-1; NOISE_TILES as usize / 2 + 1];
        // let's go
        self.play_sound(SoundType::WhiteNoise);

        let now = self.sdl.ticks_ms();

        self.wait_for_all_keys_released();
        let clip_rect = loop {
            // pick an old enough tile
            let mut next_tile;
            loop {
                next_tile = (0..i8::try_from(NOISE_TILES).unwrap())
                    .choose(&mut rng)
                    .unwrap();
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
            let clip_rect = frame_buffer.get_clip_rect();
            frame_buffer.clear_clip_rect();
            // set it
            noise_tiles[usize::try_from(next_tile).unwrap()].blit_to(frame_buffer, &mut *rect);
            frame_buffer.update_rect(rect);
            self.sdl.delay_ms(25);

            if timeout != 0 && self.sdl.ticks_ms() - now > timeout.try_into().unwrap() {
                break clip_rect;
            }

            if self.any_key_just_pressed() != 0 {
                break clip_rect;
            }
        };

        //restore previous clip-rectange
        frame_buffer.set_clip_rect(&clip_rect);
    }

    pub fn load_fonts(&mut self) -> i32 {
        let Self {
            global,
            sdl,
            b_font,
            misc,
            ..
        } = self;

        let mut fpath = crate::Data::find_file_static(
            global,
            misc,
            PARA_FONT_FILE.as_bytes(),
            Some(GRAPHICS_DIR_C),
            Themed::NoTheme as i32,
            Criticality::Critical as i32,
        )
        .unwrap_or_else(|| panic!("font file named {PARA_FONT_FILE} was not found."));

        global.para_b_font = Some(Self::load_font(
            sdl,
            b_font,
            fpath,
            global.game_config.scale,
        ));

        fpath = crate::Data::find_file_static(
            global,
            misc,
            FONT0_FILE.as_bytes(),
            Some(GRAPHICS_DIR_C),
            Themed::NoTheme as i32,
            Criticality::Critical as i32,
        )
        .unwrap_or_else(|| panic!("font file named {FONT0_FILE} was not found."));
        global.font0_b_font = Some(Self::load_font(
            sdl,
            b_font,
            fpath,
            global.game_config.scale,
        ));

        fpath = Self::find_file_static(
            global,
            misc,
            FONT1_FILE.as_bytes(),
            Some(GRAPHICS_DIR_C),
            Themed::NoTheme as i32,
            Criticality::Critical as i32,
        )
        .unwrap_or_else(|| panic!("font file named {FONT1_FILE} was not found."));
        global.font1_b_font = Some(Self::load_font(
            sdl,
            b_font,
            fpath,
            global.game_config.scale,
        ));

        fpath = Self::find_file_static(
            global,
            misc,
            FONT2_FILE.as_bytes(),
            Some(GRAPHICS_DIR_C),
            Themed::NoTheme as i32,
            Criticality::Critical as i32,
        )
        .unwrap_or_else(|| panic!("font file named {FONT2_FILE} was not found."));
        global.font2_b_font = Some(Self::load_font(
            sdl,
            b_font,
            fpath,
            global.game_config.scale,
        ));

        global.menu_b_font = global.para_b_font.clone();
        global.highscore_b_font = global.para_b_font.clone();

        self.graphics.fonts_loaded = true.into();

        defs::OK.into()
    }

    pub fn clear_graph_mem(&mut self) {
        // One this function is done, the rahmen at the
        // top of the screen surely is destroyed.  We inform the
        // DisplayBanner function of the matter...
        self.graphics.banner_is_destroyed = true.into();

        let ne_screen = self.graphics.ne_screen.as_mut().unwrap();
        ne_screen.clear_clip_rect();

        // Now we fill the screen with black color...
        ne_screen.fill(Pixel::black()).unwrap();
        assert!(ne_screen.flip());
    }

    /// Initialise the Video display and graphics engine
    pub fn init_video(&mut self) {
        let vid_info = self
            .sdl
            .video
            .get_video_info()
            .expect("SDL_SetVideoMode should have been called");
        let mut vid_driver = [0; 81];
        let vid_driver = self.sdl.video.get_driver_name(&mut vid_driver);

        if cfg!(os_target = "android") {
            self.graphics.vid_bpp = 16; // Hardcoded Android default
        } else {
            self.graphics.vid_bpp = u8::from(vid_info.format().bits_per_pixel()).into();
        }

        print_init_info(&vid_info, self.graphics.vid_bpp, vid_driver);

        let vid_flags = if self.global.game_config.use_fullscreen == 0 {
            VideoModeFlags::empty()
        } else {
            VideoModeFlags::FULLSCREEN
        };

        if vid_info.wm_available() {
            let Self {
                sdl, global, misc, ..
            } = self;

            /* if there's a window-manager */
            sdl.video
                .window_manager()
                .set_caption(cstr!("Freedroid"), cstr!(""));
            let fpath = Self::find_file_static(
                global,
                misc,
                ICON_FILE.as_bytes(),
                Some(GRAPHICS_DIR_C),
                Themed::NoTheme as i32,
                Criticality::WarnOnly as i32,
            );

            if let Some(fpath) = fpath {
                match sdl.load_image_from_c_str_path(fpath) {
                    Some(mut img) => sdl.video.window_manager().set_icon(&mut img, None),
                    None => {
                        warn!(
                            "SDL load image failed for icon file '{}'\n",
                            fpath.to_string_lossy()
                        );
                    }
                }
            } else {
                warn!("Could not find icon file '{}'", ICON_FILE);
            }
        }

        let Some(ne_screen) = self.sdl.video.set_video_mode(
            self.vars.screen_rect.width().into(),
            self.vars.screen_rect.height().into(),
            None,
            vid_flags,
        ) else {
            error!(
                "Couldn't set {} x {} video mode. SDL: {}",
                self.vars.screen_rect.width(),
                self.vars.screen_rect.height(),
                self.sdl.get_error().to_string_lossy(),
            );
            std::process::exit(-1);
        };
        self.graphics.ne_screen = Some(ne_screen);

        info!("Got video mode: ");

        if self.sdl.video.set_gamma(1., 1., 1.).not() {
            error!("Unable to set SDL gamma");
        };
        self.global.game_config.current_gamma_correction = 1.;
    }

    /// load a pic into memory and return the `SDL_RWops` pointer to it
    pub fn load_raw_pic(fpath: &CStr) -> Option<RwOpsOwned> {
        use std::{fs::File, io::Read};
        let fpath = match fpath.to_str() {
            Ok(fpath) => fpath,
            Err(err) => {
                panic!("unable to convert path with invalid UTF-8 data: {err}");
            }
        };
        let fpath = Path::new(&fpath);
        let Ok(mut file) = File::open(fpath) else {
            panic!("could not open file {}. Giving up", fpath.display());
        };

        let metadata = match file.metadata() {
            Ok(metadata) => metadata,
            Err(err) => {
                panic!("unable to get file metadata: {err}");
            }
        };

        let len = metadata.len().try_into().unwrap();
        let mut buf: Pin<Box<[u8]>> = vec![0; len].into_boxed_slice().into();
        assert!(
            file.read_exact(&mut buf).is_ok(),
            "cannot reading file {}. Giving up...",
            fpath.display()
        );
        drop(file);

        RwOpsOwned::from_buffer(buf).ok()
    }

    /// Get the pics for: druids, bullets, blasts
    ///
    /// reads all blocks and puts the right pointers into
    /// the various structs
    ///
    /// Returns true/false
    pub fn init_pictures(&mut self) -> i32 {
        use std::sync::Once;

        static DO_ONCE: Once = Once::new();

        macro_rules! find_file {
            ($file_name:expr) => {
                Self::find_file_static(
                    &self.global,
                    &mut self.misc,
                    $file_name,
                    Some(GRAPHICS_DIR_C),
                    Themed::UseTheme as i32,
                    Criticality::Critical as i32,
                )
            };
        }

        macro_rules! load_block_from_file {
            ($file_name:expr) => {{
                let fpath = find_file!($file_name);
                self.graphics
                    .load_block(fpath, 0, 0, None, i32::from(INIT_ONLY), self.sdl);
            }};
        }

        // Loading all these pictures might take a while...
        // and we do not want do deal with huge frametimes, which
        // could box the influencer out of the ship....
        self.activate_conservative_frame_computation();

        let oldfont = self.b_font.current_font.take();

        if self.graphics.fonts_loaded == 0 {
            self.load_fonts();
        }

        self.b_font.current_font = self.global.font0_b_font.clone();

        self.init_progress("Loading pictures");

        self.load_theme_configuration_file();

        self.update_progress(15);

        //---------- get Map blocks
        load_block_from_file!(MAP_BLOCK_FILE);
        self.load_orig_map_block_surface_pointer();
        self.update_progress(20);
        //---------- get Droid-model  blocks
        load_block_from_file!(DROID_BLOCK_FILE);

        self.load_influencer_enemy_surface();

        self.update_progress(30);
        //---------- get Bullet blocks
        load_block_from_file!(BULLET_BLOCK_FILE);
        self.load_bullet_surfaces();

        self.update_progress(35);

        //---------- get Blast blocks
        load_block_from_file!(BLAST_BLOCK_FILE);

        self.load_blast_surfaces();
        self.update_progress(45);

        //---------- get Digit blocks
        load_block_from_file!(DIGIT_BLOCK_FILE);
        self.load_influencer_enemy_digit_surface();

        self.update_progress(50);

        //---------- get Takeover pics
        let fpath = find_file!(TO_BLOCK_FILE);
        self.takeover.to_blocks = self.graphics.load_block(fpath, 0, 0, None, 0, self.sdl);

        self.update_progress(60);

        let path = find_file!(SHIP_ON_PIC_FILE).unwrap();
        self.graphics.ship_on_pic = Some(self.sdl.load_image_from_c_str_path(path).unwrap());
        let path = find_file!(SHIP_OFF_PIC_FILE).unwrap();
        self.graphics.ship_off_pic = Some(self.sdl.load_image_from_c_str_path(path).unwrap());

        // the following are not theme-specific and are therefore only loaded once!
        DO_ONCE.call_once(|| self.init_pictures_once());

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

    fn init_pictures_once(&mut self) {
        macro_rules! find_file {
            ($file_name:expr, $criticality:expr) => {
                Self::find_file_static(
                    &self.global,
                    &mut self.misc,
                    $file_name,
                    Some(GRAPHICS_DIR_C),
                    Themed::NoTheme as i32,
                    $criticality as i32,
                )
            };

            ($file_name:expr) => {
                find_file!($file_name, Criticality::Critical)
            };
        }

        macro_rules! load_block_from_file {
            ($file_name:expr) => {{
                let fpath = find_file!($file_name);
                self.graphics.load_block(fpath, 0, 0, None, 0, self.sdl)
            }};
        }

        //  create the tmp block-build storage
        let build_block = Surface::create_rgb(
            self.vars.block_rect.width().into(),
            self.vars.block_rect.height().into(),
            self.graphics.vid_bpp.max(0).try_into().unwrap_or(u8::MAX),
            Rgba::default(),
        )
        .unwrap()
        .display_format_alpha()
        .unwrap();
        self.graphics.build_block = Some(build_block);

        // takeover background pics
        self.graphics.takeover_bg_pic = load_block_from_file!(TAKEOVER_BG_PIC_FILE);
        self.set_takeover_rects(); // setup takeover rectangles

        // cursor shapes
        self.graphics.arrow_cursor = Some(self.sdl.cursor().from_data(&ARROW_CURSOR).unwrap());
        self.graphics.crosshair_cursor =
            Some(self.sdl.cursor().from_data(&CROSSHAIR_CURSOR).unwrap());
        //---------- get Console pictures
        self.graphics.console_pic = load_block_from_file!(CONSOLE_PIC_FILE);
        self.graphics.console_bg_pic1 = load_block_from_file!(CONSOLE_BG_PIC1_FILE);
        self.graphics.console_bg_pic2 = load_block_from_file!(CONSOLE_BG_PIC2_FILE);

        self.update_progress(80);

        let path = find_file!(b"arrow_up.png").unwrap();
        self.graphics.arrow_up = Some(self.sdl.load_image_from_c_str_path(path).unwrap());

        let path = find_file!(b"arrow_down.png").unwrap();
        self.graphics.arrow_down = Some(self.sdl.load_image_from_c_str_path(path).unwrap());

        let path = find_file!(b"arrow_right.png").unwrap();
        self.graphics.arrow_right = Some(self.sdl.load_image_from_c_str_path(path).unwrap());

        let path = find_file!(b"arrow_left.png").unwrap();
        self.graphics.arrow_left = Some(self.sdl.load_image_from_c_str_path(path).unwrap());
        //---------- get Banner
        self.graphics.banner_pic = load_block_from_file!(BANNER_BLOCK_FILE);

        self.update_progress(90);
        //---------- get Droid images ----------
        let droids = &mut self.vars.droidmap;
        let mut fname = ArrayString::<[u8; 500]>::new();
        droids
            .iter()
            .zip(self.graphics.packed_portraits.iter_mut())
            .for_each(|(droid, packed_portrait)| {
                // first check if we find a file with rotation-frames: first try .jpg
                fname.clear();
                fname.push_str(droid.druidname.to_str().unwrap());
                fname.push_str(".jpg");
                let mut fpath = find_file!(fname.as_ref(), Criticality::Ignore);
                // then try with .png
                if fpath.is_none() {
                    fname.truncate(droid.druidname.len());
                    fname.push_str(".png");
                    fpath = find_file!(fname.as_ref());
                }

                let fpath = fpath.expect("unable to find droid imag");
                *packed_portrait = Self::load_raw_pic(fpath);
            });

        self.update_progress(95);
        let droids = &self.vars.droidmap;
        // we need the 999.png in any case for transparency!
        fname.clear();
        fname.push_str(droids[Droid::Droid999 as usize].druidname.to_str().unwrap());
        fname.push_str(".png");
        self.graphics.pic999 = load_block_from_file!(fname.as_ref());

        // get the Ashes pics
        let fpath = find_file!(b"Ashes.png", Criticality::WarnOnly);

        self.graphics
            .load_block(fpath, 0, 0, None, i32::from(INIT_ONLY), self.sdl);
        self.graphics.decal_pics[0] =
            self.graphics
                .load_block(None, 0, 0, Some(ORIG_BLOCK_RECT), 0, self.sdl);
        self.graphics.decal_pics[1] =
            self.graphics
                .load_block(None, 0, 1, Some(ORIG_BLOCK_RECT), 0, self.sdl);
    }

    pub fn load_theme_configuration_file(&mut self) {
        use bstr::ByteSlice;

        const END_OF_THEME_DATA_STRING: &[u8] = b"**** End of theme data section ****";
        const BLAST_ONE_NUMBER_OF_PHASES_STRING: &[u8] = b"How many phases in Blast one :";
        const BLAST_TWO_NUMBER_OF_PHASES_STRING: &[u8] = b"How many phases in Blast two :";
        const DIGIT_ONE_POSITION_X_STRING: &[u8] = b"First digit x :";
        const DIGIT_ONE_POSITION_Y_STRING: &[u8] = b"First digit y :";
        const DIGIT_TWO_POSITION_X_STRING: &[u8] = b"Second digit x :";
        const DIGIT_TWO_POSITION_Y_STRING: &[u8] = b"Second digit y :";
        const DIGIT_THREE_POSITION_X_STRING: &[u8] = b"Third digit x :";
        const DIGIT_THREE_POSITION_Y_STRING: &[u8] = b"Third digit y :";

        let fpath = Self::find_file_static(
            &self.global,
            &mut self.misc,
            b"config.theme",
            Some(GRAPHICS_DIR_C),
            Themed::UseTheme as i32,
            Criticality::Critical as i32,
        )
        .expect("Unable to read file config.theme");
        let fpath = Path::new(
            fpath
                .to_str()
                .expect("unable to convert C string to UTF-8 string"),
        );

        let data = read_and_malloc_and_terminate_file(fpath, END_OF_THEME_DATA_STRING);

        //--------------------
        // Now the file is read in entirely and
        // we can start to analyze its content,
        //

        self.vars.blastmap[0].phases =
            read_i32_from_string(&data, BLAST_ONE_NUMBER_OF_PHASES_STRING);

        self.vars.blastmap[1].phases =
            read_i32_from_string(&data, BLAST_TWO_NUMBER_OF_PHASES_STRING);

        // Next we read in the number of phases that are to be used for each bullet type
        let mut reader = &*data;
        while let Some(read_start) = reader.find(b"For Bullettype Nr.=") {
            let read = &reader[read_start..];
            let bullet_index = read_i32_from_string(read, b"For Bullettype Nr.=");
            assert!(
                bullet_index < self.graphics.number_of_bullet_types,
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
            self.vars.bulletmap[usize::try_from(bullet_index).unwrap()].phases =
                read_i32_from_string(read, b"we will use number of phases=");
            self.vars.bulletmap[usize::try_from(bullet_index).unwrap()].phase_changes_per_second =
                read_float_from_string(read, b"and number of phase changes per second=");
            reader = &reader[read_start + 1..];
        }

        // --------------------
        // Also decidable from the theme is where in the robot to
        // display the digits.  This must also be read from the configuration
        // file of the theme
        //

        self.main.first_digit_rect.as_mut().x =
            read_i16_from_string(&data, DIGIT_ONE_POSITION_X_STRING);
        self.main.first_digit_rect.as_mut().y =
            read_i16_from_string(&data, DIGIT_ONE_POSITION_Y_STRING);

        self.main.second_digit_rect.as_mut().x =
            read_i16_from_string(&data, DIGIT_TWO_POSITION_X_STRING);
        self.main.second_digit_rect.as_mut().y =
            read_i16_from_string(&data, DIGIT_TWO_POSITION_Y_STRING);

        self.main.third_digit_rect.as_mut().x =
            read_i16_from_string(&data, DIGIT_THREE_POSITION_X_STRING);
        self.main.third_digit_rect.as_mut().y =
            read_i16_from_string(&data, DIGIT_THREE_POSITION_Y_STRING);
    }

    /// This function resizes all blocks and structures involved in assembling
    /// the combat picture to a new scale.  The new scale is relative to the
    /// standard scale with means scale=1 is 64x64 tile size.
    ///
    /// in the first call we assume the `Block_Rect` to be the original game-size
    /// and store this value for future rescalings
    pub fn set_combat_scale_to(&mut self, scale: f32) {
        use once_cell::sync::OnceCell;

        static ORIG_BLOCK: OnceCell<Rect> = OnceCell::new();

        self.graphics
            .map_block_surface_pointer
            .iter_mut()
            .zip(self.graphics.orig_map_block_surface_pointer.iter_mut())
            .flat_map(|(map_block, orig_map_block)| {
                map_block.iter_mut().zip(orig_map_block.iter_mut())
            })
            .for_each(|(surface, orig_surface)| {
                let mut orig_surface = orig_surface.as_mut().unwrap().borrow_mut();
                let mut tmp = orig_surface
                    .zoom(scale.into(), scale.into(), false)
                    .unwrap_or_else(|| panic!("surface.zoom() failed for scale = {scale}."));
                // and optimize
                *surface = Some(Rc::new(RefCell::new(tmp.display_format().unwrap())));
            });

        let orig_block = ORIG_BLOCK.get_or_init(|| self.vars.block_rect);

        self.vars.block_rect = *orig_block;
        self.vars.block_rect.scale(scale);
    }

    /// This function load an image and displays it directly to the `self.graphics.ne_screen`
    /// but without updating it.
    /// This might be very handy, especially in the Title() function to
    /// display the title image and perhaps also for displaying the ship
    /// and that.
    pub fn display_image(sdl: &Sdl, global: &Global, graphics: &mut Graphics, datafile: &CStr) {
        let mut image = sdl.load_image_from_c_str_path(datafile).unwrap_or_else(|| {
            panic!(
                "couldn't load image {}: {}",
                datafile.to_string_lossy(),
                sdl.get_error().to_string_lossy(),
            )
        });

        if (global.game_config.scale - 1.).abs() > f32::EPSILON {
            scale_pic(&mut image, global.game_config.scale);
        }

        image.blit(graphics.ne_screen.as_mut().unwrap());
    }

    #[cfg(not(target_os = "android"))]
    #[inline]
    pub fn draw_line_between_tiles(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: Pixel) {
        Self::draw_line_between_tiles_static(&self.vars, &mut self.graphics, x1, y1, x2, y2, color);
    }

    #[cfg(not(target_os = "android"))]
    pub fn draw_line_between_tiles_static(
        vars: &Vars,
        graphics: &mut Graphics,
        mut x1: f32,
        mut y1: f32,
        mut x2: f32,
        mut y2: f32,
        color: Pixel,
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
            let max = (y2 - y1) * f32::from(vars.block_rect.width());
            while i < max {
                let pix_x = f32::from(vars.user_rect.x()) + f32::from(vars.user_rect.width() / 2)
                    - f32::from(vars.block_rect.width()) * (vars.me.pos.x - x1);
                let user_center = vars.get_user_center();
                let pix_y = f32::from(user_center.y())
                    - f32::from(vars.block_rect.height()) * (vars.me.pos.y - y1)
                    + i;
                if pix_x <= vars.user_rect.x().into()
                    || pix_x
                        >= f32::from(vars.user_rect.x()) + f32::from(vars.user_rect.width()) - 1.
                    || pix_y <= f32::from(vars.user_rect.y())
                    || pix_y
                        >= f32::from(vars.user_rect.y()) + f32::from(vars.user_rect.height()) - 1.
                {
                    i += 1.;
                    continue;
                }
                let ne_screen = graphics.ne_screen.as_mut().unwrap();
                let mut ne_screen = ne_screen.lock().unwrap();
                #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                {
                    ne_screen
                        .pixels()
                        .set(pix_x as u16, pix_y as u16, color)
                        .unwrap();
                    ne_screen
                        .pixels()
                        .set(pix_x as u16 - 1, pix_y as u16, color)
                        .unwrap();
                }

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
        let max = (x2 - x1) * f32::from(vars.block_rect.width());
        while i < max {
            let pix_x = f32::from(vars.user_rect.x()) + f32::from(vars.user_rect.width() / 2)
                - f32::from(vars.block_rect.width()) * (vars.me.pos.x - x1)
                + i;
            let user_center = vars.get_user_center();
            let pix_y = f32::from(user_center.y())
                - f32::from(vars.block_rect.height()) * (vars.me.pos.y - y1)
                + i * slope;
            if pix_x <= f32::from(vars.user_rect.x())
                || pix_x >= f32::from(vars.user_rect.x()) + f32::from(vars.user_rect.width()) - 1.
                || pix_y <= f32::from(vars.user_rect.y())
                || pix_y >= f32::from(vars.user_rect.y()) + f32::from(vars.user_rect.height()) - 1.
            {
                i += 1.;
                continue;
            }
            let ne_screen = graphics.ne_screen.as_mut().unwrap();
            let mut ne_screen = ne_screen.lock().unwrap();

            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            {
                ne_screen
                    .pixels()
                    .set(pix_x as u16, pix_y as u16, color)
                    .unwrap();
                ne_screen
                    .pixels()
                    .set(pix_x as u16, pix_y as u16 - 1, color)
                    .unwrap();
            }
            i += 1.;
        }
    }

    fn load_orig_map_block_surface_pointer(&mut self) {
        let Self {
            sdl,
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
                *orig_surface = LoadBlockVidBppPic {
                    vid_bpp: *vid_bpp,
                    pic,
                    fpath: None,
                    line: color_index.try_into().unwrap(),
                    col: block_index.try_into().unwrap(),
                    block: Some(ORIG_BLOCK_RECT),
                    flags: 0,
                    sdl,
                }
                .run()
                .map(|surface| Rc::new(RefCell::new(surface)));
                *surface = orig_surface.as_ref().map(Rc::clone);
            });
    }

    fn load_influencer_enemy_surface(&mut self) {
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
                *influencer_surface = LoadBlockVidBppPic {
                    vid_bpp: *vid_bpp,
                    pic,
                    fpath: None,
                    line: 0,
                    col: index.try_into().unwrap(),
                    block: Some(ORIG_BLOCK_RECT),
                    flags: 0,
                    sdl: self.sdl,
                }
                .run();

                /* Droid pics are only used in _internal_ blits ==> clear per-surf alpha */
                if influencer_surface
                    .as_mut()
                    .unwrap()
                    .set_alpha(ColorKeyFlag::empty(), 0)
                    .not()
                {
                    error!("Cannot set alpha channel on surface");
                }
            },
        );

        enemy_surface_pointer
            .iter_mut()
            .enumerate()
            .for_each(|(index, enemy_surface)| {
                *enemy_surface = LoadBlockVidBppPic {
                    vid_bpp: *vid_bpp,
                    pic,
                    fpath: None,
                    line: 1,
                    col: index.try_into().unwrap(),
                    block: Some(ORIG_BLOCK_RECT),
                    flags: 0,
                    sdl: self.sdl,
                }
                .run();

                /* Droid pics are only used in _internal_ blits ==> clear per-surf alpha */
                if enemy_surface
                    .as_mut()
                    .unwrap()
                    .set_alpha(ColorKeyFlag::empty(), 0)
                    .not()
                {
                    error!("Cannot set alpha channel on surface");
                }
            });
    }

    fn load_bullet_surfaces(&mut self) {
        self.vars
            .bulletmap
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
                    None,
                    bullet_type_index.try_into().unwrap(),
                    phase_index.try_into().unwrap(),
                    Some(ORIG_BLOCK_RECT),
                    0,
                    self.sdl,
                );
            });
    }

    fn load_blast_surfaces(&mut self) {
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
                    None,
                    blast_type_index.try_into().unwrap(),
                    surface_index.try_into().unwrap(),
                    Some(ORIG_BLOCK_RECT),
                    0,
                    self.sdl,
                );
            });
    }

    fn load_influencer_enemy_digit_surface(&mut self) {
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
                *surface = LoadBlockVidBppPic {
                    vid_bpp: *vid_bpp,
                    pic,
                    fpath: None,
                    line: 0,
                    col: index.try_into().unwrap(),
                    block: Some(ORIG_DIGIT_RECT),
                    flags: 0,
                    sdl: self.sdl,
                }
                .run();
            });
        enemy_digit_surface_pointer
            .iter_mut()
            .enumerate()
            .for_each(|(index, surface)| {
                *surface = LoadBlockVidBppPic {
                    vid_bpp: *vid_bpp,
                    pic,
                    fpath: None,
                    line: 0,
                    col: (index + 10).try_into().unwrap(),
                    block: Some(ORIG_DIGIT_RECT),
                    flags: 0,
                    sdl: self.sdl,
                }
                .run();
            });
    }
}

struct LoadBlockVidBppPic<'a, 'sdl: 'a> {
    pub vid_bpp: i32,
    pub pic: &'a mut Option<Surface<'sdl>>,
    pub fpath: Option<&'a CStr>,
    pub line: i32,
    pub col: i32,
    pub block: Option<Rect>,
    pub flags: i32,
    pub sdl: &'sdl Sdl,
}

impl<'sdl> LoadBlockVidBppPic<'_, 'sdl> {
    pub fn run(self) -> Option<Surface<'sdl>> {
        let Self {
            vid_bpp,
            pic,
            fpath,
            line,
            col,
            block,
            flags,
            sdl,
        } = self;

        if fpath.is_none() && pic.is_none() {
            /* we need some info.. */
            return None;
        }

        if pic.is_some() && flags == i32::from(FREE_ONLY) {
            *pic = None;
            return None;
        }

        if let Some(fpath) = fpath {
            // initialize: read & malloc new pic, dont' return a copy!!
            *pic = Some(sdl.load_image_from_c_str_path(fpath).unwrap());
        }

        if (flags & i32::from(INIT_ONLY)) != 0 {
            return None; // that's it guys, only initialzing...
        }

        let pic = pic.as_mut().unwrap();
        let dim = block.map_or_else(
            || Rect::new(0, 0, pic.width(), pic.height()),
            |block| block.with_xy(0, 0),
        );

        let usealpha = pic.format().has_alpha();
        if usealpha {
            // clear per-surf alpha for internal blit */
            if pic.set_alpha(ColorKeyFlag::empty(), 0).not() {
                error!("Cannot set alpha channel on surface");
            }
        }
        let mut tmp = Surface::create_rgb(
            dim.width().into(),
            dim.height().into(),
            vid_bpp.max(0).try_into().unwrap_or(u8::MAX),
            Rgba::default(),
        )
        .unwrap();
        let mut ret = if usealpha {
            tmp.display_format_alpha().unwrap()
        } else {
            tmp.display_format().unwrap()
        };
        drop(tmp);

        let src = dim.with_xy(
            i16::try_from(col).unwrap() * i16::try_from(dim.width() + 2).unwrap(),
            i16::try_from(line).unwrap() * i16::try_from(dim.height() + 2).unwrap(),
        );
        pic.blit_from(&src, &mut ret);
        if usealpha
            && ret
                .set_alpha(ColorKeyFlag::SRC_ALPHA | ColorKeyFlag::RLE_ACCEL, 255)
                .not()
        {
            error!("Cannot set alpha channel on surface");
        }

        Some(ret)
    }
}

fn print_init_info(vid_info: &sdl::video::InfoRef<'_>, vid_bpp: i32, vid_driver: Option<&CStr>) {
    const YN: [&str; 2] = ["no", "yes"];

    macro_rules! flag {
        ($flag:ident) => {
            (vid_info.$flag())
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
        flag_yn!(blit_hw_colorkey)
    );
    info!(
        "Are hardware to hardware alpha blits accelerated: {}",
        flag_yn!(blit_hw_alpha)
    );
    info!(
        "Are software to hardware blits accelerated: {}",
        flag_yn!(blit_sw)
    );
    info!(
        "Are software to hardware colorkey blits accelerated: {}",
        flag_yn!(blit_sw_colorkey)
    );
    info!(
        "Are software to hardware alpha blits accelerated: {}",
        flag_yn!(blit_sw_alpha)
    );
    info!("Are color fills accelerated: {}", flag_yn!(blit_fill));
    info!(
        "Total amount of video memory in Kilobytes: {}",
        vid_info.video_mem()
    );
    info!(
        "Pixel format of the video device: bpp = {}, bytes/pixel = {}",
        vid_bpp,
        vid_info.format().bytes_per_pixel()
    );
    info!(
        "Video Driver Name: {}",
        vid_driver.map_or(Cow::Borrowed("UNKNOWN DRIVER"), |vid_driver| vid_driver
            .to_string_lossy())
    );
    info!("----------------------------------------------------------------------");
}
