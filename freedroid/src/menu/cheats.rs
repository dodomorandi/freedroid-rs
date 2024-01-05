use crate::{cur_level, defs::Status};

use nom::Finish;
use sdl::FrameBuffer;

pub const X0: i32 = 50;
pub const Y0: i32 = 20;

impl<'sdl> crate::Data<'sdl> {
    pub(super) fn print_cheat_menu(&mut self, ne_screen: &mut FrameBuffer<'sdl>) {
        macro_rules! print_sdl {
            ($($args:tt)*) => {
                self.printf_sdl(
                    ne_screen,
                    -1,
                    -1,
                    format_args!($($args)+),
                );
            };
        }

        Self::printf_sdl_static(
            &mut self.text,
            &self.b_font,
            &mut self.font_owner,
            ne_screen,
            X0,
            Y0,
            format_args!(
                "Current position: Level={}, X={:.0}, Y={:.0}\n",
                cur_level!(self.main).levelnum,
                self.vars.me.pos.x.clone(),
                self.vars.me.pos.y.clone(),
            ),
        );
        print_sdl!(" a. Armageddon (alle Robots sprengen)\n");
        print_sdl!(" l. robot list of current level\n");
        print_sdl!(" g. complete robot list\n");
        print_sdl!(" d. destroy robots on current level\n");
        print_sdl!(" t. Teleportation\n");
        print_sdl!(" r. change to new robot type\n");
        print_sdl!(
            " i. Invinciblemode: {}\n",
            if self.main.invincible_mode == 0 {
                "OFF"
            } else {
                "ON"
            },
        );
        print_sdl!(" e. set energy\n");
        print_sdl!(
            " n. No hidden droids: {}\n",
            if self.main.show_all_droids == 0 {
                "OFF"
            } else {
                "ON"
            },
        );
        print_sdl!(" m. Map of Deck xy\n");
        print_sdl!(
            " s. Sound: {}\n",
            if self.main.sound_on == 0 { "OFF" } else { "ON" }
        );
        print_sdl!(" w. Print current waypoints\n");
        print_sdl!(" z. change Zoom factor\n");
        print_sdl!(
            " f. Freeze on this positon: {}\n",
            if self.main.stop_influencer == 0 {
                "OFF"
            } else {
                "ON"
            },
        );
        print_sdl!(" q. RESUME game\n");
    }

    #[must_use]
    pub(super) fn change_zoom_factor(
        &mut self,
        mut ne_screen: FrameBuffer<'sdl>,
    ) -> FrameBuffer<'sdl> {
        use nom::{character::complete::space0, number::complete::float, sequence::preceded};

        self.graphics.ne_screen = Some(ne_screen);
        self.clear_graph_mem();
        ne_screen = self.graphics.ne_screen.take().unwrap();
        self.printf_sdl(
            &mut ne_screen,
            X0,
            Y0,
            format_args!(
                "Current Zoom factor: {}\n",
                self.global.current_combat_scale_factor.clone(),
            ),
        );
        self.printf_sdl(&mut ne_screen, -1, -1, format_args!("New zoom factor: "));
        self.graphics.ne_screen = Some(ne_screen);
        let input = self.get_string(40, 2).unwrap();
        ne_screen = self.graphics.ne_screen.take().unwrap();

