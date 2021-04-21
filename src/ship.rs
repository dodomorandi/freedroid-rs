use crate::{
    b_font::font_height,
    bullet::{delete_blast, delete_bullet},
    defs::{
        get_user_center, mouse_left_pressed_r, AlertNames, AssembleCombatWindowFlags,
        DisplayBannerFlags, MenuAction, Sound, Status, DROID_ROTATION_TIME, MAXBLASTS, MAXBULLETS,
        RESET, TEXT_STRETCH, UPDATE,
    },
    global::PARA_B_FONT,
    graphics::{
        clear_graph_mem, scale_pic, set_combat_scale_to, ARROW_CURSOR, ARROW_DOWN, ARROW_LEFT,
        ARROW_RIGHT, ARROW_UP, CONSOLE_BG_PIC1, CONSOLE_BG_PIC2, CONSOLE_PIC, CROSSHAIR_CURSOR,
        PACKED_PORTRAITS, SHIP_OFF_PIC, SHIP_ON_PIC, VID_BPP,
    },
    input::{
        update_input, wait_for_all_keys_released, wait_for_key_pressed, SDL_Delay, INPUT_AXIS,
        LAST_MOUSE_EVENT,
    },
    map::{get_current_lift, get_map_brick},
    menu::get_menu_action,
    misc::activate_conservative_frame_computation,
    sound::{
        enter_lift_sound, leave_lift_sound, menu_item_selected_sound, move_lift_sound,
        move_menu_position_sound, play_sound, switch_background_music_to,
    },
    structs::Point,
    vars::{
        BRAIN_NAMES, CLASSES, CLASS_NAMES, CONS_DROID_RECT, CONS_HEADER_RECT, CONS_MENU_RECT,
        CONS_MENU_RECTS, CONS_TEXT_RECT, DRIVE_NAMES, DRUIDMAP, FULL_USER_RECT, PORTRAIT_RECT,
        SENSOR_NAMES, USER_RECT, WEAPON_NAMES,
    },
    view::{display_banner, fill_rect},
    Data, ALERT_LEVEL, ALL_ENEMYS, CUR_LEVEL, CUR_SHIP, GAME_CONFIG, ME, NE_SCREEN, NUM_ENEMYS,
    SHOW_CURSOR,
};

use log::{error, warn};
use sdl::{
    event::ll::{SDL_DISABLE, SDL_ENABLE},
    ll::SDL_GetTicks,
    mouse::ll::{SDL_SetCursor, SDL_ShowCursor, SDL_WarpMouse},
    video::ll::{
        SDL_Color, SDL_CreateRGBSurface, SDL_DisplayFormat, SDL_DisplayFormatAlpha, SDL_Flip,
        SDL_FreeSurface, SDL_RWops, SDL_SetClipRect, SDL_Surface, SDL_UpdateRects, SDL_UpperBlit,
    },
    Rect,
};
use std::{
    convert::{TryFrom, TryInto},
    ffi::CStr,
    ops::Not,
    os::raw::{c_char, c_float, c_int},
    ptr::null_mut,
};

const UPDATE_ONLY: u8 = 0x01;

extern "C" {
    pub fn IMG_Load_RW(src: *mut SDL_RWops, freesrc: c_int) -> *mut SDL_Surface;
    pub fn IMG_isJPG(src: *mut SDL_RWops) -> c_int;
}

pub static mut DROID_BACKGROUND: *mut SDL_Surface = null_mut();
pub static mut DROID_PICS: *mut SDL_Surface = null_mut();
pub static mut UP_RECT: Rect = rect!(0, 0, 0, 0);
pub static mut DOWN_RECT: Rect = rect!(0, 0, 0, 0);
pub static mut LEFT_RECT: Rect = rect!(0, 0, 0, 0);
pub static mut RIGHT_RECT: Rect = rect!(0, 0, 0, 0);

#[inline]
pub unsafe fn sdl_rw_seek(ctx: *mut SDL_RWops, offset: c_int, whence: c_int) -> c_int {
    let seek: unsafe fn(*mut SDL_RWops, c_int, c_int) -> c_int = std::mem::transmute((*ctx).seek);
    seek(ctx, offset, whence)
}

