use crate::{
    graphics::{putpixel, IMG_Load, ScalePic},
    misc::MyMalloc,
    sdl_must_lock,
};

use core::fmt;
use log::warn;
use sdl::{
    sdl::Rect,
    video::{
        ll::{
            SDL_ConvertSurface, SDL_FreeSurface, SDL_GetRGB, SDL_LockSurface, SDL_MapRGB, SDL_Rect,
            SDL_SetColorKey, SDL_Surface, SDL_UnlockSurface, SDL_UpperBlit,
        },
        SurfaceFlag,
    },
};
use std::{
    convert::{TryFrom, TryInto},
    ffi::CStr,
    mem,
    os::raw::{c_char, c_float, c_int, c_void},
    ptr::null_mut,
};

pub static mut CurrentFont: *mut BFontInfo = null_mut();

#[derive(Clone)]
#[repr(C)]
pub struct BFontInfo {
    /// font height
    pub h: c_int,

    /// font surface
    pub surface: *mut SDL_Surface,

    /// characters width
    pub chars: [SDL_Rect; 256],
}

pub unsafe fn GetCurrentFont() -> *mut BFontInfo {
    CurrentFont
}

pub unsafe fn SetCurrentFont(font: *mut BFontInfo) {
    CurrentFont = font;
}

pub fn FontHeight(font: &BFontInfo) -> c_int {
    font.h
}

pub unsafe fn PutStringFont(
    surface: *mut SDL_Surface,
    font: *mut BFontInfo,
    mut x: c_int,
    y: c_int,
    text: &[u8],
) {
    for &c in text {
        x += PutCharFont(&mut *surface, &mut *font, x, y, c);
    }
}

