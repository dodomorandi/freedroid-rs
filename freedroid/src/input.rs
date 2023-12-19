#[cfg(target_os = "android")]
use crate::graphics::Graphics;
use crate::{
    defs::{Cmds, MenuAction, PointerStates},
    structs::Point,
    vars::Vars,
    Sdl,
};

#[cfg(not(target_os = "android"))]
use cstr::cstr;
use log::info;
use sdl::{
    convert::{i32_to_u8, u32_to_i32},
    event::KeyboardEventType,
    Event, Joystick,
};
#[cfg(feature = "gcw0")]
use sdl_sys::{SDLKey_SDLK_BACKSPACE, SDLKey_SDLK_LALT, SDLKey_SDLK_LCTRL, SDLKey_SDLK_TAB};
use sdl_sys::{
    SDLKey_SDLK_DOWN, SDLKey_SDLK_ESCAPE, SDLKey_SDLK_LEFT, SDLKey_SDLK_RETURN, SDLKey_SDLK_RIGHT,
    SDLKey_SDLK_SPACE, SDLKey_SDLK_UP, SDLMod, SDL_BUTTON_LEFT, SDL_BUTTON_MIDDLE,
    SDL_BUTTON_RIGHT, SDL_BUTTON_WHEELDOWN, SDL_BUTTON_WHEELUP,
};
#[cfg(not(feature = "gcw0"))]
use sdl_sys::{SDLKey_SDLK_F12, SDLKey_SDLK_PAUSE, SDLKey_SDLK_RSHIFT};
#[cfg(not(target_os = "android"))]
use std::ffi::CStr;
use std::{cell::Cell, fmt, os::raw::c_int};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct InputState {
    pub pressed: bool,
    pub fresh: bool,
}

impl InputState {
    pub fn is_just_pressed(self) -> bool {
        self.pressed && self.fresh
    }

    pub fn set_just_pressed(&mut self) {
        self.pressed = true;
        self.fresh = true;
    }

    pub fn set_just_released(&mut self) {
        self.pressed = false;
        self.fresh = true;
    }

    pub fn set_released(&mut self) {
        self.pressed = false;
        self.fresh = false;
    }
}

pub struct Input {
    pub show_cursor: bool,
    wheel_up_events: c_int,
    wheel_down_events: c_int,
    pub last_mouse_event: u32,
    current_modifiers: SDLMod,
    input_state: [InputState; PointerStates::Last as usize],
    event: Option<Event>,
    pub joy: Option<Joystick>,
    pub joy_sensitivity: c_int,
    // joystick (and mouse) axis values
    pub input_axis: Point,
    // number of joystick axes
    pub joy_num_axes: u16,
    // is firing to use axis-values or not
    pub axis_is_active: i32,
    pub key_cmds: [[c_int; 3]; Cmds::Last as usize],
}

