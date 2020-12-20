#![feature(c_variadic)]
#![feature(slice_strip)]
#![feature(const_maybe_uninit_assume_init)]

mod b_font;
mod bullet;
mod defs;
mod enemy;
mod global;
mod graphics;
mod highscore;
mod influence;
mod init;
mod input;
mod map;
mod menu;
mod misc;
mod ship;
mod sound;
mod structs;
mod takeover;
mod text;
mod view;

use sdl::video::ll::SDL_Surface;
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

#[inline]
fn sdl_must_lock(surface: &SDL_Surface) -> bool {
    use sdl::video::SurfaceFlag::*;
    surface.offset != 0
        && (surface.flags & (HWSurface as u32 | AsyncBlit as u32 | RLEAccel as u32)) != 0
}
