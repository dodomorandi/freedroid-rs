use crate::{
    b_font::{font_height, print_string_font, put_string_font},
    defs::{
        get_user_center, AssembleCombatWindowFlags, BulletKind, DisplayBannerFlags, Status,
        BLINKENERGY, CRY_SOUND_INTERVAL, FLASH_DURATION, LEFT_TEXT_LEN, MAXBLASTS, MAXBULLETS,
        RIGHT_TEXT_LEN, TRANSFER_SOUND_INTERVAL,
    },
    global::INFLUENCE_MODE_NAMES,
    graphics::{
        apply_filter, BANNER_IS_DESTROYED, BANNER_PIC, BUILD_BLOCK, DECAL_PICS,
        ENEMY_DIGIT_SURFACE_POINTER, ENEMY_SURFACE_POINTER, INFLUENCER_SURFACE_POINTER,
        INFLU_DIGIT_SURFACE_POINTER, MAP_BLOCK_SURFACE_POINTER, NE_SCREEN,
    },
    map::{get_map_brick, is_visible},
    structs::{Enemy, Finepoint, GrobPoint},
    vars::{
        BANNER_RECT, BLASTMAP, BULLETMAP, DRUIDMAP, FULL_USER_RECT, LEFT_INFO_RECT,
        RIGHT_INFO_RECT, USER_RECT,
    },
    Data, ALL_BLASTS, ALL_BULLETS, ALL_ENEMYS, CUR_LEVEL, DEATH_COUNT, FIRST_DIGIT_RECT, ME,
    NUMBER_OF_DROID_TYPES, SECOND_DIGIT_RECT, SHOW_ALL_DROIDS, SHOW_SCORE, THIRD_DIGIT_RECT,
};

use log::{info, trace};
use sdl::{
    sdl::Rect,
    video::ll::{
        SDL_Color, SDL_FillRect, SDL_MapRGB, SDL_SetClipRect, SDL_Surface, SDL_UpdateRect,
        SDL_UpperBlit,
    },
};
use std::{
    cell::{Cell, RefCell},
    convert::{TryFrom, TryInto},
    ffi::CStr,
    os::raw::{c_char, c_double, c_int},
    ptr::null_mut,
};

#[link(name = "SDL_gfx")]
extern "C" {
    pub fn rotozoomSurface(
        src: *mut SDL_Surface,
        angle: c_double,
        zoom: c_double,
        smooth: c_int,
    ) -> *mut SDL_Surface;
}

const BLINK_LEN: f32 = 1.0;

pub static BLACK: SDL_Color = SDL_Color {
    r: 0,
    g: 0,
    b: 0,
    unused: 0,
};

const FLASH_LIGHT: SDL_Color = SDL_Color {
    r: 11,
    g: 11,
    b: 11,
    unused: 0,
};
const FLASH_DARK: SDL_Color = SDL_Color {
    r: 230,
    g: 230,
    b: 230,
    unused: 0,
};

pub unsafe fn fill_rect(mut rect: Rect, color: SDL_Color) {
    let pixcolor = SDL_MapRGB((*NE_SCREEN).format, color.r, color.g, color.b);

    SDL_FillRect(NE_SCREEN, &mut rect, pixcolor);
}

