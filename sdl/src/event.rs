use std::{ffi::c_void, os::raw::c_int, ptr::NonNull};

use sdl_sys::{
    SDL_Event, SDL_EventType_SDL_ACTIVEEVENT, SDL_EventType_SDL_JOYAXISMOTION,
    SDL_EventType_SDL_JOYBALLMOTION, SDL_EventType_SDL_JOYBUTTONDOWN,
    SDL_EventType_SDL_JOYBUTTONUP, SDL_EventType_SDL_JOYHATMOTION, SDL_EventType_SDL_KEYDOWN,
    SDL_EventType_SDL_KEYUP, SDL_EventType_SDL_MOUSEBUTTONDOWN, SDL_EventType_SDL_MOUSEBUTTONUP,
    SDL_EventType_SDL_MOUSEMOTION, SDL_EventType_SDL_NOEVENT, SDL_EventType_SDL_NUMEVENTS,
    SDL_EventType_SDL_QUIT, SDL_EventType_SDL_SYSWMEVENT, SDL_EventType_SDL_USEREVENT,
    SDL_EventType_SDL_VIDEOEXPOSE, SDL_EventType_SDL_VIDEORESIZE, SDL_JoyAxisEvent,
    SDL_JoyBallEvent, SDL_JoyButtonEvent, SDL_JoyHatEvent, SDL_MouseButtonEvent,
    SDL_MouseMotionEvent, SDL_ResizeEvent, SDL_UserEvent,
};

use crate::{keyboard::KeySym, system_window_manager};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
#[non_exhaustive]
pub enum EventType {
    /// Unused (do not remove)
    None = SDL_EventType_SDL_NOEVENT as u8,
    /// Application loses/gains visibility
    ActiveEvent = SDL_EventType_SDL_ACTIVEEVENT as u8,
    /// Keys pressed
    KeyDown = SDL_EventType_SDL_KEYDOWN as u8,
    /// Keys released
    KeyUp = SDL_EventType_SDL_KEYUP as u8,
    /// Mouse moved
    MouseMotion = SDL_EventType_SDL_MOUSEMOTION as u8,
    /// Mouse button pressed
    MouseButtonDown = SDL_EventType_SDL_MOUSEBUTTONDOWN as u8,
    /// Mouse button released
    MouseButtonUp = SDL_EventType_SDL_MOUSEBUTTONUP as u8,
    /// Joystick axis motion
    JoyAxisMotion = SDL_EventType_SDL_JOYAXISMOTION as u8,
    /// Joystick trackball motion
    JoyBallMotion = SDL_EventType_SDL_JOYBALLMOTION as u8,
    /// Joystick hat position change
    JoyHatMotion = SDL_EventType_SDL_JOYHATMOTION as u8,
    /// Joystick button pressed
    JoyButtonDown = SDL_EventType_SDL_JOYBUTTONDOWN as u8,
    /// Joystick button released
    JoyButtonUp = SDL_EventType_SDL_JOYBUTTONUP as u8,
    /// User-requested quit
    Quit = SDL_EventType_SDL_QUIT as u8,
    /// System specific event
    SysWmEvent = SDL_EventType_SDL_SYSWMEVENT as u8,
    /// User resized video mode
    VideoResize = SDL_EventType_SDL_VIDEORESIZE as u8,
    /// Screen needs to be redrawn
    VideoExpose = SDL_EventType_SDL_VIDEOEXPOSE as u8,
    /// Events SDL_USEREVENT through `MAX_EVENTS - 1` are for your use
    UserEvent1 = SDL_EventType_SDL_USEREVENT as u8,
    UserEvent2 = SDL_EventType_SDL_USEREVENT as u8 + 1,
    UserEvent3 = SDL_EventType_SDL_USEREVENT as u8 + 2,
    UserEvent4 = SDL_EventType_SDL_USEREVENT as u8 + 3,
    UserEvent5 = SDL_EventType_SDL_USEREVENT as u8 + 4,
    UserEvent6 = SDL_EventType_SDL_USEREVENT as u8 + 5,
    UserEvent7 = SDL_EventType_SDL_USEREVENT as u8 + 6,
    UserEvent8 = SDL_EventType_SDL_USEREVENT as u8 + 7,
}

// Just a compile-time check
const _: [u8; EventType::UserEvent8 as usize + 1] = [0u8; EventType::MAX_EVENTS as usize];

impl Default for EventType {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("invalid raw event type")]
pub struct InvalidRawType;

impl EventType {
    pub const MAX_EVENTS: u8 = SDL_EventType_SDL_NUMEVENTS as u8;

