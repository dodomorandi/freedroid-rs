use crate::{
    defs::{
        self, Direction, Droid, Explosion, MapTile, Sound, Status, ENEMYPHASES, MAXBLASTS,
        PUSHSPEED, WAIT_COLLISION,
    },
    global::{collision_lose_energy_calibrator, Droid_Radius},
    map::{DruidPassable, GetMapBrick},
    misc::{Frame_Time, MyRandom, Terminate},
    ship::LevelEmpty,
    sound::{
        BounceSound, CollisionDamagedEnemySound, CollisionGotDamagedSound, Play_Sound, RefreshSound,
    },
    structs::Finepoint,
    takeover::Takeover,
    text::EnemyInfluCollisionText,
    vars::Druidmap,
    AllBlasts, AllEnemys, CurLevel, GameConfig, InvincibleMode, LastRefreshSound, Me, NumEnemys,
    RealScore,
};

use cstr::cstr;
use log::{error, info, warn};
use std::{
    convert::{TryFrom, TryInto},
    ops::Not,
    os::raw::{c_char, c_float, c_int},
};

extern "C" {
    pub fn MoveInfluence();
    pub fn InitInfluPositionHistory();
    pub fn GetInfluPositionHistoryX(how_long_past: c_int) -> c_float;
    pub fn GetInfluPositionHistoryY(how_long_past: c_int) -> c_float;
}

const REFRESH_ENERGY: f32 = 3.;

const COLLISION_PUSHSPEED: f32 = 2.0;
const MAXIMAL_STEP_SIZE: f32 = 7.0 / 20.;

