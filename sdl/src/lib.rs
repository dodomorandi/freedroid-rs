pub mod convert;
pub mod cursor;
pub mod event;
pub mod joystick;
pub mod keyboard;
pub mod mixer;
pub mod pixel;
pub mod rect;
pub mod rwops;
pub mod surface;
pub mod system_window_manager;
pub mod video;

use std::{
    ffi::{CStr, CString},
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::NonNull,
    sync::atomic::{AtomicBool, Ordering},
};

pub use cursor::Cursor;
pub use event::Event;
pub use joystick::Joystick;
pub use mixer::Mixer;
pub use pixel::Pixel;
pub use rect::Rect;
pub use rwops::RwOpsOwned;
use sdl_sys::{
    IMG_Load, SDL_GetError, SDL_INIT_AUDIO, SDL_INIT_JOYSTICK, SDL_InitSubSystem, SDL_PushEvent,
    SDL_Quit, SDL_WaitEvent, SDL_WarpMouse, SDL_version,
};
pub use surface::{ColorKeyFlag, FrameBuffer, Rgba, Surface};
pub use video::{Video, VideoModeFlags};

use once_cell::unsync::OnceCell;

// Temporary
#[derive(Debug)]
pub struct Timer;

#[derive(Debug)]
pub struct Sdl<V, T, J, M>
where
    V: Quittable,
    T: Quittable,
    J: Quittable,
    M: Quittable,
{
    pub video: V,
    pub timer: T,
    pub joystick: J,
    pub mixer: M,
    _marker: PhantomData<*const ()>,
}

impl<V, T, J, M> Drop for Sdl<V, T, J, M>
where
    V: Quittable,
    T: Quittable,
    J: Quittable,
    M: Quittable,
{
    fn drop(&mut self) {
        self.video.quit();
        self.timer.quit();
        self.joystick.quit();
        self.mixer.quit();

        unsafe {
            SDL_Quit();
        }

        INITIALIZED.store(false, Ordering::Release);
    }
}

mod private {
    use sdl_sys::{SDL_INIT_AUDIO, SDL_INIT_JOYSTICK, SDL_INIT_TIMER, SDL_VideoQuit};

    use crate::joystick;

    use super::{Mixer, OnceCell, Timer, Video};

    pub trait Quittable {
        fn quit(&mut self) {}
    }

    impl Quittable for Video {
        fn quit(&mut self) {
            unsafe { SDL_VideoQuit() }
        }
    }
    impl Quittable for OnceCell<Video> {
        fn quit(&mut self) {
            if self.take().is_some() {
                unsafe { SDL_VideoQuit() }
            }
        }
    }

    impl Quittable for Timer {}
    impl Quittable for OnceCell<Timer> {
        fn quit(&mut self) {
            if self.take().is_some() {
                unsafe { sdl_sys::SDL_QuitSubSystem(SDL_INIT_TIMER as u32) }
            }
        }
    }

    impl Quittable for joystick::System {}
    impl Quittable for OnceCell<joystick::System> {
        fn quit(&mut self) {
            if self.take().is_some() {
                unsafe { sdl_sys::SDL_QuitSubSystem(SDL_INIT_JOYSTICK as u32) }
            }
        }
    }

    impl Quittable for Mixer {}
    impl Quittable for OnceCell<Mixer> {
        fn quit(&mut self) {
            if self.take().is_some() {
                unsafe { sdl_sys::SDL_QuitSubSystem(SDL_INIT_AUDIO as u32) }
            }
        }
    }
}

pub trait Quittable: private::Quittable {}
impl Quittable for Video {}
impl Quittable for Timer {}
impl Quittable for joystick::System {}
impl Quittable for Mixer {}
impl Quittable for OnceCell<Video> {}
impl Quittable for OnceCell<Timer> {}
impl Quittable for OnceCell<joystick::System> {}
impl Quittable for OnceCell<Mixer> {}