impl Data {
    /// This function assembles the contents of the combat window
    /// in NE_SCREEN.
    ///
    /// Several FLAGS can be used to control its behaviour:
    ///
    /// (*) ONLY_SHOW_MAP = 0x01:  This flag indicates not do draw any
    ///     game elements but the map blocks
    ///
    /// (*) DO_SCREEgN_UPDATE = 0x02: This flag indicates for the function
    ///     to also cause an SDL_Update of the portion of the screen
    ///     that has been modified
    ///
    /// (*) SHOW_FULL_MAP = 0x04: show complete map, disregard visibility
    pub unsafe fn assemble_combat_picture(&mut self, mask: c_int) {
        thread_local! {
            static TIME_SINCE_LAST_FPS_UPDATE: Cell<f32> = Cell::new(10.);
            static FPS_DISPLAYED: Cell<i32>=Cell::new(1);
        }

        const UPDATE_FPS_HOW_OFTEN: f32 = 0.75;

        trace!("\nvoid Assemble_Combat_Picture(...): Real function call confirmed.");

        SDL_SetClipRect(NE_SCREEN, &USER_RECT);

        if self.global.game_config.all_map_visible == 0 {
            fill_rect(USER_RECT, BLACK);
        }

        let (upleft, downright) =
            if (mask & AssembleCombatWindowFlags::SHOW_FULL_MAP.bits() as i32) != 0 {
                let upleft = GrobPoint { x: -5, y: -5 };
                let downright = GrobPoint {
                    x: (*CUR_LEVEL).xlen as i8 + 5,
                    y: (*CUR_LEVEL).ylen as i8 + 5,
                };
                (upleft, downright)
            } else {
                let upleft = GrobPoint {
                    x: ME.pos.x as i8 - 6,
                    y: ME.pos.y as i8 - 5,
                };
                let downright = GrobPoint {
                    x: ME.pos.x as i8 + 7,
                    y: ME.pos.y as i8 + 5,
                };
                (upleft, downright)
            };

        let mut pos = Finepoint::default();
        let mut vect = Finepoint::default();
        let mut len = -1f32;
        let mut map_brick = 0;
        let mut target_rectangle = Rect::new(0, 0, 0, 0);
        (upleft.y..downright.y)
            .flat_map(|line| (upleft.x..downright.x).map(move |col| (line, col)))
            .for_each(|(line, col)| {
                if self.global.game_config.all_map_visible == 0
                    && ((mask & AssembleCombatWindowFlags::SHOW_FULL_MAP.bits() as i32) == 0x0)
                {
                    pos.x = col.into();
                    pos.y = line.into();
                    vect.x = ME.pos.x - pos.x;
                    vect.y = ME.pos.y - pos.y;
                    len = (vect.x * vect.x + vect.y * vect.y).sqrt() + 0.01;
                    vect.x /= len;
                    vect.y /= len;
                    if len > 0.5 {
                        pos.x += vect.x;
                        pos.y += vect.y;
                    }
                    if is_visible(&pos) == 0 {
                        return;
                    }
                }

                map_brick = get_map_brick(&*CUR_LEVEL, col.into(), line.into());
                let user_center = get_user_center();
                target_rectangle.x = user_center.x
                    + ((-ME.pos.x + 1.0 * f32::from(col) - 0.5) * f32::from(self.vars.block_rect.w))
                        .round() as i16;
                target_rectangle.y = user_center.y
                    + ((-ME.pos.y + 1.0 * f32::from(line) - 0.5)
                        * f32::from(self.vars.block_rect.h))
                    .round() as i16;
                SDL_UpperBlit(
                    MAP_BLOCK_SURFACE_POINTER[usize::try_from((*CUR_LEVEL).color).unwrap()]
                        [usize::from(map_brick)],
                    null_mut(),
                    NE_SCREEN,
                    &mut target_rectangle,
                );
            });

        // if we don't use Fullscreen mode, we have to clear the text-background manually
        // for the info-line text:

        let mut text_rect = Rect::new(
            FULL_USER_RECT.x,
            (i32::from(FULL_USER_RECT.y) + i32::from(FULL_USER_RECT.h)
                - font_height(&*self.global.font0_b_font))
            .try_into()
            .unwrap(),
            FULL_USER_RECT.w,
            font_height(&*self.global.font0_b_font).try_into().unwrap(),
        );
        SDL_SetClipRect(NE_SCREEN, &text_rect);
        if self.global.game_config.full_user_rect == 0 {
            SDL_FillRect(NE_SCREEN, &mut text_rect, 0);
        }

        if self.global.game_config.draw_position != 0 {
            print_string_font(
                NE_SCREEN,
                self.global.font0_b_font,
                (FULL_USER_RECT.x + (FULL_USER_RECT.w / 6) as i16).into(),
                i32::from(FULL_USER_RECT.y) + i32::from(FULL_USER_RECT.h)
                    - font_height(&*self.global.font0_b_font),
                format_args!(
                    "GPS: X={:.0} Y={:.0} Lev={}",
                    ME.pos.x.round(),
                    ME.pos.y.round(),
                    (*CUR_LEVEL).levelnum,
                ),
            );
        }

        if mask & AssembleCombatWindowFlags::ONLY_SHOW_MAP.bits() as i32 == 0 {
            if self.global.game_config.draw_framerate != 0 {
                TIME_SINCE_LAST_FPS_UPDATE.with(|time_cell| {
                    let mut time = time_cell.get();
                    time += self.frame_time();

                    if time > UPDATE_FPS_HOW_OFTEN {
                        FPS_DISPLAYED.with(|fps_displayed| {
                            fps_displayed.set((1.0 / self.frame_time()) as i32);
                        });
                        time_cell.set(0.);
                    } else {
                        time_cell.set(time);
                    }
                });

                FPS_DISPLAYED.with(|fps_displayed| {
                    print_string_font(
                        NE_SCREEN,
                        self.global.font0_b_font,
                        FULL_USER_RECT.x.into(),
                        FULL_USER_RECT.y as i32 + FULL_USER_RECT.h as i32
                            - font_height(&*self.global.font0_b_font) as i32,
                        format_args!("FPS: {} ", fps_displayed.get()),
                    );
                });
            }

            if self.global.game_config.draw_energy != 0 {
                print_string_font(
                    NE_SCREEN,
                    self.global.font0_b_font,
                    i32::from(FULL_USER_RECT.x) + i32::from(FULL_USER_RECT.w) / 2,
                    i32::from(FULL_USER_RECT.y) + i32::from(FULL_USER_RECT.h)
                        - font_height(&*self.global.font0_b_font),
                    format_args!("Energy: {:.0}", ME.energy),
                );
            }
            if self.global.game_config.draw_death_count != 0 {
                print_string_font(
                    NE_SCREEN,
                    self.global.font0_b_font,
                    i32::from(FULL_USER_RECT.x) + 2 * i32::from(FULL_USER_RECT.w) / 3,
                    i32::from(FULL_USER_RECT.y) + i32::from(FULL_USER_RECT.h)
                        - font_height(&*self.global.font0_b_font),
                    format_args!("Deathcount: {:.0}", DEATH_COUNT,),
                );
            }

            SDL_SetClipRect(NE_SCREEN, &USER_RECT);

            // make sure Ashes are displayed _before_ droids, so that they are _under_ them!
            for enemy in &mut ALL_ENEMYS {
                if (enemy.status == Status::Terminated as i32)
                    && (enemy.levelnum == (*CUR_LEVEL).levelnum)
                    && is_visible(&enemy.pos) != 0
                {
                    self.put_ashes(enemy.pos.x, enemy.pos.y);
                }
            }

            ALL_ENEMYS
                .iter()
                .enumerate()
                .filter(|(_, enemy)| {
                    !((enemy.levelnum != (*CUR_LEVEL).levelnum)
                        || (enemy.status == Status::Out as i32)
                        || (enemy.status == Status::Terminated as i32))
                })
                .for_each(|(index, _)| self.put_enemy(index as c_int, -1, -1));

            if ME.energy > 0. {
                self.put_influence(-1, -1);
            }

            ALL_BULLETS
                .iter()
                .take(MAXBULLETS)
                .enumerate()
                .filter(|(_, bullet)| bullet.ty != Status::Out as u8)
                .for_each(|(index, _)| self.put_bullet(index as i32));

            ALL_BLASTS
                .iter()
                .take(MAXBLASTS)
                .enumerate()
                .filter(|(_, blast)| blast.ty != Status::Out as i32)
                .for_each(|(index, _)| self.put_blast(index as i32));
        }

        // At this point we are done with the drawing procedure
        // and all that remains to be done is updating the screen.

        if mask & AssembleCombatWindowFlags::DO_SCREEN_UPDATE.bits() as i32 != 0 {
            SDL_UpdateRect(
                NE_SCREEN,
                USER_RECT.x.into(),
                USER_RECT.y.into(),
                USER_RECT.w.into(),
                USER_RECT.h.into(),
            );
            SDL_UpdateRect(
                NE_SCREEN,
                text_rect.x.into(),
                text_rect.y.into(),
                text_rect.w.into(),
                text_rect.h.into(),
            );
        }

        SDL_SetClipRect(NE_SCREEN, null_mut());
    }

