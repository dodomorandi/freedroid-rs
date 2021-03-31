use crate::{
    b_font::{FontHeight, GetCurrentFont, Para_BFont, SetCurrentFont},
    bullet::{DeleteBlast, DeleteBullet},
    curShip,
    defs::{
        get_user_center, AlertNames, AssembleCombatWindowFlags, DisplayBannerFlags, MenuAction,
        MouseLeftPressedR, Sound, Status, DROID_ROTATION_TIME, MAXBLASTS, MAXBULLETS, RESET,
        TEXT_STRETCH, UPDATE,
    },
    global::Druidmap,
    graphics::{
        arrow_cursor, arrow_down, arrow_left, arrow_right, arrow_up, console_bg_pic1,
        console_bg_pic2, console_pic, crosshair_cursor, packed_portraits, ship_off_pic,
        ship_on_pic, vid_bpp, ClearGraphMem, ScalePic, SetCombatScaleTo,
    },
    input::{
        input_axis, last_mouse_event, update_input, wait_for_all_keys_released,
        wait_for_key_pressed, SDL_Delay,
    },
    map::{GetCurrentLift, GetMapBrick},
    menu::getMenuAction,
    misc::Activate_Conservative_Frame_Computation,
    ne_screen, show_cursor,
    sound::{
        EnterLiftSound, LeaveLiftSound, MenuItemSelectedSound, MoveLiftSound,
        MoveMenuPositionSound, Play_Sound, Switch_Background_Music_To,
    },
    structs::{Level, Point},
    text::DisplayText,
    vars::{
        Cons_Droid_Rect, Cons_Header_Rect, Cons_Menu_Rect, Cons_Menu_Rects, Cons_Text_Rect,
        Full_User_Rect, Portrait_Rect, User_Rect, BRAIN_NAMES, CLASSES, CLASS_NAMES, DRIVE_NAMES,
        SENSOR_NAMES, WEAPON_NAMES,
    },
    view::{Assemble_Combat_Picture, DisplayBanner, Fill_Rect},
    AlertLevel, AllEnemys, CurLevel, GameConfig, Me, NumEnemys,
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

    pub static mut droid_background: *mut SDL_Surface;
    pub static mut droid_pics: *mut SDL_Surface;
    pub static mut up_rect: Rect;
    pub static mut down_rect: Rect;
    pub static mut left_rect: Rect;
    pub static mut right_rect: Rect;
}

#[inline]
pub unsafe fn sdl_rw_seek(ctx: *mut SDL_RWops, offset: c_int, whence: c_int) -> c_int {
    let seek: unsafe extern "C" fn(*mut SDL_RWops, c_int, c_int) -> c_int =
        std::mem::transmute((*ctx).seek);
    seek(ctx, offset, whence)
}

#[no_mangle]
pub unsafe extern "C" fn FreeDroidPics() {
    SDL_FreeSurface(droid_pics);
    SDL_FreeSurface(droid_background);
}

