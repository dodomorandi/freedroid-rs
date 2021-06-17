use crate::{
    defs::{Droid, Status, ALLBLASTTYPES, MAX_INFLU_POSITION_HISTORY},
    structs::{BlastSpec, BulletSpec, DruidSpec, Finepoint, Gps, Influence},
};

use cstr::cstr;
use sdl::Rect;
use std::{ffi::CStr, fmt, os::raw::c_int, ptr::null_mut};

pub struct Vars {
    pub block_rect: Rect,
    pub screen_rect: Rect,
    pub user_rect: Rect,
    pub classic_user_rect: Rect,
    pub full_user_rect: Rect,
    pub banner_rect: Rect,
    // for droid-pic display in console
    pub portrait_rect: Rect,
    pub cons_droid_rect: Rect,
    pub menu_rect: Rect,
    pub options_menu_rect: Rect,
    pub digit_rect: Rect,
    pub cons_header_rect: Rect,
    pub cons_menu_rect: Rect,
    pub cons_text_rect: Rect,
    pub cons_menu_rects: [Rect; 4],
    pub cons_menu_item_rect: Rect,

    pub left_info_rect: Rect,
    pub right_info_rect: Rect,
    pub progress_meter_rect: Rect,
    pub progress_bar_rect: Rect,
    pub progress_text_rect: Rect,

    /* counter to Message: you have won(this ship */
    pub ship_empty_counter: c_int,
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
        }
    }
}

impl fmt::Debug for Vars {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        #[derive(Debug)]
        struct Rect {
            x: i16,
            y: i16,
            w: u16,
            h: u16,
        }

        impl From<&::sdl::Rect> for Rect {
            fn from(rect: &::sdl::Rect) -> Rect {
                Rect {
                    x: rect.x,
                    y: rect.y,
                    w: rect.w,
                    h: rect.h,
                }
            }
        }

        let block_rect = Rect::from(&self.block_rect);
        let screen_rect = Rect::from(&self.screen_rect);
        let user_rect = Rect::from(&self.user_rect);
        let classic_user_rect = Rect::from(&self.classic_user_rect);
        let full_user_rect = Rect::from(&self.full_user_rect);
        let banner_rect = Rect::from(&self.banner_rect);
        let portrait_rect = Rect::from(&self.portrait_rect);
        let cons_droid_rect = Rect::from(&self.cons_droid_rect);
        let menu_rect = Rect::from(&self.menu_rect);
        let options_menu_rect = Rect::from(&self.options_menu_rect);
        let digit_rect = Rect::from(&self.digit_rect);
        let cons_header_rect = Rect::from(&self.cons_header_rect);
        let cons_menu_rect = Rect::from(&self.cons_menu_rect);
        let cons_text_rect = Rect::from(&self.cons_text_rect);
        let cons_menu_rects = [
            Rect::from(&self.cons_menu_rects[0]),
            Rect::from(&self.cons_menu_rects[1]),
            Rect::from(&self.cons_menu_rects[2]),
            Rect::from(&self.cons_menu_rects[3]),
        ];
        let cons_menu_item_rect = Rect::from(&self.cons_menu_item_rect);
        let left_info_rect = Rect::from(&self.left_info_rect);
        let right_info_rect = Rect::from(&self.right_info_rect);
        let progress_meter_rect = Rect::from(&self.progress_meter_rect);
        let progress_bar_rect = Rect::from(&self.progress_bar_rect);
        let progress_text_rect = Rect::from(&self.progress_text_rect);

        f.debug_struct("Vars")
            .field("block_rect", &block_rect)
            .field("screen_rect", &screen_rect)
            .field("user_rect", &user_rect)
            .field("classic_user_rect", &classic_user_rect)
            .field("full_user_rect", &full_user_rect)
            .field("banner_rect", &banner_rect)
            .field("portrait_rect", &portrait_rect)
            .field("cons_droid_rect", &cons_droid_rect)
            .field("menu_rect", &menu_rect)
            .field("options_menu_rect", &options_menu_rect)
            .field("digit_rect", &digit_rect)
            .field("cons_header_rect", &cons_header_rect)
            .field("cons_menu_rect", &cons_menu_rect)
            .field("cons_text_rect", &cons_text_rect)
            .field("cons_menu_rects", &cons_menu_rects)
            .field("cons_menu_item_rect", &cons_menu_item_rect)
            .field("left_info_rect", &left_info_rect)
            .field("right_info_rect", &right_info_rect)
            .field("progress_meter_rect", &progress_meter_rect)
            .field("progress_bar_rect", &progress_bar_rect)
            .field("progress_text_rect", &progress_text_rect)
            .field("ship_empty_counter", &self.ship_empty_counter)
            .finish()
    }
}

pub const ORIG_BLOCK_RECT: Rect = rect! {0, 0, 64, 64}; // not to be rescaled ever!!
pub const ORIG_DIGIT_RECT: Rect = rect! {0, 0, 16, 18}; // not to be rescaled!

pub static mut ME: Influence = Influence {
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
    position_history_ring_buffer: [Gps { x: 0., y: 0., z: 0 }; MAX_INFLU_POSITION_HISTORY],
};

pub static mut DRUIDMAP: *mut DruidSpec = null_mut();
pub static mut BULLETMAP: *mut BulletSpec = null_mut();
pub static mut BLASTMAP: [BlastSpec; ALLBLASTTYPES] = [BlastSpec::default_const(); ALLBLASTTYPES];
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
