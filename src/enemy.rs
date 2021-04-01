use crate::{
    defs::{Status, ENEMYPHASES},
    misc::Frame_Time,
    vars::Druidmap,
    AllEnemys, CurLevel, NumEnemys,
};

use std::{
    convert::{TryFrom, TryInto},
    os::raw::c_int,
};

extern "C" {
    pub fn ShuffleEnemys();
    pub fn ClearEnemys();
    pub fn PermanentHealRobots();
    pub fn MoveThisEnemy(enemy_num: c_int);
    pub fn AttackInfluence(enemy_num: c_int);
}

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
