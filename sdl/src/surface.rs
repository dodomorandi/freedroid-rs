pub mod lock;

use std::{
    cell::Cell,
    ffi::c_void,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    os::raw::c_int,
    ptr::{self, null_mut, NonNull},
};

use bitflags::bitflags;
use sdl_sys::{
    rotozoomSurface, zoomSurface, SDL_CreateRGBSurface, SDL_DisplayFormat, SDL_DisplayFormatAlpha,
    SDL_FillRect, SDL_Flip, SDL_FreeSurface, SDL_GetClipRect, SDL_PixelFormat, SDL_Rect,
    SDL_SaveBMP_RW, SDL_SetAlpha, SDL_SetClipRect, SDL_SetColorKey, SDL_Surface, SDL_UpdateRect,
    SDL_UpdateRects, SDL_UpperBlit, SDL_bool_SDL_TRUE, SDL_ASYNCBLIT, SDL_HWACCEL, SDL_HWSURFACE,
    SDL_PREALLOC, SDL_RLEACCEL, SDL_RLEACCELOK, SDL_SRCALPHA, SDL_SRCCOLORKEY,
};

use crate::{
    get_error,
    pixel::{self, Pixel, Pixels},
    rect,
    rwops::RwOpsCapability,
    Rect,
};

use self::lock::ResultMaybeLocked;

#[derive(Debug)]
pub struct Generic<'sdl, const FREEABLE: bool> {
    pointer: NonNull<SDL_Surface>,
    _marker: PhantomData<&'sdl *const ()>,
}

impl<'sdl, const FREEABLE: bool> Generic<'sdl, FREEABLE> {
    /// # Safety
    /// * A [`Sdl`] instance must be alive.
    /// * `pointer` must point to a valid [`SDL_Surface`].
    /// * No live references to pointed data must exist.
    /// * The ownership of the pointed [`SDL_Surface`] is transferred to `GenericSurface`, therefore
    ///   the structure **must not** be freed.
    ///
    /// [`Sdl`]: crate::Sdl
    #[must_use]
    pub unsafe fn from_ptr(pointer: NonNull<SDL_Surface>) -> Self {
        Self {
            pointer,
            _marker: PhantomData,
        }
    }

    #[must_use]
    pub fn must_lock(&self) -> bool {
        let surface = unsafe { self.pointer.as_ref() };
        surface.offset != 0
            && (surface.flags & (SDL_HWSURFACE as u32 | SDL_ASYNCBLIT as u32 | SDL_RLEACCEL as u32))
                != 0
    }

    pub fn lock(&mut self) -> ResultMaybeLocked<'_, 'sdl, FREEABLE> {
        if self.must_lock() {
            ResultMaybeLocked::Locked(lock::Guard::new(self))
        } else {
            ResultMaybeLocked::Unlocked(Usable(self))
        }
    }

    #[must_use]
    pub fn format(&self) -> pixel::FormatRef {
        // SAFETY: format becomes null once SDL_FreeSurface is called, which is performed only on
        // drop.
        unsafe {
            let pointer = NonNull::new_unchecked(self.pointer.as_ref().format);
            pixel::FormatRef::from_raw(pointer)
        }
    }

    #[must_use]
    pub fn raw(&self) -> UsableSurfaceRaw<'_, '_, FREEABLE> {
        UsableSurfaceRaw(self)
    }

    #[must_use]
    pub fn height(&self) -> u16 {
        self.raw()
            .height()
            .max(0)
            .try_into()
            .expect("invalid SDL surface height for architecture")
    }

    #[must_use]
    pub fn width(&self) -> u16 {
        self.raw()
            .width()
            .max(0)
            .try_into()
            .expect("invalid SDL surface width for architecture")
    }

    #[must_use]
    pub fn flags(&self) -> u32 {
        self.raw().flags()
    }

    #[must_use]
    pub fn as_ptr(&self) -> *const SDL_Surface {
        self.pointer.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut SDL_Surface {
        self.pointer.as_ptr()
    }

    pub fn blit<const TO_FREEABLE: bool>(&mut self, to_surface: &mut Generic<TO_FREEABLE>) {
        self.blit_inner(None::<&Rect>, to_surface, None::<&mut Rect>);
    }

    pub fn blit_to<'to, ToRect, const TO_FREEABLE: bool>(
        &mut self,
        to_surface: &mut Generic<TO_FREEABLE>,
        to: ToRect,
    ) where
        ToRect: Into<rect::RefMut<'to>>,
    {
        self.blit_inner(None::<&Rect>, to_surface, Some(to));
    }

