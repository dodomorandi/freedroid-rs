use std::{
    ffi::{c_void, CStr},
    marker::PhantomData,
    mem,
    ops::Not,
    os::raw::c_int,
    path::{Path, PathBuf},
    pin::Pin,
    ptr::NonNull,
};

use cstr::cstr;
use log::error;
use sdl_sys::{IMG_Load_RW, IMG_isJPG, SDL_FreeRW, SDL_RWFromFile, SDL_RWFromMem, SDL_RWops};

use crate::{sealed::Sealed, Surface};

#[derive(Debug)]
pub struct RwOps<'a> {
    inner: NonNull<SDL_RWops>,
    _marker: PhantomData<&'a ()>,
}

impl<'a> RwOps<'a> {
    #[cfg(unix)]
    pub fn from_pathbuf(path: PathBuf, mode: Mode) -> Option<Self> {
        use std::os::unix::ffi::OsStringExt;

        let mut path = path.into_os_string().into_vec();
        path.push(0);
        let c_path = CStr::from_bytes_with_nul(&path).ok()?;
        Self::from_c_str_path(c_path, mode)
    }

    #[cfg(not(unix))]
    pub fn from_pathbuf(path: PathBuf, mode: Mode) -> Option<Self> {
        Self::from_path(&path, mode)
    }

    pub fn from_path(path: &Path, mode: Mode) -> Option<Self> {
        let mut path = path.to_string_lossy().into_owned().into_bytes();
        path.push(0);
        let c_path = CStr::from_bytes_with_nul(&path).ok()?;
        Self::from_c_str_path(c_path, mode)
    }

    pub fn from_c_str_path(path: &CStr, mode: Mode) -> Option<Self> {
        let ret = unsafe { SDL_RWFromFile(path.as_ptr(), mode.to_c_str().as_ptr()) };
        NonNull::new(ret).map(|inner| Self {
            inner,
            _marker: PhantomData,
        })
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

    pub fn image_load(&self) -> Option<Surface<'a>> {
        unsafe { image_load(self.inner) }
    }
}

impl Drop for RwOps<'_> {
    fn drop(&mut self) {
        unsafe {
            if close_rw_ops_on_drop(self.inner).not() {
                error!("SDL rw ops has not been closed successfully");
            }
        }
    }
}

/// # Safety
///
/// If `rw_ops` is bound to the lifetime `'sdl` of an SDL instance, the output should not outlive
/// `'sdl`.
unsafe fn image_load<'a>(rw_ops: NonNull<SDL_RWops>) -> Option<Surface<'a>> {
    NonNull::new(unsafe { IMG_Load_RW(rw_ops.as_ptr(), 0) })
        .map(|ptr| unsafe { Surface::from_ptr(ptr) })
}

const RW_SEEK_SET: isize = 0;
const RW_SEEK_CUR: isize = 1;
const RW_SEEK_END: isize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Whence {
    Set = RW_SEEK_SET,
    Cur = RW_SEEK_CUR,
    End = RW_SEEK_END,
}

