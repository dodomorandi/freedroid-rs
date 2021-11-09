use num_traits::{AsPrimitive, Float};
use sdl_sys::SDL_Rect;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Rect(SDL_Rect);

impl Rect {
    #[inline]
    pub const fn new(x: i16, y: i16, width: u16, height: u16) -> Self {
        Self(SDL_Rect {
            x,
            y,
            w: width,
            h: height,
        })
    }

    #[inline]
    pub fn as_ptr(&self) -> *const SDL_Rect {
        &self.0
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut SDL_Rect {
        &mut self.0
    }

    #[inline]
    pub fn as_rect_ref(&self) -> RectRef {
        RectRef(&self.0)
    }

    #[inline]
    pub fn width(&self) -> u16 {
        self.0.w
    }

    #[inline]
    pub fn height(&self) -> u16 {
        self.0.h
    }

    #[inline]
    pub fn x(&self) -> i16 {
        self.0.x
    }

    #[inline]
    pub fn y(&self) -> i16 {
        self.0.y
    }

    #[inline]
    pub fn set_x(&mut self, value: i16) {
        self.0.x = value;
    }

    #[inline]
    pub fn set_y(&mut self, value: i16) {
        self.0.y = value;
    }

    #[inline]
    pub fn set_height(&mut self, value: u16) {
        self.0.h = value;
    }

    #[inline]
    pub fn set_width(&mut self, value: u16) {
        self.0.w = value;
    }

    #[inline]
    pub fn scale<N>(&mut self, factor: N)
    where
        N: Float + From<i16> + From<u16> + AsPrimitive<u16> + AsPrimitive<i16> + 'static,
    {
        let r = &mut self.0;
        macro_rules! scale {
            ($param:ident) => {
                r.$param = (<N as From<_>>::from(r.$param) * factor).as_();
            };
        }

        scale!(x);
        scale!(y);
        scale!(w);
        scale!(h);
    }

    #[inline]
    pub fn center(&self) -> [i16; 2] {
        [
            self.0.x + i16::try_from(self.0.w / 2).unwrap(),
            self.0.y + i16::try_from(self.0.h / 2).unwrap(),
        ]
    }

    pub fn with_xy(&self, x: i16, y: i16) -> Self {
        let SDL_Rect { w, h, .. } = self.0;
        Self(SDL_Rect { x, y, w, h })
    }

    pub fn inc_x(&mut self, value: i16) {
        self.0.x += value;
    }

    pub fn inc_y(&mut self, value: i16) {
        self.0.y += value;
    }

    pub fn dec_width(&mut self, value: u16) {
        self.0.w -= value;
    }
}

impl From<SDL_Rect> for Rect {
    fn from(rect: SDL_Rect) -> Self {
        Self(rect)
    }
}

impl AsRef<SDL_Rect> for Rect {
    fn as_ref(&self) -> &SDL_Rect {
        &self.0
    }
}

impl AsMut<SDL_Rect> for Rect {
    fn as_mut(&mut self) -> &mut SDL_Rect {
        &mut self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RectRef<'a>(&'a SDL_Rect);

impl RectRef<'_> {
    pub fn as_ptr(&self) -> *const SDL_Rect {
        self.0
    }
}

impl<'a> From<&'a SDL_Rect> for RectRef<'a> {
    fn from(rect: &'a SDL_Rect) -> Self {
        Self(rect)
    }
}

impl<'a> From<&'a Rect> for RectRef<'a> {
    fn from(rect: &'a Rect) -> Self {
        RectRef(&rect.0)
    }
}

impl<'a> AsRef<SDL_Rect> for RectRef<'a> {
    fn as_ref(&self) -> &'a SDL_Rect {
        self.0
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RectMut<'a>(&'a mut SDL_Rect);

impl RectMut<'_> {
    pub fn as_ptr(&self) -> *const SDL_Rect {
        self.0
    }

    pub fn as_mut_ptr(&mut self) -> *mut SDL_Rect {
        self.0
    }
}

impl<'a> From<&'a mut SDL_Rect> for RectMut<'a> {
    fn from(rect: &'a mut SDL_Rect) -> Self {
        Self(rect)
    }
}

impl<'a> From<&'a mut Rect> for RectMut<'a> {
    fn from(rect: &'a mut Rect) -> Self {
        RectMut(&mut rect.0)
    }
}
