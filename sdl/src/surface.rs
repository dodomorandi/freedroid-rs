mod lock;

use std::{convert::TryInto, ffi::c_void, os::raw::c_int, ptr::NonNull};

use sdl_sys::{
    SDL_FreeSurface, SDL_PixelFormat, SDL_Rect, SDL_Surface, SDL_ASYNCBLIT, SDL_HWSURFACE,
    SDL_RLEACCEL,
};

use crate::pixel::{PixelFormatRef, Pixels};

pub use self::lock::{ResultMaybeLockedSurface, SurfaceLockError, SurfaceLockGuard};

#[derive(Debug)]
pub struct GenericSurface<const FREEABLE: bool>(NonNull<SDL_Surface>);

impl<const FREEABLE: bool> GenericSurface<FREEABLE> {
    /// # Safety
    /// * `pointer` must point to a valid [SDL_Surface].
    /// * No live references to pointed data must exist.
    /// * The ownership of the pointed [SDL_Surface] is transferred to `GenericSurface`, therefore
    ///   the structure **must not** be freed.
    pub unsafe fn from_ptr(pointer: NonNull<SDL_Surface>) -> Self {
        Self(pointer)
    }

    pub fn must_lock(&self) -> bool {
        let surface = unsafe { self.0.as_ref() };
        surface.offset != 0
            && (surface.flags & (SDL_HWSURFACE as u32 | SDL_ASYNCBLIT as u32 | SDL_RLEACCEL as u32))
                != 0
    }

    pub fn lock(&mut self) -> ResultMaybeLockedSurface<'_, FREEABLE> {
        if self.must_lock() {
            ResultMaybeLockedSurface::Locked(SurfaceLockGuard::new(self))
        } else {
            ResultMaybeLockedSurface::Unlocked(UsableSurface(self))
        }
    }

    pub fn format(&self) -> PixelFormatRef {
        // SAFETY: format becomes null once SDL_FreeSurface is called, which is performed only on
        // drop.
        unsafe {
            let pointer = NonNull::new_unchecked(self.0.as_ref().format);
            PixelFormatRef::from_raw(pointer)
        }
    }

    pub fn raw(&self) -> UsableSurfaceRaw<'_, FREEABLE> {
        UsableSurfaceRaw(self)
    }

    pub fn height(&self) -> u16 {
        self.raw()
            .height()
            .max(0)
            .try_into()
            .expect("invalid SDL surface height for architecture")
    }

    pub fn width(&self) -> u16 {
        self.raw()
            .width()
            .max(0)
            .try_into()
            .expect("invalid SDL surface width for architecture")
    }

    pub fn as_ptr(&self) -> *const SDL_Surface {
        self.0.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut SDL_Surface {
        self.0.as_ptr()
    }
}

impl<const FREEABLE: bool> Drop for GenericSurface<FREEABLE> {
    fn drop(&mut self) {
        if FREEABLE {
            unsafe { SDL_FreeSurface(self.0.as_ptr()) }
        }
    }
}

/// A [GenericSurface] that must be freed on drop.
pub type Surface = GenericSurface<true>;

/// A [GenericSurface] that must not be freed on drop.
pub type FrameBuffer = GenericSurface<false>;

#[derive(Debug)]
pub struct UsableSurface<'a, const FREEABLE: bool>(&'a mut GenericSurface<FREEABLE>);

impl<'a, const FREEABLE: bool> UsableSurface<'a, FREEABLE> {
    pub fn format(&self) -> PixelFormatRef {
        self.0.format()
    }

    pub fn pixels(&mut self) -> Pixels<'_, 'a, FREEABLE> {
        Pixels::new(self)
    }

    pub fn raw(&self) -> UsableSurfaceRaw<'_, FREEABLE> {
        UsableSurfaceRaw(self.0)
    }

    pub fn height(&self) -> u16 {
        self.0.height()
    }

    pub fn width(&self) -> u16 {
        self.0.width()
    }

    pub fn buffer_size(&self) -> usize {
        let height: usize = self.height().into();
        let pitch = usize::from(self.raw().pitch());

        height
            .checked_mul(pitch)
            .expect("SDL surface with a buffer too big")
    }
}

#[derive(Debug)]
pub struct UsableSurfaceRaw<'a, const FREEABLE: bool>(&'a GenericSurface<FREEABLE>);

impl<'a, const FREEABLE: bool> UsableSurfaceRaw<'a, FREEABLE> {
    pub fn flags(&self) -> u32 {
        unsafe { self.0 .0.as_ref().flags }
    }

    pub fn format(&self) -> *const SDL_PixelFormat {
        unsafe { self.0 .0.as_ref().format }
    }

    pub fn width(&self) -> c_int {
        unsafe { self.0 .0.as_ref().w }
    }

    pub fn height(&self) -> c_int {
        unsafe { self.0 .0.as_ref().h }
    }

    pub fn pitch(&self) -> u16 {
        unsafe { self.0 .0.as_ref().pitch }
    }

    pub fn pixels(&self) -> *const c_void {
        unsafe { self.0 .0.as_ref().pixels }
    }

    pub fn clip_rect(&self) -> &SDL_Rect {
        unsafe { &self.0 .0.as_ref().clip_rect }
    }

    pub fn refcount(&self) -> c_int {
        unsafe { self.0 .0.as_ref().refcount }
    }
}
