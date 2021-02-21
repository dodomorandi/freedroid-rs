use crate::{
    curShip,
    defs::{self, Direction, MapTile, Status, MAX_REFRESHES_ON_LEVEL},
    global::{LevelDoorsNotMovedTime, Time_For_Each_Phase_Of_Door_Movement},
    menu::SHIP_EXT,
    misc::{Frame_Time, MyMalloc, Terminate},
    structs::{Finepoint, Level},
    vars::Block_Rect,
    AllEnemys, CurLevel, Me, NumEnemys,
};

use defs::{MAX_DOORS_ON_LEVEL, MAX_WP_CONNECTIONS};
use log::{error, trace};
use std::{
    convert::{TryFrom, TryInto},
    ffi::CStr,
    ops::Not,
    os::raw::{c_char, c_float, c_int, c_uchar, c_void},
};

extern "C" {
    pub static ColorNames: [*const c_char; 7];
    pub static numLevelColors: c_int;
}

const WALLPASS: f32 = 4_f32 / 64.;

const KONSOLEPASS_X: f32 = 0.5625;
const KONSOLEPASS_Y: f32 = 0.5625;

const TUERBREITE: f32 = 6_f32 / 64.;

const V_RANDSPACE: f32 = WALLPASS;
const V_RANDBREITE: f32 = 5_f32 / 64.;
const H_RANDSPACE: f32 = WALLPASS;
const H_RANDBREITE: f32 = 5_f32 / 64.;

const AREA_NAME_STRING: &str = "Area name=\"";
const LEVEL_NAME_STRING: &str = "Name of this level=";
const LEVEL_ENTER_COMMENT_STRING: &str = "Comment of the Influencer on entering this level=\"";
const BACKGROUND_SONG_NAME_STRING: &str = "Name of background song for this level=";
const MAP_BEGIN_STRING: &str = "begin_map";
const WP_BEGIN_STRING: &str = "begin_waypoints";
const LEVEL_END_STRING: &str = "end_level";
const CONNECTION_STRING: &str = "connections: ";

/// Determines wether object on x/y is visible to the 001 or not
#[no_mangle]
pub unsafe extern "C" fn IsVisible(objpos: &Finepoint) -> c_int {
    let influ_x = Me.pos.x;
    let influ_y = Me.pos.y;

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

        if IsPassable(testpos.x, testpos.y, Direction::Light as i32) != Direction::Center as i32 {
            return false.into();
        }
    }

    true.into()
}

#[no_mangle]
pub unsafe extern "C" fn GetMapBrick(deck: &Level, x: c_float, y: c_float) -> c_uchar {
    let xx = x.round() as c_int;
    let yy = y.round() as c_int;

    if yy >= deck.ylen || yy < 0 || xx >= deck.xlen || xx < 0 {
        MapTile::Void as c_uchar
    } else {
        *deck.map[usize::try_from(yy).unwrap()].offset(isize::try_from(xx).unwrap()) as c_uchar
    }
}

#[no_mangle]
pub unsafe extern "C" fn FreeShipMemory() {
    curShip
        .AllLevels
        .iter_mut()
        .take(usize::try_from(curShip.num_levels).unwrap())
        .map(|&mut level| level as *mut Level)
        .for_each(|level| {
            FreeLevelMemory(level);
            libc::free(level as *mut c_void);
        });
}

#[no_mangle]
pub unsafe extern "C" fn FreeLevelMemory(level: *mut Level) {
    if level.is_null() {
        return;
    }

    let level = &mut *level;
    libc::free(level.Levelname as *mut c_void);
    libc::free(level.Background_Song_Name as *mut c_void);
    libc::free(level.Level_Enter_Comment as *mut c_void);

    level
        .map
        .iter_mut()
        .take(level.ylen as usize)
        .map(|&mut map| map as *mut c_void)
        .for_each(|map| libc::free(map));
}

