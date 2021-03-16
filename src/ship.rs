use crate::{
    defs::{AlertNames, Sound, RESET, UPDATE},
    graphics::{packed_portraits, vid_bpp, ScalePic},
    map::GetMapBrick,
    ne_screen,
    sound::Play_Sound,
    structs::Level,
    vars::Portrait_Rect,
    AlertLevel, CurLevel, GameConfig,
};

use log::{error, warn};
use sdl::{
    ll::SDL_GetTicks,
    video::ll::{
        SDL_CreateRGBSurface, SDL_DisplayFormat, SDL_DisplayFormatAlpha, SDL_FreeSurface,
        SDL_RWops, SDL_SetClipRect, SDL_Surface, SDL_UpdateRects, SDL_UpperBlit,
    },
    Rect,
};
use std::{
    convert::TryFrom,
    ops::Not,
    os::raw::{c_float, c_int},
    ptr::null_mut,
};

extern "C" {
    pub fn show_droid_info(droid_type: c_int, page: c_int, flags: c_int);
    pub fn ShowDeckMap(deck: Level);
    pub fn EnterLift();
    pub fn EnterKonsole();

    pub fn IMG_Load_RW(src: *mut SDL_RWops, freesrc: c_int) -> *mut SDL_Surface;
    pub fn IMG_isJPG(src: *mut SDL_RWops) -> c_int;

    pub static mut droid_background: *mut SDL_Surface;
    pub static mut droid_pics: *mut SDL_Surface;
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
