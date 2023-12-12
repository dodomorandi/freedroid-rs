use crate::{graphics::scale_pic, Data, FontCell, FontCellOwner, Sdl};

use sdl::{ColorKeyFlag, Rect};
use std::{
    ffi::CStr,
    fmt,
    os::raw::{c_float, c_int},
    rc::Rc,
};

#[derive(Default)]
pub struct BFont<'sdl> {
    pub current_font: Option<Rc<FontCell<'sdl>>>,
}

impl fmt::Debug for BFont<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BFont")
            .field("current_font", {
                match self.current_font {
                    Some(_) => &"Some(Rc(BFontInfo))",
                    None => &"None",
                }
            })
            .finish()
    }
}

#[derive(Debug)]
pub struct BFontInfo<'sdl> {
    /// font height
    pub h: c_int,

    /// font surface
    pub surface: Option<sdl::Surface<'sdl>>,

    /// characters width
    pub chars: [Rect; 256],
}

pub fn font_height(font: &BFontInfo) -> c_int {
    font.h
}

pub fn put_string_font<const F: bool>(
    surface: &mut sdl::GenericSurface<F>,
    font: &mut BFontInfo,
    mut x: c_int,
    y: c_int,
    text: &[u8],
) {
    for &c in text {
        x += put_char_font(surface, font, x, y, c);
    }
}

/// Put a single char on the surface with the specified font
fn put_char_font<const F: bool>(
    surface: &mut sdl::GenericSurface<F>,
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
        font.surface.as_mut().unwrap().blit_from_to(
            &font.chars[usize::from(c)],
            surface,
            &mut dest,
        );
    }
    dest.width().into()
}

/// Return the width of the "c" character
pub fn char_width(font: &BFontInfo, c: u8) -> c_int {
    font.chars[usize::from(c)].width().into()
}