    /// put some ashes at (x,y)
    pub unsafe fn put_ashes(&self, x: f32, y: f32) {
        if self.global.game_config.show_decals == 0 {
            return;
        }

        let user_center = get_user_center();
        let mut dst = Rect::new(
            (f32::from(user_center.x) + (-ME.pos.x + x) * f32::from(self.vars.block_rect.w)
                - f32::from(self.vars.block_rect.w / 2)) as i16,
            (f32::from(user_center.y) + (-ME.pos.y + y) * f32::from(self.vars.block_rect.h)
                - f32::from(self.vars.block_rect.h / 2)) as i16,
            0,
            0,
        );
        SDL_UpperBlit(DECAL_PICS[0], null_mut(), NE_SCREEN, &mut dst);
    }

    pub unsafe fn put_enemy(&mut self, enemy_index: c_int, x: c_int, y: c_int) {
        let droid: &mut Enemy = &mut ALL_ENEMYS[usize::try_from(enemy_index).unwrap()];
        let ty = droid.ty;
        let phase = droid.phase;
        let name = &mut (*DRUIDMAP.offset(ty.try_into().unwrap())).druidname;

        if (droid.status == Status::Terminated as i32)
            || (droid.status == Status::Out as i32)
            || (droid.levelnum != (*CUR_LEVEL).levelnum)
        {
            return;
        }

        // if the enemy is out of sight, we need not do anything more here
        if SHOW_ALL_DROIDS == 0 && is_visible(&droid.pos) == 0 {
            trace!("ONSCREEN=FALSE --> usual end of function reached.");
            return;
        }

        // We check for incorrect droid types, which sometimes might occor, especially after
        // heavy editing of the crew initialisation functions ;)
        if droid.ty >= NUMBER_OF_DROID_TYPES {
            panic!("nonexistant droid-type encountered: {}", droid.ty);
        }

        //--------------------
        // First blit just the enemy hat and shoes.
        SDL_UpperBlit(
            ENEMY_SURFACE_POINTER[phase as usize],
            null_mut(),
            BUILD_BLOCK,
            null_mut(),
        );

        //--------------------
        // Now the numbers should be blittet.
        let mut dst = FIRST_DIGIT_RECT;
        SDL_UpperBlit(
            ENEMY_DIGIT_SURFACE_POINTER[usize::try_from(name[0] - b'1' as i8 + 1).unwrap()],
            null_mut(),
            BUILD_BLOCK,
            &mut dst,
        );

        dst = SECOND_DIGIT_RECT;
        SDL_UpperBlit(
            ENEMY_DIGIT_SURFACE_POINTER[usize::try_from(name[1] - b'1' as i8 + 1).unwrap()],
            null_mut(),
            BUILD_BLOCK,
            &mut dst,
        );

        dst = THIRD_DIGIT_RECT;
        SDL_UpperBlit(
            ENEMY_DIGIT_SURFACE_POINTER[usize::try_from(name[2] - b'1' as i8 + 1).unwrap()],
            null_mut(),
            BUILD_BLOCK,
            &mut dst,
        );

        // now blit the whole construction to screen:
        if x == -1 {
            let user_center = get_user_center();
            dst.x = (f32::from(user_center.x)
                + (droid.pos.x - ME.pos.x) * f32::from(self.vars.block_rect.w)
                - f32::from(self.vars.block_rect.w / 2)) as i16;
            dst.y = (f32::from(user_center.y)
                + (droid.pos.y - ME.pos.y) * f32::from(self.vars.block_rect.h)
                - f32::from(self.vars.block_rect.h / 2)) as i16;
        } else {
            dst.x = x.try_into().unwrap();
            dst.y = y.try_into().unwrap();
        }
        SDL_UpperBlit(BUILD_BLOCK, null_mut(), NE_SCREEN, &mut dst);

        //--------------------
        // At this point we can assume, that the enemys has been blittet to the
        // screen, whether it's a friendly enemy or not.
        //
        // So now we can add some text the enemys says.  That might be fun.
        //
        if x == -1
            && droid.text_visible_time < self.global.game_config.wanted_text_visible_time
            && self.global.game_config.droid_talk != 0
        {
            put_string_font(
                NE_SCREEN,
                self.global.font0_b_font,
                (f32::from(USER_RECT.x)
                    + f32::from(USER_RECT.w / 2)
                    + f32::from(self.vars.block_rect.w / 3)
                    + (droid.pos.x - ME.pos.x) * f32::from(self.vars.block_rect.w))
                    as i32,
                (f32::from(USER_RECT.y) + f32::from(USER_RECT.h / 2)
                    - f32::from(self.vars.block_rect.h / 2)
                    + (droid.pos.y - ME.pos.y) * f32::from(self.vars.block_rect.h))
                    as i32,
                CStr::from_ptr(droid.text_to_be_displayed).to_bytes(),
            );
        }

        trace!("ENEMY HAS BEEN PUT --> usual end of function reached.");
    }