    pub fn from_raw(raw_type: u8) -> Result<Self, InvalidRawType> {
        const MAX_USER_EVENTS: u32 = SDL_EventType_SDL_USEREVENT + 7;

        Ok(match raw_type.into() {
            sdl_sys::SDL_EventType_SDL_NOEVENT => Self::None,
            sdl_sys::SDL_EventType_SDL_ACTIVEEVENT => Self::ActiveEvent,
            sdl_sys::SDL_EventType_SDL_KEYDOWN => Self::KeyDown,
            sdl_sys::SDL_EventType_SDL_KEYUP => Self::KeyUp,
            sdl_sys::SDL_EventType_SDL_MOUSEMOTION => Self::MouseMotion,
            sdl_sys::SDL_EventType_SDL_MOUSEBUTTONDOWN => Self::MouseButtonDown,
            sdl_sys::SDL_EventType_SDL_MOUSEBUTTONUP => Self::MouseButtonUp,
            sdl_sys::SDL_EventType_SDL_JOYAXISMOTION => Self::JoyAxisMotion,
            sdl_sys::SDL_EventType_SDL_JOYBALLMOTION => Self::JoyBallMotion,
            sdl_sys::SDL_EventType_SDL_JOYHATMOTION => Self::JoyHatMotion,
            sdl_sys::SDL_EventType_SDL_JOYBUTTONDOWN => Self::JoyButtonDown,
            sdl_sys::SDL_EventType_SDL_JOYBUTTONUP => Self::JoyButtonUp,
            sdl_sys::SDL_EventType_SDL_QUIT => Self::Quit,
            sdl_sys::SDL_EventType_SDL_SYSWMEVENT => Self::SysWmEvent,
            sdl_sys::SDL_EventType_SDL_VIDEORESIZE => Self::VideoResize,
            sdl_sys::SDL_EventType_SDL_VIDEOEXPOSE => Self::VideoExpose,
            event @ SDL_EventType_SDL_USEREVENT..=MAX_USER_EVENTS => {
                match event - SDL_EventType_SDL_USEREVENT {
                    0 => Self::UserEvent1,
                    1 => Self::UserEvent2,
                    2 => Self::UserEvent3,
                    3 => Self::UserEvent4,
                    4 => Self::UserEvent5,
                    5 => Self::UserEvent6,
                    6 => Self::UserEvent7,
                    7 => Self::UserEvent8,
                    _ => unreachable!(),
                }
            }
            _ => return Err(InvalidRawType),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonState {
    Released = 0,
    Pressed = 1,
}

impl ButtonState {
    pub fn from_raw(state: u8) -> Result<Self, InvalidButtonState> {
        Ok(match state {
            0 => Self::Released,
            1 => Self::Pressed,
            _ => return Err(InvalidButtonState),
        })
    }

    pub fn is_pressed(self) -> bool {
        match self {
            Self::Pressed => true,
            Self::Released => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("invalid button state")]
pub struct InvalidButtonState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Active(ActiveEvent),
    Keyboard(KeyboardEvent),
    MouseMotion(MouseMotionEvent),
    MouseButton(MouseButtonEvent),
    JoyAxis(JoyAxisEvent),
    JoyBall(JoyBallEvent),
    JoyHat(JoyHatEvent),
    JoyButton(JoyButtonEvent),
    Resize(ResizeEvent),
    Exposure,
    Quit,
    User(UserEvent),
    SymWindowManager(SysWindowManagerEvent),
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InvalidRawEvent {
    #[error("invalid event type")]
    Type,

    #[error("event type cannot be None")]
    NoneType,

    #[error("event contains an invalid button state")]
    ButtonState,

    #[error("keyboard event contains an invalid keysym")]
    KeySym,

    #[error("joystick hat motion event contains an invalid position")]
    JoyHatPosition,
}

impl Event {
    pub fn try_from_raw(event: SDL_Event) -> Result<Self, InvalidRawEvent> {
        // Safety: `type` is available for each member of the union, therefore is always available.
        let ty = unsafe { EventType::from_raw(event.type_).map_err(|_| InvalidRawEvent::Type)? };

        Ok(match ty {
            EventType::None => return Err(InvalidRawEvent::NoneType),
            EventType::ActiveEvent => {
                let cur = unsafe { &event.active };
                Event::Active(ActiveEvent {
                    gain: cur.gain != 0,
                    state: cur.state,
                })
            }
            ty @ (EventType::KeyDown | EventType::KeyUp) => {
                let ty = match ty {
                    EventType::KeyDown => KeyboardEventType::KeyDown,
                    EventType::KeyUp => KeyboardEventType::KeyUp,
                    _ => unreachable!(),
                };

                let cur = unsafe { &event.key };
                let state =
                    ButtonState::from_raw(cur.state).map_err(|_| InvalidRawEvent::ButtonState)?;
                let keysym = KeySym::from_raw(cur.keysym).map_err(|_| InvalidRawEvent::KeySym)?;
                Event::Keyboard(KeyboardEvent {
                    ty,
                    which: cur.which,
                    state,
                    keysym,
                })
            }
            EventType::MouseMotion => {
                let &SDL_MouseMotionEvent {
                    which,
                    state,
                    x,
                    y,
                    xrel,
                    yrel,
                    ..
                } = unsafe { &event.motion };

                Event::MouseMotion(MouseMotionEvent {
                    which,
                    state,
                    x,
                    y,
                    xrel,
                    yrel,
                })
            }
            ty @ (EventType::MouseButtonDown | EventType::MouseButtonUp) => {
                let ty = match ty {
                    EventType::MouseButtonDown => MouseButtonEventType::Down,
                    EventType::MouseButtonUp => MouseButtonEventType::Up,
                    _ => unreachable!(),
                };

                let &SDL_MouseButtonEvent {
                    which,
                    button,
                    state,
                    x,
                    y,
                    ..
                } = unsafe { &event.button };

                let state =
                    ButtonState::from_raw(state).map_err(|_| InvalidRawEvent::ButtonState)?;

                Event::MouseButton(MouseButtonEvent {
                    ty,
                    which,
                    button,
                    state,
                    x,
                    y,
                })
            }
            EventType::JoyAxisMotion => {
                let &SDL_JoyAxisEvent {
                    which, axis, value, ..
                } = unsafe { &event.jaxis };

                Event::JoyAxis(JoyAxisEvent { which, axis, value })
            }
            EventType::JoyBallMotion => {
                let &SDL_JoyBallEvent {
                    which,
                    ball,
                    xrel,
                    yrel,
                    ..
                } = unsafe { &event.jball };

                Event::JoyBall(JoyBallEvent {
                    which,
                    ball,
                    xrel,
                    yrel,
                })
            }
            EventType::JoyHatMotion => {
                let &SDL_JoyHatEvent {
                    which, hat, value, ..
                } = unsafe { &event.jhat };

                let value =
                    JoyHatPosition::from_raw(value).map_err(|_| InvalidRawEvent::JoyHatPosition)?;
                Event::JoyHat(JoyHatEvent { which, hat, value })
            }
            ty @ (EventType::JoyButtonDown | EventType::JoyButtonUp) => {
                let ty = match ty {
                    EventType::JoyButtonDown => JoyButtonEventType::Down,
                    EventType::JoyButtonUp => JoyButtonEventType::Up,
                    _ => unreachable!(),
                };

                let &SDL_JoyButtonEvent {
                    which,
                    button,
                    state,
                    ..
                } = unsafe { &event.jbutton };

                let state =
                    ButtonState::from_raw(state).map_err(|_| InvalidRawEvent::ButtonState)?;

                Event::JoyButton(JoyButtonEvent {
                    ty,
                    which,
                    button,
                    state,
                })
            }
            EventType::Quit => Event::Quit,
            EventType::SysWmEvent => {
                let cur = unsafe { &event.syswm };

                Event::SymWindowManager(SysWindowManagerEvent {
                    msg: NonNull::new(cur.msg).map(|ptr| {
                        system_window_manager::Message::try_from(unsafe { ptr.as_ref() }).unwrap()
                    }),
                })
            }
            EventType::VideoResize => {
                let &SDL_ResizeEvent { w, h, .. } = unsafe { &event.resize };

                Event::Resize(ResizeEvent { w, h })
            }
            EventType::VideoExpose => Event::Exposure,
            ty @ (EventType::UserEvent1
            | EventType::UserEvent2
            | EventType::UserEvent3
            | EventType::UserEvent4
            | EventType::UserEvent5
            | EventType::UserEvent6
            | EventType::UserEvent7
            | EventType::UserEvent8) => {
                let &SDL_UserEvent {
                    code, data1, data2, ..
                } = unsafe { &event.user };

                Event::User(UserEvent {
                    ty: ty as u8,
                    code,
                    data1,
                    data2,
                })
            }
        })
    }

    pub fn from_raw(event: SDL_Event) -> Self {
        Self::try_from_raw(event).unwrap()
    }
}

/// Application visibility event structure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActiveEvent {
    /// Whether given states were gained or lost
    pub gain: bool,
    /// A mask of the focus states
    pub state: u8,
}

/// Keyboard event structure
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyboardEvent {
    /// The event type
    pub ty: KeyboardEventType,
    /// The keyboard device index
    pub which: u8,
    /// The state of the button
    pub state: ButtonState,
    pub keysym: KeySym,
}

/// Mouse motion event structure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyboardEventType {
    KeyDown,
    KeyUp,
}

/// Mouse button event structure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyboardEventState {
    Pressed,
    Released,
}

/// Joystick axis motion event structure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MouseMotionEvent {
    /// The mouse device index
    pub which: u8,
    /// The current button state
    pub state: u8,
    /// The X coordinates of the mouse
    pub x: u16,
    /// The Y coordinates of the mouse
    pub y: u16,
    /// The relative motion in the X direction
    pub xrel: i16,
    /// The relative motion in the Y direction
    pub yrel: i16,
}

/// Joystick trackball motion event structure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MouseButtonEvent {
    /// The event type
    pub ty: MouseButtonEventType,
    /// The mouse device index
    pub which: u8,
    /// The mouse button index
    pub button: u8,
    /// The state of the button
    pub state: ButtonState,
    /// The X coordinates of the mouse at press time
    pub x: u16,
    /// The Y coordinates of the mouse at press time
    pub y: u16,
}

/// Joystick hat position change event structure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButtonEventType {
    Up = SDL_EventType_SDL_MOUSEBUTTONUP as isize,
    Down = SDL_EventType_SDL_MOUSEBUTTONDOWN as isize,
}

