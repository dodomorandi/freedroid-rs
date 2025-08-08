#[cfg(not(target_os = "android"))]
use crate::menu::SHIP_EXT;
use crate::{
    defs::{
        self, Criticality, DIRECTIONS, Direction, Droid, MAP_DIR_C, MAX_ALERTS_ON_LEVEL,
        MAX_LEVELS, MAX_REFRESHES_ON_LEVEL, MAXWAYPOINTS, MapTile, Status, Themed,
    },
    find_subslice, map,
    misc::{
        locate_string_in_data, read_and_malloc_string_from_data, read_i32_from_string,
        read_u8_from_string,
    },
    read_and_malloc_and_terminate_file, split_at_subslice, split_at_subslice_mut,
    structs::{CoarsePoint, Enemy, Finepoint, Level, Lift, Waypoint},
};

use arrayvec::ArrayVec;
use bstr::ByteSlice;
#[cfg(not(target_os = "android"))]
use defs::MAX_DOORS_ON_LEVEL;
#[cfg(not(target_os = "android"))]
use defs::MAX_WP_CONNECTIONS;
#[cfg(not(target_os = "android"))]
use log::trace;
use log::{error, info, warn};
use nom::{Finish, IResult, Parser};
use rand::{Rng, seq::SliceRandom, thread_rng};
use sdl::Rect;
#[cfg(not(target_os = "android"))]
use std::ffi::CStr;
use std::{
    array,
    convert::identity,
    ffi::CString,
    fmt::{self, Display},
    ops::Not,
    path::Path,
};

const WALLPASS: f32 = 4_f32 / 64.;

const KONSOLEPASS_X: f32 = 0.5625;
const KONSOLEPASS_Y: f32 = 0.5625;

const TUERBREITE: f32 = 6_f32 / 64.;

const V_RANDSPACE: f32 = WALLPASS;
const V_RANDBREITE: f32 = 5_f32 / 64.;
const H_RANDSPACE: f32 = WALLPASS;
const H_RANDBREITE: f32 = 5_f32 / 64.;

const AREA_NAME_STRING: &[u8] = b"Area name=\"";
const LEVEL_NAME_STRING: &str = "Name of this level=";
const LEVEL_ENTER_COMMENT_STRING: &str = "Comment of the Influencer on entering this level=\"";
const BACKGROUND_SONG_NAME_STRING: &str = "Name of background song for this level=";
const MAP_BEGIN_STRING: &str = "begin_map";
const WP_BEGIN_STRING: &str = "begin_waypoints";
const LEVEL_END_STRING: &str = "end_level";
const CONNECTION_STRING: &str = "connections: ";

#[derive(Debug, Default)]
pub struct Map {
    inner_wait_counter: f32,
}

pub fn get_map_brick(deck: &Level, x: f32, y: f32) -> u8 {
    #[allow(clippy::cast_possible_truncation)]
    let [x, y] = [x.round() as i32, y.round() as i32];
    u8::try_from(y)
        .ok()
        .filter(|&y| y < deck.ylen)
        .zip(u8::try_from(x).ok().filter(|&x| x < deck.xlen))
        .map_or(MapTile::Void as u8, |(y, x)| {
            deck.map[usize::from(y)][usize::from(x)] as u8
        })
}

pub fn free_level_memory(level: &mut Level) {
    level.levelname = CString::default();
    level.background_song_name = CString::default();
    level.enter_comment = CString::default();

    level
        .map
        .iter_mut()
        .take(level.ylen.into())
        .for_each(Vec::clear);
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Color {
    #[default]
    Red,
    Yellow,
    Green,
    Gray,
    Blue,
    Greenblue,
    Dark,
}

impl Color {
    #[cfg(not(target_os = "android"))]
    #[must_use]
    pub const fn c_name(self) -> &'static CStr {
        match self {
            Color::Red => c"Red",
            Color::Yellow => c"Yellow",
            Color::Green => c"Green",
            Color::Gray => c"Grey",
            Color::Blue => c"Blue",
            Color::Greenblue => c"Turquoise",
            Color::Dark => c"Dark",
        }
    }

    #[cfg(not(target_os = "android"))]
    #[inline]
    #[must_use]
    pub const fn to_u8(self) -> u8 {
        self as u8
    }

    #[inline]
    #[must_use]
    pub const fn to_usize(self) -> usize {
        self as usize
    }
}

#[cfg(not(target_os = "android"))]
impl crate::menu::Steppable for Color {
    fn step_forward(&mut self) -> bool {
        *self = match *self {
            Color::Red => Color::Yellow,
            Color::Yellow => Color::Green,
            Color::Green => Color::Gray,
            Color::Gray => Color::Blue,
            Color::Blue => Color::Greenblue,
            Color::Greenblue => Color::Dark,
            Color::Dark => return false,
        };
        true
    }

    fn step_back(&mut self) -> bool {
        *self = match *self {
            Color::Red => return false,
            Color::Yellow => Color::Red,
            Color::Green => Color::Yellow,
            Color::Gray => Color::Green,
            Color::Blue => Color::Gray,
            Color::Greenblue => Color::Blue,
            Color::Dark => Color::Greenblue,
        };
        true
    }
}

impl TryFrom<u8> for Color {
    type Error = InvalidColor;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Red),
            1 => Ok(Self::Yellow),
            2 => Ok(Self::Green),
            3 => Ok(Self::Gray),
            4 => Ok(Self::Blue),
            5 => Ok(Self::Greenblue),
            6 => Ok(Self::Dark),
            _ => Err(InvalidColor),
        }
    }
}

#[derive(Debug)]
pub struct InvalidColor;

impl Display for InvalidColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("raw color value is invalid")
    }
}

impl std::error::Error for InvalidColor {}

#[cfg(not(target_os = "android"))]
fn reset_level_map(level: &mut Level) {
    // Now in the game and in the level editor, it might have happend that some open
    // doors occur.  The make life easier for the saving routine, these doors should
    // be closed first.

    use MapTile as M;
    level.map[0..usize::from(level.ylen)]
        .iter_mut()
        .flatten()
        .for_each(|tile| match tile {
            M::VZutuere | M::VHalbtuere1 | M::VHalbtuere2 | M::VHalbtuere3 | M::VGanztuere => {
                *tile = M::VZutuere;
            }
            M::HZutuere | M::HHalbtuere1 | M::HHalbtuere2 | M::HHalbtuere3 | M::HGanztuere => {
                *tile = M::HZutuere;
            }
            M::Refresh1 | M::Refresh2 | M::Refresh3 | M::Refresh4 => *tile = M::Refresh1,
            M::AlertGreen | M::AlertYellow | M::AlertAmber | M::AlertRed => *tile = M::AlertGreen,
            _ => {}
        });
}

