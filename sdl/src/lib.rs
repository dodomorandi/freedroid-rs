mod pixel;
mod surface;
mod video;

use sdl_sys::SDL_Quit;
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
