use std::os::raw::c_int;

pub const HS_EMPTY_ENTRY: &[u8] = b"--- empty ---";
pub const MAX_NAME_LEN: usize = 15;
pub const MAX_HIGHSCORES: u8 = 10;
pub const DATE_LEN: usize = 10;

pub const OK: c_int = 0;
pub const ERR: c_int = -1;

pub const MAX_INFLU_POSITION_HISTORY: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PointerStates {
    MouseUp = sdl::event::Key::Last as isize + 1,
    MouseRight,
    MouseDown,
    MouseLeft,
    MouseButton1,
    MouseButton2,
    MouseButton3,
    MouseWheelup,
    MouseWheeldown,

    JoyUp,
    JoyRight,
    JoyDown,
    JoyLeft,
    JoyButton1,
    JoyButton2,
    JoyButton3,
    JoyButton4,

    Last,
}
