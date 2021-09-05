use sdl_sys::SDL_Rect;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Rect(SDL_Rect);

impl Rect {
    pub fn as_ptr(&self) -> *const SDL_Rect {
        &self.0
    }

    pub fn as_mut_ptr(&mut self) -> *mut SDL_Rect {
        &mut self.0
    }

    pub fn as_rect_ref(&self) -> RectRef {
        RectRef(&self.0)
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