/// Joystick button event structure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JoyAxisEvent {
    /// The joystick device index
    pub which: u8,
    /// The joystick axis index
    pub axis: u8,
    /// The axis value (range: -32768 to 32767)
    pub value: i16,
}

/// The "window resized" event
/// When you get this event, you are responsible for setting a new video
/// mode with the new width and height.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JoyBallEvent {
    /// The joystick device index
    pub which: u8,
    /// The joystick trackball index
    pub ball: u8,
    /// The relative motion in the X direction
    pub xrel: i16,
    /// The relative motion in the Y direction
    pub yrel: i16,
}

/// Joystick hat position change event structure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JoyHatEvent {
    /// The joystick device index
    pub which: u8,
    /// The joystick hat index
    pub hat: u8,
    /// The hat position val
    pub value: JoyHatPosition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JoyHatPosition {
    Centered = 0x0,
    Up = 0x01,
    Right = 0x02,
    Down = 0x04,
    Left = 0x08,
    RightUp = Self::Right as isize | Self::Up as isize,
    RightDown = Self::Right as isize | Self::Down as isize,
    LeftUp = Self::Left as isize | Self::Up as isize,
    LeftDown = Self::Left as isize | Self::Down as isize,
}

impl Default for JoyHatPosition {
    fn default() -> Self {
        Self::Centered
    }
}