    pub fn blit_from<'from, FromRect, const TO_FREEABLE: bool>(
        &mut self,
        from: FromRect,
        to_surface: &mut Generic<TO_FREEABLE>,
    ) where
        FromRect: Into<rect::Ref<'from>>,
    {
        self.blit_inner(Some(from), to_surface, None::<&mut Rect>);
    }

    pub fn blit_from_to<'from, 'to, FromRect, ToRect, const TO_FREEABLE: bool>(
        &mut self,
        from: FromRect,
        to_surface: &mut Generic<TO_FREEABLE>,
        to: ToRect,
    ) where
        FromRect: Into<rect::Ref<'from>>,
        ToRect: Into<rect::RefMut<'to>>,
    {
        self.blit_inner(Some(from), to_surface, Some(to));
    }

    fn blit_inner<'from, 'to, FromRect, ToRect, const TO_FREEABLE: bool>(
        &mut self,
        from: Option<FromRect>,
        to_surface: &mut Generic<TO_FREEABLE>,
        to: Option<ToRect>,
    ) where
        FromRect: Into<rect::Ref<'from>>,
        ToRect: Into<rect::RefMut<'to>>,
    {
        // Possible errors coming from SDL_UpperBlit:
        // - src or dest null pointers -- cannot happen
        // - src or dest surfaces locked -- cannot happen
        // These are the main errors expected from SDL_UpperBlit. SDL_LowerBlit seems to possibly
        // return an error, but I am not really sure. Surely it is possible to trigger an out of
        // memory error, but in that case it is ok to panic.

        // # SAFETY
        // srcrect is not modified internally
        let result = unsafe {
            SDL_UpperBlit(
                self.pointer.as_mut(),
                from.map_or(null_mut(), |rect| rect.into().as_ptr().cast_mut()),
                to_surface.pointer.as_mut(),
                to.map_or(null_mut(), |rect| rect.into().as_mut_ptr()),
            )
        };

        debug_assert!(result <= 0);
        if result < 0 {
            // Safety: no other SDL function will be used -- we are panicking.
            unsafe {
                get_error(|err| {
                    panic!(
                        "SDL_UpperBlit returned an unexpected error: {}",
                        err.to_string_lossy(),
                    );
                });
            }
        }
    }

    pub fn clear_clip_rect(&mut self) -> bool {
        self.set_clip_rect_inner(None::<&Rect>)
    }

    pub fn set_clip_rect<'a, R>(&mut self, rect: R) -> bool
    where
        R: Into<rect::Ref<'a>>,
    {
        self.set_clip_rect_inner(Some(rect))
    }

    fn set_clip_rect_inner<'a, R>(&mut self, rect: Option<R>) -> bool
    where
        R: Into<rect::Ref<'a>>,
    {
        let rect = rect.map_or(ptr::null(), |rect| rect.into().as_ptr());
        let result = unsafe { SDL_SetClipRect(self.pointer.as_ptr(), rect) };
        result == SDL_bool_SDL_TRUE
    }

    pub fn zoom(&mut self, x: f64, y: f64, smooth: bool) -> Option<Surface<'sdl>> {
        let ptr = unsafe { zoomSurface(self.pointer.as_ptr(), x, y, smooth.into()) };
        NonNull::new(ptr).map(|ptr| unsafe { Surface::from_ptr(ptr) })
    }

    pub fn rotozoom(&mut self, angle: f64, zoom: f64, smooth: bool) -> Option<Surface<'sdl>> {
        let ptr = unsafe { rotozoomSurface(self.pointer.as_ptr(), angle, zoom, smooth.into()) };
        NonNull::new(ptr).map(|ptr| unsafe { Surface::from_ptr(ptr) })
    }

    pub fn display_format(&mut self) -> Option<Surface<'sdl>> {
        NonNull::new(unsafe { SDL_DisplayFormat(self.pointer.as_ptr()) })
            .map(|ptr| unsafe { Surface::from_ptr(ptr) })
    }

    pub fn display_format_alpha(&mut self) -> Option<Surface<'sdl>> {
        NonNull::new(unsafe { SDL_DisplayFormatAlpha(self.pointer.as_ptr()) })
            .map(|ptr| unsafe { Surface::from_ptr(ptr) })
    }

    pub fn fill(&mut self, color: Pixel) -> Result<(), i32> {
        self.fill_with_inner(None, color)
    }

    pub fn fill_with(&mut self, rect: &Rect, color: Pixel) -> Result<(), i32> {
        self.fill_with_inner(Some(rect), color)
    }

    #[inline]
    fn fill_with_inner(&mut self, rect: Option<&Rect>, color: Pixel) -> Result<(), i32> {
        let result = unsafe {
            SDL_FillRect(
                self.pointer.as_ptr(),
                rect.map_or(null_mut(), |rect| {
                    (rect.as_ref() as *const SDL_Rect).cast_mut()
                }),
                color.0,
            )
        };
        if result == 0 {
            Ok(())
        } else {
            Err(result)
        }
    }

    pub fn update_rect(&mut self, rect: &Rect) {
        let rect = &rect.0;

        // Safety: we are taking self as mut ref, therefore the an instance of [`SurfaceLockGuard`]
        // cannot exist.
        unsafe {
            SDL_UpdateRect(
                self.pointer.as_ptr(),
                rect.x.into(),
                rect.y.into(),
                rect.w.into(),
                rect.h.into(),
            );
        }
    }

    pub fn update_rects(&mut self, rects: &[Rect]) {
        // Safety:
        //
        // - we are taking self as mut ref, therefore the an instance of [`SurfaceLockGuard`]
        // cannot exist.
        // - [`Rect`] is transparent, therefore it is safe to cast from `*const Rect` to `*const
        // SDL_Rect`.
        // - `SDL_UpdateRects` does not change rects, function signature is not const-correct.
        unsafe {
            SDL_UpdateRects(
                self.pointer.as_ptr(),
                rects.len().try_into().expect("too many rectangles"),
                rects.as_ptr().cast::<SDL_Rect>().cast_mut(),
            );
        }
    }

    #[must_use = "success/failure is given as true/false"]
    pub fn set_color_key(&mut self, flag: ColorKeyFlag, key: Pixel) -> bool {
        unsafe { SDL_SetColorKey(self.pointer.as_ptr(), flag.bits(), key.0) == 0 }
    }

    #[must_use]
    pub fn get_clip_rect(&self) -> Rect {
        let mut rect = Rect::default();
        unsafe { SDL_GetClipRect(self.pointer.as_ptr(), rect.as_mut_ptr()) };
        rect
    }

    #[must_use = "success/failure is given as true/false"]
    pub fn set_alpha(&mut self, flag: ColorKeyFlag, alpha: u8) -> bool {
        unsafe { SDL_SetAlpha(self.pointer.as_ptr(), flag.bits(), alpha) == 0 }
    }

    #[must_use = "success/failure is given as true/false"]
    pub fn save_bmp_rw<R: RwOpsCapability>(&self, rw: &mut R) -> bool {
        let rw = rw.as_inner();
        unsafe { SDL_SaveBMP_RW(self.pointer.as_ptr(), rw.as_ptr(), 0) == 0 }
    }
}

