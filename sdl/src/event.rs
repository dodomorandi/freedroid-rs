use std::{ffi::c_void, os::raw::c_int, ptr::NonNull};

use sdl_sys::{
    SDL_ActiveEvent, SDL_Event, SDL_EventType_SDL_ACTIVEEVENT, SDL_EventType_SDL_JOYAXISMOTION,
    SDL_EventType_SDL_JOYBALLMOTION, SDL_EventType_SDL_JOYBUTTONDOWN,
    SDL_EventType_SDL_JOYBUTTONUP, SDL_EventType_SDL_JOYHATMOTION, SDL_EventType_SDL_KEYDOWN,
    SDL_EventType_SDL_KEYUP, SDL_EventType_SDL_MOUSEBUTTONDOWN, SDL_EventType_SDL_MOUSEBUTTONUP,
    SDL_EventType_SDL_MOUSEMOTION, SDL_EventType_SDL_NOEVENT, SDL_EventType_SDL_NUMEVENTS,
    SDL_EventType_SDL_QUIT, SDL_EventType_SDL_SYSWMEVENT, SDL_EventType_SDL_USEREVENT,
    SDL_EventType_SDL_VIDEOEXPOSE, SDL_EventType_SDL_VIDEORESIZE, SDL_ExposeEvent,
    SDL_JoyAxisEvent, SDL_JoyBallEvent, SDL_JoyButtonEvent, SDL_JoyHatEvent, SDL_KeyboardEvent,
    SDL_MouseButtonEvent, SDL_MouseMotionEvent, SDL_QuitEvent, SDL_ResizeEvent, SDL_SysWMEvent,
    SDL_SysWMmsg, SDL_UserEvent,
};

use crate::{convert, keyboard::KeySym, system_window_manager};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
#[non_exhaustive]
pub enum Type {
    /// Unused (do not remove)
    None = convert::u32_to_u8(SDL_EventType_SDL_NOEVENT),
    /// Application loses/gains visibility
    ActiveEvent = convert::u32_to_u8(SDL_EventType_SDL_ACTIVEEVENT),
    /// Keys pressed
    KeyDown = convert::u32_to_u8(SDL_EventType_SDL_KEYDOWN),
    /// Keys released
    KeyUp = convert::u32_to_u8(SDL_EventType_SDL_KEYUP),
    /// Mouse moved
    MouseMotion = convert::u32_to_u8(SDL_EventType_SDL_MOUSEMOTION),
    /// Mouse button pressed
    MouseButtonDown = convert::u32_to_u8(SDL_EventType_SDL_MOUSEBUTTONDOWN),
    /// Mouse button released
    MouseButtonUp = convert::u32_to_u8(SDL_EventType_SDL_MOUSEBUTTONUP),
    /// Joystick axis motion
    JoyAxisMotion = convert::u32_to_u8(SDL_EventType_SDL_JOYAXISMOTION),
    /// Joystick trackball motion
    JoyBallMotion = convert::u32_to_u8(SDL_EventType_SDL_JOYBALLMOTION),
    /// Joystick hat position change
    JoyHatMotion = convert::u32_to_u8(SDL_EventType_SDL_JOYHATMOTION),
    /// Joystick button pressed
    JoyButtonDown = convert::u32_to_u8(SDL_EventType_SDL_JOYBUTTONDOWN),
    /// Joystick button released
    JoyButtonUp = convert::u32_to_u8(SDL_EventType_SDL_JOYBUTTONUP),
    /// User-requested quit
    Quit = convert::u32_to_u8(SDL_EventType_SDL_QUIT),
    /// System specific event
    SysWmEvent = convert::u32_to_u8(SDL_EventType_SDL_SYSWMEVENT),
    /// User resized video mode
    VideoResize = convert::u32_to_u8(SDL_EventType_SDL_VIDEORESIZE),
    /// Screen needs to be redrawn
    VideoExpose = convert::u32_to_u8(SDL_EventType_SDL_VIDEOEXPOSE),
    /// Events SDL_USEREVENT through `MAX_EVENTS - 1` are for your use
    UserEvent1 = convert::u32_to_u8(SDL_EventType_SDL_USEREVENT),
    UserEvent2 = convert::u32_to_u8(SDL_EventType_SDL_USEREVENT + 1),
    UserEvent3 = convert::u32_to_u8(SDL_EventType_SDL_USEREVENT + 2),
    UserEvent4 = convert::u32_to_u8(SDL_EventType_SDL_USEREVENT + 3),
    UserEvent5 = convert::u32_to_u8(SDL_EventType_SDL_USEREVENT + 4),
    UserEvent6 = convert::u32_to_u8(SDL_EventType_SDL_USEREVENT + 5),
    UserEvent7 = convert::u32_to_u8(SDL_EventType_SDL_USEREVENT + 6),
    UserEvent8 = convert::u32_to_u8(SDL_EventType_SDL_USEREVENT + 7),
}

