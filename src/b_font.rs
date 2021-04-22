use crate::{
    graphics::{putpixel, IMG_Load},
    sdl_must_lock, Data,
};

use core::fmt;
use log::warn;
use sdl::{
    sdl::Rect,
    video::{
        ll::{
            SDL_LockSurface, SDL_MapRGB, SDL_Rect, SDL_SetColorKey, SDL_Surface, SDL_UnlockSurface,
            SDL_UpperBlit,
        },
        SurfaceFlag,
    },
};
use std::{
    alloc::{alloc_zeroed, dealloc, Layout},
    convert::{TryFrom, TryInto},
    os::raw::{c_char, c_float, c_int},
    ptr::null_mut,
};

#[derive(Debug)]
pub struct BFont {
    pub current_font: *mut BFontInfo,
}

impl Default for BFont {
    fn default() -> Self {
        Self {
            current_font: null_mut(),
        }
    }
}

#[derive(Clone)]
pub struct BFontInfo {
    /// font height
    pub h: c_int,

    /// font surface
    pub surface: *mut SDL_Surface,

    /// characters width
    pub chars: [SDL_Rect; 256],
}

pub fn font_height(font: &BFontInfo) -> c_int {
    font.h
}

pub unsafe fn put_string_font(
    surface: *mut SDL_Surface,
    font: *mut BFontInfo,
    mut x: c_int,
    y: c_int,
    text: &[u8],
) {
    for &c in text {
        x += put_char_font(&mut *surface, &mut *font, x, y, c);
    }
}

/// Put a single char on the surface with the specified font
pub unsafe fn put_char_font(
    surface: &mut SDL_Surface,
    font: &mut BFontInfo,
    x: c_int,
    y: c_int,
    c: u8,
) -> c_int {
    let mut dest = Rect::new(
        x.try_into().unwrap(),
        y.try_into().unwrap(),
        char_width(font, b' ').try_into().unwrap(),
        font_height(font).try_into().unwrap(),
    );

    if c != b' ' {
        SDL_UpperBlit(
            font.surface,
            &mut font.chars[usize::from(c)],
            surface,
            &mut dest,
        );
    }
    dest.w.into()
}

/// Return the width of the "c" character
pub fn char_width(font: &BFontInfo, c: u8) -> c_int {
    font.chars[usize::from(c)].w.into()
}

impl Data {
    pub unsafe fn put_string(&self, surface: *mut SDL_Surface, x: c_int, y: c_int, text: &[u8]) {
        put_string_font(surface, self.b_font.current_font, x, y, text);
    }

    /// Puts a single char on the surface
    pub unsafe fn put_char(
        &mut self,
        surface: *mut SDL_Surface,
        x: c_int,
        y: c_int,
        c: u8,
    ) -> c_int {
        put_char_font(&mut *surface, &mut *self.b_font.current_font, x, y, c)
    }
}

pub unsafe fn print_string_font(
    surface: *mut SDL_Surface,
    font: *mut BFontInfo,
    x: c_int,
    y: c_int,
    format_args: fmt::Arguments,
) {
    use std::{io::Cursor, io::Write};

    let mut temp = [0u8; 1001];
    let mut cursor = Cursor::new(temp.as_mut());
    cursor.write_fmt(format_args).unwrap();
    let written = cursor.position().try_into().unwrap();
    put_string_font(surface, font, x, y, &temp[..written]);
}

pub unsafe fn put_pixel(surface: &SDL_Surface, x: c_int, y: c_int, pixel: u32) {
    putpixel(surface, x, y, pixel)
}

impl Data {
    /// Load the font and stores it in the BFont_Info structure
    pub unsafe fn load_font(&mut self, filename: *mut c_char, scale: c_float) -> *mut BFontInfo {
        if filename.is_null() {
            return null_mut();
        }

        let font_layout = Layout::new::<BFontInfo>();
        let font = alloc_zeroed(font_layout) as *mut BFontInfo;
        if font.is_null() {
            return null_mut();
        }

        let mut surface = IMG_Load(filename);
        self.scale_pic(&mut surface, scale);

        if surface.is_null() {
            dealloc(font as *mut u8, font_layout);
            return null_mut();
        }

        (*font).surface = surface;
        (*font)
            .chars
            .iter_mut()
            .for_each(|rect| *rect = Rect::new(0, 0, 0, 0));
        /* Init the font */
        init_font(&mut *font);
        /* Set the font as the current font */
        self.b_font.current_font = font;

        font
    }
}

