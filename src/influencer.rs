use crate::{
    defs::{
        self, Direction, Droid, Explosion, MapTile, SoundType, Status, ENEMYPHASES, MAXBLASTS,
        MAXBULLETS, PUSHSPEED, WAIT_COLLISION,
    },
    global::{COLLISION_LOSE_ENERGY_CALIBRATOR, DROID_RADIUS},
    input::{AXIS_IS_ACTIVE, INPUT_AXIS},
    map::{druid_passable, get_map_brick},
    misc::my_random,
    ship::level_empty,
    structs::{Finepoint, Gps},
    text::enemy_influ_collision_text,
    vars::{BULLETMAP, DRUIDMAP},
    Data, ALL_BLASTS, ALL_BULLETS, ALL_ENEMYS, CUR_LEVEL, GAME_CONFIG, INVINCIBLE_MODE,
    LAST_REFRESH_SOUND, ME, NUM_ENEMYS, REAL_SCORE,
};

use cstr::cstr;
use defs::{Cmds, BLINKENERGY, MAX_INFLU_POSITION_HISTORY, WAIT_TRANSFERMODE};
use log::{info, warn};
use std::{
    convert::{TryFrom, TryInto},
    ops::Not,
    os::raw::{c_char, c_float, c_int},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Influencer {
    current_zero_ring_index: usize,
    time_counter: u32,
    transfer_counter: f32,
}

impl Default for Influencer {
    fn default() -> Self {
        Self {
            /* to slow down healing process */
            time_counter: 3,
            current_zero_ring_index: Default::default(),
            transfer_counter: Default::default(),
        }
    }
}

const REFRESH_ENERGY: f32 = 3.;

const COLLISION_PUSHSPEED: f32 = 2.0;
const MAXIMAL_STEP_SIZE: f32 = 7.0 / 20.;

impl Data {
    /// Refresh fields can be used to regain energy
    /// lost due to bullets or collisions, but not energy lost due to permanent
    /// loss of health in PermanentLoseEnergy.
    ///
    /// This function now takes into account the framerates.
    pub unsafe fn refresh_influencer(&mut self) {
        let time_counter = &mut self.influencer.time_counter;
        *time_counter -= 1;
        if *time_counter != 0 {
            return;
        }
        *time_counter = 3;

        if ME.energy < ME.health {
            ME.energy += REFRESH_ENERGY * self.frame_time() * 5.;
            REAL_SCORE -= REFRESH_ENERGY * self.frame_time() * 10.;

            if REAL_SCORE < 0. {
                // don't go negative...
                REAL_SCORE = 0.;
            }

            if ME.energy > ME.health {
                ME.energy = ME.health;
            }

            if LAST_REFRESH_SOUND > 0.6 {
                self.refresh_sound();
                LAST_REFRESH_SOUND = 0.;
            }

            // since robots like the refresh, the influencer might also say so...
            if GAME_CONFIG.droid_talk != 0 {
                ME.text_to_be_displayed =
                    cstr!("Ahhh, that feels so good...").as_ptr() as *mut c_char;
                ME.text_visible_time = 0.;
            }
        } else {
            // If nothing more is to be had, the influencer might also say so...
            if GAME_CONFIG.droid_talk != 0 {
                ME.text_to_be_displayed =
                    cstr!("Oh, it seems that was it again.").as_ptr() as *mut c_char;
                ME.text_visible_time = 0.;
            }
        }
    }

    pub unsafe fn check_influence_enemy_collision(&mut self) {
        for (i, enemy) in ALL_ENEMYS[..usize::try_from(NUM_ENEMYS).unwrap()]
            .iter_mut()
            .enumerate()
        {
            /* ignore enemy that are not on this level or dead */
            if enemy.levelnum != (*CUR_LEVEL).levelnum {
                continue;
            }
            if enemy.status == Status::Out as c_int || enemy.status == Status::Terminated as c_int {
                continue;
            }

            let xdist = ME.pos.x - enemy.pos.x;
            let ydist = ME.pos.y - enemy.pos.y;

            if xdist.trunc().abs() > 1. {
                continue;
            }
            if ydist.trunc().abs() > 1. {
                continue;
            }

            let dist2 = ((xdist * xdist) + (ydist * ydist)).sqrt();
            if dist2 > 2. * DROID_RADIUS {
                continue;
            }

            if ME.status != Status::Transfermode as c_int {
                ME.speed.x = -ME.speed.x;
                ME.speed.y = -ME.speed.y;

                if ME.speed.x != 0. {
                    ME.speed.x += COLLISION_PUSHSPEED * (ME.speed.x / ME.speed.x.abs());
                } else if xdist != 0. {
                    ME.speed.x = COLLISION_PUSHSPEED * (xdist / xdist.abs());
                }
                if ME.speed.y != 0. {
                    ME.speed.y += COLLISION_PUSHSPEED * (ME.speed.y / ME.speed.y.abs());
                } else if ydist != 0. {
                    ME.speed.y = COLLISION_PUSHSPEED * (ydist / ydist.abs());
                }

                // move the influencer a little bit out of the enemy AND the enemy a little bit out of the influ
                let max_step_size = if self.frame_time() < MAXIMAL_STEP_SIZE {
                    self.frame_time()
                } else {
                    MAXIMAL_STEP_SIZE
                };
                ME.pos.x += max_step_size.copysign(ME.pos.x - enemy.pos.x);
                ME.pos.y += max_step_size.copysign(ME.pos.y - enemy.pos.y);
                enemy.pos.x -= self.frame_time().copysign(ME.pos.x - enemy.pos.x);
                enemy.pos.y -= self.frame_time().copysign(ME.pos.y - enemy.pos.y);

                // there might be walls close too, so lets check again for collisions with them
                self.check_influence_wall_collisions();

                // shortly stop this enemy, then send him back to previous waypoint
                if enemy.warten == 0. {
                    enemy.warten = WAIT_COLLISION as f32;
                    std::mem::swap(&mut enemy.nextwaypoint, &mut enemy.lastwaypoint);

                    // Add some funny text!
                    enemy_influ_collision_text(i.try_into().unwrap());
                }
                self.influ_enemy_collision_lose_energy(i.try_into().unwrap()); /* someone loses energy ! */
            } else {
                self.takeover(i.try_into().unwrap());

                if level_empty() != 0 {
                    (*CUR_LEVEL).empty = true.into();
                }
            }
        }
    }

    pub unsafe fn influ_enemy_collision_lose_energy(&self, enemy_num: c_int) {
        let enemy_type = ALL_ENEMYS[usize::try_from(enemy_num).unwrap()].ty;

        let damage = ((*DRUIDMAP.add(usize::try_from(ME.ty).unwrap())).class
            - (*DRUIDMAP.add(usize::try_from(enemy_type).unwrap())).class)
            as f32
            * COLLISION_LOSE_ENERGY_CALIBRATOR;

        if damage < 0. {
            // we took damage
            self.collision_got_damaged_sound();
            if INVINCIBLE_MODE == 0 {
                ME.energy += damage;
            }
        } else if damage == 0. {
            // nobody got hurt
            self.bounce_sound();
        } else {
            // damage > 0: enemy got damaged
            ALL_ENEMYS[usize::try_from(enemy_num).unwrap()].energy -= damage;
            self.collision_damaged_enemy_sound();
        }
    }

    pub unsafe fn explode_influencer(&mut self) {
        ME.status = Status::Terminated as c_int;

        for i in 0..10 {
            /* freien Blast finden */
            let mut counter = 0;
            loop {
                let check = ALL_BLASTS[counter].ty != Status::Out as c_int;
                counter += 1;
                if check.not() {
                    break;
                }
            }
            counter -= 1;
            if counter >= MAXBLASTS {
                panic!("Went out of blasts in ExplodeInfluencer...");
            }
            let blast = &mut ALL_BLASTS[counter];
            blast.ty = Explosion::Druidblast as c_int;
            blast.px = ME.pos.x - DROID_RADIUS / 2. + my_random(10) as f32 * 0.05;
            blast.py = ME.pos.y - DROID_RADIUS / 2. + my_random(10) as f32 * 0.05;
            blast.phase = 0.2 * i as f32;
        }

        self.play_sound(SoundType::Influexplosion as c_int);
    }

    /// This function checks for collisions of the influencer with walls,
    /// doors, consoles, boxes and all other map elements.
    /// In case of a collision, the position and speed of the influencer are
    /// adapted accordingly.
    /// NOTE: Of course this functions HAS to take into account the current framerate!
    pub unsafe fn check_influence_wall_collisions(&mut self) {
        let sx = ME.speed.x * self.frame_time();
        let sy = ME.speed.y * self.frame_time();
        let mut h_door_sliding_active = false;

        let lastpos = Finepoint {
            x: ME.pos.x - sx,
            y: ME.pos.y - sy,
        };

        let res = druid_passable(ME.pos.x, ME.pos.y);

        // Influence-Wall-Collision only has to be checked in case of
        // a collision of course, which is indicated by res not CENTER.
        if res != Direction::Center as c_int {
            //--------------------
            // At first we just check in which directions (from the last position)
            // the ways are blocked and in which directions the ways are open.
            //
            let north_south_axis_blocked;
            if !((druid_passable(
                lastpos.x,
                lastpos.y
                    + (*DRUIDMAP.add(usize::try_from(ME.ty).unwrap())).maxspeed * self.frame_time(),
            ) != Direction::Center as c_int)
                || (druid_passable(
                    lastpos.x,
                    lastpos.y
                        - (*DRUIDMAP.add(usize::try_from(ME.ty).unwrap())).maxspeed
                            * self.frame_time(),
                ) != Direction::Center as c_int))
            {
                info!("North-south-Axis seems to be free.");
                north_south_axis_blocked = false;
            } else {
                north_south_axis_blocked = true;
            }

            let east_west_axis_blocked;
            if (druid_passable(
                lastpos.x
                    + (*DRUIDMAP.add(usize::try_from(ME.ty).unwrap())).maxspeed * self.frame_time(),
                lastpos.y,
            ) == Direction::Center as c_int)
                && (druid_passable(
                    lastpos.x
                        - (*DRUIDMAP.add(usize::try_from(ME.ty).unwrap())).maxspeed
                            * self.frame_time(),
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
                ME.pos.y = lastpos.y;
                ME.speed.y = 0.;

                // if its an open door, we also correct the east-west position, in the
                // sense that we move thowards the middle
                if get_map_brick(&*CUR_LEVEL, ME.pos.x, ME.pos.y - 0.5) == MapTile::HGanztuere as u8
                    || get_map_brick(&*CUR_LEVEL, ME.pos.x, ME.pos.y + 0.5)
                        == MapTile::HGanztuere as u8
                {
                    ME.pos.x +=
                        f32::copysign(PUSHSPEED * self.frame_time(), ME.pos.x.round() - ME.pos.x);
                    h_door_sliding_active = true;
                }
            }

            if east_west_axis_blocked {
                // EastWestCorrectionDone=TRUE;
                if !h_door_sliding_active {
                    ME.pos.x = lastpos.x;
                }
                ME.speed.x = 0.;

                // if its an open door, we also correct the north-south position, in the
                // sense that we move thowards the middle
                if (get_map_brick(&*CUR_LEVEL, ME.pos.x + 0.5, ME.pos.y)
                    == MapTile::VGanztuere as u8)
                    || (get_map_brick(&*CUR_LEVEL, ME.pos.x - 0.5, ME.pos.y)
                        == MapTile::VGanztuere as u8)
                {
                    ME.pos.y +=
                        f32::copysign(PUSHSPEED * self.frame_time(), ME.pos.y.round() - ME.pos.y);
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
                if druid_passable(ME.pos.x + sx, ME.pos.y) == Direction::Center as c_int {
                    ME.pos.x += sx;
                }
                if druid_passable(ME.pos.x, ME.pos.y + sy) == Direction::Center as c_int {
                    ME.pos.y += sy;
                }
            }

            // Here I introduce some extra security as a fallback:  Obviously
            // if the influencer is blocked FOR THE SECOND TIME, then the throw-back-algorithm
            // above HAS FAILED.  The absolutely fool-proof and secure handling is now done by
            // simply reverting to the last influ coordinated, where influ was NOT BLOCKED.
            // For this reason, a history of influ-coordinates has been introduced.  This will all
            // be done here and now:

            if (druid_passable(ME.pos.x, ME.pos.y) != Direction::Center as c_int)
                && (druid_passable(
                    self.get_influ_position_history_x(0),
                    self.get_influ_position_history_y(0),
                ) != Direction::Center as c_int)
                && (druid_passable(
                    self.get_influ_position_history_x(1),
                    self.get_influ_position_history_y(1),
                ) != Direction::Center as c_int)
            {
                ME.pos.x = self.get_influ_position_history_x(2);
                ME.pos.y = self.get_influ_position_history_y(2);
                warn!("ATTENTION! CheckInfluenceWallCollsision FALLBACK ACTIVATED!!",);
            }
        }
    }
}

impl Data {
    pub unsafe fn animate_influence(&mut self) {
        if ME.ty != Droid::Droid001 as c_int {
            ME.phase += (ME.energy
                / ((*DRUIDMAP.add(usize::try_from(ME.ty).unwrap())).maxenergy
                    + (*DRUIDMAP.add(Droid::Droid001 as usize)).maxenergy))
                * self.frame_time()
                * f32::from(ENEMYPHASES)
                * 3.;
        } else {
            ME.phase += (ME.energy / ((*DRUIDMAP.add(Droid::Droid001 as usize)).maxenergy))
                * self.frame_time()
                * f32::from(ENEMYPHASES)
                * 3.;
        }

        if ME.phase.round() >= ENEMYPHASES.into() {
            ME.phase = 0.;
        }
    }

    /// This function moves the influencer, adjusts his speed according to
    /// keys pressed and also adjusts his status and current "phase" of his rotation.
    pub(crate) unsafe fn move_influence(&mut self) {
        let accel = (*DRUIDMAP.add(usize::try_from(ME.ty).unwrap())).accel * self.frame_time();

        // We store the influencers position for the history record and so that others
        // can follow his trail.

        self.influencer.current_zero_ring_index += 1;
        self.influencer.current_zero_ring_index %= MAX_INFLU_POSITION_HISTORY;
        ME.position_history_ring_buffer[self.influencer.current_zero_ring_index] = Gps {
            x: ME.pos.x,
            y: ME.pos.y,
            z: (*CUR_LEVEL).levelnum,
        };

        self.permanent_lose_energy(); /* influ permanently loses energy */

        // check, if the influencer is still ok
        if ME.energy <= 0. {
            if ME.ty != Droid::Droid001 as c_int {
                ME.ty = Droid::Droid001 as c_int;
                ME.energy = BLINKENERGY;
                ME.health = BLINKENERGY;
                self.start_blast(ME.pos.x, ME.pos.y, Explosion::Rejectblast as c_int);
            } else {
                ME.status = Status::Terminated as c_int;
                self.thou_art_defeated();
                return;
            }
        }

        /* Time passed before entering Transfermode ?? */
        if self.influencer.transfer_counter >= WAIT_TRANSFERMODE {
            ME.status = Status::Transfermode as c_int;
            self.influencer.transfer_counter = 0.;
        }

        if self.up_pressed() {
            ME.speed.y -= accel;
        }
        if self.down_pressed() {
            ME.speed.y += accel;
        }
        if self.left_pressed() {
            ME.speed.x -= accel;
        }
        if self.right_pressed() {
            ME.speed.x += accel;
        }

        //  We only need this check if we want held fire to cause activate
        if !self.any_cmd_active() {
            // Used to be !SpacePressed, which causes any fire button != SPACE behave differently than space
            ME.status = Status::Mobile as c_int;
        }

        if (self.influencer.transfer_counter - 1.).abs() <= f32::EPSILON {
            ME.status = Status::Transfermode as c_int;
            self.influencer.transfer_counter = 0.;
        }

        if self.cmd_is_active(Cmds::Activate) {
            // activate mode for Konsole and Lifts
            ME.status = Status::Activate as c_int;
        }

        if GAME_CONFIG.fire_hold_takeover != 0
            && self.fire_pressed()
            && self.no_direction_pressed()
            && ME.status != Status::Weapon as c_int
            && ME.status != Status::Transfermode as c_int
        {
            // Proposed FireActivatePressed here...
            self.influencer.transfer_counter += self.frame_time(); // Or make it an option!
        }

        if self.fire_pressed()
            && !self.no_direction_pressed()
            && ME.status != Status::Transfermode as c_int
        {
            ME.status = Status::Weapon as c_int;
        }

        if self.fire_pressed()
            && !self.no_direction_pressed()
            && ME.status == Status::Weapon as c_int
            && ME.firewait == 0.
        {
            self.fire_bullet();
        }

        if ME.status != Status::Weapon as c_int && self.cmd_is_active(Cmds::Takeover) {
            ME.status = Status::Transfermode as c_int;
        }

        self.influence_friction_with_air(); // The influ should lose some of his speed when no key is pressed

        adjust_speed(); // If the influ is faster than allowed for his type, slow him

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
        let mut planned_step_x = ME.speed.x * self.frame_time();
        let mut planned_step_y = ME.speed.y * self.frame_time();
        if planned_step_x.abs() >= MAXIMAL_STEP_SIZE {
            planned_step_x = f32::copysign(MAXIMAL_STEP_SIZE, planned_step_x);
        }
        if planned_step_y.abs() >= MAXIMAL_STEP_SIZE {
            planned_step_y = f32::copysign(MAXIMAL_STEP_SIZE, planned_step_y);
        }
        ME.pos.x += planned_step_x;
        ME.pos.y += planned_step_y;

        //--------------------
        // Check it the influ is on a special field like a lift, a console or a refresh
        self.act_special_field(ME.pos.x, ME.pos.y);

        self.animate_influence(); // move the "phase" of influencers rotation
    }
}

impl Data {
    pub unsafe fn permanent_lose_energy(&mut self) {
        // Of course if in invincible mode, no energy will ever be lost...
        if INVINCIBLE_MODE != 0 {
            return;
        }

        /* health decreases with time */
        ME.health -=
            (*DRUIDMAP.add(usize::try_from(ME.ty).unwrap())).lose_health * self.frame_time();

        /* you cant have more energy than health */
        if ME.energy > ME.health {
            ME.energy = ME.health;
        }
    }

    /// Fire-Routine for the Influencer only !! (should be changed)
    pub unsafe fn fire_bullet(&mut self) {
        let guntype = (*DRUIDMAP.add(usize::try_from(ME.ty).unwrap())).gun; /* which gun do we have ? */
        let bullet_speed = (*BULLETMAP.add(usize::try_from(guntype).unwrap())).speed;

        if ME.firewait > 0. {
            return;
        }
        ME.firewait = (*BULLETMAP.add(usize::try_from(guntype).unwrap())).recharging_time;

        self.fire_bullet_sound(guntype);

        let cur_bullet = ALL_BULLETS[..MAXBULLETS]
            .iter_mut()
            .find(|bullet| bullet.ty == Status::Out as u8)
            .unwrap_or(&mut ALL_BULLETS[0]);

        cur_bullet.pos.x = ME.pos.x;
        cur_bullet.pos.y = ME.pos.y;
        cur_bullet.ty = guntype.try_into().unwrap();
        cur_bullet.mine = true;
        cur_bullet.owner = -1;

        let mut speed = Finepoint { x: 0., y: 0. };

        if self.down_pressed() {
            speed.y = 1.0;
        }
        if self.up_pressed() {
            speed.y = -1.0;
        }
        if self.left_pressed() {
            speed.x = -1.0;
        }
        if self.right_pressed() {
            speed.x = 1.0;
        }

        /* if using a joystick/mouse, allow exact directional shots! */
        if AXIS_IS_ACTIVE != 0 {
            let max_val = INPUT_AXIS.x.abs().max(INPUT_AXIS.y.abs()) as f32;
            speed.x = INPUT_AXIS.x as f32 / max_val;
            speed.y = INPUT_AXIS.y as f32 / max_val;
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

    pub unsafe fn influence_friction_with_air(&mut self) {
        const DECELERATION: f32 = 7.0;

        if !self.right_pressed() && !self.left_pressed() {
            let oldsign = ME.speed.x.signum();
            let slowdown = oldsign * DECELERATION * self.frame_time();
            ME.speed.x -= slowdown;

            #[allow(clippy::float_cmp)]
            if ME.speed.x.signum() != oldsign {
                // changed direction -> vel=0
                ME.speed.x = 0.0;
            }
        }

        if !self.up_pressed() && !self.down_pressed() {
            let oldsign = ME.speed.y.signum();
            let slowdown = oldsign * DECELERATION * self.frame_time();
            ME.speed.y -= slowdown;

            #[allow(clippy::float_cmp)]
            if ME.speed.y.signum() != oldsign {
                // changed direction -> vel=0
                ME.speed.y = 0.0;
            }
        }
    }
}

pub unsafe fn adjust_speed() {
    let maxspeed = (*DRUIDMAP.add(usize::try_from(ME.ty).unwrap())).maxspeed;
    ME.speed.x = ME.speed.x.clamp(-maxspeed, maxspeed);
    ME.speed.y = ME.speed.y.clamp(-maxspeed, maxspeed);
}

impl Data {
    pub unsafe fn get_position_history(&self, how_long_past: c_int) -> &'static Gps {
        let ring_position = self.influencer.current_zero_ring_index + MAX_INFLU_POSITION_HISTORY
            - usize::try_from(how_long_past).unwrap();

        let ring_position = ring_position % MAX_INFLU_POSITION_HISTORY;

        &ME.position_history_ring_buffer[ring_position]
    }

    pub unsafe fn get_influ_position_history_x(&self, how_long_past: c_int) -> c_float {
        self.get_position_history(how_long_past).x
    }

    pub unsafe fn get_influ_position_history_y(&self, how_long_past: c_int) -> c_float {
        self.get_position_history(how_long_past).y
    }
}

pub unsafe fn init_influ_position_history() {
    ME.position_history_ring_buffer.fill(Gps {
        x: ME.pos.x,
        y: ME.pos.y,
        z: (*CUR_LEVEL).levelnum,
    });
}
