use crate::{
    b_font::font_height,
    defs::{
        AlertNames, AssembleCombatWindowFlags, DisplayBannerFlags, MenuAction, SoundType, Status,
        DROID_ROTATION_TIME, MAXBLASTS, MAXBULLETS, RESET, TEXT_STRETCH, UPDATE,
    },
    graphics::scale_pic,
    map::get_map_brick,
    structs::Point,
    vars::{BRAIN_NAMES, CLASSES, CLASS_NAMES, DRIVE_NAMES, SENSOR_NAMES, WEAPON_NAMES},
    Data,
};

use log::{error, warn};
use sdl_sys::{
    IMG_Load_RW, IMG_isJPG, SDL_Color, SDL_CreateRGBSurface, SDL_Delay, SDL_DisplayFormat,
    SDL_DisplayFormatAlpha, SDL_Flip, SDL_FreeSurface, SDL_GetTicks, SDL_RWops, SDL_Rect,
    SDL_SetClipRect, SDL_SetCursor, SDL_ShowCursor, SDL_Surface, SDL_UpdateRects, SDL_UpperBlit,
    SDL_WarpMouse, SDL_DISABLE, SDL_ENABLE,
};
use std::{
    convert::{TryFrom, TryInto},
    ffi::CStr,
    fmt,
    ops::Not,
    os::raw::{c_char, c_float, c_int},
    ptr::null_mut,
};

const UPDATE_ONLY: u8 = 0x01;

pub struct ShipData {
    last_siren: u32,
    frame_num: c_int,
    last_droid_type: c_int,
    last_frame_time: u32,
    src_rect: SDL_Rect,
    enter_console_last_move_tick: u32,
    great_droid_show_last_move_tick: u32,
    enter_lift_last_move_tick: u32,
    droid_background: *mut SDL_Surface,
    droid_pics: *mut SDL_Surface,
    up_rect: SDL_Rect,
    down_rect: SDL_Rect,
    left_rect: SDL_Rect,
    right_rect: SDL_Rect,
}

impl fmt::Debug for ShipData {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[derive(Debug)]
        struct Rect {
            x: i16,
            y: i16,
            w: u16,
            h: u16,
        }

        impl From<&SDL_Rect> for Rect {
            fn from(rect: &SDL_Rect) -> Rect {
                Rect {
                    x: rect.x,
                    y: rect.y,
                    w: rect.w,
                    h: rect.h,
                }
            }
        }

        let src_rect = Rect::from(&self.src_rect);
        let up_rect = Rect::from(&self.up_rect);
        let down_rect = Rect::from(&self.down_rect);
        let left_rect = Rect::from(&self.left_rect);
        let right_rect = Rect::from(&self.right_rect);

        f.debug_struct("ShipData")
            .field("last_siren", &self.last_siren)
            .field("frame_num", &self.frame_num)
            .field("last_droid_type", &self.last_droid_type)
            .field("last_frame_time", &self.last_frame_time)
            .field("src_rect", &src_rect)
            .field(
                "enter_console_last_move_tick",
                &self.enter_console_last_move_tick,
            )
            .field(
                "great_droid_show_last_move_tick",
                &self.great_droid_show_last_move_tick,
            )
            .field("enter_lift_last_move_tick", &self.enter_lift_last_move_tick)
            .field("droid_background", &self.droid_background)
            .field("droid_pics", &self.droid_pics)
            .field("up_rect", &up_rect)
            .field("down_rect", &down_rect)
            .field("left_rect", &left_rect)
            .field("right_rect", &right_rect)
            .finish()
    }
}

impl Default for ShipData {
    fn default() -> Self {
        Self {
            last_siren: 0,
            frame_num: 0,
            last_droid_type: -1,
            last_frame_time: 0,
            src_rect: rect!(0, 0, 0, 0),
            enter_console_last_move_tick: 0,
            great_droid_show_last_move_tick: 0,
            enter_lift_last_move_tick: 0,
            droid_background: null_mut(),
            droid_pics: null_mut(),
            up_rect: rect!(0, 0, 0, 0),
            down_rect: rect!(0, 0, 0, 0),
            left_rect: rect!(0, 0, 0, 0),
            right_rect: rect!(0, 0, 0, 0),
        }
    }
}