pub unsafe fn free_droid_pics() {
    SDL_FreeSurface(DROID_PICS);
    SDL_FreeSurface(DROID_BACKGROUND);
}

/// do all alert-related agitations: alert-sirens and alert-lights
pub unsafe fn alert_level_warning() {
    const SIREN_WAIT: f32 = 2.5;

    static mut LAST_SIREN: u32 = 0;

    use AlertNames::*;
    match AlertNames::try_from(ALERT_LEVEL).ok() {
        Some(Green) => {}
        Some(Yellow) | Some(Amber) | Some(Red) => {
            if SDL_GetTicks() - LAST_SIREN > (SIREN_WAIT * 1000.0 / (ALERT_LEVEL as f32)) as u32 {
                // higher alert-> faster sirens!
                play_sound(Sound::Alert as c_int);
                LAST_SIREN = SDL_GetTicks();
            }
        }
        Some(Last) | None => {
            warn!(
                "illegal AlertLevel = {} > {}.. something's gone wrong!!\n",
                ALERT_LEVEL,
                AlertNames::Red as c_int
            );
        }
    }

    // so much to the sirens, now make sure the alert-tiles are updated correctly:
    let posx = (*CUR_LEVEL).alerts[0].x;
    let posy = (*CUR_LEVEL).alerts[0].y;
    if posx == -1 {
        // no alerts here...
        return;
    }

    let cur_alert = AlertNames::try_from(ALERT_LEVEL).unwrap();

    // check if alert-tiles are up-to-date
    if get_map_brick(&*CUR_LEVEL, posx.into(), posy.into()) == cur_alert as u8 {
        // ok
        return;
    }

    for alert in &mut (*CUR_LEVEL).alerts {
        let posx = alert.x;
        let posy = alert.y;
        if posx == -1 {
            break;
        }

        *(*CUR_LEVEL).map[usize::try_from(posy).unwrap()].add(usize::try_from(posx).unwrap()) =
            cur_alert as i8;
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
    mut dst: Rect,
    droid_type: c_int,
    cycle_time: c_float,
    flags: c_int,
) {
    static mut FRAME_NUM: c_int = 0;
    static mut LAST_DROID_TYPE: c_int = -1;
    static mut LAST_FRAME_TIME: u32 = 0;
    static mut SRC_RECT: Rect = Rect {
        x: 0,
        y: 0,
        h: 0,
        w: 0,
    };
    let mut need_new_frame = false;

    SDL_SetClipRect(NE_SCREEN, &dst);

    if DROID_BACKGROUND.is_null() {
        // first call
        let tmp = SDL_CreateRGBSurface(0, dst.w.into(), dst.h.into(), VID_BPP, 0, 0, 0, 0);
        DROID_BACKGROUND = SDL_DisplayFormat(tmp);
        SDL_FreeSurface(tmp);
        SDL_UpperBlit(NE_SCREEN, &mut dst, DROID_BACKGROUND, null_mut());
        SRC_RECT = PORTRAIT_RECT;
    }

    if flags & RESET != 0 {
        SDL_UpperBlit(NE_SCREEN, &mut dst, DROID_BACKGROUND, null_mut());
        FRAME_NUM = 0;
        LAST_FRAME_TIME = SDL_GetTicks();
    }

    if droid_type != LAST_DROID_TYPE || DROID_PICS.is_null() {
        // we need to unpack the droid-pics into our local storage
        if DROID_PICS.is_null().not() {
            SDL_FreeSurface(DROID_PICS);
        }
        DROID_PICS = null_mut();
        let packed_portrait = PACKED_PORTRAITS[usize::try_from(droid_type).unwrap()];
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
            DROID_PICS = SDL_DisplayFormat(tmp);
        } else {
            // else assume it's png ;)
            DROID_PICS = SDL_DisplayFormatAlpha(tmp);
        }
        SDL_FreeSurface(tmp);
        sdl_rw_seek(packed_portrait, 0, libc::SEEK_SET);

        // do we have to scale the droid pics
        #[allow(clippy::float_cmp)]
        if GAME_CONFIG.scale != 1.0 {
            scale_pic(&mut DROID_PICS, GAME_CONFIG.scale);
        }

        LAST_DROID_TYPE = droid_type;
    }

    let droid_pics_ref = &*DROID_PICS;
    let mut num_frames = droid_pics_ref.w / c_int::from(PORTRAIT_RECT.w);

    // sanity check
    if num_frames == 0 {
        warn!(
            "Only one frame found. Width droid-pics={}, Frame-width={}",
            droid_pics_ref.w, PORTRAIT_RECT.w,
        );
        num_frames = 1; // continue and hope for the best
    }

    let frame_duration = SDL_GetTicks() - LAST_FRAME_TIME;

    if cycle_time != 0. && (frame_duration as f32 > 1000.0 * cycle_time / num_frames as f32) {
        need_new_frame = true;
        FRAME_NUM += 1;
    }

    if FRAME_NUM >= num_frames {
        FRAME_NUM = 0;
    }

    if flags & (RESET | UPDATE) != 0 || need_new_frame {
        SRC_RECT.x = i16::try_from(FRAME_NUM).unwrap() * i16::try_from(SRC_RECT.w).unwrap();

        SDL_UpperBlit(DROID_BACKGROUND, null_mut(), NE_SCREEN, &mut dst);
        SDL_UpperBlit(DROID_PICS, &mut SRC_RECT, NE_SCREEN, &mut dst);

        SDL_UpdateRects(NE_SCREEN, 1, &mut dst);

        LAST_FRAME_TIME = SDL_GetTicks();
    }

    SDL_SetClipRect(NE_SCREEN, null_mut());
}

