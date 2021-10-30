use crate::{
    defs::{
        self, Criticality, Direction, MapTile, Status, Themed, DIRECTIONS, MAP_DIR_C, MAXWAYPOINTS,
        MAX_ALERTS_ON_LEVEL, MAX_ENEMYS_ON_SHIP, MAX_LEVELS, MAX_REFRESHES_ON_LEVEL,
    },
    menu::SHIP_EXT,
    misc::{
        dealloc_c_string, locate_string_in_data, my_random, read_and_malloc_string_from_data,
        read_value_from_string,
    },
    structs::{Finepoint, GrobPoint, Level},
    Data,
};

use cstr::cstr;
use defs::{MAX_DOORS_ON_LEVEL, MAX_WP_CONNECTIONS};
use log::{error, info, trace, warn};
use std::{
    alloc::{alloc_zeroed, dealloc, Layout},
    ffi::CStr,
    ops::Not,
    os::raw::{c_char, c_float, c_int, c_uchar, c_void},
    ptr::null_mut,
};

pub const COLOR_NAMES: [&CStr; 7] = [
    cstr!("Red"),
    cstr!("Yellow"),
    cstr!("Green"),
    cstr!("Gray"),
    cstr!("Blue"),
    cstr!("Turquoise"),
    cstr!("Dark"),
];

const WHITE_SPACE: &CStr = cstr!(" \t");

const WALLPASS: f32 = 4_f32 / 64.;

const KONSOLEPASS_X: f32 = 0.5625;
const KONSOLEPASS_Y: f32 = 0.5625;

const TUERBREITE: f32 = 6_f32 / 64.;

const V_RANDSPACE: f32 = WALLPASS;
const V_RANDBREITE: f32 = 5_f32 / 64.;
const H_RANDSPACE: f32 = WALLPASS;
const H_RANDBREITE: f32 = 5_f32 / 64.;

const AREA_NAME_STRING_C: &CStr = cstr!("Area name=\"");
const LEVEL_NAME_STRING: &str = "Name of this level=";
const LEVEL_NAME_STRING_C: &CStr = cstr!("Name of this level=");
const LEVEL_ENTER_COMMENT_STRING: &str = "Comment of the Influencer on entering this level=\"";
const LEVEL_ENTER_COMMENT_STRING_C: &CStr =
    cstr!("Comment of the Influencer on entering this level=\"");
const BACKGROUND_SONG_NAME_STRING: &str = "Name of background song for this level=";
const BACKGROUND_SONG_NAME_STRING_C: &CStr = cstr!("Name of background song for this level=");
const MAP_BEGIN_STRING: &str = "begin_map";
const MAP_BEGIN_STRING_C: &CStr = cstr!("begin_map");
const WP_BEGIN_STRING: &str = "begin_waypoints";
const WP_BEGIN_STRING_C: &CStr = cstr!("begin_waypoints");
const LEVEL_END_STRING: &str = "end_level";
const LEVEL_END_STRING_C: &CStr = cstr!("end_level");
const CONNECTION_STRING: &str = "connections: ";
const CONNECTION_STRING_C: &CStr = cstr!("connections: ");

#[derive(Debug, Default)]
pub struct Map {
    inner_wait_counter: f32,
}

pub unsafe fn get_map_brick(deck: &Level, x: c_float, y: c_float) -> c_uchar {
    let xx = x.round() as c_int;
    let yy = y.round() as c_int;

    if yy >= deck.ylen || yy < 0 || xx >= deck.xlen || xx < 0 {
        MapTile::Void as c_uchar
    } else {
        *deck.map[usize::try_from(yy).unwrap()].offset(isize::try_from(xx).unwrap()) as c_uchar
    }
}