/// initialize doors, refreshes and lifts for the given level-data
pub fn interpret(level: &mut Level) -> i32 {
    /* Get Doors Array */
    get_doors(level);

    // Get Refreshes
    get_refreshes(level);

    // Get Alerts
    get_alerts(level);

    defs::OK.into()
}

/// initializes the Doors array of the given level structure
/// Of course the level data must be in the structure already!!
/// Returns the number of doors found or ERR
pub fn get_doors(level: &mut Level) -> i32 {
    let mut curdoor = 0;

    level.doors.fill(None);

    /* now find the doors */
    for line in 0..level.ylen {
        for col in 0..level.xlen {
            let brick = level.map[usize::from(line)][usize::from(col)];
            if brick == MapTile::VZutuere || brick == MapTile::HZutuere {
                level.doors[curdoor] = Some(CoarsePoint { x: col, y: line });
                curdoor += 1;

                assert!(
                    curdoor <= MAX_DOORS_ON_LEVEL,
                    "\n\
\n\
----------------------------------------------------------------------\n\
Freedroid has encountered a problem:\n\
The number of doors found in level {} seems to be greater than the number\n\
of doors currently allowed in a freedroid map.\n\
\n\
The constant for the maximum number of doors currently is set to {} in the\n\
freedroid defs.h file.  You can enlarge the constant there, then start make\n\
and make install again, and the map will be loaded without complaint.\n\
\n\
The constant in defs.h is names 'MAX_DOORS_ON_LEVEL'.  If you received this \n\
message, please also tell the developers of the freedroid project, that they\n\
should enlarge the constant in all future versions as well.\n\
\n\
Thanks a lot.\n\
\n\
But for now Freedroid will terminate to draw attention to this small map problem.\n\
Sorry...\n\
----------------------------------------------------------------------\n\
\n",
                    level.levelnum,
                    MAX_DOORS_ON_LEVEL
                );
            }
        }
    }

    curdoor.try_into().unwrap()
}

/// This function initialized the array of Refreshes for animation
/// within the level
/// Returns the number of refreshes found or ERR
pub fn get_refreshes(level: &mut Level) -> i32 {
    let x_len = level.xlen;
    let y_len = level.ylen;

    level.refreshes.fill(None);

    let mut curref = 0;
    /* now find all the refreshes */
    for row in 0..y_len {
        for col in 0..x_len {
            if level.map[usize::from(row)][usize::from(col)] == MapTile::Refresh1 {
                level.refreshes[curref] = Some(CoarsePoint { x: col, y: row });
                curref += 1;

                assert!(
                    curref <= MAX_REFRESHES_ON_LEVEL,
                    "\n\
                        \n\
----------------------------------------------------------------------\n\
Freedroid has encountered a problem:\n\
The number of refreshes found in level {} seems to be greater than the number\n\
of refreshes currently allowed in a freedroid map.\n\
\n\
The constant for the maximum number of refreshes currently is set to {} in the\n\
freedroid defs.h file.  You can enlarge the constant there, then start make\n\
and make install again, and the map will be loaded without complaint.\n\
\n\
The constant in defs.h is names 'MAX_REFRESHES_ON_LEVEL'.  If you received this \n\
message, please also tell the developers of the freedroid project, that they\n\
should enlarge the constant in all future versions as well.\n\
\n\
Thanks a lot.\n\
\n\
But for now Freedroid will terminate to draw attention to this small map problem.\n\
Sorry...\n\
----------------------------------------------------------------------\n\
\n",
                    level.levelnum,
                    MAX_REFRESHES_ON_LEVEL
                );
            }
        }
    }

    curref.try_into().unwrap()
}

/// Find all alerts on this level and initialize their position-array
pub fn get_alerts(level: &mut Level) {
    let x_len = level.xlen;
    let y_len = level.ylen;

    level.alerts.fill(None);

    // now find all the alerts
    let mut curref = 0;
    for row in 0..y_len {
        for col in 0..x_len {
            if level.map[usize::from(row)][usize::from(col)] == MapTile::AlertGreen {
                level.alerts[curref] = Some(CoarsePoint { x: col, y: row });
                curref += 1;

                if curref > MAX_ALERTS_ON_LEVEL {
                    warn!(
                        "more alert-tiles found on level {} than allowed ({})!!",
                        level.levelnum, MAX_ALERTS_ON_LEVEL
                    );
                    warn!("remaining Alerts will be inactive...");
                    break;
                }
            }
        }
    }
}

fn whitespace<T, E>(input: T) -> nom::IResult<T, T, E>
where
    T: nom::InputTakeAtPosition + Default + Clone,
    E: nom::error::ParseError<T>,
    for<'a> &'a str: nom::FindToken<<T as nom::InputTakeAtPosition>::Item>,
{
    use nom::{
        bytes::complete::is_a,
        combinator::{map, opt},
    };
    map(opt(is_a(" \t")), Option::unwrap_or_default)(input)
}