    /// This function draws the influencer to the screen, either
    /// to the center of the combat window if (-1,-1) was specified, or
    /// to the specified coordinates anywhere on the screen, useful e.g.
    /// for using the influencer as a cursor in the menus.
    pub unsafe fn put_influence(&mut self, x: c_int, y: c_int) {
        let text_rect = Rect::new(
            USER_RECT.x + (USER_RECT.w / 2) as i16 + (self.vars.block_rect.w / 3) as i16,
            USER_RECT.y + (USER_RECT.h / 2) as i16 - (self.vars.block_rect.h / 2) as i16,
            USER_RECT.w / 2 - self.vars.block_rect.w / 3,
            USER_RECT.h / 2,
        );

        trace!("PutInfluence real function call confirmed.");

        // Now we draw the hat and shoes of the influencer
        SDL_UpperBlit(
            INFLUENCER_SURFACE_POINTER[(ME.phase).floor() as usize],
            null_mut(),
            BUILD_BLOCK,
            null_mut(),
        );

        // Now we draw the first digit of the influencers current number.
        let mut dst = FIRST_DIGIT_RECT;
        SDL_UpperBlit(
            INFLU_DIGIT_SURFACE_POINTER[usize::try_from(
                (*DRUIDMAP.offset(ME.ty.try_into().unwrap())).druidname[0] - b'1' as i8 + 1,
            )
            .unwrap()],
            null_mut(),
            BUILD_BLOCK,
            &mut dst,
        );

        // Now we draw the second digit of the influencers current number.
        dst = SECOND_DIGIT_RECT;
        SDL_UpperBlit(
            INFLU_DIGIT_SURFACE_POINTER[usize::try_from(
                (*DRUIDMAP.offset(ME.ty.try_into().unwrap())).druidname[1] - b'1' as i8 + 1,
            )
            .unwrap()],
            null_mut(),
            BUILD_BLOCK,
            &mut dst,
        );

        // Now we draw the third digit of the influencers current number.
        dst = THIRD_DIGIT_RECT;
        SDL_UpperBlit(
            INFLU_DIGIT_SURFACE_POINTER[usize::try_from(
                (*DRUIDMAP.offset(ME.ty.try_into().unwrap())).druidname[2] - b'1' as i8 + 1,
            )
            .unwrap()],
            null_mut(),
            BUILD_BLOCK,
            &mut dst,
        );

        if ME.energy * 100. / (*DRUIDMAP.offset(ME.ty.try_into().unwrap())).maxenergy <= BLINKENERGY
            && x == -1
        {
            // In case of low energy, do the fading effect...
            let rest = ME.timer % BLINK_LEN; // period of fading is given by BLINK_LEN
            let filt = if rest < BLINK_LEN / 2. {
                0.40 + (1.0 - 2.0 * rest / BLINK_LEN) * 0.60 // decrease white->grey
            } else {
                0.40 + (2.0 * rest / BLINK_LEN - 1.0) * 0.60 // increase back to white
            };

            apply_filter(&mut *BUILD_BLOCK, filt, filt, filt);

            // ... and also maybe start a new cry-sound

            if ME.last_crysound_time > CRY_SOUND_INTERVAL {
                ME.last_crysound_time = 0.;
                self.cry_sound();
            }
        }

        //--------------------
        // In case of transfer mode, we produce the transfer mode sound
        // but of course only in some periodic intervall...

        if ME.status == Status::Transfermode as i32 && x == -1 {
            apply_filter(&mut *BUILD_BLOCK, 1.0, 0.0, 0.0);

            if ME.last_transfer_sound_time > TRANSFER_SOUND_INTERVAL {
                ME.last_transfer_sound_time = 0.;
                self.transfer_sound();
            }
        }

        if x == -1 {
            let user_center = get_user_center();
            dst.x = user_center.x - (self.vars.block_rect.w / 2) as i16;
            dst.y = user_center.y - (self.vars.block_rect.h / 2) as i16;
        } else {
            dst.x = x.try_into().unwrap();
            dst.y = y.try_into().unwrap();
        }

        SDL_UpperBlit(BUILD_BLOCK, null_mut(), NE_SCREEN, &mut dst);

        //--------------------
        // Maybe the influencer has something to say :)
        // so let him say it..
        //
        if x == -1
            && ME.text_visible_time < self.global.game_config.wanted_text_visible_time
            && self.global.game_config.droid_talk != 0
        {
            self.b_font.current_font = self.global.font0_b_font;
            self.display_text(
                ME.text_to_be_displayed,
                i32::from(USER_RECT.x)
                    + i32::from(USER_RECT.w / 2)
                    + i32::from(self.vars.block_rect.w / 3),
                i32::from(USER_RECT.y) + i32::from(USER_RECT.h / 2)
                    - i32::from(self.vars.block_rect.h / 2),
                &text_rect,
            );
        }

        trace!("PutInfluence: end of function reached.");
    }

