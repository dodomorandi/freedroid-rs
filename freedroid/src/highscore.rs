use crate::{
    array_c_string::ArrayCString,
    b_font::{char_width, font_height},
    defs::{
        self, Criticality, DisplayBannerFlags, Status, Themed, DATE_LEN, GRAPHICS_DIR_C,
        HS_BACKGROUND_FILE, HS_EMPTY_ENTRY, MAX_HIGHSCORES, MAX_NAME_LEN,
    },
    graphics::Graphics,
};

use log::{info, warn};
use sdl::{convert::u8_to_usize, Rect};
use std::{
    fmt,
    fs::File,
    io::{BufReader, Read, Write},
    mem::{self, align_of, size_of},
    os::raw::{c_int, c_long},
    path::Path,
    rc::Rc,
};

#[derive(Debug, Default)]
pub struct Highscore {
    pub entries: Option<Box<[Entry]>>,
    pub num: i32,
}

#[derive(Debug)]
pub struct Entry {
    name: ArrayCString<{ u8_to_usize(MAX_NAME_LEN) + 5 }>,
    score: i64,
    date: ArrayCString<{ u8_to_usize(DATE_LEN) + 5 }>,
}

impl Default for Entry {
    fn default() -> Self {
        Self {
            name: ArrayCString::try_from(HS_EMPTY_ENTRY).unwrap(),
            date: ArrayCString::try_from(" --- ").unwrap(),
            score: -1,
        }
    }
}

impl Entry {
    fn new<Name, Date>(name: Name, score: i64, date: Date) -> Self
    where
        Name: TryInto<ArrayCString<{ u8_to_usize(MAX_NAME_LEN) + 5 }>>,
        Date: TryInto<ArrayCString<{ u8_to_usize(DATE_LEN) + 5 }>>,
        Name::Error: fmt::Debug,
        Date::Error: fmt::Debug,
    {
        Self {
            name: name.try_into().unwrap(),
            date: date.try_into().unwrap(),
            score,
        }
    }

    pub const fn score_padding() -> usize {
        let used_bytes = (u8_to_usize(MAX_NAME_LEN) + 5) % align_of::<i64>();
        if used_bytes == 0 {
            0
        } else {
            align_of::<i64>() - used_bytes
        }
    }

    pub const fn end_padding() -> usize {
        let used_bytes = (u8_to_usize(DATE_LEN) + 5) % align_of::<i64>();
        if used_bytes == 0 {
            0
        } else {
            align_of::<i64>() - used_bytes
        }
    }
}

impl Highscore {
    /// Set up a new highscore list: load from disk if found
    fn init_highscores_inner(&mut self, config_dir: Option<&Path>) {
        let file = config_dir.and_then(|config_dir| {
            let path = config_dir.join("highscores");
            let file = File::open(&path).ok().map(BufReader::new);
            if file.as_ref().is_some() {
                info!("Found highscore file {}", path.display());
            } else {
                warn!("No highscore file found...");
            }
            file
        });

        self.num = MAX_HIGHSCORES.into();
        let highscores = match file {
            Some(mut file) => (0..MAX_HIGHSCORES)
                .map(|_| {
                    const SCORE_PADDING: usize = Entry::score_padding();
                    const END_PADDING: usize = Entry::end_padding();

                    let mut name = ArrayCString::new();
                    name.use_slice_mut(|name| {
                        file.read_exact(name)
                            .expect("cannot read name from highscore file");
                    });

                    if SCORE_PADDING != 0 {
                        file.seek_relative(SCORE_PADDING.try_into().unwrap())
                            .expect("cannot skip padding bytes from highscore file");
                    }
                    let score = {
                        let mut raw_score = [0u8; size_of::<i64>()];
                        file.read_exact(&mut raw_score)
                            .expect("unable to read score from highscore file");

                        i64::from_le_bytes(raw_score)
                    };

                    let mut get_rest = || {
                        let mut date = ArrayCString::new();
                        date.use_slice_mut(|date| file.read_exact(date).ok())?;

                        if END_PADDING != 0 {
                            file.seek_relative(END_PADDING.try_into().unwrap()).ok()?;
                        }

                        Some(Entry { name, score, date })
                    };
                    get_rest().unwrap_or_default()
                })
                .collect(),
            None => std::iter::repeat_with(Entry::default)
                .take(MAX_HIGHSCORES.into())
                .collect(),
        };
        self.entries = Some(highscores);
    }

