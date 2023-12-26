use core::fmt;
use std::ops::{Deref, DerefMut};

use sdl_sys::{SDL_LockSurface, SDL_UnlockSurface};

use super::{Generic, Usable};

#[derive(Debug)]
pub struct Guard<'a, 'sdl, const FREEABLE: bool>(Usable<'a, 'sdl, FREEABLE>);

impl<'a, 'sdl, const FREEABLE: bool> Guard<'a, 'sdl, FREEABLE> {
    pub fn new(surface: &'a mut Generic<'sdl, FREEABLE>) -> Result<Self, Error> {
        let result = unsafe { SDL_LockSurface(surface.pointer.as_ptr()) };
        match result {
            0 => Ok(Self(Usable(surface))),
            -1 => Err(Error),
            _ => unreachable!(),
        }
    }
}

impl<'a, 'sdl, const FREEABLE: bool> AsRef<Usable<'a, 'sdl, FREEABLE>>
    for Guard<'a, 'sdl, FREEABLE>
{
    fn as_ref(&self) -> &Usable<'a, 'sdl, FREEABLE> {
        &self.0
    }
}

impl<'a, 'sdl, const FREEABLE: bool> AsMut<Usable<'a, 'sdl, FREEABLE>>
    for Guard<'a, 'sdl, FREEABLE>
{
    fn as_mut(&mut self) -> &mut Usable<'a, 'sdl, FREEABLE> {
        &mut self.0
    }
}

impl<'a, 'sdl, const FREEABLE: bool> Deref for Guard<'a, 'sdl, FREEABLE> {
    type Target = Usable<'a, 'sdl, FREEABLE>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, const FREEABLE: bool> DerefMut for Guard<'a, '_, FREEABLE> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const FREEABLE: bool> Drop for Guard<'_, '_, FREEABLE> {
    fn drop(&mut self) {
        unsafe { SDL_UnlockSurface(self.0 .0.pointer.as_ptr()) }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Error;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Unable to lock SDL surface")
    }
}

#[derive(Debug)]
pub enum ResultMaybeLocked<'a, 'sdl, const FREEABLE: bool> {
    Locked(Result<Guard<'a, 'sdl, FREEABLE>, Error>),
    Unlocked(Usable<'a, 'sdl, FREEABLE>),
}

impl<'a, 'sdl, const FREEABLE: bool> ResultMaybeLocked<'a, 'sdl, FREEABLE> {
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
    Locked(Guard<'a, 'sdl, FREEABLE>),
    Unlocked(Usable<'a, 'sdl, FREEABLE>),
}

impl<'a, 'sdl, const FREEABLE: bool> AsRef<Usable<'a, 'sdl, FREEABLE>>
    for MaybeLockedSurface<'a, 'sdl, FREEABLE>
{
    fn as_ref(&self) -> &Usable<'a, 'sdl, FREEABLE> {
        match self {
            Self::Locked(guard) => guard.as_ref(),
            Self::Unlocked(surface) => surface,
        }
    }
}

impl<'a, 'sdl, const FREEABLE: bool> AsMut<Usable<'a, 'sdl, FREEABLE>>
    for MaybeLockedSurface<'a, 'sdl, FREEABLE>
{
    fn as_mut(&mut self) -> &mut Usable<'a, 'sdl, FREEABLE> {
        match self {
            Self::Locked(guard) => guard.as_mut(),
            Self::Unlocked(surface) => surface,
        }
    }
}

impl<'a, 'sdl, const FREEABLE: bool> Deref for MaybeLockedSurface<'a, 'sdl, FREEABLE> {
    type Target = Usable<'a, 'sdl, FREEABLE>;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a, 'sdl, const FREEABLE: bool> DerefMut for MaybeLockedSurface<'a, 'sdl, FREEABLE> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}
