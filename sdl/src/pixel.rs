use std::{fmt, marker::PhantomData, ops::Not, ptr::NonNull};

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

    pub fn as_ptr(&self) -> *const SDL_PixelFormat {
        self.inner.as_ptr()
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

    pub fn get_mut(&mut self, x: u16, y: u16) -> Option<PixelMut<'_, 'a, 'b, FREEABLE>> {
        if x >= self.width || y >= self.height {
            return None;
        }

        let pos = pixel_offset(x, y, self.width);
        Some(PixelMut { pixels: self, pos })
    }

    pub fn set(&mut self, x: u16, y: u16, pixel: u32) -> Result<(), InvalidPixel> {
        self.get_mut(x, y).ok_or(InvalidPixel)?.set(pixel);
        Ok(())
    }
}

macro_rules! impl_pixel_raw_slice {
    (@inner $name:ident, $ty:ident, $from_raw_parts:ident) => {
        impl<'a, 'b: 'a, const FREEABLE: bool> Pixels<'a, 'b, FREEABLE> {
            fn $name(&self) -> $ty {
                use $ty::*;

                let buffer_size = self.surface.buffer_size();
                let pixel_ptr = self.surface.raw().pixels();

                let bpp = self.surface.format().bytes_per_pixel();
                match bpp {
                    BytesPerPixel::One => {
                        One(unsafe { std::slice::$from_raw_parts(pixel_ptr as _, buffer_size) })
                    }
                    BytesPerPixel::Two => {
                        Two(unsafe { std::slice::$from_raw_parts(pixel_ptr as _, buffer_size / 2) })
                    }
                    BytesPerPixel::Three => Three(unsafe {
                        std::slice::$from_raw_parts(pixel_ptr as _, buffer_size / 3)
                    }),
                    BytesPerPixel::Four => Four(unsafe {
                        std::slice::$from_raw_parts(pixel_ptr as _, buffer_size / 4)
                    }),
                }
            }
        }
    };

    ($name:ident, $ty:ident, mut) => {
        impl_pixel_raw_slice!(@inner $name, $ty, from_raw_parts_mut);
    };

    ($name:ident, $ty:ident) => {
        impl_pixel_raw_slice!(@inner $name, $ty, from_raw_parts);
    };
}

impl_pixel_raw_slice!(raw_slice, PixelsSlicePerBpp);
impl_pixel_raw_slice!(raw_slice_mut, PixelsSliceMutPerBpp, mut);

#[derive(Clone, Copy, PartialEq, Eq)]
enum PixelsSlicePerBpp<'a> {
    One(&'a [u8]),
    Two(&'a [u16]),
    Three(&'a [[u8; 3]]),
    Four(&'a [u32]),
}

#[derive(PartialEq, Eq)]
enum PixelsSliceMutPerBpp<'a> {
    One(&'a mut [u8]),
    Two(&'a mut [u16]),
    Three(&'a mut [[u8; 3]]),
    Four(&'a mut [u32]),
}

#[derive(Debug, Clone)]
pub struct PixelRef<'a, 'b, 'c, const FREEABLE: bool> {
    pixels: &'a Pixels<'b, 'c, FREEABLE>,
    pos: usize,
}

#[derive(Debug)]
pub struct PixelMut<'a, 'b, 'c, const FREEABLE: bool> {
    pixels: &'a mut Pixels<'b, 'c, FREEABLE>,
    pos: usize,
}

macro_rules! impl_pixel_ref {
    ($ty:ident) => {
        impl<const FREEABLE: bool> $ty<'_, '_, '_, FREEABLE> {
            pub fn get(&self) -> u32 {
                raw_get_pixel(self.pixels.raw_slice(), self.pos, || {
                    self.pixels.surface.raw().format()
                })
            }

            pub fn rgba(&self) -> [u8; 4] {
                let pixel = self.get();
                let format = self.pixels.surface.raw().format();
                assert!(format.is_null().not());
                let mut rgba = [0; 4];
                unsafe {
                    sdl_sys::SDL_GetRGBA(
                        pixel,
                        format,
                        &mut rgba[0],
                        &mut rgba[1],
                        &mut rgba[2],
                        &mut rgba[3],
                    );
                }
                rgba
            }
        }
    };
}

impl_pixel_ref!(PixelRef);
impl_pixel_ref!(PixelMut);

impl<const FREEABLE: bool> PixelMut<'_, '_, '_, FREEABLE> {
    pub fn set(&mut self, value: u32) {
        raw_set_pixel(self.pixels.raw_slice_mut(), self.pos, value);
    }
}

#[inline]
fn raw_get_pixel<F>(pixels_slice: PixelsSlicePerBpp, pos: usize, get_pixel_format: F) -> u32
where
    F: FnOnce() -> *const SDL_PixelFormat,
{
    use PixelsSlicePerBpp::*;

    match pixels_slice {
        One(slice) => slice[pos].into(),
        Two(slice) => slice[pos].into(),
        Three(slice) => {
            let [r, g, b] = slice[pos];
            let format = get_pixel_format();
            unsafe { SDL_MapRGB(format, r, g, b) }
        }
        Four(slice) => slice[pos],
    }
}

#[inline]
fn raw_set_pixel(pixels_slice: PixelsSliceMutPerBpp, pos: usize, value: u32) {
    use PixelsSliceMutPerBpp::*;

    match pixels_slice {
        One(slice) => slice[pos] = value as u8,
        Two(slice) => slice[pos] = value as u16,
        Three(slice) => {
            let [r, g, b] = &mut slice[pos];
            let value = value.to_ne_bytes();
            *r = value[0];
            *g = value[1];
            *b = value[2];
        }
        Four(slice) => slice[pos] = value,
    }
}

fn pixel_offset(x: u16, y: u16, width: u16) -> usize {
    let x: usize = x.into();
    let y: usize = y.into();
    let width: usize = width.into();

    y * width + x
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidPixel;

impl fmt::Display for InvalidPixel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Invalid pixel coordinate.")
    }
}

#[cfg(test)]
mod tests {
    use std::ptr::null_mut;

    use super::*;

    #[test]
    fn set_pixel_3bpp() {
        const VALUE: u32 = 0x123456;

        let mut data = [[0; 3]];
        raw_set_pixel(PixelsSliceMutPerBpp::Three(&mut data), 0, VALUE);

        let pixel_format = SDL_PixelFormat {
            palette: null_mut(),
            BitsPerPixel: 24,
            BytesPerPixel: 3,
            Rloss: 0,
            Gloss: 0,
            Bloss: 0,
            Aloss: 0,
            Rshift: 0,
            Gshift: 8,
            Bshift: 16,
            Ashift: 0,
            Rmask: 0xFF,
            Gmask: 0xFF00,
            Bmask: 0xFF0000,
            Amask: 0,
            colorkey: 0,
            alpha: 0,
        };
        let pixel = raw_get_pixel(PixelsSlicePerBpp::Three(&data), 0, || &pixel_format);
        assert_eq!(pixel, VALUE);
    }
}
