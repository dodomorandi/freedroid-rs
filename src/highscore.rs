#[cfg(target_os = "android")]
use crate::input::wait_for_key_pressed;
use crate::{
    b_font::{FontHeight, GetCurrentFont, Highscore_BFont, Para_BFont, SetCurrentFont},
    defs::{self, Status, DATE_LEN, HS_EMPTY_ENTRY, MAX_HIGHSCORES, MAX_NAME_LEN},
    global::{
        ne_screen, num_highscores, pic999, ConfigDir, Highscores, Me, Portrait_Rect, RealScore,
        ShowScore, User_Rect,
    },
    graphics::MakeGridOnScreen,
    text::{printf_SDL, DisplayText},
    view::Assemble_Combat_Picture,
};

use cstr::cstr;
use log::{info, warn};
use sdl::video::ll::{SDL_Flip, SDL_Rect, SDL_SetClipRect, SDL_UpperBlit};
use std::{
    ffi::CStr,
    fmt,
    fs::File,
    io::{Read, Write},
    mem,
    os::raw::{c_char, c_int, c_long, c_void},
    path::Path,
    ptr::null_mut,
};

extern "C" {
    #[no_mangle]
    fn GetString(max_len: c_int, echo: c_int) -> *mut c_char;
}

#[repr(C)]
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

        Self { name, date, score }
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
fn init_highscores(config_dir: Option<&Path>) {
    let file = config_dir.and_then(|config_dir| {
        let path = config_dir.join("highscores");
        let file = File::open(&path).ok();
        match file.as_ref() {
            Some(_) => info!("Found highscore file {}", path.display()),
            None => warn!("No highscore file found..."),
        }
        file
    });

    unsafe { num_highscores = MAX_HIGHSCORES as _ };
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
            .take(MAX_HIGHSCORES.into())
            .collect(),
    };

    unsafe {
        Highscores = Box::into_raw(highscores) as *mut *mut HighscoreEntry;
    }
}

fn save_highscores(config_dir: Option<&Path>) -> Result<(), ()> {
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
                std::slice::from_raw_parts(
                    Highscores as *mut Box<HighscoreEntry>,
                    MAX_HIGHSCORES.into(),
                )
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

fn update_highscores() {
    let score = unsafe { RealScore };
    unsafe {
        RealScore = 0.;
        ShowScore = 0;
    }

    if score <= 0. {
        return;
    }

    unsafe {
        Me.status = Status::Debriefing as c_int;
    }

    let hightscores = unsafe {
        std::slice::from_raw_parts_mut(
            Highscores as *mut Box<HighscoreEntry>,
            MAX_HIGHSCORES.into(),
        )
    };
    let entry_pos = match hightscores
        .iter()
        .position(|entry| entry.score < score as c_long)
    {
        Some(entry_pos) => entry_pos,
        None => return,
    };

    unsafe {
        let prev_font = GetCurrentFont();
        SetCurrentFont(Highscore_BFont);

        let user_center_x: i16 = User_Rect.x + (User_Rect.w / 2) as i16;
        let user_center_y: i16 = User_Rect.y + (User_Rect.h / 2) as i16;

        Assemble_Combat_Picture(0);
        MakeGridOnScreen(&mut User_Rect as *mut _);
        let mut dst = SDL_Rect::new(
            user_center_x - (Portrait_Rect.w / 2) as i16,
            user_center_y - (Portrait_Rect.h / 2) as i16,
            Portrait_Rect.w,
            Portrait_Rect.h,
        );
        SDL_UpperBlit(pic999, null_mut(), ne_screen, &mut dst);
        let h = FontHeight(Para_BFont);
        DisplayText(
            cstr!("Great Score !").as_ptr(),
            i32::from(dst.x) - h,
            i32::from(dst.y) - h,
            &User_Rect,
        );

        // TODO ARCADEINPUT
        #[cfg(not(target_os = "android"))]
        DisplayText(
            cstr!("Enter your name: ").as_ptr(),
            i32::from(dst.x) - 5 * h,
            i32::from(dst.y) + i32::from(dst.h),
            &User_Rect,
        );

        #[cfg(target_os = "android")]
        wait_for_key_pressed();

        // TODO More ARCADEINPUT

        SDL_Flip(ne_screen);
        SDL_SetClipRect(ne_screen, null_mut());

        let date = format!("{}", chrono::Local::today().format("%Y/%m/%d"));

        #[cfg(target_os = "android")]
        let new_entry = HighscoreEntry::new("Player", score as i64, &date);
        #[cfg(not(target_os = "android"))]
        let new_entry = {
            let tmp_name = GetString(MAX_NAME_LEN as c_int, 2);
            let mut new_entry = HighscoreEntry::new("", score as i64, &date);
            libc::strcpy(new_entry.name.as_mut_ptr(), tmp_name);
            libc::free(tmp_name as *mut c_void);
            new_entry
        };

        printf_SDL(ne_screen, -1, -1, cstr!("\n").as_ptr() as *mut c_char);

        hightscores[entry_pos..]
            .iter_mut()
            .fold(new_entry, |new_entry, cur_entry| {
                mem::replace(cur_entry, new_entry)
            });

        SetCurrentFont(prev_font);
    }
}

fn get_config_dir() -> Option<&'static Path> {
    if unsafe { ConfigDir[0] } == 0 {
        None
    } else {
        let config_dir = unsafe { CStr::from_ptr(ConfigDir.as_ptr()) };
        let config_dir = Path::new(config_dir.to_str().unwrap());
        Some(config_dir)
    }
}

#[no_mangle]
pub extern "C" fn InitHighscores() {
    init_highscores(get_config_dir());
}

#[no_mangle]
pub extern "C" fn SaveHighscores() -> c_int {
    match save_highscores(get_config_dir()) {
        Ok(()) => defs::OK.into(),
        Err(()) => defs::ERR.into(),
    }
}

#[no_mangle]
pub extern "C" fn UpdateHighscores() {
    update_highscores()
}
