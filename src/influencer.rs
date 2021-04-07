use crate::{
    bullet::StartBlast,
    defs::{
        self, Direction, Droid, Explosion, MapTile, Sound, Status, ENEMYPHASES, MAXBLASTS,
        MAXBULLETS, PUSHSPEED, WAIT_COLLISION,
    },
    global::{collision_lose_energy_calibrator, Droid_Radius},
    init::ThouArtDefeated,
    input::{axis_is_active, cmd_is_active, input_axis, NoDirectionPressed},
    map::{ActSpecialField, DruidPassable, GetMapBrick},
    misc::{Frame_Time, MyRandom, Terminate},
    ship::LevelEmpty,
    sound::{
        BounceSound, CollisionDamagedEnemySound, CollisionGotDamagedSound, Fire_Bullet_Sound,
        Play_Sound, RefreshSound,
    },
    structs::{Finepoint, Gps},
    takeover::Takeover,
    text::EnemyInfluCollisionText,
    vars::{Bulletmap, Druidmap},
    AllBlasts, AllBullets, AllEnemys, CurLevel, GameConfig, InvincibleMode, LastRefreshSound, Me,
    NumEnemys, RealScore,
};

use cstr::cstr;
use defs::{
    AnyCmdActive, Cmds, DownPressed, FirePressed, LeftPressed, RightPressed, UpPressed,
    BLINKENERGY, MAX_INFLU_POSITION_HISTORY, WAIT_TRANSFERMODE,
};
use log::{error, info, warn};
use std::{
    convert::{TryFrom, TryInto},
    ops::Not,
    os::raw::{c_char, c_float, c_int},
};