    /// PutBullet: draws a Bullet into the combat window.  The only
    /// parameter given is the number of the bullet in the AllBullets
    /// array. Everything else is computed in here.
    pub unsafe fn put_bullet(&self, bullet_number: c_int) {
        let cur_bullet = &mut ALL_BULLETS[usize::try_from(bullet_number).unwrap()];

        trace!("PutBullet: real function call confirmed.");

        //--------------------
        // in case our bullet is of the type "FLASH", we only
        // draw a big white or black rectangle right over the
        // combat window, white for even frames and black for
        // odd frames.
        if cur_bullet.ty == BulletKind::Flash as u8 {
            // Now the whole window will be filled with either white
            // or black each frame until the flash is over.  (Flash
            // deletion after some time is done in CheckBulletCollisions.)
            if cur_bullet.time_in_seconds <= FLASH_DURATION / 4. {
                fill_rect(USER_RECT, FLASH_LIGHT);
            } else if cur_bullet.time_in_seconds <= FLASH_DURATION / 2. {
                fill_rect(USER_RECT, FLASH_DARK);
            } else if cur_bullet.time_in_seconds <= 3. * FLASH_DURATION / 4. {
                fill_rect(USER_RECT, FLASH_LIGHT);
            } else if cur_bullet.time_in_seconds <= FLASH_DURATION {
                fill_rect(USER_RECT, FLASH_DARK);
            }

            return;
        }

        let bullet = &*BULLETMAP.offset(cur_bullet.ty.try_into().unwrap());
        let mut phase_of_bullet =
            (cur_bullet.time_in_seconds * bullet.phase_changes_per_second) as usize;

        phase_of_bullet %= usize::try_from(bullet.phases).unwrap();

        // DebugPrintf( 0 , "\nPhaseOfBullet: %d.", PhaseOfBullet );

        //--------------------
        // Maybe it's the first time this bullet is displayed.  But then, the images
        // of the rotated bullet in all phases are not yet attached to the bullet.
        // Then, we'll have to generate these
        //
        //if ( cur_bullet.time_in_frames == 1 )
        if cur_bullet.surfaces_were_generated == 0 {
            for i in 0..usize::try_from(bullet.phases).unwrap() {
                cur_bullet.surface_pointer[i] = rotozoomSurface(
                    bullet.surface_pointer[i],
                    cur_bullet.angle.into(),
                    1.0,
                    false.into(),
                );
            }
            info!(
                "This was the first time for this bullet, so images were generated... angle={}",
                cur_bullet.angle
            );
            cur_bullet.surfaces_were_generated = true.into();
        }

        // WARNING!!! PAY ATTENTION HERE!! After the rotozoom was applied to the image, it is NO
        // LONGER of dimension Block_Rect.w times Block_Rect.h, but of the dimesions of the smallest
        // rectangle containing the full rotated Block_Rect.h x Block_Rect.w rectangle!!!
        // This has to be taken into account when calculating the target position for the
        // blit of these surfaces!!!!
        let user_center = get_user_center();
        let mut dst = Rect::new(
            (f32::from(user_center.x)
                - (ME.pos.x - cur_bullet.pos.x) * f32::from(self.vars.block_rect.w)
                - ((*cur_bullet.surface_pointer[phase_of_bullet]).w / 2) as f32) as i16,
            (f32::from(user_center.y)
                - (ME.pos.y - cur_bullet.pos.y) * f32::from(self.vars.block_rect.w)
                - ((*cur_bullet.surface_pointer[phase_of_bullet]).h / 2) as f32) as i16,
            0,
            0,
        );

        SDL_UpperBlit(
            cur_bullet.surface_pointer[phase_of_bullet],
            null_mut(),
            NE_SCREEN,
            &mut dst,
        );

        trace!("PutBullet: end of function reached.");
    }

