use crate::{
    b_font::font_height,
    defs::{
        AlertNames, AssembleCombatWindowFlags, DisplayBannerFlags, MenuAction, SoundType, Status,
        DROID_ROTATION_TIME, MAXBLASTS, MAXBULLETS, RESET, TEXT_STRETCH, UPDATE,
    },
    graphics::{scale_pic_surface, Graphics},
    map::get_map_brick,
    structs::Point,
    vars::{BRAIN_NAMES, CLASSES, CLASS_NAMES, DRIVE_NAMES, SENSOR_NAMES, WEAPON_NAMES},
    Data,
};

use log::{error, warn};
use sdl::{Rect, Surface};
use sdl_sys::{
    IMG_Load_RW, IMG_isJPG, SDL_Color, SDL_CreateRGBSurface, SDL_DisplayFormat,
    SDL_DisplayFormatAlpha, SDL_FreeSurface, SDL_RWops, SDL_SetCursor, SDL_ShowCursor,
    SDL_UpdateRects, SDL_WarpMouse, SDL_DISABLE, SDL_ENABLE,
};
use std::{
    ffi::CStr,
    ops::Not,
    os::raw::{c_char, c_float, c_int},
    ptr::{null_mut, NonNull},
};

const UPDATE_ONLY: u8 = 0x01;

pub struct ShipData<'sdl> {
    last_siren: u32,
    frame_num: c_int,
    last_droid_type: c_int,
    last_frame_time: u32,
    src_rect: Rect,
    enter_console_last_move_tick: u32,
    great_droid_show_last_move_tick: u32,
    enter_lift_last_move_tick: u32,
    droid_background: Option<Surface<'sdl>>,
    droid_pics: Option<Surface<'sdl>>,
    up_rect: Rect,
    down_rect: Rect,
    left_rect: Rect,
    right_rect: Rect,
}

impl Default for ShipData<'_> {
    fn default() -> Self {
        Self {
            last_siren: 0,
            frame_num: 0,
            last_droid_type: -1,
            last_frame_time: 0,
            src_rect: Rect::default(),
            enter_console_last_move_tick: 0,
            great_droid_show_last_move_tick: 0,
            enter_lift_last_move_tick: 0,
            droid_background: None,
            droid_pics: None,
            up_rect: Rect::default(),
            down_rect: Rect::default(),
            left_rect: Rect::default(),
            right_rect: Rect::default(),
        }
    }
}

#[inline]
pub unsafe fn sdl_rw_seek(ctx: *mut SDL_RWops, offset: c_int, whence: c_int) -> c_int {
    let seek: unsafe fn(*mut SDL_RWops, c_int, c_int) -> c_int = std::mem::transmute((*ctx).seek);
    seek(ctx, offset, whence)
}

