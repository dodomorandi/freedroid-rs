use crate::{
    graphics::{putpixel, IMG_Load, ScalePic},
    misc::MyMalloc,
    sdl_must_lock,
};

use log::warn;
use sdl::{
    sdl::Rect,
    video::{
        ll::{
            SDL_FreeSurface, SDL_LockSurface, SDL_MapRGB, SDL_Rect, SDL_SetColorKey, SDL_Surface,
            SDL_UnlockSurface, SDL_UpperBlit,
        },
        SurfaceFlag,
    },
};
use std::{
    convert::{TryFrom, TryInto},
    ffi::{CStr, VaList},
    mem,
    os::raw::{c_char, c_float, c_int, c_void},
    ptr::null_mut,
};

extern "C" {
    pub static mut Highscore_BFont: *mut BFontInfo;
    pub static mut Para_BFont: *mut BFontInfo;
    pub static mut CurrentFont: *mut BFontInfo;
    fn vsprintf(str: *mut c_char, format: *const c_char, ap: VaList) -> c_int;
}

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

#[no_mangle]
pub unsafe extern "C" fn GetCurrentFont() -> *mut BFontInfo {
    CurrentFont
}

#[no_mangle]
pub unsafe extern "C" fn SetCurrentFont(font: *mut BFontInfo) {
    CurrentFont = font;
}

#[no_mangle]
pub extern "C" fn FontHeight(font: &BFontInfo) -> c_int {
    font.h
}

#[no_mangle]
pub unsafe extern "C" fn PutStringFont(
    surface: *mut SDL_Surface,
    font: *mut BFontInfo,
    mut x: c_int,
    y: c_int,
    text: *const c_char,
) {
    let mut i = 0;
    while *text.offset(i) != b'\0' as i8 {
        x += PutCharFont(&mut *surface, &mut *font, x, y, (*text.offset(i)).into());
        i += 1;
    }
}

/// Put a single char on the surface with the specified font
#[no_mangle]
pub unsafe extern "C" fn PutCharFont(
    surface: &mut SDL_Surface,
    font: &mut BFontInfo,
    x: c_int,
    y: c_int,
    c: c_int,
) -> c_int {
    let mut dest = Rect::new(
        x.try_into().unwrap(),
        y.try_into().unwrap(),
        CharWidth(font, b' '.into()).try_into().unwrap(),
        FontHeight(font).try_into().unwrap(),
    );

    if c != b' '.into() {
        SDL_UpperBlit(
            font.surface,
            &mut font.chars[usize::try_from(c).unwrap()],
            surface,
            &mut dest,
        );
    }
    dest.w.into()
}

/// Return the width of the "c" character
#[no_mangle]
pub extern "C" fn CharWidth(font: &BFontInfo, c: c_int) -> c_int {
    font.chars[usize::try_from(c).unwrap()].w.into()
}

#[no_mangle]
pub unsafe extern "C" fn PutString(
    surface: *mut SDL_Surface,
    x: c_int,
    y: c_int,
    text: *const c_char,
) {
    PutStringFont(surface, CurrentFont, x, y, text);
}

/// Puts a single char on the surface
#[no_mangle]
pub unsafe extern "C" fn PutChar(surface: *mut SDL_Surface, x: c_int, y: c_int, c: c_int) -> c_int {
    PutCharFont(&mut *surface, &mut *CurrentFont, x, y, c)
}

#[no_mangle]
pub unsafe extern "C" fn PrintStringFont(
    surface: *mut SDL_Surface,
    font: *mut BFontInfo,
    x: c_int,
    y: c_int,
    fmt: *mut c_char,
    args: ...
) {
    let mut args = args.clone();

    let mut temp = [0; 1001];
    vsprintf(temp.as_mut_ptr(), fmt, args.as_va_list());
    PutStringFont(surface, font, x, y, temp.as_mut_ptr());
}

#[no_mangle]
pub unsafe extern "C" fn PutPixel(surface: &SDL_Surface, x: c_int, y: c_int, pixel: u32) {
    putpixel(surface, x, y, pixel)
}