    pub unsafe fn put_blast(&self, blast_number: c_int) {
        trace!("PutBlast: real function call confirmed.");

        let cur_blast = &mut ALL_BLASTS[usize::try_from(blast_number).unwrap()];

        // If the blast is already long deat, we need not do anything else here
        if cur_blast.ty == Status::Out as i32 {
            return;
        }

        let user_center = get_user_center();
        let mut dst = Rect::new(
            (f32::from(user_center.x)
                - (ME.pos.x - cur_blast.px) * f32::from(self.vars.block_rect.w)
                - f32::from(self.vars.block_rect.w / 2)) as i16,
            (f32::from(user_center.y)
                - (ME.pos.y - cur_blast.py) * f32::from(self.vars.block_rect.h)
                - f32::from(self.vars.block_rect.h / 2)) as i16,
            0,
            0,
        );
        SDL_UpperBlit(
            BLASTMAP[usize::try_from(cur_blast.ty).unwrap()].surface_pointer
                [(cur_blast.phase).floor() as usize],
            null_mut(),
            NE_SCREEN,
            &mut dst,
        );
        trace!("PutBlast: end of function reached.");
    }

    /// This function updates the top status bar.
    ///
    /// To save framerate on slow machines however it will only work
    /// if it thinks that work needs to be done.
    /// You can however force update if you say so with a flag.
    ///
    /// BANNER_FORCE_UPDATE=1: Forces the redrawing of the title bar
    ///
    /// BANNER_DONT_TOUCH_TEXT=2: Prevents DisplayBanner from touching the
    /// text.
    ///
    /// BANNER_NO_SDL_UPDATE=4: Prevents any SDL_Update calls.
    pub unsafe fn display_banner(
        &mut self,
        mut left: *const c_char,
        mut right: *const c_char,
        flags: c_int,
    ) {
        use std::io::Write;

        let mut dummy: [u8; 80];

        thread_local! {
            static PREVIOUS_LEFT_BOX: RefCell<[u8; LEFT_TEXT_LEN + 10]>={
              let mut data = [0u8; LEFT_TEXT_LEN + 10];
              data[..6].copy_from_slice(b"NOUGHT");
              RefCell::new(data)
            };
            static PREVIOUS_RIGHT_BOX: RefCell<[u8; RIGHT_TEXT_LEN + 10]>= {
              let mut data = [0u8; RIGHT_TEXT_LEN + 10];
              data[..6].copy_from_slice(b"NOUGHT");
              RefCell::new(data)
            };
        }

        // --------------------
        // At first the text is prepared.  This can't hurt.
        // we will decide whether to display it or not later...
        //

        if left.is_null() {
            /* Left-DEFAULT: Mode */
            left = INFLUENCE_MODE_NAMES[ME.status as usize].as_ptr();
        }

        if right.is_null()
        /* Right-DEFAULT: Score */
        {
            dummy = [0u8; 80];
            write!(dummy.as_mut(), "{}", SHOW_SCORE).unwrap();
            right = dummy.as_mut_ptr() as *mut c_char;
        }

        // Now fill in the text
        let left = CStr::from_ptr(left);
        let left_len = left.to_bytes().len();
        if left_len > LEFT_TEXT_LEN {
            panic!(
                "String {} too long for Left Infoline!!",
                left.to_string_lossy()
            );
        }
        let right = CStr::from_ptr(right);
        let right_len = right.to_bytes().len();
        if right_len > RIGHT_TEXT_LEN {
            panic!(
                "String {} too long for Right Infoline!!",
                right.to_string_lossy()
            );
        }

        /* Now prepare the left/right text-boxes */
        let mut left_box = [b' '; LEFT_TEXT_LEN + 10];
        let mut right_box = [b' '; RIGHT_TEXT_LEN + 10];

        left_box[..left_len].copy_from_slice(&left.to_bytes()[..left_len]);
        right_box[..right_len].copy_from_slice(&right.to_bytes()[..right_len]);

        left_box[LEFT_TEXT_LEN] = b'\0'; /* that's right, we want padding! */
        right_box[RIGHT_TEXT_LEN] = b'\0';

        // --------------------
        // No we see if the screen need an update...

        let screen_needs_update = BANNER_IS_DESTROYED != 0
            || (flags & i32::from(DisplayBannerFlags::FORCE_UPDATE.bits())) != 0
            || PREVIOUS_LEFT_BOX
                .with(|previous_left_box| left_box.as_ref() != previous_left_box.borrow().as_ref())
            || PREVIOUS_RIGHT_BOX.with(|previous_right_box| {
                right_box.as_ref() != previous_right_box.borrow().as_ref()
            });
        if screen_needs_update {
            // Redraw the whole background of the top status bar
            let mut dst = Rect::new(0, 0, 0, 0);
            SDL_SetClipRect(NE_SCREEN, null_mut()); // this unsets the clipping rectangle
            SDL_UpperBlit(BANNER_PIC, null_mut(), NE_SCREEN, &mut dst);

            // Now the text should be ready and its
            // time to display it...
            let previous_left_check = PREVIOUS_LEFT_BOX
                .with(|previous_left_box| left_box.as_ref() != previous_left_box.borrow().as_ref());
            let previous_right_check = PREVIOUS_RIGHT_BOX.with(|previous_right_box| {
                right_box.as_ref() != previous_right_box.borrow().as_ref()
            });
            if previous_left_check
                || previous_right_check
                || (flags & i32::from(DisplayBannerFlags::FORCE_UPDATE.bits())) != 0
            {
                dst.x = LEFT_INFO_RECT.x;
                dst.y = LEFT_INFO_RECT.y
                    - i16::try_from(font_height(&*self.global.para_b_font)).unwrap();
                print_string_font(
                    NE_SCREEN,
                    self.global.para_b_font,
                    dst.x.into(),
                    dst.y.into(),
                    format_args!(
                        "{}",
                        CStr::from_ptr(left_box.as_ptr() as *const c_char)
                            .to_str()
                            .unwrap()
                    ),
                );
                let left_box_len = left_box.iter().position(|&c| c == 0).unwrap();
                PREVIOUS_LEFT_BOX.with(|previous_left_box| {
                    let mut previous_left_box = previous_left_box.borrow_mut();
                    previous_left_box[..left_box_len].copy_from_slice(&left_box[..left_box_len]);
                    previous_left_box[left_box_len] = b'\0';
                });

                dst.x = RIGHT_INFO_RECT.x;
                dst.y = RIGHT_INFO_RECT.y
                    - i16::try_from(font_height(&*self.global.para_b_font)).unwrap();
                print_string_font(
                    NE_SCREEN,
                    self.global.para_b_font,
                    dst.x.into(),
                    dst.y.into(),
                    format_args!(
                        "{}",
                        CStr::from_ptr(right_box.as_ptr() as *const c_char)
                            .to_str()
                            .unwrap()
                    ),
                );
                let right_box_len = right_box.iter().position(|&c| c == 0).unwrap();
                PREVIOUS_RIGHT_BOX.with(|previous_right_box| {
                    let mut previous_right_box = previous_right_box.borrow_mut();
                    previous_right_box[..right_box_len]
                        .copy_from_slice(&right_box[..right_box_len]);
                    previous_right_box[right_box_len] = b'\0';
                });
            }

            // finally update the whole top status box
            if (flags & i32::from(DisplayBannerFlags::NO_SDL_UPDATE.bits())) == 0 {
                SDL_UpdateRect(NE_SCREEN, 0, 0, BANNER_RECT.w.into(), BANNER_RECT.h.into());
            }

            BANNER_IS_DESTROYED = false.into();
        }
    }
}
