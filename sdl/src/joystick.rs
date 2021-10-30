use std::{
    cell::RefCell, convert::TryInto, ffi::CStr, ops::Not, os::raw::c_int, ptr::NonNull, rc::Rc,
};

use sdl_sys::{
    SDL_Joystick, SDL_JoystickClose, SDL_JoystickEventState, SDL_JoystickName, SDL_JoystickNumAxes,
    SDL_JoystickNumButtons, SDL_JoystickOpen, SDL_NumJoysticks, SDL_ENABLE,
};

type ActiveJoysticks = Rc<RefCell<Vec<c_int>>>;

#[derive(Debug, Default)]
pub struct JoystickSystem {
    active_joysticks: ActiveJoysticks,
}

pub struct Joystick {
    active_joysticks: ActiveJoysticks,
    inner: NonNull<SDL_Joystick>,
    index: c_int,
}

impl JoystickSystem {
    pub fn num_joysticks(&self) -> Option<u32> {
        let num = unsafe { SDL_NumJoysticks() };
        num.try_into().ok()
    }

    pub fn open(&self, index: u32) -> Option<Joystick> {
        let index = index.try_into().expect("invalid joystick index");
        let joystick = unsafe { SDL_JoystickOpen(index) };

        NonNull::new(joystick).map(move |inner| {
            let mut active_joysticks = self.active_joysticks.borrow_mut();
            active_joysticks.push(index);
            drop(active_joysticks);

            let active_joysticks = Rc::clone(&self.active_joysticks);

            Joystick {
                active_joysticks,
                inner,
                index,
            }
        })
    }

    pub fn enable_event_polling(&self) {
        unsafe {
            SDL_JoystickEventState(SDL_ENABLE);
        }
    }
}

impl Joystick {
    pub fn name(&self) -> Option<&CStr> {
        unsafe {
            let ptr = SDL_JoystickName(self.index);
            ptr.is_null().not().then(|| CStr::from_ptr(ptr))
        }
    }

    pub fn axes(&self) -> u16 {
        let axes = unsafe { SDL_JoystickNumAxes(self.inner.as_ptr()) };
        axes.try_into().unwrap()
    }

    pub fn buttons(&self) -> u16 {
        let buttons = unsafe { SDL_JoystickNumButtons(self.inner.as_ptr()) };
        buttons.try_into().unwrap()
    }
}

impl Drop for Joystick {
    fn drop(&mut self) {
        unsafe { SDL_JoystickClose(self.inner.as_ptr()) }
        let mut active_joysticks = self.active_joysticks.borrow_mut();
        let joystick_index = active_joysticks
            .iter()
            .position(|&joy_index| joy_index == self.index)
            .unwrap();
        active_joysticks.remove(joystick_index);
    }
}