impl Data<'_> {
    /// do all alert-related agitations: alert-sirens and alert-lights
    pub unsafe fn alert_level_warning(&mut self) {
        const SIREN_WAIT: f32 = 2.5;

        use AlertNames::*;
        match AlertNames::try_from(self.main.alert_level).ok() {
            Some(Green) => {}
            Some(Yellow) | Some(Amber) | Some(Red) => {
                if self.sdl.ticks_ms() - self.ship.last_siren
                    > (SIREN_WAIT * 1000.0 / (self.main.alert_level as f32)) as u32
                {
                    // higher alert-> faster sirens!
                    self.play_sound(SoundType::Alert as c_int);
                    self.ship.last_siren = self.sdl.ticks_ms();
                }
            }
            Some(Last) | None => {
                warn!(
                    "illegal AlertLevel = {} > {}.. something's gone wrong!!\n",
                    self.main.alert_level,
                    AlertNames::Red as c_int
                );
            }
        }

        // so much to the sirens, now make sure the alert-tiles are updated correctly:
        let posx = (*self.main.cur_level).alerts[0].x;
        let posy = (*self.main.cur_level).alerts[0].y;
        if posx == -1 {
            // no alerts here...
            return;
        }

        let cur_alert = AlertNames::try_from(self.main.alert_level).unwrap();

        // check if alert-tiles are up-to-date
        if get_map_brick(&*self.main.cur_level, posx.into(), posy.into()) == cur_alert as u8 {
            // ok
            return;
        }

        for alert in &mut (*self.main.cur_level).alerts {
            let posx = alert.x;
            let posy = alert.y;
            if posx == -1 {
                break;
            }

            *(*self.main.cur_level).map[usize::try_from(posy).unwrap()]
                .add(usize::try_from(posx).unwrap()) = cur_alert as i8;
        }
    }

    /// Show a an animated droid-pic: automatically counts frames and frametimes
    /// stored internally, so you just have to keep calling this function to get
    /// an animation. The target-rect dst is only updated when a new frame is set
    /// if flags & RESET: to restart a fresh animation at frame 0
    /// if flags & UPDATE: force a blit of droid-pic
    ///
    /// cycle_time is the time in seconds for a full animation-cycle,
    /// if cycle_time == 0 : display static pic, using only first frame
    pub unsafe fn show_droid_portrait(
        &mut self,
        mut dst: Rect,
        droid_type: c_int,
        cycle_time: c_float,
        flags: c_int,
    ) {
        let mut need_new_frame = false;

        self.graphics
            .ne_screen
            .as_mut()
            .unwrap()
            .set_clip_rect(&dst);

        let Data {
            ship:
                ShipData {
                    droid_background,
                    src_rect,
                    ..
                },
            graphics,
            vars,
            ..
        } = self;
        let droid_background = droid_background.get_or_insert_with(|| {
            // first call
            let tmp = SDL_CreateRGBSurface(
                0,
                dst.width().into(),
                dst.height().into(),
                graphics.vid_bpp,
                0,
                0,
                0,
                0,
            );
            let mut droid_background =
                Surface::from_ptr(NonNull::new(SDL_DisplayFormat(tmp)).unwrap());
            SDL_FreeSurface(tmp);
            graphics
                .ne_screen
                .as_mut()
                .unwrap()
                .blit_from(&dst, &mut droid_background);
            *src_rect = vars.portrait_rect;
            droid_background
        });

        if flags & RESET != 0 {
            self.graphics
                .ne_screen
                .as_mut()
                .unwrap()
                .blit_from(&dst, droid_background);
            self.ship.frame_num = 0;
            self.ship.last_frame_time = self.sdl.ticks_ms();
        }

        if droid_type != self.ship.last_droid_type || self.ship.droid_pics.is_none() {
            // we need to unpack the droid-pics into our local storage
            self.ship.droid_pics = None;
            let packed_portrait =
                self.graphics.packed_portraits[usize::try_from(droid_type).unwrap()];
            let tmp = IMG_Load_RW(packed_portrait, 0);
            // important: return seek-position to beginning of RWops for next operation to succeed!
            sdl_rw_seek(packed_portrait, 0, libc::SEEK_SET);
            if tmp.is_null() {
                error!(
                    "failed to unpack droid-portraits of droid-type {}",
                    droid_type,
                );
                return; // ok, so no pic but we continue ;)
            }
            // now see if its a jpg, then we add some transparency by color-keying:
            let raw_droid_pics = if IMG_isJPG(packed_portrait) != 0 {
                SDL_DisplayFormat(tmp)
            } else {
                // else assume it's png ;)
                SDL_DisplayFormatAlpha(tmp)
            };
            self.ship.droid_pics = Some(Surface::from_ptr(NonNull::new(raw_droid_pics).unwrap()));
            SDL_FreeSurface(tmp);
            sdl_rw_seek(packed_portrait, 0, libc::SEEK_SET);

            // do we have to scale the droid pics
            #[allow(clippy::float_cmp)]
            if self.global.game_config.scale != 1.0 {
                scale_pic_surface(
                    self.ship.droid_pics.as_mut().unwrap(),
                    self.global.game_config.scale,
                );
            }

            self.ship.last_droid_type = droid_type;
        }

        let droid_pics_ref = self.ship.droid_pics.as_ref().unwrap();
        let mut num_frames = droid_pics_ref.width() / self.vars.portrait_rect.width();

        // sanity check
        if num_frames == 0 {
            warn!(
                "Only one frame found. Width droid-pics={}, Frame-width={}",
                droid_pics_ref.width(),
                self.vars.portrait_rect.width(),
            );
            num_frames = 1; // continue and hope for the best
        }

        let frame_duration = self.sdl.ticks_ms() - self.ship.last_frame_time;

        if cycle_time != 0. && (frame_duration as f32 > 1000.0 * cycle_time / num_frames as f32) {
            need_new_frame = true;
            self.ship.frame_num += 1;
        }

        if self.ship.frame_num >= i32::from(num_frames) {
            self.ship.frame_num = 0;
        }

        if flags & (RESET | UPDATE) != 0 || need_new_frame {
            self.ship.src_rect.set_x(
                i16::try_from(self.ship.frame_num).unwrap()
                    * i16::try_from(self.ship.src_rect.width()).unwrap(),
            );

            let Data {
                ship:
                    ShipData {
                        droid_background,
                        droid_pics,
                        src_rect,
                        ..
                    },
                graphics: Graphics { ne_screen, .. },
                ..
            } = self;
            droid_background
                .as_mut()
                .unwrap()
                .blit_to(ne_screen.as_mut().unwrap(), &mut dst);
            droid_pics.as_mut().unwrap().blit_from_to(
                &*src_rect,
                ne_screen.as_mut().unwrap(),
                &mut dst,
            );

            SDL_UpdateRects(
                self.graphics.ne_screen.as_mut().unwrap().as_mut_ptr(),
                1,
                dst.as_mut(),
            );

            self.ship.last_frame_time = self.sdl.ticks_ms();
        }

        self.graphics.ne_screen.as_mut().unwrap().clear_clip_rect();
    }

    /// display infopage page of droidtype
    ///
    /// if flags == UPDATE_ONLY : don't blit a new background&banner,
    ///                           only  update the text-regions
    ///
    ///  does update the screen: all if flags=0, text-rect if flags=UPDATE_ONLY
    pub unsafe fn show_droid_info(&mut self, droid_type: c_int, page: c_int, flags: c_int) {
        use std::io::Write;

        self.graphics.ne_screen.as_mut().unwrap().clear_clip_rect();
        self.b_font.current_font = self.global.para_b_font;

        let lineskip =
            ((f64::from(font_height(&*self.b_font.current_font)) * TEXT_STRETCH) as f32) as i16;
        let lastline = self.vars.cons_header_rect.y()
            + i16::try_from(self.vars.cons_header_rect.height()).unwrap();
        self.ship.up_rect = Rect::new(self.vars.cons_header_rect.x(), lastline - lineskip, 25, 13);
        self.ship.down_rect = Rect::new(
            self.vars.cons_header_rect.x(),
            (f32::from(lastline) - 0.5 * f32::from(lineskip)) as i16,
            25,
            13,
        );
        self.ship.left_rect = Rect::new(
            (f32::from(
                self.vars.cons_header_rect.x()
                    + i16::try_from(self.vars.cons_header_rect.width()).unwrap(),
            ) - 1.5 * f32::from(lineskip)) as i16,
            (f32::from(lastline) - 0.9 * f32::from(lineskip)) as i16,
            13,
            25,
        );
        self.ship.right_rect = Rect::new(
            (f32::from(
                self.vars.cons_header_rect.x()
                    + i16::try_from(self.vars.cons_header_rect.width()).unwrap(),
            ) - 1.0 * f32::from(lineskip)) as i16,
            (f32::from(lastline) - 0.9 * f32::from(lineskip)) as i16,
            13,
            25,
        );

        let mut droid_name = [0u8; 80];
        let droid = &*self.vars.droidmap.add(usize::try_from(droid_type).unwrap());
        write!(
            droid_name.as_mut(),
            "  Unit type {} - {}\0",
            CStr::from_ptr(droid.druidname.as_ptr()).to_str().unwrap(),
            CLASS_NAMES[usize::try_from(droid.class).unwrap()]
                .to_str()
                .unwrap()
        )
        .unwrap();

        let mut info_text = [0u8; 1000];
        let mut show_arrows = false;
        match page {
            -3 => {
                // Title screen: intro unit
                write!(
                    info_text.as_mut(),
                    "This is the unit that you currently control. Prepare to board Robo-frighter \
Paradroid to eliminate all rogue robots.\0",
                )
                .unwrap();
            }
            -2 => {
                // Takeover: unit that you wish to control
                write!(
                    info_text.as_mut(),
                    "This is the unit that you wish to control.\n\n Prepare to takeover.\0",
                )
                .unwrap();
            }
            -1 => {
                // Takeover: unit that you control
                write!(
                    info_text.as_mut(),
                    "This is the unit that you currently control.\0"
                )
                .unwrap();
            }
            0 => {
                show_arrows = true;
                write!(
                    info_text.as_mut(),
                    "Entry : {:02}\n\
                 Class : {}\n\
                 Height : {:5.2} m\n\
                 Weight: {} kg\n\
                 Drive : {} \n\
                 Brain : {}\0",
                    droid_type + 1,
                    CLASSES[usize::try_from(droid.class).unwrap()]
                        .to_str()
                        .unwrap(),
                    droid.height,
                    droid.weight,
                    DRIVE_NAMES[usize::try_from(droid.drive).unwrap()]
                        .to_str()
                        .unwrap(),
                    BRAIN_NAMES[usize::try_from(droid.brain).unwrap()]
                        .to_str()
                        .unwrap(),
                )
                .unwrap();
            }
            1 => {
                show_arrows = true;
                write!(
                    info_text.as_mut(),
                    "Armament : {}\n\
                 Sensors  1: {}\n\
                    2: {}\n\
                    3: {}\0",
                    WEAPON_NAMES[usize::try_from(droid.gun).unwrap()]
                        .to_str()
                        .unwrap(),
                    SENSOR_NAMES[usize::try_from(droid.sensor1).unwrap()]
                        .to_str()
                        .unwrap(),
                    SENSOR_NAMES[usize::try_from(droid.sensor2).unwrap()]
                        .to_str()
                        .unwrap(),
                    SENSOR_NAMES[usize::try_from(droid.sensor3).unwrap()]
                        .to_str()
                        .unwrap(),
                )
                .unwrap();
            }
            2 => {
                show_arrows = true;
                write!(
                    info_text.as_mut(),
                    "Notes: {}\0",
                    CStr::from_ptr(droid.notes).to_str().unwrap()
                )
                .unwrap();
            }
            _ => {
                write!(
                    info_text.as_mut(),
                    "ERROR: Page not implemented!! \nPlease report bug!\0",
                )
                .unwrap();
            }
        }

        let Data {
            graphics:
                Graphics {
                    console_bg_pic2,
                    ne_screen,
                    ..
                },
            vars,
            ..
        } = self;

        // if UPDATE_ONLY then the background has not been cleared, so we have do it
        // it for each menu-rect:
        if flags & i32::from(UPDATE_ONLY) != 0 {
            ne_screen
                .as_mut()
                .unwrap()
                .set_clip_rect(&vars.cons_text_rect);
            console_bg_pic2
                .as_mut()
                .unwrap()
                .blit(ne_screen.as_mut().unwrap());
            ne_screen
                .as_mut()
                .unwrap()
                .set_clip_rect(&vars.cons_header_rect);
            console_bg_pic2
                .as_mut()
                .unwrap()
                .blit(ne_screen.as_mut().unwrap());
            ne_screen.as_mut().unwrap().clear_clip_rect();
        } else {
            // otherwise we just redraw the whole screen
            console_bg_pic2
                .as_mut()
                .unwrap()
                .blit(ne_screen.as_mut().unwrap());
            self.display_banner(
                null_mut(),
                null_mut(),
                (DisplayBannerFlags::NO_SDL_UPDATE | DisplayBannerFlags::FORCE_UPDATE)
                    .bits()
                    .into(),
            );
        }

        self.display_text(
            info_text.as_mut_ptr() as *mut c_char,
            self.vars.cons_text_rect.x().into(),
            self.vars.cons_text_rect.y().into(),
            &self.vars.cons_text_rect,
        );

        self.display_text(
            droid_name.as_mut_ptr() as *mut c_char,
            i32::from(self.vars.cons_header_rect.x()) + i32::from(lineskip),
            (f32::from(lastline) - 0.9 * f32::from(lineskip)) as i32,
            null_mut(),
        );

        if show_arrows {
            let Data {
                graphics:
                    Graphics {
                        arrow_up,
                        arrow_down,
                        arrow_left,
                        arrow_right,
                        ne_screen,
                        ..
                    },
                ship,
                vars,
                ..
            } = self;

            if vars.me.ty > droid_type {
                arrow_up
                    .as_mut()
                    .unwrap()
                    .blit_to(ne_screen.as_mut().unwrap(), &mut ship.up_rect);
            }

            if droid_type > 0 {
                arrow_down
                    .as_mut()
                    .unwrap()
                    .blit_to(ne_screen.as_mut().unwrap(), &mut ship.down_rect);
            }

            if page > 0 {
                arrow_left
                    .as_mut()
                    .unwrap()
                    .blit_to(ne_screen.as_mut().unwrap(), &mut ship.left_rect);
            }

            if page < 2 {
                arrow_right
                    .as_mut()
                    .unwrap()
                    .blit_to(ne_screen.as_mut().unwrap(), &mut ship.right_rect);
            }
        }

        if flags & i32::from(UPDATE_ONLY) != 0 {
            SDL_UpdateRects(
                self.graphics.ne_screen.as_mut().unwrap().as_mut_ptr(),
                1,
                self.vars.cons_header_rect.as_mut(),
            );
            SDL_UpdateRects(
                self.graphics.ne_screen.as_mut().unwrap().as_mut_ptr(),
                1,
                self.vars.cons_text_rect.as_mut(),
            );
        } else {
            assert!(self.graphics.ne_screen.as_mut().unwrap().flip());
        }
    }

    /// Displays the concept view of deck
    ///
    /// Note: we no longer wait here for a key-press, but return
    /// immediately
    pub unsafe fn show_deck_map(&mut self) {
        let tmp = self.vars.me.pos;

        let cur_level = &*self.main.cur_level;
        self.vars.me.pos.x = (cur_level.xlen / 2) as f32;
        self.vars.me.pos.y = (cur_level.ylen / 2) as f32;

        SDL_ShowCursor(SDL_DISABLE);

        self.set_combat_scale_to(0.25);

        self.assemble_combat_picture(
            (AssembleCombatWindowFlags::ONLY_SHOW_MAP | AssembleCombatWindowFlags::SHOW_FULL_MAP)
                .bits()
                .into(),
        );

        assert!(self.graphics.ne_screen.as_mut().unwrap().flip());

        self.vars.me.pos = tmp;

        self.wait_for_key_pressed();

        self.set_combat_scale_to(1.0);
    }

    /// EnterKonsole(): does all konsole- duties
    /// This function runs the consoles. This means the following duties:
    /// 2 * Show a small-scale plan of the current deck
    /// 3 * Show a side-elevation on the ship
    /// 1 * Give all available data on lower druid types
    /// 0 * Reenter the game without squashing the colortable
    pub unsafe fn enter_konsole(&mut self) {
        // Prevent distortion of framerate by the delay coming from
        // the time spend in the menu.
        self.activate_conservative_frame_computation();

        let tmp_rect = self.vars.user_rect;
        self.vars.user_rect = self.vars.full_user_rect;

        self.wait_for_all_keys_released();

        self.vars.me.status = Status::Console as c_int;

        if cfg!(target_os = "android") {
            self.input.show_cursor = false;
        }

        SDL_SetCursor(self.graphics.arrow_cursor);

        self.b_font.current_font = self.global.para_b_font;

        let mut pos = 0; // starting menu position
        self.paint_console_menu(c_int::try_from(pos).unwrap(), 0);

        let wait_move_ticks: u32 = 100;
        let mut finished = false;
        let mut need_update = true;
        while !finished {
            if self.input.show_cursor {
                SDL_ShowCursor(SDL_ENABLE);
            } else {
                SDL_ShowCursor(SDL_DISABLE);
            }

            // check if the mouse-cursor is on any of the console-menu points
            for (i, rect) in self.vars.cons_menu_rects.iter().enumerate() {
                if self.input.show_cursor && pos != i && self.cursor_is_on_rect(rect) != 0 {
                    self.move_menu_position_sound();
                    pos = i;
                    need_update = true;
                }
            }
            let action = self.get_menu_action(250);
            if self.sdl.ticks_ms() - self.ship.enter_console_last_move_tick > wait_move_ticks {
                match action {
                    MenuAction::BACK => {
                        finished = true;
                        self.wait_for_all_keys_released();
                    }

                    MenuAction::UP => {
                        if pos > 0 {
                            pos -= 1;
                        } else {
                            pos = 3;
                        }
                        // when warping the mouse-cursor: don't count that as a mouse-activity
                        // this is a dirty hack, but that should be enough for here...
                        if self.input.show_cursor {
                            let mousemove_buf = self.input.last_mouse_event;
                            SDL_WarpMouse(
                                (self.vars.cons_menu_rects[pos].x()
                                    + i16::try_from(self.vars.cons_menu_rects[pos].width() / 2)
                                        .unwrap())
                                .try_into()
                                .unwrap(),
                                (self.vars.cons_menu_rects[pos].y()
                                    + i16::try_from(self.vars.cons_menu_rects[pos].height() / 2)
                                        .unwrap())
                                .try_into()
                                .unwrap(),
                            );
                            self.update_input(); // this sets a new last_mouse_event
                            self.input.last_mouse_event = mousemove_buf; //... which we override.. ;)
                        }
                        self.move_menu_position_sound();
                        need_update = true;
                        self.ship.enter_console_last_move_tick = self.sdl.ticks_ms();
                    }

                    MenuAction::DOWN => {
                        if pos < 3 {
                            pos += 1;
                        } else {
                            pos = 0;
                        }
                        // when warping the mouse-cursor: don't count that as a mouse-activity
                        // this is a dirty hack, but that should be enough for here...
                        if self.input.show_cursor {
                            let mousemove_buf = self.input.last_mouse_event;
                            SDL_WarpMouse(
                                (self.vars.cons_menu_rects[pos].x()
                                    + i16::try_from(self.vars.cons_menu_rects[pos].width() / 2)
                                        .unwrap())
                                .try_into()
                                .unwrap(),
                                (self.vars.cons_menu_rects[pos].y()
                                    + i16::try_from(self.vars.cons_menu_rects[pos].height() / 2)
                                        .unwrap())
                                .try_into()
                                .unwrap(),
                            );
                            self.update_input(); // this sets a new last_mouse_event
                            self.input.last_mouse_event = mousemove_buf; //... which we override.. ;)
                        }
                        self.move_menu_position_sound();
                        need_update = true;
                        self.ship.enter_console_last_move_tick = self.sdl.ticks_ms();
                    }

                    MenuAction::CLICK => {
                        self.menu_item_selected_sound();
                        self.wait_for_all_keys_released();
                        need_update = true;
                        match pos {
                            0 => {
                                finished = true;
                            }
                            1 => {
                                self.great_druid_show();
                                self.paint_console_menu(pos.try_into().unwrap(), 0);
                            }
                            2 => {
                                self.clear_graph_mem();
                                self.display_banner(
                                    null_mut(),
                                    null_mut(),
                                    DisplayBannerFlags::FORCE_UPDATE.bits().into(),
                                );
                                self.show_deck_map();
                                self.paint_console_menu(pos.try_into().unwrap(), 0);
                            }
                            3 => {
                                self.clear_graph_mem();
                                self.display_banner(
                                    null_mut(),
                                    null_mut(),
                                    DisplayBannerFlags::FORCE_UPDATE.bits().into(),
                                );
                                self.show_lifts((*self.main.cur_level).levelnum, -1);
                                self.wait_for_key_pressed();
                                self.paint_console_menu(pos.try_into().unwrap(), 0);
                            }
                            _ => {
                                error!("Konsole menu out of bounds... pos = {}", pos);
                                pos = 0;
                            }
                        }
                    }
                    _ => {}
                }
            }

            if need_update {
                self.paint_console_menu(pos.try_into().unwrap(), UPDATE_ONLY.into());
                if cfg!(not(target_os = "android")) {
                    assert!(self.graphics.ne_screen.as_mut().unwrap().flip());
                }

                need_update = false;
            }
            if cfg!(target_os = "android") {
                assert!(self.graphics.ne_screen.as_mut().unwrap().flip());
                // for responsive input on Android, we need to run this every cycle
            }

            self.sdl.delay_ms(1); // don't hog CPU
        }

        self.vars.user_rect = tmp_rect;

        self.vars.me.status = Status::Mobile as c_int;

        self.clear_graph_mem();

        SDL_SetCursor(self.graphics.crosshair_cursor);
        if !self.input.show_cursor {
            SDL_ShowCursor(SDL_DISABLE);
        }
    }

    /// This function does the robot show when the user has selected robot
    /// show from the console menu.
    pub unsafe fn great_druid_show(&mut self) {
        let mut finished = false;

        let mut droidtype = self.vars.me.ty;
        let mut page = 0;

        self.show_droid_info(droidtype, page, 0);
        self.show_droid_portrait(self.vars.cons_droid_rect, droidtype, 0.0, UPDATE | RESET);

        self.wait_for_all_keys_released();
        let mut need_update = true;
        let wait_move_ticks: u32 = 100;

        while !finished {
            self.show_droid_portrait(self.vars.cons_droid_rect, droidtype, DROID_ROTATION_TIME, 0);

            if self.input.show_cursor {
                SDL_ShowCursor(SDL_ENABLE);
            } else {
                SDL_ShowCursor(SDL_DISABLE);
            }

            if need_update {
                self.show_droid_info(droidtype, page, UPDATE_ONLY.into());
                need_update = false;
            }

            let mut action = MenuAction::empty();
            // special handling of mouse-clicks: check if move-arrows were clicked on
            if self.mouse_left_pressed_r() {
                if self.cursor_is_on_rect(&self.ship.left_rect) != 0 {
                    action = MenuAction::LEFT;
                } else if self.cursor_is_on_rect(&self.ship.right_rect) != 0 {
                    action = MenuAction::RIGHT;
                } else if self.cursor_is_on_rect(&self.ship.up_rect) != 0 {
                    action = MenuAction::UP;
                } else if self.cursor_is_on_rect(&self.ship.down_rect) != 0 {
                    action = MenuAction::DOWN;
                }
            } else {
                action = self.get_menu_action(250);
            }

            let time_for_move =
                self.sdl.ticks_ms() - self.ship.great_droid_show_last_move_tick > wait_move_ticks;
            match action {
                MenuAction::BACK | MenuAction::CLICK => {
                    finished = true;
                    self.wait_for_all_keys_released();
                }

                MenuAction::UP => {
                    if !time_for_move {
                        continue;
                    }

                    if droidtype < self.vars.me.ty {
                        self.move_menu_position_sound();
                        droidtype += 1;
                        need_update = true;
                        self.ship.great_droid_show_last_move_tick = self.sdl.ticks_ms();
                    }
                }

                MenuAction::DOWN => {
                    if !time_for_move {
                        continue;
                    }

                    if droidtype > 0 {
                        self.move_menu_position_sound();
                        droidtype -= 1;
                        need_update = true;
                        self.ship.great_droid_show_last_move_tick = self.sdl.ticks_ms();
                    }
                }

                MenuAction::RIGHT => {
                    if !time_for_move {
                        continue;
                    }

                    if page < 2 {
                        self.move_menu_position_sound();
                        page += 1;
                        need_update = true;
                        self.ship.great_droid_show_last_move_tick = self.sdl.ticks_ms();
                    }
                }

                MenuAction::LEFT => {
                    if !time_for_move {
                        continue;
                    }

                    if page > 0 {
                        self.move_menu_position_sound();
                        page -= 1;
                        need_update = true;
                        self.ship.great_droid_show_last_move_tick = self.sdl.ticks_ms();
                    }
                }
                _ => {}
            }

            self.sdl.delay_ms(1); // don't hog CPU
        }
    }

    /// This function should check if the mouse cursor is in the given Rectangle
    pub unsafe fn cursor_is_on_rect(&self, rect: &Rect) -> c_int {
        let user_center = self.vars.get_user_center();
        let cur_pos = Point {
            x: self.input.input_axis.x + (i32::from(user_center.x()) - 16),
            y: self.input.input_axis.y + (i32::from(user_center.y()) - 16),
        };

        (cur_pos.x >= rect.x().into()
            && cur_pos.x <= i32::from(rect.x()) + i32::from(rect.width())
            && cur_pos.y >= rect.y().into()
            && cur_pos.y <= i32::from(rect.y()) + i32::from(rect.height()))
        .into()
    }

    /// @Desc: show side-view of the ship, and hightlight the current
    ///        level + lift
    ///
    ///  if level==-1: don't highlight any level
    ///  if liftrow==-1: dont' highlight any liftrows
    pub unsafe fn show_lifts(&mut self, level: c_int, liftrow: c_int) {
        let lift_bg_color = SDL_Color {
            r: 0,
            g: 0,
            b: 0,
            unused: 0,
        }; /* black... */
        let xoffs: i16 = (self.vars.user_rect.width() / 20).try_into().unwrap();
        let yoffs: i16 = (self.vars.user_rect.height() / 5).try_into().unwrap();

        SDL_ShowCursor(SDL_DISABLE);
        // fill the user fenster with some color
        self.fill_rect(self.vars.user_rect, lift_bg_color);

        /* First blit ship "lights off" */
        let mut dst = self.vars.user_rect;
        self.graphics
            .ne_screen
            .as_mut()
            .unwrap()
            .set_clip_rect(&dst);
        dst = self.vars.user_rect;
        dst.inc_x(xoffs);
        dst.inc_y(yoffs);

        let Graphics {
            ship_off_pic,
            ship_on_pic,
            ne_screen,
            ..
        } = &mut self.graphics;
        ship_off_pic
            .as_mut()
            .unwrap()
            .blit_to(ne_screen.as_mut().unwrap(), &mut dst);

        if level >= 0 {
            for i in 0..self.main.cur_ship.num_level_rects[usize::try_from(level).unwrap()] {
                let src = self.main.cur_ship.level_rects[usize::try_from(level).unwrap()]
                    [usize::try_from(i).unwrap()];
                dst = src;
                dst.inc_x(self.vars.user_rect.x() + xoffs); /* offset respective to User-Rectangle */
                dst.inc_y(self.vars.user_rect.y() + yoffs);
                ship_on_pic.as_mut().unwrap().blit_from_to(
                    &src,
                    ne_screen.as_mut().unwrap(),
                    &mut dst,
                );
            }
        }

        if liftrow >= 0 {
            let src = self.main.cur_ship.lift_row_rect[usize::try_from(liftrow).unwrap()];
            dst = src;
            dst.inc_x(self.vars.user_rect.x() + xoffs); /* offset respective to User-Rectangle */
            dst.inc_y(self.vars.user_rect.y() + yoffs);
            ship_on_pic
                .as_mut()
                .unwrap()
                .blit_from_to(&src, ne_screen.as_mut().unwrap(), &mut dst);
        }

        assert!(self.graphics.ne_screen.as_mut().unwrap().flip());
    }

    /// diese Funktion zeigt die m"oglichen Auswahlpunkte des Menus
    /// Sie soll die Schriftfarben nicht ver"andern
    ///
    /// NOTE: this function does not actually _display_ anything yet,
    ///       it just prepares the display, so you need
    ///       to call SDL_Flip() to display the result!
    /// pos  : 0<=pos<=3: which menu-position is currently active?
    /// flag : UPDATE_ONLY  only update the console-menu bar, not text & background
    pub unsafe fn paint_console_menu(&mut self, pos: c_int, flag: c_int) {
        use std::io::Write;
        let mut menu_text: [u8; 200] = [0; 200];

        if (flag & i32::from(UPDATE_ONLY)) == 0 {
            self.clear_graph_mem();

            let Data {
                graphics:
                    Graphics {
                        ne_screen,
                        console_bg_pic1,
                        ..
                    },
                ..
            } = self;
            ne_screen.as_mut().unwrap().clear_clip_rect();
            console_bg_pic1
                .as_mut()
                .unwrap()
                .blit(ne_screen.as_mut().unwrap());

            self.display_banner(
                null_mut(),
                null_mut(),
                DisplayBannerFlags::FORCE_UPDATE.bits().into(),
            );

            write!(
                &mut menu_text[..],
                "Area : {}\nDeck : {}    Alert: {}\0",
                CStr::from_ptr(self.main.cur_ship.area_name.as_ptr())
                    .to_str()
                    .unwrap(),
                CStr::from_ptr((*self.main.cur_level).levelname)
                    .to_str()
                    .unwrap(),
                AlertNames::try_from(self.main.alert_level)
                    .unwrap()
                    .to_str(),
            )
            .unwrap();
            self.display_text(
                menu_text.as_mut_ptr() as *mut c_char,
                self.vars.cons_header_rect.x().into(),
                self.vars.cons_header_rect.y().into(),
                &self.vars.cons_header_rect,
            );

            write!(
                &mut menu_text[..],
                "Logout from console\n\nDroid info\n\nDeck map\n\nShip map\0"
            )
            .unwrap();
            self.display_text(
                menu_text.as_mut_ptr() as *mut c_char,
                self.vars.cons_text_rect.x().into(),
                c_int::from(self.vars.cons_text_rect.y()) + 25,
                &self.vars.cons_text_rect,
            );
        } // only if not UPDATE_ONLY was required

        let src = Rect::new(
            i16::try_from(self.vars.cons_menu_rects[0].width()).unwrap()
                * i16::try_from(pos).unwrap()
                + (2. * pos as f32 * self.global.game_config.scale) as i16,
            0,
            self.vars.cons_menu_rect.width(),
            4 * self.vars.cons_menu_rect.height(),
        );
        let Data {
            graphics:
                Graphics {
                    console_pic,
                    ne_screen,
                    ..
                },
            vars,
            ..
        } = self;
        console_pic.as_mut().unwrap().blit_from_to(
            &src,
            ne_screen.as_mut().unwrap(),
            &mut vars.cons_menu_rect,
        );
    }

    /// does all the work when we enter a lift
    pub unsafe fn enter_lift(&mut self) {
        /* Prevent distortion of framerate by the delay coming from
         * the time spend in the menu. */
        self.activate_conservative_frame_computation();

        /* make sure to release the fire-key */
        self.wait_for_all_keys_released();

        /* Prevent the influ from coming out of the lift in transfer mode
         * by turning off transfer mode as soon as the influ enters the lift */
        self.vars.me.status = Status::Elevator as c_int;

        SDL_ShowCursor(SDL_DISABLE);

        let mut cur_level = (*self.main.cur_level).levelnum;

        let cur_lift = self.get_current_lift();
        if cur_lift == -1 {
            error!("Lift out of order, I'm so sorry !");
            return;
        }
        let mut cur_lift: usize = cur_lift.try_into().unwrap();

        self.enter_lift_sound();
        self.switch_background_music_to(null_mut()); // turn off Bg music

        let mut up_lift = self.main.cur_ship.all_lifts[cur_lift].up;
        let mut down_lift = self.main.cur_ship.all_lifts[cur_lift].down;

        let liftrow = self.main.cur_ship.all_lifts[cur_lift].lift_row;

        // clear the whole screen
        self.clear_graph_mem();
        self.display_banner(
            null_mut(),
            null_mut(),
            DisplayBannerFlags::FORCE_UPDATE.bits().into(),
        );

        let wait_move_ticks: u32 = 100;
        let mut finished = false;
        while !finished {
            self.show_lifts(cur_level, liftrow);

            let action = self.get_menu_action(500);
            if self.sdl.ticks_ms() - self.ship.enter_lift_last_move_tick > wait_move_ticks {
                match action {
                    MenuAction::CLICK => {
                        finished = true;
                        self.wait_for_all_keys_released();
                    }

                    MenuAction::UP | MenuAction::UP_WHEEL => {
                        self.ship.enter_lift_last_move_tick = self.sdl.ticks_ms();
                        if up_lift != -1 {
                            if self.main.cur_ship.all_lifts[usize::try_from(up_lift).unwrap()].x
                                == 99
                            {
                                error!("Lift out of order, so sorry ..");
                            } else {
                                down_lift = cur_lift.try_into().unwrap();
                                cur_lift = up_lift.try_into().unwrap();
                                cur_level = self.main.cur_ship.all_lifts[cur_lift].level;
                                up_lift = self.main.cur_ship.all_lifts[cur_lift].up;
                                self.show_lifts(cur_level, liftrow);
                                self.move_lift_sound();
                            }
                        }
                    }

                    MenuAction::DOWN | MenuAction::DOWN_WHEEL => {
                        self.ship.enter_lift_last_move_tick = self.sdl.ticks_ms();
                        if down_lift != -1 {
                            if self.main.cur_ship.all_lifts[usize::try_from(down_lift).unwrap()].x
                                == 99
                            {
                                error!("Lift Out of order, so sorry ..");
                            } else {
                                up_lift = cur_lift.try_into().unwrap();
                                cur_lift = down_lift.try_into().unwrap();
                                cur_level = self.main.cur_ship.all_lifts[cur_lift].level;
                                down_lift = self.main.cur_ship.all_lifts[cur_lift].down;
                                self.show_lifts(cur_level, liftrow);
                                self.move_lift_sound();
                            }
                        }
                    }
                    _ => {}
                }
            }
            self.sdl.delay_ms(1); // don't hog CPU
        }

        // It might happen, that the influencer enters the elevator, but then decides to
        // come out on the same level where he has been before.  In this case of course there
        // is no need to reshuffle enemys or to reset influencers position.  Therefore, only
        // when a real level change has occured, we need to do real changes as below, where
        // we set the new level and set new position and initiate timers and all that...
        if cur_level != (*self.main.cur_level).levelnum {
            let mut array_num = 0;

            loop {
                let tmp = self.main.cur_ship.all_levels[array_num];
                if tmp.is_null() {
                    break;
                }

                if (*tmp).levelnum == cur_level {
                    break;
                } else {
                    array_num += 1;
                }
            }

            self.main.cur_level = self.main.cur_ship.all_levels[array_num];

            // set the position of the influencer to the correct locatiohn
            self.vars.me.pos.x = self.main.cur_ship.all_lifts[cur_lift].x as f32;
            self.vars.me.pos.y = self.main.cur_ship.all_lifts[cur_lift].y as f32;

            for i in 0..c_int::try_from(MAXBLASTS).unwrap() {
                self.delete_blast(i);
            }
            for i in 0..c_int::try_from(MAXBULLETS).unwrap() {
                self.delete_bullet(i);
            }
        }

        let cur_level = &*self.main.cur_level;
        self.leave_lift_sound();
        self.switch_background_music_to(cur_level.background_song_name);
        self.clear_graph_mem();
        self.display_banner(
            null_mut(),
            null_mut(),
            DisplayBannerFlags::FORCE_UPDATE.bits().into(),
        );

        self.vars.me.status = Status::Mobile as c_int;
        self.vars.me.text_visible_time = 0.;
        self.vars.me.text_to_be_displayed = cur_level.level_enter_comment;
    }

    pub unsafe fn level_empty(&self) -> c_int {
        let cur_level = &*self.main.cur_level;
        if cur_level.empty != 0 {
            return true.into();
        }

        let levelnum = cur_level.levelnum;

        self.main.all_enemys[0..usize::try_from(self.main.num_enemys).unwrap()]
            .iter()
            .any(|enemy| {
                enemy.levelnum == levelnum
                    && enemy.status != Status::Out as c_int
                    && enemy.status != Status::Terminated as c_int
            })
            .not()
            .into()
    }
}
