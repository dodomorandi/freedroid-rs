use crate::{
    cur_level,
    defs::{
        Explosion, Status, AGGRESSIONMAX, DECKCOMPLETEBONUS, ENEMYMAXWAIT, ENEMYPHASES, MAXBULLETS,
        MAXWAYPOINTS, MAX_ENEMYS_ON_SHIP, ROBOT_MAX_WAIT_BETWEEN_SHOTS, SLOWMO_FACTOR,
        WAIT_COLLISION, WAIT_LEVELEMPTY,
    },
    misc::my_random,
    structs::Finepoint,
    Data,
};

use cstr::cstr;
use log::warn;
use std::{
    ops::Not,
    os::raw::{c_char, c_int},
};

/// according to the intro, the laser can be "focused on any target
/// within a range of eight metres"
const FIREDIST2: f32 = 8.;

const COL_SPEED: f32 = 3.;

impl Data<'_> {
    pub unsafe fn class_of_druid(&self, druid_type: c_int) -> c_int {
        /* first digit is class */
        let class_char =
            self.vars.droidmap[usize::try_from(druid_type).unwrap()].druidname[0] as u8;
        match class_char {
            b'0'..=b'9' => (class_char - b'0').into(),
            _ => 0,
        }
    }

    pub unsafe fn animate_enemys(&mut self) {
        let Self {
            main,
            misc,
            global,
            vars,
            ..
        } = self;

        let cur_level = cur_level!(mut main);
        for enemy in &mut main.all_enemys[..usize::try_from(main.num_enemys).unwrap()] {
            /* ignore enemys that are dead or on other levels or dummys */
            if enemy.levelnum != cur_level.levelnum {
                continue;
            }
            if enemy.status == Status::Out as i32 {
                continue;
            }

            enemy.phase += (enemy.energy
                / vars.droidmap[usize::try_from(enemy.ty).unwrap()].maxenergy)
                * misc.frame_time(global, main.f_p_sover1)
                * ENEMYPHASES as f32
                * 2.5;

            if enemy.phase >= ENEMYPHASES as f32 {
                enemy.phase = 0.;
            }
        }
    }

    /// This is the function, that move each of the enemys according to
    /// their orders and their program
    pub unsafe fn move_enemys(&mut self) {
        self.permanent_heal_robots(); // enemy robots heal as time passes...

        self.animate_enemys(); // move the "phase" of the rotation of enemys

        for enemy_index in 0..usize::try_from(self.main.num_enemys).unwrap() {
            let enemy = &self.main.all_enemys[enemy_index];
            if enemy.status == Status::Out as i32
                || enemy.status == Status::Terminated as i32
                || enemy.levelnum != self.main.cur_level().levelnum
            {
                continue;
            }

            let enemy_ty = enemy.ty;
            self.move_this_enemy(enemy_index.try_into().unwrap());

            // If its a combat droid, then if might attack...
            if self.vars.droidmap[usize::try_from(enemy_ty).unwrap()].aggression != 0 {
                self.attack_influence(enemy_index.try_into().unwrap());
            }
        }
    }

    /// AttackInfluence(): This function sometimes fires a bullet from
    /// enemy number enemynum directly into the direction of the influencer,
    /// but of course only if the odds are good i.e. requirements are met.
    pub unsafe fn attack_influence(&mut self, enemy_num: c_int) {
        let this_robot = &self.main.all_enemys[usize::try_from(enemy_num).unwrap()];
        // At first, we check for a lot of cases in which we do not
        // need to move anything for this reason or for that
        //

        // ignore robots on other levels
        if this_robot.levelnum != self.main.cur_level().levelnum {
            return;
        }

        // ignore dead robots as well...
        if this_robot.status == Status::Out as c_int {
            return;
        }

        let mut xdist = self.vars.me.pos.x - this_robot.pos.x;
        let mut ydist = self.vars.me.pos.y - this_robot.pos.y;

        // Add some security against division by zero
        if xdist == 0. {
            xdist = 0.01;
        }
        if ydist == 0. {
            ydist = 0.01;
        }

        // if odds are good, make a shot at your target
        let guntype = self.vars.droidmap[usize::try_from(this_robot.ty).unwrap()].gun;

        let dist2 = (xdist * xdist + ydist * ydist).sqrt();

        //--------------------
        //
        // From here on, it's classical Paradroid robot behaviour concerning fireing....
        //

        // distance limitation only for MS mechs
        if dist2 >= FIREDIST2 || this_robot.firewait != 0. || self.is_visible(&this_robot.pos) == 0
        {
            return;
        }

        if my_random(AGGRESSIONMAX)
            >= self.vars.droidmap[usize::try_from(this_robot.ty).unwrap()].aggression
        {
            self.main.all_enemys[usize::try_from(enemy_num).unwrap()].firewait +=
                my_random(1000) as f32 * ROBOT_MAX_WAIT_BETWEEN_SHOTS / 1000.0;
            return;
        }

        self.fire_bullet_sound(guntype);
        let this_robot = &mut self.main.all_enemys[usize::try_from(enemy_num).unwrap()];

        // find a bullet entry, that isn't currently used...
        let mut j = 0;
        while j < MAXBULLETS {
            if self.main.all_bullets[j].ty == Status::Out as u8 {
                break;
            }

            j += 1;
        }

        if j == MAXBULLETS {
            warn!("AttackInfluencer: no free bullets... giving up");
            return;
        }

        let cur_bullet = &mut self.main.all_bullets[j];
        // determine the direction of the shot, so that it will go into the direction of
        // the target

        if xdist.abs() > ydist.abs() {
            cur_bullet.speed.x = self.vars.bulletmap[usize::try_from(guntype).unwrap()].speed;
            cur_bullet.speed.y = ydist * cur_bullet.speed.x / xdist;
            if xdist < 0. {
                cur_bullet.speed.x = -cur_bullet.speed.x;
                cur_bullet.speed.y = -cur_bullet.speed.y;
            }
        }

        if xdist.abs() < ydist.abs() {
            cur_bullet.speed.y = self.vars.bulletmap[usize::try_from(guntype).unwrap()].speed;
            cur_bullet.speed.x = xdist * cur_bullet.speed.y / ydist;
            if ydist < 0. {
                cur_bullet.speed.x = -cur_bullet.speed.x;
                cur_bullet.speed.y = -cur_bullet.speed.y;
            }
        }

        cur_bullet.angle = -(90.
            + 180. * f32::atan2(cur_bullet.speed.y, cur_bullet.speed.x) / std::f32::consts::PI);

        cur_bullet.pos.x = this_robot.pos.x;
        cur_bullet.pos.y = this_robot.pos.y;

        cur_bullet.pos.x += (cur_bullet.speed.x)
            / (self.vars.bulletmap[usize::try_from(guntype).unwrap()].speed).abs()
            * 0.5;
        cur_bullet.pos.y += (cur_bullet.speed.y)
            / (self.vars.bulletmap[usize::try_from(guntype).unwrap()].speed).abs()
            * 0.5;

        this_robot.firewait = self.vars.bulletmap[usize::try_from(
            self.vars.droidmap[usize::try_from(this_robot.ty).unwrap()].gun,
        )
        .unwrap()]
        .recharging_time;

        cur_bullet.ty = guntype.try_into().unwrap();
        cur_bullet.time_in_frames = 0;
        cur_bullet.time_in_seconds = 0.;
    }

    pub unsafe fn move_this_enemy(&mut self, enemy_num: c_int) {
        let this_robot = &mut self.main.all_enemys[usize::try_from(enemy_num).unwrap()];

        // Now check if the robot is still alive
        // if the robot just got killed, initiate the
        // explosion and all that...
        if this_robot.energy <= 0. && (this_robot.status != Status::Terminated as c_int) {
            this_robot.status = Status::Terminated as c_int;
            self.main.real_score +=
                self.vars.droidmap[usize::try_from(this_robot.ty).unwrap()].score as f32;

            self.main.death_count += (this_robot.ty * this_robot.ty) as f32; // quadratic "importance", max=529

            let pos_x = this_robot.pos.x;
            let pos_y = this_robot.pos.y;
            self.start_blast(pos_x, pos_y, Explosion::Druidblast as c_int);
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
        if self.main.all_enemys[usize::try_from(enemy_num).unwrap()].warten > 0. {
            return;
        }

        // Now check for collisions of this enemy with his colleagues
        self.check_enemy_enemy_collision(enemy_num);

        // Now comes the real movement part
        self.move_this_robot_thowards_his_waypoint(enemy_num);

        self.select_next_waypoint_classical(enemy_num);
    }

    pub unsafe fn check_enemy_enemy_collision(&mut self, enemy_num: c_int) -> c_int {
        let Self {
            main, misc, global, ..
        } = self;

        let curlev = main.cur_level().levelnum;

        let enemy_num: usize = enemy_num.try_into().unwrap();
        let (enemys_before, rest) =
            main.all_enemys[..usize::try_from(main.num_enemys).unwrap()].split_at_mut(enemy_num);
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
            if dist <= (2. * global.droid_radius) {
                // am I waiting already?  If so, keep waiting...
                if cur_enemy.warten != 0. {
                    cur_enemy.warten = my_random(2 * WAIT_COLLISION) as f32;
                    continue;
                }

                enemy.warten = my_random(2 * WAIT_COLLISION) as f32;

                if xdist != 0. {
                    enemy.pos.x -= xdist / xdist.abs() * misc.frame_time(global, main.f_p_sover1);
                }
                if ydist != 0. {
                    enemy.pos.y -= ydist / ydist.abs() * misc.frame_time(global, main.f_p_sover1);
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
    pub unsafe fn select_next_waypoint_classical(&mut self, enemy_num: c_int) {
        let this_robot = &mut self.main.all_enemys[usize::try_from(enemy_num).unwrap()];

        // We do some definitions to save us some more typing later...
        let wp_list = cur_level!(self.main).all_waypoints;
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
            this_robot.warten = my_random(ENEMYMAXWAIT) as f32;

            let num_con = wp_list[nextwp].num_connections;
            if num_con > 0 {
                this_robot.nextwaypoint =
                    wp_list[nextwp].connections[usize::try_from(my_random(num_con - 1)).unwrap()];
            }
        }
    }

    pub unsafe fn shuffle_enemys(&mut self) {
        let cur_level = cur_level!(self.main);
        let cur_level_num = cur_level.levelnum;
        let mut used_wp: [bool; MAXWAYPOINTS] = [false; MAXWAYPOINTS];
        let mut warned = false;

        let num_wp = cur_level.num_waypoints;
        let mut nth_enemy = 0;

        for enemy in &mut self.main.all_enemys[..usize::try_from(self.main.num_enemys).unwrap()] {
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
                wp = usize::try_from(my_random(num_wp - 1)).unwrap();
                if used_wp[wp].not() {
                    break;
                }
            }

            used_wp[wp] = true;
            enemy.pos.x = cur_level.all_waypoints[wp].x.into();
            enemy.pos.y = cur_level.all_waypoints[wp].y.into();

            enemy.lastwaypoint = wp.try_into().unwrap();
            enemy.nextwaypoint = wp.try_into().unwrap();
        }
    }

    /// This function moves one robot thowards his next waypoint.  If already
    /// there, the function does nothing more.
    pub unsafe fn move_this_robot_thowards_his_waypoint(&mut self, enemy_num: c_int) {
        let Self {
            main,
            misc,
            global,
            vars,
            ..
        } = self;
        let this_robot = &mut main.all_enemys[usize::try_from(enemy_num).unwrap()];

        // We do some definitions to save us some more typing later...
        let wp_list = &cur_level!(main).all_waypoints;
        let nextwp: usize = this_robot.nextwaypoint.try_into().unwrap();
        let maxspeed = vars.droidmap[usize::try_from(this_robot.ty).unwrap()].maxspeed;

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

    pub unsafe fn clear_enemys(&mut self) {
        for enemy in &mut self.main.all_enemys[..MAX_ENEMYS_ON_SHIP] {
            enemy.ty = -1;
            enemy.levelnum = -1;
            enemy.phase = 0.;
            enemy.nextwaypoint = 0;
            enemy.lastwaypoint = 0;
            enemy.status = Status::Out as c_int;
            enemy.warten = 0.;
            enemy.firewait = 0.;
            enemy.energy = -1.;
            enemy.text_visible_time = 0.;
            enemy.text_to_be_displayed = cstr!("").as_ptr() as *mut c_char;
        }

        self.main.num_enemys = 0;
    }

    pub unsafe fn permanent_heal_robots(&mut self) {
        let Self {
            vars,
            misc,
            global,
            main,
            ..
        } = self;

        let f_p_sover1 = main.f_p_sover1;
        main.all_enemys[0..usize::try_from(main.num_enemys).unwrap()]
            .iter_mut()
            .filter(|enemy| {
                enemy.status != Status::Out as c_int
                    && enemy.energy > 0.
                    && enemy.energy < vars.droidmap[usize::try_from(enemy.ty).unwrap()].maxenergy
            })
            .for_each(|enemy| {
                enemy.energy += vars.droidmap[usize::try_from(enemy.ty).unwrap()].lose_health
                    * misc.frame_time(global, f_p_sover1);
            });
    }
}
