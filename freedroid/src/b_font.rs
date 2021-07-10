use crate::{
    graphics::{putpixel, scale_pic},
    Data,
};

use core::fmt;
use sdl::Surface;
use sdl_sys::{IMG_Load, SDL_Rect, SDL_SetColorKey, SDL_Surface, SDL_UpperBlit, SDL_SRCCOLORKEY};
use std::{
    alloc::{alloc_zeroed, dealloc, Layout},
    convert::TryInto,
    os::raw::{c_char, c_float, c_int},
    ptr::{null_mut, NonNull},
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

pub struct BFontInfo {
    /// font height
    pub h: c_int,

    /// font surface
    pub surface: Option<sdl::Surface>,

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
    let mut dest = rect!(
        x.try_into().unwrap(),
        y.try_into().unwrap(),
        char_width(font, b' ').try_into().unwrap(),
        font_height(font).try_into().unwrap(),
    );

    if c != b' ' {
        SDL_UpperBlit(
            font.surface.as_mut().unwrap().as_mut_ptr(),
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
        scale_pic(&mut surface, scale);

        if surface.is_null() {
            dealloc(font as *mut u8, font_layout);
            return null_mut();
        }

        (*font).surface = Some(Surface::from_ptr(NonNull::new_unchecked(surface)));
        (*font).chars.iter_mut().for_each(|rect| *rect = rect!());
        /* Init the font */
        init_font(&mut *font);
        /* Set the font as the current font */
        self.b_font.current_font = font;

        font
    }
}

pub unsafe fn init_font(font: &mut BFontInfo) {
    let mut i: usize = b'!'.into();

    let mut surface = font.surface.as_mut().unwrap().lock().unwrap();
    let surface_width = surface.width();
    let surface_height = surface.height();
    let pixels = surface.pixels();

    let sentry = pixels.get(0, 0).unwrap().get();

    let mut x = 0;
    while x < (surface_width - 1) {
        if pixels.get(x, 0).unwrap().get() != sentry {
            font.chars[i].x = x.try_into().unwrap();
            font.chars[i].y = 1;
            font.chars[i].h = surface_height;
            while pixels.get(x, 0).unwrap().get() != sentry && x < surface_width {
                x += 1;
            }
            font.chars[i].w = (i32::from(x) - i32::from(font.chars[i].x))
                .try_into()
                .unwrap();
            i += 1;
        } else {
            x += 1;
        }
    }
    font.chars[b' ' as usize].x = 0;
    font.chars[b' ' as usize].y = 0;
    font.chars[b' ' as usize].h = surface_height;
    font.chars[b' ' as usize].w = font.chars[b'!' as usize].w;

    let last_row_pixel = pixels.get(0, surface_height - 1).unwrap().get();
    drop(surface);
    let surface = font.surface.as_mut().unwrap();

    font.h = surface.height().into();

    SDL_SetColorKey(surface.as_mut_ptr(), SDL_SRCCOLORKEY as u32, last_row_pixel);
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
