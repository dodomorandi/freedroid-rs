mod b_font;
mod defs;
mod global;
mod graphics;
mod highscore;
mod input;
mod map;
mod misc;
mod sound;
mod structs;
mod takeover;
mod text;
mod view;

use std::{
    env,
    ffi::CString,
    os::raw::{c_char, c_int},
    process,
};

extern "C" {
    fn c_main(argc: c_int, argv: *const *const c_char) -> c_int;
}

fn main() {
    env_logger::init();

    let args: Vec<_> = env::args().map(|arg| CString::new(arg).unwrap()).collect();
    let c_args: Vec<_> = args.iter().map(|arg| arg.as_ptr()).collect();

    let retval = unsafe { c_main(c_args.len() as c_int, c_args.as_ptr()) };
    if retval != 0 {
        process::exit(retval);
    }
}