#[cfg(not(target_os = "android"))]
pub const KEY_STRINGS: [Option<&'static CStr>; PointerStates::Last as usize] = create_key_strings();

#[cfg(not(target_os = "android"))]
const fn create_key_strings() -> [Option<&'static CStr>; PointerStates::Last as usize] {
    let mut out = [None; PointerStates::Last as usize];

    macro_rules! set_out {
        (ps::$key:ident = $str:literal) => {
            out[PointerStates::$key as usize] = Some(cstr!($str));
        };

        ($key:ident = $str:literal) => {
            out[sdl_sys::$key as usize] = Some(cstr!($str));
        };
    }

    out[0] = Some(cstr!("None"));
    if cfg!(feature = "gcw0") {
        set_out!(SDLKey_SDLK_BACKSPACE = "RSldr");
        set_out!(SDLKey_SDLK_TAB = "LSldr");
        set_out!(SDLKey_SDLK_RETURN = "Start");
        set_out!(SDLKey_SDLK_SPACE = "Y");
        set_out!(SDLKey_SDLK_ESCAPE = "Select");
    }

    if cfg!(not(feature = "gcw0")) {
        set_out!(SDLKey_SDLK_BACKSPACE = "BS");
        set_out!(SDLKey_SDLK_TAB = "Tab");
        set_out!(SDLKey_SDLK_RETURN = "Return");
        set_out!(SDLKey_SDLK_SPACE = "Space");
        set_out!(SDLKey_SDLK_ESCAPE = "Esc");
    }

    set_out!(SDLKey_SDLK_CLEAR = "Clear");
    set_out!(SDLKey_SDLK_PAUSE = "Pause");
    set_out!(SDLKey_SDLK_EXCLAIM = "!");
    set_out!(SDLKey_SDLK_QUOTEDBL = "\"");
    set_out!(SDLKey_SDLK_HASH = "#");
    set_out!(SDLKey_SDLK_DOLLAR = "$");
    set_out!(SDLKey_SDLK_AMPERSAND = "&");
    set_out!(SDLKey_SDLK_QUOTE = "'");
    set_out!(SDLKey_SDLK_LEFTPAREN = "(");
    set_out!(SDLKey_SDLK_RIGHTPAREN = ")");
    set_out!(SDLKey_SDLK_ASTERISK = "*");
    set_out!(SDLKey_SDLK_PLUS = "+");
    set_out!(SDLKey_SDLK_COMMA = ",");
    set_out!(SDLKey_SDLK_MINUS = "-");
    set_out!(SDLKey_SDLK_PERIOD = ".");
    set_out!(SDLKey_SDLK_SLASH = "/");
    set_out!(SDLKey_SDLK_0 = "0");
    set_out!(SDLKey_SDLK_1 = "1");
    set_out!(SDLKey_SDLK_2 = "2");
    set_out!(SDLKey_SDLK_3 = "3");
    set_out!(SDLKey_SDLK_4 = "4");
    set_out!(SDLKey_SDLK_5 = "5");
    set_out!(SDLKey_SDLK_6 = "6");
    set_out!(SDLKey_SDLK_7 = "7");
    set_out!(SDLKey_SDLK_8 = "8");
    set_out!(SDLKey_SDLK_9 = "9");
    set_out!(SDLKey_SDLK_COLON = ":");
    set_out!(SDLKey_SDLK_SEMICOLON = ";");
    set_out!(SDLKey_SDLK_LESS = "<");
    set_out!(SDLKey_SDLK_EQUALS = "=");
    set_out!(SDLKey_SDLK_GREATER = ">");
    set_out!(SDLKey_SDLK_QUESTION = "?");
    set_out!(SDLKey_SDLK_AT = "@");
    set_out!(SDLKey_SDLK_LEFTBRACKET = "[");
    set_out!(SDLKey_SDLK_BACKSLASH = "\\");
    set_out!(SDLKey_SDLK_RIGHTBRACKET = "]");
    set_out!(SDLKey_SDLK_CARET = "^");
    set_out!(SDLKey_SDLK_UNDERSCORE = "_");
    set_out!(SDLKey_SDLK_BACKQUOTE = "`");
    set_out!(SDLKey_SDLK_a = "a");
    set_out!(SDLKey_SDLK_b = "b");
    set_out!(SDLKey_SDLK_c = "c");
    set_out!(SDLKey_SDLK_d = "d");
    set_out!(SDLKey_SDLK_e = "e");
    set_out!(SDLKey_SDLK_f = "f");
    set_out!(SDLKey_SDLK_g = "g");
    set_out!(SDLKey_SDLK_h = "h");
    set_out!(SDLKey_SDLK_i = "i");
    set_out!(SDLKey_SDLK_j = "j");
    set_out!(SDLKey_SDLK_k = "k");
    set_out!(SDLKey_SDLK_l = "l");
    set_out!(SDLKey_SDLK_m = "m");
    set_out!(SDLKey_SDLK_n = "n");
    set_out!(SDLKey_SDLK_o = "o");
    set_out!(SDLKey_SDLK_p = "p");
    set_out!(SDLKey_SDLK_q = "q");
    set_out!(SDLKey_SDLK_r = "r");
    set_out!(SDLKey_SDLK_s = "s");
    set_out!(SDLKey_SDLK_t = "t");
    set_out!(SDLKey_SDLK_u = "u");
    set_out!(SDLKey_SDLK_v = "v");
    set_out!(SDLKey_SDLK_w = "w");
    set_out!(SDLKey_SDLK_x = "x");
    set_out!(SDLKey_SDLK_y = "y");
    set_out!(SDLKey_SDLK_z = "z");
    set_out!(SDLKey_SDLK_DELETE = "Del");

    /* Numeric keypad */
    set_out!(SDLKey_SDLK_KP0 = "Num[0 as usize]");
    set_out!(SDLKey_SDLK_KP1 = "Num[1 as usize]");
    set_out!(SDLKey_SDLK_KP2 = "Num[2 as usize]");
    set_out!(SDLKey_SDLK_KP3 = "Num[3 as usize]");
    set_out!(SDLKey_SDLK_KP4 = "Num[4 as usize]");
    set_out!(SDLKey_SDLK_KP5 = "Num[5 as usize]");
    set_out!(SDLKey_SDLK_KP6 = "Num[6 as usize]");
    set_out!(SDLKey_SDLK_KP7 = "Num[7 as usize]");
    set_out!(SDLKey_SDLK_KP8 = "Num[8 as usize]");
    set_out!(SDLKey_SDLK_KP9 = "Num[9 as usize]");
    set_out!(SDLKey_SDLK_KP_PERIOD = "Num[. as usize]");
    set_out!(SDLKey_SDLK_KP_DIVIDE = "Num[/ as usize]");
    set_out!(SDLKey_SDLK_KP_MULTIPLY = "Num[* as usize]");
    set_out!(SDLKey_SDLK_KP_MINUS = "Num[- as usize]");
    set_out!(SDLKey_SDLK_KP_PLUS = "Num[+ as usize]");
    set_out!(SDLKey_SDLK_KP_ENTER = "Num[Enter as usize]");
    set_out!(SDLKey_SDLK_KP_EQUALS = "Num[= as usize]");

    /* Arrows + Home/End pad */
    set_out!(SDLKey_SDLK_UP = "Up");
    set_out!(SDLKey_SDLK_DOWN = "Down");
    set_out!(SDLKey_SDLK_RIGHT = "Right");
    set_out!(SDLKey_SDLK_LEFT = "Left");
    set_out!(SDLKey_SDLK_INSERT = "Insert");
    set_out!(SDLKey_SDLK_HOME = "Home");
    set_out!(SDLKey_SDLK_END = "End");
    set_out!(SDLKey_SDLK_PAGEUP = "PageUp");
    set_out!(SDLKey_SDLK_PAGEDOWN = "PageDown");

    /* Function keys */
    set_out!(SDLKey_SDLK_F1 = "F1");
    set_out!(SDLKey_SDLK_F2 = "F2");
    set_out!(SDLKey_SDLK_F3 = "F3");
    set_out!(SDLKey_SDLK_F4 = "F4");
    set_out!(SDLKey_SDLK_F5 = "F5");
    set_out!(SDLKey_SDLK_F6 = "F6");
    set_out!(SDLKey_SDLK_F7 = "F7");
    set_out!(SDLKey_SDLK_F8 = "F8");
    set_out!(SDLKey_SDLK_F9 = "F9");
    set_out!(SDLKey_SDLK_F10 = "F10");
    set_out!(SDLKey_SDLK_F11 = "F11");
    set_out!(SDLKey_SDLK_F12 = "F12");
    set_out!(SDLKey_SDLK_F13 = "F13");
    set_out!(SDLKey_SDLK_F14 = "F14");
    set_out!(SDLKey_SDLK_F15 = "F15");

    /* Key state modifier keys */
    set_out!(SDLKey_SDLK_NUMLOCK = "NumLock");
    set_out!(SDLKey_SDLK_CAPSLOCK = "CapsLock");
    set_out!(SDLKey_SDLK_SCROLLOCK = "ScrlLock");
    if cfg!(feature = "gcw0") {
        set_out!(SDLKey_SDLK_LSHIFT = "X");
        set_out!(SDLKey_SDLK_LCTRL = "A");
        set_out!(SDLKey_SDLK_LALT = "B");
    }

    if cfg!(not(feature = "gcw0")) {
        set_out!(SDLKey_SDLK_LSHIFT = "LShift");
        set_out!(SDLKey_SDLK_LCTRL = "LCtrl");
        set_out!(SDLKey_SDLK_LALT = "LAlt");
    }

    set_out!(SDLKey_SDLK_RSHIFT = "RShift");
    set_out!(SDLKey_SDLK_RCTRL = "RCtrl");
    set_out!(SDLKey_SDLK_RALT = "RAlt");
    set_out!(SDLKey_SDLK_RMETA = "RMeta");
    set_out!(SDLKey_SDLK_LMETA = "LMeta");
    set_out!(SDLKey_SDLK_LSUPER = "LSuper");
    set_out!(SDLKey_SDLK_RSUPER = "RSuper");
    set_out!(SDLKey_SDLK_MODE = "Mode");
    set_out!(SDLKey_SDLK_COMPOSE = "Compose");

    /* Miscellaneous function keys */
    set_out!(SDLKey_SDLK_HELP = "Help");
    set_out!(SDLKey_SDLK_PRINT = "Print");
    set_out!(SDLKey_SDLK_SYSREQ = "SysReq");
    set_out!(SDLKey_SDLK_BREAK = "Break");
    set_out!(SDLKey_SDLK_MENU = "Menu");
    set_out!(SDLKey_SDLK_POWER = "Power");
    set_out!(SDLKey_SDLK_EURO = "Euro");
    set_out!(SDLKey_SDLK_UNDO = "Undo");

    /* Mouse und Joy buttons */
    set_out!(ps::MouseButton1 = "Mouse1");
    set_out!(ps::MouseButton2 = "Mouse2");
    set_out!(ps::MouseButton3 = "Mouse3");
    set_out!(ps::MouseWheelup = "WheelUp");
    set_out!(ps::MouseWheeldown = "WheelDown");

    set_out!(ps::JoyUp = "JoyUp");
    set_out!(ps::JoyDown = "JoyDown");
    set_out!(ps::JoyLeft = "JoyLeft");
    set_out!(ps::JoyRight = "JoyRight");
    set_out!(ps::JoyButton1 = "Joy-A");
    set_out!(ps::JoyButton2 = "Joy-B");
    set_out!(ps::JoyButton3 = "Joy-X");
    set_out!(ps::JoyButton4 = "Joy-Y");

    out
}

impl fmt::Debug for Input {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Input")
            .field("show_cursor", &self.show_cursor)
            .field("wheel_up_events", &self.wheel_up_events)
            .field("wheel_down_events", &self.wheel_down_events)
            .field("last_mouse_event", &self.last_mouse_event)
            .field("current_modifiers", &self.current_modifiers)
            .field("input_state", &self.input_state)
            .field("event", &"[SDL_Event]")
            .field("joy", &"[SDL Joystick]")
            .field("joy_sensitivity", &self.joy_sensitivity)
            .field("input_axis", &self.input_axis)
            .field("joy_num_axes", &self.joy_num_axes)
            .field("axis_is_active", &self.axis_is_active)
            .field("key_cmds", &self.key_cmds)
            .finish()
    }
}