// Just a compile-time check
const _: [u8; Type::UserEvent8 as usize + 1] = [0u8; Type::MAX_EVENTS as usize];

impl Default for Type {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
#[error("invalid raw event type")]
pub struct InvalidRawType;

impl Type {
    pub const MAX_EVENTS: u8 = convert::u32_to_u8(SDL_EventType_SDL_NUMEVENTS);

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

    #[must_use]
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
    Active(Active),
    Keyboard(Keyboard),
    MouseMotion(MouseMotion),
    MouseButton(MouseButton),
    JoyAxis(JoyAxis),
    JoyBall(JoyBall),
    JoyHat(JoyHat),
    JoyButton(JoyButton),
    Resize(Resize),
    Exposure,
    Quit,
    User(User),
    SymWindowManager(SysWindowManager),
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum InvalidRaw {
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
    pub fn try_from_raw(event: SDL_Event) -> Result<Self, InvalidRaw> {
        // Safety: `type` is available for each member of the union, therefore is always available.
        let ty = unsafe { Type::from_raw(event.type_).map_err(|_| InvalidRaw::Type)? };

        Ok(match ty {
            Type::None => return Err(InvalidRaw::NoneType),
            Type::ActiveEvent => {
                let cur = unsafe { &event.active };
                Event::Active(Active {
                    gain: cur.gain != 0,
                    state: cur.state,
                })
            }
            ty @ (Type::KeyDown | Type::KeyUp) => {
                let ty = match ty {
                    Type::KeyDown => KeyboardEventType::KeyDown,
                    Type::KeyUp => KeyboardEventType::KeyUp,
                    _ => unreachable!(),
                };

                let cur = unsafe { &event.key };
                let state =
                    ButtonState::from_raw(cur.state).map_err(|_| InvalidRaw::ButtonState)?;
                let keysym = KeySym::from_raw(cur.keysym).map_err(|_| InvalidRaw::KeySym)?;
                Event::Keyboard(Keyboard {
                    ty,
                    which: cur.which,
                    state,
                    keysym,
                })
            }
            Type::MouseMotion => {
                let &SDL_MouseMotionEvent {
                    which,
                    state,
                    x,
                    y,
                    xrel,
                    yrel,
                    ..
                } = unsafe { &event.motion };

                Event::MouseMotion(MouseMotion {
                    which,
                    state,
                    x,
                    y,
                    xrel,
                    yrel,
                })
            }
            ty @ (Type::MouseButtonDown | Type::MouseButtonUp) => {
                let ty = match ty {
                    Type::MouseButtonDown => MouseButtonEventType::Down,
                    Type::MouseButtonUp => MouseButtonEventType::Up,
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

                let state = ButtonState::from_raw(state).map_err(|_| InvalidRaw::ButtonState)?;

                Event::MouseButton(MouseButton {
                    ty,
                    which,
                    button,
                    state,
                    x,
                    y,
                })
            }
            Type::JoyAxisMotion => {
                let &SDL_JoyAxisEvent {
                    which, axis, value, ..
                } = unsafe { &event.jaxis };

                Event::JoyAxis(JoyAxis { which, axis, value })
            }
            Type::JoyBallMotion => {
                let &SDL_JoyBallEvent {
                    which,
                    ball,
                    xrel,
                    yrel,
                    ..
                } = unsafe { &event.jball };

                Event::JoyBall(JoyBall {
                    which,
                    ball,
                    xrel,
                    yrel,
                })
            }
            Type::JoyHatMotion => {
                let &SDL_JoyHatEvent {
                    which, hat, value, ..
                } = unsafe { &event.jhat };

                let value =
                    JoyHatPosition::from_raw(value).map_err(|_| InvalidRaw::JoyHatPosition)?;
                Event::JoyHat(JoyHat { which, hat, value })
            }
            ty @ (Type::JoyButtonDown | Type::JoyButtonUp) => {
                let ty = match ty {
                    Type::JoyButtonDown => JoyButtonEventType::Down,
                    Type::JoyButtonUp => JoyButtonEventType::Up,
                    _ => unreachable!(),
                };

                let &SDL_JoyButtonEvent {
                    which,
                    button,
                    state,
                    ..
                } = unsafe { &event.jbutton };

                let state = ButtonState::from_raw(state).map_err(|_| InvalidRaw::ButtonState)?;

                Event::JoyButton(JoyButton {
                    ty,
                    which,
                    button,
                    state,
                })
            }
            Type::Quit => Event::Quit,
            Type::SysWmEvent => {
                let cur = unsafe { &event.syswm };

                Event::SymWindowManager(SysWindowManager {
                    msg: NonNull::new(cur.msg).map(|ptr| {
                        system_window_manager::Message::try_from(unsafe { ptr.as_ref() }).unwrap()
                    }),
                    pointer: cur.msg,
                })
            }
            Type::VideoResize => {
                let &SDL_ResizeEvent { w, h, .. } = unsafe { &event.resize };

                Event::Resize(Resize { w, h })
            }
            Type::VideoExpose => Event::Exposure,
            ty @ (Type::UserEvent1
            | Type::UserEvent2
            | Type::UserEvent3
            | Type::UserEvent4
            | Type::UserEvent5
            | Type::UserEvent6
            | Type::UserEvent7
            | Type::UserEvent8) => {
                let &SDL_UserEvent {
                    code, data1, data2, ..
                } = unsafe { &event.user };

                Event::User(User {
                    ty: ty as u8,
                    code,
                    data1,
                    data2,
                })
            }
        })
    }

    #[must_use]
    pub fn from_raw(event: SDL_Event) -> Self {
        Self::try_from_raw(event).unwrap()
    }

    #[must_use]
    pub fn to_raw(&self) -> SDL_Event {
        match self {
            &Event::Active(Active { gain, state }) => SDL_Event {
                active: SDL_ActiveEvent {
                    type_: Type::ActiveEvent as u8,
                    gain: gain.into(),
                    state,
                },
            },
            &Event::Keyboard(Keyboard {
                ty,
                which,
                state,
                ref keysym,
            }) => {
                let type_ = match ty {
                    KeyboardEventType::KeyDown => Type::KeyDown as u8,
                    KeyboardEventType::KeyUp => Type::KeyUp as u8,
                };

                SDL_Event {
                    key: SDL_KeyboardEvent {
                        type_,
                        which,
                        state: state as u8,
                        keysym: keysym.to_raw(),
                    },
                }
            }
            &Event::MouseMotion(MouseMotion {
                which,
                state,
                x,
                y,
                xrel,
                yrel,
            }) => SDL_Event {
                motion: SDL_MouseMotionEvent {
                    type_: Type::MouseMotion as u8,
                    which,
                    state,
                    x,
                    y,
                    xrel,
                    yrel,
                },
            },
            &Event::MouseButton(MouseButton {
                ty,
                which,
                button,
                state,
                x,
                y,
            }) => SDL_Event {
                button: SDL_MouseButtonEvent {
                    type_: ty as u8,
                    which,
                    button,
                    state: state as u8,
                    x,
                    y,
                },
            },
            &Event::JoyAxis(JoyAxis { which, axis, value }) => SDL_Event {
                jaxis: SDL_JoyAxisEvent {
                    type_: Type::JoyAxisMotion as u8,
                    which,
                    axis,
                    value,
                },
            },
            &Event::JoyBall(JoyBall {
                which,
                ball,
                xrel,
                yrel,
            }) => SDL_Event {
                jball: SDL_JoyBallEvent {
                    type_: Type::JoyBallMotion as u8,
                    which,
                    ball,
                    xrel,
                    yrel,
                },
            },
            &Event::JoyHat(JoyHat { which, hat, value }) => SDL_Event {
                jhat: SDL_JoyHatEvent {
                    type_: Type::JoyHatMotion as u8,
                    which,
                    hat,
                    value: value as u8,
                },
            },
            &Event::JoyButton(JoyButton {
                ty,
                which,
                button,
                state,
            }) => SDL_Event {
                jbutton: SDL_JoyButtonEvent {
                    type_: ty as u8,
                    which,
                    button,
                    state: state as u8,
                },
            },
            &Event::Resize(Resize { w, h }) => SDL_Event {
                resize: SDL_ResizeEvent {
                    type_: Type::VideoResize as u8,
                    w,
                    h,
                },
            },
            Event::Exposure => SDL_Event {
                expose: SDL_ExposeEvent {
                    type_: Type::VideoExpose as u8,
                },
            },
            Event::Quit => SDL_Event {
                quit: SDL_QuitEvent {
                    type_: Type::Quit as u8,
                },
            },
            &Event::User(User {
                ty,
                code,
                data1,
                data2,
            }) => SDL_Event {
                user: SDL_UserEvent {
                    type_: ty,
                    code,
                    data1,
                    data2,
                },
            },
            &Event::SymWindowManager(SysWindowManager { pointer, .. }) => SDL_Event {
                syswm: SDL_SysWMEvent {
                    type_: Type::SysWmEvent as u8,
                    msg: pointer,
                },
            },
        }
    }
}

impl TryFrom<SDL_Event> for Event {
    type Error = InvalidRaw;

    fn try_from(event: SDL_Event) -> Result<Self, Self::Error> {
        Event::try_from_raw(event)
    }
}

/// Application visibility event structure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Active {
    /// Whether given states were gained or lost
    pub gain: bool,
    /// A mask of the focus states
    pub state: u8,
}

/// Keyboard event structure
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Keyboard {
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
pub struct MouseMotion {
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
pub struct MouseButton {
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
    Up = convert::u32_to_isize(SDL_EventType_SDL_MOUSEBUTTONUP),
    Down = convert::u32_to_isize(SDL_EventType_SDL_MOUSEBUTTONDOWN),
}

/// Joystick button event structure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JoyAxis {
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
pub struct JoyBall {
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
pub struct JoyHat {
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
pub struct JoyButton {
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
    Up = convert::u32_to_isize(SDL_EventType_SDL_JOYBUTTONUP),
    Down = convert::u32_to_isize(SDL_EventType_SDL_JOYBUTTONDOWN),
}

/// The "window resized" event
/// When you get this event, you are responsible for setting a new video
/// mode with the new width and height.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Resize {
    /// New width
    pub w: c_int,
    /// New height
    pub h: c_int,
}

/// A user-defined event type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct User {
    /// Event type. Can be any value between [`EventType::UserEvent1`] and
    /// `[EventType::UserEvent8]`
    pub ty: u8,
    /// User defined event code
    pub code: c_int,
    /// User defined data pointer
    pub data1: *mut c_void,
    /// User defined data pointer
    pub data2: *mut c_void,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SysWindowManager {
    pub msg: Option<system_window_manager::Message>,
    pub pointer: *mut SDL_SysWMmsg,
}
