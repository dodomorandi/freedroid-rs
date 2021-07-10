use core::fmt;
use std::ops::{Deref, DerefMut};

use sdl_sys::{SDL_LockSurface, SDL_UnlockSurface};

use super::{GenericSurface, UsableSurface};

#[derive(Debug)]
pub struct SurfaceLockGuard<'a, const FREEABLE: bool>(UsableSurface<'a, FREEABLE>);

impl<'a, const FREEABLE: bool> SurfaceLockGuard<'a, FREEABLE> {
    pub fn new(surface: &'a mut GenericSurface<FREEABLE>) -> Result<Self, SurfaceLockError> {
        let result = unsafe { SDL_LockSurface(surface.0.as_ptr()) };
        match result {
            0 => Ok(Self(UsableSurface(surface))),
            -1 => Err(SurfaceLockError),
            _ => unreachable!(),
        }
    }
}

impl<'a, const FREEABLE: bool> AsRef<UsableSurface<'a, FREEABLE>>
    for SurfaceLockGuard<'a, FREEABLE>
{
    fn as_ref(&self) -> &UsableSurface<'a, FREEABLE> {
        &self.0
    }
}

impl<'a, const FREEABLE: bool> AsMut<UsableSurface<'a, FREEABLE>>
    for SurfaceLockGuard<'a, FREEABLE>
{
    fn as_mut(&mut self) -> &mut UsableSurface<'a, FREEABLE> {
        &mut self.0
    }
}

impl<'a, const FREEABLE: bool> Deref for SurfaceLockGuard<'a, FREEABLE> {
    type Target = UsableSurface<'a, FREEABLE>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, const FREEABLE: bool> DerefMut for SurfaceLockGuard<'a, FREEABLE> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const FREEABLE: bool> Drop for SurfaceLockGuard<'_, FREEABLE> {
    fn drop(&mut self) {
        unsafe { SDL_UnlockSurface(self.0 .0 .0.as_ptr()) }
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
pub enum ResultMaybeLockedSurface<'a, const FREEABLE: bool> {
    Locked(Result<SurfaceLockGuard<'a, FREEABLE>, SurfaceLockError>),
    Unlocked(UsableSurface<'a, FREEABLE>),
}

impl<'a, const FREEABLE: bool> ResultMaybeLockedSurface<'a, FREEABLE> {
    pub fn unwrap(self) -> MaybeLockedSurface<'a, FREEABLE> {
        match self {
            Self::Locked(result) => MaybeLockedSurface::Locked(result.unwrap()),
            Self::Unlocked(surface) => MaybeLockedSurface::Unlocked(surface),
        }
    }
}

#[derive(Debug)]
pub enum MaybeLockedSurface<'a, const FREEABLE: bool> {
    Locked(SurfaceLockGuard<'a, FREEABLE>),
    Unlocked(UsableSurface<'a, FREEABLE>),
}

impl<'a, const FREEABLE: bool> AsRef<UsableSurface<'a, FREEABLE>>
    for MaybeLockedSurface<'a, FREEABLE>
{
    fn as_ref(&self) -> &UsableSurface<'a, FREEABLE> {
        match self {
            Self::Locked(guard) => guard.as_ref(),
            Self::Unlocked(surface) => surface,
        }
    }
}

impl<'a, const FREEABLE: bool> AsMut<UsableSurface<'a, FREEABLE>>
    for MaybeLockedSurface<'a, FREEABLE>
{
    fn as_mut(&mut self) -> &mut UsableSurface<'a, FREEABLE> {
        match self {
            Self::Locked(guard) => guard.as_mut(),
            Self::Unlocked(surface) => surface,
        }
    }
}

impl<'a, const FREEABLE: bool> Deref for MaybeLockedSurface<'a, FREEABLE> {
    type Target = UsableSurface<'a, FREEABLE>;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a, const FREEABLE: bool> DerefMut for MaybeLockedSurface<'a, FREEABLE> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}