impl Default for Input {
    fn default() -> Self {
        #[cfg(feature = "gcw0")]
        let key_cmds = [
            [SDLKey_SDLK_UP as c_int, PointerStates::JoyUp as c_int, 0], // CMD_UP
            [
                u32_to_i32(SDLKey_SDLK_DOWN),
                PointerStates::JoyDown as c_int,
                0,
            ], // CMD_DOWN
            [
                u32_to_i32(SDLKey_SDLK_LEFT),
                PointerStates::JoyLeft as c_int,
                0,
            ], // CMD_LEFT
            [
                u32_to_i32(SDLKey_SDLK_RIGHT),
                PointerStates::JoyRight as c_int,
                0,
            ], // CMD_RIGHT
            [
                u32_to_i32(SDLKey_SDLK_SPACE),
                u32_to_i32(SDLKey_SDLK_LCTRL),
                0,
            ], // CMD_FIRE
            [
                u32_to_i32(SDLKey_SDLK_LALT),
                PointerStates::JoyButton2 as c_int,
                0,
            ], // CMD_ACTIVATE
            [
                u32_to_i32(SDLKey_SDLK_BACKSPACE),
                u32_to_i32(SDLKey_SDLK_TAB),
                0,
            ], // CMD_TAKEOVER
            [0, 0, 0],                                                   // CMD_QUIT,
            [u32_to_i32(SDLKey_SDLK_RETURN), 0, 0],                      // CMD_PAUSE,
            [0, 0, 0],                                                   // CMD_SCREENSHOT
            [0, 0, 0],                                                   // CMD_FULLSCREEN,
            [
                u32_to_i32(SDLKey_SDLK_ESCAPE),
                PointerStates::JoyButton4 as c_int,
                0,
            ], // CMD_MENU,
            [
                u32_to_i32(SDLKey_SDLK_ESCAPE),
                PointerStates::JoyButton2 as c_int,
                PointerStates::MouseButton2 as c_int,
            ], // CMD_BACK
        ];

        #[cfg(not(feature = "gcw0"))]
        let key_cmds = [
            [
                u32_to_i32(SDLKey_SDLK_UP),
                PointerStates::JoyUp as c_int,
                i32::from(b'w'),
            ], // CMD_UP
            [
                u32_to_i32(SDLKey_SDLK_DOWN),
                PointerStates::JoyDown as c_int,
                i32::from(b's'),
            ], // CMD_DOWN
            [
                u32_to_i32(SDLKey_SDLK_LEFT),
                PointerStates::JoyLeft as c_int,
                i32::from(b'a'),
            ], // CMD_LEFT
            [
                u32_to_i32(SDLKey_SDLK_RIGHT),
                PointerStates::JoyRight as c_int,
                i32::from(b'd'),
            ], // CMD_RIGHT
            [
                u32_to_i32(SDLKey_SDLK_SPACE),
                PointerStates::JoyButton1 as c_int,
                PointerStates::MouseButton1 as c_int,
            ], // CMD_FIRE
            [
                u32_to_i32(SDLKey_SDLK_RETURN),
                u32_to_i32(SDLKey_SDLK_RSHIFT),
                i32::from(b'e'),
            ], // CMD_ACTIVATE
            [
                u32_to_i32(SDLKey_SDLK_SPACE),
                PointerStates::JoyButton2 as c_int,
                PointerStates::MouseButton2 as c_int,
            ], // CMD_TAKEOVER
            [i32::from(b'q'), 0, 0],                             // CMD_QUIT,
            [u32_to_i32(SDLKey_SDLK_PAUSE), i32::from(b'p'), 0], // CMD_PAUSE,
            [u32_to_i32(SDLKey_SDLK_F12), 0, 0],                 // CMD_SCREENSHOT
            [i32::from(b'f'), 0, 0],                             // CMD_FULLSCREEN,
            [
                u32_to_i32(SDLKey_SDLK_ESCAPE),
                PointerStates::JoyButton4 as c_int,
                0,
            ], // CMD_MENU,
            [
                u32_to_i32(SDLKey_SDLK_ESCAPE),
                PointerStates::JoyButton2 as c_int,
                PointerStates::MouseButton2 as c_int,
            ], // CMD_BACK
        ];

        Self {
            show_cursor: false,
            wheel_up_events: 0,
            wheel_down_events: 0,
            last_mouse_event: 0,
            current_modifiers: 0,
            input_state: [InputState::default(); PointerStates::Last as usize],
            event: None,
            joy: None,
            joy_sensitivity: 0,
            input_axis: Point { x: 0, y: 0 },
            joy_num_axes: 0,
            axis_is_active: 0,
            key_cmds,
        }
    }
}