impl Data {
    /// display infopage page of droidtype
    ///
    /// if flags == UPDATE_ONLY : don't blit a new background&banner,
    ///                           only  update the text-regions
    ///
    ///  does update the screen: all if flags=0, text-rect if flags=UPDATE_ONLY
    pub unsafe fn show_droid_info(&mut self, droid_type: c_int, page: c_int, flags: c_int) {
        use std::io::Write;

        SDL_SetClipRect(NE_SCREEN, null_mut());
        self.b_font.current_font = PARA_B_FONT;

        let lineskip =
            ((f64::from(font_height(&*self.b_font.current_font)) * TEXT_STRETCH) as f32) as i16;
        let lastline = CONS_HEADER_RECT.y + i16::try_from(CONS_HEADER_RECT.h).unwrap();
        UP_RECT = Rect {
            x: CONS_HEADER_RECT.x,
            y: lastline - lineskip,
            w: 25,
            h: 13,
        };
        DOWN_RECT = Rect {
            x: CONS_HEADER_RECT.x,
            y: (f32::from(lastline) - 0.5 * f32::from(lineskip)) as i16,
            w: 25,
            h: 13,
        };
        LEFT_RECT = Rect {
            x: (f32::from(CONS_HEADER_RECT.x + i16::try_from(CONS_HEADER_RECT.w).unwrap())
                - 1.5 * f32::from(lineskip)) as i16,
            y: (f32::from(lastline) - 0.9 * f32::from(lineskip)) as i16,
            w: 13,
            h: 25,
        };
        RIGHT_RECT = Rect {
            x: (f32::from(CONS_HEADER_RECT.x + i16::try_from(CONS_HEADER_RECT.w).unwrap())
                - 1.0 * f32::from(lineskip)) as i16,
            y: (f32::from(lastline) - 0.9 * f32::from(lineskip)) as i16,
            w: 13,
            h: 25,
        };

        let mut droid_name = [0u8; 80];
        let droid = &*DRUIDMAP.add(usize::try_from(droid_type).unwrap());
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
            SDL_SetClipRect(NE_SCREEN, &CONS_TEXT_RECT);
            SDL_UpperBlit(CONSOLE_BG_PIC2, null_mut(), NE_SCREEN, null_mut());
            SDL_SetClipRect(NE_SCREEN, &CONS_HEADER_RECT);
            SDL_UpperBlit(CONSOLE_BG_PIC2, null_mut(), NE_SCREEN, null_mut());
            SDL_SetClipRect(NE_SCREEN, null_mut());
        } else {
            // otherwise we just redraw the whole screen
            SDL_UpperBlit(CONSOLE_BG_PIC2, null_mut(), NE_SCREEN, null_mut());
            display_banner(
                null_mut(),
                null_mut(),
                (DisplayBannerFlags::NO_SDL_UPDATE | DisplayBannerFlags::FORCE_UPDATE)
                    .bits()
                    .into(),
            );
        }

