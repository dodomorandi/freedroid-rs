use std::{ffi::CStr, marker::PhantomData, mem, ptr::NonNull};

use cstr::cstr;
use sdl_sys::{SDL_FreeRW, SDL_RWFromFile, SDL_RWops};

#[derive(Debug)]
pub struct RwOps<'a> {
    inner: NonNull<SDL_RWops>,
    _marker: PhantomData<&'a ()>,
}

impl RwOps<'_> {
    pub fn from_c_str_path(path: &CStr, mode: Mode) -> Option<Self> {
        let ret = unsafe { SDL_RWFromFile(path.as_ptr(), mode.to_c_str().as_ptr()) };
        NonNull::new(ret).map(|inner| Self {
            inner,
            _marker: PhantomData,
        })
    }

    pub fn inner(&self) -> NonNull<SDL_RWops> {
        self.inner
    }

    pub fn into_inner(self) -> NonNull<SDL_RWops> {
        let Self { inner, .. } = self;
        mem::forget(self);
        inner
    }

    pub fn close(mut self) -> Result<bool, Self> {
        let inner = unsafe { self.inner.as_mut() };
        if let Some(close_fn) = inner.close {
            let ret = unsafe { close_fn(self.inner.as_ptr()) };
            unsafe {
                SDL_FreeRW(self.inner.as_ptr());
            }
            Ok(ret == 0)
        } else {
            Err(self)
        }
    }
}

impl Drop for RwOps<'_> {
    fn drop(&mut self) {
        let inner = unsafe { self.inner.as_mut() };
        if let Some(close_fn) = inner.close {
            unsafe { close_fn(self.inner.as_ptr()) };
        }

        unsafe {
            SDL_FreeRW(self.inner.as_ptr());
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Read,
    Write,
    ReadWrite,
    Append,
    AppendRead,
    Truncate,
}

impl Mode {
    pub fn to_c_str(self) -> &'static CStr {
        use Mode::*;

        match self {
            Read => cstr!("r"),
            Write => cstr!("w"),
            ReadWrite => cstr!("r+"),
            Append => cstr!("a"),
            AppendRead => cstr!("a+"),
            Truncate => cstr!("w+"),
        }
    }
}
