use std::{
    cell::Cell,
    ffi::CStr,
    num::NonZeroU8,
    ops::Not,
    os::raw::{c_char, c_int},
    ptr::{null_mut, NonNull},
};

use bitflags::bitflags;
use sdl_sys::{
    SDL_SetGamma, SDL_SetVideoMode, SDL_VideoDriverName, SDL_WM_SetCaption, SDL_WM_SetIcon,
    SDL_ANYFORMAT, SDL_ASYNCBLIT, SDL_DOUBLEBUF, SDL_FULLSCREEN, SDL_HWPALETTE, SDL_HWSURFACE,
    SDL_NOFRAME, SDL_OPENGL, SDL_OPENGLBLIT, SDL_RESIZABLE,
};

use crate::{FrameBuffer, Surface};

#[derive(Debug)]
pub struct Video {
    set_video_mode_called: Cell<bool>,
}

impl Video {
    pub(crate) const fn new() -> Self {
        Self {
            set_video_mode_called: Cell::new(false),
        }
    }

    pub fn set_video_mode(
        &self,
        width: c_int,
        height: c_int,
        bits_per_pixel: Option<NonZeroU8>,
        flags: VideoModeFlags,
    ) -> Option<FrameBuffer> {
        self.set_video_mode_called.set(true);
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

    #[must_use = "success/failure is given as true/false"]
    pub fn set_gamma(&self, red: f32, green: f32, blue: f32) -> bool {
        unsafe { SDL_SetGamma(red, green, blue) == 0 }
    }

    pub fn get_driver_name<'a>(&self, buffer: &'a mut [u8]) -> Option<&'a CStr> {
        if buffer.is_empty() {
            return None;
        }

        let len = buffer.len().try_into().unwrap_or(c_int::MAX);
        let pointer = unsafe { SDL_VideoDriverName(buffer.as_mut_ptr() as *mut c_char, len) };
        pointer
            .is_null()
            .not()
            .then(|| unsafe { CStr::from_ptr(buffer.as_ptr() as *const c_char) })
    }

    pub fn window_manager(&self) -> WindowManager<'_> {
        WindowManager(self)
    }
}

#[derive(Debug)]
pub struct WindowManager<'a>(&'a Video);

impl WindowManager<'_> {
    pub fn set_caption(&self, title: &CStr, icon: &CStr) {
        unsafe { SDL_WM_SetCaption(title.as_ptr(), icon.as_ptr()) }
    }

    pub fn set_icon(&self, icon: &mut Surface, mask: Option<&mut [u8]>) {
        if self.0.set_video_mode_called.get() {
            panic!("SDL video wm set_icon must be called before set_video_mode");
        }

        if let Some(mask) = mask.as_ref() {
            assert_eq!(mask.len(), (icon.height() * (icon.width() / 8)).into());
        }

        unsafe {
            SDL_WM_SetIcon(
                icon.as_mut_ptr(),
                mask.map(|mask| mask.as_mut_ptr()).unwrap_or(null_mut()),
            )
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