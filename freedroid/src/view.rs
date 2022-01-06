use crate::{
    b_font::{font_height, print_string_font, put_string_font},
    defs::{
        AssembleCombatWindowFlags, BulletKind, DisplayBannerFlags, Status, BLINKENERGY,
        CRY_SOUND_INTERVAL, FLASH_DURATION, LEFT_TEXT_LEN, MAXBLASTS, MAXBULLETS, RIGHT_TEXT_LEN,
        TRANSFER_SOUND_INTERVAL,
    },
    global::INFLUENCE_MODE_NAMES,
    graphics::{apply_filter, Graphics},
    map::get_map_brick,
    structs::{Blast, Finepoint, GrobPoint},
    vars::Vars,
    Data, Main,
};

use log::{info, trace};
use sdl::Rect;
use sdl_sys::{SDL_Color, SDL_FillRect, SDL_MapRGB, SDL_UpdateRect};
use std::{
    cell::{Cell, RefCell},
    ffi::CStr,
    os::raw::{c_char, c_int},
};

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

impl Data<'_> {
    pub unsafe fn fill_rect(&mut self, mut rect: Rect, color: SDL_Color) {
        let pixcolor = SDL_MapRGB(
            self.graphics.ne_screen.as_ref().unwrap().format().as_ptr(),
            color.r,
            color.g,
            color.b,
        );

        SDL_FillRect(
            self.graphics.ne_screen.as_mut().unwrap().as_mut_ptr(),
            rect.as_mut(),
            pixcolor,
        );
    }

    /// This function assembles the contents of the combat window
    /// in self.graphics.ne_screen.
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

        self.graphics
            .ne_screen
            .as_mut()
            .unwrap()
            .set_clip_rect(&self.vars.user_rect);
        if self.global.game_config.all_map_visible == 0 {
            self.fill_rect(self.vars.user_rect, BLACK);
        }

        let (upleft, downright) =
            if (mask & AssembleCombatWindowFlags::SHOW_FULL_MAP.bits() as i32) != 0 {
                let upleft = GrobPoint { x: -5, y: -5 };
                let downright = GrobPoint {
                    x: (*self.main.cur_level).xlen as i8 + 5,
                    y: (*self.main.cur_level).ylen as i8 + 5,
                };
                (upleft, downright)
            } else {
                let upleft = GrobPoint {
                    x: self.vars.me.pos.x as i8 - 6,
                    y: self.vars.me.pos.y as i8 - 5,
                };
                let downright = GrobPoint {
                    x: self.vars.me.pos.x as i8 + 7,
                    y: self.vars.me.pos.y as i8 + 5,
                };
                (upleft, downright)
            };

        let mut pos = Finepoint::default();
        let mut vect = Finepoint::default();
        let mut len = -1f32;
        let mut map_brick = 0;
        let mut target_rectangle = Rect::default();
        (upleft.y..downright.y)
            .flat_map(|line| (upleft.x..downright.x).map(move |col| (line, col)))
            .for_each(|(line, col)| {
                if self.global.game_config.all_map_visible == 0
                    && ((mask & AssembleCombatWindowFlags::SHOW_FULL_MAP.bits() as i32) == 0x0)
                {
                    pos.x = col.into();
                    pos.y = line.into();
                    vect.x = self.vars.me.pos.x - pos.x;
                    vect.y = self.vars.me.pos.y - pos.y;
                    len = (vect.x * vect.x + vect.y * vect.y).sqrt() + 0.01;
                    vect.x /= len;
                    vect.y /= len;
                    if len > 0.5 {
                        pos.x += vect.x;
                        pos.y += vect.y;
                    }
                    if self.is_visible(&pos) == 0 {
                        return;
                    }
                }

                map_brick = get_map_brick(&*self.main.cur_level, col.into(), line.into());
                let user_center = self.vars.get_user_center();
                target_rectangle.set_x(
                    user_center.x()
                        + ((-self.vars.me.pos.x + 1.0 * f32::from(col) - 0.5)
                            * f32::from(self.vars.block_rect.width()))
                        .round() as i16,
                );
                target_rectangle.set_y(
                    user_center.y()
                        + ((-self.vars.me.pos.y + 1.0 * f32::from(line) - 0.5)
                            * f32::from(self.vars.block_rect.height()))
                        .round() as i16,
                );

                let mut surface = self.graphics.map_block_surface_pointer
                    [usize::try_from((*self.main.cur_level).color).unwrap()]
                    [usize::from(map_brick)]
                .as_mut()
                .unwrap()
                .borrow_mut();
                surface.blit_to(
                    self.graphics.ne_screen.as_mut().unwrap(),
                    &mut target_rectangle,
                );
            });

        // if we don't use Fullscreen mode, we have to clear the text-background manually
        // for the info-line text:

        let mut text_rect = Rect::new(
            self.vars.full_user_rect.x(),
            (i32::from(self.vars.full_user_rect.y())
                + i32::from(self.vars.full_user_rect.height())
                - font_height(&*self.global.font0_b_font))
            .try_into()
            .unwrap(),
            self.vars.full_user_rect.width(),
            font_height(&*self.global.font0_b_font).try_into().unwrap(),
        );
        self.graphics
            .ne_screen
            .as_mut()
            .unwrap()
            .set_clip_rect(&text_rect);
        if self.global.game_config.full_user_rect == 0 {
            SDL_FillRect(
                self.graphics.ne_screen.as_mut().unwrap().as_mut_ptr(),
                text_rect.as_mut(),
                0,
            );
        }

        if self.global.game_config.draw_position != 0 {
            print_string_font(
                self.graphics.ne_screen.as_mut().unwrap(),
                self.global.font0_b_font,
                (self.vars.full_user_rect.x() + (self.vars.full_user_rect.width() / 6) as i16)
                    .into(),
                i32::from(self.vars.full_user_rect.y())
                    + i32::from(self.vars.full_user_rect.height())
                    - font_height(&*self.global.font0_b_font),
                format_args!(
                    "GPS: X={:.0} Y={:.0} Lev={}",
                    self.vars.me.pos.x.round(),
                    self.vars.me.pos.y.round(),
                    (*self.main.cur_level).levelnum,
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
                        self.graphics.ne_screen.as_mut().unwrap(),
                        self.global.font0_b_font,
                        self.vars.full_user_rect.x().into(),
                        self.vars.full_user_rect.y() as i32
                            + self.vars.full_user_rect.height() as i32
                            - font_height(&*self.global.font0_b_font) as i32,
                        format_args!("FPS: {} ", fps_displayed.get()),
                    );
                });
            }

            if self.global.game_config.draw_energy != 0 {
                print_string_font(
                    self.graphics.ne_screen.as_mut().unwrap(),
                    self.global.font0_b_font,
                    i32::from(self.vars.full_user_rect.x())
                        + i32::from(self.vars.full_user_rect.width()) / 2,
                    i32::from(self.vars.full_user_rect.y())
                        + i32::from(self.vars.full_user_rect.height())
                        - font_height(&*self.global.font0_b_font),
                    format_args!("Energy: {:.0}", self.vars.me.energy),
                );
            }
            if self.global.game_config.draw_death_count != 0 {
                print_string_font(
                    self.graphics.ne_screen.as_mut().unwrap(),
                    self.global.font0_b_font,
                    i32::from(self.vars.full_user_rect.x())
                        + 2 * i32::from(self.vars.full_user_rect.width()) / 3,
                    i32::from(self.vars.full_user_rect.y())
                        + i32::from(self.vars.full_user_rect.height())
                        - font_height(&*self.global.font0_b_font),
                    format_args!("Deathcount: {:.0}", self.main.death_count,),
                );
            }

            self.graphics
                .ne_screen
                .as_mut()
                .unwrap()
                .set_clip_rect(&self.vars.user_rect);

            // make sure Ashes are displayed _before_ droids, so that they are _under_ them!
            for enemy_index in 0..usize::try_from(self.main.num_enemys).unwrap() {
                let enemy = &self.main.all_enemys[enemy_index];
                if (enemy.status == Status::Terminated as i32)
                    && (enemy.levelnum == (*self.main.cur_level).levelnum)
                    && self.is_visible(&enemy.pos) != 0
                {
                    let x = enemy.pos.x;
                    let y = enemy.pos.y;
                    self.put_ashes(x, y);
                }
            }

            let levelnum = (*self.main.cur_level).levelnum;
            for enemy_index in 0..usize::try_from(self.main.num_enemys).unwrap() {
                let enemy = &self.main.all_enemys[enemy_index];
                if !((enemy.levelnum != levelnum)
                    || (enemy.status == Status::Out as i32)
                    || (enemy.status == Status::Terminated as i32))
                {
                    self.put_enemy(enemy_index.try_into().unwrap(), -1, -1)
                }
            }

            if self.vars.me.energy > 0. {
                self.put_influence(-1, -1);
            }

            for bullet_index in 0..MAXBULLETS {
                if self.main.all_bullets[bullet_index].ty != Status::Out as u8 {
                    self.put_bullet(bullet_index.try_into().unwrap())
                }
            }

            let &mut Data {
                main: Main { ref all_blasts, .. },
                ref mut vars,
                ref mut graphics,
                ..
            } = self;
            all_blasts
                .iter()
                .take(MAXBLASTS)
                .filter(|blast| blast.ty != Status::Out as i32)
                .for_each(|blast| put_blast(blast, vars, graphics));
        }

        // At this point we are done with the drawing procedure
        // and all that remains to be done is updating the screen.

        if mask & AssembleCombatWindowFlags::DO_SCREEN_UPDATE.bits() as i32 != 0 {
            SDL_UpdateRect(
                self.graphics.ne_screen.as_mut().unwrap().as_mut_ptr(),
                self.vars.user_rect.x().into(),
                self.vars.user_rect.y().into(),
                self.vars.user_rect.width().into(),
                self.vars.user_rect.height().into(),
            );
            SDL_UpdateRect(
                self.graphics.ne_screen.as_mut().unwrap().as_mut_ptr(),
                text_rect.x().into(),
                text_rect.y().into(),
                text_rect.width().into(),
                text_rect.height().into(),
            );
        }

        self.graphics.ne_screen.as_mut().unwrap().clear_clip_rect();
    }

    /// put some ashes at (x,y)
    pub unsafe fn put_ashes(&mut self, x: f32, y: f32) {
        if self.global.game_config.show_decals == 0 {
            return;
        }

        let user_center = self.vars.get_user_center();
        let mut dst = Rect::new(
            (f32::from(user_center.x())
                + (-self.vars.me.pos.x + x) * f32::from(self.vars.block_rect.width())
                - f32::from(self.vars.block_rect.width() / 2)) as i16,
            (f32::from(user_center.y())
                + (-self.vars.me.pos.y + y) * f32::from(self.vars.block_rect.height())
                - f32::from(self.vars.block_rect.height() / 2)) as i16,
            0,
            0,
        );

        let Graphics {
            decal_pics,
            ne_screen,
            ..
        } = &mut self.graphics;
        decal_pics[0]
            .as_mut()
            .unwrap()
            .blit_to(ne_screen.as_mut().unwrap(), &mut dst);
    }

    pub unsafe fn put_enemy(&mut self, enemy_index: c_int, x: c_int, y: c_int) {
        let droid = &self.main.all_enemys[usize::try_from(enemy_index).unwrap()];
        let ty = droid.ty;
        let phase = droid.phase;
        let name = &mut (*self.vars.droidmap.offset(ty.try_into().unwrap())).druidname;

        if (droid.status == Status::Terminated as i32)
            || (droid.status == Status::Out as i32)
            || (droid.levelnum != (*self.main.cur_level).levelnum)
        {
            return;
        }

        // if the enemy is out of sight, we need not do anything more here
        if self.main.show_all_droids == 0 && self.is_visible(&droid.pos) == 0 {
            trace!("ONSCREEN=FALSE --> usual end of function reached.");
            return;
        }

        // We check for incorrect droid types, which sometimes might occor, especially after
        // heavy editing of the crew initialisation functions ;)
        assert!(
            droid.ty < self.main.number_of_droid_types,
            "nonexistant droid-type encountered: {}",
            droid.ty
        );

        //--------------------
        // First blit just the enemy hat and shoes.
        let Graphics {
            enemy_surface_pointer,
            build_block,
            ..
        } = &mut self.graphics;
        enemy_surface_pointer[phase as usize]
            .as_mut()
            .unwrap()
            .blit(build_block.as_mut().unwrap());

        //--------------------
        // Now the numbers should be blittet.
        let mut dst = self.main.first_digit_rect;

        let Graphics {
            enemy_digit_surface_pointer,
            build_block,
            ne_screen,
            ..
        } = &mut self.graphics;
        enemy_digit_surface_pointer[usize::try_from(name[0] - b'1' as i8 + 1).unwrap()]
            .as_mut()
            .unwrap()
            .blit_to(build_block.as_mut().unwrap(), &mut dst);

        dst = self.main.second_digit_rect;
        enemy_digit_surface_pointer[usize::try_from(name[1] - b'1' as i8 + 1).unwrap()]
            .as_mut()
            .unwrap()
            .blit_to(build_block.as_mut().unwrap(), &mut dst);

        dst = self.main.third_digit_rect;
        enemy_digit_surface_pointer[usize::try_from(name[2] - b'1' as i8 + 1).unwrap()]
            .as_mut()
            .unwrap()
            .blit_to(build_block.as_mut().unwrap(), &mut dst);

        // now blit the whole construction to screen:
        if x == -1 {
            let user_center = self.vars.get_user_center();
            dst.set_x(
                (f32::from(user_center.x())
                    + (droid.pos.x - self.vars.me.pos.x) * f32::from(self.vars.block_rect.width())
                    - f32::from(self.vars.block_rect.width() / 2)) as i16,
            );
            dst.set_y(
                (f32::from(user_center.y())
                    + (droid.pos.y - self.vars.me.pos.y) * f32::from(self.vars.block_rect.height())
                    - f32::from(self.vars.block_rect.height() / 2)) as i16,
            );
        } else {
            dst.set_x(x.try_into().unwrap());
            dst.set_y(y.try_into().unwrap());
        }

        build_block
            .as_mut()
            .unwrap()
            .blit_to(ne_screen.as_mut().unwrap(), &mut dst);

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
                self.graphics.ne_screen.as_mut().unwrap(),
                self.global.font0_b_font,
                (f32::from(self.vars.user_rect.x())
                    + f32::from(self.vars.user_rect.width() / 2)
                    + f32::from(self.vars.block_rect.width() / 3)
                    + (droid.pos.x - self.vars.me.pos.x) * f32::from(self.vars.block_rect.width()))
                    as i32,
                (f32::from(self.vars.user_rect.y()) + f32::from(self.vars.user_rect.height() / 2)
                    - f32::from(self.vars.block_rect.height() / 2)
                    + (droid.pos.y - self.vars.me.pos.y) * f32::from(self.vars.block_rect.height()))
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
            self.vars.user_rect.x()
                + (self.vars.user_rect.width() / 2) as i16
                + (self.vars.block_rect.width() / 3) as i16,
            self.vars.user_rect.y() + (self.vars.user_rect.height() / 2) as i16
                - (self.vars.block_rect.height() / 2) as i16,
            self.vars.user_rect.width() / 2 - self.vars.block_rect.width() / 3,
            self.vars.user_rect.height() / 2,
        );

        trace!("PutInfluence real function call confirmed.");

        // Now we draw the hat and shoes of the influencer
        let Data {
            graphics:
                Graphics {
                    influencer_surface_pointer,
                    influ_digit_surface_pointer,
                    build_block,
                    ..
                },
            vars,
            ..
        } = self;
        influencer_surface_pointer[(vars.me.phase).floor() as usize]
            .as_mut()
            .unwrap()
            .blit(build_block.as_mut().unwrap());

        // Now we draw the first digit of the influencers current number.
        let mut dst = self.main.first_digit_rect;
        influ_digit_surface_pointer[usize::try_from(
            (*vars.droidmap.offset(vars.me.ty.try_into().unwrap())).druidname[0] - b'1' as i8 + 1,
        )
        .unwrap()]
        .as_mut()
        .unwrap()
        .blit_to(build_block.as_mut().unwrap(), &mut dst);

        // Now we draw the second digit of the influencers current number.
        dst = self.main.second_digit_rect;
        influ_digit_surface_pointer[usize::try_from(
            (*vars.droidmap.offset(vars.me.ty.try_into().unwrap())).druidname[1] - b'1' as i8 + 1,
        )
        .unwrap()]
        .as_mut()
        .unwrap()
        .blit_to(build_block.as_mut().unwrap(), &mut dst);

        // Now we draw the third digit of the influencers current number.
        dst = self.main.third_digit_rect;

        influ_digit_surface_pointer[usize::try_from(
            (*vars.droidmap.offset(vars.me.ty.try_into().unwrap())).druidname[2] - b'1' as i8 + 1,
        )
        .unwrap()]
        .as_mut()
        .unwrap()
        .blit_to(build_block.as_mut().unwrap(), &mut dst);

        if self.vars.me.energy * 100.
            / (*self
                .vars
                .droidmap
                .offset(self.vars.me.ty.try_into().unwrap()))
            .maxenergy
            <= BLINKENERGY
            && x == -1
        {
            // In case of low energy, do the fading effect...
            let rest = self.vars.me.timer % BLINK_LEN; // period of fading is given by BLINK_LEN
            let filt = if rest < BLINK_LEN / 2. {
                0.40 + (1.0 - 2.0 * rest / BLINK_LEN) * 0.60 // decrease white->grey
            } else {
                0.40 + (2.0 * rest / BLINK_LEN - 1.0) * 0.60 // increase back to white
            };

            apply_filter(
                self.graphics.build_block.as_mut().unwrap(),
                filt,
                filt,
                filt,
            );

            // ... and also maybe start a new cry-sound

            if self.vars.me.last_crysound_time > CRY_SOUND_INTERVAL {
                self.vars.me.last_crysound_time = 0.;
                self.cry_sound();
            }
        }

        //--------------------
        // In case of transfer mode, we produce the transfer mode sound
        // but of course only in some periodic intervall...

        if self.vars.me.status == Status::Transfermode as i32 && x == -1 {
            apply_filter(self.graphics.build_block.as_mut().unwrap(), 1.0, 0.0, 0.0);

            if self.vars.me.last_transfer_sound_time > TRANSFER_SOUND_INTERVAL {
                self.vars.me.last_transfer_sound_time = 0.;
                self.transfer_sound();
            }
        }

        if x == -1 {
            let user_center = self.vars.get_user_center();
            dst.set_x(user_center.x() - (self.vars.block_rect.width() / 2) as i16);
            dst.set_y(user_center.y() - (self.vars.block_rect.height() / 2) as i16);
        } else {
            dst.set_x(x.try_into().unwrap());
            dst.set_y(y.try_into().unwrap());
        }

        let Graphics {
            build_block,
            ne_screen,
            ..
        } = &mut self.graphics;
        build_block
            .as_mut()
            .unwrap()
            .blit_to(ne_screen.as_mut().unwrap(), &mut dst);

        //--------------------
        // Maybe the influencer has something to say :)
        // so let him say it..
        //
        if x == -1
            && self.vars.me.text_visible_time < self.global.game_config.wanted_text_visible_time
            && self.global.game_config.droid_talk != 0
        {
            self.b_font.current_font = self.global.font0_b_font;
            self.display_text(
                self.vars.me.text_to_be_displayed,
                i32::from(self.vars.user_rect.x())
                    + i32::from(self.vars.user_rect.width() / 2)
                    + i32::from(self.vars.block_rect.width() / 3),
                i32::from(self.vars.user_rect.y()) + i32::from(self.vars.user_rect.height() / 2)
                    - i32::from(self.vars.block_rect.height() / 2),
                &text_rect,
            );
        }

        trace!("PutInfluence: end of function reached.");
    }

    /// PutBullet: draws a Bullet into the combat window.  The only
    /// parameter given is the number of the bullet in the AllBullets
    /// array. Everything else is computed in here.
    pub unsafe fn put_bullet(&mut self, bullet_number: c_int) {
        let cur_bullet = &mut self.main.all_bullets[usize::try_from(bullet_number).unwrap()];

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
                self.fill_rect(self.vars.user_rect, FLASH_LIGHT);
            } else if cur_bullet.time_in_seconds <= FLASH_DURATION / 2. {
                self.fill_rect(self.vars.user_rect, FLASH_DARK);
            } else if cur_bullet.time_in_seconds <= 3. * FLASH_DURATION / 4. {
                self.fill_rect(self.vars.user_rect, FLASH_LIGHT);
            } else if cur_bullet.time_in_seconds <= FLASH_DURATION {
                self.fill_rect(self.vars.user_rect, FLASH_DARK);
            }

            return;
        }

        let bullet = &mut *self
            .vars
            .bulletmap
            .offset(cur_bullet.ty.try_into().unwrap());
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
                cur_bullet.surfaces[i] = Some(
                    bullet.surfaces[i]
                        .as_mut()
                        .unwrap()
                        .rotozoom(cur_bullet.angle.into(), 1.0, false)
                        .unwrap(),
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
        let user_center = self.vars.get_user_center();
        let cur_bullet = &mut self.main.all_bullets[usize::try_from(bullet_number).unwrap()];
        let mut dst = Rect::new(
            (f32::from(user_center.x())
                - (self.vars.me.pos.x - cur_bullet.pos.x) * f32::from(self.vars.block_rect.width())
                - (cur_bullet.surfaces[phase_of_bullet]
                    .as_ref()
                    .unwrap()
                    .width()
                    / 2) as f32) as i16,
            (f32::from(user_center.y())
                - (self.vars.me.pos.y - cur_bullet.pos.y) * f32::from(self.vars.block_rect.width())
                - (cur_bullet.surfaces[phase_of_bullet]
                    .as_ref()
                    .unwrap()
                    .height()
                    / 2) as f32) as i16,
            0,
            0,
        );

        cur_bullet.surfaces[phase_of_bullet]
            .as_mut()
            .unwrap()
            .blit_to(self.graphics.ne_screen.as_mut().unwrap(), &mut dst);

        trace!("PutBullet: end of function reached.");
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
            left = INFLUENCE_MODE_NAMES[self.vars.me.status as usize].as_ptr();
        }

        if right.is_null()
        /* Right-DEFAULT: Score */
        {
            dummy = [0u8; 80];
            write!(dummy.as_mut(), "{}", self.main.show_score).unwrap();
            right = dummy.as_mut_ptr() as *mut c_char;
        }

        // Now fill in the text
        let left = CStr::from_ptr(left);
        let left_len = left.to_bytes().len();
        assert!(
            left_len <= LEFT_TEXT_LEN,
            "String {} too long for Left Infoline!!",
            left.to_string_lossy()
        );
        let right = CStr::from_ptr(right);
        let right_len = right.to_bytes().len();
        assert!(
            right_len <= RIGHT_TEXT_LEN,
            "String {} too long for Right Infoline!!",
            right.to_string_lossy()
        );

        /* Now prepare the left/right text-boxes */
        let mut left_box = [b' '; LEFT_TEXT_LEN + 10];
        let mut right_box = [b' '; RIGHT_TEXT_LEN + 10];

        left_box[..left_len].copy_from_slice(&left.to_bytes()[..left_len]);
        right_box[..right_len].copy_from_slice(&right.to_bytes()[..right_len]);

        left_box[LEFT_TEXT_LEN] = b'\0'; /* that's right, we want padding! */
        right_box[RIGHT_TEXT_LEN] = b'\0';

        // --------------------
        // No we see if the screen need an update...

        let screen_needs_update = self.graphics.banner_is_destroyed != 0
            || (flags & i32::from(DisplayBannerFlags::FORCE_UPDATE.bits())) != 0
            || PREVIOUS_LEFT_BOX
                .with(|previous_left_box| left_box.as_ref() != previous_left_box.borrow().as_ref())
            || PREVIOUS_RIGHT_BOX.with(|previous_right_box| {
                right_box.as_ref() != previous_right_box.borrow().as_ref()
            });
        if screen_needs_update {
            // Redraw the whole background of the top status bar
            let Graphics {
                ne_screen,
                banner_pic,
                ..
            } = &mut self.graphics;
            ne_screen.as_mut().unwrap().clear_clip_rect();
            let mut dst = Rect::default();
            banner_pic
                .as_mut()
                .unwrap()
                .blit_to(ne_screen.as_mut().unwrap(), &mut dst);

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
                dst.set_x(self.vars.left_info_rect.x());
                dst.set_y(
                    self.vars.left_info_rect.y()
                        - i16::try_from(font_height(&*self.global.para_b_font)).unwrap(),
                );
                print_string_font(
                    self.graphics.ne_screen.as_mut().unwrap(),
                    self.global.para_b_font,
                    dst.x().into(),
                    dst.y().into(),
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

                dst.set_x(self.vars.right_info_rect.x());
                dst.set_y(
                    self.vars.right_info_rect.y()
                        - i16::try_from(font_height(&*self.global.para_b_font)).unwrap(),
                );
                print_string_font(
                    self.graphics.ne_screen.as_mut().unwrap(),
                    self.global.para_b_font,
                    dst.x().into(),
                    dst.y().into(),
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
                SDL_UpdateRect(
                    self.graphics.ne_screen.as_mut().unwrap().as_mut_ptr(),
                    0,
                    0,
                    self.vars.banner_rect.width().into(),
                    self.vars.banner_rect.height().into(),
                );
            }

            self.graphics.banner_is_destroyed = false.into();
        }
    }
}

pub unsafe fn put_blast(blast: &Blast, vars: &mut Vars, graphics: &mut Graphics) {
    trace!("PutBlast: real function call confirmed.");

    // If the blast is already long deat, we need not do anything else here
    if blast.ty == Status::Out as i32 {
        return;
    }

    let user_center = vars.get_user_center();
    let mut dst = Rect::new(
        (f32::from(user_center.x())
            - (vars.me.pos.x - blast.px) * f32::from(vars.block_rect.width())
            - f32::from(vars.block_rect.width() / 2)) as i16,
        (f32::from(user_center.y())
            - (vars.me.pos.y - blast.py) * f32::from(vars.block_rect.height())
            - f32::from(vars.block_rect.height() / 2)) as i16,
        0,
        0,
    );

    vars.blastmap[usize::try_from(blast.ty).unwrap()].surfaces[(blast.phase).floor() as usize]
        .as_mut()
        .unwrap()
        .blit_to(graphics.ne_screen.as_mut().unwrap(), &mut dst);
    trace!("PutBlast: end of function reached.");
}
