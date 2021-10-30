use crate::{
    b_font::{char_width, font_height},
    defs::{
        self, Criticality, DisplayBannerFlags, Status, Themed, DATE_LEN, GRAPHICS_DIR_C,
        HS_BACKGROUND_FILE_C, HS_EMPTY_ENTRY, MAX_HIGHSCORES, MAX_NAME_LEN,
    },
    graphics::Graphics,
    Data,
};

use cstr::cstr;
use log::{info, warn};
use std::{
    ffi::CStr,
    fmt,
    fs::File,
    io::{Read, Write},
    mem,
    ops::Not,
    os::raw::{c_char, c_int, c_long},
    path::Path,
    ptr::null_mut,
};

#[derive(Debug, Default)]
pub struct Highscore {
    pub entries: Option<Box<[HighscoreEntry]>>,
    pub num: i32,
}

pub struct HighscoreEntry {
    name: [c_char; MAX_NAME_LEN + 5],
    score: c_long,
    date: [c_char; DATE_LEN + 5],
}

impl fmt::Debug for HighscoreEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name =
            unsafe { std::slice::from_raw_parts(self.name.as_ptr() as *const u8, self.name.len()) };
        let date =
            unsafe { std::slice::from_raw_parts(self.date.as_ptr() as *const u8, self.date.len()) };

        f.debug_struct("HighscoreEntry")
            .field(
                "name",
                &CStr::from_bytes_with_nul(name)
                    .ok()
                    .and_then(|name| name.to_str().ok())
                    .unwrap_or("[INVALID]"),
            )
            .field("score", &self.score)
            .field(
                "date",
                &CStr::from_bytes_with_nul(date)
                    .ok()
                    .and_then(|date| date.to_str().ok())
                    .unwrap_or("[INVALID]"),
            )
            .finish()
    }
}

impl Default for HighscoreEntry {
    fn default() -> Self {
        let mut name = [0; MAX_NAME_LEN + 5];
        name.iter_mut()
            .zip(HS_EMPTY_ENTRY.bytes().map(|c| c as c_char))
            .for_each(|(dst, src)| *dst = src);

        let mut date = [0; DATE_LEN + 5];
        date.iter_mut()
            .zip(b" --- ".iter().copied().map(|c| c as c_char))
            .for_each(|(dst, src)| *dst = src);
        let score = -1;

        Self { name, score, date }
    }
}

impl HighscoreEntry {
    fn new(name: &str, score: i64, date: &str) -> Self {
        let mut real_name = [0; MAX_NAME_LEN + 5];
        name.bytes()
            .take(MAX_NAME_LEN)
            .zip(real_name.iter_mut())
            .for_each(|(src, dst)| *dst = src as c_char);

        let mut real_date = [0; DATE_LEN + 5];
        date.bytes()
            .take(DATE_LEN)
            .zip(real_date.iter_mut())
            .for_each(|(src, dst)| *dst = src as c_char);

        Self {
            name: real_name,
            score,
            date: real_date,
        }
    }
}

