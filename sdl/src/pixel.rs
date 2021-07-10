use std::{convert::TryInto, marker::PhantomData, ptr::NonNull};

use sdl_sys::{SDL_MapRGB, SDL_PixelFormat};

use crate::UsableSurface;

#[derive(Debug, Clone, Copy)]
pub struct PixelFormatRef<'a> {
    inner: NonNull<SDL_PixelFormat>,
    _marker: PhantomData<&'a SDL_PixelFormat>,
}

impl PixelFormatRef<'_> {
    /// # Safety
    /// No mutable borrows must be alive.
    pub unsafe fn from_raw(pointer: NonNull<SDL_PixelFormat>) -> Self {
        Self {
            inner: pointer,
            _marker: PhantomData,
        }
    }

    pub fn bytes_per_pixel(&self) -> BytesPerPixel {
        use BytesPerPixel::*;

        let bpp = unsafe { self.inner.as_ref().BytesPerPixel };
        match bpp {
            1 => One,
            2 => Two,
            3 => Three,
            4 => Four,
            _ => panic!("SDL returned an invalid BytesPerPixel value"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BytesPerPixel {
    One,
    Two,
    Three,
    Four,
}

impl From<BytesPerPixel> for u8 {
    fn from(bpp: BytesPerPixel) -> Self {
        use BytesPerPixel::*;

        match bpp {
            One => 1,
            Two => 2,
            Three => 3,
            Four => 4,
        }
    }
}

#[derive(Debug)]
pub struct Pixels<'a, 'b, const FREEABLE: bool> {
    surface: &'a mut UsableSurface<'b, FREEABLE>,
    width: u16,
    height: u16,
}

impl<'a, 'b: 'a, const FREEABLE: bool> Pixels<'a, 'b, FREEABLE> {
    pub fn new(surface: &'a mut UsableSurface<'b, FREEABLE>) -> Self {
        let raw_surface = surface.raw();
        let width: u16 = raw_surface
            .width()
            .max(0)
            .try_into()
            .expect("cannot fit width in u16");

        let height: u16 = raw_surface
            .height()
            .max(0)
            .try_into()
            .expect("cannot fit width in u16");

        Self {
            surface,
            width,
            height,
        }
    }

    pub fn get(&self, x: u16, y: u16) -> Option<PixelRef<'_, 'a, 'b, FREEABLE>> {
        if x >= self.width || y >= self.height {
            return None;
        }

        let pos = pixel_offset(x, y, self.width);
        Some(PixelRef { pixels: self, pos })
    }

    fn raw_slice(&self) -> PixelsSlicePerBpp {
        use PixelsSlicePerBpp::*;

        let buffer_size = self.surface.buffer_size();
        let pixel_ptr = self.surface.raw().pixels();

        let bpp = self.surface.format().bytes_per_pixel();
        match bpp {
            BytesPerPixel::One => {
                One(unsafe { std::slice::from_raw_parts(pixel_ptr as *const u8, buffer_size) })
            }
            BytesPerPixel::Two => {
                Two(unsafe { std::slice::from_raw_parts(pixel_ptr as *const u16, buffer_size / 2) })
            }
            BytesPerPixel::Three => Three(unsafe {
                std::slice::from_raw_parts(pixel_ptr as *const [u8; 3], buffer_size / 3)
            }),
            BytesPerPixel::Four => Four(unsafe {
                std::slice::from_raw_parts(pixel_ptr as *const u32, buffer_size / 4)
            }),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PixelsSlicePerBpp<'a> {
    One(&'a [u8]),
    Two(&'a [u16]),
    Three(&'a [[u8; 3]]),
    Four(&'a [u32]),
}

#[derive(Debug, Clone)]
pub struct PixelRef<'a, 'b, 'c, const FREEABLE: bool> {
    pixels: &'a Pixels<'b, 'c, FREEABLE>,
    pos: usize,
}

impl<const FREEABLE: bool> PixelRef<'_, '_, '_, FREEABLE> {
    pub fn get(&self) -> u32 {
        use PixelsSlicePerBpp::*;

        match self.pixels.raw_slice() {
            One(slice) => slice[self.pos].into(),
            Two(slice) => slice[self.pos].into(),
            Three(slice) => {
                let [r, g, b] = slice[self.pos];
                let format = self.pixels.surface.raw().format();
                unsafe { SDL_MapRGB(format, r, g, b) }
            }
            Four(slice) => slice[self.pos],
        }
    }
}

fn pixel_offset(x: u16, y: u16, width: u16) -> usize {
    let x: usize = x.into();
    let y: usize = y.into();
    let width: usize = width.into();

    x * width + y
}
