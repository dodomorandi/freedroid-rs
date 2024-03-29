use std::{
    fmt::{self, Display, Write},
    marker::PhantomData,
    ops::Not,
    ptr::NonNull,
};

use sdl_sys::{SDL_MapRGB, SDL_MapRGBA, SDL_PixelFormat};

use crate::surface;

#[derive(Debug, Clone, Copy)]
pub struct FormatRef<'a> {
    inner: NonNull<SDL_PixelFormat>,
    _marker: PhantomData<&'a SDL_PixelFormat>,
}

impl FormatRef<'_> {
    /// # Safety
    /// No mutable borrows must be alive.
    #[must_use]
    pub unsafe fn from_raw(pointer: NonNull<SDL_PixelFormat>) -> Self {
        Self {
            inner: pointer,
            _marker: PhantomData,
        }
    }

    #[must_use]
    pub fn as_ptr(&self) -> *const SDL_PixelFormat {
        self.inner.as_ptr()
    }

    #[must_use]
    pub fn bytes_per_pixel(self) -> BytesPerPixel {
        use BytesPerPixel::{Four, One, Three, Two};

        let bpp = unsafe { self.inner.as_ref().BytesPerPixel };
        match bpp {
            1 => One,
            2 => Two,
            3 => Three,
            4 => Four,
            _ => panic!("SDL returned an invalid BytesPerPixel value"),
        }
    }

    #[must_use]
    pub fn map_rgb(self, red: u8, green: u8, blue: u8) -> Pixel {
        let result = unsafe { SDL_MapRGB(self.inner.as_ptr(), red, green, blue) };
        Pixel(result)
    }

    #[must_use]
    pub fn map_rgba(self, red: u8, green: u8, blue: u8, alpha: u8) -> Pixel {
        let result = unsafe { SDL_MapRGBA(self.inner.as_ptr(), red, green, blue, alpha) };
        Pixel(result)
    }

    #[must_use]
    pub fn has_alpha(self) -> bool {
        let format = unsafe { self.inner.as_ref() };
        format.Amask != 0
    }

    #[must_use]
    pub fn bits_per_pixel(self) -> BitsPerPixel {
        let bpp = unsafe { self.inner.as_ref().BitsPerPixel };
        match bpp {
            8 => BitsPerPixel::Eight,
            15 => BitsPerPixel::Fifteen,
            16 => BitsPerPixel::Sixteen,
            24 => BitsPerPixel::Twentyfour,
            32 => BitsPerPixel::Thirtytwo,
            _ => panic!("SDL returned an invalid BitsPerPixel value"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pixel(pub(crate) u32);

impl Pixel {
    #[must_use]
    pub const fn black() -> Self {
        Self(0)
    }

    #[must_use]
    pub const fn from_u8(color: u8) -> Self {
        Self(color as u32)
    }
}

impl From<u32> for Pixel {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BytesPerPixel {
    One,
    Two,
    Three,
    Four,
}

impl From<BytesPerPixel> for u8 {
    fn from(bpp: BytesPerPixel) -> Self {
        use BytesPerPixel::{Four, One, Three, Two};

        match bpp {
            One => 1,
            Two => 2,
            Three => 3,
            Four => 4,
        }
    }
}

impl Display for BytesPerPixel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BytesPerPixel::One => f.write_char('1'),
            BytesPerPixel::Two => f.write_char('2'),
            BytesPerPixel::Three => f.write_char('3'),
            BytesPerPixel::Four => f.write_char('4'),
        }
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BitsPerPixel {
    Eight,
    Fifteen,
    Sixteen,
    Twentyfour,
    Thirtytwo,
}

impl From<BitsPerPixel> for u8 {
    fn from(bpp: BitsPerPixel) -> Self {
        match bpp {
            BitsPerPixel::Eight => 8,
            BitsPerPixel::Fifteen => 15,
            BitsPerPixel::Sixteen => 16,
            BitsPerPixel::Twentyfour => 24,
            BitsPerPixel::Thirtytwo => 32,
        }
    }
}

#[derive(Debug)]
pub struct Pixels<'a, 'b, 'sdl, const FREEABLE: bool> {
    surface: &'a mut surface::Usable<'b, 'sdl, FREEABLE>,
    width: u16,
    height: u16,
}

impl<'a, 'b: 'a, 'sdl, const FREEABLE: bool> Pixels<'a, 'b, 'sdl, FREEABLE> {
    pub fn new(surface: &'a mut surface::Usable<'b, 'sdl, FREEABLE>) -> Self {
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

    #[must_use]
    pub fn get(&self, x: u16, y: u16) -> Option<Ref<'_, 'a, 'b, 'sdl, FREEABLE>> {
        if x >= self.width || y >= self.height {
            return None;
        }

        let pos = pixel_offset(x, y, self.width);
        Some(Ref { pixels: self, pos })
    }

    pub fn get_mut(&mut self, x: u16, y: u16) -> Option<RefMut<'_, 'a, 'b, 'sdl, FREEABLE>> {
        if x >= self.width || y >= self.height {
            return None;
        }

        let pos = pixel_offset(x, y, self.width);
        Some(RefMut { pixels: self, pos })
    }

    pub fn set(&mut self, x: u16, y: u16, pixel: Pixel) -> Result<(), InvalidError> {
        self.get_mut(x, y).ok_or(InvalidError)?.set(pixel);
        Ok(())
    }
}

macro_rules! impl_pixel_raw_slice {
    (@inner $name:ident, $ty:ident, $from_raw_parts:ident) => {
        impl<'a, 'b: 'a, 'sdl, const FREEABLE: bool> Pixels<'a, 'b, 'sdl, FREEABLE> {
            fn $name(&self) -> $ty {
                use $ty::*;

                let buffer_size = self.surface.buffer_size();
                let pixel_ptr = self.surface.raw().pixels();

                let bpp = self.surface.format().bytes_per_pixel();
                match bpp {
                    BytesPerPixel::One => {
                        One(unsafe { std::slice::$from_raw_parts(pixel_ptr.cast_mut().cast(), buffer_size) })
                    }
                    BytesPerPixel::Two => {
                        Two(unsafe { std::slice::$from_raw_parts(pixel_ptr.cast_mut().cast(), buffer_size / 2) })
                    }
                    BytesPerPixel::Three => Three(unsafe {
                        std::slice::$from_raw_parts(pixel_ptr.cast_mut().cast(), buffer_size / 3)
                    }),
                    BytesPerPixel::Four => Four(unsafe {
                        std::slice::$from_raw_parts(pixel_ptr.cast_mut().cast(), buffer_size / 4)
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
pub struct Ref<'a, 'b, 'c, 'sdl, const FREEABLE: bool> {
    pixels: &'a Pixels<'b, 'c, 'sdl, FREEABLE>,
    pos: usize,
}

#[derive(Debug)]
pub struct RefMut<'a, 'b, 'c, 'sdl, const FREEABLE: bool> {
    pixels: &'a mut Pixels<'b, 'c, 'sdl, FREEABLE>,
    pos: usize,
}

macro_rules! impl_pixel_ref {
    ($ty:ident) => {
        impl<const FREEABLE: bool> $ty<'_, '_, '_, '_, FREEABLE> {
            #[must_use]
            pub fn get(&self) -> u32 {
                raw_get_pixel(self.pixels.raw_slice(), self.pos, || {
                    self.pixels.surface.raw().format()
                })
            }

            #[must_use]
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

impl_pixel_ref!(Ref);
impl_pixel_ref!(RefMut);

impl<const FREEABLE: bool> RefMut<'_, '_, '_, '_, FREEABLE> {
    pub fn set(&mut self, value: Pixel) {
        raw_set_pixel(self.pixels.raw_slice_mut(), self.pos, value);
    }
}

#[inline]
fn raw_get_pixel<F>(pixels_slice: PixelsSlicePerBpp, pos: usize, get_pixel_format: F) -> u32
where
    F: FnOnce() -> *const SDL_PixelFormat,
{
    use PixelsSlicePerBpp::{Four, One, Three, Two};

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
fn raw_set_pixel(pixels_slice: PixelsSliceMutPerBpp, pos: usize, value: Pixel) {
    use PixelsSliceMutPerBpp::{Four, One, Three, Two};

    let value = value.0;
    #[allow(clippy::cast_possible_truncation)]
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
pub struct InvalidError;

impl fmt::Display for InvalidError {
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
        const VALUE: Pixel = Pixel(0x0012_3456);

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
            Bmask: 0x00FF_0000,
            Amask: 0,
            colorkey: 0,
            alpha: 0,
        };
        let pixel = raw_get_pixel(PixelsSlicePerBpp::Three(&data), 0, || &pixel_format);
        assert_eq!(pixel, VALUE.0);
    }
}