impl From<Whence> for c_int {
    fn from(whence: Whence) -> Self {
        use Whence::*;
        (match whence {
            Set => RW_SEEK_SET,
            Cur => RW_SEEK_CUR,
            End => RW_SEEK_END,
        }) as c_int
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
pub enum RwSeekError {
    #[error("offset too large")]
    OffsetTooLarge,

    #[error("SDL error")]
    Sdl,
}

// # Safety
//
// - rw_ops must point to valid data.
// - This should only be called on drop.
#[must_use = "returns whether the closing has been successful"]
unsafe fn close_rw_ops_on_drop(rw_ops: NonNull<SDL_RWops>) -> bool {
    match unsafe { rw_ops.as_ref().close } {
        Some(close_fn) => unsafe { close_fn(rw_ops.as_ptr()) == 0 },
        None => {
            unsafe {
                SDL_FreeRW(rw_ops.as_ptr());
            }
            true
        }
    }
}

type Buffer = Pin<Box<[u8]>>;

#[derive(Debug)]
pub struct RwOpsOwned {
    _buffer: Buffer,
    rw_ops: NonNull<SDL_RWops>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
pub enum RwOpsOwnedError {
    #[error("buffer is too big to fit into c_int")]
    BufferTooBig,

    #[error("error from SDL")]
    Sdl,
}

impl RwOpsOwned {
    pub fn from_buffer(mut buffer: Pin<Box<[u8]>>) -> Result<Self, (RwOpsOwnedError, Buffer)> {
        // Safety:
        //
        // - Buffer is pinned, therefore it won't be moved.
        // - SDL_RWFromMem expects a memory buffer, using a pointer to u8 is safe.
        // - rw_ops is closed on drop
        let ptr = unsafe {
            let len = match buffer.len().try_into() {
                Ok(len) => len,
                Err(_) => return Err((RwOpsOwnedError::BufferTooBig, buffer)),
            };
            SDL_RWFromMem(buffer.as_mut_ptr() as *mut c_void, len)
        };

        match NonNull::new(ptr) {
            Some(rw_ops) => Ok(Self {
                _buffer: buffer,
                rw_ops,
            }),
            None => Err((RwOpsOwnedError::Sdl, buffer)),
        }
    }

    pub fn image_load(&self) -> Option<Surface<'static>> {
        unsafe { image_load(self.rw_ops) }
    }
}

impl Drop for RwOpsOwned {
    fn drop(&mut self) {
        unsafe {
            if close_rw_ops_on_drop(self.rw_ops).not() {
                error!("SDL rw ops has not been closed successfully");
            }
        }
    }
}

pub trait RwOpsCapability: Sized + Sealed {
    fn as_inner(&self) -> NonNull<SDL_RWops>;

    fn seek(&self, offset: i64, whence: Whence) -> Result<u64, RwSeekError> {
        let offset = offset.try_into().map_err(|_| RwSeekError::OffsetTooLarge)?;

        let rw_ops = self.as_inner().as_ptr();
        let seek_fn = unsafe { (*rw_ops).seek };
        debug_assert!(seek_fn.is_some());
        let seek_fn = unsafe { seek_fn.unwrap_unchecked() };
        unsafe { seek_fn(rw_ops, offset, whence.into()) }
            .try_into()
            .map_err(|_| RwSeekError::Sdl)
    }

    fn as_ptr(&self) -> *const SDL_RWops {
        self.as_inner().as_ptr()
    }

    fn as_mut_ptr(&mut self) -> *mut SDL_RWops {
        self.as_inner().as_ptr()
    }

    fn into_inner(self) -> NonNull<SDL_RWops> {
        let inner = self.as_inner();
        mem::forget(self);
        inner
    }

    #[allow(clippy::wrong_self_convention)]
    fn is_jpg(&mut self) -> bool {
        unsafe { IMG_isJPG(self.as_inner().as_ptr()) != 0 }
    }
}

impl Sealed for RwOps<'_> {}
impl Sealed for RwOpsOwned {}

impl RwOpsCapability for RwOps<'_> {
    fn as_inner(&self) -> NonNull<SDL_RWops> {
        self.inner
    }
}

impl RwOpsCapability for RwOpsOwned {
    fn as_inner(&self) -> NonNull<SDL_RWops> {
        self.rw_ops
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadWriteMode {
    Read,
    Write,
    ReadWrite,
    Append,
    AppendRead,
    Truncate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mode {
    pub rw_mode: ReadWriteMode,
    pub binary: bool,
}

impl From<ReadWriteMode> for Mode {
    fn from(rw_mode: ReadWriteMode) -> Self {
        Self {
            rw_mode,
            binary: false,
        }
    }
}

impl Mode {
    #[must_use]
    pub fn with_binary(self, binary: bool) -> Self {
        Self {
            rw_mode: self.rw_mode,
            binary,
        }
    }

    pub fn to_c_str(self) -> &'static CStr {
        use ReadWriteMode::*;

        match (self.rw_mode, self.binary) {
            (Read, false) => cstr!("r"),
            (Read, true) => cstr!("rb"),
            (Write, false) => cstr!("w"),
            (Write, true) => cstr!("wb"),
            (ReadWrite, false) => cstr!("r+"),
            (ReadWrite, true) => cstr!("r+b"),
            (Append, false) => cstr!("a"),
            (Append, true) => cstr!("ab"),
            (AppendRead, false) => cstr!("a+"),
            (AppendRead, true) => cstr!("a+b"),
            (Truncate, false) => cstr!("w+"),
            (Truncate, true) => cstr!("w+b"),
        }
    }
}