pub unsafe fn free_level_memory(level: *mut Level) {
    if level.is_null() {
        return;
    }

    let level = &mut *level;
    drop(Vec::from_raw_parts(level.levelname as *mut u8, 20, 20));
    dealloc_c_string(level.background_song_name);
    dealloc_c_string(level.level_enter_comment);

    let xlen = level.xlen;
    level
        .map
        .iter_mut()
        .take(level.ylen as usize)
        .for_each(|&mut map| {
            dealloc(
                map as *mut u8,
                Layout::array::<i8>(usize::try_from(xlen).unwrap()).unwrap(),
            )
        });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[allow(dead_code)]
pub enum ColorNames {
    Red,
    Yellow,
    Green,
    Gray,
    Blue,
    Greenblue,
    Dark,
}

unsafe fn reset_level_map(level: &mut Level) {
    // Now in the game and in the level editor, it might have happend that some open
    // doors occur.  The make life easier for the saving routine, these doors should
    // be closed first.

    use MapTile::*;
    level.map[0..usize::try_from(level.ylen).unwrap()]
        .iter()
        .copied()
        .flat_map(|row| {
            std::slice::from_raw_parts_mut(row as *mut u8, usize::try_from(level.xlen).unwrap())
        })
        .for_each(|tile| match MapTile::try_from(*tile).unwrap() {
            VZutuere | VHalbtuere1 | VHalbtuere2 | VHalbtuere3 | VGanztuere => {
                *tile = VZutuere as u8
            }
            HZutuere | HHalbtuere1 | HHalbtuere2 | HHalbtuere3 | HGanztuere => {
                *tile = HZutuere as u8
            }
            Refresh1 | Refresh2 | Refresh3 | Refresh4 => *tile = Refresh1 as u8,
            AlertGreen | AlertYellow | AlertAmber | AlertRed => *tile = AlertGreen as u8,
            _ => {}
        });
}

/// initialize doors, refreshes and lifts for the given level-data
pub unsafe fn interpret_map(level: &mut Level) -> c_int {
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
pub unsafe fn get_doors(level: &mut Level) -> c_int {
    let mut curdoor = 0;

    let xlen = level.xlen;
    let ylen = level.ylen;

    /* init Doors- Array to 0 */
    level.doors.fill(GrobPoint { x: -1, y: -1 });

    /* now find the doors */
    for line in 0..i8::try_from(ylen).unwrap() {
        for col in 0..i8::try_from(xlen).unwrap() {
            let brick = *level.map[usize::try_from(line).unwrap()].add(col.try_into().unwrap());
            if brick == MapTile::VZutuere as i8 || brick == MapTile::HZutuere as i8 {
                level.doors[curdoor].x = col;
                level.doors[curdoor].y = line;
                curdoor += 1;

                assert!(
                    !(curdoor > MAX_DOORS_ON_LEVEL),
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
pub unsafe fn get_refreshes(level: &mut Level) -> c_int {
    let xlen = level.xlen;
    let ylen = level.ylen;

    /* init refreshes array to -1 */
    level.refreshes.fill(GrobPoint { x: -1, y: -1 });

    let mut curref = 0;
    /* now find all the refreshes */
    for row in 0..u8::try_from(ylen).unwrap() {
        for col in 0..u8::try_from(xlen).unwrap() {
            if *level.map[usize::from(row)].add(col.into()) == MapTile::Refresh1 as i8 {
                level.refreshes[curref].x = col.try_into().unwrap();
                level.refreshes[curref].y = row.try_into().unwrap();
                curref += 1;

                assert!(
                    !(curref > MAX_REFRESHES_ON_LEVEL),
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
pub unsafe fn get_alerts(level: &mut Level) {
    let xlen = level.xlen;
    let ylen = level.ylen;

    // init alert array to -1
    level.alerts.fill(GrobPoint { x: -1, y: -1 });

    // now find all the alerts
    let mut curref = 0;
    for row in 0..u8::try_from(ylen).unwrap() {
        for col in 0..u8::try_from(xlen).unwrap() {
            if *level.map[usize::from(row)].add(col.into()) == MapTile::AlertGreen as i8 {
                level.alerts[curref].x = col.try_into().unwrap();
                level.alerts[curref].y = row.try_into().unwrap();
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

/// This function is for LOADING map data!
/// This function extracts the data from *data and writes them
/// into a Level-struct:
///
/// Doors and Waypoints Arrays are initialized too
pub unsafe fn level_to_struct(data: *mut c_char) -> *mut Level {
    /* Get the memory for one level */
    let loadlevel_ptr = alloc_zeroed(Layout::new::<Level>()) as *mut Level;
    let loadlevel = &mut *loadlevel_ptr;

    loadlevel.empty = false.into();

    info!("Starting to process information for another level:");

    /* Read Header Data: levelnum and x/ylen */
    let data_pointer = libc::strstr(data, cstr!("Levelnumber:").as_ptr() as *mut c_char);
    assert!(
        !data_pointer.is_null(),
        "No Levelnumber entry found! Terminating! "
    );
    libc::sscanf(
        data_pointer,
        cstr!(
            "Levelnumber: %u \n xlen of this level: %u \n ylen of this level: %u \n color of this \
             level: %u"
        )
        .as_ptr(),
        &mut (loadlevel.levelnum) as *mut _ as *mut c_void,
        &mut (loadlevel.xlen) as *mut _ as *mut c_void,
        &mut (loadlevel.ylen) as *mut _ as *mut c_void,
        &mut (loadlevel.color) as *mut _ as *mut c_void,
    );

    info!("Levelnumber : {} ", loadlevel.levelnum);
    info!("xlen of this level: {} ", loadlevel.xlen);
    info!("ylen of this level: {} ", loadlevel.ylen);
    info!("color of this level: {} ", loadlevel.ylen);

    loadlevel.levelname = read_and_malloc_string_from_data(
        data,
        LEVEL_NAME_STRING_C.as_ptr() as *mut c_char,
        cstr!("\n").as_ptr() as *mut c_char,
    );
    loadlevel.background_song_name = read_and_malloc_string_from_data(
        data,
        BACKGROUND_SONG_NAME_STRING_C.as_ptr() as *mut c_char,
        cstr!("\n").as_ptr() as *mut c_char,
    );
    loadlevel.level_enter_comment = read_and_malloc_string_from_data(
        data,
        LEVEL_ENTER_COMMENT_STRING_C.as_ptr() as *mut c_char,
        cstr!("\n").as_ptr() as *mut c_char,
    );

    // find the map data
    let map_begin = libc::strstr(data, MAP_BEGIN_STRING_C.as_ptr());
    if map_begin.is_null() {
        return null_mut();
    }

    /* set position to Waypoint-Data */
    let wp_begin = libc::strstr(data, WP_BEGIN_STRING_C.as_ptr());
    if wp_begin.is_null() {
        return null_mut();
    }

    // find end of level-data
    let level_end = libc::strstr(data, LEVEL_END_STRING_C.as_ptr());
    if level_end.is_null() {
        return null_mut();
    }

    /* now scan the map */
    let mut next_line = map_begin;
    libc::strtok(next_line, cstr!("\n").as_ptr());

    /* read MapData */
    for i in 0..usize::try_from(loadlevel.ylen).unwrap() {
        let this_line = libc::strtok(null_mut(), cstr!("\n").as_ptr());
        if this_line.is_null() {
            return null_mut();
        }
        loadlevel.map[i] =
            alloc_zeroed(Layout::array::<i8>(loadlevel.xlen.try_into().unwrap()).unwrap())
                as *mut c_char;
        let mut pos = this_line;
        pos = pos.add(libc::strspn(pos, WHITE_SPACE.as_ptr())); // skip initial whitespace

        for k in 0..usize::try_from(loadlevel.xlen).unwrap() {
            if *pos == 0 {
                return null_mut();
            }
            let mut tmp: c_int = 0;
            let res = libc::sscanf(pos, cstr!("%d").as_ptr(), &mut tmp as *mut _ as *mut c_void);
            *(loadlevel.map[i].add(k)) = tmp as c_char;
            if res == 0 || res == libc::EOF {
                return null_mut();
            }
            pos = pos.add(libc::strcspn(pos, WHITE_SPACE.as_ptr())); // skip last token
            pos = pos.add(libc::strspn(pos, WHITE_SPACE.as_ptr())); // skip initial whitespace of next one
        }
    }

    /* Get Waypoints */
    next_line = wp_begin;
    libc::strtok(next_line, cstr!("\n").as_ptr());

    for i in 0..MAXWAYPOINTS {
        let this_line = libc::strtok(null_mut(), cstr!("\n").as_ptr());
        if this_line.is_null() {
            return null_mut();
        }
        if this_line == level_end {
            loadlevel.num_waypoints = i.try_into().unwrap();
            break;
        }

        let mut nr: c_int = 0;
        let mut x: c_int = 0;
        let mut y: c_int = 0;
        libc::sscanf(
            this_line,
            cstr!("Nr.=%d \t x=%d \t y=%d").as_ptr(),
            &mut nr as *mut _ as *mut c_void,
            &mut x as *mut _ as *mut c_void,
            &mut y as *mut _ as *mut c_void,
        );

        loadlevel.all_waypoints[i].x = x.try_into().unwrap();
        loadlevel.all_waypoints[i].y = y.try_into().unwrap();

        let mut pos = libc::strstr(this_line, CONNECTION_STRING_C.as_ptr());
        pos = pos.add(CONNECTION_STRING_C.to_bytes().len()); // skip connection-string
        pos = pos.add(libc::strspn(pos, WHITE_SPACE.as_ptr())); // skip initial whitespace

        let mut k = 0;
        while k < MAX_WP_CONNECTIONS {
            if *pos == 0 {
                break;
            }
            let mut connection: c_int = 0;
            let res = libc::sscanf(
                pos,
                cstr!("%d").as_ptr(),
                &mut connection as *mut _ as *mut c_void,
            );
            if connection == -1 || res == 0 || res == libc::EOF {
                break;
            }
            loadlevel.all_waypoints[i].connections[k] = connection;

            pos = pos.add(libc::strcspn(pos, WHITE_SPACE.as_ptr())); // skip last token
            pos = pos.add(libc::strspn(pos, WHITE_SPACE.as_ptr())); // skip initial whitespace for next one

            k += 1;
        }

        loadlevel.all_waypoints[i].num_connections = k.try_into().unwrap();
    }

    loadlevel_ptr
}

impl Data<'_> {
    /// Determines wether object on x/y is visible to the 001 or not
    pub unsafe fn is_visible(&self, objpos: &Finepoint) -> c_int {
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

        let mut testpos = *objpos;

        let step_num = step_num as i32;
        for _ in 1..step_num {
            testpos.x += step.x;
            testpos.y += step.y;

            if self.is_passable(testpos.x, testpos.y, Direction::Light as i32)
                != Direction::Center as i32
            {
                return false.into();
            }
        }

        true.into()
    }

    pub unsafe fn free_ship_memory(&mut self) {
        self.main
            .cur_ship
            .all_levels
            .iter_mut()
            .take(usize::try_from(self.main.cur_ship.num_levels).unwrap())
            .map(|&mut level| level as *mut Level)
            .for_each(|level| {
                free_level_memory(level);
                dealloc(level as *mut u8, Layout::new::<Level>());
            });
    }

    pub unsafe fn animate_refresh(&mut self) {
        self.map.inner_wait_counter += self.frame_time() * 10.;

        let cur_level = &*self.main.cur_level;
        cur_level
            .refreshes
            .iter()
            .take(MAX_REFRESHES_ON_LEVEL)
            .take_while(|refresh| refresh.x != -1 && refresh.y != -1)
            .for_each(|refresh| {
                let x = isize::try_from(refresh.x).unwrap();
                let y = usize::try_from(refresh.y).unwrap();

                *cur_level.map[y].offset(x) = (((self.map.inner_wait_counter.round() as c_int) % 4)
                    + MapTile::Refresh1 as c_int)
                    as c_char;
            });
    }

    pub unsafe fn is_passable(&self, x: c_float, y: c_float, check_pos: c_int) -> c_int {
        let map_brick = get_map_brick(&*self.main.cur_level, x, y);

        let fx = (x - 0.5) - (x - 0.5).floor();
        let fy = (y - 0.5) - (y - 0.5).floor();

        let map_tile = match MapTile::try_from(map_brick) {
            Ok(map_tile) => map_tile,
            Err(_) => return -1,
        };

        use Direction::*;
        use MapTile::*;
        match map_tile {
            Floor | Lift | Void | Block4 | Block5 | Refresh1 | Refresh2 | Refresh3 | Refresh4
            | FineGrid => {
                Center as c_int /* these are passable */
            }

            AlertGreen | AlertYellow | AlertAmber | AlertRed => {
                if check_pos.try_into() == Ok(Light) {
                    Center as c_int
                } else {
                    -1
                }
            }

            KonsoleL => {
                if check_pos.try_into() == Ok(Light) || fx > 1.0 - KONSOLEPASS_X {
                    Center as c_int
                } else {
                    -1
                }
            }

            KonsoleR => {
                if check_pos.try_into() == Ok(Light) || fx < KONSOLEPASS_X {
                    Center as c_int
                } else {
                    -1
                }
            }

            KonsoleO => {
                if check_pos.try_into() == Ok(Light) || fy > 1. - KONSOLEPASS_Y {
                    Center as c_int
                } else {
                    -1
                }
            }

            KonsoleU => {
                if check_pos.try_into() == Ok(Light) || fy < KONSOLEPASS_Y {
                    Center as c_int
                } else {
                    -1
                }
            }

            HWall => {
                if (WALLPASS..=1. - WALLPASS).contains(&fy).not() {
                    Center as c_int
                } else {
                    -1
                }
            }

            VWall => {
                if (WALLPASS..=1. - WALLPASS).contains(&fx).not() {
                    Center as c_int
                } else {
                    -1
                }
            }

            EckRo => {
                if fx > 1. - WALLPASS || fy < WALLPASS || (fx < WALLPASS && fy > 1. - WALLPASS) {
                    Center as c_int
                } else {
                    -1
                }
            }

            EckRu => {
                if fx > 1. - WALLPASS || fy > 1. - WALLPASS || (fx < WALLPASS && fy < WALLPASS) {
                    Center as c_int
                } else {
                    -1
                }
            }

            EckLu => {
                if fx < WALLPASS || fy > 1. - WALLPASS || (fx > 1. - WALLPASS && fy < WALLPASS) {
                    Center as c_int
                } else {
                    -1
                }
            }

            EckLo => {
                if fx < WALLPASS || fy < WALLPASS || (fx > 1. - WALLPASS && fy > 1. - WALLPASS) {
                    Center as c_int
                } else {
                    -1
                }
            }

            To => {
                if fy < WALLPASS
                    || (fy > 1. - WALLPASS && (WALLPASS..=1. - WALLPASS).contains(&fx).not())
                {
                    Center as c_int
                } else {
                    -1
                }
            }

            Tr => {
                if fx > 1. - WALLPASS
                    || (fx < WALLPASS && (WALLPASS..=1. - WALLPASS).contains(&fy).not())
                {
                    Center as c_int
                } else {
                    -1
                }
            }

            Tu => {
                if fy > 1. - WALLPASS
                    || (fy < WALLPASS && (WALLPASS..=1. - WALLPASS).contains(&fx).not())
                {
                    Center as c_int
                } else {
                    -1
                }
            }

            Tl => {
                if fx < WALLPASS
                    || (fx > 1. - WALLPASS && (WALLPASS..=1. - WALLPASS).contains(&fy).not())
                {
                    Center as c_int
                } else {
                    -1
                }
            }

            HGanztuere | HHalbtuere3 | HHalbtuere2 if (check_pos.try_into() == Ok(Light)) => {
                Center as c_int
            }
            HHalbtuere1 | HZutuere if (check_pos.try_into() == Ok(Light)) => -1,

            HGanztuere | HHalbtuere3 | HHalbtuere2 | HHalbtuere1 | HZutuere => {
                if (H_RANDBREITE..=1. - H_RANDBREITE).contains(&fx).not()
                    && (H_RANDSPACE..=1. - H_RANDSPACE).contains(&fy)
                {
                    let check_pos = match check_pos.try_into() {
                        Ok(check_pos) => check_pos,
                        Err(_) => return -1,
                    };
                    if check_pos != Center && check_pos != Light && self.vars.me.speed.y != 0. {
                        match check_pos {
                            Rechtsoben | Rechtsunten | Rechts => {
                                if fx > 1. - H_RANDBREITE {
                                    Links as c_int
                                } else {
                                    -1
                                }
                            }
                            Linksoben | Linksunten | Links => {
                                if fx < H_RANDBREITE {
                                    Rechts as c_int
                                } else {
                                    -1
                                }
                            }
                            _ => -1, /* switch check_pos */
                        }
                    }
                    /* if DRUID && Me.speed.y != 0 */
                    else {
                        -1
                    }
                } else if map_tile == HGanztuere
                    || map_tile == HHalbtuere3
                    || fy < TUERBREITE
                    || fy > 1. - TUERBREITE
                {
                    Center as c_int
                } else {
                    -1
                }
            }
            VGanztuere | VHalbtuere3 | VHalbtuere2 if (check_pos.try_into() == Ok(Light)) => {
                Center as c_int
            }

            VHalbtuere1 | VZutuere if (check_pos.try_into() == Ok(Light)) => -1,
            VGanztuere | VHalbtuere3 | VHalbtuere2 | VHalbtuere1 | VZutuere => {
                if (V_RANDBREITE..=1. - V_RANDBREITE).contains(&fy).not()
                    && (V_RANDSPACE..=1. - V_RANDSPACE).contains(&fx)
                {
                    let check_pos = match check_pos.try_into() {
                        Ok(check_pos) => check_pos,
                        Err(_) => return -1,
                    };
                    if check_pos != Center && check_pos != Light && self.vars.me.speed.x != 0. {
                        match check_pos {
                            Rechtsoben | Linksoben | Oben => {
                                if fy < V_RANDBREITE {
                                    Unten as c_int
                                } else {
                                    -1
                                }
                            }
                            Rechtsunten | Linksunten | Unten => {
                                if fy > 1. - V_RANDBREITE {
                                    Oben as c_int
                                } else {
                                    -1
                                }
                            }
                            _ => -1,
                        }
                    } else {
                        -1
                    }
                } else if map_tile == VGanztuere
                    || map_tile == VHalbtuere3
                    || fx < TUERBREITE
                    || fx > 1. - TUERBREITE
                {
                    Center as c_int
                } else {
                    -1
                }
            }
            _ => -1,
        }
    }

    /// Saves ship-data to disk
    pub unsafe fn save_ship(&mut self, shipname: *const c_char) -> c_int {
        use std::{fs::File, io::Write, path::PathBuf};

        trace!("SaveShip(): real function call confirmed.");

        let filename = PathBuf::from(format!(
            "{}{}",
            CStr::from_ptr(shipname).to_str().unwrap(),
            SHIP_EXT
        ));

        /* count the levels */
        let level_anz = self
            .main
            .cur_ship
            .all_levels
            .iter()
            .take_while(|level| level.is_null().not())
            .count();

        trace!("SaveShip(): now opening the ship file...");

        let mut ship_file = match File::create(filename) {
            Ok(file) => file,
            Err(err) => {
                panic!("Error opening ship file: {}. Terminating", err);
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

            ship_file.write_all(MAP_HEADER_STRING.as_bytes())?;

            const AREA_NAME_STRING: &str = "Area name=\"";
            ship_file.write_all(AREA_NAME_STRING.as_bytes())?;
            ship_file
                .write_all(CStr::from_ptr(self.main.cur_ship.area_name.as_ptr()).to_bytes())?;
            ship_file.write_all(b"\"\n\n  ")?;

            /* Save all Levels */

            trace!("SaveShip(): now saving levels...");

            for i in 0..i32::try_from(level_anz).unwrap() {
                let mut level_iter = self
                    .main
                    .cur_ship
                    .all_levels
                    .iter()
                    .copied()
                    .take_while(|level| level.is_null().not())
                    .filter(|&level| (*level).levelnum == i);

                let level = match level_iter.next() {
                    Some(level) => level,
                    None => {
                        panic!("Missing Levelnumber error in SaveShip.");
                    }
                };

                assert!(
                    !level_iter.next().is_some(),
                    "Identical Levelnumber Error in SaveShip."
                );

                //--------------------
                // Now comes the real saving part FOR ONE LEVEL.  First THE LEVEL is packed into a string and
                // then this string is wirtten to the file.  easy. simple.
                let level_mem = self.struct_to_mem(level);
                ship_file.write_all(CStr::from_ptr(level_mem).to_bytes())?;

                let mem_amount = usize::try_from((*level).xlen + 1).unwrap()
                    * usize::try_from((*level).ylen).unwrap()
                    + usize::try_from((*level).num_waypoints).unwrap() * MAX_WP_CONNECTIONS * 4
                    + 50000;
                dealloc(
                    level_mem as *mut u8,
                    Layout::array::<u8>(mem_amount).unwrap(),
                );
            }

            //--------------------
            // Now we are almost done writing.  Everything that is missing is
            // the termination string for the ship file.  This termination string
            // is needed later for the ship loading functions to find the end of
            // the data and to be able to terminate the long file-string with a
            // null character at the right position.
            //
            const END_OF_SHIP_DATA_STRING: &str = "*** End of Ship Data ***";
            writeln!(ship_file, "{}\n", END_OF_SHIP_DATA_STRING)?;

            trace!("SaveShip(): now flushing ship file...");
            ship_file.flush()?;

            trace!("SaveShip(): end of function reached.");
            Ok(())
        })();

        match result {
            Ok(()) => defs::OK.into(),
            Err(err) => {
                panic!("Error writing to ship file: {}. Terminating", err);
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
    pub unsafe fn move_level_doors(&mut self) {
        // This prevents animation going too quick.
        // The constant should be replaced by a variable, that can be
        // set from within the theme, but that may be done later...
        if self.global.level_doors_not_moved_time < self.global.time_for_each_phase_of_door_movement
        {
            return;
        }
        self.global.level_doors_not_moved_time = 0.;

        let cur_level = &*self.main.cur_level;
        for i in 0..MAX_DOORS_ON_LEVEL {
            let doorx = cur_level.doors[i].x;
            let doory = cur_level.doors[i].y;

            /* Keine weiteren Tueren */
            if doorx == -1 && doory == -1 {
                break;
            }

            let pos =
                cur_level.map[usize::try_from(doory).unwrap()].add(usize::try_from(doorx).unwrap());

            // NORMALISATION doorx = doorx * Block_Rect.w + Block_Rect.w / 2;
            // NORMALISATION doory = doory * Block_Rect.h + Block_Rect.h / 2;

            /* first check Influencer gegen Tuer */
            let xdist = self.vars.me.pos.x - f32::from(doorx);
            let ydist = self.vars.me.pos.y - f32::from(doory);
            let dist2 = xdist * xdist + ydist * ydist;

            const DOOROPENDIST2: f32 = 1.;
            if dist2 < DOOROPENDIST2 {
                if *pos != MapTile::HGanztuere as i8 && *pos != MapTile::VGanztuere as i8 {
                    *pos += 1;
                }
            } else {
                /* alle Enemys checken */
                let mut j = 0;
                while j < usize::try_from(self.main.num_enemys).unwrap() {
                    /* ignore druids that are dead or on other levels */
                    if self.main.all_enemys[j].status == Status::Out as i32
                        || self.main.all_enemys[j].status == Status::Terminated as i32
                        || self.main.all_enemys[j].levelnum != cur_level.levelnum
                    {
                        j += 1;
                        continue;
                    }

                    let xdist = (self.main.all_enemys[j].pos.x - f32::from(doorx))
                        .trunc()
                        .abs();
                    if xdist < self.vars.block_rect.w.into() {
                        let ydist = (self.main.all_enemys[j].pos.y - f32::from(doory))
                            .trunc()
                            .abs();
                        if ydist < self.vars.block_rect.h.into() {
                            let dist2 = xdist * xdist + ydist * ydist;
                            if dist2 < DOOROPENDIST2 {
                                if *pos != MapTile::HGanztuere as i8
                                    && *pos != MapTile::VGanztuere as i8
                                {
                                    *pos += 1;
                                }

                                break; /* one druid is enough to open a door */
                            }
                        }
                    }

                    j += 1;
                }

                /* No druid near: close door if it isnt closed */
                if j == usize::try_from(self.main.num_enemys).unwrap()
                    && *pos != MapTile::VZutuere as i8
                    && *pos != MapTile::HZutuere as i8
                {
                    *pos -= 1;
                }
            }
        }
    }

    /// Returns a pointer to Map in a memory field
    pub unsafe fn struct_to_mem(&mut self, level: *mut Level) -> *mut c_char {
        use std::io::Write;

        let level = &mut *level;
        let xlen = level.xlen;
        let ylen = level.ylen;

        let anz_wp = usize::try_from(level.num_waypoints).unwrap();

        /* estimate the amount of memory needed */
        let mem_amount = usize::try_from(xlen + 1).unwrap() * usize::try_from(ylen).unwrap()
            + anz_wp * MAX_WP_CONNECTIONS * 4
            + 50000; /* Map-memory; Puffer fuer Dimensionen, mark-strings .. */

        /* allocate some memory */
        let level_mem = alloc_zeroed(Layout::array::<u8>(mem_amount).unwrap());
        assert!(
            !level_mem.is_null(),
            "could not allocate memory, terminating."
        );
        let mut level_cursor =
            std::io::Cursor::new(std::slice::from_raw_parts_mut(level_mem, mem_amount));

        // Write the data to memory:
        // Here the levelnumber and general information about the level is written
        writeln!(level_cursor, "Levelnumber: {}", level.levelnum).unwrap();
        writeln!(level_cursor, "xlen of this level: {}", level.xlen).unwrap();
        writeln!(level_cursor, "ylen of this level: {}", level.ylen).unwrap();
        writeln!(level_cursor, "color of this level: {}", level.color).unwrap();
        writeln!(
            level_cursor,
            "{}{}",
            LEVEL_NAME_STRING,
            CStr::from_ptr(level.levelname).to_str().unwrap()
        )
        .unwrap();
        writeln!(
            level_cursor,
            "{}{}",
            LEVEL_ENTER_COMMENT_STRING,
            CStr::from_ptr(level.level_enter_comment).to_str().unwrap()
        )
        .unwrap();
        writeln!(
            level_cursor,
            "{}{}",
            BACKGROUND_SONG_NAME_STRING,
            CStr::from_ptr(level.background_song_name).to_str().unwrap()
        )
        .unwrap();

        // Now the beginning of the actual map data is marked:
        writeln!(level_cursor, "{}", MAP_BEGIN_STRING).unwrap();

        // Now in the loop each line of map data should be saved as a whole
        for i in 0..usize::try_from(ylen).unwrap() {
            reset_level_map(level); // make sure all doors are closed
            for j in 0..usize::try_from(xlen).unwrap() {
                write!(level_cursor, "{:02} ", *level.map[i].add(j)).unwrap();
            }
            writeln!(level_cursor).unwrap();
        }

        // --------------------
        // The next thing we must do is write the waypoints of this level

        writeln!(level_cursor, "{}", WP_BEGIN_STRING).unwrap();

        for i in 0..usize::try_from(level.num_waypoints).unwrap() {
            write!(
                level_cursor,
                "Nr.={:3} x={:4} y={:4}\t {}",
                i, level.all_waypoints[i].x, level.all_waypoints[i].y, CONNECTION_STRING
            )
            .unwrap();

            let this_wp = &level.all_waypoints[i];
            for j in 0..usize::try_from(this_wp.num_connections).unwrap() {
                write!(level_cursor, "{:2} ", this_wp.connections[j]).unwrap();
            }
            writeln!(level_cursor).unwrap();
        }

        writeln!(level_cursor, "{}", LEVEL_END_STRING).unwrap();
        writeln!(
            level_cursor,
            "----------------------------------------------------------------------"
        )
        .unwrap();

        level_mem as *mut c_char
    }

    pub unsafe fn druid_passable(&self, x: c_float, y: c_float) -> c_int {
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
                    c_int::try_from(direction_index).unwrap(),
                )
            })
            .find(|&is_passable| is_passable != Direction::Center as c_int)
            .unwrap_or(Direction::Center as c_int)
    }

    /// This function receives a pointer to the already read in crew section
    /// in a already read in droids file and decodes all the contents of that
    /// droid section to fill the AllEnemys array with droid types accoriding
    /// to the specifications made in the file.
    pub unsafe fn get_this_levels_droids(&mut self, section_pointer: *mut c_char) {
        const DROIDS_LEVEL_INDICATION_STRING: &CStr = cstr!("Level=");
        const DROIDS_LEVEL_END_INDICATION_STRING: &CStr =
            cstr!("** End of this levels droid data **");
        const DROIDS_MAXRAND_INDICATION_STRING: &CStr = cstr!("Maximum number of Random Droids=");
        const DROIDS_MINRAND_INDICATION_STRING: &CStr = cstr!("Minimum number of Random Droids=");
        const ALLOWED_TYPE_INDICATION_STRING: &CStr =
            cstr!("Allowed Type of Random Droid for this level: ");

        let end_of_this_level_data = locate_string_in_data(
            section_pointer,
            DROIDS_LEVEL_END_INDICATION_STRING.as_ptr() as *mut c_char,
        );
        *end_of_this_level_data = 0;

        // Now we read in the level number for this level
        let mut our_level_number: c_int = 0;
        read_value_from_string(
            section_pointer,
            DROIDS_LEVEL_INDICATION_STRING.as_ptr() as *mut c_char,
            cstr!("%d").as_ptr() as *mut c_char,
            &mut our_level_number as *mut c_int as *mut c_void,
        );

        // Now we read in the maximal number of random droids for this level
        let mut max_rand: c_int = 0;
        read_value_from_string(
            section_pointer,
            DROIDS_MAXRAND_INDICATION_STRING.as_ptr() as *mut c_char,
            cstr!("%d").as_ptr() as *mut c_char,
            &mut max_rand as *mut c_int as *mut c_void,
        );

        // Now we read in the minimal number of random droids for this level
        let mut min_rand: c_int = 0;
        read_value_from_string(
            section_pointer,
            DROIDS_MINRAND_INDICATION_STRING.as_ptr() as *mut c_char,
            cstr!("%d").as_ptr() as *mut c_char,
            &mut min_rand as *mut c_int as *mut c_void,
        );

        let mut different_random_types = 0;
        let mut search_pointer = libc::strstr(
            section_pointer,
            ALLOWED_TYPE_INDICATION_STRING.as_ptr() as *mut c_char,
        );
        let mut type_indication_string: [c_char; 1000] = [0; 1000];
        let mut list_of_types_allowed: [c_int; 1000] = [0; 1000];
        while search_pointer.is_null().not() {
            search_pointer = search_pointer.add(ALLOWED_TYPE_INDICATION_STRING.to_bytes().len());
            libc::strncpy(type_indication_string.as_mut_ptr(), search_pointer, 3); // Every type is 3 characters long
            type_indication_string[3] = 0;
            // Now that we have got a type indication string, we only need to translate it
            // into a number corresponding to that droid in the droid list
            let mut list_index = 0;
            while list_index < self.main.number_of_droid_types {
                if libc::strcmp(
                    (*self.vars.droidmap.add(usize::try_from(list_index).unwrap()))
                        .druidname
                        .as_ptr(),
                    type_indication_string.as_ptr(),
                ) == 0
                {
                    break;
                }
                list_index += 1;
            }
            if list_index >= self.main.number_of_droid_types {
                panic!(
                    "unknown droid type: {} found in data file for level {}",
                    CStr::from_ptr(type_indication_string.as_ptr()).to_string_lossy(),
                    our_level_number,
                );
            } else {
                info!(
                    "Type indication string {} translated to type Nr.{}.",
                    CStr::from_ptr(type_indication_string.as_ptr()).to_string_lossy(),
                    list_index,
                );
            }
            list_of_types_allowed[different_random_types] = list_index;
            different_random_types += 1;

            search_pointer = libc::strstr(
                search_pointer,
                ALLOWED_TYPE_INDICATION_STRING.as_ptr() as *mut c_char,
            )
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

        let mut real_number_of_random_droids = my_random(max_rand - min_rand) + min_rand;

        while real_number_of_random_droids > 0 {
            real_number_of_random_droids -= 1;

            let mut free_all_enemys_position = 0;
            while free_all_enemys_position < MAX_ENEMYS_ON_SHIP {
                if self.main.all_enemys[free_all_enemys_position].status == Status::Out as c_int {
                    break;
                }
                free_all_enemys_position += 1;
            }

            assert!(
                !(free_all_enemys_position == MAX_ENEMYS_ON_SHIP),
                "No more free position to fill random droids into in GetCrew...Terminating...."
            );

            self.main.all_enemys[free_all_enemys_position].ty = list_of_types_allowed
                [usize::try_from(my_random(
                    c_int::try_from(different_random_types).unwrap() - 1,
                ))
                .unwrap()];
            self.main.all_enemys[free_all_enemys_position].levelnum = our_level_number;
            self.main.all_enemys[free_all_enemys_position].status = Status::Mobile as c_int;
        }
    }

    /// This function initializes all enemys
    pub unsafe fn get_crew(&mut self, filename: *mut c_char) -> c_int {
        const END_OF_DROID_DATA_STRING: &CStr = cstr!("*** End of Droid Data ***");
        const DROIDS_LEVEL_DESCRIPTION_START_STRING: &CStr = cstr!("** Beginning of new Level **");
        const DROIDS_LEVEL_DESCRIPTION_END_STRING: &CStr =
            cstr!("** End of this levels droid data **");

        /* Clear Enemy - Array */
        self.clear_enemys();

        //Now its time to start decoding the droids file.
        //For that, we must get it into memory first.
        //The procedure is the same as with LoadShip
        let fpath = self.find_file(
            filename,
            MAP_DIR_C.as_ptr() as *mut c_char,
            Themed::NoTheme as c_int,
            Criticality::Critical as c_int,
        );

        let main_droids_file_pointer = self.read_and_malloc_and_terminate_file(
            fpath,
            END_OF_DROID_DATA_STRING.as_ptr() as *mut c_char,
        );

        // The Droid crew file for this map is now completely read into memory
        // It's now time to decode the file and to fill the array of enemys with
        // new droids of the given types.
        let mut droid_section_pointer = libc::strstr(
            main_droids_file_pointer.as_ptr(),
            DROIDS_LEVEL_DESCRIPTION_START_STRING.as_ptr() as *mut c_char,
        );
        while droid_section_pointer.is_null().not() {
            droid_section_pointer = droid_section_pointer.add(libc::strlen(
                DROIDS_LEVEL_DESCRIPTION_START_STRING.as_ptr() as *mut c_char,
            ));
            info!("Found another levels droids description starting point entry!");
            let end_of_this_droid_section_pointer = libc::strstr(
                droid_section_pointer,
                DROIDS_LEVEL_DESCRIPTION_END_STRING.as_ptr() as *mut c_char,
            );
            assert!(
                !end_of_this_droid_section_pointer.is_null(),
                "GetCrew: Unterminated droid section encountered!! Terminating."
            );
            self.get_this_levels_droids(droid_section_pointer);
            droid_section_pointer = end_of_this_droid_section_pointer.add(2); // Move past the inserted String terminator

            droid_section_pointer = libc::strstr(
                droid_section_pointer,
                DROIDS_LEVEL_DESCRIPTION_START_STRING.as_ptr() as *mut c_char,
            );
        }

        // Now that the correct crew types have been filled into the
        // right structure, it's time to set the energy of the corresponding
        // droids to "full" which means to the maximum of each type.
        self.main.num_enemys = 0;
        for enemy in &mut self.main.all_enemys {
            let ty = enemy.ty;
            if ty == -1 {
                // Do nothing to unused entries
                continue;
            }
            enemy.energy = (*self.vars.droidmap.add(usize::try_from(ty).unwrap())).maxenergy;
            enemy.status = Status::Mobile as c_int;
            self.main.num_enemys += 1;
        }

        defs::OK.into()
    }

    /// loads lift-connctions to cur-ship struct
    pub unsafe fn get_lift_connections(&mut self, filename: *mut c_char) -> c_int {
        macro_rules! read_int_from_string_into {
            ($ptr:expr, $str:tt, $value:expr) => {
                read_value_from_string(
                    $ptr,
                    cstr!($str).as_ptr() as *mut c_char,
                    cstr!("%d").as_ptr() as *mut c_char,
                    &mut $value as *mut _ as *mut c_void,
                );
            };
        }

        macro_rules! read_int_from_string {
            ($ptr:expr, $str:tt, $value:ident) => {
                let mut $value: c_int = 0;
                read_int_from_string_into!($ptr, $str, $value);
            };
        }

        const END_OF_LIFT_DATA_STRING: &CStr = cstr!("*** End of elevator specification file ***");
        const START_OF_LIFT_DATA_STRING: &CStr = cstr!("*** Beginning of Lift Data ***");
        const START_OF_LIFT_RECTANGLE_DATA_STRING: &CStr =
            cstr!("*** Beginning of elevator rectangles ***");

        /* Now get the lift-connection data from "FILE.elv" file */
        let fpath = self.find_file(
            filename,
            MAP_DIR_C.as_ptr() as *mut c_char,
            Themed::NoTheme as c_int,
            Criticality::Critical as c_int,
        );

        let mut data = self.read_and_malloc_and_terminate_file(
            fpath,
            END_OF_LIFT_DATA_STRING.as_ptr() as *mut c_char,
        );

        // At first we read in the rectangles that define where the colums of the
        // lift are, so that we can highlight them later.
        self.main.cur_ship.num_lift_rows = 0;
        let mut entry_pointer = locate_string_in_data(
            data.as_mut_ptr(),
            START_OF_LIFT_RECTANGLE_DATA_STRING.as_ptr() as *mut c_char,
        );
        loop {
            entry_pointer = libc::strstr(
                entry_pointer,
                cstr!("Elevator Number=").as_ptr() as *mut c_char,
            );
            if entry_pointer.is_null() {
                break;
            }

            read_int_from_string!(entry_pointer, "Elevator Number=", elevator_index);
            entry_pointer = entry_pointer.add(1);

            read_int_from_string!(entry_pointer, "ElRowX=", x);
            read_int_from_string!(entry_pointer, "ElRowY=", y);
            read_int_from_string!(entry_pointer, "ElRowW=", w);
            read_int_from_string!(entry_pointer, "ElRowH=", h);

            let rect =
                &mut self.main.cur_ship.lift_row_rect[usize::try_from(elevator_index).unwrap()];
            rect.x = x.try_into().unwrap();
            rect.y = y.try_into().unwrap();
            rect.w = w.try_into().unwrap();
            rect.h = h.try_into().unwrap();

            self.main.cur_ship.num_lift_rows += 1;
        }

        //--------------------
        // Now we read in the rectangles that define where the decks of the
        // current area system are, so that we can highlight them later in the
        // elevator and console functions.
        //
        self.main.cur_ship.num_level_rects.fill(0); // this initializes zeros for the number
        entry_pointer = data.as_mut_ptr();

        loop {
            entry_pointer = libc::strstr(entry_pointer, cstr!("DeckNr=").as_ptr() as *mut c_char);
            if entry_pointer.is_null() {
                break;
            }

            read_int_from_string!(entry_pointer, "DeckNr=", deck_index);
            read_int_from_string!(entry_pointer, "RectNumber=", rect_index);
            entry_pointer = entry_pointer.add(1); // to prevent doubly taking this entry

            self.main.cur_ship.num_level_rects[usize::try_from(deck_index).unwrap()] += 1; // count the number of rects for this deck one up

            read_int_from_string!(entry_pointer, "DeckX=", x);
            read_int_from_string!(entry_pointer, "DeckY=", y);
            read_int_from_string!(entry_pointer, "DeckW=", w);
            read_int_from_string!(entry_pointer, "DeckH=", h);

            let rect = &mut self.main.cur_ship.level_rects[usize::try_from(deck_index).unwrap()]
                [usize::try_from(rect_index).unwrap()];
            rect.x = x.try_into().unwrap();
            rect.y = y.try_into().unwrap();
            rect.w = w.try_into().unwrap();
            rect.h = h.try_into().unwrap();
        }

        entry_pointer = libc::strstr(
            data.as_ptr(),
            START_OF_LIFT_DATA_STRING.as_ptr() as *mut c_char,
        );
        assert!(
            !entry_pointer.is_null(),
            "START OF LIFT DATA STRING NOT FOUND!  Terminating..."
        );

        let mut label: c_int = 0;
        entry_pointer = data.as_mut_ptr();
        loop {
            entry_pointer = libc::strstr(entry_pointer, cstr!("Label=").as_ptr() as *mut c_char);
            if entry_pointer.is_null() {
                break;
            }

            read_int_from_string_into!(entry_pointer, "Label=", label);
            let cur_lift = &mut self.main.cur_ship.all_lifts[usize::try_from(label).unwrap()];
            entry_pointer = entry_pointer.add(1); // to avoid doubly taking this entry

            read_int_from_string_into!(entry_pointer, "Deck=", cur_lift.level);
            read_int_from_string_into!(entry_pointer, "PosX=", cur_lift.x);
            read_int_from_string_into!(entry_pointer, "PosY=", cur_lift.y);
            read_int_from_string_into!(entry_pointer, "LevelUp=", cur_lift.up);
            read_int_from_string_into!(entry_pointer, "LevelDown=", cur_lift.down);
            read_int_from_string_into!(entry_pointer, "LiftRow=", cur_lift.lift_row);
        }

        self.main.cur_ship.num_lifts = label;

        defs::OK.into()
    }

    pub unsafe fn load_ship(&mut self, filename: *mut c_char) -> c_int {
        let mut level_start: [*mut c_char; MAX_LEVELS] = [null_mut(); MAX_LEVELS];
        self.free_ship_memory(); // clear vestiges of previous ship data, if any

        /* Read the whole ship-data to memory */
        const END_OF_SHIP_DATA_STRING: &CStr = cstr!("*** End of Ship Data ***");
        let fpath = self.find_file(
            filename,
            MAP_DIR_C.as_ptr() as *mut c_char,
            Themed::NoTheme as c_int,
            Criticality::Critical as c_int,
        );
        let mut ship_data = self.read_and_malloc_and_terminate_file(
            fpath,
            END_OF_SHIP_DATA_STRING.as_ptr() as *mut c_char,
        );

        // Now we read the Area-name from the loaded data
        let buffer = read_and_malloc_string_from_data(
            ship_data.as_mut_ptr(),
            AREA_NAME_STRING_C.as_ptr() as *mut c_char,
            cstr!("\"").as_ptr() as *mut c_char,
        );
        libc::strncpy(self.main.cur_ship.area_name.as_mut_ptr(), buffer, 99);
        self.main.cur_ship.area_name[99] = 0;
        dealloc_c_string(buffer);

        // Now we count the number of levels and remember their start-addresses.
        // This is done by searching for the LEVEL_END_STRING again and again
        // until it is no longer found in the ship file.  good.

        let mut level_anz = 0;
        let mut endpt = ship_data.as_mut_ptr();
        level_start[level_anz] = ship_data.as_mut_ptr();

        loop {
            endpt = libc::strstr(endpt, LEVEL_END_STRING_C.as_ptr());
            if endpt.is_null() {
                break;
            }

            endpt = endpt.add(LEVEL_END_STRING_C.to_bytes().len());
            level_anz += 1;
            level_start[level_anz] = endpt.add(1);
        }

        /* init the level-structs */
        self.main.cur_ship.num_levels = level_anz.try_into().unwrap();

        let result = self
            .main
            .cur_ship
            .all_levels
            .iter_mut()
            .zip(level_start.iter().copied())
            .enumerate()
            .take(level_anz)
            .try_for_each(|(index, (level, start))| {
                *level = level_to_struct(start);

                if level.is_null() {
                    error!("reading of level {} failed", index);
                    return None;
                }
                interpret_map(&mut **level); // initialize doors, refreshes and lifts
                Some(())
            });
        if result.is_none() {
            return defs::ERR.into();
        }

        defs::OK.into()
    }

    /// ActSpecialField: checks Influencer on SpecialFields like
    /// Lifts and Konsoles and acts on it
    pub unsafe fn act_special_field(&mut self, x: c_float, y: c_float) {
        let map_tile = get_map_brick(&*self.main.cur_level, x, y);

        let myspeed2 = self.vars.me.speed.x * self.vars.me.speed.x
            + self.vars.me.speed.y * self.vars.me.speed.y;

        if let Ok(map_tile) = MapTile::try_from(map_tile) {
            use MapTile::*;
            match map_tile {
                Lift => {
                    if myspeed2 <= 1.0
                        && (self.vars.me.status == Status::Activate as c_int
                            || (self.global.game_config.takeover_activates != 0
                                && self.vars.me.status == Status::Transfermode as c_int))
                    {
                        let cx = x.round() - x;
                        let cy = y.round() - y;

                        if cx * cx + cy * cy < self.global.droid_radius * self.global.droid_radius {
                            self.enter_lift();
                        }
                    }
                }

                KonsoleR | KonsoleL | KonsoleO | KonsoleU => {
                    if myspeed2 <= 1.0
                        && (self.vars.me.status == Status::Activate as c_int
                            || (self.global.game_config.takeover_activates != 0
                                && self.vars.me.status == Status::Transfermode as c_int))
                    {
                        self.enter_konsole();
                    }
                }
                Refresh1 | Refresh2 | Refresh3 | Refresh4 => self.refresh_influencer(),
                _ => {}
            }
        }
    }

    pub unsafe fn get_current_lift(&self) -> c_int {
        let curlev = (*self.main.cur_level).levelnum;

        let gx = self.vars.me.pos.x.round() as c_int;
        let gy = self.vars.me.pos.y.round() as c_int;

        info!("curlev={} gx={} gy={}", curlev, gx, gy);
        info!("List of elevators:");
        for i in 0..usize::try_from(self.main.cur_ship.num_lifts).unwrap() + 1 {
            info!(
                "Index={} level={} gx={} gy={}",
                i,
                self.main.cur_ship.all_lifts[i].level,
                self.main.cur_ship.all_lifts[i].x,
                self.main.cur_ship.all_lifts[i].y
            );
        }

        let mut i = 0;
        while i < usize::try_from(self.main.cur_ship.num_lifts).unwrap() + 1
        // we check for one more than present, so the last reached
        // will really mean: NONE FOUND.
        {
            if self.main.cur_ship.all_lifts[i].level != curlev {
                i += 1;
                continue;
            }
            if self.main.cur_ship.all_lifts[i].x == gx && self.main.cur_ship.all_lifts[i].y == gy {
                break;
            }

            i += 1;
        }

        if i == usize::try_from(self.main.cur_ship.num_lifts).unwrap() + 1 {
            // none found
            -1
        } else {
            i.try_into().unwrap()
        }
    }
}
