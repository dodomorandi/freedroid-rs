use crate::{
    bullet::StartBlast,
    defs::{
        Explosion, Status, AGGRESSIONMAX, DECKCOMPLETEBONUS, ENEMYMAXWAIT, ENEMYPHASES, MAXBULLETS,
        MAXWAYPOINTS, ROBOT_MAX_WAIT_BETWEEN_SHOTS, SLOWMO_FACTOR, WAIT_COLLISION, WAIT_LEVELEMPTY,
    },
    global::Droid_Radius,
    map::IsVisible,
    misc::{set_time_factor, Frame_Time, MyRandom},
    ship::LevelEmpty,
    sound::Fire_Bullet_Sound,
    structs::Finepoint,
    vars::{Bulletmap, Druidmap, Me},
    AllBullets, AllEnemys, CurLevel, DeathCount, NumEnemys, RealScore,
};

use log::warn;
use std::{
    convert::{TryFrom, TryInto},
    ops::Not,
    os::raw::c_int,
};

extern "C" {
    pub fn ClearEnemys();
    pub fn PermanentHealRobots();
    pub fn MoveThisRobotThowardsHisWaypoint(enemy_num: c_int);
}

/// according to the intro, the laser can be "focused on any target
/// within a range of eight metres"
const FIREDIST2: f32 = 8.;

const COL_SPEED: f32 = 3.;

#[no_mangle]
pub unsafe extern "C" fn ClassOfDruid(druid_type: c_int) -> c_int {
    /* first digit is class */
    let class_char = (*Druidmap.add(usize::try_from(druid_type).unwrap())).druidname[0] as u8;
    match class_char {
        b'0'..=b'9' => (class_char - b'0').into(),
        _ => 0,
    }
}

#[no_mangle]
pub unsafe extern "C" fn AnimateEnemys() {
    for enemy in &mut AllEnemys[..usize::try_from(NumEnemys).unwrap()] {
        /* ignore enemys that are dead or on other levels or dummys */
        if enemy.levelnum != (*CurLevel).levelnum {
            continue;
        }
        if enemy.status == Status::Out as i32 {
            continue;
        }

        enemy.phase += (enemy.energy / (*Druidmap.add(enemy.ty.try_into().unwrap())).maxenergy)
            * Frame_Time()
            * ENEMYPHASES as f32
            * 2.5;

        if enemy.phase >= ENEMYPHASES as f32 {
            enemy.phase = 0.;
        }
    }
}

/// This is the function, that move each of the enemys according to
/// their orders and their program
#[no_mangle]
pub unsafe extern "C" fn MoveEnemys() {
    PermanentHealRobots(); // enemy robots heal as time passes...

    AnimateEnemys(); // move the "phase" of the rotation of enemys

    for (i, enemy) in AllEnemys[0..usize::try_from(NumEnemys).unwrap()]
        .iter_mut()
        .enumerate()
    {
        if enemy.status == Status::Out as i32
            || enemy.status == Status::Terminated as i32
            || enemy.levelnum != (*CurLevel).levelnum
        {
            continue;
        }

        MoveThisEnemy(i.try_into().unwrap());

        // If its a combat droid, then if might attack...
        if (*Druidmap.add(usize::try_from(enemy.ty).unwrap())).aggression != 0 {
            AttackInfluence(i.try_into().unwrap());
        }
    }
}

