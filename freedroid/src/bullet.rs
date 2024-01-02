use crate::{
    defs::{
        BulletKind, Direction, Explosion, BULLET_COLL_DIST2, COLLISION_STEPSIZE, FLASH_DURATION,
        MAXBLASTS, MAXBULLETS,
    },
    structs::{Finepoint, Vect},
    Status,
};

use log::info;
use std::os::raw::{c_float, c_int};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Data {
    fbt_counter: u32,
}

impl crate::Data<'_> {
    #[inline]
    fn get_druid_hit_dist_squared(&self) -> f32 {
        (0.3 + 4. / 64.) * (self.global.droid_radius + 4. / 64.)
    }

    pub fn check_bullet_collisions(&mut self, num: c_int) {
        let bullet_index = usize::try_from(num).unwrap();
        let cur_bullet = &mut self.main.all_bullets[bullet_index];

        match BulletKind::try_from(cur_bullet.ty) {
            // Never do any collision checking if the bullet is OUT already...
            Err(_) => {}

            // --------------------
            // Next we handle the case that the bullet is of type FLASH
            Ok(BulletKind::Flash) => self.check_collision_with_flash(bullet_index),

            // --------------------
            // If its a "normal" Bullet, several checks have to be
            // done, one for collisions with background,
            // one for collision with influencer
            // some for collisions with enemys
            // and some for collisions with other bullets
            // and some for collisions with blast
            //
            _ => self.check_collision_with_normal(bullet_index),
        }
    }

    #[inline]
    fn check_collision_with_flash(&mut self, bullet_index: usize) {
        let level = self.main.cur_level().levelnum;
        let cur_bullet = &mut self.main.all_bullets[bullet_index];

        // if the flash is over, just delete it and return
        if cur_bullet.time_in_seconds >= FLASH_DURATION {
            cur_bullet.time_in_frames = 0;
            cur_bullet.time_in_seconds = 0.;
            cur_bullet.ty = Status::Out as u8;
            cur_bullet.mine = false;
        }

        // if the flash is not yet over, do some checking for who gets
        // hurt by it.
        // Two different methode for doing this are available:
        // The first but less elegant Method is just to check for
        // flash immunity, for distance and visiblity.
        // The second and more elegant method is to recursively fill
        // out the room where the flash-maker is in and to hurt all
        // robots in there except of course for those immune.
        if cur_bullet.time_in_frames != 1 {
            return;
        } // we only do the damage once and thats at frame nr. 1 of the flash

        for enemy_index in 0..usize::try_from(self.main.num_enemys).unwrap() {
            let enemy = &self.main.all_enemys[enemy_index];
            // !! dont't forget: Only droids on our level are harmed!! (bugfix)
            if enemy.levelnum != level {
                continue;
            }

            #[allow(clippy::cast_precision_loss)]
            if self.is_visible(enemy.pos) != 0
                && self.vars.droidmap[usize::try_from(enemy.ty).unwrap()].flashimmune == 0
            {
                let enemy = &mut self.main.all_enemys[enemy_index];
                enemy.energy -= self.vars.bulletmap[BulletKind::Flash as usize].damage as f32;

                // Since the enemy just got hit, it might as well say so :)
                self.enemy_hit_by_bullet_text(enemy_index.try_into().unwrap());
            }
        }

        // droids with flash are always flash-immune!
        // -> we don't get hurt by our own flashes!
        #[allow(clippy::cast_precision_loss)]
        if self.main.invincible_mode == 0
            && self.vars.droidmap[usize::try_from(self.vars.me.ty).unwrap()].flashimmune == 0
        {
            self.vars.me.energy -= self.vars.bulletmap[BulletKind::Flash as usize].damage as f32;
        }
    }

    #[inline]
    fn check_collision_with_normal(&mut self, cur_bullet_index: usize) {
        let level = self.main.cur_level().levelnum;
        let cur_bullet = &mut self.main.all_bullets[cur_bullet_index];

        // first check for collision with background
        let mut step = Finepoint {
            x: cur_bullet.pos.x - cur_bullet.prev_pos.x,
            y: cur_bullet.pos.y - cur_bullet.prev_pos.y,
        };
        let mut num_check_steps =
            ((step.x * step.x + step.y * step.y).sqrt() / COLLISION_STEPSIZE).trunc();
        if num_check_steps == 0. {
            num_check_steps = 1.;
        }
        step.x /= num_check_steps;
        step.y /= num_check_steps;

        cur_bullet.pos.x = cur_bullet.prev_pos.x;
        cur_bullet.pos.y = cur_bullet.prev_pos.y;

        #[allow(clippy::cast_possible_truncation)]
        for _ in 0..(num_check_steps as i32) {
            let cur_bullet = &mut self.main.all_bullets[cur_bullet_index];
            cur_bullet.pos.x += step.x;
            cur_bullet.pos.y += step.y;

            let cur_bullet = &self.main.all_bullets[cur_bullet_index];
            if self.is_passable(
                cur_bullet.pos.x,
                cur_bullet.pos.y,
                Direction::Center as c_int,
            ) != Direction::Center as c_int
            {
                let pos_x = cur_bullet.pos.x;
                let pos_y = cur_bullet.pos.y;
                self.start_blast(pos_x, pos_y, Explosion::Bulletblast as c_int);
                self.delete_bullet(cur_bullet_index.try_into().unwrap());
                return;
            }

            // check for collision with influencer
            if !cur_bullet.mine {
                let x_dist = self.vars.me.pos.x - cur_bullet.pos.x;
                let y_dist = self.vars.me.pos.y - cur_bullet.pos.y;
                // FIXME: don't use DRUIDHITDIST2!!
                if (x_dist * x_dist + y_dist * y_dist) < self.get_druid_hit_dist_squared() {
                    self.got_hit_sound();

                    #[allow(clippy::cast_precision_loss)]
                    if self.main.invincible_mode == 0 {
                        self.vars.me.energy -=
                            self.vars.bulletmap[usize::from(cur_bullet.ty)].damage as f32;
                    }

                    self.delete_bullet(cur_bullet_index.try_into().unwrap());
                    return;
                }
            }

            // check for collision with enemys
            for (enemy_index, enemy) in self.main.all_enemys
                [..usize::try_from(self.main.num_enemys).unwrap()]
                .iter()
                .enumerate()
            {
                if enemy.status == Status::Out as c_int
                    || enemy.status == Status::Terminated as c_int
                    || enemy.levelnum != level
                {
                    continue;
                }

                let x_dist = cur_bullet.pos.x - enemy.pos.x;
                let y_dist = cur_bullet.pos.y - enemy.pos.y;

                // FIXME
                #[allow(clippy::cast_precision_loss)]
                if (x_dist * x_dist + y_dist * y_dist) < self.get_druid_hit_dist_squared() {
                    // The enemy who was hit, loses some energy, depending on the bullet
                    self.main.all_enemys[enemy_index].energy -=
                        self.vars.bulletmap[usize::from(cur_bullet.ty)].damage as f32;

                    self.delete_bullet(cur_bullet_index.try_into().unwrap());
                    self.got_hit_sound();

                    let cur_bullet = &mut self.main.all_bullets[cur_bullet_index];
                    if !cur_bullet.mine {
                        self.bullet.fbt_counter += 1;
                    }
                    cur_bullet.ty = Status::Out as u8;
                    cur_bullet.mine = false;
                    return;
                }
            }

            // check for collisions with other bullets
            for bullet_index in 0..MAXBULLETS {
                // never check for collision with youself.. ;)
                if bullet_index == cur_bullet_index {
                    continue;
                }
                let bullet = &self.main.all_bullets[bullet_index];
                if bullet.ty == Status::Out as u8 {
                    continue;
                } // never check for collisions with dead bullets..
                if bullet.ty == BulletKind::Flash as u8 {
                    continue;
                } // never check for collisions with flashes bullets..

                let cur_bullet = &self.main.all_bullets[cur_bullet_index];
                let x_dist = bullet.pos.x - cur_bullet.pos.x;
                let y_dist = bullet.pos.y - cur_bullet.pos.y;
                if x_dist * x_dist + y_dist * y_dist > BULLET_COLL_DIST2 {
                    continue;
                }

                // it seems like we have a collision of two bullets!
                // both will be deleted and replaced by blasts..
                info!("Bullet-Bullet-Collision detected...");

                let pos_x = cur_bullet.pos.x;
                let pos_y = cur_bullet.pos.y;
                self.start_blast(pos_x, pos_y, Explosion::Druidblast as c_int);

                self.delete_bullet(cur_bullet_index.try_into().unwrap());
                self.delete_bullet(bullet_index.try_into().unwrap());
            }
        }
    }

    pub fn delete_blast(&mut self, num: c_int) {
        self.main.all_blasts[usize::try_from(num).unwrap()].ty = Status::Out as c_int;
    }

    pub fn explode_blasts(&mut self) {
        for blast_index in 0..MAXBLASTS {
            let cur_blast = &self.main.all_blasts[blast_index];
            #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
            if cur_blast.ty != Status::Out as c_int {
                if cur_blast.ty == Explosion::Druidblast as c_int {
                    self.check_blast_collisions(blast_index.try_into().unwrap());
                }

                let Self {
                    main,
                    misc,
                    global,
                    vars,
                    ..
                } = self;

                let frame_time = misc.frame_time(global, main.f_p_sover1);
                let cur_blast = &mut main.all_blasts[blast_index];
                let blast_spec = &vars.blastmap[usize::try_from(cur_blast.ty).unwrap()];
                cur_blast.phase +=
                    frame_time * blast_spec.phases as f32 / blast_spec.total_animation_time;
                if cur_blast.phase.floor() as c_int >= blast_spec.phases {
                    self.delete_blast(blast_index.try_into().unwrap());
                }
            }
        }
    }

    pub fn check_blast_collisions(&mut self, num: c_int) {
        let level = self.main.cur_level().levelnum;
        /* check Blast-Bullet Collisions and kill hit Bullets */
        for bullet_index in 0..MAXBULLETS {
            let cur_blast = &self.main.all_blasts[usize::try_from(num).unwrap()];
            let cur_bullet = &self.main.all_bullets[bullet_index];
            if cur_bullet.ty == Status::Out as u8 {
                continue;
            }

            let v_dist = Vect {
                x: cur_bullet.pos.x - cur_blast.px,
                y: cur_bullet.pos.y - cur_blast.py,
            };
            let dist = (v_dist.x * v_dist.x + v_dist.y * v_dist.y).sqrt();
            if dist < self.global.blast_radius {
                let pos_x = cur_bullet.pos.x;
                let pos_y = cur_bullet.pos.y;
                self.start_blast(pos_x, pos_y, Explosion::Bulletblast as c_int);
                self.delete_bullet(bullet_index.try_into().unwrap());
            }
        }

        /* Check Blast-Enemy Collisions and smash energy of hit enemy */
        let Self {
            main, global, misc, ..
        } = self;
        let cur_blast = &main.all_blasts[usize::try_from(num).unwrap()];
        for enemy in &mut main.all_enemys[..usize::try_from(main.num_enemys).unwrap()] {
            if enemy.status == Status::Out as c_int || enemy.levelnum != level {
                continue;
            }

            let v_dist = Vect {
                x: enemy.pos.x - cur_blast.px,
                y: enemy.pos.y - cur_blast.py,
            };
            let dist = (v_dist.x * v_dist.x + v_dist.y * v_dist.y).sqrt();

            if dist < global.blast_radius + global.droid_radius {
                /* drag energy of enemy */
                enemy.energy -=
                    global.blast_damage_per_second * misc.frame_time(global, main.f_p_sover1);
            }

            if enemy.energy < 0. {
                enemy.energy = 0.;
            }
        }

        /* Check influence-Blast collisions */
        let v_dist = Vect {
            x: self.vars.me.pos.x - cur_blast.px,
            y: self.vars.me.pos.y - cur_blast.py,
        };
        let dist = (v_dist.x * v_dist.x + v_dist.y * v_dist.y).sqrt();

        if self.vars.me.status != Status::Out as c_int
            && !cur_blast.mine
            && dist < self.global.blast_radius + self.global.droid_radius
        {
            if self.main.invincible_mode == 0 {
                self.vars.me.energy -= self.global.blast_damage_per_second * self.frame_time();
                let cur_blast = &self.main.all_blasts[usize::try_from(num).unwrap()];

                // So the influencer got some damage from the hot blast
                // Now most likely, he then will also say so :)
                if cur_blast.message_was_done == 0 {
                    self.add_influ_burnt_text();
                    let cur_blast = &mut self.main.all_blasts[usize::try_from(num).unwrap()];
                    cur_blast.message_was_done = true.into();
                }
            }
            // In order to avoid a new sound EVERY frame we check for how long the previous blast
            // lies back in time.  LastBlastHit is a float, that counts SECONDS real-time !!
            if self.main.last_got_into_blast_sound > 1.2 {
                self.got_into_blast_sound();
                self.main.last_got_into_blast_sound = 0.;
            }
        }
    }

    pub fn start_blast(&mut self, x: c_float, y: c_float, mut ty: c_int) {
        let mut i = 0;
        while i < MAXBLASTS {
            if self.main.all_blasts[i].ty == Status::Out as c_int {
                break;
            }

            i += 1;
        }

        if i >= MAXBLASTS {
            i = 0;
        }

        /* Get Pointer to it: more comfortable */
        let new_blast = &mut self.main.all_blasts[i];

        if ty == Explosion::Rejectblast as c_int {
            new_blast.mine = true;
            ty = Explosion::Druidblast as c_int; // not really a different type, just avoid damaging influencer
        } else {
            new_blast.mine = false;
        }

        new_blast.px = x;
        new_blast.py = y;

        new_blast.ty = ty;
        new_blast.phase = 0.;

        new_blast.message_was_done = 0;

        if ty == Explosion::Druidblast as c_int {
            self.druid_blast_sound();
        }
    }

    /// delete bullet of given number, set it type=OUT, put it at x/y=-1/-1
    /// and create a Bullet-blast if `with_blast==TRUE`
    pub fn delete_bullet(&mut self, bullet_number: c_int) {
        let cur_bullet = &mut self.main.all_bullets[usize::try_from(bullet_number).unwrap()];

        if cur_bullet.ty == Status::Out as u8 {
            // ignore dead bullets
            return;
        }

        //--------------------
        // At first we generate the blast at the collision spot of the bullet,
        // cause later, after the bullet is deleted, it will be hard to know
        // the correct location ;)

        // RP (18/11/02): nay, we do that manually before DeleteBullet() now,
        // --> not all bullets should create Blasts (i.e. not if droid was hit)
        //  StartBlast (CurBullet->pos.x, CurBullet->pos.y, BULLETBLAST);

        //--------------------
        // maybe, the bullet had several SDL_Surfaces attached to it.  Then we need to
        // free the SDL_Surfaces again as well...
        //
        if cur_bullet.surfaces_were_generated != 0 {
            info!("DeleteBullet: freeing this bullets attached surfaces...");
            let bullet_spec = &self.vars.bulletmap[usize::from(cur_bullet.ty)];
            for phase in 0..usize::try_from(bullet_spec.phases).unwrap() {
                cur_bullet.surfaces[phase] = None;
            }
            cur_bullet.surfaces_were_generated = false.into();
        }

        //--------------------
        // Now that the memory has been freed again, we can finally delete this bullet entry.
        // Hope, that this does not give us a SEGFAULT, but it should not do so.
        //
        cur_bullet.ty = Status::Out as u8;
        cur_bullet.time_in_seconds = 0.;
        cur_bullet.time_in_frames = 0;
        cur_bullet.mine = false;
        cur_bullet.phase = 0;
        cur_bullet.pos.x = -1.;
        cur_bullet.pos.y = -1.;
        cur_bullet.angle = 0.;
    }

    /// This function moves all the bullets according to their speeds.
    ///
    /// NEW: this function also takes into accoung the current framerate.
    pub fn move_bullets(&mut self) {
        let Self {
            main, misc, global, ..
        } = self;
        for cur_bullet in &mut main.all_bullets[..MAXBULLETS] {
            if cur_bullet.ty == Status::Out as u8 {
                continue;
            }

            cur_bullet.prev_pos.x = cur_bullet.pos.x;
            cur_bullet.prev_pos.y = cur_bullet.pos.y;

            cur_bullet.pos.x += cur_bullet.speed.x * misc.frame_time(global, main.f_p_sover1);
            cur_bullet.pos.y += cur_bullet.speed.y * misc.frame_time(global, main.f_p_sover1);

            cur_bullet.time_in_frames += 1;
            cur_bullet.time_in_seconds += misc.frame_time(global, main.f_p_sover1);
        }
    }
}