impl Data<'_> {
    /// Set up a new highscore list: load from disk if found
    unsafe fn init_highscores_inner(&mut self, config_dir: Option<&Path>) {
        let file = config_dir.and_then(|config_dir| {
            let path = config_dir.join("highscores");
            let file = File::open(&path).ok();
            match file.as_ref() {
                Some(_) => info!("Found highscore file {}", path.display()),
                None => warn!("No highscore file found..."),
            }
            file
        });

        self.highscore.num = MAX_HIGHSCORES as _;
        let highscores = match file {
            Some(mut file) => (0..MAX_HIGHSCORES)
                .map(|_| {
                    let mut entry = mem::MaybeUninit::uninit();
                    let as_slice = std::slice::from_raw_parts_mut(
                        entry.as_mut_ptr() as *mut u8,
                        mem::size_of::<HighscoreEntry>(),
                    );
                    file.read_exact(as_slice).unwrap();
                    entry.assume_init()
                })
                .collect(),
            None => std::iter::repeat_with(HighscoreEntry::default)
                .take(MAX_HIGHSCORES)
                .collect(),
        };
        self.highscore.entries = Some(highscores);
    }

    unsafe fn save_highscores_inner(&mut self, config_dir: Option<&Path>) -> Result<(), ()> {
        match config_dir {
            Some(config_dir) => {
                let path = config_dir.join("highscores");
                let mut file = match File::create(&path) {
                    Ok(file) => file,
                    Err(_) => {
                        warn!("Failed to create highscores file. Giving up...");
                        return Err(());
                    }
                };

                for entry in self.highscore.entries.as_mut().unwrap().iter_mut() {
                    let as_slice = std::slice::from_raw_parts(
                        entry as *mut HighscoreEntry as *const u8,
                        mem::size_of::<HighscoreEntry>(),
                    );
                    file.write_all(as_slice).unwrap();
                }
                file.sync_all().unwrap();
                info!("Successfully updated highscores file '{}'", path.display());

                Ok(())
            }
            None => {
                warn!("No config-dir found, cannot save highscores!");
                Err(())
            }
        }
    }

    pub unsafe fn update_highscores(&mut self) {
        let score = self.main.real_score;
        self.main.real_score = 0.;
        self.main.show_score = 0;

        if score <= 0. {
            return;
        }

        self.vars.me.status = Status::Debriefing as c_int;

        let entry_pos = match self
            .highscore
            .entries
            .as_ref()
            .unwrap()
            .iter()
            .position(|entry| entry.score < score as c_long)
        {
            Some(entry_pos) => entry_pos,
            None => return,
        };

        let prev_font =
            std::mem::replace(&mut self.b_font.current_font, self.global.highscore_b_font);

        let user_center_x: i16 = self.vars.user_rect.x + (self.vars.user_rect.w / 2) as i16;
        let user_center_y: i16 = self.vars.user_rect.y + (self.vars.user_rect.h / 2) as i16;

        self.assemble_combat_picture(0);
        self.make_grid_on_screen(Some(&self.vars.user_rect.clone()));
        let mut dst = rect!(
            user_center_x - (self.vars.portrait_rect.w / 2) as i16,
            user_center_y - (self.vars.portrait_rect.h / 2) as i16,
            self.vars.portrait_rect.w,
            self.vars.portrait_rect.h,
        );

        let Data {
            graphics: Graphics {
                pic999, ne_screen, ..
            },
            ..
        } = self;
        pic999
            .as_mut()
            .unwrap()
            .blit_to(ne_screen.as_mut().unwrap(), &mut dst);

        let h = font_height(&*self.global.para_b_font);
        self.display_text(
            cstr!("Great Score !").as_ptr(),
            i32::from(dst.x) - h,
            i32::from(dst.y) - h,
            &self.vars.user_rect,
        );

        // TODO ARCADEINPUT
        #[cfg(not(target_os = "android"))]
        self.display_text(
            cstr!("Enter your name: ").as_ptr(),
            i32::from(dst.x) - 5 * h,
            i32::from(dst.y) + i32::from(dst.h),
            &self.vars.user_rect,
        );

        #[cfg(target_os = "android")]
        wait_for_key_pressed();

        // TODO More ARCADEINPUT

        let ne_screen = self.graphics.ne_screen.as_mut().unwrap();
        assert!(ne_screen.flip());
        ne_screen.clear_clip_rect();

        let date = format!("{}", chrono::Local::today().format("%Y/%m/%d"));

        #[cfg(target_os = "android")]
        let new_entry = HighscoreEntry::new("Player", score as i64, &date);
        #[cfg(not(target_os = "android"))]
        let new_entry = {
            let tmp_name = self.get_string(MAX_NAME_LEN as c_int, 2);
            let mut new_entry = HighscoreEntry::new("", score as i64, &date);
            libc::strcpy(new_entry.name.as_mut_ptr(), tmp_name);
            drop(Vec::from_raw_parts(
                tmp_name,
                MAX_NAME_LEN + 5,
                MAX_NAME_LEN + 5,
            ));
            new_entry
        };

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

    unsafe fn get_config_dir(&self) -> Option<&'static Path> {
        if self.main.config_dir[0] == 0 {
            None
        } else {
            let config_dir = CStr::from_ptr(self.main.config_dir.as_ptr());
            let config_dir = Path::new(config_dir.to_str().unwrap());
            Some(config_dir)
        }
    }

    pub unsafe fn init_highscores(&mut self) {
        self.init_highscores_inner(self.get_config_dir());
    }

    pub unsafe fn save_highscores(&mut self) -> c_int {
        match self.save_highscores_inner(self.get_config_dir()) {
            Ok(()) => defs::OK.into(),
            Err(()) => defs::ERR.into(),
        }
    }

    /// Display the high scores of the single player game.
    /// This function is actually a submenu of the MainMenu.
    pub unsafe fn show_highscores(&mut self) {
        let fpath = self.find_file(
            HS_BACKGROUND_FILE_C.as_ptr() as *mut c_char,
            GRAPHICS_DIR_C.as_ptr() as *mut c_char,
            Themed::NoTheme as c_int,
            Criticality::WarnOnly as c_int,
        );
        if fpath.is_null().not() {
            self.display_image(fpath);
        }
        self.make_grid_on_screen(Some(&self.vars.screen_rect.clone()));
        self.display_banner(
            null_mut(),
            null_mut(),
            DisplayBannerFlags::FORCE_UPDATE.bits().into(),
        );

        let prev_font =
            std::mem::replace(&mut self.b_font.current_font, self.global.highscore_b_font);

        let len = char_width(&*self.b_font.current_font, b'9');

        let x0 = i32::from(self.vars.screen_rect.w) / 8;
        let x1 = x0 + 2 * len;
        let x2 = x1 + 11 * len;
        let x3 = x2 + i32::try_from(MAX_NAME_LEN).unwrap() * len;

        let height = font_height(&*self.b_font.current_font);

        let y0 = i32::from(self.vars.full_user_rect.y) + height;

        let mut ne_screen = self.graphics.ne_screen.take().unwrap();
        self.centered_print_string(
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
                    format_args!(
                        "{}",
                        CStr::from_ptr(highscore.date.as_ptr()).to_str().unwrap()
                    ),
                );
            }
            self.print_string(
                &mut ne_screen,
                x2,
                y0 + (i + 2) * height,
                format_args!(
                    "{}",
                    CStr::from_ptr(highscore.name.as_ptr()).to_str().unwrap()
                ),
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
