use crate::{
    array_c_string::ArrayCString,
    b_font::{font_height, print_string_font, put_string_font},
    defs::{
        AssembleCombatWindowFlags, BulletKind, DisplayBannerFlags, Status, BLINKENERGY,
        CRY_SOUND_INTERVAL, FLASH_DURATION, LEFT_TEXT_LEN, MAXBULLETS, RIGHT_TEXT_LEN,
        TRANSFER_SOUND_INTERVAL,
    },
    graphics::{apply_filter, Graphics},
    map::get_map_brick,
    structs::{Blast, CoarsePoint, Finepoint, TextToBeDisplayed},
    text,
    vars::Vars,
    view::screen_updater::{screen_needs_update, update_screen},
    Main,
};

use arrayvec::ArrayString;
use log::{info, trace};
use sdl::{Pixel, Rect};
use sdl_sys::SDL_Color;
use std::{
    cell::Cell,
    ffi::CStr,
    ops::{Deref, Not},
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

impl crate::Data<'_> {
    pub fn fill_rect(&mut self, rect: Rect, color: SDL_Color) {
        let pixcolor = self
            .graphics
            .ne_screen
            .as_ref()
            .unwrap()
            .format()
            .map_rgb(color.r, color.g, color.b);

        self.graphics
            .ne_screen
            .as_mut()
            .unwrap()
            .fill_with(&rect, pixcolor)
            .unwrap();
    }

    /// This function assembles the contents of the combat window
    /// in `self.graphics.ne_screen`.
    ///
    /// Several FLAGS can be used to control its behaviour:
    ///
    /// (*) `ONLY_SHOW_MAP` = 0x01:  This flag indicates not do draw any
    ///     game elements but the map blocks
    ///
    /// (*) `DO_SCREEgN_UPDATE` = 0x02: This flag indicates for the function
    ///     to also cause an `SDL_Update` of the portion of the screen
    ///     that has been modified
    ///
    /// (*) `SHOW_FULL_MAP` = 0x04: show complete map, disregard visibility
    pub fn assemble_combat_picture(&mut self, mask: AssembleCombatWindowFlags) {
        trace!("\nvoid Assemble_Combat_Picture(...): Real function call confirmed.");

        self.graphics
            .ne_screen
            .as_mut()
            .unwrap()
            .set_clip_rect(&self.vars.user_rect);
        if self.global.game_config.all_map_visible.not() {
            self.fill_rect(self.vars.user_rect, BLACK);
        }

        let [upleft, downright] = self.assemble_combat_picture_get_upleft_downright(mask);
        (upleft.y..downright.y)
            .flat_map(|line| (upleft.x..downright.x).map(move |col| (line, col)))
            .fold(
                BlitCombatCellState {
                    pos: Finepoint::default(),
                    vect: Finepoint::default(),
                    len: -1.,
                    map_brick: 0,
                    target_rectangle: Rect::default(),
                },
                |state, (line, col)| self.blit_combat_cell(line, col, mask, state),
            );

        // if we don't use Fullscreen mode, we have to clear the text-background manually
        // for the info-line text:

        let font0_b_font = self
            .global
            .font0_b_font
            .as_ref()
            .unwrap()
            .rw(&mut self.font_owner);
        let text_rect = Rect::new(
            self.vars.full_user_rect.x(),
            (i32::from(self.vars.full_user_rect.y())
                + i32::from(self.vars.full_user_rect.height())
                - i32::from(font_height(font0_b_font)))
            .try_into()
            .unwrap(),
            self.vars.full_user_rect.width(),
            font_height(font0_b_font),
        );
        self.graphics
            .ne_screen
            .as_mut()
            .unwrap()
            .set_clip_rect(&text_rect);
        if self.global.game_config.full_user_rect.not() {
            self.graphics
                .ne_screen
                .as_mut()
                .unwrap()
                .fill_with(&text_rect, Pixel::black())
                .unwrap();
        }

        if self.global.game_config.draw_position {
            #[allow(clippy::cast_possible_wrap)]
            print_string_font(
                self.graphics.ne_screen.as_mut().unwrap(),
                font0_b_font,
                (self.vars.full_user_rect.x() + (self.vars.full_user_rect.width() / 6) as i16)
                    .into(),
                i32::from(self.vars.full_user_rect.y())
                    + i32::from(self.vars.full_user_rect.height())
                    - i32::from(font_height(font0_b_font)),
                format_args!(
                    "GPS: X={:.0} Y={:.0} Lev={}",
                    self.vars.me.pos.x.round(),
                    self.vars.me.pos.y.round(),
                    self.main.cur_level().levelnum,
                ),
            );
        }

        if mask
            .contains(AssembleCombatWindowFlags::ONLY_SHOW_MAP)
            .not()
        {
            self.assemble_combat_window_draw();
        }

        // At this point we are done with the drawing procedure
        // and all that remains to be done is updating the screen.

        if mask.contains(AssembleCombatWindowFlags::DO_SCREEN_UPDATE) {
            let screen = self.graphics.ne_screen.as_mut().unwrap();
            screen.update_rect(&self.vars.user_rect);
            screen.update_rect(&text_rect);
        }

        self.graphics.ne_screen.as_mut().unwrap().clear_clip_rect();
    }

    fn assemble_combat_picture_get_upleft_downright(
        &self,
        mask: AssembleCombatWindowFlags,
    ) -> [CoarsePoint<i8>; 2] {
        if mask
            .contains(AssembleCombatWindowFlags::SHOW_FULL_MAP)
            .not()
        {
            #[allow(clippy::cast_possible_truncation)]
            let upleft = CoarsePoint {
                x: self.vars.me.pos.x as i8 - 6,
                y: self.vars.me.pos.y as i8 - 5,
            };
            #[allow(clippy::cast_possible_truncation)]
            let downright = CoarsePoint {
                x: self.vars.me.pos.x as i8 + 7,
                y: self.vars.me.pos.y as i8 + 5,
            };
            [upleft, downright]
        } else {
            let upleft = CoarsePoint { x: -5, y: -5 };
            let downright = CoarsePoint {
                x: i8::try_from(self.main.cur_level().xlen.checked_add(5).unwrap()).unwrap(),
                y: i8::try_from(self.main.cur_level().ylen.checked_add(5).unwrap()).unwrap(),
            };
            [upleft, downright]
        }
    }

    /// put some ashes at (x,y)
    pub fn put_ashes(&mut self, x: f32, y: f32) {
        if self.global.game_config.show_decals.not() {
            return;
        }

        let user_center = self.vars.get_user_center();
        #[allow(clippy::cast_possible_truncation)]
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

    pub fn put_enemy(&mut self, enemy_index: u16, x: i32, y: i32) {
        let droid = &self.main.enemys[usize::from(enemy_index)];
        let phase = droid.phase;
        let name = &self.vars.droidmap[droid.ty.to_usize()].druidname;

        if matches!(droid.status, Status::Out | Status::Terminated)
            || (droid.levelnum != self.main.cur_level().levelnum)
        {
            return;
        }

        // if the enemy is out of sight, we need not do anything more here
        if self.main.show_all_droids.not() && self.is_visible(droid.pos) == 0 {
            trace!("ONSCREEN=FALSE --> usual end of function reached.");
            return;
        }

        //--------------------
        // First blit just the enemy hat and shoes.
        let Graphics {
            enemy_surface_pointer,
            build_block,
            ..
        } = &mut self.graphics;
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
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
        enemy_digit_surface_pointer[usize::from(name[0] + 1 - b'1')]
            .as_mut()
            .unwrap()
            .blit_to(build_block.as_mut().unwrap(), &mut dst);

        dst = self.main.second_digit_rect;
        enemy_digit_surface_pointer[usize::from(name[1] + 1 - b'1')]
            .as_mut()
            .unwrap()
            .blit_to(build_block.as_mut().unwrap(), &mut dst);

        dst = self.main.third_digit_rect;
        enemy_digit_surface_pointer[usize::from(name[2] + 1 - b'1')]
            .as_mut()
            .unwrap()
            .blit_to(build_block.as_mut().unwrap(), &mut dst);

        // now blit the whole construction to screen:
        if x == -1 {
            let user_center = self.vars.get_user_center();
            #[allow(clippy::cast_possible_truncation)]
            dst.set_x(
                (f32::from(user_center.x())
                    + (droid.pos.x - self.vars.me.pos.x) * f32::from(self.vars.block_rect.width())
                    - f32::from(self.vars.block_rect.width() / 2)) as i16,
            );
            #[allow(clippy::cast_possible_truncation)]
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
            && self.global.game_config.droid_talk
        {
            #[allow(clippy::cast_possible_truncation)]
            put_string_font(
                self.graphics.ne_screen.as_mut().unwrap(),
                self.global
                    .font0_b_font
                    .as_ref()
                    .unwrap()
                    .rw(&mut self.font_owner),
                (f32::from(self.vars.user_rect.x())
                    + f32::from(self.vars.user_rect.width() / 2)
                    + f32::from(self.vars.block_rect.width() / 3)
                    + (droid.pos.x - self.vars.me.pos.x) * f32::from(self.vars.block_rect.width()))
                    as i32,
                (f32::from(self.vars.user_rect.y()) + f32::from(self.vars.user_rect.height() / 2)
                    - f32::from(self.vars.block_rect.height() / 2)
                    + (droid.pos.y - self.vars.me.pos.y) * f32::from(self.vars.block_rect.height()))
                    as i32,
                droid.text_to_be_displayed.as_bytes(),
            );
        }

        trace!("ENEMY HAS BEEN PUT --> usual end of function reached.");
    }

    /// This function draws the influencer to the screen, either
    /// to the center of the combat window if (-1,-1) was specified, or
    /// to the specified coordinates anywhere on the screen, useful e.g.
    /// for using the influencer as a cursor in the menus.
    pub fn put_influence(&mut self, x: i32, y: i32) {
        trace!("PutInfluence real function call confirmed.");

        // Now we draw the hat and shoes of the influencer
        let crate::Data {
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
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        influencer_surface_pointer[(vars.me.phase).floor() as usize]
            .as_mut()
            .unwrap()
            .blit(build_block.as_mut().unwrap());

        // Now we draw the three digits of the influencers current number.
        [
            self.main.first_digit_rect,
            self.main.second_digit_rect,
            self.main.third_digit_rect,
        ]
        .into_iter()
        .zip(vars.droidmap[vars.me.ty.to_usize()].druidname)
        .for_each(|(mut dst, name_char)| {
            influ_digit_surface_pointer[usize::from(name_char + 1 - b'1')]
                .as_mut()
                .unwrap()
                .blit_to(build_block.as_mut().unwrap(), &mut dst);
        });

        if self.vars.me.energy * 100. / self.vars.droidmap[self.vars.me.ty.to_usize()].maxenergy
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

        if self.vars.me.status == Status::Transfermode && x == -1 {
            apply_filter(self.graphics.build_block.as_mut().unwrap(), 1.0, 0.0, 0.0);

            if self.vars.me.last_transfer_sound_time > TRANSFER_SOUND_INTERVAL {
                self.vars.me.last_transfer_sound_time = 0.;
                self.transfer_sound();
            }
        }

        let mut dst = if x == -1 {
            let user_center = self.vars.get_user_center();
            #[allow(clippy::cast_possible_wrap)]
            let x = user_center.x() - (self.vars.block_rect.width() / 2) as i16;
            #[allow(clippy::cast_possible_wrap)]
            let y = user_center.y() - (self.vars.block_rect.height() / 2) as i16;
            Rect::new(x, y, 0, 0)
        } else {
            Rect::new(x.try_into().unwrap(), y.try_into().unwrap(), 0, 0)
        };

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
            && self.global.game_config.droid_talk
        {
            self.make_influencer_talk();
        }

        trace!("PutInfluence: end of function reached.");
    }

    fn make_influencer_talk(&mut self) {
        self.b_font
            .current_font
            .clone_from(&self.global.font0_b_font);
        let text_to_display = match &self.vars.me.text_to_be_displayed {
            TextToBeDisplayed::None => b"",
            TextToBeDisplayed::String(s) => s.to_bytes(),
            TextToBeDisplayed::LevelEnterComment => self.main.cur_level().enter_comment.to_bytes(),
        };

        let Self {
            b_font,
            text,
            vars,
            graphics,
            font_owner,
            ..
        } = self;

        #[allow(clippy::cast_possible_wrap)]
        let text_rect = Rect::new(
            vars.user_rect.x()
                + (vars.user_rect.width() / 2) as i16
                + (vars.block_rect.width() / 3) as i16,
            vars.user_rect.y() + (vars.user_rect.height() / 2) as i16
                - (vars.block_rect.height() / 2) as i16,
            vars.user_rect.width() / 2 - vars.block_rect.width() / 3,
            vars.user_rect.height() / 2,
        );

        text::Displayer {
            data_text: text,
            graphics,
            vars,
            b_font,
            font_owner,
            text: text_to_display,
            start_x: i32::from(vars.user_rect.x())
                + i32::from(vars.user_rect.width() / 2)
                + i32::from(vars.block_rect.width() / 3),
            start_y: i32::from(vars.user_rect.y()) + i32::from(vars.user_rect.height() / 2)
                - i32::from(vars.block_rect.height() / 2),
            clip: Some(text_rect),
        }
        .run();
    }

    /// `PutBullet`: draws a Bullet into the combat window.  The only
    /// parameter given is the number of the bullet in the `AllBullets`
    /// array. Everything else is computed in here.
    pub fn put_bullet(&mut self, bullet_number: u8) {
        let cur_bullet = self.main.all_bullets[usize::from(bullet_number)]
            .as_mut()
            .unwrap();

        trace!("PutBullet: real function call confirmed.");

        //--------------------
        // in case our bullet is of the type "FLASH", we only
        // draw a big white or black rectangle right over the
        // combat window, white for even frames and black for
        // odd frames.
        if cur_bullet.ty == BulletKind::Flash {
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

        let bullet = &mut self.vars.bulletmap[cur_bullet.ty.to_usize()];
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let mut phase_of_bullet =
            (cur_bullet.time_in_seconds * f32::from(bullet.phase_changes_per_second)) as usize;

        phase_of_bullet %= usize::from(bullet.phases);

        // DebugPrintf( 0 , "\nPhaseOfBullet: %d.", PhaseOfBullet );

        //--------------------
        // Maybe it's the first time this bullet is displayed.  But then, the images
        // of the rotated bullet in all phases are not yet attached to the bullet.
        // Then, we'll have to generate these
        //
        //if ( cur_bullet.time_in_frames == 1 )
        if cur_bullet.surfaces_were_generated.not() {
            for i in 0..usize::from(bullet.phases) {
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
            cur_bullet.surfaces_were_generated = true;
        }

        // WARNING!!! PAY ATTENTION HERE!! After the rotozoom was applied to the image, it is NO
        // LONGER of dimension Block_Rect.w times Block_Rect.h, but of the dimesions of the smallest
        // rectangle containing the full rotated Block_Rect.h x Block_Rect.w rectangle!!!
        // This has to be taken into account when calculating the target position for the
        // blit of these surfaces!!!!
        let user_center = self.vars.get_user_center();
        let cur_bullet = self.main.all_bullets[usize::from(bullet_number)]
            .as_mut()
            .unwrap();
        #[allow(clippy::cast_possible_truncation)]
        let mut dst = Rect::new(
            (f32::from(user_center.x())
                - (self.vars.me.pos.x - cur_bullet.pos.x) * f32::from(self.vars.block_rect.width())
                - f32::from(
                    cur_bullet.surfaces[phase_of_bullet]
                        .as_ref()
                        .unwrap()
                        .width()
                        / 2,
                )) as i16,
            (f32::from(user_center.y())
                - (self.vars.me.pos.y - cur_bullet.pos.y) * f32::from(self.vars.block_rect.width())
                - f32::from(
                    cur_bullet.surfaces[phase_of_bullet]
                        .as_ref()
                        .unwrap()
                        .height()
                        / 2,
                )) as i16,
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
    /// `BANNER_FORCE_UPDATE=1`: Forces the redrawing of the title bar
    ///
    /// `BANNER_DONT_TOUCH_TEXT=2`: Prevent`DisplayBanner`er from touching the
    /// text.
    ///
    /// `BANNER_NO_SDL_UPDATE=4`: Prevents any `SDL_Update` calls.
    pub fn display_banner(
        &mut self,
        left: Option<&CStr>,
        right: Option<&CStr>,
        flags: DisplayBannerFlags,
    ) {
        // --------------------
        // At first the text is prepared.  This can't hurt.
        // we will decide whether to display it or not later...
        //

        let left = left.unwrap_or(self.vars.me.status.c_name());

        let right = right.map_or_else(
            || {
                use std::fmt::Write;

                let mut buffer = ArrayCString::<80>::default();
                write!(buffer, "{}", self.main.show_score).unwrap();
                CStrFixedCow::Owned(buffer)
            },
            CStrFixedCow::Borrowed,
        );

        // Now fill in the text
        let left_len = left.to_bytes().len();
        assert!(
            left_len <= LEFT_TEXT_LEN,
            "String {} too long for Left Infoline!!",
            left.to_string_lossy()
        );
        let right_len = right.to_bytes().len();
        assert!(
            right_len <= RIGHT_TEXT_LEN,
            "String {} too long for Right Infoline!!",
            right.to_string_lossy()
        );

        /* Now prepare the left/right text-boxes */
        let left_box =
            ArrayString::<LEFT_TEXT_LEN>::from(std::str::from_utf8(left.to_bytes()).unwrap())
                .unwrap();
        let right_box =
            ArrayString::<RIGHT_TEXT_LEN>::from(std::str::from_utf8(right.to_bytes()).unwrap())
                .unwrap();
        // --------------------
        // No we see if the screen need an update...

        if screen_needs_update(self, left_box, right_box, flags) {
            update_screen(self, left_box, right_box, flags);
        }
    }

    fn blit_combat_cell(
        &mut self,
        line: i8,
        col: i8,
        mask: AssembleCombatWindowFlags,
        state: BlitCombatCellState,
    ) -> BlitCombatCellState {
        let BlitCombatCellState {
            mut pos,
            mut vect,
            mut len,
            mut map_brick,
            mut target_rectangle,
        } = state;

        if self.global.game_config.all_map_visible.not()
            && mask
                .contains(AssembleCombatWindowFlags::SHOW_FULL_MAP)
                .not()
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
            if self.is_visible(pos) == 0 {
                return BlitCombatCellState {
                    pos,
                    vect,
                    len,
                    map_brick,
                    target_rectangle,
                };
            }
        }

        map_brick = get_map_brick(self.main.cur_level(), col.into(), line.into());
        let user_center = self.vars.get_user_center();
        #[allow(clippy::cast_possible_truncation)]
        target_rectangle.set_x(
            user_center.x()
                + ((-self.vars.me.pos.x + 1.0 * f32::from(col) - 0.5)
                    * f32::from(self.vars.block_rect.width()))
                .round() as i16,
        );
        #[allow(clippy::cast_possible_truncation)]
        target_rectangle.set_y(
            user_center.y()
                + ((-self.vars.me.pos.y + 1.0 * f32::from(line) - 0.5)
                    * f32::from(self.vars.block_rect.height()))
                .round() as i16,
        );

        let mut surface = self.graphics.map_block_surface_pointer
            [self.main.cur_level().color.to_usize()][usize::from(map_brick)]
        .as_mut()
        .unwrap()
        .borrow_mut();
        surface.blit_to(
            self.graphics.ne_screen.as_mut().unwrap(),
            &mut target_rectangle,
        );

        BlitCombatCellState {
            pos,
            vect,
            len,
            map_brick,
            target_rectangle,
        }
    }

    fn assemble_combat_window_draw(&mut self) {
        if self.global.game_config.draw_framerate {
            self.assemble_combat_window_draw_framerate();
        }

        let font0_b_font = self
            .global
            .font0_b_font
            .as_ref()
            .unwrap()
            .rw(&mut self.font_owner);

        if self.global.game_config.draw_energy {
            print_string_font(
                self.graphics.ne_screen.as_mut().unwrap(),
                font0_b_font,
                i32::from(self.vars.full_user_rect.x())
                    + i32::from(self.vars.full_user_rect.width()) / 2,
                i32::from(self.vars.full_user_rect.y())
                    + i32::from(self.vars.full_user_rect.height())
                    - i32::from(font_height(font0_b_font)),
                format_args!("Energy: {:.0}", self.vars.me.energy),
            );
        }
        if self.global.game_config.draw_death_count {
            print_string_font(
                self.graphics.ne_screen.as_mut().unwrap(),
                font0_b_font,
                i32::from(self.vars.full_user_rect.x())
                    + 2 * i32::from(self.vars.full_user_rect.width()) / 3,
                i32::from(self.vars.full_user_rect.y())
                    + i32::from(self.vars.full_user_rect.height())
                    - i32::from(font_height(font0_b_font)),
                format_args!("Deathcount: {:.0}", self.main.death_count,),
            );
        }

        self.graphics
            .ne_screen
            .as_mut()
            .unwrap()
            .set_clip_rect(&self.vars.user_rect);

        // make sure Ashes are displayed _before_ droids, so that they are _under_ them!
        for enemy_index in 0..self.main.enemys.len() {
            let enemy = &self.main.enemys[enemy_index];
            if enemy.status == Status::Terminated
                && (enemy.levelnum == self.main.cur_level().levelnum)
                && self.is_visible(enemy.pos) != 0
            {
                let x = enemy.pos.x;
                let y = enemy.pos.y;
                self.put_ashes(x, y);
            }
        }

        let levelnum = self.main.cur_level().levelnum;
        for enemy_index in 0..self.main.enemys.len() {
            let enemy = &self.main.enemys[enemy_index];
            if enemy.levelnum == levelnum
                && matches!(enemy.status, Status::Out | Status::Terminated).not()
            {
                self.put_enemy(enemy_index.try_into().unwrap(), -1, -1);
            }
        }

        if self.vars.me.energy > 0. {
            self.put_influence(-1, -1);
        }

        for bullet_index in 0..MAXBULLETS {
            if self.main.all_bullets[usize::from(bullet_index)].is_some() {
                self.put_bullet(bullet_index);
            }
        }

        let &mut crate::Data {
            main: Main { ref all_blasts, .. },
            ref mut vars,
            ref mut graphics,
            ..
        } = self;
        all_blasts
            .iter()
            .filter(|blast| blast.ty.is_some())
            .for_each(|blast| put_blast(blast, vars, graphics));
    }

    fn assemble_combat_window_draw_framerate(&mut self) {
        thread_local! {
            static TIME_SINCE_LAST_FPS_UPDATE: Cell<f32> = const { Cell::new(10.) };
            static FPS_DISPLAYED: Cell<i32> = const { Cell::new(1) };
        }

        const UPDATE_FPS_HOW_OFTEN: f32 = 0.75;

        TIME_SINCE_LAST_FPS_UPDATE.with(|time_cell| {
            let mut time = time_cell.get();
            time += self.frame_time();

            if time > UPDATE_FPS_HOW_OFTEN {
                #[allow(clippy::cast_possible_truncation)]
                FPS_DISPLAYED.with(|fps_displayed| {
                    fps_displayed.set((1.0 / self.frame_time()) as i32);
                });
                time_cell.set(0.);
            } else {
                time_cell.set(time);
            }
        });

        let font0_b_font = self
            .global
            .font0_b_font
            .as_ref()
            .unwrap()
            .rw(&mut self.font_owner);
        FPS_DISPLAYED.with(|fps_displayed| {
            print_string_font(
                self.graphics.ne_screen.as_mut().unwrap(),
                font0_b_font,
                self.vars.full_user_rect.x().into(),
                i32::from(self.vars.full_user_rect.y())
                    + i32::from(self.vars.full_user_rect.height())
                    - i32::from(font_height(font0_b_font)),
                format_args!("FPS: {} ", fps_displayed.get()),
            );
        });
    }
}

pub fn put_blast(blast: &Blast, vars: &mut Vars, graphics: &mut Graphics) {
    trace!("PutBlast: real function call confirmed.");

    // If the blast is already long deat, we need not do anything else here
    let Some(blast_type) = blast.ty else {
        return;
    };

    let user_center = vars.get_user_center();
    #[allow(clippy::cast_possible_truncation)]
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

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    vars.blastmap[blast_type].surfaces[(blast.phase).floor() as usize]
        .as_mut()
        .unwrap()
        .blit_to(graphics.ne_screen.as_mut().unwrap(), &mut dst);
    trace!("PutBlast: end of function reached.");
}

#[derive(Debug, Clone, Copy)]
struct BlitCombatCellState {
    pos: Finepoint,
    vect: Finepoint,
    len: f32,
    map_brick: u8,
    target_rectangle: Rect,
}

#[derive(Debug)]
enum CStrFixedCow<'a, const N: usize> {
    Owned(ArrayCString<N>),
    Borrowed(&'a CStr),
}

impl<const N: usize> Deref for CStrFixedCow<'_, N> {
    type Target = CStr;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(s) => s,
            &Self::Borrowed(s) => s,
        }
    }
}

mod screen_updater {
    use std::{cell::RefCell, ops::Not};

    use arrayvec::ArrayString;
    use sdl::Rect;

    use crate::{
        b_font::{font_height, print_string_font},
        defs::{DisplayBannerFlags, LEFT_TEXT_LEN, RIGHT_TEXT_LEN},
        graphics::Graphics,
    };

    thread_local! {
        static PREVIOUS_LEFT_BOX: RefCell<ArrayString::<LEFT_TEXT_LEN>>={
          RefCell::new(ArrayString::from("NOUGHT").unwrap())
        };
        static PREVIOUS_RIGHT_BOX: RefCell<ArrayString::<RIGHT_TEXT_LEN>>= {
          RefCell::new(ArrayString::from("NOUGHT").unwrap())
        };
    }

    pub fn screen_needs_update(
        data: &crate::Data,
        left_box: ArrayString<LEFT_TEXT_LEN>,
        right_box: ArrayString<RIGHT_TEXT_LEN>,
        flags: DisplayBannerFlags,
    ) -> bool {
        data.graphics.banner_is_destroyed
            || flags.contains(DisplayBannerFlags::FORCE_UPDATE)
            || PREVIOUS_LEFT_BOX
                .with(|previous_left_box| &left_box != previous_left_box.borrow().as_ref())
            || PREVIOUS_RIGHT_BOX
                .with(|previous_right_box| &right_box != previous_right_box.borrow().as_ref())
    }

    pub fn update_screen(
        data: &mut crate::Data,
        left_box: ArrayString<LEFT_TEXT_LEN>,
        right_box: ArrayString<RIGHT_TEXT_LEN>,
        flags: DisplayBannerFlags,
    ) {
        // Redraw the whole background of the top status bar
        let Graphics {
            ne_screen,
            banner_pic,
            ..
        } = &mut data.graphics;
        ne_screen.as_mut().unwrap().clear_clip_rect();
        let mut dst = Rect::default();
        banner_pic
            .as_mut()
            .unwrap()
            .blit_to(ne_screen.as_mut().unwrap(), &mut dst);

        // Now the text should be ready and its
        // time to display it...
        let previous_left_check = PREVIOUS_LEFT_BOX
            .with(|previous_left_box| &left_box != previous_left_box.borrow().as_ref());
        let previous_right_check = PREVIOUS_RIGHT_BOX
            .with(|previous_right_box| &right_box != previous_right_box.borrow().as_ref());
        if previous_left_check
            || previous_right_check
            || flags.contains(DisplayBannerFlags::FORCE_UPDATE)
        {
            let para_b_font = data
                .global
                .para_b_font
                .as_ref()
                .unwrap()
                .rw(&mut data.font_owner);
            dst.set_x(data.vars.left_info_rect.x());
            dst.set_y(
                data.vars.left_info_rect.y() - i16::try_from(font_height(para_b_font)).unwrap(),
            );
            print_string_font(
                data.graphics.ne_screen.as_mut().unwrap(),
                para_b_font,
                dst.x().into(),
                dst.y().into(),
                format_args!("{left_box}"),
            );
            PREVIOUS_LEFT_BOX.with(|previous_left_box| {
                let mut previous_left_box = previous_left_box.borrow_mut();
                *previous_left_box = left_box;
            });

            dst.set_x(data.vars.right_info_rect.x());
            dst.set_y(
                data.vars.right_info_rect.y() - i16::try_from(font_height(para_b_font)).unwrap(),
            );
            print_string_font(
                data.graphics.ne_screen.as_mut().unwrap(),
                para_b_font,
                dst.x().into(),
                dst.y().into(),
                format_args!("{right_box}"),
            );
            PREVIOUS_RIGHT_BOX.with(|previous_right_box| {
                let mut previous_right_box = previous_right_box.borrow_mut();
                *previous_right_box = right_box;
            });
        }

        // finally update the whole top status box
        if flags.contains(DisplayBannerFlags::NO_SDL_UPDATE).not() {
            data.graphics
                .ne_screen
                .as_mut()
                .unwrap()
                .update_rect(&data.vars.banner_rect.with_xy(0, 0));
        }

        data.graphics.banner_is_destroyed = false;
    }
}