        self.display_text(
            info_text.as_mut_ptr() as *mut c_char,
            CONS_TEXT_RECT.x.into(),
            CONS_TEXT_RECT.y.into(),
            &CONS_TEXT_RECT,
        );

        self.display_text(
            droid_name.as_mut_ptr() as *mut c_char,
            i32::from(CONS_HEADER_RECT.x) + i32::from(lineskip),
            (f32::from(lastline) - 0.9 * f32::from(lineskip)) as i32,
            null_mut(),
        );

        if show_arrows {
            if ME.ty > droid_type {
                SDL_UpperBlit(ARROW_UP, null_mut(), NE_SCREEN, &mut UP_RECT);
            }

            if droid_type > 0 {
                SDL_UpperBlit(ARROW_DOWN, null_mut(), NE_SCREEN, &mut DOWN_RECT);
            }

            if page > 0 {
                SDL_UpperBlit(ARROW_LEFT, null_mut(), NE_SCREEN, &mut LEFT_RECT);
            }

            if page < 2 {
                SDL_UpperBlit(ARROW_RIGHT, null_mut(), NE_SCREEN, &mut RIGHT_RECT);
            }
        }

        if flags & i32::from(UPDATE_ONLY) != 0 {
            SDL_UpdateRects(NE_SCREEN, 1, &mut CONS_HEADER_RECT);
            SDL_UpdateRects(NE_SCREEN, 1, &mut CONS_TEXT_RECT);
        } else {
            SDL_Flip(NE_SCREEN);
        }
    }
}