impl JoyHatPosition {
    pub fn from_raw(position: u8) -> Result<Self, InvalidJoyHatPosition> {
        Ok(match position {
            0x00 => Self::Centered,
            0x01 => Self::Up,
            0x02 => Self::Right,
            0x03 => Self::RightUp,
            0x04 => Self::Down,
            0x06 => Self::RightDown,
            0x08 => Self::Left,
            0x09 => Self::LeftUp,
            0x0c => Self::LeftDown,
            _ => return Err(InvalidJoyHatPosition),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("invalid joystick hat state")]
pub struct InvalidJoyHatPosition;

/// Joystick button event structure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JoyButtonEvent {
    /// The event type
    pub ty: JoyButtonEventType,
    /// The joystick device index
    pub which: u8,
    /// The joystick button index
    pub button: u8,
    /// The state of the button
    pub state: ButtonState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JoyButtonEventType {
    Up = SDL_EventType_SDL_JOYBUTTONUP as isize,
    Down = SDL_EventType_SDL_JOYBUTTONDOWN as isize,
}

/// The "window resized" event
/// When you get this event, you are responsible for setting a new video
/// mode with the new width and height.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResizeEvent {
    /// New width
    pub w: c_int,
    /// New height
    pub h: c_int,
}

/// A user-defined event type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserEvent {
    /// Event type. Can be any value between [`EventType::UserEvent`] and
    /// `[EventType::MAX_EVENTS] - 1`
    pub ty: u8,
    /// User defined event code
    pub code: c_int,
    /// User defined data pointer
    pub data1: *mut c_void,
    /// User defined data pointer
    pub data2: *mut c_void,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SysWindowManagerEvent {
    pub msg: Option<system_window_manager::Message>,
}