#[inline]
pub unsafe fn sdl_rw_seek(ctx: *mut SDL_RWops, offset: c_int, whence: c_int) -> c_int {
    let seek: unsafe fn(*mut SDL_RWops, c_int, c_int) -> c_int = std::mem::transmute((*ctx).seek);
    seek(ctx, offset, whence)
}

impl Data {
    pub unsafe fn free_droid_pics(&self) {
        SDL_FreeSurface(self.ship.droid_pics);
        SDL_FreeSurface(self.ship.droid_background);
    }

    /// do all alert-related agitations: alert-sirens and alert-lights
    pub unsafe fn alert_level_warning(&mut self) {
        const SIREN_WAIT: f32 = 2.5;

        use AlertNames::*;
        match AlertNames::try_from(self.main.alert_level).ok() {
            Some(Green) => {}
            Some(Yellow) | Some(Amber) | Some(Red) => {
                if SDL_GetTicks() - self.ship.last_siren
                    > (SIREN_WAIT * 1000.0 / (self.main.alert_level as f32)) as u32
                {
                    // higher alert-> faster sirens!
                    self.play_sound(SoundType::Alert as c_int);
                    self.ship.last_siren = SDL_GetTicks();
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
}

impl Data {
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
        mut dst: SDL_Rect,
        droid_type: c_int,
        cycle_time: c_float,
        flags: c_int,
    ) {
        let mut need_new_frame = false;

        SDL_SetClipRect(self.graphics.ne_screen, &dst);

        if self.ship.droid_background.is_null() {
            // first call
            let tmp = SDL_CreateRGBSurface(
                0,
                dst.w.into(),
                dst.h.into(),
                self.graphics.vid_bpp,
                0,
                0,
                0,
                0,
            );
            self.ship.droid_background = SDL_DisplayFormat(tmp);
            SDL_FreeSurface(tmp);
            SDL_UpperBlit(
                self.graphics.ne_screen,
                &mut dst,
                self.ship.droid_background,
                null_mut(),
            );
            self.ship.src_rect = self.vars.portrait_rect;
        }

        if flags & RESET != 0 {
            SDL_UpperBlit(
                self.graphics.ne_screen,
                &mut dst,
                self.ship.droid_background,
                null_mut(),
            );
            self.ship.frame_num = 0;
            self.ship.last_frame_time = SDL_GetTicks();
        }

        if droid_type != self.ship.last_droid_type || self.ship.droid_pics.is_null() {
            // we need to unpack the droid-pics into our local storage
            if self.ship.droid_pics.is_null().not() {
                SDL_FreeSurface(self.ship.droid_pics);
            }
            self.ship.droid_pics = null_mut();
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
            if IMG_isJPG(packed_portrait) != 0 {
                self.ship.droid_pics = SDL_DisplayFormat(tmp);
            } else {
                // else assume it's png ;)
                self.ship.droid_pics = SDL_DisplayFormatAlpha(tmp);
            }
            SDL_FreeSurface(tmp);
            sdl_rw_seek(packed_portrait, 0, libc::SEEK_SET);

            // do we have to scale the droid pics
            #[allow(clippy::float_cmp)]
            if self.global.game_config.scale != 1.0 {
                scale_pic(&mut self.ship.droid_pics, self.global.game_config.scale);
            }

            self.ship.last_droid_type = droid_type;
        }

        let droid_pics_ref = &*self.ship.droid_pics;
        let mut num_frames = droid_pics_ref.w / c_int::from(self.vars.portrait_rect.w);

        // sanity check
        if num_frames == 0 {
            warn!(
                "Only one frame found. Width droid-pics={}, Frame-width={}",
                droid_pics_ref.w, self.vars.portrait_rect.w,
            );
            num_frames = 1; // continue and hope for the best
        }

        let frame_duration = SDL_GetTicks() - self.ship.last_frame_time;

        if cycle_time != 0. && (frame_duration as f32 > 1000.0 * cycle_time / num_frames as f32) {
            need_new_frame = true;
            self.ship.frame_num += 1;
        }

        if self.ship.frame_num >= num_frames {
            self.ship.frame_num = 0;
        }

        if flags & (RESET | UPDATE) != 0 || need_new_frame {
            self.ship.src_rect.x = i16::try_from(self.ship.frame_num).unwrap()
                * i16::try_from(self.ship.src_rect.w).unwrap();

            SDL_UpperBlit(
                self.ship.droid_background,
                null_mut(),
                self.graphics.ne_screen,
                &mut dst,
            );
            SDL_UpperBlit(
                self.ship.droid_pics,
                &mut self.ship.src_rect,
                self.graphics.ne_screen,
                &mut dst,
            );

            SDL_UpdateRects(self.graphics.ne_screen, 1, &mut dst);

            self.ship.last_frame_time = SDL_GetTicks();
        }

        SDL_SetClipRect(self.graphics.ne_screen, null_mut());
    }

    /// display infopage page of droidtype
    ///
    /// if flags == UPDATE_ONLY : don't blit a new background&banner,
    ///                           only  update the text-regions
    ///
    ///  does update the screen: all if flags=0, text-rect if flags=UPDATE_ONLY
    pub unsafe fn show_droid_info(&mut self, droid_type: c_int, page: c_int, flags: c_int) {
        use std::io::Write;

        SDL_SetClipRect(self.graphics.ne_screen, null_mut());
        self.b_font.current_font = self.global.para_b_font;

        let lineskip =
            ((f64::from(font_height(&*self.b_font.current_font)) * TEXT_STRETCH) as f32) as i16;
        let lastline =
            self.vars.cons_header_rect.y + i16::try_from(self.vars.cons_header_rect.h).unwrap();
        self.ship.up_rect = SDL_Rect {
            x: self.vars.cons_header_rect.x,
            y: lastline - lineskip,
            w: 25,
            h: 13,
        };
        self.ship.down_rect = SDL_Rect {
            x: self.vars.cons_header_rect.x,
            y: (f32::from(lastline) - 0.5 * f32::from(lineskip)) as i16,
            w: 25,
            h: 13,
        };
        self.ship.left_rect = SDL_Rect {
            x: (f32::from(
                self.vars.cons_header_rect.x + i16::try_from(self.vars.cons_header_rect.w).unwrap(),
            ) - 1.5 * f32::from(lineskip)) as i16,
            y: (f32::from(lastline) - 0.9 * f32::from(lineskip)) as i16,
            w: 13,
            h: 25,
        };
        self.ship.right_rect = SDL_Rect {
            x: (f32::from(
                self.vars.cons_header_rect.x + i16::try_from(self.vars.cons_header_rect.w).unwrap(),
            ) - 1.0 * f32::from(lineskip)) as i16,
            y: (f32::from(lastline) - 0.9 * f32::from(lineskip)) as i16,
            w: 13,
            h: 25,
        };

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

        // if UPDATE_ONLY then the background has not been cleared, so we have do it
        // it for each menu-rect:
        if flags & i32::from(UPDATE_ONLY) != 0 {
            SDL_SetClipRect(self.graphics.ne_screen, &self.vars.cons_text_rect);
            SDL_UpperBlit(
                self.graphics.console_bg_pic2,
                null_mut(),
                self.graphics.ne_screen,
                null_mut(),
            );
            SDL_SetClipRect(self.graphics.ne_screen, &self.vars.cons_header_rect);
            SDL_UpperBlit(
                self.graphics.console_bg_pic2,
                null_mut(),
                self.graphics.ne_screen,
                null_mut(),
            );
            SDL_SetClipRect(self.graphics.ne_screen, null_mut());
        } else {
            // otherwise we just redraw the whole screen
            SDL_UpperBlit(
                self.graphics.console_bg_pic2,
                null_mut(),
                self.graphics.ne_screen,
                null_mut(),
            );
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
            self.vars.cons_text_rect.x.into(),
            self.vars.cons_text_rect.y.into(),
            &self.vars.cons_text_rect,
        );

        self.display_text(
            droid_name.as_mut_ptr() as *mut c_char,
            i32::from(self.vars.cons_header_rect.x) + i32::from(lineskip),
            (f32::from(lastline) - 0.9 * f32::from(lineskip)) as i32,
            null_mut(),
        );

        if show_arrows {
            if self.vars.me.ty > droid_type {
                SDL_UpperBlit(
                    self.graphics.arrow_up,
                    null_mut(),
                    self.graphics.ne_screen,
                    &mut self.ship.up_rect,
                );
            }

            if droid_type > 0 {
                SDL_UpperBlit(
                    self.graphics.arrow_down,
                    null_mut(),
                    self.graphics.ne_screen,
                    &mut self.ship.down_rect,
                );
            }

            if page > 0 {
                SDL_UpperBlit(
                    self.graphics.arrow_left,
                    null_mut(),
                    self.graphics.ne_screen,
                    &mut self.ship.left_rect,
                );
            }

            if page < 2 {
                SDL_UpperBlit(
                    self.graphics.arrow_right,
                    null_mut(),
                    self.graphics.ne_screen,
                    &mut self.ship.right_rect,
                );
            }
        }

        if flags & i32::from(UPDATE_ONLY) != 0 {
            SDL_UpdateRects(self.graphics.ne_screen, 1, &mut self.vars.cons_header_rect);
            SDL_UpdateRects(self.graphics.ne_screen, 1, &mut self.vars.cons_text_rect);
        } else {
            SDL_Flip(self.graphics.ne_screen);
        }
    }
}

impl Data {
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

        SDL_Flip(self.graphics.ne_screen);

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
            if SDL_GetTicks() - self.ship.enter_console_last_move_tick > wait_move_ticks {
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
                                (self.vars.cons_menu_rects[pos].x
                                    + i16::try_from(self.vars.cons_menu_rects[pos].w / 2).unwrap())
                                .try_into()
                                .unwrap(),
                                (self.vars.cons_menu_rects[pos].y
                                    + i16::try_from(self.vars.cons_menu_rects[pos].h / 2).unwrap())
                                .try_into()
                                .unwrap(),
                            );
                            self.update_input(); // this sets a new last_mouse_event
                            self.input.last_mouse_event = mousemove_buf; //... which we override.. ;)
                        }
                        self.move_menu_position_sound();
                        need_update = true;
                        self.ship.enter_console_last_move_tick = SDL_GetTicks();
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
                                (self.vars.cons_menu_rects[pos].x
                                    + i16::try_from(self.vars.cons_menu_rects[pos].w / 2).unwrap())
                                .try_into()
                                .unwrap(),
                                (self.vars.cons_menu_rects[pos].y
                                    + i16::try_from(self.vars.cons_menu_rects[pos].h / 2).unwrap())
                                .try_into()
                                .unwrap(),
                            );
                            self.update_input(); // this sets a new last_mouse_event
                            self.input.last_mouse_event = mousemove_buf; //... which we override.. ;)
                        }
                        self.move_menu_position_sound();
                        need_update = true;
                        self.ship.enter_console_last_move_tick = SDL_GetTicks();
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
                    SDL_Flip(self.graphics.ne_screen);
                }

                need_update = false;
            }
            if cfg!(target_os = "android") {
                SDL_Flip(self.graphics.ne_screen); // for responsive input on Android, we need to run this every cycle
            }

            SDL_Delay(1); // don't hog CPU
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
                SDL_GetTicks() - self.ship.great_droid_show_last_move_tick > wait_move_ticks;
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
                        self.ship.great_droid_show_last_move_tick = SDL_GetTicks();
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
                        self.ship.great_droid_show_last_move_tick = SDL_GetTicks();
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
                        self.ship.great_droid_show_last_move_tick = SDL_GetTicks();
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
                        self.ship.great_droid_show_last_move_tick = SDL_GetTicks();
                    }
                }
                _ => {}
            }

            SDL_Delay(1); // don't hog CPU
        }
    }

    /// This function should check if the mouse cursor is in the given Rectangle
    pub unsafe fn cursor_is_on_rect(&self, rect: &SDL_Rect) -> c_int {
        let user_center = self.get_user_center();
        let cur_pos = Point {
            x: self.input.input_axis.x + (i32::from(user_center.x) - 16),
            y: self.input.input_axis.y + (i32::from(user_center.y) - 16),
        };

        (cur_pos.x >= rect.x.into()
            && cur_pos.x <= i32::from(rect.x) + i32::from(rect.w)
            && cur_pos.y >= rect.y.into()
            && cur_pos.y <= i32::from(rect.y) + i32::from(rect.h))
        .into()
    }

    /// @Desc: show side-view of the ship, and hightlight the current
    ///        level + lift
    ///
    ///  if level==-1: don't highlight any level
    ///  if liftrow==-1: dont' highlight any liftrows
    pub unsafe fn show_lifts(&self, level: c_int, liftrow: c_int) {
        let lift_bg_color = SDL_Color {
            r: 0,
            g: 0,
            b: 0,
            unused: 0,
        }; /* black... */
        let xoffs: i16 = (self.vars.user_rect.w / 20).try_into().unwrap();
        let yoffs: i16 = (self.vars.user_rect.h / 5).try_into().unwrap();

        SDL_ShowCursor(SDL_DISABLE);
        // fill the user fenster with some color
        self.fill_rect(self.vars.user_rect, lift_bg_color);

        /* First blit ship "lights off" */
        let mut dst = self.vars.user_rect;
        SDL_SetClipRect(self.graphics.ne_screen, &dst);
        dst = self.vars.user_rect;
        dst.x += xoffs;
        dst.y += yoffs;
        SDL_UpperBlit(
            self.graphics.ship_off_pic,
            null_mut(),
            self.graphics.ne_screen,
            &mut dst,
        );

        if level >= 0 {
            for i in 0..self.main.cur_ship.num_level_rects[usize::try_from(level).unwrap()] {
                let mut src = self.main.cur_ship.level_rects[usize::try_from(level).unwrap()]
                    [usize::try_from(i).unwrap()];
                dst = src;
                dst.x += self.vars.user_rect.x + xoffs; /* offset respective to User-Rectangle */
                dst.y += self.vars.user_rect.y + yoffs;
                SDL_UpperBlit(
                    self.graphics.ship_on_pic,
                    &mut src,
                    self.graphics.ne_screen,
                    &mut dst,
                );
            }
        }

        if liftrow >= 0 {
            let mut src = self.main.cur_ship.lift_row_rect[usize::try_from(liftrow).unwrap()];
            dst = src;
            dst.x += self.vars.user_rect.x + xoffs; /* offset respective to User-Rectangle */
            dst.y += self.vars.user_rect.y + yoffs;
            SDL_UpperBlit(
                self.graphics.ship_on_pic,
                &mut src,
                self.graphics.ne_screen,
                &mut dst,
            );
        }

        SDL_Flip(self.graphics.ne_screen);
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
            SDL_SetClipRect(self.graphics.ne_screen, null_mut());
            SDL_UpperBlit(
                self.graphics.console_bg_pic1,
                null_mut(),
                self.graphics.ne_screen,
                null_mut(),
            );

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
                self.vars.cons_header_rect.x.into(),
                self.vars.cons_header_rect.y.into(),
                &self.vars.cons_header_rect,
            );

            write!(
                &mut menu_text[..],
                "Logout from console\n\nDroid info\n\nDeck map\n\nShip map\0"
            )
            .unwrap();
            self.display_text(
                menu_text.as_mut_ptr() as *mut c_char,
                self.vars.cons_text_rect.x.into(),
                c_int::from(self.vars.cons_text_rect.y) + 25,
                &self.vars.cons_text_rect,
            );
        } // only if not UPDATE_ONLY was required

        let mut src = SDL_Rect {
            x: i16::try_from(self.vars.cons_menu_rects[0].w).unwrap() * i16::try_from(pos).unwrap()
                + (2. * pos as f32 * self.global.game_config.scale) as i16,
            y: 0,
            w: self.vars.cons_menu_rect.w,
            h: 4 * self.vars.cons_menu_rect.h,
        };
        SDL_UpperBlit(
            self.graphics.console_pic,
            &mut src,
            self.graphics.ne_screen,
            &mut self.vars.cons_menu_rect,
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
            if SDL_GetTicks() - self.ship.enter_lift_last_move_tick > wait_move_ticks {
                match action {
                    MenuAction::CLICK => {
                        finished = true;
                        self.wait_for_all_keys_released();
                    }

                    MenuAction::UP | MenuAction::UP_WHEEL => {
                        self.ship.enter_lift_last_move_tick = SDL_GetTicks();
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
                        self.ship.enter_lift_last_move_tick = SDL_GetTicks();
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
            SDL_Delay(1); // don't hog CPU
        }

        // It might happen, that the influencer enters the elevator, but then decides to
        // come out on the same level where he has been before.  In this case of course there
        // is no need to reshuffle enemys or to reset influencers position.  Therefore, only
        // when a real level change has occured, we need to do real changes as below, where
        // we set the new level and set new position and initiate timers and all that...
        if cur_level != (*self.main.cur_level).levelnum {
            let mut array_num = 0;

            let mut tmp;
            while {
                tmp = self.main.cur_ship.all_levels[array_num];
                tmp.is_null().not()
            } {
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