/// do all alert-related agitations: alert-sirens and alert-lights
#[no_mangle]
pub unsafe extern "C" fn AlertLevelWarning() {
    const SIREN_WAIT: f32 = 2.5;

    static mut LAST_SIREN: u32 = 0;

    use AlertNames::*;
    match AlertNames::try_from(AlertLevel).ok() {
        Some(Green) => {}
        Some(Yellow) | Some(Amber) | Some(Red) => {
            if SDL_GetTicks() - LAST_SIREN > (SIREN_WAIT * 1000.0 / (AlertLevel as f32)) as u32 {
                // higher alert-> faster sirens!
                Play_Sound(Sound::Alert as c_int);
                LAST_SIREN = SDL_GetTicks();
            }
        }
        Some(Last) | None => {
            warn!(
                "illegal AlertLevel = {} > {}.. something's gone wrong!!\n",
                AlertLevel,
                AlertNames::Red as c_int
            );
        }
    }

    // so much to the sirens, now make sure the alert-tiles are updated correctly:
    let posx = (*CurLevel).alerts[0].x;
    let posy = (*CurLevel).alerts[0].y;
    if posx == -1 {
        // no alerts here...
        return;
    }

    let cur_alert = AlertNames::try_from(AlertLevel).unwrap();

    // check if alert-tiles are up-to-date
    if GetMapBrick(&*CurLevel, posx.into(), posy.into()) == cur_alert as u8 {
        // ok
        return;
    }

    for alert in &mut (*CurLevel).alerts {
        let posx = alert.x;
        let posy = alert.y;
        if posx == -1 {
            break;
        }

        *(*CurLevel).map[usize::try_from(posy).unwrap()].add(usize::try_from(posx).unwrap()) =
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
#[no_mangle]
pub unsafe extern "C" fn show_droid_portrait(
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

    SDL_SetClipRect(ne_screen, &dst);

    if droid_background.is_null() {
        // first call
        let tmp = SDL_CreateRGBSurface(0, dst.w.into(), dst.h.into(), vid_bpp, 0, 0, 0, 0);
        droid_background = SDL_DisplayFormat(tmp);
        SDL_FreeSurface(tmp);
        SDL_UpperBlit(ne_screen, &mut dst, droid_background, null_mut());
        SRC_RECT = Portrait_Rect;
    }

    if flags & RESET != 0 {
        SDL_UpperBlit(ne_screen, &mut dst, droid_background, null_mut());
        FRAME_NUM = 0;
        LAST_FRAME_TIME = SDL_GetTicks();
    }

    if droid_type != LAST_DROID_TYPE || droid_pics.is_null() {
        // we need to unpack the droid-pics into our local storage
        if droid_pics.is_null().not() {
            SDL_FreeSurface(droid_pics);
        }
        droid_pics = null_mut();
        let packed_portrait = packed_portraits[usize::try_from(droid_type).unwrap()];
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
            droid_pics = SDL_DisplayFormat(tmp);
        } else {
            // else assume it's png ;)
            droid_pics = SDL_DisplayFormatAlpha(tmp);
        }
        SDL_FreeSurface(tmp);
        sdl_rw_seek(packed_portrait, 0, libc::SEEK_SET);

        // do we have to scale the droid pics
        #[allow(clippy::float_cmp)]
        if GameConfig.scale != 1.0 {
            ScalePic(&mut droid_pics, GameConfig.scale);
        }

        LAST_DROID_TYPE = droid_type;
    }

    let droid_pics_ref = &*droid_pics;
    let mut num_frames = droid_pics_ref.w / c_int::from(Portrait_Rect.w);

    // sanity check
    if num_frames == 0 {
        warn!(
            "Only one frame found. Width droid-pics={}, Frame-width={}",
            droid_pics_ref.w, Portrait_Rect.w,
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

        SDL_UpperBlit(droid_background, null_mut(), ne_screen, &mut dst);
        SDL_UpperBlit(droid_pics, &mut SRC_RECT, ne_screen, &mut dst);

        SDL_UpdateRects(ne_screen, 1, &mut dst);

        LAST_FRAME_TIME = SDL_GetTicks();
    }

    SDL_SetClipRect(ne_screen, null_mut());
}

/// display infopage page of droidtype
///
/// if flags == UPDATE_ONLY : don't blit a new background&banner,
///                           only  update the text-regions
///
///  does update the screen: all if flags=0, text-rect if flags=UPDATE_ONLY
#[no_mangle]
pub unsafe extern "C" fn show_droid_info(droid_type: c_int, page: c_int, flags: c_int) {
    use std::io::Write;

    SDL_SetClipRect(ne_screen, null_mut());
    SetCurrentFont(Para_BFont);

    let lineskip = ((f64::from(FontHeight(&*GetCurrentFont())) * TEXT_STRETCH) as f32) as i16;
    let lastline = Cons_Header_Rect.y + i16::try_from(Cons_Header_Rect.h).unwrap();
    up_rect = Rect {
        x: Cons_Header_Rect.x,
        y: lastline - lineskip,
        w: 25,
        h: 13,
    };
    down_rect = Rect {
        x: Cons_Header_Rect.x,
        y: (f32::from(lastline) - 0.5 * f32::from(lineskip)) as i16,
        w: 25,
        h: 13,
    };
    left_rect = Rect {
        x: (f32::from(Cons_Header_Rect.x + i16::try_from(Cons_Header_Rect.w).unwrap())
            - 1.5 * f32::from(lineskip)) as i16,
        y: (f32::from(lastline) - 0.9 * f32::from(lineskip)) as i16,
        w: 13,
        h: 25,
    };
    right_rect = Rect {
        x: (f32::from(Cons_Header_Rect.x + i16::try_from(Cons_Header_Rect.w).unwrap())
            - 1.0 * f32::from(lineskip)) as i16,
        y: (f32::from(lastline) - 0.9 * f32::from(lineskip)) as i16,
        w: 13,
        h: 25,
    };

    let mut droid_name = [0u8; 80];
    let droid = &*Druidmap.add(usize::try_from(droid_type).unwrap());
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
        SDL_SetClipRect(ne_screen, &Cons_Text_Rect);
        SDL_UpperBlit(console_bg_pic2, null_mut(), ne_screen, null_mut());
        SDL_SetClipRect(ne_screen, &Cons_Header_Rect);
        SDL_UpperBlit(console_bg_pic2, null_mut(), ne_screen, null_mut());
        SDL_SetClipRect(ne_screen, null_mut());
    } else {
        // otherwise we just redraw the whole screen
        SDL_UpperBlit(console_bg_pic2, null_mut(), ne_screen, null_mut());
        DisplayBanner(
            null_mut(),
            null_mut(),
            (DisplayBannerFlags::NO_SDL_UPDATE | DisplayBannerFlags::FORCE_UPDATE)
                .bits()
                .into(),
        );
    }

    DisplayText(
        info_text.as_mut_ptr() as *mut c_char,
        Cons_Text_Rect.x.into(),
        Cons_Text_Rect.y.into(),
        &Cons_Text_Rect,
    );

    DisplayText(
        droid_name.as_mut_ptr() as *mut c_char,
        i32::from(Cons_Header_Rect.x) + i32::from(lineskip),
        (f32::from(lastline) - 0.9 * f32::from(lineskip)) as i32,
        null_mut(),
    );

    if show_arrows {
        if Me.ty > droid_type {
            SDL_UpperBlit(arrow_up, null_mut(), ne_screen, &mut up_rect);
        }

        if droid_type > 0 {
            SDL_UpperBlit(arrow_down, null_mut(), ne_screen, &mut down_rect);
        }

        if page > 0 {
            SDL_UpperBlit(arrow_left, null_mut(), ne_screen, &mut left_rect);
        }

        if page < 2 {
            SDL_UpperBlit(arrow_right, null_mut(), ne_screen, &mut right_rect);
        }
    }

    if flags & i32::from(UPDATE_ONLY) != 0 {
        SDL_UpdateRects(ne_screen, 1, &mut Cons_Header_Rect);
        SDL_UpdateRects(ne_screen, 1, &mut Cons_Text_Rect);
    } else {
        SDL_Flip(ne_screen);
    }
}

/// Displays the concept view of Level "deck" in Userfenster
///
/// Note: we no longer wait here for a key-press, but return
/// immediately
#[no_mangle]
pub unsafe extern "C" fn ShowDeckMap(deck: Level) {
    let tmp = Me.pos;

    let cur_level = &*CurLevel;
    Me.pos.x = (cur_level.xlen / 2) as f32;
    Me.pos.y = (cur_level.ylen / 2) as f32;

    SDL_ShowCursor(SDL_DISABLE);

    SetCombatScaleTo(0.25);

    Assemble_Combat_Picture(
        (AssembleCombatWindowFlags::ONLY_SHOW_MAP | AssembleCombatWindowFlags::SHOW_FULL_MAP)
            .bits()
            .into(),
    );

    SDL_Flip(ne_screen);

    Me.pos = tmp;

    wait_for_key_pressed();

    SetCombatScaleTo(1.0);
}

/// EnterKonsole(): does all konsole- duties
/// This function runs the consoles. This means the following duties:
/// 2 * Show a small-scale plan of the current deck
/// 3 * Show a side-elevation on the ship
/// 1 * Give all available data on lower druid types
/// 0 * Reenter the game without squashing the colortable
#[no_mangle]
pub unsafe extern "C" fn EnterKonsole() {
    // Prevent distortion of framerate by the delay coming from
    // the time spend in the menu.
    Activate_Conservative_Frame_Computation();

    let tmp_rect = User_Rect;
    User_Rect = Full_User_Rect;

    wait_for_all_keys_released();

    Me.status = Status::Console as c_int;

    if cfg!(target_os = "android") {
        show_cursor = false;
    }

    SDL_SetCursor(arrow_cursor);

    SetCurrentFont(Para_BFont);

    let mut pos = 0; // starting menu position
    PaintConsoleMenu(c_int::try_from(pos).unwrap(), 0);

    let wait_move_ticks: u32 = 100;
    static mut LAST_MOVE_TICK: u32 = 0;
    let mut finished = false;
    let mut need_update = true;
    while !finished {
        if show_cursor {
            SDL_ShowCursor(SDL_ENABLE);
        } else {
            SDL_ShowCursor(SDL_DISABLE);
        }

        // check if the mouse-cursor is on any of the console-menu points
        for (i, rect) in Cons_Menu_Rects.iter_mut().enumerate() {
            if show_cursor && pos != i && CursorIsOnRect(rect) != 0 {
                MoveMenuPositionSound();
                pos = i;
                need_update = true;
            }
        }
        let action = getMenuAction(250);
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
                    if show_cursor {
                        let mousemove_buf = last_mouse_event;
                        SDL_WarpMouse(
                            (Cons_Menu_Rects[pos].x
                                + i16::try_from(Cons_Menu_Rects[pos].w / 2).unwrap())
                            .try_into()
                            .unwrap(),
                            (Cons_Menu_Rects[pos].y
                                + i16::try_from(Cons_Menu_Rects[pos].h / 2).unwrap())
                            .try_into()
                            .unwrap(),
                        );
                        update_input(); // this sets a new last_mouse_event
                        last_mouse_event = mousemove_buf; //... which we override.. ;)
                    }
                    MoveMenuPositionSound();
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
                    if show_cursor {
                        let mousemove_buf = last_mouse_event;
                        SDL_WarpMouse(
                            (Cons_Menu_Rects[pos].x
                                + i16::try_from(Cons_Menu_Rects[pos].w / 2).unwrap())
                            .try_into()
                            .unwrap(),
                            (Cons_Menu_Rects[pos].y
                                + i16::try_from(Cons_Menu_Rects[pos].h / 2).unwrap())
                            .try_into()
                            .unwrap(),
                        );
                        update_input(); // this sets a new last_mouse_event
                        last_mouse_event = mousemove_buf; //... which we override.. ;)
                    }
                    MoveMenuPositionSound();
                    need_update = true;
                    LAST_MOVE_TICK = SDL_GetTicks();
                }

                MenuAction::CLICK => {
                    MenuItemSelectedSound();
                    wait_for_all_keys_released();
                    need_update = true;
                    match pos {
                        0 => {
                            finished = true;
                        }
                        1 => {
                            GreatDruidShow();
                            PaintConsoleMenu(pos.try_into().unwrap(), 0);
                        }
                        2 => {
                            ClearGraphMem();
                            DisplayBanner(
                                null_mut(),
                                null_mut(),
                                DisplayBannerFlags::FORCE_UPDATE.bits().into(),
                            );
                            ShowDeckMap(*CurLevel);
                            PaintConsoleMenu(pos.try_into().unwrap(), 0);
                        }
                        3 => {
                            ClearGraphMem();
                            DisplayBanner(
                                null_mut(),
                                null_mut(),
                                DisplayBannerFlags::FORCE_UPDATE.bits().into(),
                            );
                            ShowLifts((*CurLevel).levelnum, -1);
                            wait_for_key_pressed();
                            PaintConsoleMenu(pos.try_into().unwrap(), 0);
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
            PaintConsoleMenu(pos.try_into().unwrap(), UPDATE_ONLY.into());
            if cfg!(not(target_os = "android")) {
                SDL_Flip(ne_screen);
            }

            need_update = false;
        }
        if cfg!(target_os = "android") {
            SDL_Flip(ne_screen); // for responsive input on Android, we need to run this every cycle
        }

        SDL_Delay(1); // don't hog CPU
    }

    User_Rect = tmp_rect;

    Me.status = Status::Mobile as c_int;

    ClearGraphMem();

    SDL_SetCursor(crosshair_cursor);
    if !show_cursor {
        SDL_ShowCursor(SDL_DISABLE);
    }
}

/// This function does the robot show when the user has selected robot
/// show from the console menu.
#[no_mangle]
pub unsafe extern "C" fn GreatDruidShow() {
    let mut finished = false;

    let mut droidtype = Me.ty;
    let mut page = 0;

    show_droid_info(droidtype, page, 0);
    show_droid_portrait(Cons_Droid_Rect, droidtype, 0.0, UPDATE | RESET);

    wait_for_all_keys_released();
    let mut need_update = true;
    let wait_move_ticks: u32 = 100;
    static mut LAST_MOVE_TICK: u32 = 0;

    while !finished {
        show_droid_portrait(Cons_Droid_Rect, droidtype, DROID_ROTATION_TIME, 0);

        if show_cursor {
            SDL_ShowCursor(SDL_ENABLE);
        } else {
            SDL_ShowCursor(SDL_DISABLE);
        }

        if need_update {
            show_droid_info(droidtype, page, UPDATE_ONLY.into());
            need_update = false;
        }

        let mut action = MenuAction::empty();
        // special handling of mouse-clicks: check if move-arrows were clicked on
        if MouseLeftPressedR() {
            if CursorIsOnRect(&left_rect) != 0 {
                action = MenuAction::LEFT;
            } else if CursorIsOnRect(&right_rect) != 0 {
                action = MenuAction::RIGHT;
            } else if CursorIsOnRect(&up_rect) != 0 {
                action = MenuAction::UP;
            } else if CursorIsOnRect(&down_rect) != 0 {
                action = MenuAction::DOWN;
            }
        } else {
            action = getMenuAction(250);
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

                if droidtype < Me.ty {
                    MoveMenuPositionSound();
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
                    MoveMenuPositionSound();
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
                    MoveMenuPositionSound();
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
                    MoveMenuPositionSound();
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

/// This function should check if the mouse cursor is in the given Rectangle
#[no_mangle]
pub unsafe extern "C" fn CursorIsOnRect(rect: &Rect) -> c_int {
    let user_center = get_user_center();
    let cur_pos = Point {
        x: input_axis.x + (i32::from(user_center.x) - 16),
        y: input_axis.y + (i32::from(user_center.y) - 16),
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
#[no_mangle]
pub unsafe extern "C" fn ShowLifts(level: c_int, liftrow: c_int) {
    let lift_bg_color = SDL_Color {
        r: 0,
        g: 0,
        b: 0,
        unused: 0,
    }; /* black... */
    let xoffs: i16 = (User_Rect.w / 20).try_into().unwrap();
    let yoffs: i16 = (User_Rect.h / 5).try_into().unwrap();

    SDL_ShowCursor(SDL_DISABLE);
    // fill the user fenster with some color
    Fill_Rect(User_Rect, lift_bg_color);

    /* First blit ship "lights off" */
    let mut dst = User_Rect;
    SDL_SetClipRect(ne_screen, &dst);
    dst = User_Rect;
    dst.x += xoffs;
    dst.y += yoffs;
    SDL_UpperBlit(ship_off_pic, null_mut(), ne_screen, &mut dst);

    if level >= 0 {
        for i in 0..curShip.num_level_rects[usize::try_from(level).unwrap()] {
            let mut src =
                curShip.Level_Rects[usize::try_from(level).unwrap()][usize::try_from(i).unwrap()];
            dst = src;
            dst.x += User_Rect.x + xoffs; /* offset respective to User-Rectangle */
            dst.y += User_Rect.y + yoffs;
            SDL_UpperBlit(ship_on_pic, &mut src, ne_screen, &mut dst);
        }
    }

    if liftrow >= 0 {
        let mut src = curShip.LiftRow_Rect[usize::try_from(liftrow).unwrap()];
        dst = src;
        dst.x += User_Rect.x + xoffs; /* offset respective to User-Rectangle */
        dst.y += User_Rect.y + yoffs;
        SDL_UpperBlit(ship_on_pic, &mut src, ne_screen, &mut dst);
    }

    SDL_Flip(ne_screen);
}

/// diese Funktion zeigt die m"oglichen Auswahlpunkte des Menus
/// Sie soll die Schriftfarben nicht ver"andern
///
/// NOTE: this function does not actually _display_ anything yet,
///       it just prepares the display, so you need
///       to call SDL_Flip() to display the result!
/// pos  : 0<=pos<=3: which menu-position is currently active?
/// flag : UPDATE_ONLY  only update the console-menu bar, not text & background
#[no_mangle]
pub unsafe extern "C" fn PaintConsoleMenu(pos: c_int, flag: c_int) {
    use std::io::Write;
    let mut menu_text: [u8; 200] = [0; 200];

    if (flag & i32::from(UPDATE_ONLY)) == 0 {
        ClearGraphMem();
        SDL_SetClipRect(ne_screen, null_mut());
        SDL_UpperBlit(console_bg_pic1, null_mut(), ne_screen, null_mut());

        DisplayBanner(
            null_mut(),
            null_mut(),
            DisplayBannerFlags::FORCE_UPDATE.bits().into(),
        );

        write!(
            &mut menu_text[..],
            "Area : {}\nDeck : {}    Alert: {}\0",
            CStr::from_ptr(curShip.AreaName.as_ptr()).to_str().unwrap(),
            CStr::from_ptr((*CurLevel).Levelname).to_str().unwrap(),
            AlertNames::try_from(AlertLevel).unwrap().to_str(),
        )
        .unwrap();
        DisplayText(
            menu_text.as_mut_ptr() as *mut c_char,
            Cons_Header_Rect.x.into(),
            Cons_Header_Rect.y.into(),
            &Cons_Header_Rect,
        );

        write!(
            &mut menu_text[..],
            "Logout from console\n\nDroid info\n\nDeck map\n\nShip map\0"
        )
        .unwrap();
        DisplayText(
            menu_text.as_mut_ptr() as *mut c_char,
            Cons_Text_Rect.x.into(),
            c_int::from(Cons_Text_Rect.y) + 25,
            &Cons_Text_Rect,
        );
    } // only if not UPDATE_ONLY was required

    let mut src = Rect {
        x: i16::try_from(Cons_Menu_Rects[0].w).unwrap() * i16::try_from(pos).unwrap()
            + (2. * pos as f32 * GameConfig.scale) as i16,
        y: 0,
        w: Cons_Menu_Rect.w,
        h: 4 * Cons_Menu_Rect.h,
    };
    SDL_UpperBlit(console_pic, &mut src, ne_screen, &mut Cons_Menu_Rect);
}

/// does all the work when we enter a lift
#[no_mangle]
pub unsafe extern "C" fn EnterLift() {
    /* Prevent distortion of framerate by the delay coming from
     * the time spend in the menu. */
    Activate_Conservative_Frame_Computation();

    /* make sure to release the fire-key */
    wait_for_all_keys_released();

    /* Prevent the influ from coming out of the lift in transfer mode
     * by turning off transfer mode as soon as the influ enters the lift */
    Me.status = Status::Elevator as c_int;

    SDL_ShowCursor(SDL_DISABLE);

    let mut cur_level = (*CurLevel).levelnum;

    let cur_lift = GetCurrentLift();
    if cur_lift == -1 {
        error!("Lift out of order, I'm so sorry !");
        return;
    }
    let mut cur_lift: usize = cur_lift.try_into().unwrap();

    EnterLiftSound();
    Switch_Background_Music_To(null_mut()); // turn off Bg music

    let mut up_lift = curShip.AllLifts[cur_lift].up;
    let mut down_lift = curShip.AllLifts[cur_lift].down;

    let liftrow = curShip.AllLifts[cur_lift].lift_row;

    // clear the whole screen
    ClearGraphMem();
    DisplayBanner(
        null_mut(),
        null_mut(),
        DisplayBannerFlags::FORCE_UPDATE.bits().into(),
    );

    let wait_move_ticks: u32 = 100;
    static mut LAST_MOVE_TICK: u32 = 0;
    let mut finished = false;
    while !finished {
        ShowLifts(cur_level, liftrow);

        let action = getMenuAction(500);
        if SDL_GetTicks() - LAST_MOVE_TICK > wait_move_ticks {
            match action {
                MenuAction::CLICK => {
                    finished = true;
                    wait_for_all_keys_released();
                }

                MenuAction::UP | MenuAction::UP_WHEEL => {
                    LAST_MOVE_TICK = SDL_GetTicks();
                    if up_lift != -1 {
                        if curShip.AllLifts[usize::try_from(up_lift).unwrap()].x == 99 {
                            error!("Lift out of order, so sorry ..");
                        } else {
                            down_lift = cur_lift.try_into().unwrap();
                            cur_lift = up_lift.try_into().unwrap();
                            cur_level = curShip.AllLifts[cur_lift].level;
                            up_lift = curShip.AllLifts[cur_lift].up;
                            ShowLifts(cur_level, liftrow);
                            MoveLiftSound();
                        }
                    }
                }

                MenuAction::DOWN | MenuAction::DOWN_WHEEL => {
                    LAST_MOVE_TICK = SDL_GetTicks();
                    if down_lift != -1 {
                        if curShip.AllLifts[usize::try_from(down_lift).unwrap()].x == 99 {
                            error!("Lift Out of order, so sorry ..");
                        } else {
                            up_lift = cur_lift.try_into().unwrap();
                            cur_lift = down_lift.try_into().unwrap();
                            cur_level = curShip.AllLifts[cur_lift].level;
                            down_lift = curShip.AllLifts[cur_lift].down;
                            ShowLifts(cur_level, liftrow);
                            MoveLiftSound();
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
    if cur_level != (*CurLevel).levelnum {
        let mut array_num = 0;

        let mut tmp;
        while {
            tmp = curShip.AllLevels[array_num];
            tmp.is_null().not()
        } {
            if (*tmp).levelnum == cur_level {
                break;
            } else {
                array_num += 1;
            }
        }

        CurLevel = curShip.AllLevels[array_num];

        // set the position of the influencer to the correct locatiohn
        Me.pos.x = curShip.AllLifts[cur_lift].x as f32;
        Me.pos.y = curShip.AllLifts[cur_lift].y as f32;

        for i in 0..c_int::try_from(MAXBLASTS).unwrap() {
            DeleteBlast(i);
        }
        for i in 0..c_int::try_from(MAXBULLETS).unwrap() {
            DeleteBullet(i);
        }
    }

    let cur_level = &*CurLevel;
    LeaveLiftSound();
    Switch_Background_Music_To(cur_level.Background_Song_Name);
    ClearGraphMem();
    DisplayBanner(
        null_mut(),
        null_mut(),
        DisplayBannerFlags::FORCE_UPDATE.bits().into(),
    );

    Me.status = Status::Mobile as c_int;
    Me.TextVisibleTime = 0.;
    Me.TextToBeDisplayed = cur_level.Level_Enter_Comment;
}

#[no_mangle]
pub unsafe extern "C" fn LevelEmpty() -> c_int {
    let cur_level = &*CurLevel;
    if cur_level.empty != 0 {
        return true.into();
    }

    let levelnum = cur_level.levelnum;

    AllEnemys[0..usize::try_from(NumEnemys).unwrap()]
        .iter()
        .find(|enemy| {
            enemy.levelnum == levelnum
                && enemy.status != Status::Out as c_int
                && enemy.status != Status::Terminated as c_int
        })
        .is_none()
        .into()
}