    fn save_highscores_inner(&mut self, config_dir: Option<&Path>) -> Result<(), ()> {
        if let Some(config_dir) = config_dir {
            let path = config_dir.join("highscores");
            let Ok(mut file) = File::create(&path) else {
                warn!("Failed to create highscores file. Giving up...");
                return Err(());
            };

            for entry in self.entries.as_mut().unwrap().iter_mut() {
                const SCORE_PADDING: usize = Entry::score_padding();
                const END_PADDING: usize = Entry::end_padding();

                file.write_all(entry.name.as_buffer_bytes()).unwrap();
                if SCORE_PADDING != 0 {
                    file.write_all(&[0; SCORE_PADDING]).unwrap();
                }
                file.write_all(&entry.score.to_le_bytes()).unwrap();
                file.write_all(entry.date.as_buffer_bytes()).unwrap();

                if END_PADDING != 0 {
                    file.write_all(&[0; END_PADDING]).unwrap();
                }
            }
            file.flush().unwrap();
            info!("Successfully updated highscores file '{}'", path.display());

            Ok(())
        } else {
            warn!("No config-dir found, cannot save highscores!");
            Err(())
        }
    }
}

impl crate::Data<'_> {
    pub fn update_highscores(&mut self) {
        let score = self.main.real_score;
        self.main.real_score = 0.;
        self.main.show_score = 0;

        if score <= 0. {
            return;
        }

        self.vars.me.status = Status::Debriefing as c_int;

        #[allow(clippy::cast_possible_truncation)]
        let Some(entry_pos) = self
            .highscore
            .entries
            .as_ref()
            .unwrap()
            .iter()
            .position(|entry| entry.score < score as c_long)
        else {
            return;
        };

        let prev_font = std::mem::replace(
            &mut self.b_font.current_font,
            self.global.highscore_b_font.clone(),
        );

        #[allow(clippy::cast_possible_wrap)]
        let user_center_x: i16 = self.vars.user_rect.x() + (self.vars.user_rect.width() / 2) as i16;
        #[allow(clippy::cast_possible_wrap)]
        let user_center_y: i16 =
            self.vars.user_rect.y() + (self.vars.user_rect.height() / 2) as i16;

        self.assemble_combat_picture(0);
        self.make_grid_on_screen(Some(&self.vars.user_rect.clone()));
        #[allow(clippy::cast_possible_wrap)]
        let mut dst = Rect::new(
            user_center_x - (self.vars.portrait_rect.width() / 2) as i16,
            user_center_y - (self.vars.portrait_rect.height() / 2) as i16,
            self.vars.portrait_rect.width(),
            self.vars.portrait_rect.height(),
        );

        let Self {
            graphics: Graphics {
                pic999, ne_screen, ..
            },
            ..
        } = self;
        pic999
            .as_mut()
            .unwrap()
            .blit_to(ne_screen.as_mut().unwrap(), &mut dst);

        let h = font_height(
            self.global
                .para_b_font
                .as_ref()
                .unwrap()
                .ro(&self.font_owner),
        );
        self.display_text(
            b"Great Score !",
            i32::from(dst.x()) - h,
            i32::from(dst.y()) - h,
            Some(self.vars.user_rect),
        );

        // TODO ARCADEINPUT
        #[cfg(not(target_os = "android"))]
        self.display_text(
            b"Enter your name: ",
            i32::from(dst.x()) - 5 * h,
            i32::from(dst.y()) + i32::from(dst.height()),
            Some(self.vars.user_rect),
        );

        #[cfg(target_os = "android")]
        self.wait_for_key_pressed();

        // TODO More ARCADEINPUT

        let ne_screen = self.graphics.ne_screen.as_mut().unwrap();
        assert!(ne_screen.flip());
        ne_screen.clear_clip_rect();

        let date = format!("{}", chrono::Local::now().format("%Y/%m/%d"));

        #[cfg(target_os = "android")]
        #[allow(clippy::cast_possible_truncation)]
        let new_entry = Entry::new("Player", score as i64, &*date);

        #[cfg(not(target_os = "android"))]
        #[allow(clippy::cast_possible_truncation)]
        let new_entry = Entry::new(
            &*self.get_string(MAX_NAME_LEN.into(), 2).unwrap(),
            score as i64,
            &*date,
        );

        let mut ne_screen = self.graphics.ne_screen.take().unwrap();
        self.printf_sdl(&mut ne_screen, -1, -1, format_args!("\n"));
        self.graphics.ne_screen = Some(ne_screen);

        self.highscore.entries.as_mut().unwrap()[entry_pos..]
            .iter_mut()
            .fold(new_entry, |new_entry, cur_entry| {
                mem::replace(cur_entry, new_entry)
            });

        self.b_font.current_font = prev_font;
    }

    pub fn init_highscores(&mut self) {
        self.highscore
            .init_highscores_inner(self.main.get_config_dir());
    }

    pub fn save_highscores(&mut self) -> c_int {
        match self
            .highscore
            .save_highscores_inner(self.main.get_config_dir())
        {
            Ok(()) => defs::OK.into(),
            Err(()) => defs::ERR.into(),
        }
    }

    /// Display the high scores of the single player game.
    /// This function is actually a submenu of the `MainMenu`.
    pub fn show_highscores(&mut self) {
        let fpath = Self::find_file_static(
            &self.global,
            &mut self.misc,
            HS_BACKGROUND_FILE,
            Some(GRAPHICS_DIR_C),
            Themed::NoTheme as c_int,
            Criticality::WarnOnly as c_int,
        );
        if let Some(fpath) = fpath {
            Self::display_image(self.sdl, &self.global, &mut self.graphics, fpath);
        }
        self.make_grid_on_screen(Some(&self.vars.screen_rect.clone()));
        self.display_banner(None, None, DisplayBannerFlags::FORCE_UPDATE.bits().into());

        let highscore_font = self.global.highscore_b_font.as_ref().unwrap();
        let prev_font = self.b_font.current_font.replace(Rc::clone(highscore_font));
        let highscore_font = highscore_font.ro(&self.font_owner);

        let len = char_width(highscore_font, b'9');

        let x0 = i32::from(self.vars.screen_rect.width()) / 8;
        let x1 = x0 + 2 * len;
        let x2 = x1 + 11 * len;
        let x3 = x2 + i32::try_from(MAX_NAME_LEN).unwrap() * len;

        let height = font_height(highscore_font);

        let y0 = i32::from(self.vars.full_user_rect.y()) + height;

        let mut ne_screen = self.graphics.ne_screen.take().unwrap();
        Self::centered_print_string(
            &self.b_font,
            &mut self.font_owner,
            &mut ne_screen,
            y0,
            format_args!("Top {}  scores\n", self.highscore.num),
        );

        let highscore_entries = self.highscore.entries.take().unwrap();
        for (i, highscore) in highscore_entries.iter().enumerate() {
            let i = i32::try_from(i).unwrap();
            self.print_string(
                &mut ne_screen,
                x0,
                y0 + (i + 2) * height,
                format_args!("{}", i + 1),
            );
            if highscore.score >= 0 {
                self.print_string(
                    &mut ne_screen,
                    x1,
                    y0 + (i + 2) * height,
                    format_args!("{}", highscore.date.to_str().unwrap()),
                );
            }
            self.print_string(
                &mut ne_screen,
                x2,
                y0 + (i + 2) * height,
                format_args!("{}", highscore.name.to_str().unwrap()),
            );
            if highscore.score >= 0 {
                self.print_string(
                    &mut ne_screen,
                    x3,
                    y0 + (i + 2) * height,
                    format_args!("{}", highscore.score),
                );
            }
        }
        self.highscore.entries = Some(highscore_entries);
        assert!(ne_screen.flip());
        self.graphics.ne_screen = Some(ne_screen);

        self.wait_for_key_pressed();

        self.b_font.current_font = prev_font;
    }
}