/// This function is for LOADING map data!
/// This function extracts the data from *data and writes them
/// into a Level-struct:
///
/// Doors and Waypoints Arrays are initialized too
pub fn level_to_struct(data: &[u8]) -> Option<Level> {
    use nom::{character::complete::u8, sequence::tuple};

    /* Get the memory for one level */
    let mut loadlevel = Level {
        empty: false,
        timer: 0.,
        levelnum: 0,
        levelname: CString::default(),
        background_song_name: CString::default(),
        enter_comment: CString::default(),
        xlen: 0,
        ylen: 0,
        color: Color::default(),
        map: array::from_fn(|_| Vec::default()),
        refreshes: [None; MAX_REFRESHES_ON_LEVEL],
        doors: [None; MAX_DOORS_ON_LEVEL],
        alerts: [None; MAX_ALERTS_ON_LEVEL],
        waypoints: ArrayVec::default(),
    };

    info!("Starting to process information for another level:");

    /* Read Header Data: levelnum and x/ylen */
    let data_pointer = find_subslice(data, b"Levelnumber:")
        .map(|pos| &data[pos..])
        .expect("No Levelnumber entry found! Terminating! ");

    (
        loadlevel.levelnum,
        loadlevel.xlen,
        loadlevel.ylen,
        loadlevel.color,
    ) = parse_levelnum_xlen_ylen_color(data_pointer);

    info!("Levelnumber : {} ", loadlevel.levelnum);
    info!("xlen of this level: {} ", loadlevel.xlen);
    info!("ylen of this level: {} ", loadlevel.ylen);
    info!("color of this level: {} ", loadlevel.ylen);

    loadlevel.levelname =
        read_and_malloc_string_from_data(data, LEVEL_NAME_STRING.as_bytes(), b"\n");
    loadlevel.background_song_name =
        read_and_malloc_string_from_data(data, BACKGROUND_SONG_NAME_STRING.as_bytes(), b"\n");
    loadlevel.enter_comment =
        read_and_malloc_string_from_data(data, LEVEL_ENTER_COMMENT_STRING.as_bytes(), b"\n");

    // find the map data
    let map_begin = data.find(MAP_BEGIN_STRING.as_bytes())?;

    /* set position to Waypoint-Data */
    let wp_begin = data.find(WP_BEGIN_STRING.as_bytes())?;

    // find end of level-data
    let level_end = data.find(LEVEL_END_STRING.as_bytes())?;

    /* now scan the map */
    let mut lines = data[map_begin..].lines().skip(1);

    /* read MapData */
    for i in 0..usize::from(loadlevel.ylen) {
        let this_line = lines.next()?;
        loadlevel.map[i].resize(usize::from(loadlevel.xlen), MapTile::Void);
        let mut pos = this_line.trim_start();

        for k in 0..usize::from(loadlevel.xlen) {
            if pos.is_empty() {
                return None;
            }
            let raw_number;
            (raw_number, pos) = pos
                .iter()
                .position(|&c| c.is_ascii_digit().not())
                .map_or((pos, b"".as_slice()), |end_of_digits| {
                    pos.split_at(end_of_digits)
                });
            let tmp = std::str::from_utf8(raw_number)
                .ok()
                .and_then(|s| s.trim_start().parse::<u8>().ok())?;
            loadlevel.map[i][k] = tmp.try_into().unwrap();
            pos = pos.trim_start();
        }
    }

    /* Get Waypoints */
    let mut lines = data[wp_begin..level_end].lines().skip(1);

    for _ in 0..MAXWAYPOINTS {
        let Some(this_line) = lines.next() else {
            break;
        };

        let [x, y] = parse_waypoint_x_y(this_line);
        let mut waypoint = Waypoint {
            x: x.try_into().unwrap(),
            y: y.try_into().unwrap(),
            connections: ArrayVec::new(),
        };

        let mut pos = this_line
            [this_line.find(CONNECTION_STRING).unwrap() + CONNECTION_STRING.len()..]
            .trim_start();

        loop {
            if pos.is_empty() {
                break;
            }

            match tuple((whitespace, u8))(pos).finish() {
                Ok((rest, (_, connection))) => {
                    if waypoint.connections.try_push(connection).is_err() {
                        break;
                    }
                    pos = rest.trim_start();
                }
                Err(()) => break,
            }
        }
        loadlevel.waypoints.push(waypoint);
    }

    Some(loadlevel)
}

fn parse_levelnum_xlen_ylen_color(data: &[u8]) -> (u8, u8, u8, map::Color) {
    use nom::{bytes::complete::tag, character::complete::u8, sequence::tuple};

    let (_, (_, _, levelnum, _, _, x_len, _, _, y_len, _, _, color)) = tuple::<_, _, (), _>((
        tag("Levelnumber: "),
        whitespace,
        u8,
        tag("\nxlen of this level: "),
        whitespace,
        u8,
        tag("\nylen of this level: "),
        whitespace,
        u8,
        tag("\ncolor of this level: "),
        whitespace,
        u8,
    ))(data)
    .finish()
    .unwrap();

    (levelnum, x_len, y_len, color.try_into().unwrap())
}

fn parse_waypoint_x_y(data: &[u8]) -> [i32; 2] {
    use nom::{bytes::complete::tag, character::complete::i32, sequence::tuple};

    let (_, (_, _, _, _, _, _, x, _, _, _, y)) = tuple::<_, _, (), _>((
        tag("Nr.="),
        whitespace,
        i32,
        whitespace,
        tag("x="),
        whitespace,
        i32,
        whitespace,
        tag("y="),
        whitespace,
        i32,
    ))(data)
    .finish()
    .unwrap();

    [x, y]
}

