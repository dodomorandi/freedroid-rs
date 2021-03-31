use crate::{
    defs::{
        BulletKind, Direction, Explosion, BULLET_COLL_DIST2, COLLISION_STEPSIZE, FLASH_DURATION,
        MAXBLASTS, MAXBULLETS,
    },
    global::Droid_Radius,
    map::{IsPassable, IsVisible},
    misc::Frame_Time,
    sound::GotHitSound,
    structs::Finepoint,
    text::EnemyHitByBulletText,
    vars::{Blastmap, Bulletmap, Druidmap},
    AllBlasts, AllBullets, AllEnemys, CurLevel, InvincibleMode, Me, NumEnemys, Status,
};

use log::info;
use std::{
    convert::{TryFrom, TryInto},
    os::raw::{c_float, c_int},
};

extern "C" {
    pub fn DeleteBullet(num: c_int);
    pub fn MoveBullets();
    pub fn StartBlast(x: c_float, y: c_float, ty: c_int);
    pub fn CheckBlastCollisions(num: c_int);
}

#[inline]
unsafe fn get_druid_hit_dist_squared() -> f32 {
    (0.3 + 4. / 64.) * (Droid_Radius + 4. / 64.)
}

#[no_mangle]
pub unsafe extern "C" fn CheckBulletCollisions(num: c_int) {
    let level = (*CurLevel).levelnum;
    let cur_bullet = &mut AllBullets[usize::try_from(num).unwrap()];
    static mut FBT_COUNTER: c_int = 0;

    match BulletKind::try_from(cur_bullet.ty) {
        // Never do any collision checking if the bullet is OUT already...
        Err(_) => {}

        // --------------------
        // Next we handle the case that the bullet is of type FLASH
        Ok(BulletKind::Flash) => {
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

            for (i, enemy) in AllEnemys[..usize::try_from(NumEnemys).unwrap()]
                .iter_mut()
                .enumerate()
            {
                // !! dont't forget: Only droids on our level are harmed!! (bugfix)
                if enemy.levelnum != level {
                    continue;
                }

                if IsVisible(&enemy.pos) != 0
                    && (*Druidmap.add(usize::try_from(enemy.ty).unwrap())).flashimmune == 0
                {
                    enemy.energy -= (*Bulletmap.add(BulletKind::Flash as usize)).damage as f32;
                    // Since the enemy just got hit, it might as well say so :)
                    EnemyHitByBulletText(i.try_into().unwrap());
                }
            }

            // droids with flash are always flash-immune!
            // -> we don't get hurt by our own flashes!
            if InvincibleMode == 0
                && (*Druidmap.add(usize::try_from(Me.ty).unwrap())).flashimmune == 0
            {
                Me.energy -= (*Bulletmap.add(BulletKind::Flash as usize)).damage as f32;
            }
        }

        // --------------------
        // If its a "normal" Bullet, several checks have to be
        // done, one for collisions with background,
        // one for collision with influencer
        // some for collisions with enemys
        // and some for collisions with other bullets
        // and some for collisions with blast
        //
        _ => {
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

            for _ in 0..(num_check_steps as i32) {
                cur_bullet.pos.x += step.x;
                cur_bullet.pos.y += step.y;

                if IsPassable(
                    cur_bullet.pos.x,
                    cur_bullet.pos.y,
                    Direction::Center as c_int,
                ) != Direction::Center as c_int
                {
                    StartBlast(
                        cur_bullet.pos.x,
                        cur_bullet.pos.y,
                        Explosion::Bulletblast as c_int,
                    );
                    DeleteBullet(num);
                    return;
                }

                // check for collision with influencer
                if !cur_bullet.mine {
                    let xdist = Me.pos.x - cur_bullet.pos.x;
                    let ydist = Me.pos.y - cur_bullet.pos.y;
                    // FIXME: don't use DRUIDHITDIST2!!
                    if (xdist * xdist + ydist * ydist) < get_druid_hit_dist_squared() {
                        GotHitSound();

                        if InvincibleMode == 0 {
                            Me.energy -= (*Bulletmap.add(cur_bullet.ty.into())).damage as f32;
                        }

                        DeleteBullet(num);
                        return;
                    }
                }

                // check for collision with enemys
                for enemy in AllEnemys[..usize::try_from(NumEnemys).unwrap()].iter_mut() {
                    if enemy.status == Status::Out as c_int
                        || enemy.status == Status::Terminated as c_int
                        || enemy.levelnum != level
                    {
                        continue;
                    }

                    let xdist = cur_bullet.pos.x - enemy.pos.x;
                    let ydist = cur_bullet.pos.y - enemy.pos.y;

                    // FIXME
                    if (xdist * xdist + ydist * ydist) < get_druid_hit_dist_squared() {
                        // The enemy who was hit, loses some energy, depending on the bullet
                        enemy.energy -=
                            (*Bulletmap.add(cur_bullet.ty.try_into().unwrap())).damage as f32;

                        DeleteBullet(num);
                        GotHitSound();

                        if !cur_bullet.mine {
                            FBT_COUNTER += 1;
                        }
                        cur_bullet.ty = Status::Out as u8;
                        cur_bullet.mine = false;
                        return;
                    }
                }

                // check for collisions with other bullets
                for (i, bullet) in AllBullets[..MAXBULLETS].iter().enumerate() {
                    if Some(i) == usize::try_from(num).ok() {
                        continue;
                    } // never check for collision with youself.. ;)
                    if bullet.ty == Status::Out as u8 {
                        continue;
                    } // never check for collisions with dead bullets..
                    if bullet.ty == BulletKind::Flash as u8 {
                        continue;
                    } // never check for collisions with flashes bullets..

                    let xdist = bullet.pos.x - cur_bullet.pos.x;
                    let ydist = bullet.pos.y - cur_bullet.pos.y;
                    if xdist * xdist + ydist * ydist > BULLET_COLL_DIST2 {
                        continue;
                    }

                    // it seems like we have a collision of two bullets!
                    // both will be deleted and replaced by blasts..
                    info!("Bullet-Bullet-Collision detected...");

                    StartBlast(
                        cur_bullet.pos.x,
                        cur_bullet.pos.y,
                        Explosion::Druidblast as c_int,
                    );

                    DeleteBullet(num);
                    DeleteBullet(i.try_into().unwrap());
                }
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn DeleteBlast(num: c_int) {
    AllBlasts[usize::try_from(num).unwrap()].ty = Status::Out as c_int;
}

#[no_mangle]
pub unsafe extern "C" fn ExplodeBlasts() {
    AllBlasts[..MAXBLASTS]
        .iter_mut()
        .enumerate()
        .filter(|(_, blast)| blast.ty != Status::Out as c_int)
        .for_each(|(i, cur_blast)| {
            if cur_blast.ty == Explosion::Druidblast as c_int {
                CheckBlastCollisions(i.try_into().unwrap());
            }

            let blast_spec = &Blastmap[usize::try_from(cur_blast.ty).unwrap()];
            cur_blast.phase +=
                Frame_Time() * blast_spec.phases as f32 / blast_spec.total_animation_time;
            if cur_blast.phase.floor() as c_int >= blast_spec.phases {
                DeleteBlast(i.try_into().unwrap());
            }
        });
}