impl Data {
    /// Displays the concept view of deck
    ///
    /// Note: we no longer wait here for a key-press, but return
    /// immediately
    pub unsafe fn show_deck_map(&mut self) {
        let tmp = ME.pos;

        let cur_level = &*CUR_LEVEL;
        ME.pos.x = (cur_level.xlen / 2) as f32;
        ME.pos.y = (cur_level.ylen / 2) as f32;

        SDL_ShowCursor(SDL_DISABLE);

        set_combat_scale_to(0.25);

        self.assemble_combat_picture(
            (AssembleCombatWindowFlags::ONLY_SHOW_MAP | AssembleCombatWindowFlags::SHOW_FULL_MAP)
                .bits()
                .into(),
        );

        SDL_Flip(NE_SCREEN);

        ME.pos = tmp;

        wait_for_key_pressed();

        set_combat_scale_to(1.0);
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
        activate_conservative_frame_computation();

        let tmp_rect = USER_RECT;
        USER_RECT = FULL_USER_RECT;

        wait_for_all_keys_released();

        ME.status = Status::Console as c_int;

        if cfg!(target_os = "android") {
            SHOW_CURSOR = false;
        }

        SDL_SetCursor(ARROW_CURSOR);

        self.b_font.current_font = PARA_B_FONT;

        let mut pos = 0; // starting menu position
        self.paint_console_menu(c_int::try_from(pos).unwrap(), 0);

        let wait_move_ticks: u32 = 100;
        static mut LAST_MOVE_TICK: u32 = 0;
        let mut finished = false;
        let mut need_update = true;
        while !finished {
            if SHOW_CURSOR {
                SDL_ShowCursor(SDL_ENABLE);
            } else {
                SDL_ShowCursor(SDL_DISABLE);
            }

            // check if the mouse-cursor is on any of the console-menu points
            for (i, rect) in CONS_MENU_RECTS.iter_mut().enumerate() {
                if SHOW_CURSOR && pos != i && cursor_is_on_rect(rect) != 0 {
                    move_menu_position_sound();
                    pos = i;
                    need_update = true;
                }
            }
            let action = get_menu_action(250);
            if SDL_GetTicks() - LAST_MOVE_TICK > wait_move_ticks {
                match action {
                    MenuAction::BACK => {
                        finished = true;
                        wait_for_all_keys_released();
                    }

                    MenuAction::UP => {
                        if pos > 0 {
                            pos -= 1;
                        } else {
                            pos = 3;
                        }
                        // when warping the mouse-cursor: don't count that as a mouse-activity
                        // this is a dirty hack, but that should be enough for here...
                        if SHOW_CURSOR {
                            let mousemove_buf = LAST_MOUSE_EVENT;
                            SDL_WarpMouse(
                                (CONS_MENU_RECTS[pos].x
                                    + i16::try_from(CONS_MENU_RECTS[pos].w / 2).unwrap())
                                .try_into()
                                .unwrap(),
                                (CONS_MENU_RECTS[pos].y
                                    + i16::try_from(CONS_MENU_RECTS[pos].h / 2).unwrap())
                                .try_into()
                                .unwrap(),
                            );
                            update_input(); // this sets a new last_mouse_event
                            LAST_MOUSE_EVENT = mousemove_buf; //... which we override.. ;)
                        }
                        move_menu_position_sound();
                        need_update = true;
                        LAST_MOVE_TICK = SDL_GetTicks();
                    }

                    MenuAction::DOWN => {
                        if pos < 3 {
                            pos += 1;
                        } else {
                            pos = 0;
                        }
                        // when warping the mouse-cursor: don't count that as a mouse-activity
                        // this is a dirty hack, but that should be enough for here...
                        if SHOW_CURSOR {
                            let mousemove_buf = LAST_MOUSE_EVENT;
                            SDL_WarpMouse(
                                (CONS_MENU_RECTS[pos].x
                                    + i16::try_from(CONS_MENU_RECTS[pos].w / 2).unwrap())
                                .try_into()
                                .unwrap(),
                                (CONS_MENU_RECTS[pos].y
                                    + i16::try_from(CONS_MENU_RECTS[pos].h / 2).unwrap())
                                .try_into()
                                .unwrap(),
                            );
                            update_input(); // this sets a new last_mouse_event
                            LAST_MOUSE_EVENT = mousemove_buf; //... which we override.. ;)
                        }
                        move_menu_position_sound();
                        need_update = true;
                        LAST_MOVE_TICK = SDL_GetTicks();
                    }

                    MenuAction::CLICK => {
                        menu_item_selected_sound();
                        wait_for_all_keys_released();
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
                                clear_graph_mem();
                                display_banner(
                                    null_mut(),
                                    null_mut(),
                                    DisplayBannerFlags::FORCE_UPDATE.bits().into(),
                                );
                                self.show_deck_map();
                                self.paint_console_menu(pos.try_into().unwrap(), 0);
                            }
                            3 => {
                                clear_graph_mem();
                                display_banner(
                                    null_mut(),
                                    null_mut(),
                                    DisplayBannerFlags::FORCE_UPDATE.bits().into(),
                                );
                                show_lifts((*CUR_LEVEL).levelnum, -1);
                                wait_for_key_pressed();
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
                    SDL_Flip(NE_SCREEN);
                }

                need_update = false;
            }
            if cfg!(target_os = "android") {
                SDL_Flip(NE_SCREEN); // for responsive input on Android, we need to run this every cycle
            }

            SDL_Delay(1); // don't hog CPU
        }

        USER_RECT = tmp_rect;

        ME.status = Status::Mobile as c_int;

        clear_graph_mem();

        SDL_SetCursor(CROSSHAIR_CURSOR);
        if !SHOW_CURSOR {
            SDL_ShowCursor(SDL_DISABLE);
        }
    }

    /// This function does the robot show when the user has selected robot
    /// show from the console menu.
    pub unsafe fn great_druid_show(&mut self) {
        let mut finished = false;

        let mut droidtype = ME.ty;
        let mut page = 0;

        self.show_droid_info(droidtype, page, 0);
        show_droid_portrait(CONS_DROID_RECT, droidtype, 0.0, UPDATE | RESET);

        wait_for_all_keys_released();
        let mut need_update = true;
        let wait_move_ticks: u32 = 100;
        static mut LAST_MOVE_TICK: u32 = 0;

        while !finished {
            show_droid_portrait(CONS_DROID_RECT, droidtype, DROID_ROTATION_TIME, 0);

            if SHOW_CURSOR {
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
            if mouse_left_pressed_r() {
                if cursor_is_on_rect(&LEFT_RECT) != 0 {
                    action = MenuAction::LEFT;
                } else if cursor_is_on_rect(&RIGHT_RECT) != 0 {
                    action = MenuAction::RIGHT;
                } else if cursor_is_on_rect(&UP_RECT) != 0 {
                    action = MenuAction::UP;
                } else if cursor_is_on_rect(&DOWN_RECT) != 0 {
                    action = MenuAction::DOWN;
                }
            } else {
                action = get_menu_action(250);
            }

            let time_for_move = SDL_GetTicks() - LAST_MOVE_TICK > wait_move_ticks;
            match action {
                MenuAction::BACK | MenuAction::CLICK => {
                    finished = true;
                    wait_for_all_keys_released();
                }

                MenuAction::UP => {
                    if !time_for_move {
                        continue;
                    }

                    if droidtype < ME.ty {
                        move_menu_position_sound();
                        droidtype += 1;
                        need_update = true;
                        LAST_MOVE_TICK = SDL_GetTicks();
                    }
                }

                MenuAction::DOWN => {
                    if !time_for_move {
                        continue;
                    }

                    if droidtype > 0 {
                        move_menu_position_sound();
                        droidtype -= 1;
                        need_update = true;
                        LAST_MOVE_TICK = SDL_GetTicks();
                    }
                }

                MenuAction::RIGHT => {
                    if !time_for_move {
                        continue;
                    }

                    if page < 2 {
                        move_menu_position_sound();
                        page += 1;
                        need_update = true;
                        LAST_MOVE_TICK = SDL_GetTicks();
                    }
                }

                MenuAction::LEFT => {
                    if !time_for_move {
                        continue;
                    }

                    if page > 0 {
                        move_menu_position_sound();
                        page -= 1;
                        need_update = true;
                        LAST_MOVE_TICK = SDL_GetTicks();
                    }
                }
                _ => {}
            }

            SDL_Delay(1); // don't hog CPU
        }
    }
}

/// This function should check if the mouse cursor is in the given Rectangle
pub unsafe fn cursor_is_on_rect(rect: &Rect) -> c_int {
    let user_center = get_user_center();
    let cur_pos = Point {
        x: INPUT_AXIS.x + (i32::from(user_center.x) - 16),
        y: INPUT_AXIS.y + (i32::from(user_center.y) - 16),
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
pub unsafe fn show_lifts(level: c_int, liftrow: c_int) {
    let lift_bg_color = SDL_Color {
        r: 0,
        g: 0,
        b: 0,
        unused: 0,
    }; /* black... */
    let xoffs: i16 = (USER_RECT.w / 20).try_into().unwrap();
    let yoffs: i16 = (USER_RECT.h / 5).try_into().unwrap();

    SDL_ShowCursor(SDL_DISABLE);
    // fill the user fenster with some color
    fill_rect(USER_RECT, lift_bg_color);

    /* First blit ship "lights off" */
    let mut dst = USER_RECT;
    SDL_SetClipRect(NE_SCREEN, &dst);
    dst = USER_RECT;
    dst.x += xoffs;
    dst.y += yoffs;
    SDL_UpperBlit(SHIP_OFF_PIC, null_mut(), NE_SCREEN, &mut dst);

    if level >= 0 {
        for i in 0..CUR_SHIP.num_level_rects[usize::try_from(level).unwrap()] {
            let mut src =
                CUR_SHIP.level_rects[usize::try_from(level).unwrap()][usize::try_from(i).unwrap()];
            dst = src;
            dst.x += USER_RECT.x + xoffs; /* offset respective to User-Rectangle */
            dst.y += USER_RECT.y + yoffs;
            SDL_UpperBlit(SHIP_ON_PIC, &mut src, NE_SCREEN, &mut dst);
        }
    }

    if liftrow >= 0 {
        let mut src = CUR_SHIP.lift_row_rect[usize::try_from(liftrow).unwrap()];
        dst = src;
        dst.x += USER_RECT.x + xoffs; /* offset respective to User-Rectangle */
        dst.y += USER_RECT.y + yoffs;
        SDL_UpperBlit(SHIP_ON_PIC, &mut src, NE_SCREEN, &mut dst);
    }

    SDL_Flip(NE_SCREEN);
}

impl Data {
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
            clear_graph_mem();
            SDL_SetClipRect(NE_SCREEN, null_mut());
            SDL_UpperBlit(CONSOLE_BG_PIC1, null_mut(), NE_SCREEN, null_mut());

            display_banner(
                null_mut(),
                null_mut(),
                DisplayBannerFlags::FORCE_UPDATE.bits().into(),
            );

            write!(
                &mut menu_text[..],
                "Area : {}\nDeck : {}    Alert: {}\0",
                CStr::from_ptr(CUR_SHIP.area_name.as_ptr())
                    .to_str()
                    .unwrap(),
                CStr::from_ptr((*CUR_LEVEL).levelname).to_str().unwrap(),
                AlertNames::try_from(ALERT_LEVEL).unwrap().to_str(),
            )
            .unwrap();
            self.display_text(
                menu_text.as_mut_ptr() as *mut c_char,
                CONS_HEADER_RECT.x.into(),
                CONS_HEADER_RECT.y.into(),
                &CONS_HEADER_RECT,
            );

            write!(
                &mut menu_text[..],
                "Logout from console\n\nDroid info\n\nDeck map\n\nShip map\0"
            )
            .unwrap();
            self.display_text(
                menu_text.as_mut_ptr() as *mut c_char,
                CONS_TEXT_RECT.x.into(),
                c_int::from(CONS_TEXT_RECT.y) + 25,
                &CONS_TEXT_RECT,
            );
        } // only if not UPDATE_ONLY was required

        let mut src = Rect {
            x: i16::try_from(CONS_MENU_RECTS[0].w).unwrap() * i16::try_from(pos).unwrap()
                + (2. * pos as f32 * GAME_CONFIG.scale) as i16,
            y: 0,
            w: CONS_MENU_RECT.w,
            h: 4 * CONS_MENU_RECT.h,
        };
        SDL_UpperBlit(CONSOLE_PIC, &mut src, NE_SCREEN, &mut CONS_MENU_RECT);
    }
}

/// does all the work when we enter a lift
pub unsafe fn enter_lift() {
    /* Prevent distortion of framerate by the delay coming from
     * the time spend in the menu. */
    activate_conservative_frame_computation();

    /* make sure to release the fire-key */
    wait_for_all_keys_released();

    /* Prevent the influ from coming out of the lift in transfer mode
     * by turning off transfer mode as soon as the influ enters the lift */
    ME.status = Status::Elevator as c_int;

    SDL_ShowCursor(SDL_DISABLE);

    let mut cur_level = (*CUR_LEVEL).levelnum;

    let cur_lift = get_current_lift();
    if cur_lift == -1 {
        error!("Lift out of order, I'm so sorry !");
        return;
    }
    let mut cur_lift: usize = cur_lift.try_into().unwrap();

    enter_lift_sound();
    switch_background_music_to(null_mut()); // turn off Bg music

    let mut up_lift = CUR_SHIP.all_lifts[cur_lift].up;
    let mut down_lift = CUR_SHIP.all_lifts[cur_lift].down;

    let liftrow = CUR_SHIP.all_lifts[cur_lift].lift_row;

    // clear the whole screen
    clear_graph_mem();
    display_banner(
        null_mut(),
        null_mut(),
        DisplayBannerFlags::FORCE_UPDATE.bits().into(),
    );

    let wait_move_ticks: u32 = 100;
    static mut LAST_MOVE_TICK: u32 = 0;
    let mut finished = false;
    while !finished {
        show_lifts(cur_level, liftrow);

        let action = get_menu_action(500);
        if SDL_GetTicks() - LAST_MOVE_TICK > wait_move_ticks {
            match action {
                MenuAction::CLICK => {
                    finished = true;
                    wait_for_all_keys_released();
                }

                MenuAction::UP | MenuAction::UP_WHEEL => {
                    LAST_MOVE_TICK = SDL_GetTicks();
                    if up_lift != -1 {
                        if CUR_SHIP.all_lifts[usize::try_from(up_lift).unwrap()].x == 99 {
                            error!("Lift out of order, so sorry ..");
                        } else {
                            down_lift = cur_lift.try_into().unwrap();
                            cur_lift = up_lift.try_into().unwrap();
                            cur_level = CUR_SHIP.all_lifts[cur_lift].level;
                            up_lift = CUR_SHIP.all_lifts[cur_lift].up;
                            show_lifts(cur_level, liftrow);
                            move_lift_sound();
                        }
                    }
                }

                MenuAction::DOWN | MenuAction::DOWN_WHEEL => {
                    LAST_MOVE_TICK = SDL_GetTicks();
                    if down_lift != -1 {
                        if CUR_SHIP.all_lifts[usize::try_from(down_lift).unwrap()].x == 99 {
                            error!("Lift Out of order, so sorry ..");
                        } else {
                            up_lift = cur_lift.try_into().unwrap();
                            cur_lift = down_lift.try_into().unwrap();
                            cur_level = CUR_SHIP.all_lifts[cur_lift].level;
                            down_lift = CUR_SHIP.all_lifts[cur_lift].down;
                            show_lifts(cur_level, liftrow);
                            move_lift_sound();
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
    if cur_level != (*CUR_LEVEL).levelnum {
        let mut array_num = 0;

        let mut tmp;
        while {
            tmp = CUR_SHIP.all_levels[array_num];
            tmp.is_null().not()
        } {
            if (*tmp).levelnum == cur_level {
                break;
            } else {
                array_num += 1;
            }
        }

        CUR_LEVEL = CUR_SHIP.all_levels[array_num];

        // set the position of the influencer to the correct locatiohn
        ME.pos.x = CUR_SHIP.all_lifts[cur_lift].x as f32;
        ME.pos.y = CUR_SHIP.all_lifts[cur_lift].y as f32;

        for i in 0..c_int::try_from(MAXBLASTS).unwrap() {
            delete_blast(i);
        }
        for i in 0..c_int::try_from(MAXBULLETS).unwrap() {
            delete_bullet(i);
        }
    }

    let cur_level = &*CUR_LEVEL;
    leave_lift_sound();
    switch_background_music_to(cur_level.background_song_name);
    clear_graph_mem();
    display_banner(
        null_mut(),
        null_mut(),
        DisplayBannerFlags::FORCE_UPDATE.bits().into(),
    );

    ME.status = Status::Mobile as c_int;
    ME.text_visible_time = 0.;
    ME.text_to_be_displayed = cur_level.level_enter_comment;
}

pub unsafe fn level_empty() -> c_int {
    let cur_level = &*CUR_LEVEL;
    if cur_level.empty != 0 {
        return true.into();
    }

    let levelnum = cur_level.levelnum;

    ALL_ENEMYS[0..usize::try_from(NUM_ENEMYS).unwrap()]
        .iter()
        .find(|enemy| {
            enemy.levelnum == levelnum
                && enemy.status != Status::Out as c_int
                && enemy.status != Status::Terminated as c_int
        })
        .is_none()
        .into()
}
