use core::fmt;
use std::ops::{Deref, DerefMut};

use sdl_sys::{SDL_LockSurface, SDL_UnlockSurface};

use super::{GenericSurface, UsableSurface};

#[derive(Debug)]
pub struct SurfaceLockGuard<'a, 'sdl, const FREEABLE: bool>(UsableSurface<'a, 'sdl, FREEABLE>);

impl<'a, 'sdl, const FREEABLE: bool> SurfaceLockGuard<'a, 'sdl, FREEABLE> {
    pub fn new(surface: &'a mut GenericSurface<'sdl, FREEABLE>) -> Result<Self, SurfaceLockError> {
        let result = unsafe { SDL_LockSurface(surface.pointer.as_ptr()) };
        match result {
            0 => Ok(Self(UsableSurface(surface))),
            -1 => Err(SurfaceLockError),
            _ => unreachable!(),
        }
    }
}

impl<'a, 'sdl, const FREEABLE: bool> AsRef<UsableSurface<'a, 'sdl, FREEABLE>>
    for SurfaceLockGuard<'a, 'sdl, FREEABLE>
{
    fn as_ref(&self) -> &UsableSurface<'a, 'sdl, FREEABLE> {
        &self.0
    }
}

impl<'a, 'sdl, const FREEABLE: bool> AsMut<UsableSurface<'a, 'sdl, FREEABLE>>
    for SurfaceLockGuard<'a, 'sdl, FREEABLE>
{
    fn as_mut(&mut self) -> &mut UsableSurface<'a, 'sdl, FREEABLE> {
        &mut self.0
    }
}

impl<'a, 'sdl, const FREEABLE: bool> Deref for SurfaceLockGuard<'a, 'sdl, FREEABLE> {
    type Target = UsableSurface<'a, 'sdl, FREEABLE>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, const FREEABLE: bool> DerefMut for SurfaceLockGuard<'a, '_, FREEABLE> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const FREEABLE: bool> Drop for SurfaceLockGuard<'_, '_, FREEABLE> {
    fn drop(&mut self) {
        unsafe { SDL_UnlockSurface(self.0 .0.pointer.as_ptr()) }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceLockError;

impl fmt::Display for SurfaceLockError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Unable to lock SDL surface")
    }
}

#[derive(Debug)]
pub enum ResultMaybeLockedSurface<'a, 'sdl, const FREEABLE: bool> {
    Locked(Result<SurfaceLockGuard<'a, 'sdl, FREEABLE>, SurfaceLockError>),
    Unlocked(UsableSurface<'a, 'sdl, FREEABLE>),
}

impl<'a, 'sdl, const FREEABLE: bool> ResultMaybeLockedSurface<'a, 'sdl, FREEABLE> {
    #[must_use]
    pub fn unwrap(self) -> MaybeLockedSurface<'a, 'sdl, FREEABLE> {
        match self {
            Self::Locked(result) => MaybeLockedSurface::Locked(result.unwrap()),
            Self::Unlocked(surface) => MaybeLockedSurface::Unlocked(surface),
        }
    }
}

#[derive(Debug)]
pub enum MaybeLockedSurface<'a, 'sdl, const FREEABLE: bool> {
    Locked(SurfaceLockGuard<'a, 'sdl, FREEABLE>),
    Unlocked(UsableSurface<'a, 'sdl, FREEABLE>),
}

impl<'a, 'sdl, const FREEABLE: bool> AsRef<UsableSurface<'a, 'sdl, FREEABLE>>
    for MaybeLockedSurface<'a, 'sdl, FREEABLE>
{
    fn as_ref(&self) -> &UsableSurface<'a, 'sdl, FREEABLE> {
        match self {
            Self::Locked(guard) => guard.as_ref(),
            Self::Unlocked(surface) => surface,
        }
    }
}

impl<'a, 'sdl, const FREEABLE: bool> AsMut<UsableSurface<'a, 'sdl, FREEABLE>>
    for MaybeLockedSurface<'a, 'sdl, FREEABLE>
{
    fn as_mut(&mut self) -> &mut UsableSurface<'a, 'sdl, FREEABLE> {
        match self {
            Self::Locked(guard) => guard.as_mut(),
            Self::Unlocked(surface) => surface,
        }
    }
}

impl<'a, 'sdl, const FREEABLE: bool> Deref for MaybeLockedSurface<'a, 'sdl, FREEABLE> {
    type Target = UsableSurface<'a, 'sdl, FREEABLE>;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a, 'sdl, const FREEABLE: bool> DerefMut for MaybeLockedSurface<'a, 'sdl, FREEABLE> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}