/// Load the font and stores it in the BFont_Info structure
#[no_mangle]
pub unsafe extern "C" fn LoadFont(filename: *mut c_char, scale: c_float) -> *mut BFontInfo {
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

#[no_mangle]
pub unsafe extern "C" fn InitFont(font: &mut BFontInfo) {
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

#[no_mangle]
pub unsafe extern "C" fn GetPixel(surface: &mut SDL_Surface, x: i32, y: i32) -> u32 {
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

#[no_mangle]
pub unsafe extern "C" fn JustifiedPrintStringFont(
    surface: *mut SDL_Surface,
    font: *mut BFontInfo,
    y: c_int,
    fmt: *mut c_char,
    args: ...
) {
    let mut args = args.clone();

    let mut temp = vec![0u8; 10001].into_boxed_slice();
    vsprintf(temp.as_mut_ptr() as *mut c_char, fmt, args.as_va_list());
    JustifiedPutStringFont(surface, font, y, temp.as_mut_ptr() as *mut c_char);
}

#[no_mangle]
pub unsafe extern "C" fn JustifiedPrintString(
    surface: *mut SDL_Surface,
    y: c_int,
    fmt: *mut c_char,
    args: ...
) {
    let mut args = args.clone();

    let mut temp = vec![0u8; 10001].into_boxed_slice();
    vsprintf(temp.as_mut_ptr() as *mut c_char, fmt, args.as_va_list());
    JustifiedPutString(surface, y, temp.as_mut_ptr() as *mut c_char);
}

#[no_mangle]
pub unsafe extern "C" fn LeftPrintStringFont(
    surface: *mut SDL_Surface,
    font: *mut BFontInfo,
    y: c_int,
    fmt: *mut c_char,
    args: ...
) {
    let mut args = args.clone();

    let mut temp = vec![0u8; 10001].into_boxed_slice();
    vsprintf(temp.as_mut_ptr() as *mut c_char, fmt, args.as_va_list());
    LeftPutStringFont(surface, font, y, temp.as_mut_ptr() as *mut c_char);
}

#[no_mangle]
pub unsafe extern "C" fn LeftPrintString(
    surface: *mut SDL_Surface,
    y: c_int,
    fmt: *mut c_char,
    args: ...
) {
    let mut args = args.clone();

    let mut temp = vec![0u8; 10001].into_boxed_slice();
    vsprintf(temp.as_mut_ptr() as *mut c_char, fmt, args.as_va_list());
    LeftPutString(surface, y, temp.as_mut_ptr() as *mut c_char);
}

#[no_mangle]
pub unsafe extern "C" fn RightPrintStringFont(
    surface: *mut SDL_Surface,
    font: *mut BFontInfo,
    y: c_int,
    fmt: *mut c_char,
    args: ...
) {
    let mut args = args.clone();

    let mut temp = vec![0u8; 10001].into_boxed_slice();
    vsprintf(temp.as_mut_ptr() as *mut c_char, fmt, args.as_va_list());
    RightPutStringFont(surface, font, y, temp.as_mut_ptr() as *mut c_char);
}

#[no_mangle]
pub unsafe extern "C" fn RightPrintString(
    surface: *mut SDL_Surface,
    y: c_int,
    fmt: *mut c_char,
    args: ...
) {
    let mut args = args.clone();

    let mut temp = vec![0u8; 10001].into_boxed_slice();
    vsprintf(temp.as_mut_ptr() as *mut c_char, fmt, args.as_va_list());
    RightPutString(surface, y, temp.as_mut_ptr() as *mut c_char);
}

#[no_mangle]
pub unsafe extern "C" fn CenteredPrintStringFont(
    surface: *mut SDL_Surface,
    font: *mut BFontInfo,
    y: c_int,
    fmt: *mut c_char,
    args: ...
) {
    let mut args = args.clone();

    let mut temp = vec![0u8; 10001].into_boxed_slice();
    vsprintf(temp.as_mut_ptr() as *mut c_char, fmt, args.as_va_list());
    CenteredPutStringFont(surface, font, y, temp.as_mut_ptr() as *mut c_char);
}

#[no_mangle]
pub unsafe extern "C" fn CenteredPrintString(
    surface: *mut SDL_Surface,
    y: c_int,
    fmt: *mut c_char,
    args: ...
) {
    let mut args = args.clone();

    let mut temp = vec![0u8; 10001].into_boxed_slice();
    vsprintf(temp.as_mut_ptr() as *mut c_char, fmt, args.as_va_list());
    CenteredPutString(surface, y, temp.as_mut_ptr() as *mut c_char);
}

#[no_mangle]
pub unsafe extern "C" fn PrintString(
    surface: *mut SDL_Surface,
    x: c_int,
    y: c_int,
    fmt: *mut c_char,
    args: ...
) {
    let mut args = args.clone();

    let mut temp = vec![0u8; 10001].into_boxed_slice();
    vsprintf(temp.as_mut_ptr() as *mut c_char, fmt, args.as_va_list());

    PutStringFont(surface, CurrentFont, x, y, temp.as_mut_ptr() as *mut c_char);
}

#[no_mangle]
pub unsafe extern "C" fn LeftPutStringFont(
    surface: *mut SDL_Surface,
    font: *mut BFontInfo,
    y: c_int,
    text: *mut c_char,
) {
    PutStringFont(surface, font, 0, y, text);
}

#[no_mangle]
pub unsafe extern "C" fn LeftPutString(surface: *mut SDL_Surface, y: c_int, text: *mut c_char) {
    LeftPutStringFont(surface, CurrentFont, y, text);
}

#[no_mangle]
pub unsafe extern "C" fn RightPutStringFont(
    surface: *mut SDL_Surface,
    font: *mut BFontInfo,
    y: c_int,
    text: *mut c_char,
) {
    PutStringFont(
        surface,
        font,
        (*surface).w - TextWidthFont(&*font, text) - 1,
        y,
        text,
    );
}

#[no_mangle]
pub unsafe extern "C" fn RightPutString(surface: *mut SDL_Surface, y: c_int, text: *mut c_char) {
    RightPutStringFont(surface, CurrentFont, y, text);
}

#[no_mangle]
pub unsafe extern "C" fn CenteredPutStringFont(
    surface: *mut SDL_Surface,
    font: *mut BFontInfo,
    y: c_int,
    text: *mut c_char,
) {
    PutStringFont(
        surface,
        font,
        (*surface).w / 2 - TextWidthFont(&*font, text) / 2,
        y,
        text,
    );
}

#[no_mangle]
pub unsafe extern "C" fn CenteredPutString(surface: *mut SDL_Surface, y: c_int, text: *mut c_char) {
    CenteredPutStringFont(surface, CurrentFont, y, text);
}

#[no_mangle]
pub unsafe extern "C" fn JustifiedPutStringFont(
    surface: *mut SDL_Surface,
    font: *mut BFontInfo,
    y: c_int,
    text: *mut c_char,
) {
    let text = CStr::from_ptr(text).to_bytes();
    if !text.contains(&b' ') {
        PutStringFont(surface, font, 0, y, text.as_ptr() as *const c_char);
    } else {
        let gap = ((*surface).w - 1) - TextWidthFont(&*font, text.as_ptr() as *const c_char);

        if gap <= 0 {
            PutStringFont(surface, font, 0, y, text.as_ptr() as *const c_char);
        } else {
            let mut spaces = count(text.as_ptr() as *mut c_char);
            let mut dif = gap % spaces;
            let single_gap = (gap - dif) / spaces;
            let mut xpos = 0;
            let mut pos = -1;
            while spaces > 0 {
                let p = text[usize::try_from(pos + 1).unwrap()..]
                    .splitn(2, |&c| c == b' ')
                    .nth(1)
                    .unwrap();
                let strtmp = p.to_vec();
                PutStringFont(surface, font, xpos, y, strtmp.as_ptr() as *const c_char);
                xpos = xpos
                    + TextWidthFont(&*font, strtmp.as_ptr() as *const c_char)
                    + single_gap
                    + CharWidth(&*font, b' '.into());
                if dif >= 0 {
                    xpos += 1;
                    dif -= 1;
                }
                pos = p.as_ptr().offset_from(text.as_ptr());
                spaces -= 1;
            }

            let strtmp = text[usize::try_from(pos + 1).unwrap()..].to_vec();
            PutStringFont(surface, font, xpos, y, strtmp.as_ptr() as *const c_char);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn JustifiedPutString(
    surface: *mut SDL_Surface,
    y: c_int,
    text: *mut c_char,
) {
    JustifiedPutStringFont(surface, CurrentFont, y, text);
}

#[no_mangle]
pub unsafe extern "C" fn count(text: *const c_char) -> c_int {
    CStr::from_ptr(text)
        .to_bytes()
        .iter()
        .copied()
        .filter(|&c| c == b' ')
        .count()
        .try_into()
        .unwrap()
}

#[no_mangle]
pub unsafe extern "C" fn TextWidth(text: *const c_char) -> c_int {
    TextWidthFont(&*CurrentFont, text)
}

#[no_mangle]
pub unsafe extern "C" fn TextWidthFont(font: &BFontInfo, text: *const c_char) -> c_int {
    CStr::from_ptr(text)
        .to_bytes()
        .iter()
        .map(|&c| CharWidth(font, c.into()))
        .sum()
}

#[no_mangle]
pub unsafe extern "C" fn SetFontHeight(font: &mut BFontInfo, height: c_int) {
    font.h = height;
}

#[no_mangle]
pub unsafe extern "C" fn FreeFont(font: *mut BFontInfo) {
    SDL_FreeSurface((*font).surface);
    libc::free(font as *mut c_void);
}