        self.global.current_combat_scale_factor =
            preceded(space0::<_, ()>, float)(input.to_bytes())
                .finish()
                .unwrap()
                .1;
        self.set_combat_scale_to(self.global.current_combat_scale_factor);
        ne_screen
    }

    #[must_use]
    pub(super) fn level_robots_list(
        &mut self,
        mut ne_screen: FrameBuffer<'sdl>,
    ) -> FrameBuffer<'sdl> {
        let mut l = 0; /* line counter for enemy output */
        for i in 0..usize::from(self.main.num_enemys) {
            let Some(enemy) = &self.main.all_enemys[i] else {
                continue;
            };
            if enemy.levelnum == cur_level!(self.main).levelnum {
                if l != 0 && l % 20 == 0 {
                    Self::printf_sdl_static(
                        &mut self.text,
                        &self.b_font,
                        &mut self.font_owner,
                        &mut ne_screen,
                        -1,
                        -1,
                        format_args!(" --- MORE --- \n"),
                    );
                    if self.getchar_raw() == b'q'.into() {
                        break;
                    }
                }
                if l % 20 == 0 {
                    self.graphics.ne_screen = Some(ne_screen);
                    self.clear_graph_mem();
                    ne_screen = self.graphics.ne_screen.take().unwrap();
                    Self::printf_sdl_static(
                        &mut self.text,
                        &self.b_font,
                        &mut self.font_owner,
                        &mut ne_screen,
                        X0,
                        Y0,
                        format_args!(" NR.   ID  X    Y   ENERGY   Status\n"),
                    );
                    Self::printf_sdl_static(
                        &mut self.text,
                        &self.b_font,
                        &mut self.font_owner,
                        &mut ne_screen,
                        -1,
                        -1,
                        format_args!("---------------------------------------------\n"),
                    );
                }

                l += 1;
                let enemy = self.main.all_enemys[i].as_ref().unwrap();
                let status = if enemy.status == Status::Out {
                    "OUT"
                } else if enemy.status == Status::Terminated {
                    "DEAD"
                } else {
                    "ACTIVE"
                };

                Self::printf_sdl_static(
                    &mut self.text,
                    &self.b_font,
                    &mut self.font_owner,
                    &mut ne_screen,
                    -1,
                    -1,
                    format_args!(
                        "{}.   {}   {:.0}   {:.0}   {:.0}    {}.\n",
                        i,
                        self.vars.droidmap[enemy.ty.to_usize()]
                            .druidname
                            .to_str()
                            .unwrap(),
                        enemy.pos.x.clone(),
                        enemy.pos.y.clone(),
                        enemy.energy.clone(),
                        status,
                    ),
                );
            }
        }

        self.printf_sdl(&mut ne_screen, -1, -1, format_args!(" --- END --- \n"));
        self.getchar_raw();
        ne_screen
    }

    #[must_use]
    pub(super) fn ship_robots_list(
        &mut self,
        mut ne_screen: FrameBuffer<'sdl>,
    ) -> FrameBuffer<'sdl> {
        for i in 0..usize::from(self.main.num_enemys) {
            if self.main.all_enemys[i].is_none() {
                continue;
            };

            if i != 0 && !i % 13 == 0 {
                self.printf_sdl(
                    &mut ne_screen,
                    -1,
                    -1,
                    format_args!(" --- MORE --- ('q' to quit)\n"),
                );
                if self.getchar_raw() == b'q'.into() {
                    break;
                }
            }
            if i % 13 == 0 {
                self.graphics.ne_screen = Some(ne_screen);
                self.clear_graph_mem();
                ne_screen = self.graphics.ne_screen.take().unwrap();
                self.printf_sdl(
                    &mut ne_screen,
                    X0,
                    Y0,
                    format_args!("Nr.  Lev. ID  Energy  Status.\n"),
                );
                self.printf_sdl(
                    &mut ne_screen,
                    -1,
                    -1,
                    format_args!("------------------------------\n"),
                );
            }

            let enemy = self.main.all_enemys[i].as_ref().unwrap();
            Self::printf_sdl_static(
                &mut self.text,
                &self.b_font,
                &mut self.font_owner,
                &mut ne_screen,
                -1,
                -1,
                format_args!(
                    "{}  {}  {}  {:.0}  {}\n",
                    i,
                    enemy.levelnum.clone(),
                    self.vars.droidmap[enemy.ty.to_usize()]
                        .druidname
                        .to_str()
                        .unwrap(),
                    enemy.energy.clone(),
                    enemy.status.name(),
                ),
            );
        }

        self.printf_sdl(&mut ne_screen, -1, -1, format_args!(" --- END ---\n"));
        self.getchar_raw();

        ne_screen
    }

    pub fn level_robots_destroy(&mut self, ne_screen: &mut FrameBuffer<'sdl>) {
        let cur_level = cur_level!(self.main);
        for enemy in self.main.all_enemys.iter_mut().filter_map(Option::as_mut) {
            if enemy.levelnum == cur_level.levelnum {
                enemy.energy = -100.;
            }
        }
        self.printf_sdl(
            ne_screen,
            -1,
            -1,
            format_args!("All robots on this deck killed!\n"),
        );
        self.getchar_raw();
    }

    #[must_use]
    pub(super) fn cheating_teleport(
        &mut self,
        mut ne_screen: FrameBuffer<'sdl>,
    ) -> FrameBuffer<'sdl> {
        use nom::{
            bytes::complete::tag,
            character::complete::{i32, space0, u8},
            sequence::{delimited, pair, preceded, tuple},
        };

        /* Teleportation */
        self.graphics.ne_screen = Some(ne_screen);
        self.clear_graph_mem();
        ne_screen = self.graphics.ne_screen.take().unwrap();
        self.printf_sdl(&mut ne_screen, X0, Y0, format_args!("Enter Level, X, Y: "));
        self.graphics.ne_screen = Some(ne_screen);
        let input = self.get_string(40, 2).unwrap();
        ne_screen = self.graphics.ne_screen.take().unwrap();

        let (l_num, x, y) = tuple((
            preceded(space0::<_, ()>, u8),
            preceded(pair(tag(", "), space0), i32),
            delimited(pair(tag(", "), space0), i32, tag("\n")),
        ))(input.to_bytes())
        .finish()
        .unwrap()
        .1;
        self.teleport(l_num, x, y);
        ne_screen
    }

    #[must_use]
    pub(super) fn change_robot_type(
        &mut self,
        mut ne_screen: FrameBuffer<'sdl>,
    ) -> FrameBuffer<'sdl> {
        self.graphics.ne_screen = Some(ne_screen);
        self.clear_graph_mem();
        ne_screen = self.graphics.ne_screen.take().unwrap();
        self.printf_sdl(
            &mut ne_screen,
            X0,
            Y0,
            format_args!("Type number of new robot: "),
        );
        self.graphics.ne_screen = Some(ne_screen);
        let input = self.get_string(40, 2).unwrap();
        ne_screen = self.graphics.ne_screen.take().unwrap();
        let mut i = 0u8;
        for _ in 0..self.main.number_of_droid_types {
            if self.vars.droidmap[usize::from(i)].druidname != *input {
                break;
            }
            i += 1;
        }

        if i == self.main.number_of_droid_types {
            self.printf_sdl(
                &mut ne_screen,
                X0,
                Y0 + 20,
                format_args!("Unrecognized robot-type: {}", input.to_str().unwrap(),),
            );
            self.getchar_raw();
            self.graphics.ne_screen = Some(ne_screen);
            self.clear_graph_mem();
            ne_screen = self.graphics.ne_screen.take().unwrap();
        } else {
            self.vars.me.ty = i.try_into().unwrap();
            self.vars.me.energy = self.vars.droidmap[self.vars.me.ty.to_usize()].maxenergy;
            self.vars.me.health = self.vars.me.energy;
            self.printf_sdl(
                &mut ne_screen,
                X0,
                Y0 + 20,
                format_args!("You are now a {}. Have fun!\n", input.to_str().unwrap(),),
            );
            self.getchar_raw();
        }
        ne_screen
    }

    #[must_use]
    pub(super) fn complete_heal(&mut self, mut ne_screen: FrameBuffer<'sdl>) -> FrameBuffer<'sdl> {
        use nom::{
            character::complete::{i32, space0},
            sequence::preceded,
        };

        /* complete heal */
        self.graphics.ne_screen = Some(ne_screen);
        self.clear_graph_mem();
        ne_screen = self.graphics.ne_screen.take().unwrap();
        self.printf_sdl(
            &mut ne_screen,
            X0,
            Y0,
            format_args!("Current energy: {}\n", self.vars.me.energy.clone()),
        );
        self.printf_sdl(
            &mut ne_screen,
            -1,
            -1,
            format_args!("Enter your new energy: "),
        );
        self.graphics.ne_screen = Some(ne_screen);
        let input = self.get_string(40, 2).unwrap();
        ne_screen = self.graphics.ne_screen.take().unwrap();

        let num = preceded(space0::<_, ()>, i32)(input.to_bytes())
            .finish()
            .unwrap()
            .1;
        #[allow(clippy::cast_precision_loss)]
        {
            self.vars.me.energy = num as f32;
        }
        if self.vars.me.energy > self.vars.me.health {
            self.vars.me.health = self.vars.me.energy;
        }
        ne_screen
    }

    #[must_use]
    pub(super) fn cheating_show_deck_map(
        &mut self,
        mut ne_screen: FrameBuffer<'sdl>,
    ) -> FrameBuffer<'sdl> {
        /* Show deck map in Concept view */
        self.printf_sdl(&mut ne_screen, -1, -1, format_args!("\nLevelnum: "));
        self.graphics.ne_screen = Some(ne_screen);
        self.show_deck_map();
        ne_screen = self.graphics.ne_screen.take().unwrap();
        self.getchar_raw();
        ne_screen
    }

    #[must_use]
    pub(super) fn print_waypoints(
        &mut self,
        mut ne_screen: FrameBuffer<'sdl>,
    ) -> FrameBuffer<'sdl> {
        for i in 0..cur_level!(self.main).all_waypoints.len() {
            if i != 0 && i % 20 == 0 {
                self.printf_sdl(&mut ne_screen, -1, -1, format_args!(" ---- MORE -----\n"));
                if self.getchar_raw() == b'q'.into() {
                    break;
                }
            }
            if i % 20 == 0 {
                self.graphics.ne_screen = Some(ne_screen);
                self.clear_graph_mem();
                ne_screen = self.graphics.ne_screen.take().unwrap();
                self.printf_sdl(
                    &mut ne_screen,
                    X0,
                    Y0,
                    format_args!("Nr.   X   Y      C1  C2  C3  C4\n"),
                );
                self.printf_sdl(
                    &mut ne_screen,
                    -1,
                    -1,
                    format_args!("------------------------------------\n"),
                );
            }
            let cur_level = cur_level!(self.main);
            let waypoint = &cur_level.all_waypoints[i];
            Self::printf_sdl_static(
                &mut self.text,
                &self.b_font,
                &mut self.font_owner,
                &mut ne_screen,
                -1,
                -1,
                format_args!(
                    "{:2}   {:2}  {:2}      {:2}  {:2}  {:2}  {:2}\n",
                    i,
                    waypoint.x,
                    waypoint.y,
                    waypoint.connections[0],
                    waypoint.connections[1],
                    waypoint.connections[2],
                    waypoint.connections[3],
                ),
            );
        }
        self.printf_sdl(&mut ne_screen, -1, -1, format_args!(" --- END ---\n"));
        self.getchar_raw();

        ne_screen
    }
}
