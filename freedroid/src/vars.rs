use crate::{
    defs::{ALLBLASTTYPES, Droid, Explosion, MAX_INFLU_POSITION_HISTORY, Status},
    structs::{BlastSpec, BulletSpec, DruidSpec, Finepoint, Gps, Influence, TextToBeDisplayed},
};

use sdl::Rect;
use std::{
    array,
    ffi::CStr,
    ops::{Deref, DerefMut, Index, IndexMut},
};

#[derive(Debug)]
pub struct Vars<'sdl> {
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

    pub me: Influence,

    pub droidmap: Vec<DruidSpec>,
    pub bulletmap: [BulletSpec<'sdl>; 6],
    pub blastmap: BlastMap<'sdl>,
}

#[derive(Debug)]
#[repr(transparent)]
pub struct BlastMap<'sdl>([BlastSpec<'sdl>; ALLBLASTTYPES]);

impl Default for BlastMap<'_> {
    fn default() -> Self {
        Self(array::from_fn(|_| BlastSpec::default_const()))
    }
}

impl<'sdl> AsRef<[BlastSpec<'sdl>; ALLBLASTTYPES]> for BlastMap<'sdl> {
    fn as_ref(&self) -> &[BlastSpec<'sdl>; ALLBLASTTYPES] {
        &self.0
    }
}

impl<'sdl> AsMut<[BlastSpec<'sdl>; ALLBLASTTYPES]> for BlastMap<'sdl> {
    fn as_mut(&mut self) -> &mut [BlastSpec<'sdl>; ALLBLASTTYPES] {
        &mut self.0
    }
}

impl<'sdl> Deref for BlastMap<'sdl> {
    type Target = [BlastSpec<'sdl>; ALLBLASTTYPES];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BlastMap<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'sdl> Index<Explosion> for BlastMap<'sdl> {
    type Output = BlastSpec<'sdl>;

    fn index(&self, index: Explosion) -> &Self::Output {
        &self.0[usize::from(index.to_u8())]
    }
}

impl IndexMut<Explosion> for BlastMap<'_> {
    fn index_mut(&mut self, index: Explosion) -> &mut Self::Output {
        &mut self.0[usize::from(index.to_u8())]
    }
}

impl Default for Vars<'_> {
    fn default() -> Self {
        Self {
            block_rect: Rect::new(0, 0, 64, 64),
            screen_rect: Rect::new(0, 0, 640, 480),
            user_rect: Rect::new(0, 0, 0, 0),
            classic_user_rect: Rect::new(32, 150, 9 * 64, 4 * 64),
            full_user_rect: Rect::new(0, 64, 640, 480 - 64),
            banner_rect: Rect::new(0, 0, 640, 64),
            portrait_rect: Rect::new(0, 0, 132, 180),
            cons_droid_rect: Rect::new(30, 190, 132, 180),
            menu_rect: Rect::new(2 * 64, 150, 640 - 3 * 64, 480 - 64),
            options_menu_rect: Rect::new(232, 0, 0, 0),
            digit_rect: Rect::new(0, 0, 16, 18),
            cons_header_rect: Rect::new(75, 64 + 40, 640 - 80, 135 - 64),
            cons_menu_rect: Rect::new(60, 180, 100, 256),
            cons_text_rect: Rect::new(180, 180, 640 - 185, 480 - 185),
            cons_menu_rects: [
                Rect::new(60, 180, 100, 62),
                Rect::new(60, 181 + 64, 100, 62),
                Rect::new(60, 181 + 2 * 64, 100, 62),
                Rect::new(60, 181 + 3 * 64, 100, 62),
            ],
            cons_menu_item_rect: Rect::new(0, 0, 0, 0),
            left_info_rect: Rect::new(26, 44, 0, 0),
            right_info_rect: Rect::new(484, 44, 0, 0),
            progress_meter_rect: Rect::new(0, 0, 640, 480),
            progress_bar_rect: Rect::new(446, 155, 22, 111),
            progress_text_rect: Rect::new(213, 390, 157, 30),
            me: Influence {
                ty: Droid::Droid001,
                status: Status::Transfermode,
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
                text_to_be_displayed: TextToBeDisplayed::None,
                position_history_ring_buffer: [Gps { x: 0., y: 0., z: 0 };
                    MAX_INFLU_POSITION_HISTORY],
            },
            droidmap: Vec::default(),
            bulletmap: array::from_fn(|_| BulletSpec::default()),
            blastmap: BlastMap::default(),
        }
    }
}

pub const ORIG_BLOCK_RECT: Rect = Rect::new(0, 0, 64, 64); // not to be rescaled ever!!
pub const ORIG_DIGIT_RECT: Rect = Rect::new(0, 0, 16, 18); // not to be rescaled!

pub const CLASS_NAMES: [&CStr; 10] = [
    c"Influence device",
    c"Disposal robot",
    c"Servant robot",
    c"Messenger robot",
    c"Maintenance robot",
    c"Crew droid",
    c"Sentinel droid",
    c"Battle droid",
    c"Security droid",
    c"Command Cyborg",
];

pub const CLASSES: [&CStr; 11] = [
    c"influence",
    c"disposal",
    c"servant",
    c"messenger",
    c"maintenance",
    c"crew",
    c"sentinel",
    c"battle",
    c"security",
    c"command",
    c"error",
];

pub const DRIVE_NAMES: [&CStr; 7] = [
    c"none",
    c"tracks",
    c"anti-grav",
    c"tripedal",
    c"wheels",
    c"bipedal",
    c"error",
];

pub const SENSOR_NAMES: [&CStr; 7] = [
    c" - ",
    c"spectral",
    c"infra-red",
    c"subsonic",
    c"ultra-sonic",
    c"radar",
    c"error",
];

pub const BRAIN_NAMES: [&CStr; 4] = [c"none", c"neutronic", c"primode", c"error"];

impl Vars<'_> {
    #[inline]
    pub fn get_user_center(&self) -> Rect {
        let [x, y] = self.user_rect.center();
        self.user_rect.with_xy(x, y)
    }
}
