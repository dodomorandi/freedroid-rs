use std::{
    cell::Cell,
    ffi::CStr,
    marker::PhantomData,
    num::NonZeroU8,
    ops::Not,
    os::raw::{c_char, c_int},
    ptr::{null_mut, NonNull},
};

use bitflags::bitflags;
use sdl_sys::{
    SDL_GetVideoInfo, SDL_SetGamma, SDL_SetVideoMode, SDL_VideoDriverName, SDL_VideoInfo,
    SDL_WM_SetCaption, SDL_WM_SetIcon, SDL_ANYFORMAT, SDL_ASYNCBLIT, SDL_DOUBLEBUF, SDL_FULLSCREEN,
    SDL_HWPALETTE, SDL_HWSURFACE, SDL_NOFRAME, SDL_OPENGL, SDL_OPENGLBLIT, SDL_RESIZABLE,
    SDL_SWSURFACE,
};

use crate::{convert, pixel, FrameBuffer, Surface};

#[derive(Debug)]
pub struct Video {
    refs_to_frame_buffer: Cell<u8>,
}

impl Video {
    pub(crate) const fn new() -> Self {
        Self {
            refs_to_frame_buffer: Cell::new(0),
        }
    }

    pub fn set_video_mode(
        &self,
        width: c_int,
        height: c_int,
        bits_per_pixel: Option<NonZeroU8>,
        flags: VideoModeFlags,
    ) -> Option<FrameBuffer> {
        assert!(
            self.refs_to_frame_buffer.get() == 0,
            "Video::set_video_mode is called when references to video mode are alive"
        );
        unsafe {
            let surface_ptr = SDL_SetVideoMode(
                width,
                height,
                bits_per_pixel.map_or(0, std::num::NonZeroU8::get).into(),
                flags.bits(),
            );
            NonNull::new(surface_ptr).map(|surface_ptr| {
                self.refs_to_frame_buffer.set(1);
                FrameBuffer::from_ptr_and_refcount(surface_ptr, &self.refs_to_frame_buffer)
            })
        }
    }

    pub fn get_video_info(&self) -> Option<InfoRef<'_>> {
        // Safety: `SDL_GetVideoInfo` is not thread safe because it gets the info from a static
        // variable, but `Video` is `!Sync` and `Info` holds a reference to `Video`, so it is not
        // possible to trigger a data race from multiple threads or to change the value because
        // `set_video_mode` can only be called once.
        let video_info = unsafe { SDL_GetVideoInfo() };
        NonNull::new(video_info.cast_mut()).map(|inner| InfoRef {
            inner,
            _marker: PhantomData,
        })
    }

    #[must_use = "success/failure is given as true/false"]
    pub fn set_gamma(&self, red: f32, green: f32, blue: f32) -> bool {
        unsafe { SDL_SetGamma(red, green, blue) == 0 }
    }

    pub fn get_driver_name<'a>(&self, buffer: &'a mut [u8]) -> Option<&'a CStr> {
        if buffer.is_empty() {
            return None;
        }

        let len = buffer.len().try_into().unwrap_or(c_int::MAX);
        let pointer = unsafe { SDL_VideoDriverName(buffer.as_mut_ptr().cast::<c_char>(), len) };
        pointer
            .is_null()
            .not()
            .then(|| unsafe { CStr::from_ptr(buffer.as_ptr().cast::<c_char>()) })
    }

    pub fn window_manager(&self) -> WindowManager<'_> {
        WindowManager(self)
    }
}

#[derive(Debug)]
pub struct WindowManager<'a>(&'a Video);

impl WindowManager<'_> {
    #[allow(clippy::unused_self)]
    pub fn set_caption(&self, title: &CStr, icon: &CStr) {
        unsafe { SDL_WM_SetCaption(title.as_ptr(), icon.as_ptr()) }
    }

    pub fn set_icon(&self, icon: &mut Surface, mask: Option<&mut [u8]>) {
        assert!(
            self.0.refs_to_frame_buffer.get() == 0,
            "SDL video wm set_icon must be called before set_video_mode"
        );

        if let Some(mask) = mask.as_ref() {
            assert_eq!(mask.len(), (icon.height() * (icon.width() / 8)).into());
        }

        unsafe {
            SDL_WM_SetIcon(
                icon.as_mut_ptr(),
                mask.map_or(null_mut(), <[u8]>::as_mut_ptr),
            );
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct VideoModeFlags: u32 {
        const SOFTWARE_SURFACE = convert::i32_to_u32(SDL_SWSURFACE);
        const HARDWARE_SURFACE = convert::i32_to_u32(SDL_HWSURFACE);
        const ASYNC_BLIT = convert::i32_to_u32(SDL_ASYNCBLIT);
        const ANY_FORMAT = convert::i32_to_u32(SDL_ANYFORMAT);
        const HARDWARE_PALETTE = convert::i32_to_u32(SDL_HWPALETTE);
        const DOUBLE_BUFFER = convert::i32_to_u32(SDL_DOUBLEBUF);
        const FULLSCREEN = convert::i64_to_u32(SDL_FULLSCREEN);
        const OPENGL = convert::i32_to_u32(SDL_OPENGL);
        const OPENGL_BLIT = convert::i32_to_u32(SDL_OPENGLBLIT);
        const RESIZABLE = convert::i32_to_u32(SDL_RESIZABLE);
        const NO_FRAME = convert::i32_to_u32(SDL_NOFRAME);
    }
}

#[derive(Debug)]
pub struct InfoRef<'a> {
    inner: NonNull<SDL_VideoInfo>,
    _marker: PhantomData<&'a ()>,
}

macro_rules! impl_flag {
    ($(
        $flag:ident
    ),* $(,)?) => {
        $(
            #[inline]
            #[must_use]
            pub fn $flag(&self) -> bool {
                self.inner().$flag() != 0
            }
        )*
    };
}

impl<'a> InfoRef<'a> {
    #[inline]
    #[must_use]
    fn inner(&self) -> &'a SDL_VideoInfo {
        // Safety: the pointer is returned by `SDL_GetVideoInfo`.
        unsafe { self.inner.as_ref() }
    }

    #[inline]
    #[must_use]
    pub fn format(&self) -> pixel::FormatRef<'a> {
        // Safety: the `vfmt` is set to point to an sdl surface by `SDL_SetVideoMode`.
        let vfmt = unsafe { NonNull::new_unchecked(self.inner().vfmt) };

        // Safety: there should not be possible to have a mutable reference to this pointed pixel
        // format.
        unsafe { pixel::FormatRef::from_raw(vfmt) }
    }

    impl_flag!(hw_available, wm_available, blit_hw, blit_sw, blit_fill);

    #[inline]
    #[must_use]
    pub fn blit_hw_colorkey(&self) -> bool {
        self.inner().blit_hw_CC() != 0
    }

    #[inline]
    #[must_use]
    pub fn blit_hw_alpha(&self) -> bool {
        self.inner().blit_hw_A() != 0
    }

    #[inline]
    #[must_use]
    pub fn blit_sw_colorkey(&self) -> bool {
        self.inner().blit_sw_CC() != 0
    }

    #[inline]
    #[must_use]
    pub fn blit_sw_alpha(&self) -> bool {
        self.inner().blit_sw_A() != 0
    }

    #[inline]
    #[must_use]
    pub fn video_mem(&self) -> u32 {
        self.inner().video_mem
    }
}