impl<V, T, J, M> Sdl<V, T, J, M>
where
    V: Quittable,
    T: Quittable,
    J: Quittable,
    M: Quittable,
{
    pub fn get_error(&self) -> CString {
        // SAFETY
        // [SDL_GetError] always return a valid C string, even without errors.
        // is taken.
        unsafe { CStr::from_ptr(SDL_GetError()) }.to_owned()
    }

    pub fn delay_ms(&self, duration_ms: u32) {
        unsafe { sdl_sys::SDL_Delay(duration_ms) };
    }

    /// Get the number of milliseconds since the SDL library initialization.
    /// Note that this value wraps if the program runs for more than ~49 days.
    pub fn ticks_ms(&self) -> u32 {
        unsafe { sdl_sys::SDL_GetTicks() }
    }

    pub fn cursor(&self) -> cursor::Unassociated<'_> {
        cursor::Unassociated::new()
    }

    pub fn next_event(&self) -> Option<Event> {
        let event = unsafe {
            let mut event = MaybeUninit::<sdl_sys::SDL_Event>::uninit();
            if sdl_sys::SDL_PollEvent(event.as_mut_ptr()) == 0 {
                return None;
            }

            event.assume_init()
        };
        Some(Event::from_raw(event))
    }

    #[must_use = "success/failure is given as true/false"]
    pub fn push_even(&self, event: &Event) -> bool {
        unsafe { SDL_PushEvent(&mut event.to_raw()) == 0 }
    }

    pub fn wait_event(&self) -> Result<Event, WaitEventError> {
        let mut event = MaybeUninit::uninit();
        let result = unsafe { SDL_WaitEvent(event.as_mut_ptr()) };
        if result != 1 {
            return Err(WaitEventError::Sdl);
        }

        let event = unsafe { event.assume_init() }.try_into()?;
        Ok(event)
    }

    pub fn load_image_from_c_str_path<'a>(&'a self, path: &CStr) -> Option<Surface<'a>> {
        NonNull::new(unsafe { IMG_Load(path.as_ptr()) })
            .map(|ptr| unsafe { Surface::from_ptr(ptr) })
    }

    pub fn warp_mouse(&self, x: u16, y: u16) {
        unsafe { SDL_WarpMouse(x, y) }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum WaitEventError {
    #[error("unable to convert SDL error to wrapped error: {0}")]
    InvalidEvent(#[from] event::InvalidRaw),
    #[error("SDL error")]
    Sdl,
}

impl<V, T, M> Sdl<V, T, OnceCell<joystick::System>, M>
where
    V: Quittable,
    T: Quittable,
    M: Quittable,
{
    pub fn init_joystick(&self) -> Option<&joystick::System> {
        self.joystick
            .get_or_try_init(|| unsafe {
                let ret = SDL_InitSubSystem(SDL_INIT_JOYSTICK as u32);
                if ret == 0 {
                    Ok(joystick::System::default())
                } else {
                    Err(())
                }
            })
            .ok()
    }
}

impl<V, T, J> Sdl<V, T, J, OnceCell<Mixer>>
where
    V: Quittable,
    T: Quittable,
    J: Quittable,
{
    pub fn init_audio(&self) -> Option<&Mixer> {
        self.mixer
            .get_or_try_init(|| unsafe {
                let ret = SDL_InitSubSystem(SDL_INIT_AUDIO as u32);
                if ret == 0 { Ok(Mixer) } else { Err(()) }
            })
            .ok()
    }
}

#[derive(Debug)]
pub struct Builder<V, T, J, M> {
    value: u32,
    video: V,
    timer: T,
    joystick: J,
    mixer: M,
}

static INITIALIZED: AtomicBool = AtomicBool::new(false);

#[must_use]
pub fn init()
-> Builder<OnceCell<Video>, OnceCell<Timer>, OnceCell<joystick::System>, OnceCell<Mixer>> {
    Builder {
        value: 0,
        video: OnceCell::new(),
        timer: OnceCell::new(),
        joystick: OnceCell::new(),
        mixer: OnceCell::new(),
    }
}

impl<V, T, J, M> Builder<V, T, J, M> {
    pub fn video(self) -> Builder<Video, T, J, M> {
        let Self {
            mut value,
            video: _,
            timer,
            joystick,
            mixer: audio,
        } = self;
        value |= u32::try_from(sdl_sys::SDL_INIT_VIDEO).unwrap();
        Builder {
            value,
            video: Video::new(),
            timer,
            joystick,
            mixer: audio,
        }
    }

    pub fn timer(self) -> Builder<V, Timer, J, M> {
        let Self {
            mut value,
            video,
            timer: _,
            joystick,
            mixer: audio,
        } = self;
        value |= u32::try_from(sdl_sys::SDL_INIT_TIMER).unwrap();
        Builder {
            value,
            video,
            timer: Timer,
            joystick,
            mixer: audio,
        }
    }

    pub fn audio(self) -> Builder<V, T, J, Mixer> {
        let Self {
            mut value,
            video,
            timer,
            joystick,
            mixer: _,
        } = self;
        value |= u32::try_from(sdl_sys::SDL_INIT_AUDIO).unwrap();
        Builder {
            value,
            video,
            timer,
            joystick,
            mixer: Mixer,
        }
    }
}

impl<V, T, J, M> Builder<V, T, J, M>
where
    V: Quittable,
    T: Quittable,
    J: Quittable,
    M: Quittable,
{
    pub fn build(self) -> Option<Sdl<V, T, J, M>> {
        INITIALIZED
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Relaxed)
            .ok()?;

        let Self {
            value,
            video,
            timer,
            joystick,
            mixer,
        } = self;

        let ret = unsafe { sdl_sys::SDL_Init(value) };
        (ret == 0).then_some(Sdl {
            video,
            timer,
            joystick,
            mixer,
            _marker: PhantomData,
        })
    }
}

/// Get the last SDL error.
///
/// # Safety
/// No other calls to SDL function can be made until the function returns.
/// The function is not thread-safe.
pub unsafe fn get_error<F, T>(f: F) -> T
where
    F: for<'a> FnOnce(&'a CStr) -> T,
{
    // SAFETY
    // [SDL_GetError] always return a valid C string, even without errors.
    let err = unsafe { CStr::from_ptr(SDL_GetError()) };
    f(err)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Version {
    major: u8,
    minor: u8,
    patch: u8,
}

impl Version {
    #[must_use]
    pub fn to_raw(&self) -> SDL_version {
        let &Self {
            major,
            minor,
            patch,
        } = self;

        SDL_version {
            major,
            minor,
            patch,
        }
    }
}

impl From<SDL_version> for Version {
    fn from(version: SDL_version) -> Self {
        Self::from(&version)
    }
}

impl From<&SDL_version> for Version {
    fn from(version: &SDL_version) -> Self {
        let &SDL_version {
            major,
            minor,
            patch,
        } = version;

        Self {
            major,
            minor,
            patch,
        }
    }
}

mod sealed {
    pub trait Sealed {}
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;

    use super::*;

    static SDL_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[test]
    fn one_instance_allowed() {
        let _sdl_lock = SDL_MUTEX.lock().unwrap();

        let _sdl = init().build().unwrap();
        assert!(init().build().is_none(), "only one SDL instance is allowed");
    }

    #[test]
    fn can_reinitialize() {
        let _sdl_lock = SDL_MUTEX.lock().unwrap();

        let sdl = init().build().unwrap();
        drop(sdl);
        let _sdl = init().build().unwrap();
    }
}
