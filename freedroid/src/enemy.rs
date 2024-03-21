use std::ops::Not;

use crate::{
    cur_level,
    defs::{
        Droid, Explosion, Status, AGGRESSIONMAX, DECKCOMPLETEBONUS, ENEMYMAXWAIT, ENEMYPHASES,
        MAXBULLETS, MAXWAYPOINTS, ROBOT_MAX_WAIT_BETWEEN_SHOTS, SLOWMO_FACTOR, WAIT_COLLISION,
        WAIT_LEVELEMPTY,
    },
    structs::{Bullet, Finepoint},
};

use log::warn;
use rand::{
    seq::{IteratorRandom, SliceRandom},
    thread_rng, Rng,
};
use sdl::convert::u8_to_usize;

/// according to the intro, the laser can be "focused on any target
/// within a range of eight metres"
const FIREDIST2: f32 = 8.;

const COL_SPEED: f32 = 3.;

impl crate::Data<'_> {
    pub fn class_of_druid(&self, druid_type: Droid) -> i32 {
        /* first digit is class */
        let class_char = self.vars.droidmap[druid_type.to_usize()].druidname[0];
        match class_char {
            b'0'..=b'9' => (class_char - b'0').into(),
            _ => 0,
        }
    }

    pub fn animate_enemys(&mut self) {
        let Self {
            main,
            misc,
            global,
            vars,
            ..
        } = self;

        let cur_level = cur_level!(mut main);
        for enemy in &mut main.enemys {
            /* ignore enemys that are dead or on other levels or dummys */
            if enemy.levelnum != cur_level.levelnum {
                continue;
            }
            if enemy.status == Status::Out {
                continue;
            }

            enemy.phase += (enemy.energy / vars.droidmap[enemy.ty.to_usize()].maxenergy)
                * misc.frame_time(global, main.f_p_sover1)
                * f32::from(ENEMYPHASES)
                * 2.5;

            if enemy.phase >= f32::from(ENEMYPHASES) {
                enemy.phase = 0.;
            }
        }
    }

    /// This is the function, that move each of the enemys according to
    /// their orders and their program
    pub fn move_enemys(&mut self) {
        self.permanent_heal_robots(); // enemy robots heal as time passes...

        self.animate_enemys(); // move the "phase" of the rotation of enemys

        for enemy_index in 0..self.main.enemys.len() {
            let enemy = &self.main.enemys[enemy_index];
            if matches!(enemy.status, Status::Out | Status::Terminated)
                || enemy.levelnum != self.main.cur_level().levelnum
            {
                continue;
            }

            let enemy_ty = enemy.ty;
            self.move_this_enemy(enemy_index.try_into().unwrap());

            // If its a combat droid, then if might attack...
            if self.vars.droidmap[enemy_ty.to_usize()].aggression != 0 {
                self.attack_influence(enemy_index.try_into().unwrap());
            }
        }
    }

    /// `AttackInfluence()`: This function sometimes fires a bullet from
    /// enemy number enemynum directly into the direction of the influencer,
    /// but of course only if the odds are good i.e. requirements are met.
    pub fn attack_influence(&mut self, enemy_num: i32) {
        let this_robot = &self.main.enemys[usize::try_from(enemy_num).unwrap()];
        // At first, we check for a lot of cases in which we do not
        // need to move anything for this reason or for that

        // ignore robots on other levels
        if this_robot.levelnum != self.main.cur_level().levelnum {
            return;
        }

        // ignore dead robots as well...
        if this_robot.status == Status::Out {
            return;
        }

        let mut x_dist = self.vars.me.pos.x - this_robot.pos.x;
        let mut y_dist = self.vars.me.pos.y - this_robot.pos.y;

        // Add some security against division by zero
        if x_dist == 0. {
            x_dist = 0.01;
        }
        if y_dist == 0. {
            y_dist = 0.01;
        }

        // if odds are good, make a shot at your target
        let guntype = self.vars.droidmap[this_robot.ty.to_usize()].gun;

        let dist2 = (x_dist * x_dist + y_dist * y_dist).sqrt();

        //--------------------
        //
        // From here on, it's classical Paradroid robot behaviour concerning fireing....
        //

        // distance limitation only for MS mechs
        if dist2 >= FIREDIST2 || this_robot.firewait != 0. || self.is_visible(this_robot.pos) == 0 {
            return;
        }

        let mut rng = thread_rng();
        #[allow(clippy::cast_precision_loss)]
        if (0..=AGGRESSIONMAX).choose(&mut rng).unwrap()
            >= self.vars.droidmap[this_robot.ty.to_usize()].aggression
        {
            self.main.enemys[usize::try_from(enemy_num).unwrap()].firewait +=
                rng.gen_range(0f32..=1f32) * ROBOT_MAX_WAIT_BETWEEN_SHOTS;
            return;
        }

        self.fire_bullet_sound(guntype);
        let this_robot = &mut self.main.enemys[usize::try_from(enemy_num).unwrap()];

        // find a bullet entry, that isn't currently used...
        let mut j = 0;
        while j < MAXBULLETS {
            if self.main.all_bullets[j].is_none() {
                break;
            }

            j += 1;
        }

        if j == MAXBULLETS {
            warn!("AttackInfluencer: no free bullets... giving up");
            return;
        }

        // determine the direction of the shot, so that it will go into the direction of
        // the target

        let mut speed = Finepoint::default_const();
        if x_dist.abs() > y_dist.abs() {
            speed.x = self.vars.bulletmap[guntype.to_usize()].speed;
            speed.y = y_dist * speed.x / x_dist;
            if x_dist < 0. {
                speed.x = -speed.x;
                speed.y = -speed.y;
            }
        }

        if x_dist.abs() < y_dist.abs() {
            speed.y = self.vars.bulletmap[guntype.to_usize()].speed;
            speed.x = x_dist * speed.y / y_dist;
            if y_dist < 0. {
                speed.x = -speed.x;
                speed.y = -speed.y;
            }
        }

        let angle = -(90. + 180. * f32::atan2(speed.y, speed.x) / std::f32::consts::PI);

        let mut pos = this_robot.pos;
        pos.x += speed.x / (self.vars.bulletmap[guntype.to_usize()].speed).abs() * 0.5;
        pos.y += speed.y / (self.vars.bulletmap[guntype.to_usize()].speed).abs() * 0.5;

        this_robot.firewait = self.vars.bulletmap
            [self.vars.droidmap[this_robot.ty.to_usize()].gun.to_usize()]
        .recharging_time;

        self.main.all_bullets[j] = Some(Bullet {
            pos,
            speed,
            ty: guntype,
            angle,
            time_in_frames: 0,
            time_in_seconds: 0.,
            ..Bullet::default_const()
        });
    }

    pub fn move_this_enemy(&mut self, enemy_num: i32) {
        let this_robot = &mut self.main.enemys[usize::try_from(enemy_num).unwrap()];

        // Now check if the robot is still alive
        // if the robot just got killed, initiate the
        // explosion and all that...
        #[allow(clippy::cast_precision_loss)]
        if this_robot.energy <= 0. && (this_robot.status != Status::Terminated) {
            this_robot.status = Status::Terminated;
            self.main.real_score += self.vars.droidmap[this_robot.ty.to_usize()].score as f32;

            self.main.death_count += f32::from(this_robot.ty.to_u16().pow(2)); // quadratic "importance", max=529

            let pos_x = this_robot.pos.x;
            let pos_y = this_robot.pos.y;
            self.start_blast(pos_x, pos_y, Explosion::Druidblast as i32);
            if self.level_empty() != 0 {
                self.main.real_score += DECKCOMPLETEBONUS;

                let cur_level = self.main.cur_level_mut();
                cur_level.empty = true.into();
                cur_level.timer = WAIT_LEVELEMPTY;
                self.set_time_factor(SLOWMO_FACTOR); // watch final explosion in slow-motion
            }
            return; // this one's down, so we can move on to the next
        }

        // robots that still have to wait also do not need to
        // be processed for movement
        if this_robot.warten > 0. {
            return;
        }

        // Now check for collisions of this enemy with his colleagues
        self.check_enemy_enemy_collision(enemy_num);

        // Now comes the real movement part
        self.move_this_robot_thowards_his_waypoint(enemy_num);

        self.select_next_waypoint_classical(enemy_num);
    }

    pub fn check_enemy_enemy_collision(&mut self, enemy_num: i32) -> i32 {
        let Self {
            main, misc, global, ..
        } = self;

        let curlev = main.cur_level().levelnum;

        let enemy_num: usize = enemy_num.try_into().unwrap();
        let (enemys_before, rest) = main.enemys.split_at_mut(enemy_num);
        let (cur_enemy, enemys_after) = rest.split_first_mut().unwrap();
        let check_x = cur_enemy.pos.x;
        let check_y = cur_enemy.pos.y;

        let mut rng = thread_rng();
        for enemy in enemys_before.iter_mut().chain(enemys_after) {
            // check only collisions of LIVING enemys on this level
            if matches!(enemy.status, Status::Out | Status::Terminated) || enemy.levelnum != curlev
            {
                continue;
            }

            /* get distance between enemy and cur_enemy */
            let x_dist = check_x - enemy.pos.x;
            let y_dist = check_y - enemy.pos.y;

            let dist = (x_dist * x_dist + y_dist * y_dist).sqrt();

            // Is there a Collision?
            if dist <= (2. * global.droid_radius) {
                // am I waiting already?  If so, keep waiting...
                if cur_enemy.warten != 0. {
                    cur_enemy.warten = rng.gen_range(0..=(WAIT_COLLISION * 2)).into();
                    continue;
                }

                enemy.warten = rng.gen_range(0..=(WAIT_COLLISION * 2)).into();

                if x_dist != 0. {
                    enemy.pos.x -= x_dist / x_dist.abs() * misc.frame_time(global, main.f_p_sover1);
                }
                if y_dist != 0. {
                    enemy.pos.y -= y_dist / y_dist.abs() * misc.frame_time(global, main.f_p_sover1);
                }

                std::mem::swap(&mut cur_enemy.nextwaypoint, &mut cur_enemy.lastwaypoint);

                let speed_x = cur_enemy.speed.x;
                let speed_y = cur_enemy.speed.y;

                if speed_x != 0. {
                    cur_enemy.pos.x -=
                        misc.frame_time(global, main.f_p_sover1) * COL_SPEED * (speed_x)
                            / speed_x.abs();
                }
                if speed_y != 0. {
                    cur_enemy.pos.y -=
                        misc.frame_time(global, main.f_p_sover1) * COL_SPEED * (speed_y)
                            / speed_y.abs();
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
    pub fn select_next_waypoint_classical(&mut self, enemy_num: i32) {
        let this_robot = &mut self.main.enemys[usize::try_from(enemy_num).unwrap()];

        // We do some definitions to save us some more typing later...
        let wp_list = &cur_level!(self.main).waypoints;
        let nextwp = usize::from(this_robot.nextwaypoint);

        // determine the remaining way until the target point is reached
        let restweg = Finepoint {
            x: f32::from(wp_list[nextwp].x) - this_robot.pos.x,
            y: f32::from(wp_list[nextwp].y) - this_robot.pos.y,
        };

        // Now we can see if we are perhaps already there?
        // then it might be time to set a new waypoint.
        //
        if restweg.x == 0. && restweg.y == 0. {
            let mut rng = thread_rng();
            this_robot.lastwaypoint = this_robot.nextwaypoint;
            this_robot.warten = rng.gen_range(0..=ENEMYMAXWAIT).into();

            if let Some(connection) = wp_list[nextwp].connections.choose(&mut rng).copied() {
                this_robot.nextwaypoint = connection;
            }
        }
    }

    pub fn shuffle_enemys(&mut self) {
        let cur_level = cur_level!(self.main);
        let cur_level_num = cur_level.levelnum;
        let mut used_wp = [false; u8_to_usize(MAXWAYPOINTS)];
        let mut warned = false;

        let num_wp = u8::try_from(cur_level.waypoints.len()).unwrap();
        let mut nth_enemy = 0;

        let mut rng = thread_rng();
        for enemy in &mut self.main.enemys {
            if enemy.status == Status::Out || enemy.levelnum != cur_level_num {
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
                enemy.status = Status::Out;
                continue;
            }

            let mut wp;
            loop {
                wp = rng.gen_range(0..num_wp);
                if used_wp[usize::from(wp)].not() {
                    break;
                }
            }

            used_wp[usize::from(wp)] = true;
            enemy.pos.x = cur_level.waypoints[usize::from(wp)].x.into();
            enemy.pos.y = cur_level.waypoints[usize::from(wp)].y.into();

            enemy.lastwaypoint = wp;
            enemy.nextwaypoint = wp;
        }
    }

    /// This function moves one robot thowards his next waypoint.  If already
    /// there, the function does nothing more.
    pub fn move_this_robot_thowards_his_waypoint(&mut self, enemy_num: i32) {
        let Self {
            main,
            misc,
            global,
            vars,
            ..
        } = self;
        let this_robot = &mut main.enemys[usize::try_from(enemy_num).unwrap()];

        // We do some definitions to save us some more typing later...
        let wp_list = &cur_level!(main).waypoints;
        let nextwp: usize = this_robot.nextwaypoint.into();
        let maxspeed = vars.droidmap[this_robot.ty.to_usize()].maxspeed;

        let nextwp_pos = Finepoint {
            x: wp_list[nextwp].x.into(),
            y: wp_list[nextwp].y.into(),
        };

        // determine the remaining way until the target point is reached
        let restweg = Finepoint {
            x: nextwp_pos.x - this_robot.pos.x,
            y: nextwp_pos.y - this_robot.pos.y,
        };

        let steplen = misc.frame_time(global, main.f_p_sover1) * maxspeed;
        // As long a the distance from the current position of the enemy
        // to its next wp is large, movement is rather simple:

        let dist = (restweg.x * restweg.x + restweg.y * restweg.y).sqrt();
        if dist > steplen {
            this_robot.speed.x = (restweg.x / dist) * maxspeed;
            this_robot.speed.y = (restweg.y / dist) * maxspeed;
            this_robot.pos.x += this_robot.speed.x * misc.frame_time(global, main.f_p_sover1);
            this_robot.pos.y += this_robot.speed.y * misc.frame_time(global, main.f_p_sover1);
        } else {
            // If this enemy is just one step ahead of his target, we just put him there now
            this_robot.pos.x = nextwp_pos.x;
            this_robot.pos.y = nextwp_pos.y;
            this_robot.speed.x = 0.;
            this_robot.speed.y = 0.;
        }
    }

    pub fn permanent_heal_robots(&mut self) {
        let Self {
            vars,
            misc,
            global,
            main,
            ..
        } = self;

        let f_p_sover1 = main.f_p_sover1;
        main.enemys
            .iter_mut()
            .filter(|enemy| {
                enemy.status != Status::Out
                    && enemy.energy > 0.
                    && enemy.energy < vars.droidmap[enemy.ty.to_usize()].maxenergy
            })
            .for_each(|enemy| {
                enemy.energy += vars.droidmap[enemy.ty.to_usize()].lose_health
                    * misc.frame_time(global, f_p_sover1);
            });
    }
}