impl crate::Data<'_> {
    /// Determines wether object on x/y is visible to the 001 or not
    pub fn is_visible(&self, objpos: Finepoint) -> i32 {
        let influ_x = self.vars.me.pos.x;
        let influ_y = self.vars.me.pos.y;

        let a_x = influ_x - objpos.x;
        let a_y = influ_y - objpos.y;

        let a_len = (a_x * a_x + a_y * a_y).sqrt();
        let mut step_num = a_len * 4.0;

        if step_num == 0. {
            step_num = 1.;
        }
        let step = Finepoint {
            x: a_x / step_num,
            y: a_y / step_num,
        };

        let mut testpos = objpos;

        #[allow(clippy::cast_possible_truncation)]
        let step_num = step_num as i32;
        for _ in 1..step_num {
            testpos.x += step.x;
            testpos.y += step.y;

            if self.is_passable(testpos.x, testpos.y, Direction::Light as i32)
                != Some(Direction::Center)
            {
                return false.into();
            }
        }

        true.into()
    }

    pub fn free_ship_memory(&mut self) {
        self.main.cur_ship.levels.drain(..).for_each(|mut level| {
            free_level_memory(&mut level);
        });
    }

    pub fn animate_refresh(&mut self) {
        self.map.inner_wait_counter += self.frame_time() * 10.;

        let cur_level = self.main.cur_level_mut();
        cur_level
            .refreshes
            .iter()
            .take(MAX_REFRESHES_ON_LEVEL)
            .copied()
            .map_while(identity)
            .for_each(|refresh| {
                let x = usize::from(refresh.x);
                let y = usize::from(refresh.y);

                cur_level.map[y][x] = MapTile::refresh(
                    #[allow(clippy::cast_possible_truncation)]
                    (self.map.inner_wait_counter.round() as i32 % 4)
                        .try_into()
                        .unwrap(),
                )
                .unwrap();
            });
    }

    #[allow(clippy::too_many_lines)]
    pub fn is_passable(&self, x: f32, y: f32, check_pos: i32) -> Option<Direction> {
        use Direction as D;
        use MapTile as M;

        let map_brick = get_map_brick(self.main.cur_level(), x, y);

        let fx = (x - 0.5) - (x - 0.5).floor();
        let fy = (y - 0.5) - (y - 0.5).floor();

        let map_tile = MapTile::try_from(map_brick).ok()?;

        match map_tile {
            M::Floor
            | M::Lift
            | M::Void
            | M::Block4
            | M::Block5
            | M::Refresh1
            | M::Refresh2
            | M::Refresh3
            | M::Refresh4
            | M::FineGrid => {
                Some(D::Center) /* these are passable */
            }

            M::AlertGreen | M::AlertYellow | M::AlertAmber | M::AlertRed => {
                (check_pos.try_into() == Ok(D::Light)).then_some(D::Center)
            }

            M::KonsoleL => (check_pos.try_into() == Ok(D::Light) || fx > 1.0 - KONSOLEPASS_X)
                .then_some(D::Center),

            M::KonsoleR => {
                (check_pos.try_into() == Ok(D::Light) || fx < KONSOLEPASS_X).then_some(D::Center)
            }

            M::KonsoleO => (check_pos.try_into() == Ok(D::Light) || fy > 1. - KONSOLEPASS_Y)
                .then_some(D::Center),

            M::KonsoleU => {
                (check_pos.try_into() == Ok(D::Light) || fy < KONSOLEPASS_Y).then_some(D::Center)
            }

            M::HWall => ((WALLPASS..=1. - WALLPASS).contains(&fy).not()).then_some(D::Center),

            M::VWall => ((WALLPASS..=1. - WALLPASS).contains(&fx).not()).then_some(D::Center),

            M::EckRo => {
                (fx > 1. - WALLPASS || fy < WALLPASS || (fx < WALLPASS && fy > 1. - WALLPASS))
                    .then_some(D::Center)
            }

            M::EckRu => {
                (fx > 1. - WALLPASS || fy > 1. - WALLPASS || (fx < WALLPASS && fy < WALLPASS))
                    .then_some(D::Center)
            }

            M::EckLu => {
                (fx < WALLPASS || fy > 1. - WALLPASS || (fx > 1. - WALLPASS && fy < WALLPASS))
                    .then_some(D::Center)
            }

            M::EckLo => {
                (fx < WALLPASS || fy < WALLPASS || (fx > 1. - WALLPASS && fy > 1. - WALLPASS))
                    .then_some(D::Center)
            }

            M::To => (fy < WALLPASS
                || (fy > 1. - WALLPASS && (WALLPASS..=1. - WALLPASS).contains(&fx).not()))
            .then_some(D::Center),

            M::Tr => (fx > 1. - WALLPASS
                || (fx < WALLPASS && (WALLPASS..=1. - WALLPASS).contains(&fy).not()))
            .then_some(D::Center),

            M::Tu => (fy > 1. - WALLPASS
                || (fy < WALLPASS && (WALLPASS..=1. - WALLPASS).contains(&fx).not()))
            .then_some(D::Center),

            M::Tl => (fx < WALLPASS
                || (fx > 1. - WALLPASS && (WALLPASS..=1. - WALLPASS).contains(&fy).not()))
            .then_some(D::Center),

            M::HGanztuere
            | M::HHalbtuere3
            | M::HHalbtuere2
            | M::VGanztuere
            | M::VHalbtuere3
            | M::VHalbtuere2
                if (check_pos.try_into() == Ok(D::Light)) =>
            {
                Some(D::Center)
            }
            M::HHalbtuere1 | M::HZutuere | M::VHalbtuere1 | M::VZutuere
                if (check_pos.try_into() == Ok(D::Light)) =>
            {
                None
            }

            M::HGanztuere | M::HHalbtuere3 | M::HHalbtuere2 | M::HHalbtuere1 | M::HZutuere => {
                if (H_RANDBREITE..=1. - H_RANDBREITE).contains(&fx).not()
                    && (H_RANDSPACE..=1. - H_RANDSPACE).contains(&fy)
                {
                    let Ok(check_pos) = check_pos.try_into() else {
                        return None;
                    };
                    if check_pos != D::Center && check_pos != D::Light && self.vars.me.speed.y != 0.
                    {
                        match check_pos {
                            D::Rechtsoben | D::Rechtsunten | D::Rechts => {
                                (fx > 1. - H_RANDBREITE).then_some(D::Links)
                            }
                            D::Linksoben | D::Linksunten | D::Links => {
                                (fx < H_RANDBREITE).then_some(D::Rechts)
                            }
                            _ => None, /* switch check_pos */
                        }
                    }
                    /* if DRUID && Me.speed.y != 0 */
                    else {
                        None
                    }
                } else if map_tile == M::HGanztuere
                    || map_tile == M::HHalbtuere3
                    || !(TUERBREITE..=1. - TUERBREITE).contains(&fy)
                {
                    Some(D::Center)
                } else {
                    None
                }
            }

            M::VGanztuere | M::VHalbtuere3 | M::VHalbtuere2 | M::VHalbtuere1 | M::VZutuere => {
                if (V_RANDBREITE..=1. - V_RANDBREITE).contains(&fy).not()
                    && (V_RANDSPACE..=1. - V_RANDSPACE).contains(&fx)
                {
                    let Ok(check_pos) = check_pos.try_into() else {
                        return None;
                    };
                    if check_pos != D::Center && check_pos != D::Light && self.vars.me.speed.x != 0.
                    {
                        match check_pos {
                            D::Rechtsoben | D::Linksoben | D::Oben => {
                                (fy < V_RANDBREITE).then_some(D::Unten)
                            }
                            D::Rechtsunten | D::Linksunten | D::Unten => {
                                (fy > 1. - V_RANDBREITE).then_some(D::Oben)
                            }
                            _ => None,
                        }
                    } else {
                        None
                    }
                } else if map_tile == M::VGanztuere
                    || map_tile == M::VHalbtuere3
                    || !(TUERBREITE..=1. - TUERBREITE).contains(&fx)
                {
                    Some(D::Center)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Saves ship-data to disk
    #[cfg(not(target_os = "android"))]
    pub fn save_ship(&mut self, shipname: &str) -> i32 {
        use std::{fs::File, io::Write, path::PathBuf};

        trace!("SaveShip(): real function call confirmed.");

        let filename = PathBuf::from(format!("{shipname}{SHIP_EXT}"));

        /* count the levels */
        let level_anz: u8 = self.main.cur_ship.levels.iter().count().try_into().unwrap();

        trace!("SaveShip(): now opening the ship file...");

        let mut ship_file = match File::create(filename) {
            Ok(file) => file,
            Err(err) => {
                panic!("Error opening ship file: {err}. Terminating");
            }
        };

        let result = (|| -> Result<(), std::io::Error> {
            //--------------------
            // Now that the file is opend for writing, we can start writing.  And the first thing
            // we will write to the file will be a fine header, indicating what this file is about
            // and things like that...
            //
            const MAP_HEADER_STRING: &str = "\n\
----------------------------------------------------------------------\n\
This file was generated using the Freedroid level editor.\n\
Please feel free to make any modifications you like, but in order for you\n\
to have an easier time, it is recommended that you use the Freedroid level\n\
editor for this purpose.  If you have created some good new maps, please \n\
send a short notice (not too large files attached) to the freedroid project.\n\
\n\
freedroid-discussion@lists.sourceforge.net\n\
----------------------------------------------------------------------\n\
\n";
            const AREA_NAME_STRING: &str = "Area name=\"";
            const END_OF_SHIP_DATA_STRING: &str = "*** End of Ship Data ***";

            ship_file.write_all(MAP_HEADER_STRING.as_bytes())?;
            ship_file.write_all(AREA_NAME_STRING.as_bytes())?;
            ship_file.write_all(self.main.cur_ship.area_name.to_bytes())?;
            ship_file.write_all(b"\"\n\n  ")?;

            /* Save all Levels */

            trace!("SaveShip(): now saving levels...");

            for i in 0..level_anz {
                let mut level_iter = self
                    .main
                    .cur_ship
                    .levels
                    .iter_mut()
                    .filter(|level| level.levelnum == i);

                let level = level_iter
                    .next()
                    .expect("Missing Levelnumber error in SaveShip.");

                assert!(
                    level_iter.next().is_none(),
                    "Identical Levelnumber Error in SaveShip."
                );

                //--------------------
                // Now comes the real saving part FOR ONE LEVEL.  First THE LEVEL is packed into a string and
                // then this string is wirtten to the file.  easy. simple.
                let level_mem = struct_to_mem(level);
                let end = level_mem
                    .iter()
                    .copied()
                    .position(|c| c == b'\0')
                    .unwrap_or(level_mem.len());
                ship_file.write_all(&level_mem[..end])?;
            }

            //--------------------
            // Now we are almost done writing.  Everything that is missing is
            // the termination string for the ship file.  This termination string
            // is needed later for the ship loading functions to find the end of
            // the data and to be able to terminate the long file-string with a
            // null character at the right position.
            //
            writeln!(ship_file, "{END_OF_SHIP_DATA_STRING}\n")?;

            trace!("SaveShip(): now flushing ship file...");
            ship_file.flush()?;

            trace!("SaveShip(): end of function reached.");
            Ok(())
        })();

        match result {
            Ok(()) => defs::OK.into(),
            Err(err) => {
                panic!("Error writing to ship file: {err}. Terminating");
            }
        }
    }

    /// This funtion moves the level doors in the sense that they are opened
    /// or closed depending on whether there is a robot close to the door or
    /// not.  Initially this function did not take into account the framerate
    /// and just worked every frame.  But this WASTES COMPUTATION time and it
    /// DOES THE ANIMATION TOO QUICKLY.  So, the most reasonable way out seems
    /// to be to operate this function only from time to time, e.g. after a
    /// specified delay has passed.
    pub fn move_level_doors(&mut self) {
        // This prevents animation going too quick.
        // The constant should be replaced by a variable, that can be
        // set from within the theme, but that may be done later...
        if self.global.level_doors_not_moved_time < self.global.time_for_each_phase_of_door_movement
        {
            return;
        }
        self.global.level_doors_not_moved_time = 0.;

        let cur_level = crate::cur_level!(mut self.main);
        for i in 0..MAX_DOORS_ON_LEVEL {
            const DOOROPENDIST2: f32 = 1.;

            let Some(door) = cur_level.doors[i] else {
                break;
            };

            let pos = &mut cur_level.map[usize::from(door.y)][usize::from(door.x)];

            // NORMALISATION doorx = doorx * Block_Rect.w + Block_Rect.w / 2;
            // NORMALISATION doory = doory * Block_Rect.h + Block_Rect.h / 2;

            /* first check Influencer gegen Tuer */
            let x_dist = self.vars.me.pos.x - f32::from(door.x);
            let y_dist = self.vars.me.pos.y - f32::from(door.y);
            let dist2 = x_dist * x_dist + y_dist * y_dist;

            if dist2 < DOOROPENDIST2 {
                if *pos != MapTile::HGanztuere && *pos != MapTile::VGanztuere {
                    *pos = pos.next().unwrap();
                }
            } else {
                let droid_is_nearby = self
                    .main
                    .enemys
                    .iter()
                    .filter(|enemy| {
                        (matches!(enemy.status, Status::Out | Status::Terminated)
                            || enemy.levelnum != cur_level.levelnum)
                            .not()
                    })
                    .any(|enemy| {
                        let x_dist = (enemy.pos.x - f32::from(door.x)).trunc().abs();
                        if x_dist < self.vars.block_rect.width().into() {
                            let y_dist = (enemy.pos.y - f32::from(door.y)).trunc().abs();
                            if y_dist < self.vars.block_rect.height().into() {
                                let dist2 = x_dist * x_dist + y_dist * y_dist;
                                return dist2 < DOOROPENDIST2;
                            }
                        }
                        false
                    });

                if droid_is_nearby {
                    if *pos != MapTile::HGanztuere && *pos != MapTile::VGanztuere {
                        *pos = pos.next().unwrap();
                    }
                } else if *pos != MapTile::VZutuere && *pos != MapTile::HZutuere {
                    *pos = pos.prev().unwrap();
                }
            }
        }
    }

    pub fn druid_passable(&self, x: f32, y: f32) -> Option<Direction> {
        let testpos: [Finepoint; DIRECTIONS] = [
            Finepoint {
                x,
                y: y - self.global.droid_radius,
            },
            Finepoint {
                x: x + self.global.droid_radius,
                y: y - self.global.droid_radius,
            },
            Finepoint {
                x: x + self.global.droid_radius,
                y,
            },
            Finepoint {
                x: x + self.global.droid_radius,
                y: y + self.global.droid_radius,
            },
            Finepoint {
                x,
                y: y + self.global.droid_radius,
            },
            Finepoint {
                x: x - self.global.droid_radius,
                y: y + self.global.droid_radius,
            },
            Finepoint {
                x: x - self.global.droid_radius,
                y,
            },
            Finepoint {
                x: x - self.global.droid_radius,
                y: y - self.global.droid_radius,
            },
        ];

        testpos
            .iter()
            .enumerate()
            .map(|(direction_index, test_pos)| {
                self.is_passable(
                    test_pos.x,
                    test_pos.y,
                    i32::try_from(direction_index).unwrap(),
                )
            })
            .find(|&is_passable| is_passable != Some(Direction::Center))
            .unwrap_or(Some(Direction::Center))
    }

    /// This function receives a pointer to the already read in crew section in a already read in
    /// droids file and decodes all the contents of that droid section to fill the `AllEnemys`
    /// array with droid types according to the specifications made in the file.
    pub fn get_this_levels_droids(&mut self, section_data: &[u8]) {
        const DROIDS_LEVEL_INDICATION_STRING: &[u8] = b"Level=";
        const DROIDS_LEVEL_END_INDICATION_STRING: &[u8] = b"** End of this levels droid data **";
        const DROIDS_MAXRAND_INDICATION_STRING: &[u8] = b"Maximum number of Random Droids=";
        const DROIDS_MINRAND_INDICATION_STRING: &[u8] = b"Minimum number of Random Droids=";
        const ALLOWED_TYPE_INDICATION_STRING: &[u8] =
            b"Allowed Type of Random Droid for this level: ";

        let section_data = &section_data
            [..locate_string_in_data(section_data, DROIDS_LEVEL_END_INDICATION_STRING)];

        // Now we read in the level number for this level
        let our_level_number = read_u8_from_string(section_data, DROIDS_LEVEL_INDICATION_STRING);

        // Now we read in the maximal number of random droids for this level
        let max_rand = read_i32_from_string(section_data, DROIDS_MAXRAND_INDICATION_STRING);

        // Now we read in the minimal number of random droids for this level
        let min_rand = read_i32_from_string(section_data, DROIDS_MINRAND_INDICATION_STRING);

        let mut different_random_types = 0;
        let mut search_pos_opt = section_data.find(ALLOWED_TYPE_INDICATION_STRING);
        let mut list_of_types_allowed: [Droid; 1000] = [Droid::Droid001; 1000];
        while let Some(mut search_pos) = search_pos_opt {
            search_pos += ALLOWED_TYPE_INDICATION_STRING.len();
            let remaining_data = &section_data[search_pos..];
            let type_indication_string = &remaining_data[..3];
            // Now that we have got a type indication string, we only need to translate it
            // into a number corresponding to that droid in the droid list
            let mut list_index = 0;
            while list_index < self.main.number_of_droid_types {
                if self.vars.droidmap[usize::from(list_index)].druidname
                    == std::str::from_utf8(type_indication_string).unwrap()
                {
                    break;
                }
                list_index += 1;
            }
            if list_index >= self.main.number_of_droid_types {
                panic!(
                    "unknown droid type: {} found in data file for level {}",
                    String::from_utf8_lossy(type_indication_string),
                    our_level_number,
                );
            } else {
                info!(
                    "Type indication string {} translated to type Nr.{}.",
                    String::from_utf8_lossy(type_indication_string),
                    list_index,
                );
            }
            list_of_types_allowed[different_random_types] = list_index.try_into().unwrap();
            different_random_types += 1;

            search_pos_opt = remaining_data
                .find(ALLOWED_TYPE_INDICATION_STRING)
                .map(|pos| pos + search_pos);
        }
        info!(
            "Found {} different allowed random types for this level. ",
            different_random_types,
        );

        //--------------------
        // At this point, the List "ListOfTypesAllowed" has been filled with the NUMBERS of
        // the allowed types.  The number of different allowed types found is also available.
        // That means that now we can add the apropriate droid types into the list of existing
        // droids in that mission.

        let mut rng = thread_rng();
        let mut real_number_of_random_droids = rng.gen_range(min_rand..=max_rand);

        while real_number_of_random_droids > 0 {
            real_number_of_random_droids -= 1;

            let enemy_slot = self
                .main
                .enemys
                .iter_mut()
                .find(|enemy| enemy.status == Status::Out);
            let random_droid_type = list_of_types_allowed[0..different_random_types]
                .choose(&mut rng)
                .copied()
                .unwrap();

            let new_enemy = Enemy::new(random_droid_type, our_level_number);
            if let Some(enemy_slot) = enemy_slot {
                *enemy_slot = new_enemy;
            } else {
                self.main
                    .enemys
                    .try_push(new_enemy)
                    .expect("No more free position to fill random droids into in GetCrew");
            }
        }
    }

    /// This function initializes all enemys
    pub fn get_crew(&mut self, filename: &[u8]) -> i32 {
        const END_OF_DROID_DATA_STRING: &[u8] = b"*** End of Droid Data ***";
        const DROIDS_LEVEL_DESCRIPTION_START_STRING: &[u8] = b"** Beginning of new Level **";
        const DROIDS_LEVEL_DESCRIPTION_END_STRING: &[u8] = b"** End of this levels droid data **";

        self.main.enemys.clear();

        //Now its time to start decoding the droids file.
        //For that, we must get it into memory first.
        //The procedure is the same as with LoadShip
        let fpath = self
            .find_file(
                filename,
                Some(MAP_DIR_C),
                Themed::NoTheme as i32,
                Criticality::Critical as i32,
            )
            .unwrap();
        let fpath = Path::new(
            fpath
                .to_str()
                .expect("unable to convert C string to UTF-8 string"),
        );

        let mut main_droids_file =
            read_and_malloc_and_terminate_file(fpath, END_OF_DROID_DATA_STRING);

        // The Droid crew file for this map is now completely read into memory
        // It's now time to decode the file and to fill the array of enemys with
        // new droids of the given types.
        let mut droid_section_slice_opt =
            split_at_subslice_mut(&mut main_droids_file, DROIDS_LEVEL_DESCRIPTION_START_STRING)
                .map(|(_, s)| s);
        while let Some(droid_section_slice) = droid_section_slice_opt {
            info!("Found another levels droids description starting point entry!");
            let end_of_this_droid_section_index =
                find_subslice(droid_section_slice, DROIDS_LEVEL_DESCRIPTION_END_STRING)
                    .expect("GetCrew: Unterminated droid section encountered.");
            self.get_this_levels_droids(droid_section_slice);

            droid_section_slice_opt = split_at_subslice_mut(
                &mut droid_section_slice[(end_of_this_droid_section_index + 2)..],
                DROIDS_LEVEL_DESCRIPTION_START_STRING,
            )
            .map(|(_, s)| s);
        }

        // Now that the correct crew types have been filled into the
        // right structure, it's time to set the energy of the corresponding
        // droids to "full" which means to the maximum of each type.
        for enemy in &mut self.main.enemys {
            enemy.energy = self.vars.droidmap[enemy.ty.to_usize()].maxenergy;
            enemy.status = Status::Mobile;
        }

        defs::OK.into()
    }

    /// loads lift-connctions to cur-ship struct
    pub fn get_lift_connections(&mut self, filename: &[u8]) -> i32 {
        const END_OF_LIFT_DATA_STRING: &[u8] = b"*** End of elevator specification file ***";
        const START_OF_LIFT_RECTANGLE_DATA_STRING: &[u8] =
            b"*** Beginning of elevator rectangles ***";

        /* Now get the lift-connection data from "FILE.elv" file */
        let fpath = self
            .find_file(
                filename,
                Some(MAP_DIR_C),
                Themed::NoTheme as i32,
                Criticality::Critical as i32,
            )
            .unwrap();
        let fpath = Path::new(
            fpath
                .to_str()
                .expect("unable to convert C string to UTF-8 string"),
        );

        let data = read_and_malloc_and_terminate_file(fpath, END_OF_LIFT_DATA_STRING);

        // At first we read in the rectangles that define where the colums of the
        // lift are, so that we can highlight them later.
        self.main.cur_ship.lift_row_rects.clear();
        let mut entry_slice =
            &data[find_subslice(&data, START_OF_LIFT_RECTANGLE_DATA_STRING).unwrap()..];
        loop {
            let next_entry_slice =
                split_at_subslice(entry_slice, b"Elevator Number=").map(|(_, s)| s);
            entry_slice = match next_entry_slice {
                Some(x) => x,
                None => break,
            };

            let elevator_index = nom::character::complete::u16::<_, ()>(entry_slice)
                .finish()
                .unwrap()
                .1;
            assert_eq!(
                usize::from(elevator_index),
                self.main.cur_ship.lift_row_rects.len()
            );
            entry_slice = &entry_slice[1..];

            let x = read_tagged_i16(entry_slice, "ElRowX=");
            let y = read_tagged_i16(entry_slice, "ElRowY=");
            let w = read_tagged_u16(entry_slice, "ElRowW=");
            let h = read_tagged_u16(entry_slice, "ElRowH=");

            self.main
                .cur_ship
                .lift_row_rects
                .push(Rect::new(x, y, w, h));
        }

        //--------------------
        // Now we read in the rectangles that define where the decks of the
        // current area system are, so that we can highlight them later in the
        // elevator and console functions.
        //
        self.main
            .cur_ship
            .level_rects
            .iter_mut()
            .for_each(ArrayVec::clear);
        let mut entry_slice = &*data;

        loop {
            let next_entry_slice = split_at_subslice(entry_slice, b"DeckNr=").map(|(_, s)| s);

            entry_slice = match next_entry_slice {
                Some(x) => x,
                None => break,
            };

            let deck_index = nom::character::complete::u8::<_, ()>(entry_slice)
                .finish()
                .unwrap()
                .1;

            let deck = &mut self.main.cur_ship.level_rects[usize::from(deck_index)];
            let rect_index = read_tagged_u16(entry_slice, "RectNumber=");
            assert_eq!(usize::from(rect_index), deck.len());
            entry_slice = &entry_slice[1..];

            let x = read_tagged_i16(&entry_slice[1..], "DeckX=");
            let y = read_tagged_i16(entry_slice, "DeckY=");
            let w = read_tagged_u16(entry_slice, "DeckW=");
            let h = read_tagged_u16(entry_slice, "DeckH=");

            deck.push(Rect::new(x, y, w, h));
        }

        self.load_lifts_from_data(entry_slice);

        defs::OK.into()
    }

    fn load_lifts_from_data(&mut self, data: &[u8]) {
        const START_OF_LIFT_DATA_STRING: &[u8] = b"*** Beginning of Lift Data ***";

        let mut entry_slice = &data[find_subslice(data, START_OF_LIFT_DATA_STRING)
            .expect("START OF LIFT DATA STRING NOT FOUND!  Terminating...")..];

        self.main.cur_ship.lifts.clear();
        loop {
            let next_entry_slice = split_at_subslice(entry_slice, b"Label=").map(|(_, s)| s);

            entry_slice = match next_entry_slice {
                Some(x) => x,
                None => break,
            };

            let label = nom::character::complete::u16::<_, ()>(entry_slice)
                .finish()
                .unwrap()
                .1;
            entry_slice = &entry_slice[1..];

            assert_eq!(usize::from(label), self.main.cur_ship.lifts.len());
            let level = read_tagged_u8(entry_slice, "Deck=");
            let x = read_tagged_i32(entry_slice, "PosX=");
            let y = read_tagged_i32(entry_slice, "PosY=");
            let up = read_tagged_i32(entry_slice, "LevelUp=");
            let down = read_tagged_i32(entry_slice, "LevelDown=");
            let row = read_tagged_i32(entry_slice, "LiftRow=");
            self.main.cur_ship.lifts.push(Lift {
                level,
                x,
                y,
                up,
                down,
                row,
            });
        }
    }

    pub fn load_ship(&mut self, filename: &[u8]) -> i32 {
        const END_OF_SHIP_DATA_STRING: &[u8] = b"*** End of Ship Data ***";

        let mut level_start: [Option<&[u8]>; MAX_LEVELS] = [None; MAX_LEVELS];
        self.free_ship_memory(); // clear vestiges of previous ship data, if any

        /* Read the whole ship-data to memory */
        let fpath = self
            .find_file(
                filename,
                Some(MAP_DIR_C),
                Themed::NoTheme as i32,
                Criticality::Critical as i32,
            )
            .unwrap();
        let fpath = Path::new(
            fpath
                .to_str()
                .expect("unable to convert C string to UTF-8 string"),
        );

        let ship_data = read_and_malloc_and_terminate_file(fpath, END_OF_SHIP_DATA_STRING);

        // Now we read the Area-name from the loaded data
        let buffer = read_and_malloc_string_from_data(&ship_data, AREA_NAME_STRING, b"\"");
        self.main.cur_ship.area_name.set_slice(buffer.to_bytes());
        drop(buffer);

        // Now we count the number of levels and remember their start-addresses.
        // This is done by searching for the LEVEL_END_STRING again and again
        // until it is no longer found in the ship file.  good.

        let mut levels_count = 0u8;
        let mut ship_rest = &*ship_data;
        level_start[0] = Some(ship_rest);

        loop {
            let next_ship_rest =
                split_at_subslice(ship_rest, LEVEL_END_STRING.as_bytes()).map(|(_, s)| s);
            ship_rest = match next_ship_rest {
                Some(x) => x,
                None => break,
            };

            levels_count += 1;
            level_start[usize::from(levels_count)] = Some(&ship_rest[1..]);
        }

        self.main.cur_ship.levels.clear();
        let result = level_start
            .iter()
            .copied()
            .enumerate()
            .take(levels_count.into())
            .try_for_each(|(index, start)| {
                if let Some(new_level) = level_to_struct(start.unwrap()) {
                    self.main.cur_ship.levels.push(new_level);

                    // initialize doors, refreshes and lifts
                    map::interpret(self.main.cur_ship.levels.last_mut().unwrap());
                    Some(())
                } else {
                    error!("reading of level {} failed", index);
                    None
                }
            });

        if result.is_none() {
            return defs::ERR.into();
        }

        defs::OK.into()
    }

    /// Checks Influencer on `SpecialFields` like Lifts and Konsoles and acts on it
    pub fn act_special_field(&mut self, x: f32, y: f32) {
        let map_tile = get_map_brick(self.main.cur_level(), x, y);

        let myspeed2 = self.vars.me.speed.x * self.vars.me.speed.x
            + self.vars.me.speed.y * self.vars.me.speed.y;

        if let Ok(map_tile) = MapTile::try_from(map_tile) {
            use MapTile as M;

            match map_tile {
                M::Lift => {
                    if myspeed2 <= 1.0
                        && (self.vars.me.status == Status::Activate
                            || (self.global.game_config.takeover_activates
                                && self.vars.me.status == Status::Transfermode))
                    {
                        let cx = x.round() - x;
                        let cy = y.round() - y;

                        if cx * cx + cy * cy < self.global.droid_radius * self.global.droid_radius {
                            self.enter_lift();
                        }
                    }
                }

                M::KonsoleR | M::KonsoleL | M::KonsoleO | M::KonsoleU => {
                    if myspeed2 <= 1.0
                        && (self.vars.me.status == Status::Activate
                            || (self.global.game_config.takeover_activates
                                && self.vars.me.status == Status::Transfermode))
                    {
                        self.enter_konsole();
                    }
                }
                M::Refresh1 | M::Refresh2 | M::Refresh3 | M::Refresh4 => self.refresh_influencer(),
                _ => {}
            }
        }
    }

    pub fn get_current_lift(&self) -> i32 {
        let curlev = self.main.cur_level().levelnum;

        #[allow(clippy::cast_possible_truncation)]
        let [gx, gy] = {
            let gx = self.vars.me.pos.x.round() as i32;
            let gy = self.vars.me.pos.y.round() as i32;
            [gx, gy]
        };

        info!("curlev={} gx={} gy={}", curlev, gx, gy);
        info!("List of elevators:");
        for (i, lift) in self.main.cur_ship.lifts.iter().enumerate() {
            info!(
                "Index={} level={} gx={} gy={}",
                i, lift.level, lift.x, lift.y
            );
        }

        self.main
            .cur_ship
            .lifts
            .iter()
            .position(|lift| lift.level == curlev && lift.x == gx && lift.y == gy)
            .map_or(-1, |index| index.try_into().unwrap())
    }
}

fn read_tagged_generic<'a, F, T>(s: &'a [u8], tag: &str, f: F) -> T
where
    F: Fn(&'a [u8]) -> IResult<&'a [u8], T, ()>,
{
    let pos = s
        .windows(tag.len())
        .enumerate()
        .find(|&(_, s)| s == tag.as_bytes())
        .unwrap()
        .0;

    whitespace::<_, ()>
        .and(f)
        .parse(&s[(pos + tag.len())..])
        .map(|(_, (_, n))| n)
        .unwrap()
}

#[inline]
fn read_tagged_i32(s: &[u8], tag: &str) -> i32 {
    read_tagged_generic(s, tag, nom::character::complete::i32)
}

#[inline]
fn read_tagged_u8(s: &[u8], tag: &str) -> u8 {
    read_tagged_generic(s, tag, nom::character::complete::u8)
}

#[inline]
fn read_tagged_u16(s: &[u8], tag: &str) -> u16 {
    read_tagged_generic(s, tag, nom::character::complete::u16)
}

#[inline]
fn read_tagged_i16(s: &[u8], tag: &str) -> i16 {
    read_tagged_generic(s, tag, nom::character::complete::i16)
}

/// Returns a pointer to Map in a memory field
#[cfg(not(target_os = "android"))]
pub fn struct_to_mem(level: &mut Level) -> Box<[u8]> {
    use std::io::Write;

    let x_len = level.xlen;
    let y_len = level.ylen;

    let anz_wp = level.waypoints.len();

    /* estimate the amount of memory needed */
    let mem_amount = usize::from(x_len + 1) * usize::from(y_len)
        + anz_wp * usize::from(MAX_WP_CONNECTIONS) * 4
        + 50000; /* Map-memory; Puffer fuer Dimensionen, mark-strings .. */

    /* allocate some memory */
    let mut level_mem = vec![0; mem_amount].into_boxed_slice();
    let mut level_cursor = std::io::Cursor::new(&mut *level_mem);

    // Write the data to memory:
    // Here the levelnumber and general information about the level is written
    writeln!(level_cursor, "Levelnumber: {}", level.levelnum).unwrap();
    writeln!(level_cursor, "xlen of this level: {}", level.xlen).unwrap();
    writeln!(level_cursor, "ylen of this level: {}", level.ylen).unwrap();
    writeln!(level_cursor, "color of this level: {}", level.color.to_u8()).unwrap();
    writeln!(
        level_cursor,
        "{}{}",
        LEVEL_NAME_STRING,
        level.levelname.to_str().unwrap()
    )
    .unwrap();
    writeln!(
        level_cursor,
        "{}{}",
        LEVEL_ENTER_COMMENT_STRING,
        level.enter_comment.to_str().unwrap()
    )
    .unwrap();
    writeln!(
        level_cursor,
        "{}{}",
        BACKGROUND_SONG_NAME_STRING,
        level.background_song_name.to_str().unwrap()
    )
    .unwrap();

    // Now the beginning of the actual map data is marked:
    writeln!(level_cursor, "{MAP_BEGIN_STRING}").unwrap();

    // Now in the loop each line of map data should be saved as a whole
    for i in 0..usize::from(y_len) {
        reset_level_map(level); // make sure all doors are closed
        for j in 0..usize::from(x_len) {
            write!(level_cursor, "{:02} ", level.map[i][j] as u8).unwrap();
        }
        writeln!(level_cursor).unwrap();
    }

    // --------------------
    // The next thing we must do is write the waypoints of this level

    writeln!(level_cursor, "{WP_BEGIN_STRING}").unwrap();

    for (i, waypoint) in level.waypoints.iter().enumerate() {
        write!(
            level_cursor,
            "Nr.={:3} x={:4} y={:4}\t {}",
            i, waypoint.x, waypoint.y, CONNECTION_STRING
        )
        .unwrap();

        for &connection in &waypoint.connections {
            write!(level_cursor, "{connection:2} ").unwrap();
        }
        writeln!(level_cursor).unwrap();
    }

    writeln!(level_cursor, "{LEVEL_END_STRING}").unwrap();
    writeln!(
        level_cursor,
        "----------------------------------------------------------------------"
    )
    .unwrap();

    level_mem
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_tagged_i32_simple() {
        assert_eq!(
            read_tagged_i32(b"assd Hello=       5 World".as_slice(), "Hello="),
            5,
        );
    }
}
