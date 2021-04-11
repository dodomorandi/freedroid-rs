use crate::{
    b_font::{
        centered_print_string, char_width, font_height, get_current_font, print_string,
        set_current_font,
    },
    defs::{
        self, Criticality, DisplayBannerFlags, Status, Themed, DATE_LEN, GRAPHICS_DIR_C,
        HS_BACKGROUND_FILE_C, HS_EMPTY_ENTRY, MAX_HIGHSCORES, MAX_NAME_LEN,
    },
    global::{HIGHSCORE_B_FONT, PARA_B_FONT},
    graphics::{display_image, make_grid_on_screen, NE_SCREEN, PIC999},
    input::wait_for_key_pressed,
    misc::find_file,
    text::{display_text, printf_sdl},
    vars::{FULL_USER_RECT, ME, PORTRAIT_RECT, SCREEN_RECT, USER_RECT},
    view::{assemble_combat_picture, display_banner},
    CONFIG_DIR, REAL_SCORE, SHOW_SCORE,
};

#[cfg(not(target_os = "android"))]
use crate::text::get_string;

use cstr::cstr;
use log::{info, warn};
use sdl::video::ll::{SDL_Flip, SDL_Rect, SDL_SetClipRect, SDL_UpperBlit};
use std::{
    convert::TryFrom,
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

#[cfg(not(target_os = "android"))]
use std::os::raw::c_void;

pub static mut HIGHSCORES: *mut *mut HighscoreEntry = null_mut();
pub static mut NUM_HIGHSCORES: i32 = 0; /* total number of entries in our list (fixed) */

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

/// Set up a new highscore list: load from disk if found
fn init_highscores_inner(config_dir: Option<&Path>) {
    let file = config_dir.and_then(|config_dir| {
        let path = config_dir.join("highscores");
        let file = File::open(&path).ok();
        match file.as_ref() {
            Some(_) => info!("Found highscore file {}", path.display()),
            None => warn!("No highscore file found..."),
        }
        file
    });

    unsafe { NUM_HIGHSCORES = MAX_HIGHSCORES as _ };
    let highscores: Box<_> = match file {
        Some(mut file) => (0..MAX_HIGHSCORES)
            .map(|_| {
                let mut entry = mem::MaybeUninit::uninit();
                unsafe {
                    let as_slice = std::slice::from_raw_parts_mut(
                        entry.as_mut_ptr() as *mut u8,
                        mem::size_of::<HighscoreEntry>(),
                    );
                    file.read_exact(as_slice).unwrap();
                    Box::new(entry.assume_init())
                }
            })
            .collect(),
        None => std::iter::repeat_with(|| Box::new(HighscoreEntry::default()))
            .take(MAX_HIGHSCORES)
            .collect(),
    };

    unsafe {
        HIGHSCORES = Box::into_raw(highscores) as *mut *mut HighscoreEntry;
    }
}

fn save_highscores_inner(config_dir: Option<&Path>) -> Result<(), ()> {
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

            let highscores = unsafe {
                std::slice::from_raw_parts(HIGHSCORES as *mut Box<HighscoreEntry>, MAX_HIGHSCORES)
            };
            for entry in highscores.iter() {
                let as_slice = unsafe {
                    std::slice::from_raw_parts(
                        entry.as_ref() as *const HighscoreEntry as *const u8,
                        mem::size_of::<HighscoreEntry>(),
                    )
                };
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

pub fn update_highscores() {
    let score = unsafe { REAL_SCORE };
    unsafe {
        REAL_SCORE = 0.;
        SHOW_SCORE = 0;
    }

    if score <= 0. {
        return;
    }

    unsafe {
        ME.status = Status::Debriefing as c_int;
    }

    let hightscores = unsafe {
        std::slice::from_raw_parts_mut(HIGHSCORES as *mut Box<HighscoreEntry>, MAX_HIGHSCORES)
    };
    let entry_pos = match hightscores
        .iter()
        .position(|entry| entry.score < score as c_long)
    {
        Some(entry_pos) => entry_pos,
        None => return,
    };

    unsafe {
        let prev_font = get_current_font();
        set_current_font(HIGHSCORE_B_FONT);

        let user_center_x: i16 = USER_RECT.x + (USER_RECT.w / 2) as i16;
        let user_center_y: i16 = USER_RECT.y + (USER_RECT.h / 2) as i16;

        assemble_combat_picture(0);
        make_grid_on_screen(Some(&USER_RECT));
        let mut dst = SDL_Rect::new(
            user_center_x - (PORTRAIT_RECT.w / 2) as i16,
            user_center_y - (PORTRAIT_RECT.h / 2) as i16,
            PORTRAIT_RECT.w,
            PORTRAIT_RECT.h,
        );
        SDL_UpperBlit(PIC999, null_mut(), NE_SCREEN, &mut dst);
        let h = font_height(&*PARA_B_FONT);
        display_text(
            cstr!("Great Score !").as_ptr(),
            i32::from(dst.x) - h,
            i32::from(dst.y) - h,
            &USER_RECT,
        );

        // TODO ARCADEINPUT
        #[cfg(not(target_os = "android"))]
        display_text(
            cstr!("Enter your name: ").as_ptr(),
            i32::from(dst.x) - 5 * h,
            i32::from(dst.y) + i32::from(dst.h),
            &USER_RECT,
        );

        #[cfg(target_os = "android")]
        wait_for_key_pressed();

        // TODO More ARCADEINPUT

        SDL_Flip(NE_SCREEN);
        SDL_SetClipRect(NE_SCREEN, null_mut());

        let date = format!("{}", chrono::Local::today().format("%Y/%m/%d"));

        #[cfg(target_os = "android")]
        let new_entry = HighscoreEntry::new("Player", score as i64, &date);
        #[cfg(not(target_os = "android"))]
        let new_entry = {
            let tmp_name = get_string(MAX_NAME_LEN as c_int, 2);
            let mut new_entry = HighscoreEntry::new("", score as i64, &date);
            libc::strcpy(new_entry.name.as_mut_ptr(), tmp_name);
            libc::free(tmp_name as *mut c_void);
            new_entry
        };

        printf_sdl(NE_SCREEN, -1, -1, format_args!("\n"));

        hightscores[entry_pos..]
            .iter_mut()
            .fold(new_entry, |new_entry, cur_entry| {
                mem::replace(cur_entry, new_entry)
            });

        set_current_font(prev_font);
    }
}

fn get_config_dir() -> Option<&'static Path> {
    if unsafe { CONFIG_DIR[0] } == 0 {
        None
    } else {
        let config_dir = unsafe { CStr::from_ptr(CONFIG_DIR.as_ptr()) };
        let config_dir = Path::new(config_dir.to_str().unwrap());
        Some(config_dir)
    }
}

pub fn init_highscores() {
    init_highscores_inner(get_config_dir());
}

pub fn save_highscores() -> c_int {
    match save_highscores_inner(get_config_dir()) {
        Ok(()) => defs::OK.into(),
        Err(()) => defs::ERR.into(),
    }
}

/// Display the high scores of the single player game.
/// This function is actually a submenu of the MainMenu.
pub unsafe fn show_highscores() {
    let fpath = find_file(
        HS_BACKGROUND_FILE_C.as_ptr() as *mut c_char,
        GRAPHICS_DIR_C.as_ptr() as *mut c_char,
        Themed::NoTheme as c_int,
        Criticality::WarnOnly as c_int,
    );
    if fpath.is_null().not() {
        display_image(fpath);
    }
    make_grid_on_screen(Some(&SCREEN_RECT));
    display_banner(
        null_mut(),
        null_mut(),
        DisplayBannerFlags::FORCE_UPDATE.bits().into(),
    );

    let prev_font = get_current_font();
    set_current_font(HIGHSCORE_B_FONT);

    let len = char_width(&*get_current_font(), b'9');

    let x0 = i32::from(SCREEN_RECT.w) / 8;
    let x1 = x0 + 2 * len;
    let x2 = x1 + 11 * len;
    let x3 = x2 + i32::try_from(MAX_NAME_LEN).unwrap() * len;

    let height = font_height(&*get_current_font());

    let y0 = i32::from(FULL_USER_RECT.y) + height;

    centered_print_string(
        NE_SCREEN,
        y0,
        format_args!("Top {}  scores\n", NUM_HIGHSCORES),
    );

    let highscores =
        std::slice::from_raw_parts(HIGHSCORES, usize::try_from(NUM_HIGHSCORES).unwrap());
    for (i, highscore) in highscores.iter().copied().enumerate() {
        let i = i32::try_from(i).unwrap();
        print_string(
            NE_SCREEN,
            x0,
            y0 + (i + 2) * height,
            format_args!("{}", i + 1),
        );
        if (*highscore).score >= 0 {
            print_string(
                NE_SCREEN,
                x1,
                y0 + (i + 2) * height,
                format_args!(
                    "{}",
                    CStr::from_ptr((*highscore).date.as_ptr()).to_str().unwrap()
                ),
            );
        }
        print_string(
            NE_SCREEN,
            x2,
            y0 + (i + 2) * height,
            format_args!(
                "{}",
                CStr::from_ptr((*highscore).name.as_ptr()).to_str().unwrap()
            ),
        );
        if (*highscore).score >= 0 {
            print_string(
                NE_SCREEN,
                x3,
                y0 + (i + 2) * height,
                format_args!("{}", (*highscore).score),
            );
        }
    }
    SDL_Flip(NE_SCREEN);

    wait_for_key_pressed();

    set_current_font(prev_font);
}
