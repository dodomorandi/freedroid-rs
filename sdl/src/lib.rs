#![deny(unsafe_op_in_unsafe_fn)]

mod pixel;
mod rect;
mod surface;
mod video;

use std::ffi::CStr;

pub use rect::*;
use sdl_sys::{SDL_GetError, SDL_Quit};
pub use surface::*;
pub use video::Video;

#[derive(Debug)]
pub struct Sdl {
    video: Option<Video>,
}

impl Drop for Sdl {
    fn drop(&mut self) {
        unsafe {
            SDL_Quit();
        }
    }
}

impl Sdl {
    pub fn get_error(&mut self) -> &CStr {
        unsafe { get_error() }
    }
}

/// Get the last SDL error.
///
/// # Safety
/// - [SDL_Init](sdl_sys::SDL_Init) must have been called.
/// - The returned `CStr` must be dropped before [SDL_Quit].
unsafe fn get_error<'a>() -> &'a CStr {
    // SAFETY
    // [SDL_GetError] always return a valid C string, even without errors.
    unsafe { CStr::from_ptr(SDL_GetError()) }
}