pub fn print_string_font<const F: bool>(
    surface: &mut sdl::GenericSurface<F>,
    font: &mut BFontInfo,
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

pub fn init_font(font: &mut BFontInfo) {
    let mut i: usize = b'!'.into();

    let mut surface = font.surface.as_mut().unwrap().lock().unwrap();
    let surface_width = surface.width();
    let surface_height = surface.height();
    let pixels = surface.pixels();

    let sentry = pixels.get(0, 0).unwrap().get();

    let mut x = 0;
    while x < (surface_width - 1) {
        if pixels.get(x, 0).unwrap().get() != sentry {
            font.chars[i].set_x(x.try_into().unwrap());
            font.chars[i].set_y(1);
            font.chars[i].set_height(surface_height);
            while x < surface_width && pixels.get(x, 0).unwrap().get() != sentry {
                x += 1;
            }
            font.chars[i].set_width(
                (i32::from(x) - i32::from(font.chars[i].x()))
                    .try_into()
                    .unwrap(),
            );
            i += 1;
        } else {
            x += 1;
        }
    }
    font.chars[b' ' as usize].set_x(0);
    font.chars[b' ' as usize].set_y(0);
    font.chars[b' ' as usize].set_height(surface_height);
    font.chars[b' ' as usize].set_width(font.chars[b'!' as usize].width());

    let last_row_pixel = pixels.get(0, surface_height - 1).unwrap().get();
    drop(surface);
    let surface = font.surface.as_mut().unwrap();

    font.h = surface.height().into();

    assert!(
        surface.set_color_key(ColorKeyFlag::SRC_COLOR_KEY, last_row_pixel.into()),
        "SDL set color key failed"
    );
}

pub fn centered_put_string_font<const F: bool>(
    surface: &mut sdl::GenericSurface<F>,
    font: &mut BFontInfo,
    y: c_int,
    text: &[u8],
) {
    put_string_font(
        surface,
        font,
        c_int::from(surface.width() / 2) - text_width_font(&*font, text) / 2,
        y,
        text,
    );
}

impl<'sdl> Data<'sdl> {
    pub fn put_string<const F: bool>(
        &mut self,
        surface: &mut sdl::GenericSurface<F>,
        x: c_int,
        y: c_int,
        text: &[u8],
    ) {
        Self::put_string_static(&self.b_font, &mut self.font_owner, surface, x, y, text);
    }

    pub fn put_string_static<const F: bool>(
        b_font: &BFont,
        font_owner: &mut FontCellOwner,
        surface: &mut sdl::GenericSurface<F>,
        x: c_int,
        y: c_int,
        text: &[u8],
    ) {
        put_string_font(
            surface,
            b_font.current_font.as_ref().unwrap().rw(font_owner),
            x,
            y,
            text,
        );
    }

    /// Puts a single char on the surface
    pub fn put_char<const F: bool>(
        b_font: &BFont,
        font_owner: &mut FontCellOwner,
        surface: &mut sdl::GenericSurface<F>,
        x: c_int,
        y: c_int,
        c: u8,
    ) -> c_int {
        put_char_font(
            surface,
            b_font.current_font.as_ref().unwrap().rw(font_owner),
            x,
            y,
            c,
        )
    }

    /// Load the font and stores it in the BFont_Info structure
    pub fn load_font(
        sdl: &'sdl Sdl,
        b_font: &mut BFont<'sdl>,
        filename: &CStr,
        scale: c_float,
    ) -> Rc<FontCell<'sdl>> {
        let mut surface = sdl.load_image_from_c_str_path(filename).unwrap();
        scale_pic(&mut surface, scale);

        let mut font = BFontInfo {
            h: 0,
            surface: Some(surface),
            chars: [Rect::default(); 256],
        };
        /* Init the font */
        init_font(&mut font);

        /* Set the font as the current font */
        let font = Rc::new(FontCell::new(font));
        b_font.current_font = Some(Rc::clone(&font));
        font
    }

    pub fn centered_print_string<const F: bool>(
        b_font: &BFont,
        font_owner: &mut FontCellOwner,
        surface: &mut sdl::GenericSurface<F>,
        y: c_int,
        format_args: fmt::Arguments,
    ) {
        use std::{io::Cursor, io::Write};

        let mut temp = [0u8; 10001];
        let mut cursor = Cursor::new(temp.as_mut());
        cursor.write_fmt(format_args).unwrap();
        let written = cursor.position().try_into().unwrap();
        Self::centered_put_string_static(b_font, font_owner, surface, y, &temp[..written]);
    }

    pub fn print_string<const F: bool>(
        &mut self,
        surface: &mut sdl::GenericSurface<F>,
        x: c_int,
        y: c_int,
        format_args: fmt::Arguments,
    ) {
        use std::{io::Cursor, io::Write};

        let mut temp = vec![0u8; 10001].into_boxed_slice();
        let mut cursor = Cursor::new(temp.as_mut());
        cursor.write_fmt(format_args).unwrap();
        let written = cursor.position().try_into().unwrap();
        put_string_font(
            surface,
            self.b_font
                .current_font
                .as_ref()
                .unwrap()
                .rw(&mut self.font_owner),
            x,
            y,
            &temp[..written],
        );
    }

    #[cfg(not(target_os = "android"))]
    pub fn centered_put_string<const F: bool>(
        &mut self,
        surface: &mut sdl::GenericSurface<F>,
        y: c_int,
        text: &[u8],
    ) {
        centered_put_string_font(
            surface,
            self.b_font
                .current_font
                .as_ref()
                .unwrap()
                .rw(&mut self.font_owner),
            y,
            text,
        );
    }

    pub fn centered_put_string_static<const F: bool>(
        b_font: &BFont,
        font_owner: &mut FontCellOwner,
        surface: &mut sdl::GenericSurface<F>,
        y: c_int,
        text: &[u8],
    ) {
        centered_put_string_font(
            surface,
            b_font.current_font.as_ref().unwrap().rw(font_owner),
            y,
            text,
        );
    }

    pub fn text_width(&self, text: &[u8]) -> c_int {
        text_width_font(
            self.b_font
                .current_font
                .as_ref()
                .unwrap()
                .ro(&self.font_owner),
            text,
        )
    }
}

pub fn text_width_font(font: &BFontInfo, text: &[u8]) -> c_int {
    text.iter().map(|&c| char_width(font, c)).sum()
}