bitflags! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ColorKeyFlag: u32 {
        const HW_ACCEL = SDL_HWACCEL as u32;
        const SRC_COLOR_KEY = SDL_SRCCOLORKEY as u32;
        const RLE_ACCEL_OK = SDL_RLEACCELOK as u32;
        const RLE_ACCEL = SDL_RLEACCEL as u32;
        const SRC_ALPHA = SDL_SRCALPHA as u32;
        const PRE_ALLOC = SDL_PREALLOC as u32;
    }
}

impl Generic<'_, true> {
    #[must_use]
    pub fn create_rgb(width: u32, height: u32, depth: u8, mask: Rgba<u32>) -> Option<Self> {
        let width = width.try_into().expect("width greater than c_int::MAX");
        let height = height.try_into().expect("height greater than c_int::MAX");
        let depth = depth.into();
        let Rgba {
            red,
            green,
            blue,
            alpha,
        } = mask;

        let ptr = unsafe { SDL_CreateRGBSurface(0, width, height, depth, red, green, blue, alpha) };
        NonNull::new(ptr).map(|ptr| unsafe { Self::from_ptr(ptr) })
    }
}

impl Generic<'_, false> {
    #[must_use]
    pub fn flip(&mut self) -> bool {
        unsafe { SDL_Flip(self.pointer.as_ptr()) == 0 }
    }
}

impl<const FREEABLE: bool> Drop for Generic<'_, FREEABLE> {
    fn drop(&mut self) {
        if FREEABLE {
            unsafe { SDL_FreeSurface(self.pointer.as_ptr()) }
        }
    }
}

/// A [`Generic`] that must be freed on drop.
pub type Surface<'sdl> = Generic<'sdl, true>;