pub unsafe fn init_font(font: &mut BFontInfo) {
    let mut i: usize = b'!'.into();
    assert!(!font.surface.is_null());
    let sentry = get_pixel(&mut *font.surface, 0, 0);

    if font.surface.is_null() {
        panic!("BFont: The font has not been loaded!");
    }

    let surface = &mut *font.surface;
    if sdl_must_lock(surface) {
        SDL_LockSurface(surface);
    }
    let mut x = 0;
    while x < (surface.w - 1) {
        if get_pixel(surface, x, 0) != sentry {
            font.chars[i].x = x.try_into().unwrap();
            font.chars[i].y = 1;
            font.chars[i].h = surface.h.try_into().unwrap();
            while get_pixel(surface, x, 0) != sentry && x < (surface.w) {
                x += 1;
            }
            font.chars[i].w = (x - i32::from(font.chars[i].x)).try_into().unwrap();
            i += 1;
        } else {
            x += 1;
        }
    }
    font.chars[b' ' as usize].x = 0;
    font.chars[b' ' as usize].y = 0;
    font.chars[b' ' as usize].h = surface.h.try_into().unwrap();
    font.chars[b' ' as usize].w = font.chars[b'!' as usize].w;

    if sdl_must_lock(surface) {
        SDL_UnlockSurface(surface);
    }

    font.h = surface.h;

    SDL_SetColorKey(
        surface,
        SurfaceFlag::SrcColorKey as u32,
        get_pixel(surface, 0, surface.h - 1),
    );
}

pub unsafe fn get_pixel(surface: &mut SDL_Surface, x: i32, y: i32) -> u32 {
    if x < 0 {
        warn!("x too small in GetPixel!");
    }
    if x >= surface.w {
        warn!("x too big in GetPixel!");
    }

    let bpp = (*surface.format).BytesPerPixel;

    // Get the pixel
    match bpp {
        1 => (*(surface.pixels.offset(
            isize::try_from(y).unwrap() * isize::try_from(surface.pitch).unwrap()
                + isize::try_from(x).unwrap(),
        ) as *const u8))
            .into(),
        2 => (*((surface.pixels as *const u16).offset(
            isize::try_from(y).unwrap() * isize::try_from(surface.pitch).unwrap() / 2
                + isize::try_from(x).unwrap(),
        )))
        .into(),
        3 => {
            // Format/endian independent
            let bits = surface.pixels.offset(
                isize::try_from(y).unwrap() * isize::try_from(surface.pitch).unwrap()
                    + isize::try_from(x).unwrap() * isize::try_from(bpp).unwrap(),
            ) as *mut u8;
            let format = &*surface.format;
            let red = *((bits).offset(isize::from(format.Rshift) / 8));
            let green = *((bits).offset(isize::from(format.Gshift) / 8));
            let blue = *((bits).offset(isize::from(format.Bshift) / 8));
            SDL_MapRGB(surface.format, red, green, blue)
        }
        4 => {
            *((surface.pixels as *const u32).offset(
                isize::try_from(y).unwrap() * isize::try_from(surface.pitch).unwrap() / 4
                    + isize::try_from(x).unwrap(),
            ))
        }
        _ => u32::MAX,
    }
}

impl Data {
    pub unsafe fn centered_print_string(
        &self,
        surface: *mut SDL_Surface,
        y: c_int,
        format_args: fmt::Arguments,
    ) {
        use std::{io::Cursor, io::Write};

        let mut temp = [0u8; 10001];
        let mut cursor = Cursor::new(temp.as_mut());
        cursor.write_fmt(format_args).unwrap();
        let written = cursor.position().try_into().unwrap();
        self.centered_put_string(surface, y, &temp[..written]);
    }

    pub unsafe fn print_string(
        &mut self,
        surface: *mut SDL_Surface,
        x: c_int,
        y: c_int,
        format_args: fmt::Arguments,
    ) {
        use std::{io::Cursor, io::Write};

        let mut temp = vec![0u8; 10001].into_boxed_slice();
        let mut cursor = Cursor::new(temp.as_mut());
        cursor.write_fmt(format_args).unwrap();
        let written = cursor.position().try_into().unwrap();
        put_string_font(surface, self.b_font.current_font, x, y, &temp[..written]);
    }
}

pub unsafe fn centered_put_string_font(
    surface: *mut SDL_Surface,
    font: *mut BFontInfo,
    y: c_int,
    text: &[u8],
) {
    put_string_font(
        surface,
        font,
        (*surface).w / 2 - text_width_font(&*font, text) / 2,
        y,
        text,
    );
}

impl Data {
    pub unsafe fn centered_put_string(&self, surface: *mut SDL_Surface, y: c_int, text: &[u8]) {
        centered_put_string_font(surface, self.b_font.current_font, y, text);
    }

    pub unsafe fn text_width(&self, text: &[u8]) -> c_int {
        text_width_font(&*self.b_font.current_font, text)
    }
}

pub fn text_width_font(font: &BFontInfo, text: &[u8]) -> c_int {
    text.iter().map(|&c| char_width(font, c)).sum()
}