#[no_mangle]
pub unsafe extern "C" fn AnimateRefresh() {
    static mut INNER_WAIT_COUNTER: f32 = 0.;

    trace!("AnimateRefresh():  real function call confirmed.");

    INNER_WAIT_COUNTER += Frame_Time() * 10.;

    let cur_level = &*CurLevel;
    cur_level
        .refreshes
        .iter()
        .take(MAX_REFRESHES_ON_LEVEL)
        .take_while(|refresh| refresh.x != -1 && refresh.y != -1)
        .for_each(|refresh| {
            let x = isize::try_from(refresh.x).unwrap();
            let y = usize::try_from(refresh.y).unwrap();

            *cur_level.map[y].offset(x) = (((INNER_WAIT_COUNTER.round() as c_int) % 4)
                + MapTile::Refresh1 as c_int) as c_char;
        });

    trace!("AnimateRefresh():  end of function reached.");
}

#[no_mangle]
pub unsafe extern "C" fn IsPassable(x: c_float, y: c_float, check_pos: c_int) -> c_int {
    let map_brick = GetMapBrick(&*CurLevel, x, y);

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
            if fy < WALLPASS || fy > 1. - WALLPASS {
                Center as c_int
            } else {
                -1
            }
        }

        VWall => {
            if fx < WALLPASS || fx > 1. - WALLPASS {
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

        TO => {
            if fy < WALLPASS || (fy > 1. - WALLPASS && (fx < WALLPASS || fx > 1. - WALLPASS)) {
                Center as c_int
            } else {
                -1
            }
        }

        TR => {
            if fx > 1. - WALLPASS || (fx < WALLPASS && (fy < WALLPASS || fy > 1. - WALLPASS)) {
                Center as c_int
            } else {
                -1
            }
        }

        TU => {
            if fy > 1. - WALLPASS || (fy < WALLPASS && (fx < WALLPASS || fx > 1. - WALLPASS)) {
                Center as c_int
            } else {
                -1
            }
        }

        TL => {
            if fx < WALLPASS || (fx > 1. - WALLPASS && (fy < WALLPASS || fy > 1. - WALLPASS)) {
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
            if (fx < H_RANDBREITE || fx > 1. - H_RANDBREITE)
                && (fy >= H_RANDSPACE && fy <= 1. - H_RANDSPACE)
            {
                let check_pos = match check_pos.try_into() {
                    Ok(check_pos) => check_pos,
                    Err(_) => return -1,
                };
                if check_pos != Center && check_pos != Light && Me.speed.y != 0. {
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
            if (fy < V_RANDBREITE || fy > 1. - V_RANDBREITE)
                && (fx >= V_RANDSPACE && fx <= 1. - V_RANDSPACE)
            {
                let check_pos = match check_pos.try_into() {
                    Ok(check_pos) => check_pos,
                    Err(_) => return -1,
                };
                if check_pos != Center && check_pos != Light && Me.speed.x != 0. {
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

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ColorNames {
    Red,
    Yellow,
    Green,
    Gray,
    Blue,
    Greenblue,
    Dark,
}

/// Saves ship-data to disk
#[no_mangle]
pub unsafe extern "C" fn SaveShip(shipname: *const c_char) -> c_int {
    use std::{fs::File, io::Write, path::PathBuf};

    trace!("SaveShip(): real function call confirmed.");

    let filename = PathBuf::from(format!(
        "{}{}",
        CStr::from_ptr(shipname).to_str().unwrap(),
        SHIP_EXT
    ));

    /* count the levels */
    let level_anz = curShip
        .AllLevels
        .iter()
        .take_while(|level| level.is_null().not())
        .count();

    trace!("SaveShip(): now opening the ship file...");

    let mut ship_file = match File::create(filename) {
        Ok(file) => file,
        Err(err) => {
            error!("Error opening ship file: {}. Terminating", err);
            Terminate(defs::ERR.into());
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
        ship_file.write_all(CStr::from_ptr(curShip.AreaName.as_ptr()).to_bytes())?;
        ship_file.write_all(b"\"\n\n  ")?;

        /* Save all Levels */

        trace!("SaveShip(): now saving levels...");

        for i in 0..i32::try_from(level_anz).unwrap() {
            let mut level_iter = curShip
                .AllLevels
                .iter()
                .copied()
                .take_while(|level| level.is_null().not())
                .filter(|&level| (*level).levelnum == i);

            let level = match level_iter.next() {
                Some(level) => level,
                None => {
                    error!("Missing Levelnumber error in SaveShip.");
                    Terminate(defs::ERR.into());
                }
            };

            if level_iter.next().is_some() {
                error!("Identical Levelnumber Error in SaveShip.");
                Terminate(defs::ERR.into());
            }

            //--------------------
            // Now comes the real saving part FOR ONE LEVEL.  First THE LEVEL is packed into a string and
            // then this string is wirtten to the file.  easy. simple.
            let level_mem = StructToMem(level);
            ship_file.write_all(CStr::from_ptr(level_mem).to_bytes())?;

            libc::free(level_mem as *mut c_void);
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
            error!("Error writing to ship file: {}. Terminating", err);
            Terminate(defs::ERR.into());
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
#[no_mangle]
pub unsafe extern "C" fn MoveLevelDoors() {
    // This prevents animation going too quick.
    // The constant should be replaced by a variable, that can be
    // set from within the theme, but that may be done later...
    if LevelDoorsNotMovedTime < Time_For_Each_Phase_Of_Door_Movement {
        return;
    }
    LevelDoorsNotMovedTime = 0.;

    let cur_level = &*CurLevel;
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
        let xdist = Me.pos.x - f32::from(doorx);
        let ydist = Me.pos.y - f32::from(doory);
        let dist2 = xdist * xdist + ydist * ydist;

        const DOOROPENDIST2: f32 = 1.;
        if dist2 < DOOROPENDIST2 {
            if *pos != MapTile::HGanztuere as i8 && *pos != MapTile::VGanztuere as i8 {
                *pos += 1;
            }
        } else {
            /* alle Enemys checken */
            let mut j = 0;
            while j < usize::try_from(NumEnemys).unwrap() {
                /* ignore druids that are dead or on other levels */
                if AllEnemys[j].status == Status::Out as i32
                    || AllEnemys[j].status == Status::Terminated as i32
                    || AllEnemys[j].levelnum != cur_level.levelnum
                {
                    j += 1;
                    continue;
                }

                let xdist = (AllEnemys[j].pos.x - f32::from(doorx)).trunc().abs();
                if xdist < Block_Rect.w.into() {
                    let ydist = (AllEnemys[j].pos.y - f32::from(doory)).trunc().abs();
                    if ydist < Block_Rect.h.into() {
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
            if j == usize::try_from(NumEnemys).unwrap()
                && *pos != MapTile::VZutuere as i8
                && *pos != MapTile::HZutuere as i8
            {
                *pos -= 1;
            }
        }
    }
}

/// Returns a pointer to Map in a memory field
pub unsafe extern "C" fn StructToMem(level: *mut Level) -> *mut c_char {
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
    let level_mem = MyMalloc(i64::try_from(mem_amount).unwrap()) as *mut u8;
    if level_mem.is_null() {
        error!("could not allocate memory, terminating.");
        Terminate(defs::ERR.into());
    }
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
        CStr::from_ptr(level.Levelname).to_str().unwrap()
    )
    .unwrap();
    writeln!(
        level_cursor,
        "{}{}",
        LEVEL_ENTER_COMMENT_STRING,
        CStr::from_ptr(level.Level_Enter_Comment).to_str().unwrap()
    )
    .unwrap();
    writeln!(
        level_cursor,
        "{}{}",
        BACKGROUND_SONG_NAME_STRING,
        CStr::from_ptr(level.Background_Song_Name).to_str().unwrap()
    )
    .unwrap();

    // Now the beginning of the actual map data is marked:
    writeln!(level_cursor, "{}", MAP_BEGIN_STRING).unwrap();

    // Now in the loop each line of map data should be saved as a whole
    for i in 0..usize::try_from(ylen).unwrap() {
        ResetLevelMap(level); // make sure all doors are closed
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
            i, level.AllWaypoints[i].x, level.AllWaypoints[i].y, CONNECTION_STRING
        )
        .unwrap();

        let this_wp = &level.AllWaypoints[i];
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

#[no_mangle]
unsafe extern "C" fn ResetLevelMap(level: &mut Level) {
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
