use std::{num::NonZeroU8, os::raw::c_int, ptr::NonNull};

use bitflags::bitflags;
use sdl_sys::{
    SDL_SetVideoMode, SDL_ANYFORMAT, SDL_ASYNCBLIT, SDL_DOUBLEBUF, SDL_FULLSCREEN, SDL_HWPALETTE,
    SDL_HWSURFACE, SDL_NOFRAME, SDL_OPENGL, SDL_OPENGLBLIT, SDL_RESIZABLE,
};

use crate::FrameBuffer;

#[derive(Debug)]
pub struct Video;

impl Video {
    pub fn set_video_mode(
        &self,
        width: c_int,
        height: c_int,
        bits_per_pixel: Option<NonZeroU8>,
        flags: VideoModeFlags,
    ) -> Option<FrameBuffer> {
        unsafe {
            let surface_ptr = SDL_SetVideoMode(
                width,
                height,
                bits_per_pixel.map(|bpp| bpp.get()).unwrap_or(0).into(),
                flags.bits(),
            );
            NonNull::new(surface_ptr).map(|surface_ptr| FrameBuffer::from_ptr(surface_ptr))
        }
    }
}

bitflags! {
    pub struct VideoModeFlags: u32 {
        const SOFTWARE_SURFACE = SDL_HWSURFACE as u32;
        const HARDWARE_SURFACE = SDL_HWSURFACE as u32;
        const ASYNC_BLIT = SDL_ASYNCBLIT as u32;
        const ANY_FORMAT = SDL_ANYFORMAT as u32;
        const HARDWARE_PALETTE = SDL_HWPALETTE as u32;
        const DOUBLE_BUFFER = SDL_DOUBLEBUF as u32;
        const FULLSCREEN = SDL_FULLSCREEN as u32;
        const OPENGL = SDL_OPENGL as u32;
        const OPENGL_BLIT = SDL_OPENGLBLIT as u32;
        const RESIZABLE = SDL_RESIZABLE as u32;
        const NO_FRAME = SDL_NOFRAME as u32;
    }
}