/// AttackInfluence(): This function sometimes fires a bullet from
/// enemy number enemynum directly into the direction of the influencer,
/// but of course only if the odds are good i.e. requirements are met.
#[no_mangle]
pub unsafe extern "C" fn AttackInfluence(enemy_num: c_int) {
    let this_robot = &mut AllEnemys[usize::try_from(enemy_num).unwrap()];
    // At first, we check for a lot of cases in which we do not
    // need to move anything for this reason or for that
    //

    // ignore robots on other levels
    if this_robot.levelnum != (*CurLevel).levelnum {
        return;
    }

    // ignore dead robots as well...
    if this_robot.status == Status::Out as c_int {
        return;
    }

    let mut xdist = Me.pos.x - this_robot.pos.x;
    let mut ydist = Me.pos.y - this_robot.pos.y;

    // Add some security against division by zero
    if xdist == 0. {
        xdist = 0.01;
    }
    if ydist == 0. {
        ydist = 0.01;
    }

    // if odds are good, make a shot at your target
    let guntype = (*Druidmap.add(this_robot.ty.try_into().unwrap())).gun;

    let dist2 = (xdist * xdist + ydist * ydist).sqrt();

    //--------------------
    //
    // From here on, it's classical Paradroid robot behaviour concerning fireing....
    //

    // distance limitation only for MS mechs
    if dist2 >= FIREDIST2 || this_robot.firewait != 0. || IsVisible(&this_robot.pos) == 0 {
        return;
    }

    if MyRandom(AGGRESSIONMAX) >= (*Druidmap.add(this_robot.ty.try_into().unwrap())).aggression {
        this_robot.firewait += MyRandom(1000) as f32 * ROBOT_MAX_WAIT_BETWEEN_SHOTS / 1000.0;
        return;
    }

    Fire_Bullet_Sound(guntype);

    // find a bullet entry, that isn't currently used...
    let mut j = 0;
    while j < MAXBULLETS {
        if AllBullets[j].ty == Status::Out as u8 {
            break;
        }

        j += 1;
    }

    if j == MAXBULLETS {
        warn!("AttackInfluencer: no free bullets... giving up");
        return;
    }

    let cur_bullet = &mut AllBullets[j];
    // determine the direction of the shot, so that it will go into the direction of
    // the target

    if xdist.abs() > ydist.abs() {
        cur_bullet.speed.x = (*Bulletmap.add(guntype.try_into().unwrap())).speed;
        cur_bullet.speed.y = ydist * cur_bullet.speed.x / xdist;
        if xdist < 0. {
            cur_bullet.speed.x = -cur_bullet.speed.x;
            cur_bullet.speed.y = -cur_bullet.speed.y;
        }
    }

    if xdist.abs() < ydist.abs() {
        cur_bullet.speed.y = (*Bulletmap.add(guntype.try_into().unwrap())).speed;
        cur_bullet.speed.x = xdist * cur_bullet.speed.y / ydist;
        if ydist < 0. {
            cur_bullet.speed.x = -cur_bullet.speed.x;
            cur_bullet.speed.y = -cur_bullet.speed.y;
        }
    }

    cur_bullet.angle =
        -(90. + 180. * f32::atan2(cur_bullet.speed.y, cur_bullet.speed.x) / std::f32::consts::PI);

    cur_bullet.pos.x = this_robot.pos.x;
    cur_bullet.pos.y = this_robot.pos.y;

    cur_bullet.pos.x +=
        (cur_bullet.speed.x) / ((*Bulletmap.add(guntype.try_into().unwrap())).speed).abs() * 0.5;
    cur_bullet.pos.y +=
        (cur_bullet.speed.y) / ((*Bulletmap.add(guntype.try_into().unwrap())).speed).abs() * 0.5;

    this_robot.firewait = (*Bulletmap.add(
        (*Druidmap.add(this_robot.ty.try_into().unwrap()))
            .gun
            .try_into()
            .unwrap(),
    ))
    .recharging_time;

    cur_bullet.ty = guntype.try_into().unwrap();
    cur_bullet.time_in_frames = 0;
    cur_bullet.time_in_seconds = 0.;
}

#[no_mangle]
pub unsafe extern "C" fn MoveThisEnemy(enemy_num: c_int) {
    let this_robot = &mut AllEnemys[usize::try_from(enemy_num).unwrap()];

    // Now check if the robot is still alive
    // if the robot just got killed, initiate the
    // explosion and all that...
    if this_robot.energy <= 0. && (this_robot.status != Status::Terminated as c_int) {
        this_robot.status = Status::Terminated as c_int;
        RealScore += (*Druidmap.add(usize::try_from(this_robot.ty).unwrap())).score as f32;

        DeathCount += (this_robot.ty * this_robot.ty) as f32; // quadratic "importance", max=529

        StartBlast(
            this_robot.pos.x,
            this_robot.pos.y,
            Explosion::Druidblast as c_int,
        );
        if LevelEmpty() != 0 {
            RealScore += DECKCOMPLETEBONUS;

            let cur_level = &mut *CurLevel;
            cur_level.empty = true.into();
            cur_level.timer = WAIT_LEVELEMPTY;
            set_time_factor(SLOWMO_FACTOR); // watch final explosion in slow-motion
        }
        return; // this one's down, so we can move on to the next
    }

    // robots that still have to wait also do not need to
    // be processed for movement
    if this_robot.warten > 0. {
        return;
    }

    // Now check for collisions of this enemy with his colleagues
    CheckEnemyEnemyCollision(enemy_num);

    // Now comes the real movement part
    MoveThisRobotThowardsHisWaypoint(enemy_num);

    SelectNextWaypointClassical(enemy_num);
}