extern "C" {
    static mut CurrentZeroRingIndex: c_int;
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

/// This function moves the influencer, adjusts his speed according to
/// keys pressed and also adjusts his status and current "phase" of his rotation.
#[no_mangle]
pub unsafe extern "C" fn MoveInfluence() {
    static mut TRANSFER_COUNTER: c_float = 0.;

    let accel = (*Druidmap.add(usize::try_from(Me.ty).unwrap())).accel * Frame_Time();

    // We store the influencers position for the history record and so that others
    // can follow his trail.

    CurrentZeroRingIndex += 1;
    CurrentZeroRingIndex %= c_int::try_from(MAX_INFLU_POSITION_HISTORY).unwrap();
    Me.Position_History_Ring_Buffer[usize::try_from(CurrentZeroRingIndex).unwrap()] = Gps {
        x: Me.pos.x,
        y: Me.pos.y,
        z: (*CurLevel).levelnum,
    };

    PermanentLoseEnergy(); /* influ permanently loses energy */

    // check, if the influencer is still ok
    if Me.energy <= 0. {
        if Me.ty != Droid::Droid001 as c_int {
            Me.ty = Droid::Droid001 as c_int;
            Me.energy = BLINKENERGY;
            Me.health = BLINKENERGY;
            StartBlast(Me.pos.x, Me.pos.y, Explosion::Rejectblast as c_int);
        } else {
            Me.status = Status::Terminated as c_int;
            ThouArtDefeated();
            return;
        }
    }

    /* Time passed before entering Transfermode ?? */
    if TRANSFER_COUNTER >= WAIT_TRANSFERMODE {
        Me.status = Status::Transfermode as c_int;
        TRANSFER_COUNTER = 0.;
    }

    if UpPressed() {
        Me.speed.y -= accel;
    }
    if DownPressed() {
        Me.speed.y += accel;
    }
    if LeftPressed() {
        Me.speed.x -= accel;
    }
    if RightPressed() {
        Me.speed.x += accel;
    }

    //  We only need this check if we want held fire to cause activate
    if !AnyCmdActive() {
        // Used to be !SpacePressed, which causes any fire button != SPACE behave differently than space
        Me.status = Status::Mobile as c_int;
    }

    if (TRANSFER_COUNTER - 1.).abs() <= f32::EPSILON {
        Me.status = Status::Transfermode as c_int;
        TRANSFER_COUNTER = 0.;
    }

    if cmd_is_active(Cmds::Activate) {
        // activate mode for Konsole and Lifts
        Me.status = Status::Activate as c_int;
    }

    if GameConfig.FireHoldTakeover != 0
        && FirePressed()
        && NoDirectionPressed()
        && Me.status != Status::Weapon as c_int
        && Me.status != Status::Transfermode as c_int
    {
        // Proposed FireActivatePressed here...
        TRANSFER_COUNTER += Frame_Time(); // Or make it an option!
    }

    if FirePressed() && !NoDirectionPressed() && Me.status != Status::Transfermode as c_int {
        Me.status = Status::Weapon as c_int;
    }

    if FirePressed()
        && !NoDirectionPressed()
        && Me.status == Status::Weapon as c_int
        && Me.firewait == 0.
    {
        FireBullet();
    }

    if Me.status != Status::Weapon as c_int && cmd_is_active(Cmds::Takeover) {
        Me.status = Status::Transfermode as c_int;
    }

    InfluenceFrictionWithAir(); // The influ should lose some of his speed when no key is pressed

    AdjustSpeed(); // If the influ is faster than allowed for his type, slow him

    // Now we move influence according to current speed.  But there has been a problem
    // reported from people, that the influencer would (*very* rarely) jump throught walls
    // and even out of the ship.  This has *never* occured on my fast machine.  Therefore
    // I assume that the problem is related to sometimes very low framerates on these machines.
    // So, we do a sanity check not to make steps too big.
    //
    // So what do we do?  We allow a maximum step of exactly that, what the 302 (with a speed
    // of 7) could get when the framerate is as low as 20 FPS.  This should be sufficient to
    // prevent the influencer from *ever* leaving the ship.  I hope this really does work.
    // The definition of that speed is made in MAXIMAL_STEP_SIZE at the top of this file.
    //
    // And on machines with FPS << 20, it will certainly alter the game behaviour, so people
    // should really start using a pentium or better machine.
    //
    // NOTE:  PLEASE LEAVE THE .0 in the code or gcc will round it down to 0 like an integer.
    //
    let mut planned_step_x = Me.speed.x * Frame_Time();
    let mut planned_step_y = Me.speed.y * Frame_Time();
    if planned_step_x.abs() >= MAXIMAL_STEP_SIZE {
        planned_step_x = f32::copysign(MAXIMAL_STEP_SIZE, planned_step_x);
    }
    if planned_step_y.abs() >= MAXIMAL_STEP_SIZE {
        planned_step_y = f32::copysign(MAXIMAL_STEP_SIZE, planned_step_y);
    }
    Me.pos.x += planned_step_x;
    Me.pos.y += planned_step_y;

    //--------------------
    // Check it the influ is on a special field like a lift, a console or a refresh
    ActSpecialField(Me.pos.x, Me.pos.y);

    AnimateInfluence(); // move the "phase" of influencers rotation
}

#[no_mangle]
pub unsafe extern "C" fn PermanentLoseEnergy() {
    // Of course if in invincible mode, no energy will ever be lost...
    if InvincibleMode != 0 {
        return;
    }

    /* health decreases with time */
    Me.health -= (*Druidmap.add(usize::try_from(Me.ty).unwrap())).lose_health * Frame_Time();

    /* you cant have more energy than health */
    if Me.energy > Me.health {
        Me.energy = Me.health;
    }
}

/// Fire-Routine for the Influencer only !! (should be changed)
#[no_mangle]
pub unsafe extern "C" fn FireBullet() {
    let guntype = (*Druidmap.add(usize::try_from(Me.ty).unwrap())).gun; /* which gun do we have ? */
    let bullet_speed = (*Bulletmap.add(usize::try_from(guntype).unwrap())).speed;

    if Me.firewait > 0. {
        return;
    }
    Me.firewait = (*Bulletmap.add(usize::try_from(guntype).unwrap())).recharging_time;

    Fire_Bullet_Sound(guntype);

    let cur_bullet = AllBullets[..MAXBULLETS]
        .iter_mut()
        .find(|bullet| bullet.ty == Status::Out as u8)
        .unwrap_or(&mut AllBullets[0]);

    cur_bullet.pos.x = Me.pos.x;
    cur_bullet.pos.y = Me.pos.y;
    cur_bullet.ty = guntype.try_into().unwrap();
    cur_bullet.mine = true;
    cur_bullet.owner = -1;

    let mut speed = Finepoint { x: 0., y: 0. };

    if DownPressed() {
        speed.y = 1.0;
    }
    if UpPressed() {
        speed.y = -1.0;
    }
    if LeftPressed() {
        speed.x = -1.0;
    }
    if RightPressed() {
        speed.x = 1.0;
    }

    /* if using a joystick/mouse, allow exact directional shots! */
    if axis_is_active != 0 {
        let max_val = input_axis.x.abs().max(input_axis.y.abs()) as f32;
        speed.x = input_axis.x as f32 / max_val;
        speed.y = input_axis.y as f32 / max_val;
    }

    let speed_norm = (speed.x * speed.x + speed.y * speed.y).sqrt();
    cur_bullet.speed.x = speed.x / speed_norm;
    cur_bullet.speed.y = speed.y / speed_norm;

    // now determine the angle of the shot
    cur_bullet.angle = -speed.y.atan2(speed.x) * 180. / std::f32::consts::PI + 90.;

    info!("FireBullet: Phase of bullet={}.", cur_bullet.phase);
    info!("FireBullet: angle of bullet={}.", cur_bullet.angle);

    cur_bullet.speed.x *= bullet_speed;
    cur_bullet.speed.y *= bullet_speed;

    // To prevent influ from hitting himself with his own bullets,
    // move them a bit..
    cur_bullet.pos.x += 0.5 * (cur_bullet.speed.x / bullet_speed);
    cur_bullet.pos.y += 0.5 * (cur_bullet.speed.y / bullet_speed);

    cur_bullet.time_in_frames = 0;
    cur_bullet.time_in_seconds = 0.;
}

#[no_mangle]
pub unsafe extern "C" fn InfluenceFrictionWithAir() {
    const DECELERATION: f32 = 7.0;

    if !RightPressed() && !LeftPressed() {
        let oldsign = Me.speed.x.signum();
        let slowdown = oldsign * DECELERATION * Frame_Time();
        Me.speed.x -= slowdown;

        #[allow(clippy::float_cmp)]
        if Me.speed.x.signum() != oldsign {
            // changed direction -> vel=0
            Me.speed.x = 0.0;
        }
    }

    if !UpPressed() && !DownPressed() {
        let oldsign = Me.speed.y.signum();
        let slowdown = oldsign * DECELERATION * Frame_Time();
        Me.speed.y -= slowdown;

        #[allow(clippy::float_cmp)]
        if Me.speed.y.signum() != oldsign {
            // changed direction -> vel=0
            Me.speed.y = 0.0;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn AdjustSpeed() {
    let maxspeed = (*Druidmap.add(usize::try_from(Me.ty).unwrap())).maxspeed;
    Me.speed.x = Me.speed.x.clamp(-maxspeed, maxspeed);
    Me.speed.y = Me.speed.y.clamp(-maxspeed, maxspeed);
}

pub unsafe fn get_position_history(how_long_past: c_int) -> &'static Gps {
    let ring_position =
        CurrentZeroRingIndex - how_long_past + i32::try_from(MAX_INFLU_POSITION_HISTORY).unwrap();

    let ring_position = usize::try_from(ring_position).unwrap() % MAX_INFLU_POSITION_HISTORY;

    &Me.Position_History_Ring_Buffer[ring_position]
}

#[no_mangle]
pub unsafe extern "C" fn GetInfluPositionHistoryX(how_long_past: c_int) -> c_float {
    get_position_history(how_long_past).x
}

#[no_mangle]
pub unsafe extern "C" fn GetInfluPositionHistoryY(how_long_past: c_int) -> c_float {
    get_position_history(how_long_past).y
}

#[no_mangle]
pub unsafe extern "C" fn InitInfluPositionHistory() {
    Me.Position_History_Ring_Buffer.fill(Gps {
        x: Me.pos.x,
        y: Me.pos.y,
        z: (*CurLevel).levelnum,
    });
}
