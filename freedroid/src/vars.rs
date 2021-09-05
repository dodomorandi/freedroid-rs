use crate::{
    defs::{Droid, Status, ALLBLASTTYPES, MAX_INFLU_POSITION_HISTORY},
    structs::{BlastSpec, BulletSpec, DruidSpec, Finepoint, Gps, Influence},
};

use cstr::cstr;
use sdl_sys::SDL_Rect;
use std::{ffi::CStr, os::raw::c_int, ptr::null_mut};

#[derive(Debug)]
pub struct Vars {
    pub block_rect: SDL_Rect,
    pub screen_rect: SDL_Rect,
    pub user_rect: SDL_Rect,
    pub classic_user_rect: SDL_Rect,
    pub full_user_rect: SDL_Rect,
    pub banner_rect: SDL_Rect,
    // for droid-pic display in console
    pub portrait_rect: SDL_Rect,
    pub cons_droid_rect: SDL_Rect,
    pub menu_rect: SDL_Rect,
    pub options_menu_rect: SDL_Rect,
    pub digit_rect: SDL_Rect,
    pub cons_header_rect: SDL_Rect,
    pub cons_menu_rect: SDL_Rect,
    pub cons_text_rect: SDL_Rect,
    pub cons_menu_rects: [SDL_Rect; 4],
    pub cons_menu_item_rect: SDL_Rect,

    pub left_info_rect: SDL_Rect,
    pub right_info_rect: SDL_Rect,
    pub progress_meter_rect: SDL_Rect,
    pub progress_bar_rect: SDL_Rect,
    pub progress_text_rect: SDL_Rect,

    /* counter to Message: you have won(this ship */
    pub ship_empty_counter: c_int,
    pub me: Influence,

    pub droidmap: *mut DruidSpec,
    pub bulletmap: *mut BulletSpec,
    pub blastmap: [BlastSpec; ALLBLASTTYPES],
}

impl Default for Vars {
    fn default() -> Self {
        Self {
            block_rect: rect! {0, 0, 64, 64},
            screen_rect: rect! {0, 0, 640, 480},
            user_rect: rect! {0, 0, 0, 0},
            classic_user_rect: rect! {32, 150, 9*64, 4*64},
            full_user_rect: rect! {0, 64, 640, 480 - 64},
            banner_rect: rect! {0, 0, 640, 64 },
            portrait_rect: rect! {0, 0, 132, 180},
            cons_droid_rect: rect! {30, 190, 132, 180},
            menu_rect: rect! {2*64, 150, 640 - 3*64, 480 - 64},
            options_menu_rect: rect! {232, 0, 0, 0},
            digit_rect: rect! {0, 0, 16, 18},
            cons_header_rect: rect! {75, 64+40, 640 - 80, 135 - 64},
            cons_menu_rect: rect! {60, 180, 100, 256},
            cons_text_rect: rect! {180, 180, 640-185, 480 - 185},
            cons_menu_rects: [
                rect! {60, 180, 100, 62},
                rect! {60, 181 + 64, 100, 62},
                rect! {60, 181 + 2*64, 100, 62},
                rect! {60, 181 + 3*64, 100, 62},
            ],
            cons_menu_item_rect: rect! {0, 0, 0, 0},
            left_info_rect: rect! { 26, 44, 0, 0 },
            right_info_rect: rect! {484, 44, 0, 0 },
            progress_meter_rect: rect! {0, 0, 640, 480},
            progress_bar_rect: rect! {446, 155, 22, 111},
            progress_text_rect: rect! {213, 390, 157, 30},
            ship_empty_counter: 0,
            me: Influence {
                ty: Droid::Droid001 as i32,
                status: Status::Transfermode as i32,
                speed: Finepoint { x: 0., y: 0. },
                pos: Finepoint { x: 120., y: 48. },
                health: 100.,
                energy: 100.,
                firewait: 0.,
                phase: 0.,
                timer: 0.,
                last_crysound_time: 0.,
                last_transfer_sound_time: 0.,
                text_visible_time: 0.,
                text_to_be_displayed: null_mut(),
                position_history_ring_buffer: [Gps { x: 0., y: 0., z: 0 };
                    MAX_INFLU_POSITION_HISTORY],
            },
            droidmap: null_mut(),
            bulletmap: null_mut(),
            blastmap: [BlastSpec::default_const(); ALLBLASTTYPES],
        }
    }
}

pub const ORIG_BLOCK_RECT: SDL_Rect = rect! {0, 0, 64, 64}; // not to be rescaled ever!!
pub const ORIG_DIGIT_RECT: SDL_Rect = rect! {0, 0, 16, 18}; // not to be rescaled!

pub const CLASS_NAMES: [&CStr; 10] = [
    cstr!("Influence device"),
    cstr!("Disposal robot"),
    cstr!("Servant robot"),
    cstr!("Messenger robot"),
    cstr!("Maintenance robot"),
    cstr!("Crew droid"),
    cstr!("Sentinel droid"),
    cstr!("Battle droid"),
    cstr!("Security droid"),
    cstr!("Command Cyborg"),
];

pub const CLASSES: [&CStr; 11] = [
    cstr!("influence"),
    cstr!("disposal"),
    cstr!("servant"),
    cstr!("messenger"),
    cstr!("maintenance"),
    cstr!("crew"),
    cstr!("sentinel"),
    cstr!("battle"),
    cstr!("security"),
    cstr!("command"),
    cstr!("error"),
];

pub const DRIVE_NAMES: [&CStr; 7] = [
    cstr!("none"),
    cstr!("tracks"),
    cstr!("anti-grav"),
    cstr!("tripedal"),
    cstr!("wheels"),
    cstr!("bipedal"),
    cstr!("error"),
];

pub const SENSOR_NAMES: [&CStr; 7] = [
    cstr!(" - "),
    cstr!("spectral"),
    cstr!("infra-red"),
    cstr!("subsonic"),
    cstr!("ultra-sonic"),
    cstr!("radar"),
    cstr!("error"),
];

pub const BRAIN_NAMES: [&CStr; 4] = [
    cstr!("none"),
    cstr!("neutronic"),
    cstr!("primode"),
    cstr!("error"),
];

// Bullet-names:
pub const WEAPON_NAMES: [&CStr; 7] = [
    cstr!("none"),         // pulse
    cstr!("lasers"),       // single
    cstr!("lasers"),       // Military
    cstr!("disruptor"),    // flash
    cstr!("exterminator"), // exterminator
    cstr!("laser rifle"),  // laser-rifle
    cstr!("error"),
];

impl Vars {
    #[inline]
    pub fn get_user_center(&self) -> SDL_Rect {
        let SDL_Rect { x, y, w, h } = self.user_rect;
        SDL_Rect {
            x: x + (w / 2) as i16,
            y: y + (h / 2) as i16,
            w,
            h,
        }
    }
}