#[no_mangle]
pub unsafe extern "C" fn CheckEnemyEnemyCollision(enemy_num: c_int) -> c_int {
    let curlev = (*CurLevel).levelnum;

    let enemy_num: usize = enemy_num.try_into().unwrap();
    let (enemys_before, rest) =
        AllEnemys[..usize::try_from(NumEnemys).unwrap()].split_at_mut(enemy_num);
    let (cur_enemy, enemys_after) = rest.split_first_mut().unwrap();
    let check_x = cur_enemy.pos.x;
    let check_y = cur_enemy.pos.y;

    for enemy in enemys_before.iter_mut().chain(enemys_after) {
        // check only collisions of LIVING enemys on this level
        if enemy.status == Status::Out as c_int
            || enemy.status == Status::Terminated as c_int
            || enemy.levelnum != curlev
        {
            continue;
        }

        /* get distance between enemy and cur_enemy */
        let xdist = check_x - enemy.pos.x;
        let ydist = check_y - enemy.pos.y;

        let dist = (xdist * xdist + ydist * ydist).sqrt();

        // Is there a Collision?
        if dist <= (2. * Droid_Radius) {
            // am I waiting already?  If so, keep waiting...
            if cur_enemy.warten != 0. {
                cur_enemy.warten = MyRandom(2 * WAIT_COLLISION) as f32;
                continue;
            }

            enemy.warten = MyRandom(2 * WAIT_COLLISION) as f32;

            if xdist != 0. {
                enemy.pos.x -= xdist / xdist.abs() * Frame_Time();
            }
            if ydist != 0. {
                enemy.pos.y -= ydist / ydist.abs() * Frame_Time();
            }

            std::mem::swap(&mut cur_enemy.nextwaypoint, &mut cur_enemy.lastwaypoint);

            let speed_x = cur_enemy.speed.x;
            let speed_y = cur_enemy.speed.y;

            if speed_x != 0. {
                cur_enemy.pos.x -= Frame_Time() * COL_SPEED * (speed_x) / speed_x.abs();
            }
            if speed_y != 0. {
                cur_enemy.pos.y -= Frame_Time() * COL_SPEED * (speed_y) / speed_y.abs();
            }

            return true.into();
        }
    }

    false.into()
}

/// This function checks if the connection between two points is free of
/// droids.
///
/// Map tiles are not taken into consideration, only droids.
#[no_mangle]
pub unsafe extern "C" fn SelectNextWaypointClassical(enemy_num: c_int) {
    let this_robot = &mut AllEnemys[usize::try_from(enemy_num).unwrap()];

    // We do some definitions to save us some more typing later...
    let wp_list = (*CurLevel).AllWaypoints;
    let nextwp = usize::try_from(this_robot.nextwaypoint).unwrap();

    // determine the remaining way until the target point is reached
    let restweg = Finepoint {
        x: f32::from(wp_list[nextwp].x) - this_robot.pos.x,
        y: f32::from(wp_list[nextwp].y) - this_robot.pos.y,
    };

    // Now we can see if we are perhaps already there?
    // then it might be time to set a new waypoint.
    //
    if restweg.x == 0. && restweg.y == 0. {
        this_robot.lastwaypoint = this_robot.nextwaypoint;
        this_robot.warten = MyRandom(ENEMYMAXWAIT) as f32;

        let num_con = wp_list[nextwp].num_connections;
        if num_con > 0 {
            this_robot.nextwaypoint =
                wp_list[nextwp].connections[usize::try_from(MyRandom(num_con - 1)).unwrap()];
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn ShuffleEnemys() {
    let cur_level = &*CurLevel;
    let cur_level_num = cur_level.levelnum;
    let mut used_wp: [bool; MAXWAYPOINTS] = [false; MAXWAYPOINTS];
    let mut warned = false;

    let num_wp = cur_level.num_waypoints;
    let mut nth_enemy = 0;

    for enemy in &mut AllEnemys[..usize::try_from(NumEnemys).unwrap()] {
        if enemy.status == Status::Out as c_int || enemy.levelnum != cur_level_num {
            /* dont handle dead enemys or on other level */
            continue;
        }

        nth_enemy += 1;
        if nth_enemy > num_wp {
            if !warned {
                warn!(
                    "Less waypoints ({}) than enemys on level {}? ...cannot insert all droids on \
                     this level.",
                    num_wp, cur_level_num
                );
            }

            warned = true;
            enemy.status = Status::Out as c_int;
            continue;
        }

        let mut wp;
        loop {
            wp = usize::try_from(MyRandom(num_wp - 1)).unwrap();
            if used_wp[wp].not() {
                break;
            }
        }

        used_wp[wp] = true;
        enemy.pos.x = cur_level.AllWaypoints[wp].x.into();
        enemy.pos.y = cur_level.AllWaypoints[wp].y.into();

        enemy.lastwaypoint = wp.try_into().unwrap();
        enemy.nextwaypoint = wp.try_into().unwrap();
    }
}
