use crate::defs::*;

use log::{info, warn};
use std::{
    ffi::CStr,
    fmt,
    fs::File,
    io::Read,
    mem,
    os::raw::{c_char, c_int, c_long},
    path::Path,
};

extern "C" {
    #[no_mangle]
    static mut num_highscores: c_int;
    #[no_mangle]
    static mut Highscores: *mut *mut HighscoreEntry;
    #[no_mangle]
    static mut ConfigDir: [c_char; 255];
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
            .zip(HS_EMPTY_ENTRY.iter().copied().map(|c| c as c_char))
            .for_each(|(dst, src)| *dst = src);

        let mut date = [0; DATE_LEN + 5];
        date.iter_mut()
            .zip(b" --- ".iter().copied().map(|c| c as c_char))
            .for_each(|(dst, src)| *dst = src);
        let score = -1;

        Self { name, date, score }
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

    unsafe { num_highscores = MAX_HIGHSCORES.into() };
    let mut highscores: Box<_> = match file {
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
        Highscores = highscores.as_mut_ptr() as *mut *mut HighscoreEntry;
    }
    mem::forget(highscores);
}

#[no_mangle]
pub extern "C" fn InitHighscores() {
    let config_dir = if unsafe { ConfigDir[0] } == 0 {
        None
    } else {
        let config_dir = unsafe { CStr::from_ptr(ConfigDir.as_ptr()) };
        let config_dir = Path::new(config_dir.to_str().unwrap());
        Some(config_dir)
    };

    init_highscores(config_dir);
}
