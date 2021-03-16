use crate::{
    b_font::{FontHeight, GetCurrentFont, Para_BFont, SetCurrentFont},
    defs::{AlertNames, DisplayBannerFlags, Sound, RESET, TEXT_STRETCH, UPDATE},
    global::Druidmap,
    graphics::{
        arrow_down, arrow_left, arrow_right, arrow_up, console_bg_pic2, packed_portraits, vid_bpp,
        ScalePic,
    },
    map::GetMapBrick,
    ne_screen,
    sound::Play_Sound,
    structs::Level,
    text::DisplayText,
    vars::{
        Cons_Header_Rect, Cons_Text_Rect, Portrait_Rect, BRAIN_NAMES, CLASSES, CLASS_NAMES,
        DRIVE_NAMES, SENSOR_NAMES, WEAPON_NAMES,
    },
    view::DisplayBanner,
    AlertLevel, CurLevel, GameConfig, Me,
};

use log::{error, warn};
use sdl::{
    ll::SDL_GetTicks,
    video::ll::{
        SDL_CreateRGBSurface, SDL_DisplayFormat, SDL_DisplayFormatAlpha, SDL_Flip, SDL_FreeSurface,
        SDL_RWops, SDL_SetClipRect, SDL_Surface, SDL_UpdateRects, SDL_UpperBlit,
    },
    Rect,
};
use std::{
    convert::TryFrom,
    ffi::CStr,
    ops::Not,
    os::raw::{c_char, c_float, c_int},
    ptr::null_mut,
};

const UPDATE_ONLY: u8 = 0x01;

extern "C" {
    pub fn ShowDeckMap(deck: Level);
    pub fn EnterLift();
    pub fn EnterKonsole();

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