pub const CMD_STRINGS: [&str; Cmds::Last as usize] = [
    "UP",
    "DOWN",
    "LEFT",
    "RIGHT",
    "FIRE",
    "ACTIVATE",
    "TAKEOVER",
    "QUIT",
    "PAUSE",
    "SCREENSHOT",
    "FULLSCREEN",
    "MENU",
    "BACK",
];

pub const CURSOR_KEEP_VISIBLE: u32 = 3000; // ticks to keep mouse-cursor visible without mouse-input

impl crate::Data<'_> {
    /// Check if any keys have been 'freshly' pressed. If yes, return key-code, otherwise 0.
    pub fn wait_for_key_pressed(&mut self) -> c_int {
        loop {
            match self.any_key_just_pressed() {
                0 => self.sdl.delay_ms(1),
                key => break key,
            }
        }
    }

    pub fn any_key_just_pressed(&mut self) -> c_int {
        let Self {
            sdl,
            input,
            vars,
            quit,
            #[cfg(target_os = "android")]
            graphics,
            ..
        } = self;

        Self::any_key_just_pressed_static(
            sdl,
            input,
            vars,
            quit,
            #[cfg(target_os = "android")]
            graphics,
        )
    }

    pub fn any_key_just_pressed_static(
        sdl: &Sdl,
        input: &mut Input,
        vars: &Vars,
        quit: &Cell<bool>,
        #[cfg(target_os = "android")] graphics: &mut Graphics<'_>,
    ) -> c_int {
        #[cfg(target_os = "android")]
        assert!(graphics.ne_screen.as_mut().unwrap().flip());

        Self::update_input_static(sdl, input, vars, quit);

        #[allow(clippy::cast_sign_loss)]
        let pressed_key = (0..PointerStates::Last as c_int)
            .find(|&key| input.input_state[key as usize].is_just_pressed());

        #[allow(clippy::cast_sign_loss)]
        match pressed_key {
            Some(key) => {
                input.input_state[key as usize].fresh = false;
                key
            }
            None => 0,
        }
    }

    pub fn update_input(&mut self) -> c_int {
        let Self {
            sdl,
            input,
            quit,
            vars,
            ..
        } = self;
        Self::update_input_static(sdl, input, vars, quit)
    }

    pub fn update_input_static(
        sdl: &Sdl,
        input: &mut Input,
        vars: &Vars,
        quit: &Cell<bool>,
    ) -> c_int {
        // switch mouse-cursor visibility as a function of time of last activity
        input.show_cursor = sdl.ticks_ms() - input.last_mouse_event <= CURSOR_KEEP_VISIBLE;

        loop {
            input.event = sdl.next_event();
            match &input.event {
                Some(event) => {
                    match event {
                        Event::Quit => {
                            info!("User requested termination, terminating.");
                            quit.set(true);
                            return 0;
                        }

                        Event::Keyboard(event) => {
                            input.current_modifiers = event.keysym.mod_.bits();
                            match event.ty {
                                KeyboardEventType::KeyDown => {
                                    input.input_state
                                        [usize::try_from(event.keysym.symbol as isize).unwrap()]
                                    .set_just_pressed();
                                    #[cfg(feature = "gcw0")]
                                    if input.input_axis.x != 0 || input.input_axis.y != 0 {
                                        input.axis_is_active = true.into(); // 4 GCW-0 ; breaks cursor keys after axis has been active...
                                    }
                                }
                                KeyboardEventType::KeyUp => {
                                    input.input_state
                                        [usize::try_from(event.keysym.symbol as isize).unwrap()]
                                    .set_just_released();
                                    #[cfg(feature = "gcw0")]
                                    {
                                        input.axis_is_active = false.into();
                                    }
                                }
                            }
                        }

                        Event::JoyAxis(event) => {
                            let axis = event.axis;
                            if axis == 0 || ((input.joy_num_axes >= 5) && (axis == 3))
                            /* x-axis */
                            {
                                input.input_axis.x = event.value.into();

                                // this is a bit tricky, because we want to allow direction keys
                                // to be soft-released. When mapping the joystick->keyboard, we
                                // therefore have to make sure that this mapping only occurs when
                                // and actual _change_ of the joystick-direction ('digital') occurs
                                // so that it behaves like "set"/"release"
                                if input.joy_sensitivity * i32::from(event.value) > 10000 {
                                    /* about half tilted */
                                    input.input_state[PointerStates::JoyRight as usize]
                                        .set_just_pressed();
                                    input.input_state[PointerStates::JoyLeft as usize]
                                        .set_released();
                                } else if input.joy_sensitivity * i32::from(event.value) < -10000 {
                                    input.input_state[PointerStates::JoyLeft as usize]
                                        .set_just_pressed();
                                    input.input_state[PointerStates::JoyRight as usize]
                                        .set_released();
                                } else {
                                    input.input_state[PointerStates::JoyLeft as usize]
                                        .set_released();
                                    input.input_state[PointerStates::JoyRight as usize]
                                        .set_released();
                                }
                            } else if (axis == 1) || ((input.joy_num_axes >= 5) && (axis == 4)) {
                                /* y-axis */
                                input.input_axis.y = event.value.into();

                                if input.joy_sensitivity * i32::from(event.value) > 10000 {
                                    input.input_state[PointerStates::JoyDown as usize]
                                        .set_just_pressed();
                                    input.input_state[PointerStates::JoyUp as usize].set_released();
                                } else if input.joy_sensitivity * i32::from(event.value) < -10000 {
                                    input.input_state[PointerStates::JoyUp as usize]
                                        .set_just_pressed();
                                    input.input_state[PointerStates::JoyDown as usize]
                                        .set_released();
                                } else {
                                    input.input_state[PointerStates::JoyUp as usize].set_released();
                                    input.input_state[PointerStates::JoyDown as usize]
                                        .set_released();
                                }
                            }
                        }

                        Event::JoyButton(event) => {
                            let is_pressed = event.state.is_pressed();
                            let input_state_index = match event.button {
                                0 => Some(PointerStates::JoyButton1 as usize),
                                1 => Some(PointerStates::JoyButton2 as usize),
                                2 => Some(PointerStates::JoyButton3 as usize),
                                3 => Some(PointerStates::JoyButton4 as usize),
                                _ => None,
                            };
                            if let Some(input_state_index) = input_state_index {
                                let input_state = &mut input.input_state[input_state_index];
                                input_state.pressed = is_pressed;
                                input_state.fresh = true;
                            }
                            input.axis_is_active = is_pressed.into();
                        }

                        Event::MouseMotion(event) => {
                            let user_center = vars.get_user_center();
                            input.input_axis.x =
                                i32::from(event.x) - i32::from(user_center.x()) + 16;
                            input.input_axis.y =
                                i32::from(event.y) - i32::from(user_center.y()) + 16;

                            input.last_mouse_event = sdl.ticks_ms();
                        }

                        Event::MouseButton(event) => {
                            const BUTTON_LEFT: u8 = i32_to_u8(SDL_BUTTON_LEFT);
                            const BUTTON_RIGHT: u8 = i32_to_u8(SDL_BUTTON_RIGHT);
                            const BUTTON_MIDDLE: u8 = i32_to_u8(SDL_BUTTON_MIDDLE);
                            const BUTTON_WHEELUP: u8 = i32_to_u8(SDL_BUTTON_WHEELUP);
                            const BUTTON_WHEELDOWN: u8 = i32_to_u8(SDL_BUTTON_WHEELDOWN);

                            let is_pressed = event.state.is_pressed();
                            let input_state_index = match event.button {
                                BUTTON_LEFT => {
                                    input.axis_is_active = is_pressed.into();
                                    Some(PointerStates::MouseButton1 as usize)
                                }
                                BUTTON_RIGHT => Some(PointerStates::MouseButton2 as usize),
                                BUTTON_MIDDLE => Some(PointerStates::MouseButton3 as usize),
                                // wheel events are immediately released, so we rather
                                // count the number of not yet read-out events
                                BUTTON_WHEELUP => {
                                    if is_pressed {
                                        input.wheel_up_events += 1;
                                    }
                                    None
                                }
                                BUTTON_WHEELDOWN => {
                                    if is_pressed {
                                        input.wheel_down_events += 1;
                                    }
                                    None
                                }
                                _ => None,
                            };

                            if let Some(input_state_index) = input_state_index {
                                let input_state = &mut input.input_state[input_state_index];
                                input_state.pressed = is_pressed;
                                input_state.fresh = true;
                            }
                            if is_pressed {
                                input.last_mouse_event = sdl.ticks_ms();
                            }
                        }

                        _ => break,
                    }
                }
                None => break,
            }
        }
        0
    }

    pub fn key_is_pressed(&mut self, key: c_int) -> bool {
        let Self {
            sdl,
            input,
            vars,
            quit,
            ..
        } = self;

        Self::key_is_pressed_static(sdl, input, vars, quit, key)
    }

    pub fn key_is_pressed_static(
        sdl: &Sdl,
        input: &mut Input,
        vars: &Vars,
        quit: &Cell<bool>,
        key: c_int,
    ) -> bool {
        Self::update_input_static(sdl, input, vars, quit);

        input.input_state[usize::try_from(key).unwrap()].is_just_pressed()
    }

    /// Does the same as `KeyIsPressed`, but automatically releases the key as well..
    pub fn key_is_pressed_r(&mut self, key: c_int) -> bool {
        let Self {
            sdl,
            input,
            vars,
            quit,
            ..
        } = self;

        Self::key_is_pressed_r_static(sdl, input, vars, quit, key)
    }

    pub fn key_is_pressed_r_static(
        sdl: &Sdl,
        input: &mut Input,
        vars: &Vars,
        quit: &Cell<bool>,
        key: c_int,
    ) -> bool {
        let ret = Self::key_is_pressed_static(sdl, input, vars, quit, key);

        input.release_key(key);
        ret
    }

    pub fn wheel_up_pressed(&mut self) -> bool {
        let Self {
            sdl,
            input,
            vars,
            quit,
            ..
        } = self;
        Self::wheel_up_pressed_static(sdl, input, vars, quit)
    }

    pub fn wheel_up_pressed_static(
        sdl: &Sdl,
        input: &mut Input,
        vars: &Vars,
        quit: &Cell<bool>,
    ) -> bool {
        Self::update_input_static(sdl, input, vars, quit);
        if input.wheel_up_events == 0 {
            false
        } else {
            input.wheel_up_events -= 1;
            true
        }
    }

    pub fn wheel_down_pressed(&mut self) -> bool {
        let Self {
            sdl,
            input,
            vars,
            quit,
            ..
        } = self;
        Self::wheel_down_pressed_static(sdl, input, vars, quit)
    }

    pub fn wheel_down_pressed_static(
        sdl: &Sdl,
        input: &mut Input,
        vars: &Vars,
        quit: &Cell<bool>,
    ) -> bool {
        Self::update_input_static(sdl, input, vars, quit);
        if input.wheel_down_events == 0 {
            false
        } else {
            input.wheel_down_events -= 1;
            true
        }
    }

    pub fn cmd_is_active(&mut self, cmd: Cmds) -> bool {
        let Self {
            sdl,
            input,
            vars,
            quit,
            ..
        } = self;

        Self::cmd_is_active_static(sdl, input, vars, quit, cmd)
    }

    pub fn cmd_is_active_static(
        sdl: &Sdl,
        input: &mut Input,
        vars: &Vars,
        quit: &Cell<bool>,
        cmd: Cmds,
    ) -> bool {
        let cmd = cmd as usize;
        Self::key_is_pressed_static(sdl, input, vars, quit, input.key_cmds[cmd][0])
            || Self::key_is_pressed_static(sdl, input, vars, quit, input.key_cmds[cmd][1])
            || Self::key_is_pressed_static(sdl, input, vars, quit, input.key_cmds[cmd][2])
    }

    /// the same but release the keys: use only for menus!
    pub fn cmd_is_active_r(&mut self, cmd: Cmds) -> bool {
        let Self {
            sdl,
            input,
            vars,
            quit,
            ..
        } = self;
        Self::cmd_is_active_r_static(sdl, input, vars, quit, cmd)
    }

    /// the same but release the keys: use only for menus!
    pub fn cmd_is_active_r_static(
        sdl: &Sdl,
        input: &mut Input,
        vars: &Vars,
        quit: &Cell<bool>,
        cmd: Cmds,
    ) -> bool {
        let cmd = cmd as usize;

        let c1 = Self::key_is_pressed_r_static(sdl, input, vars, quit, input.key_cmds[cmd][0]);
        let c2 = Self::key_is_pressed_r_static(sdl, input, vars, quit, input.key_cmds[cmd][1]);
        let c3 = Self::key_is_pressed_r_static(sdl, input, vars, quit, input.key_cmds[cmd][2]);

        c1 || c2 || c3
    }

    #[cfg(not(target_os = "android"))]
    pub fn wait_for_all_keys_released(&mut self) {
        let Self {
            input,
            sdl,
            vars,
            quit,
            ..
        } = self;

        Self::wait_for_all_keys_released_static(input, sdl, vars, quit);
    }

    #[cfg(target_os = "android")]
    pub fn wait_for_all_keys_released(&mut self) {
        let Self {
            input,
            sdl,
            graphics,
            ..
        } = self;

        Self::wait_for_all_keys_released_static(input, sdl, graphics)
    }

    #[cfg(not(target_os = "android"))]
    pub fn wait_for_all_keys_released_static(
        input: &mut Input,
        sdl: &Sdl,
        vars: &Vars,
        quit: &Cell<bool>,
        #[cfg(target_os = "android")] graphics: &mut Graphics,
    ) {
        while Self::any_key_is_pressed_r_static(
            input,
            #[cfg(not(target_os = "android"))]
            sdl,
            #[cfg(not(target_os = "android"))]
            vars,
            #[cfg(not(target_os = "android"))]
            quit,
            #[cfg(target_os = "android")]
            graphics,
        ) {
            sdl.delay_ms(1);
        }
        input.reset_mouse_wheel();
    }

    #[cfg(target_os = "android")]
    pub fn wait_for_all_keys_released_static(
        input: &mut Input,
        sdl: &Sdl,
        graphics: &mut Graphics,
    ) {
        while Self::any_key_is_pressed_r_static(input, graphics) {
            sdl.delay_ms(1);
        }
        input.reset_mouse_wheel();
    }

    #[cfg(not(target_os = "android"))]
    pub fn any_key_is_pressed_r(&mut self) -> bool {
        let Self {
            input,
            vars,
            quit,
            sdl,
            ..
        } = self;

        Self::any_key_is_pressed_r_static(input, sdl, vars, quit)
    }

    #[cfg(target_os = "android")]
    pub fn any_key_is_pressed_r(&mut self) -> bool {
        Self::any_key_is_pressed_r_static(&mut self.input, &mut self.graphics)
    }

    pub fn any_key_is_pressed_r_static(
        input: &mut Input,
        #[cfg(not(target_os = "android"))] sdl: &Sdl,
        #[cfg(not(target_os = "android"))] vars: &Vars,
        #[cfg(not(target_os = "android"))] quit: &Cell<bool>,
        #[cfg(target_os = "android")] graphics: &mut Graphics,
    ) -> bool {
        #[cfg(target_os = "android")]
        assert!(graphics.ne_screen.as_mut().unwrap().flip());

        #[cfg(not(target_os = "android"))]
        Self::update_input_static(sdl, input, vars, quit);

        for state in &mut input.input_state {
            if state.pressed || state.fresh {
                state.set_released();
                return true;
            }
        }
        false
    }

    pub fn mod_is_pressed(&mut self, sdl_mod: SDLMod) -> bool {
        self.update_input();
        (self.input.current_modifiers & sdl_mod) != 0
    }

    pub fn no_direction_pressed(&mut self) -> bool {
        !((self.input.axis_is_active != 0
            && (self.input.input_axis.x != 0 || self.input.input_axis.y != 0))
            || self.down_pressed()
            || self.up_pressed()
            || self.left_pressed()
            || self.right_pressed())
    }

    pub fn react_to_special_keys(&mut self) {
        if self.cmd_is_active_r(Cmds::Quit) {
            self.handle_quit_game(MenuAction::CLICK);
        }

        if self.cmd_is_active_r(Cmds::Pause) {
            self.pause();
        }

        if self.cmd_is_active(Cmds::Screenshot) {
            self.take_screenshot();
        }

        if self.cmd_is_active_r(Cmds::Fullscreen) {
            self.toggle_fullscreen();
        }

        if self.cmd_is_active_r(Cmds::Menu) {
            self.show_main_menu();
        }

        // this stuff remains hardcoded to keys
        if self.key_is_pressed_r(b'c'.into())
            && self.alt_pressed()
            && self.ctrl_pressed()
            && self.shift_pressed()
        {
            self.cheatmenu();
        }
    }

    pub fn init_joy(&mut self) {
        let joystick = self.sdl.init_joystick().unwrap_or_else(|| {
            panic!(
                "Couldn't initialize SDL-Joystick: {}",
                self.sdl.get_error().to_string_lossy()
            )
        });
        info!("SDL Joystick initialisation successful.");

        let num_joy = joystick.num_joysticks().unwrap_or(0);
        info!("{} Joysticks found!\n", num_joy);

        if let Some(joy) = (num_joy > 0).then(|| joystick.open(0)).flatten() {
            let joystick_name = joy
                .name()
                .map(|joystick_name| joystick_name.to_string_lossy());
            info!(
                "Identifier: {}",
                joystick_name.as_deref().unwrap_or("[NO JOYSTICK NAME]")
            );

            self.input.joy_num_axes = joy.axes();
            info!("Number of Axes: {}", self.input.joy_num_axes);
            info!("Number of Buttons: {}", joy.buttons());

            /* aktivate Joystick event handling */
            joystick.enable_event_polling();
        }
    }
}

impl Input {
    // forget the wheel-counters
    pub fn reset_mouse_wheel(&mut self) {
        self.wheel_up_events = 0;
        self.wheel_down_events = 0;
    }

    pub fn release_key(&mut self, key: c_int) {
        self.input_state[usize::try_from(key).unwrap()].set_released();
    }
}
