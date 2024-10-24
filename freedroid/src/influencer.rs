use crate::{
    cur_level,
    defs::{
        self, Direction, Droid, Explosion, MapTile, SoundType, Status, ENEMYPHASES, MAXBLASTS,
        PUSHSPEED, WAIT_COLLISION,
    },
    map::get_map_brick,
    structs::{Bullet, Finepoint, Gps, TextToBeDisplayed},
};

use defs::{Cmds, BLINKENERGY, MAX_INFLU_POSITION_HISTORY, WAIT_TRANSFERMODE};
use log::{info, warn};
use rand::{thread_rng, Rng};
use std::ops::Not;

#[derive(Debug, Clone, PartialEq)]
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

impl crate::Data<'_> {
    /// Refresh fields can be used to regain energy
    /// lost due to bullets or collisions, but not energy lost due to permanent
    /// loss of health in `PermanentLoseEnergy`.
    ///
    /// This function now takes into account the framerates.
    pub fn refresh_influencer(&mut self) {
        let time_counter = &mut self.influencer.time_counter;
        *time_counter -= 1;
        if *time_counter != 0 {
            return;
        }
        *time_counter = 3;

        if self.vars.me.energy < self.vars.me.health {
            self.vars.me.energy += REFRESH_ENERGY * self.frame_time() * 5.;
            self.main.real_score -= REFRESH_ENERGY * self.frame_time() * 10.;

            if self.main.real_score < 0. {
                // don't go negative...
                self.main.real_score = 0.;
            }

            if self.vars.me.energy > self.vars.me.health {
                self.vars.me.energy = self.vars.me.health;
            }

            if self.main.last_refresh_sound > 0.6 {
                self.refresh_sound();
                self.main.last_refresh_sound = 0.;
            }

            // since robots like the refresh, the influencer might also say so...
            if self.global.game_config.droid_talk {
                self.vars.me.text_to_be_displayed =
                    TextToBeDisplayed::String(c"Ahhh, that feels so good...");
                self.vars.me.text_visible_time = 0.;
            }
        } else {
            // If nothing more is to be had, the influencer might also say so...
            if self.global.game_config.droid_talk {
                self.vars.me.text_to_be_displayed =
                    TextToBeDisplayed::String(c"Oh, it seems that was it again.");
                self.vars.me.text_visible_time = 0.;
            }
        }
    }

    pub fn check_influence_enemy_collision(&mut self) {
        for enemy_index in 0..self.main.enemys.len() {
            let Self {
                vars,
                main,
                misc,
                global,
                ..
            } = self;
            let enemy = &mut main.enemys[enemy_index];

            /* ignore enemy that are not on this level or dead */
            if enemy.levelnum != cur_level!(main).levelnum {
                continue;
            }
            if matches!(enemy.status, Status::Out | Status::Terminated) {
                continue;
            }

            let x_dist = vars.me.pos.x - enemy.pos.x;
            let y_dist = vars.me.pos.y - enemy.pos.y;

            if x_dist.trunc().abs() > 1. {
                continue;
            }
            if y_dist.trunc().abs() > 1. {
                continue;
            }

            let dist2 = ((x_dist * x_dist) + (y_dist * y_dist)).sqrt();
            if dist2 > 2. * global.droid_radius {
                continue;
            }

            if vars.me.status == Status::Transfermode {
                self.takeover(enemy_index.try_into().unwrap());

                if self.level_empty() != 0 {
                    self.main.cur_level_mut().empty = true;
                }
            } else {
                vars.me.speed.x = -vars.me.speed.x;
                vars.me.speed.y = -vars.me.speed.y;

                if vars.me.speed.x != 0. {
                    vars.me.speed.x +=
                        COLLISION_PUSHSPEED * (vars.me.speed.x / vars.me.speed.x.abs());
                } else if x_dist != 0. {
                    vars.me.speed.x = COLLISION_PUSHSPEED * (x_dist / x_dist.abs());
                }
                if vars.me.speed.y != 0. {
                    vars.me.speed.y +=
                        COLLISION_PUSHSPEED * (vars.me.speed.y / vars.me.speed.y.abs());
                } else if y_dist != 0. {
                    vars.me.speed.y = COLLISION_PUSHSPEED * (y_dist / y_dist.abs());
                }

                // move the influencer a little bit out of the enemy AND the enemy a little bit out of the influ
                let max_step_size = if misc.frame_time(global, main.f_p_sover1) < MAXIMAL_STEP_SIZE
                {
                    misc.frame_time(global, main.f_p_sover1)
                } else {
                    MAXIMAL_STEP_SIZE
                };
                vars.me.pos.x += max_step_size.copysign(vars.me.pos.x - enemy.pos.x);
                vars.me.pos.y += max_step_size.copysign(vars.me.pos.y - enemy.pos.y);
                enemy.pos.x -= misc
                    .frame_time(global, main.f_p_sover1)
                    .copysign(vars.me.pos.x - enemy.pos.x);
                enemy.pos.y -= misc
                    .frame_time(global, main.f_p_sover1)
                    .copysign(vars.me.pos.y - enemy.pos.y);

                // there might be walls close too, so lets check again for collisions with them
                self.check_influence_wall_collisions();

                // shortly stop this enemy, then send him back to previous waypoint
                let enemy = &mut self.main.enemys[enemy_index];
                if enemy.warten == 0. {
                    enemy.warten = f32::from(WAIT_COLLISION);
                    std::mem::swap(&mut enemy.nextwaypoint, &mut enemy.lastwaypoint);

                    // Add some funny text!
                    self.enemy_influ_collision_text(enemy_index.try_into().unwrap());
                }
                /* someone loses energy ! */
                self.influ_enemy_collision_lose_energy(enemy_index.try_into().unwrap());
            }
        }
    }

    pub fn influ_enemy_collision_lose_energy(&mut self, enemy_num: i32) {
        let enemy = &mut self.main.enemys[usize::try_from(enemy_num).unwrap()];

        let damage = f32::from(
            i16::from(self.vars.droidmap[self.vars.me.ty.to_usize()].class)
                - i16::from(self.vars.droidmap[enemy.ty.to_usize()].class),
        ) * self.global.collision_lose_energy_calibrator;

        if damage < 0. {
            // we took damage
            self.collision_got_damaged_sound();
            if self.main.invincible_mode.not() {
                self.vars.me.energy += damage;
            }
        } else if damage == 0. {
            // nobody got hurt
            self.bounce_sound();
        } else {
            // damage > 0: enemy got damaged
            enemy.energy -= damage;
            self.collision_damaged_enemy_sound();
        }
    }

    pub fn explode_influencer(&mut self) {
        self.vars.me.status = Status::Terminated;
        let mut rng = thread_rng();

        for i in 0..10 {
            /* freien Blast finden */
            let mut counter = 0;
            loop {
                let check = self.main.all_blasts[usize::from(counter)].ty.is_some();
                counter += 1;
                if check.not() {
                    break;
                }
            }
            counter -= 1;
            assert!(
                counter < MAXBLASTS,
                "Went out of blasts in ExplodeInfluencer..."
            );
            let blast = &mut self.main.all_blasts[usize::from(counter)];
            blast.ty = Some(Explosion::Druidblast {
                from_influencer: false,
            });
            #[allow(clippy::cast_precision_loss)]
            {
                blast.px = self.vars.me.pos.x - self.global.droid_radius / 2.
                    + f32::from(rng.gen_range(0u8..=10)) * 0.05;
                blast.py = self.vars.me.pos.y - self.global.droid_radius / 2.
                    + f32::from(rng.gen_range(0u8..=10)) * 0.05;
                blast.phase = 0.2 * i as f32;
            }
        }

        self.play_sound(SoundType::Influexplosion);
    }

    /// This function checks for collisions of the influencer with walls,
    /// doors, consoles, boxes and all other map elements.
    /// In case of a collision, the position and speed of the influencer are
    /// adapted accordingly.
    /// NOTE: Of course this functions HAS to take into account the current framerate!
    pub fn check_influence_wall_collisions(&mut self) {
        let sx = self.vars.me.speed.x * self.frame_time();
        let sy = self.vars.me.speed.y * self.frame_time();
        let mut h_door_sliding_active = false;

        let lastpos = Finepoint {
            x: self.vars.me.pos.x - sx,
            y: self.vars.me.pos.y - sy,
        };

        let res = self.druid_passable(self.vars.me.pos.x, self.vars.me.pos.y);

        // Influence-Wall-Collision only has to be checked in case of
        // a collision of course, which is indicated by res not CENTER.
        if res != Some(Direction::Center) {
            //--------------------
            // At first we just check in which directions (from the last position)
            // the ways are blocked and in which directions the ways are open.
            //
            let north_south_axis_blocked = self.is_north_south_axis_blocked(lastpos);
            let east_west_axis_blocked = self.is_east_west_axis_blocked(lastpos);

            // Now we try to handle the sitution:

            if north_south_axis_blocked {
                // NorthSouthCorrectionDone=TRUE;
                self.vars.me.pos.y = lastpos.y;
                self.vars.me.speed.y = 0.;

                // if its an open door, we also correct the east-west position, in the
                // sense that we move thowards the middle
                if get_map_brick(
                    self.main.cur_level(),
                    self.vars.me.pos.x,
                    self.vars.me.pos.y - 0.5,
                ) == MapTile::HGanztuere as u8
                    || get_map_brick(
                        self.main.cur_level(),
                        self.vars.me.pos.x,
                        self.vars.me.pos.y + 0.5,
                    ) == MapTile::HGanztuere as u8
                {
                    self.vars.me.pos.x += f32::copysign(
                        PUSHSPEED * self.frame_time(),
                        self.vars.me.pos.x.round() - self.vars.me.pos.x,
                    );
                    h_door_sliding_active = true;
                }
            }

            if east_west_axis_blocked {
                // EastWestCorrectionDone=TRUE;
                if !h_door_sliding_active {
                    self.vars.me.pos.x = lastpos.x;
                }
                self.vars.me.speed.x = 0.;

                // if its an open door, we also correct the north-south position, in the
                // sense that we move thowards the middle
                if (get_map_brick(
                    self.main.cur_level(),
                    self.vars.me.pos.x + 0.5,
                    self.vars.me.pos.y,
                ) == MapTile::VGanztuere as u8)
                    || (get_map_brick(
                        self.main.cur_level(),
                        self.vars.me.pos.x - 0.5,
                        self.vars.me.pos.y,
                    ) == MapTile::VGanztuere as u8)
                {
                    self.vars.me.pos.y += f32::copysign(
                        PUSHSPEED * self.frame_time(),
                        self.vars.me.pos.y.round() - self.vars.me.pos.y,
                    );
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
                if self.druid_passable(self.vars.me.pos.x + sx, self.vars.me.pos.y)
                    == Some(Direction::Center)
                {
                    self.vars.me.pos.x += sx;
                }
                if self.druid_passable(self.vars.me.pos.x, self.vars.me.pos.y + sy)
                    == Some(Direction::Center)
                {
                    self.vars.me.pos.y += sy;
                }
            }

            // Here I introduce some extra security as a fallback:  Obviously
            // if the influencer is blocked FOR THE SECOND TIME, then the throw-back-algorithm
            // above HAS FAILED.  The absolutely fool-proof and secure handling is now done by
            // simply reverting to the last influ coordinated, where influ was NOT BLOCKED.
            // For this reason, a history of influ-coordinates has been introduced.  This will all
            // be done here and now:

            if (self.druid_passable(self.vars.me.pos.x, self.vars.me.pos.y)
                != Some(Direction::Center))
                && (self.druid_passable(
                    self.get_influ_position_history_x(0),
                    self.get_influ_position_history_y(0),
                ) != Some(Direction::Center))
                && (self.druid_passable(
                    self.get_influ_position_history_x(1),
                    self.get_influ_position_history_y(1),
                ) != Some(Direction::Center))
            {
                self.vars.me.pos.x = self.get_influ_position_history_x(2);
                self.vars.me.pos.y = self.get_influ_position_history_y(2);
                warn!("ATTENTION! CheckInfluenceWallCollsision FALLBACK ACTIVATED!!",);
            }
        }
    }

    pub fn animate_influence(&mut self) {
        if self.vars.me.ty == Droid::Droid001 {
            self.vars.me.phase += (self.vars.me.energy
                / (self.vars.droidmap[Droid::Droid001 as usize].maxenergy))
                * self.frame_time()
                * f32::from(ENEMYPHASES)
                * 3.;
        } else {
            self.vars.me.phase += (self.vars.me.energy
                / (self.vars.droidmap[self.vars.me.ty.to_usize()].maxenergy
                    + self.vars.droidmap[Droid::Droid001 as usize].maxenergy))
                * self.frame_time()
                * f32::from(ENEMYPHASES)
                * 3.;
        }

        if self.vars.me.phase.round() >= ENEMYPHASES.into() {
            self.vars.me.phase = 0.;
        }
    }

    /// This function moves the influencer, adjusts his speed according to
    /// keys pressed and also adjusts his status and current "phase" of his rotation.
    pub(crate) fn move_influence(&mut self) {
        let accel = self.vars.droidmap[self.vars.me.ty.to_usize()].accel * self.frame_time();

        // We store the influencers position for the history record and so that others
        // can follow his trail.

        self.influencer.current_zero_ring_index += 1;
        self.influencer.current_zero_ring_index %= MAX_INFLU_POSITION_HISTORY;
        self.vars.me.position_history_ring_buffer[self.influencer.current_zero_ring_index] = Gps {
            x: self.vars.me.pos.x,
            y: self.vars.me.pos.y,
            z: self.main.cur_level().levelnum,
        };

        self.permanent_lose_energy(); /* influ permanently loses energy */

        // check, if the influencer is still ok
        if self.vars.me.energy <= 0. {
            if self.vars.me.ty == Droid::Droid001 {
                self.vars.me.status = Status::Terminated;
                self.thou_art_defeated();
                return;
            }

            self.vars.me.ty = Droid::Droid001;
            self.vars.me.energy = BLINKENERGY;
            self.vars.me.health = BLINKENERGY;
            self.start_blast(
                self.vars.me.pos.x,
                self.vars.me.pos.y,
                Explosion::Druidblast {
                    from_influencer: true,
                },
            );
        }

        /* Time passed before entering Transfermode ?? */
        if self.influencer.transfer_counter >= WAIT_TRANSFERMODE {
            self.vars.me.status = Status::Transfermode;
            self.influencer.transfer_counter = 0.;
        }

        if self.up_pressed() {
            self.vars.me.speed.y -= accel;
        }
        if self.down_pressed() {
            self.vars.me.speed.y += accel;
        }
        if self.left_pressed() {
            self.vars.me.speed.x -= accel;
        }
        if self.right_pressed() {
            self.vars.me.speed.x += accel;
        }

        //  We only need this check if we want held fire to cause activate
        if !self.any_cmd_active() {
            // Used to be !SpacePressed, which causes any fire button != SPACE behave differently than space
            self.vars.me.status = Status::Mobile;
        }

        if (self.influencer.transfer_counter - 1.).abs() <= f32::EPSILON {
            self.vars.me.status = Status::Transfermode;
            self.influencer.transfer_counter = 0.;
        }

        if self.cmd_is_active(Cmds::Activate) {
            // activate mode for Konsole and Lifts
            self.vars.me.status = Status::Activate;
        }

        if self.global.game_config.fire_hold_takeover
            && self.fire_pressed()
            && self.no_direction_pressed()
            && matches!(self.vars.me.status, Status::Weapon | Status::Transfermode).not()
        {
            // Proposed FireActivatePressed here...
            self.influencer.transfer_counter += self.frame_time(); // Or make it an option!
        }

        if self.fire_pressed()
            && !self.no_direction_pressed()
            && self.vars.me.status != Status::Transfermode
        {
            self.vars.me.status = Status::Weapon;
        }

        if self.fire_pressed()
            && !self.no_direction_pressed()
            && self.vars.me.status == Status::Weapon
            && self.vars.me.firewait == 0.
        {
            self.fire_bullet();
        }

        if self.vars.me.status != Status::Weapon && self.cmd_is_active(Cmds::Takeover) {
            self.vars.me.status = Status::Transfermode;
        }

        self.influence_friction_with_air(); // The influ should lose some of his speed when no key is pressed

        self.adjust_speed(); // If the influ is faster than allowed for his type, slow him

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
        let mut planned_step_x = self.vars.me.speed.x * self.frame_time();
        let mut planned_step_y = self.vars.me.speed.y * self.frame_time();
        if planned_step_x.abs() >= MAXIMAL_STEP_SIZE {
            planned_step_x = f32::copysign(MAXIMAL_STEP_SIZE, planned_step_x);
        }
        if planned_step_y.abs() >= MAXIMAL_STEP_SIZE {
            planned_step_y = f32::copysign(MAXIMAL_STEP_SIZE, planned_step_y);
        }
        self.vars.me.pos.x += planned_step_x;
        self.vars.me.pos.y += planned_step_y;

        //--------------------
        // Check it the influ is on a special field like a lift, a console or a refresh
        self.act_special_field(self.vars.me.pos.x, self.vars.me.pos.y);

        self.animate_influence(); // move the "phase" of influencers rotation
    }

    pub fn permanent_lose_energy(&mut self) {
        // Of course if in invincible mode, no energy will ever be lost...
        if self.main.invincible_mode {
            return;
        }

        /* health decreases with time */
        self.vars.me.health -=
            self.vars.droidmap[self.vars.me.ty.to_usize()].lose_health * self.frame_time();

        /* you cant have more energy than health */
        if self.vars.me.energy > self.vars.me.health {
            self.vars.me.energy = self.vars.me.health;
        }
    }

    /// Fire-Routine for the Influencer only !! (should be changed)
    pub fn fire_bullet(&mut self) {
        let guntype = self.vars.droidmap[self.vars.me.ty.to_usize()].gun; /* which gun do we have ? */
        let bullet_speed = self.vars.bulletmap[guntype.to_usize()].speed;

        if self.vars.me.firewait > 0. {
            return;
        }
        self.vars.me.firewait = self.vars.bulletmap[guntype.to_usize()].recharging_time;

        self.fire_bullet_sound(guntype);

        let cur_bullet_index = self
            .main
            .all_bullets
            .iter()
            .position(Option::is_none)
            .unwrap_or(0);
        self.main.all_bullets[cur_bullet_index] = Some(Bullet {
            pos: self.vars.me.pos,
            ty: guntype,
            mine: true,
            ..Bullet::default_const()
        });

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
        #[allow(clippy::cast_precision_loss)]
        if self.input.axis_is_active {
            let max_val = self.input.axis.x.abs().max(self.input.axis.y.abs()) as f32;
            speed.x = self.input.axis.x as f32 / max_val;
            speed.y = self.input.axis.y as f32 / max_val;
        }

        let cur_bullet = self.main.all_bullets[cur_bullet_index].as_mut().unwrap();
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

    pub fn influence_friction_with_air(&mut self) {
        const DECELERATION: f32 = 7.0;

        if !self.right_pressed() && !self.left_pressed() {
            let oldsign = self.vars.me.speed.x.signum();
            let slowdown = oldsign * DECELERATION * self.frame_time();
            self.vars.me.speed.x -= slowdown;

            #[allow(clippy::float_cmp)]
            if self.vars.me.speed.x.signum() != oldsign {
                // changed direction -> vel=0
                self.vars.me.speed.x = 0.0;
            }
        }

        if !self.up_pressed() && !self.down_pressed() {
            let oldsign = self.vars.me.speed.y.signum();
            let slowdown = oldsign * DECELERATION * self.frame_time();
            self.vars.me.speed.y -= slowdown;

            #[allow(clippy::float_cmp)]
            if self.vars.me.speed.y.signum() != oldsign {
                // changed direction -> vel=0
                self.vars.me.speed.y = 0.0;
            }
        }
    }

    pub fn adjust_speed(&mut self) {
        let maxspeed = self.vars.droidmap[self.vars.me.ty.to_usize()].maxspeed;
        self.vars.me.speed.x = self.vars.me.speed.x.clamp(-maxspeed, maxspeed);
        self.vars.me.speed.y = self.vars.me.speed.y.clamp(-maxspeed, maxspeed);
    }

    pub fn get_position_history(&self, how_long_past: i32) -> &Gps {
        let ring_position = self.influencer.current_zero_ring_index + MAX_INFLU_POSITION_HISTORY
            - usize::try_from(how_long_past).unwrap();

        let ring_position = ring_position % MAX_INFLU_POSITION_HISTORY;

        &self.vars.me.position_history_ring_buffer[ring_position]
    }

    pub fn get_influ_position_history_x(&self, how_long_past: i32) -> f32 {
        self.get_position_history(how_long_past).x
    }

    pub fn get_influ_position_history_y(&self, how_long_past: i32) -> f32 {
        self.get_position_history(how_long_past).y
    }

    pub fn init_influ_position_history(&mut self) {
        self.vars.me.position_history_ring_buffer.fill(Gps {
            x: self.vars.me.pos.x,
            y: self.vars.me.pos.y,
            z: self.main.cur_level().levelnum,
        });
    }

    fn is_north_south_axis_blocked(&mut self, lastpos: Finepoint) -> bool {
        if {
            let pos_y = lastpos.y
                + self.vars.droidmap[self.vars.me.ty.to_usize()].maxspeed * self.frame_time();
            self.druid_passable(lastpos.x, pos_y) != Some(Direction::Center)
        } || {
            let pos_y = lastpos.y
                - self.vars.droidmap[self.vars.me.ty.to_usize()].maxspeed * self.frame_time();
            self.druid_passable(lastpos.x, pos_y) != Some(Direction::Center)
        } {
            true
        } else {
            info!("North-south-Axis seems to be free.");
            false
        }
    }

    fn is_east_west_axis_blocked(&mut self, lastpos: Finepoint) -> bool {
        ({
            let pos_x = lastpos.x
                + self.vars.droidmap[self.vars.me.ty.to_usize()].maxspeed * self.frame_time();
            self.druid_passable(pos_x, lastpos.y) == Some(Direction::Center)
        } && {
            let pos_x = lastpos.x
                - self.vars.droidmap[self.vars.me.ty.to_usize()].maxspeed * self.frame_time();
            self.druid_passable(pos_x, lastpos.y) == Some(Direction::Center)
        })
        .not()
    }
}
