use crate::{
    defs::{
        BulletKind, Direction, Explosion, BULLET_COLL_DIST2, COLLISION_STEPSIZE, FLASH_DURATION,
        MAXBLASTS, MAXBULLETS,
    },
    global::{Blast_Damage_Per_Second, Blast_Radius, Droid_Radius},
    map::{IsPassable, IsVisible},
    misc::Frame_Time,
    sound::{DruidBlastSound, GotHitSound, GotIntoBlastSound},
    structs::{Finepoint, Vect},
    text::{AddInfluBurntText, EnemyHitByBulletText},
    vars::{Blastmap, Bulletmap, Druidmap},
    AllBlasts, AllBullets, AllEnemys, CurLevel, InvincibleMode, LastGotIntoBlastSound, Me,
    NumEnemys, Status,
};

use log::info;
use sdl::video::ll::SDL_FreeSurface;
use std::{
    convert::{TryFrom, TryInto},
    os::raw::{c_float, c_int},
    ptr::null_mut,
};

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

#[no_mangle]
pub unsafe extern "C" fn CheckBlastCollisions(num: c_int) {
    let level = (*CurLevel).levelnum;
    let cur_blast = &mut AllBlasts[usize::try_from(num).unwrap()];

    /* check Blast-Bullet Collisions and kill hit Bullets */
    for (i, cur_bullet) in AllBullets[0..MAXBULLETS].iter().enumerate() {
        if cur_bullet.ty == Status::Out as u8 {
            continue;
        }

        let vdist = Vect {
            x: cur_bullet.pos.x - cur_blast.PX,
            y: cur_bullet.pos.y - cur_blast.PY,
        };
        let dist = (vdist.x * vdist.x + vdist.y * vdist.y).sqrt();
        if dist < Blast_Radius {
            StartBlast(
                cur_bullet.pos.x,
                cur_bullet.pos.y,
                Explosion::Bulletblast as c_int,
            );
            DeleteBullet(i.try_into().unwrap());
        }
    }

    /* Check Blast-Enemy Collisions and smash energy of hit enemy */
    for enemy in AllEnemys
        .iter_mut()
        .take(usize::try_from(NumEnemys).unwrap())
    {
        if enemy.status == Status::Out as c_int || enemy.levelnum != level {
            continue;
        }

        let vdist = Vect {
            x: enemy.pos.x - cur_blast.PX,
            y: enemy.pos.y - cur_blast.PY,
        };
        let dist = (vdist.x * vdist.x + vdist.y * vdist.y).sqrt();

        if dist < Blast_Radius + Droid_Radius {
            /* drag energy of enemy */
            enemy.energy -= Blast_Damage_Per_Second * Frame_Time();
        }

        if enemy.energy < 0. {
            enemy.energy = 0.;
        }
    }

    /* Check influence-Blast collisions */
    let vdist = Vect {
        x: Me.pos.x - cur_blast.PX,
        y: Me.pos.y - cur_blast.PY,
    };
    let dist = (vdist.x * vdist.x + vdist.y * vdist.y).sqrt();

    if Me.status != Status::Out as c_int && !cur_blast.mine && dist < Blast_Radius + Droid_Radius {
        if InvincibleMode == 0 {
            Me.energy -= Blast_Damage_Per_Second * Frame_Time();

            // So the influencer got some damage from the hot blast
            // Now most likely, he then will also say so :)
            if cur_blast.MessageWasDone == 0 {
                AddInfluBurntText();
                cur_blast.MessageWasDone = true.into();
            }
        }
        // In order to avoid a new sound EVERY frame we check for how long the previous blast
        // lies back in time.  LastBlastHit is a float, that counts SECONDS real-time !!
        if LastGotIntoBlastSound > 1.2 {
            GotIntoBlastSound();
            LastGotIntoBlastSound = 0.;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn StartBlast(x: c_float, y: c_float, mut ty: c_int) {
    let mut i = 0;
    while i < MAXBLASTS {
        if AllBlasts[i].ty == Status::Out as c_int {
            break;
        }

        i += 1;
    }

    if i >= MAXBLASTS {
        i = 0;
    }

    /* Get Pointer to it: more comfortable */
    let new_blast = &mut AllBlasts[i];

    if ty == Explosion::Rejectblast as c_int {
        new_blast.mine = true;
        ty = Explosion::Druidblast as c_int; // not really a different type, just avoid damaging influencer
    } else {
        new_blast.mine = false;
    }

    new_blast.PX = x;
    new_blast.PY = y;

    new_blast.ty = ty;
    new_blast.phase = 0.;

    new_blast.MessageWasDone = 0;

    if ty == Explosion::Druidblast as c_int {
        DruidBlastSound();
    }
}

/// delete bullet of given number, set it type=OUT, put it at x/y=-1/-1
/// and create a Bullet-blast if with_blast==TRUE
#[no_mangle]
pub unsafe extern "C" fn DeleteBullet(bullet_number: c_int) {
    let cur_bullet = &mut AllBullets[usize::try_from(bullet_number).unwrap()];

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
    if cur_bullet.Surfaces_were_generated != 0 {
        info!("DeleteBullet: freeing this bullets attached surfaces...");
        let bullet_spec = &*Bulletmap.add(cur_bullet.ty.into());
        for phase in 0..usize::try_from(bullet_spec.phases).unwrap() {
            SDL_FreeSurface(cur_bullet.SurfacePointer[phase]);
            cur_bullet.SurfacePointer[phase] = null_mut();
        }
        cur_bullet.Surfaces_were_generated = false.into();
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
#[no_mangle]
pub unsafe extern "C" fn MoveBullets() {
    for cur_bullet in &mut AllBullets[..MAXBULLETS] {
        if cur_bullet.ty == Status::Out as u8 {
            continue;
        }

        cur_bullet.prev_pos.x = cur_bullet.pos.x;
        cur_bullet.prev_pos.y = cur_bullet.pos.y;

        cur_bullet.pos.x += cur_bullet.speed.x * Frame_Time();
        cur_bullet.pos.y += cur_bullet.speed.y * Frame_Time();

        cur_bullet.time_in_frames += 1;
        cur_bullet.time_in_seconds += Frame_Time();
    }
}