/// A [`Generic`] that must not be freed on drop.
#[derive(Debug)]
pub struct FrameBuffer<'sdl> {
    inner: Generic<'sdl, false>,
    refcount: &'sdl Cell<u8>,
}

impl<'sdl> FrameBuffer<'sdl> {
    /// # Safety
    /// * An [`Sdl`] instance must be alive.
    /// * `pointer` must point to a valid [`SDL_Surface`].
    /// * No live references to pointed data must exist.
    /// * The ownership of the pointed [`SDL_Surface`] is transferred to `GenericSurface`, therefore
    ///   the structure **must not** be freed.
    /// * `refcount` must be in the [`Video`] struct which holds the count of the references for
    ///   the SDL framebuffer pointed by `pointer`.
    ///
    /// [`Sdl`]: crate::Sdl
    /// [`Video`]: crate::video::Video
    pub unsafe fn from_ptr_and_refcount(
        pointer: NonNull<SDL_Surface>,
        refcount: &'sdl Cell<u8>,
    ) -> Self {
        // Safety: the invariants of the function include the invariants of this call.
        let inner = unsafe { Generic::from_ptr(pointer) };
        Self { inner, refcount }
    }
}

impl Drop for FrameBuffer<'_> {
    fn drop(&mut self) {
        let refcount = self.refcount.get().saturating_sub(1);
        self.refcount.set(refcount);
    }
}

impl<'sdl> Deref for FrameBuffer<'sdl> {
    type Target = Generic<'sdl, false>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for FrameBuffer<'_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'sdl> AsRef<Generic<'sdl, false>> for FrameBuffer<'sdl> {
    #[inline]
    fn as_ref(&self) -> &Generic<'sdl, false> {
        &self.inner
    }
}

impl<'sdl> AsMut<Generic<'sdl, false>> for FrameBuffer<'sdl> {
    #[inline]
    fn as_mut(&mut self) -> &mut Generic<'sdl, false> {
        &mut self.inner
    }
}

#[derive(Debug)]
pub struct Usable<'a, 'sdl, const FREEABLE: bool>(&'a mut Generic<'sdl, FREEABLE>);

impl<'a, 'sdl, const FREEABLE: bool> Usable<'a, 'sdl, FREEABLE> {
    #[must_use]
    pub fn format(&self) -> pixel::FormatRef {
        self.0.format()
    }

    pub fn pixels(&mut self) -> Pixels<'_, 'a, 'sdl, FREEABLE> {
        Pixels::new(self)
    }

    #[must_use]
    pub fn raw(&self) -> UsableSurfaceRaw<'_, '_, FREEABLE> {
        UsableSurfaceRaw(self.0)
    }

    #[must_use]
    pub fn height(&self) -> u16 {
        self.0.height()
    }

    #[must_use]
    pub fn width(&self) -> u16 {
        self.0.width()
    }

    #[must_use]
    pub fn buffer_size(&self) -> usize {
        let height: usize = self.height().into();
        let pitch = usize::from(self.raw().pitch());

        height
            .checked_mul(pitch)
            .expect("SDL surface with a buffer too big")
    }
}

#[derive(Debug)]
pub struct UsableSurfaceRaw<'a, 'sdl, const FREEABLE: bool>(&'a Generic<'sdl, FREEABLE>);

impl<'a, 'sdl, const FREEABLE: bool> UsableSurfaceRaw<'a, 'sdl, FREEABLE> {
    #[must_use]
    pub fn flags(&self) -> u32 {
        unsafe { self.0.pointer.as_ref().flags }
    }

    #[must_use]
    pub fn format(&self) -> *const SDL_PixelFormat {
        unsafe { self.0.pointer.as_ref().format }
    }

    #[must_use]
    pub fn width(&self) -> c_int {
        unsafe { self.0.pointer.as_ref().w }
    }

    #[must_use]
    pub fn height(&self) -> c_int {
        unsafe { self.0.pointer.as_ref().h }
    }

    #[must_use]
    pub fn pitch(&self) -> u16 {
        unsafe { self.0.pointer.as_ref().pitch }
    }

    #[must_use]
    pub fn pixels(&self) -> *const c_void {
        unsafe { self.0.pointer.as_ref().pixels }
    }

    #[must_use]
    pub fn clip_rect(&self) -> &SDL_Rect {
        unsafe { &self.0.pointer.as_ref().clip_rect }
    }

    #[must_use]
    pub fn refcount(&self) -> c_int {
        unsafe { self.0.pointer.as_ref().refcount }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgba<T> {
    pub red: T,
    pub green: T,
    pub blue: T,
    pub alpha: T,
}