/// Put a single char on the surface with the specified font
pub unsafe fn PutCharFont(
    surface: &mut SDL_Surface,
    font: &mut BFontInfo,
    x: c_int,
    y: c_int,
    c: u8,
) -> c_int {
    let mut dest = Rect::new(
        x.try_into().unwrap(),
        y.try_into().unwrap(),
        CharWidth(font, b' '.into()).try_into().unwrap(),
        FontHeight(font).try_into().unwrap(),
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
pub fn CharWidth(font: &BFontInfo, c: u8) -> c_int {
    font.chars[usize::from(c)].w.into()
}

pub unsafe fn PutString(surface: *mut SDL_Surface, x: c_int, y: c_int, text: &[u8]) {
    PutStringFont(surface, CurrentFont, x, y, text);
}

/// Puts a single char on the surface
pub unsafe fn PutChar(surface: *mut SDL_Surface, x: c_int, y: c_int, c: u8) -> c_int {
    PutCharFont(&mut *surface, &mut *CurrentFont, x, y, c)
}

pub unsafe fn PrintStringFont(
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
    PutStringFont(surface, font, x, y, &temp[..written]);
}

pub unsafe fn PutPixel(surface: &SDL_Surface, x: c_int, y: c_int, pixel: u32) {
    putpixel(surface, x, y, pixel)
}

/// Load the font and stores it in the BFont_Info structure
pub unsafe fn LoadFont(filename: *mut c_char, scale: c_float) -> *mut BFontInfo {
    if filename.is_null() {
        return null_mut();
    }

    let font = MyMalloc(mem::size_of::<BFontInfo>().try_into().unwrap()) as *mut BFontInfo;
    if font.is_null() {
        return null_mut();
    }

    let mut surface = IMG_Load(filename);
    ScalePic(&mut surface, scale);

    if surface.is_null() {
        libc::free(font as *mut c_void);
        return null_mut();
    }

    (*font).surface = surface;
    (*font)
        .chars
        .iter_mut()
        .for_each(|rect| *rect = Rect::new(0, 0, 0, 0));
    /* Init the font */
    InitFont(&mut *font);
    /* Set the font as the current font */
    SetCurrentFont(font);

    font
}

pub unsafe fn InitFont(font: &mut BFontInfo) {
    let mut i: usize = b'!'.into();
    assert!(!font.surface.is_null());
    let sentry = GetPixel(&mut *font.surface, 0, 0);

    if font.surface.is_null() {
        panic!("BFont: The font has not been loaded!");
    }

    let surface = &mut *font.surface;
    if sdl_must_lock(surface) {
        SDL_LockSurface(surface);
    }
    let mut x = 0;
    while x < (surface.w - 1) {
        if GetPixel(surface, x, 0) != sentry {
            font.chars[i].x = x.try_into().unwrap();
            font.chars[i].y = 1;
            font.chars[i].h = surface.h.try_into().unwrap();
            while GetPixel(surface, x, 0) != sentry && x < (surface.w) {
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
        GetPixel(surface, 0, surface.h - 1),
    );
}

pub unsafe fn GetPixel(surface: &mut SDL_Surface, x: i32, y: i32) -> u32 {
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

pub unsafe fn CenteredPrintStringFont(
    surface: *mut SDL_Surface,
    font: *mut BFontInfo,
    y: c_int,
    format_args: fmt::Arguments,
) {
    use std::{io::Cursor, io::Write};

    let mut temp = [0u8; 10001];
    let mut cursor = Cursor::new(temp.as_mut());
    cursor.write_fmt(format_args).unwrap();
    let written = cursor.position().try_into().unwrap();
    CenteredPutStringFont(surface, font, y, &temp[..written]);
}

pub unsafe fn CenteredPrintString(
    surface: *mut SDL_Surface,
    y: c_int,
    format_args: fmt::Arguments,
) {
    use std::{io::Cursor, io::Write};

    let mut temp = [0u8; 10001];
    let mut cursor = Cursor::new(temp.as_mut());
    cursor.write_fmt(format_args).unwrap();
    let written = cursor.position().try_into().unwrap();
    CenteredPutString(surface, y, &temp[..written]);
}

pub unsafe fn PrintString(
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
    PutStringFont(surface, CurrentFont, x, y, &temp[..written]);
}

pub unsafe fn LeftPutStringFont(
    surface: *mut SDL_Surface,
    font: *mut BFontInfo,
    y: c_int,
    text: &[u8],
) {
    PutStringFont(surface, font, 0, y, text);
}

pub unsafe fn LeftPutString(surface: *mut SDL_Surface, y: c_int, text: &[u8]) {
    LeftPutStringFont(surface, CurrentFont, y, text);
}

pub unsafe fn RightPutStringFont(
    surface: *mut SDL_Surface,
    font: *mut BFontInfo,
    y: c_int,
    text: &[u8],
) {
    PutStringFont(
        surface,
        font,
        (*surface).w - TextWidthFont(&*font, text) - 1,
        y,
        text,
    );
}

pub unsafe fn RightPutString(surface: *mut SDL_Surface, y: c_int, text: &[u8]) {
    RightPutStringFont(surface, CurrentFont, y, text);
}

pub unsafe fn CenteredPutStringFont(
    surface: *mut SDL_Surface,
    font: *mut BFontInfo,
    y: c_int,
    text: &[u8],
) {
    PutStringFont(
        surface,
        font,
        (*surface).w / 2 - TextWidthFont(&*font, text) / 2,
        y,
        text,
    );
}

pub unsafe fn CenteredPutString(surface: *mut SDL_Surface, y: c_int, text: &[u8]) {
    CenteredPutStringFont(surface, CurrentFont, y, text);
}

pub unsafe fn JustifiedPutStringFont(
    surface: *mut SDL_Surface,
    font: *mut BFontInfo,
    y: c_int,
    text: &[u8],
) {
    if !text.contains(&b' ') {
        PutStringFont(surface, font, 0, y, text);
    } else {
        let gap = ((*surface).w - 1) - TextWidthFont(&*font, text);

        if gap <= 0 {
            PutStringFont(surface, font, 0, y, text);
        } else {
            let mut spaces = count(text);
            let mut dif = gap % spaces;
            let single_gap = (gap - dif) / spaces;
            let mut xpos = 0;
            let mut pos = -1;
            while spaces > 0 {
                let strtmp = text[usize::try_from(pos + 1).unwrap()..]
                    .splitn(2, |&c| c == b' ')
                    .nth(1)
                    .unwrap();
                PutStringFont(surface, font, xpos, y, strtmp);
                xpos = xpos
                    + TextWidthFont(&*font, strtmp)
                    + single_gap
                    + CharWidth(&*font, b' '.into());
                if dif >= 0 {
                    xpos += 1;
                    dif -= 1;
                }
                pos = strtmp.as_ptr().offset_from(text.as_ptr());
                spaces -= 1;
            }

            let strtmp = &text[usize::try_from(pos + 1).unwrap()..];
            PutStringFont(surface, font, xpos, y, strtmp);
        }
    }
}

pub unsafe fn JustifiedPutString(surface: *mut SDL_Surface, y: c_int, text: &[u8]) {
    JustifiedPutStringFont(surface, CurrentFont, y, text);
}

pub unsafe fn count(text: &[u8]) -> c_int {
    text.iter()
        .copied()
        .filter(|&c| c == b' ')
        .count()
        .try_into()
        .unwrap()
}

pub unsafe fn TextWidth(text: &[u8]) -> c_int {
    TextWidthFont(&*CurrentFont, text)
}

pub fn TextWidthFont(font: &BFontInfo, text: &[u8]) -> c_int {
    text.iter().map(|&c| CharWidth(font, c)).sum()
}

pub unsafe fn SetFontHeight(font: &mut BFontInfo, height: c_int) {
    font.h = height;
}

pub unsafe fn FreeFont(font: *mut BFontInfo) {
    SDL_FreeSurface((*font).surface);
    libc::free(font as *mut c_void);
}

pub unsafe fn SetFontColor(font: &BFontInfo, r: u8, g: u8, b: u8) -> *mut BFontInfo {
    let newfont_ptr = libc::malloc(std::mem::size_of::<BFontInfo>()) as *mut BFontInfo;
    if newfont_ptr.is_null() {
        return null_mut();
    }

    let newfont = &mut *newfont_ptr;
    newfont.h = font.h;
    newfont.chars.copy_from_slice(&font.chars);

    let surface_ptr =
        SDL_ConvertSurface(font.surface, (*font.surface).format, (*font.surface).flags);
    if !surface_ptr.is_null() {
        let surface = &mut *surface_ptr;
        if sdl_must_lock(surface) {
            SDL_LockSurface(surface);
        }
        if sdl_must_lock(&*font.surface) {
            SDL_LockSurface(font.surface);
        }

        let color_key = GetPixel(surface, 0, surface.h - 1);

        for x in 0..(*font.surface).w {
            for y in 0..(*font.surface).h {
                let pixel = GetPixel(&mut *font.surface, x, y);

                if pixel != color_key {
                    let mut old_r = 0;
                    let mut old_g = 0;
                    let mut old_b = 0;
                    SDL_GetRGB(pixel, surface.format, &mut old_r, &mut old_g, &mut old_b);

                    let new_r = ((u16::from(old_r) * u16::from(r)) / 255)
                        .try_into()
                        .unwrap();
                    let new_g = ((u16::from(old_g) * u16::from(g)) / 255)
                        .try_into()
                        .unwrap();
                    let new_b = ((u16::from(old_b) * u16::from(b)) / 255)
                        .try_into()
                        .unwrap();

                    let pixel = SDL_MapRGB(surface.format, new_r, new_g, new_b);
                    PutPixel(surface, x, y, pixel);
                }
            }
        }

        if sdl_must_lock(surface) {
            SDL_UnlockSurface(surface);
        }
        if sdl_must_lock(&*font.surface) {
            SDL_UnlockSurface(font.surface);
        }

        SDL_SetColorKey(surface, SurfaceFlag::SrcColorKey as u32, color_key);
    }

    newfont.surface = surface_ptr;
    newfont_ptr
}