/// Refresh fields can be used to regain energy
/// lost due to bullets or collisions, but not energy lost due to permanent
/// loss of health in PermanentLoseEnergy.
///
/// This function now takes into account the framerates.
#[no_mangle]
pub unsafe extern "C" fn RefreshInfluencer() {
    static mut TIME_COUNTER: c_int = 3; /* to slow down healing process */

    TIME_COUNTER -= 1;
    if TIME_COUNTER != 0 {
        return;
    }
    TIME_COUNTER = 3;

    if Me.energy < Me.health {
        Me.energy += REFRESH_ENERGY * Frame_Time() * 5.;
        RealScore -= REFRESH_ENERGY * Frame_Time() * 10.;

        if RealScore < 0. {
            // don't go negative...
            RealScore = 0.;
        }

        if Me.energy > Me.health {
            Me.energy = Me.health;
        }

        if LastRefreshSound > 0.6 {
            RefreshSound();
            LastRefreshSound = 0.;
        }

        // since robots like the refresh, the influencer might also say so...
        if GameConfig.Droid_Talk != 0 {
            Me.TextToBeDisplayed = cstr!("Ahhh, that feels so good...").as_ptr() as *mut c_char;
            Me.TextVisibleTime = 0.;
        }
    } else {
        // If nothing more is to be had, the influencer might also say so...
        if GameConfig.Droid_Talk != 0 {
            Me.TextToBeDisplayed = cstr!("Oh, it seems that was it again.").as_ptr() as *mut c_char;
            Me.TextVisibleTime = 0.;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn CheckInfluenceEnemyCollision() {
    for (i, enemy) in AllEnemys[..usize::try_from(NumEnemys).unwrap()]
        .iter_mut()
        .enumerate()
    {
        /* ignore enemy that are not on this level or dead */
        if enemy.levelnum != (*CurLevel).levelnum {
            continue;
        }
        if enemy.status == Status::Out as c_int || enemy.status == Status::Terminated as c_int {
            continue;
        }

        let xdist = Me.pos.x - enemy.pos.x;
        let ydist = Me.pos.y - enemy.pos.y;

        if xdist.trunc().abs() > 1. {
            continue;
        }
        if ydist.trunc().abs() > 1. {
            continue;
        }

        let dist2 = ((xdist * xdist) + (ydist * ydist)).sqrt();
        if dist2 > 2. * Droid_Radius {
            continue;
        }

        if Me.status != Status::Transfermode as c_int {
            Me.speed.x = -Me.speed.x;
            Me.speed.y = -Me.speed.y;

            if Me.speed.x != 0. {
                Me.speed.x += COLLISION_PUSHSPEED * (Me.speed.x / Me.speed.x.abs());
            } else if xdist != 0. {
                Me.speed.x = COLLISION_PUSHSPEED * (xdist / xdist.abs());
            }
            if Me.speed.y != 0. {
                Me.speed.y += COLLISION_PUSHSPEED * (Me.speed.y / Me.speed.y.abs());
            } else if ydist != 0. {
                Me.speed.y = COLLISION_PUSHSPEED * (ydist / ydist.abs());
            }

            // move the influencer a little bit out of the enemy AND the enemy a little bit out of the influ
            let max_step_size = if Frame_Time() < MAXIMAL_STEP_SIZE {
                Frame_Time()
            } else {
                MAXIMAL_STEP_SIZE
            };
            Me.pos.x += max_step_size.copysign(Me.pos.x - enemy.pos.x);
            Me.pos.y += max_step_size.copysign(Me.pos.y - enemy.pos.y);
            enemy.pos.x -= Frame_Time().copysign(Me.pos.x - enemy.pos.x);
            enemy.pos.y -= Frame_Time().copysign(Me.pos.y - enemy.pos.y);

            // there might be walls close too, so lets check again for collisions with them
            CheckInfluenceWallCollisions();

            // shortly stop this enemy, then send him back to previous waypoint
            if enemy.warten == 0. {
                enemy.warten = WAIT_COLLISION as f32;
                std::mem::swap(&mut enemy.nextwaypoint, &mut enemy.lastwaypoint);

                // Add some funny text!
                EnemyInfluCollisionText(i.try_into().unwrap());
            }
            InfluEnemyCollisionLoseEnergy(i.try_into().unwrap()); /* someone loses energy ! */
        } else {
            Takeover(i.try_into().unwrap());

            if LevelEmpty() != 0 {
                (*CurLevel).empty = true.into();
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn InfluEnemyCollisionLoseEnergy(enemy_num: c_int) {
    let enemy_type = AllEnemys[usize::try_from(enemy_num).unwrap()].ty;

    let damage = ((*Druidmap.add(usize::try_from(Me.ty).unwrap())).class
        - (*Druidmap.add(usize::try_from(enemy_type).unwrap())).class) as f32
        * collision_lose_energy_calibrator;

    if damage < 0. {
        // we took damage
        CollisionGotDamagedSound();
        if InvincibleMode == 0 {
            Me.energy += damage;
        }
    } else if damage == 0. {
        // nobody got hurt
        BounceSound();
    } else {
        // damage > 0: enemy got damaged
        AllEnemys[usize::try_from(enemy_num).unwrap()].energy -= damage;
        CollisionDamagedEnemySound();
    }
}

#[no_mangle]
pub unsafe extern "C" fn ExplodeInfluencer() {
    Me.status = Status::Terminated as c_int;

    for i in 0..10 {
        /* freien Blast finden */
        let mut counter = 0;
        loop {
            let check = AllBlasts[counter].ty != Status::Out as c_int;
            counter += 1;
            if check.not() {
                break;
            }
        }
        counter -= 1;
        if counter >= MAXBLASTS {
            error!("Went out of blasts in ExplodeInfluencer...");
            Terminate(defs::ERR.into());
        }
        let blast = &mut AllBlasts[counter];
        blast.ty = Explosion::Druidblast as c_int;
        blast.PX = Me.pos.x - Droid_Radius / 2. + MyRandom(10) as f32 * 0.05;
        blast.PY = Me.pos.y - Droid_Radius / 2. + MyRandom(10) as f32 * 0.05;
        blast.phase = 0.2 * i as f32;
    }

    Play_Sound(Sound::Influexplosion as c_int);
}

/// This function checks for collisions of the influencer with walls,
/// doors, consoles, boxes and all other map elements.
/// In case of a collision, the position and speed of the influencer are
/// adapted accordingly.
/// NOTE: Of course this functions HAS to take into account the current framerate!
#[no_mangle]
pub unsafe extern "C" fn CheckInfluenceWallCollisions() {
    let sx = Me.speed.x * Frame_Time();
    let sy = Me.speed.y * Frame_Time();
    let mut h_door_sliding_active = false;

    let lastpos = Finepoint {
        x: Me.pos.x - sx,
        y: Me.pos.y - sy,
    };

    let res = DruidPassable(Me.pos.x, Me.pos.y);

    // Influence-Wall-Collision only has to be checked in case of
    // a collision of course, which is indicated by res not CENTER.
    if res != Direction::Center as c_int {
        //--------------------
        // At first we just check in which directions (from the last position)
        // the ways are blocked and in which directions the ways are open.
        //
        let north_south_axis_blocked;
        if !((DruidPassable(
            lastpos.x,
            lastpos.y + (*Druidmap.add(usize::try_from(Me.ty).unwrap())).maxspeed * Frame_Time(),
        ) != Direction::Center as c_int)
            || (DruidPassable(
                lastpos.x,
                lastpos.y
                    - (*Druidmap.add(usize::try_from(Me.ty).unwrap())).maxspeed * Frame_Time(),
            ) != Direction::Center as c_int))
        {
            info!("North-south-Axis seems to be free.");
            north_south_axis_blocked = false;
        } else {
            north_south_axis_blocked = true;
        }

        let east_west_axis_blocked;
        if (DruidPassable(
            lastpos.x + (*Druidmap.add(usize::try_from(Me.ty).unwrap())).maxspeed * Frame_Time(),
            lastpos.y,
        ) == Direction::Center as c_int)
            && (DruidPassable(
                lastpos.x
                    - (*Druidmap.add(usize::try_from(Me.ty).unwrap())).maxspeed * Frame_Time(),
                lastpos.y,
            ) == Direction::Center as c_int)
        {
            east_west_axis_blocked = false;
        } else {
            east_west_axis_blocked = true;
        }

        // Now we try to handle the sitution:

        if north_south_axis_blocked {
            // NorthSouthCorrectionDone=TRUE;
            Me.pos.y = lastpos.y;
            Me.speed.y = 0.;

            // if its an open door, we also correct the east-west position, in the
            // sense that we move thowards the middle
            if GetMapBrick(&*CurLevel, Me.pos.x, Me.pos.y - 0.5) == MapTile::HGanztuere as u8
                || GetMapBrick(&*CurLevel, Me.pos.x, Me.pos.y + 0.5) == MapTile::HGanztuere as u8
            {
                Me.pos.x += f32::copysign(PUSHSPEED * Frame_Time(), Me.pos.x.round() - Me.pos.x);
                h_door_sliding_active = true;
            }
        }

        if east_west_axis_blocked {
            // EastWestCorrectionDone=TRUE;
            if !h_door_sliding_active {
                Me.pos.x = lastpos.x;
            }
            Me.speed.x = 0.;

            // if its an open door, we also correct the north-south position, in the
            // sense that we move thowards the middle
            if (GetMapBrick(&*CurLevel, Me.pos.x + 0.5, Me.pos.y) == MapTile::VGanztuere as u8)
                || (GetMapBrick(&*CurLevel, Me.pos.x - 0.5, Me.pos.y) == MapTile::VGanztuere as u8)
            {
                Me.pos.y += f32::copysign(PUSHSPEED * Frame_Time(), Me.pos.y.round() - Me.pos.y);
            }
        }

        if east_west_axis_blocked && north_south_axis_blocked {
            // printf("\nBOTH AXES BLOCKED... Corner handling activated...");
            // in case both axes were blocked, we must be at a corner.
            // both axis-blocked-routines have been executed, so the speed has
            // been set to absolutely zero and we are at the previous position.
            //
            // But perhaps everything would be fine,
            // if we just restricted ourselves to moving in only ONE direction.
            // try if this would make sense...
            // (Of course we may only move into the one direction that is free)
            //
            if DruidPassable(Me.pos.x + sx, Me.pos.y) == Direction::Center as c_int {
                Me.pos.x += sx;
            }
            if DruidPassable(Me.pos.x, Me.pos.y + sy) == Direction::Center as c_int {
                Me.pos.y += sy;
            }
        }

        // Here I introduce some extra security as a fallback:  Obviously
        // if the influencer is blocked FOR THE SECOND TIME, then the throw-back-algorithm
        // above HAS FAILED.  The absolutely fool-proof and secure handling is now done by
        // simply reverting to the last influ coordinated, where influ was NOT BLOCKED.
        // For this reason, a history of influ-coordinates has been introduced.  This will all
        // be done here and now:

        if (DruidPassable(Me.pos.x, Me.pos.y) != Direction::Center as c_int)
            && (DruidPassable(GetInfluPositionHistoryX(0), GetInfluPositionHistoryY(0))
                != Direction::Center as c_int)
            && (DruidPassable(GetInfluPositionHistoryX(1), GetInfluPositionHistoryY(1))
                != Direction::Center as c_int)
        {
            Me.pos.x = GetInfluPositionHistoryX(2);
            Me.pos.y = GetInfluPositionHistoryY(2);
            warn!("ATTENTION! CheckInfluenceWallCollsision FALLBACK ACTIVATED!!",);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn AnimateInfluence() {
    if Me.ty != Droid::Droid001 as c_int {
        Me.phase += (Me.energy
            / ((*Druidmap.add(usize::try_from(Me.ty).unwrap())).maxenergy
                + (*Druidmap.add(Droid::Droid001 as usize)).maxenergy))
            * Frame_Time()
            * f32::from(ENEMYPHASES)
            * 3.;
    } else {
        Me.phase += (Me.energy / ((*Druidmap.add(Droid::Droid001 as usize)).maxenergy))
            * Frame_Time()
            * f32::from(ENEMYPHASES)
            * 3.;
    }

    if Me.phase.round() >= ENEMYPHASES.into() {
        Me.phase = 0.;
    }
}
