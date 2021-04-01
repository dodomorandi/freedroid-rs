use crate::{
    defs::{Status, AGGRESSIONMAX, ENEMYPHASES, MAXBULLETS, ROBOT_MAX_WAIT_BETWEEN_SHOTS},
    map::IsVisible,
    misc::{Frame_Time, MyRandom},
    sound::Fire_Bullet_Sound,
    vars::{Bulletmap, Druidmap, Me},
    AllBullets, AllEnemys, CurLevel, NumEnemys,
};

use log::warn;
use std::{
    convert::{TryFrom, TryInto},
    os::raw::c_int,
};

extern "C" {
    pub fn ShuffleEnemys();
    pub fn ClearEnemys();
    pub fn PermanentHealRobots();
    pub fn MoveThisEnemy(enemy_num: c_int);
}

/// according to the intro, the laser can be "focused on any target
/// within a range of eight metres"
const FIREDIST2: f32 = 8.;

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
