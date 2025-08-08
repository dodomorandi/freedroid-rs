#[cfg(target_os = "android")]
use crate::graphics::Graphics;
use crate::{
    Sdl,
    defs::{Cmds, MenuAction, PointerStates},
    structs::Point,
    vars::Vars,
};

use log::info;
use sdl::{
    Event, Joystick,
    convert::{i32_to_u8, u32_to_u16},
    event::{self, KeyboardEventType},
};
use sdl_sys::{
    SDL_BUTTON_LEFT, SDL_BUTTON_MIDDLE, SDL_BUTTON_RIGHT, SDL_BUTTON_WHEELDOWN, SDL_BUTTON_WHEELUP,
    SDLKey_SDLK_DOWN, SDLKey_SDLK_ESCAPE, SDLKey_SDLK_LEFT, SDLKey_SDLK_RETURN, SDLKey_SDLK_RIGHT,
    SDLKey_SDLK_SPACE, SDLKey_SDLK_UP, SDLMod,
};
#[cfg(feature = "gcw0")]
use sdl_sys::{SDLKey_SDLK_BACKSPACE, SDLKey_SDLK_LALT, SDLKey_SDLK_LCTRL, SDLKey_SDLK_TAB};
#[cfg(not(feature = "gcw0"))]
use sdl_sys::{SDLKey_SDLK_F12, SDLKey_SDLK_PAUSE, SDLKey_SDLK_RSHIFT};
#[cfg(not(target_os = "android"))]
use std::ffi::CStr;
use std::{cell::Cell, fmt};

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
    wheel_up_events: u32,
    wheel_down_events: u32,
    pub last_mouse_event: u32,
    current_modifiers: SDLMod,
    state: [InputState; PointerStates::Last.to_usize()],
    pub joy: Option<Joystick>,
    pub joy_sensitivity: u8,
    // joystick (and mouse) axis values
    pub axis: Point,
    // number of joystick axes
    pub joy_num_axes: u16,
    // is firing to use axis-values or not
    pub axis_is_active: bool,
    pub key_cmds: [[u16; 3]; Cmds::Last as usize],
}

#[cfg(not(target_os = "android"))]
pub const KEY_STRINGS: [Option<&'static CStr>; PointerStates::Last.to_usize()] =
    create_key_strings();

#[cfg(not(target_os = "android"))]
#[allow(clippy::too_many_lines)]
const fn create_key_strings() -> [Option<&'static CStr>; PointerStates::Last.to_usize()] {
    let mut out = [None; PointerStates::Last.to_usize()];

    macro_rules! set_out {
        ($(ps::$key:ident = $str:literal);+ $(;)?) => {
            $(
                out[PointerStates::$key.to_usize()] = Some($str);
            )+
        };

        ($($key:ident = $str:literal);+ $(;)?) => {
            $(
                out[::sdl::convert::u32_to_usize(sdl_sys::$key)] = Some($str);
            )+
        };
    }

    out[0] = Some(c"None");
    if cfg!(feature = "gcw0") {
        set_out!(
            SDLKey_SDLK_BACKSPACE = c"RSldr";
            SDLKey_SDLK_TAB = c"LSldr";
            SDLKey_SDLK_RETURN = c"Start";
            SDLKey_SDLK_SPACE = c"Y";
            SDLKey_SDLK_ESCAPE = c"Select";
        );
    }

    if cfg!(not(feature = "gcw0")) {
        set_out!(
            SDLKey_SDLK_BACKSPACE = c"BS";
            SDLKey_SDLK_TAB = c"Tab";
            SDLKey_SDLK_RETURN = c"Return";
            SDLKey_SDLK_SPACE = c"Space";
            SDLKey_SDLK_ESCAPE = c"Esc";
        );
    }

    set_out!(
        SDLKey_SDLK_CLEAR = c"Clear";
        SDLKey_SDLK_PAUSE = c"Pause";
        SDLKey_SDLK_EXCLAIM = c"!";
        SDLKey_SDLK_QUOTEDBL = c"\"";
        SDLKey_SDLK_HASH = c"#";
        SDLKey_SDLK_DOLLAR = c"$";
        SDLKey_SDLK_AMPERSAND = c"&";
        SDLKey_SDLK_QUOTE = c"'";
        SDLKey_SDLK_LEFTPAREN = c"(";
        SDLKey_SDLK_RIGHTPAREN = c")";
        SDLKey_SDLK_ASTERISK = c"*";
        SDLKey_SDLK_PLUS = c"+";
        SDLKey_SDLK_COMMA = c",";
        SDLKey_SDLK_MINUS = c"-";
        SDLKey_SDLK_PERIOD = c".";
        SDLKey_SDLK_SLASH = c"/";
        SDLKey_SDLK_0 = c"0";
        SDLKey_SDLK_1 = c"1";
        SDLKey_SDLK_2 = c"2";
        SDLKey_SDLK_3 = c"3";
        SDLKey_SDLK_4 = c"4";
        SDLKey_SDLK_5 = c"5";
        SDLKey_SDLK_6 = c"6";
        SDLKey_SDLK_7 = c"7";
        SDLKey_SDLK_8 = c"8";
        SDLKey_SDLK_9 = c"9";
        SDLKey_SDLK_COLON = c":";
        SDLKey_SDLK_SEMICOLON = c";";
        SDLKey_SDLK_LESS = c"<";
        SDLKey_SDLK_EQUALS = c"=";
        SDLKey_SDLK_GREATER = c">";
        SDLKey_SDLK_QUESTION = c"?";
        SDLKey_SDLK_AT = c"@";
        SDLKey_SDLK_LEFTBRACKET = c"[";
        SDLKey_SDLK_BACKSLASH = c"\\";
        SDLKey_SDLK_RIGHTBRACKET = c"]";
        SDLKey_SDLK_CARET = c"^";
        SDLKey_SDLK_UNDERSCORE = c"_";
        SDLKey_SDLK_BACKQUOTE = c"`";
        SDLKey_SDLK_a = c"a";
        SDLKey_SDLK_b = c"b";
        SDLKey_SDLK_c = c"c";
        SDLKey_SDLK_d = c"d";
        SDLKey_SDLK_e = c"e";
        SDLKey_SDLK_f = c"f";
        SDLKey_SDLK_g = c"g";
        SDLKey_SDLK_h = c"h";
        SDLKey_SDLK_i = c"i";
        SDLKey_SDLK_j = c"j";
        SDLKey_SDLK_k = c"k";
        SDLKey_SDLK_l = c"l";
        SDLKey_SDLK_m = c"m";
        SDLKey_SDLK_n = c"n";
        SDLKey_SDLK_o = c"o";
        SDLKey_SDLK_p = c"p";
        SDLKey_SDLK_q = c"q";
        SDLKey_SDLK_r = c"r";
        SDLKey_SDLK_s = c"s";
        SDLKey_SDLK_t = c"t";
        SDLKey_SDLK_u = c"u";
        SDLKey_SDLK_v = c"v";
        SDLKey_SDLK_w = c"w";
        SDLKey_SDLK_x = c"x";
        SDLKey_SDLK_y = c"y";
        SDLKey_SDLK_z = c"z";
        SDLKey_SDLK_DELETE = c"Del";
    );

    /* Numeric keypad */
    set_out!(
        SDLKey_SDLK_KP0 = c"Num[0 as usize]";
        SDLKey_SDLK_KP1 = c"Num[1 as usize]";
        SDLKey_SDLK_KP2 = c"Num[2 as usize]";
        SDLKey_SDLK_KP3 = c"Num[3 as usize]";
        SDLKey_SDLK_KP4 = c"Num[4 as usize]";
        SDLKey_SDLK_KP5 = c"Num[5 as usize]";
        SDLKey_SDLK_KP6 = c"Num[6 as usize]";
        SDLKey_SDLK_KP7 = c"Num[7 as usize]";
        SDLKey_SDLK_KP8 = c"Num[8 as usize]";
        SDLKey_SDLK_KP9 = c"Num[9 as usize]";
        SDLKey_SDLK_KP_PERIOD = c"Num[. as usize]";
        SDLKey_SDLK_KP_DIVIDE = c"Num[/ as usize]";
        SDLKey_SDLK_KP_MULTIPLY = c"Num[* as usize]";
        SDLKey_SDLK_KP_MINUS = c"Num[- as usize]";
        SDLKey_SDLK_KP_PLUS = c"Num[+ as usize]";
        SDLKey_SDLK_KP_ENTER = c"Num[Enter as usize]";
        SDLKey_SDLK_KP_EQUALS = c"Num[= as usize]";
    );

    /* Arrows + Home/End pad */
    set_out!(
        SDLKey_SDLK_UP = c"Up";
        SDLKey_SDLK_DOWN = c"Down";
        SDLKey_SDLK_RIGHT = c"Right";
        SDLKey_SDLK_LEFT = c"Left";
        SDLKey_SDLK_INSERT = c"Insert";
        SDLKey_SDLK_HOME = c"Home";
        SDLKey_SDLK_END = c"End";
        SDLKey_SDLK_PAGEUP = c"PageUp";
        SDLKey_SDLK_PAGEDOWN = c"PageDown";
    );

    /* Function keys */
    set_out!(
        SDLKey_SDLK_F1 = c"F1";
        SDLKey_SDLK_F2 = c"F2";
        SDLKey_SDLK_F3 = c"F3";
        SDLKey_SDLK_F4 = c"F4";
        SDLKey_SDLK_F5 = c"F5";
        SDLKey_SDLK_F6 = c"F6";
        SDLKey_SDLK_F7 = c"F7";
        SDLKey_SDLK_F8 = c"F8";
        SDLKey_SDLK_F9 = c"F9";
        SDLKey_SDLK_F10 = c"F10";
        SDLKey_SDLK_F11 = c"F11";
        SDLKey_SDLK_F12 = c"F12";
        SDLKey_SDLK_F13 = c"F13";
        SDLKey_SDLK_F14 = c"F14";
        SDLKey_SDLK_F15 = c"F15";
    );

    /* Key state modifier keys */
    set_out!(
        SDLKey_SDLK_NUMLOCK = c"NumLock";
        SDLKey_SDLK_CAPSLOCK = c"CapsLock";
        SDLKey_SDLK_SCROLLOCK = c"ScrlLock";
    );
    if cfg!(feature = "gcw0") {
        set_out!(
            SDLKey_SDLK_LSHIFT = c"X";
            SDLKey_SDLK_LCTRL = c"A";
            SDLKey_SDLK_LALT = c"B";
        );
    }

    if cfg!(not(feature = "gcw0")) {
        set_out!(
            SDLKey_SDLK_LSHIFT = c"LShift";
            SDLKey_SDLK_LCTRL = c"LCtrl";
            SDLKey_SDLK_LALT = c"LAlt";
        );
    }

    set_out!(
        SDLKey_SDLK_RSHIFT = c"RShift";
        SDLKey_SDLK_RCTRL = c"RCtrl";
        SDLKey_SDLK_RALT = c"RAlt";
        SDLKey_SDLK_RMETA = c"RMeta";
        SDLKey_SDLK_LMETA = c"LMeta";
        SDLKey_SDLK_LSUPER = c"LSuper";
        SDLKey_SDLK_RSUPER = c"RSuper";
        SDLKey_SDLK_MODE = c"Mode";
        SDLKey_SDLK_COMPOSE = c"Compose";
    );

    /* Miscellaneous function keys */
    set_out!(
        SDLKey_SDLK_HELP = c"Help";
        SDLKey_SDLK_PRINT = c"Print";
        SDLKey_SDLK_SYSREQ = c"SysReq";
        SDLKey_SDLK_BREAK = c"Break";
        SDLKey_SDLK_MENU = c"Menu";
        SDLKey_SDLK_POWER = c"Power";
        SDLKey_SDLK_EURO = c"Euro";
        SDLKey_SDLK_UNDO = c"Undo";
    );

    /* Mouse und Joy buttons */
    set_out!(
        ps::MouseButton1 = c"Mouse1";
        ps::MouseButton2 = c"Mouse2";
        ps::MouseButton3 = c"Mouse3";
        ps::MouseWheelup = c"WheelUp";
        ps::MouseWheeldown = c"WheelDown";
        ps::JoyUp = c"JoyUp";
        ps::JoyDown = c"JoyDown";
        ps::JoyLeft = c"JoyLeft";
        ps::JoyRight = c"JoyRight";
        ps::JoyButton1 = c"Joy-A";
        ps::JoyButton2 = c"Joy-B";
        ps::JoyButton3 = c"Joy-X";
        ps::JoyButton4 = c"Joy-Y";
    );

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
            .field("state", &self.state)
            .field("event", &"[SDL_Event]")
            .field("joy", &"[SDL Joystick]")
            .field("joy_sensitivity", &self.joy_sensitivity)
            .field("axis", &self.axis)
            .field("joy_num_axes", &self.joy_num_axes)
            .field("axis_is_active", &self.axis_is_active)
            .field("key_cmds", &self.key_cmds)
            .finish()
    }
}

impl Default for Input {
    fn default() -> Self {
        Self {
            show_cursor: false,
            wheel_up_events: 0,
            wheel_down_events: 0,
            last_mouse_event: 0,
            current_modifiers: 0,
            state: [InputState::default(); PointerStates::Last as usize],
            joy: None,
            joy_sensitivity: 0,
            axis: Point { x: 0, y: 0 },
            joy_num_axes: 0,
            axis_is_active: false,
            key_cmds: default_key_cmds(),
        }
    }
}

#[cfg(feature = "gcw0")]
fn default_key_cmds() -> [[u16; 3]; Cmds::Last as usize] {
    [
        [u32_to_u16(SDLKey_SDLK_UP), PointerStates::JoyUp.to_u16(), 0], // CMD_UP
        [
            u32_to_u16(SDLKey_SDLK_DOWN),
            PointerStates::JoyDown.to_u16(),
            0,
        ], // CMD_DOWN
        [
            u32_to_u16(SDLKey_SDLK_LEFT),
            PointerStates::JoyLeft.to_u16(),
            0,
        ], // CMD_LEFT
        [
            u32_to_u16(SDLKey_SDLK_RIGHT),
            PointerStates::JoyRight.to_u16(),
            0,
        ], // CMD_RIGHT
        [
            u32_to_u16(SDLKey_SDLK_SPACE),
            u32_to_u16(SDLKey_SDLK_LCTRL),
            0,
        ], // CMD_FIRE
        [
            u32_to_u16(SDLKey_SDLK_LALT),
            PointerStates::JoyButton2.to_u16(),
            0,
        ], // CMD_ACTIVATE
        [
            u32_to_u16(SDLKey_SDLK_BACKSPACE),
            u32_to_u16(SDLKey_SDLK_TAB),
            0,
        ], // CMD_TAKEOVER
        [0, 0, 0],                                                      // CMD_QUIT,
        [u32_to_u16(SDLKey_SDLK_RETURN), 0, 0],                         // CMD_PAUSE,
        [0, 0, 0],                                                      // CMD_SCREENSHOT
        [0, 0, 0],                                                      // CMD_FULLSCREEN,
        [
            u32_to_u16(SDLKey_SDLK_ESCAPE),
            PointerStates::JoyButton4.to_u16(),
            0,
        ], // CMD_MENU,
        [
            u32_to_u16(SDLKey_SDLK_ESCAPE),
            PointerStates::JoyButton2.to_u16(),
            PointerStates::MouseButton2.to_u16(),
        ], // CMD_BACK
    ]
}

#[cfg(not(feature = "gcw0"))]
fn default_key_cmds() -> [[u16; 3]; Cmds::Last as usize] {
    [
        [
            u32_to_u16(SDLKey_SDLK_UP),
            PointerStates::JoyUp.to_u16(),
            b'w'.into(),
        ], // CMD_UP
        [
            u32_to_u16(SDLKey_SDLK_DOWN),
            PointerStates::JoyDown.to_u16(),
            b's'.into(),
        ], // CMD_DOWN
        [
            u32_to_u16(SDLKey_SDLK_LEFT),
            PointerStates::JoyLeft.to_u16(),
            b'a'.into(),
        ], // CMD_LEFT
        [
            u32_to_u16(SDLKey_SDLK_RIGHT),
            PointerStates::JoyRight.to_u16(),
            b'd'.into(),
        ], // CMD_RIGHT
        [
            u32_to_u16(SDLKey_SDLK_SPACE),
            PointerStates::JoyButton1.to_u16(),
            PointerStates::MouseButton1.to_u16(),
        ], // CMD_FIRE
        [
            u32_to_u16(SDLKey_SDLK_RETURN),
            u32_to_u16(SDLKey_SDLK_RSHIFT),
            b'e'.into(),
        ], // CMD_ACTIVATE
        [
            u32_to_u16(SDLKey_SDLK_SPACE),
            PointerStates::JoyButton2.to_u16(),
            PointerStates::MouseButton2.to_u16(),
        ], // CMD_TAKEOVER
        [b'q'.into(), 0, 0],                             // CMD_QUIT,
        [u32_to_u16(SDLKey_SDLK_PAUSE), b'p'.into(), 0], // CMD_PAUSE,
        [u32_to_u16(SDLKey_SDLK_F12), 0, 0],             // CMD_SCREENSHOT
        [b'f'.into(), 0, 0],                             // CMD_FULLSCREEN,
        [
            u32_to_u16(SDLKey_SDLK_ESCAPE),
            PointerStates::JoyButton4.to_u16(),
            0,
        ], // CMD_MENU,
        [
            u32_to_u16(SDLKey_SDLK_ESCAPE),
            PointerStates::JoyButton2.to_u16(),
            PointerStates::MouseButton2.to_u16(),
        ], // CMD_BACK
    ]
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
    pub fn wait_for_key_pressed(&mut self) -> u16 {
        loop {
            match self.any_key_just_pressed() {
                0 => self.sdl.delay_ms(1),
                key => break key,
            }
        }
    }

    pub fn any_key_just_pressed(&mut self) -> u16 {
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
    ) -> u16 {
        #[cfg(target_os = "android")]
        assert!(graphics.ne_screen.as_mut().unwrap().flip());

        Self::update_input_static(sdl, input, vars, quit);

        let pressed_key = (0..PointerStates::Last.to_u16())
            .find(|&key| input.state[usize::from(key)].is_just_pressed());

        match pressed_key {
            Some(key) => {
                input.state[usize::from(key)].fresh = false;
                key
            }
            None => 0,
        }
    }

    pub fn update_input(&mut self) -> i32 {
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
    ) -> i32 {
        // switch mouse-cursor visibility as a function of time of last activity
        input.show_cursor = sdl.ticks_ms() - input.last_mouse_event <= CURSOR_KEEP_VISIBLE;

        loop {
            let Some(event) = sdl.next_event() else {
                break;
            };

            match event {
                Event::Quit => {
                    info!("User requested termination, terminating.");
                    quit.set(true);
                    return 0;
                }

                Event::Keyboard(event) => handle_keyboard_event(&event, input),
                Event::JoyAxis(event) => handle_joy_axis_event(event, input),
                Event::JoyButton(event) => handle_joy_button_event(event, input),
                Event::MouseMotion(event) => handle_mouse_motion_event(event, input, vars, sdl),
                Event::MouseButton(event) => handle_mouse_button_event(event, input, sdl),

                _ => break,
            }
        }
        0
    }

    pub fn key_is_pressed(&mut self, key: u16) -> bool {
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
        key: u16,
    ) -> bool {
        Self::update_input_static(sdl, input, vars, quit);

        input.state[usize::from(key)].is_just_pressed()
    }

    /// Does the same as `KeyIsPressed`, but automatically releases the key as well..
    pub fn key_is_pressed_r(&mut self, key: u16) -> bool {
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
        key: u16,
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

        Self::wait_for_all_keys_released_static(input, sdl, graphics);
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

        for state in &mut input.state {
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
        !((self.input.axis_is_active && (self.input.axis.x != 0 || self.input.axis.y != 0))
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
            self.input.joy = Some(joy);
        }
    }
}

impl Input {
    // forget the wheel-counters
    pub fn reset_mouse_wheel(&mut self) {
        self.wheel_up_events = 0;
        self.wheel_down_events = 0;
    }

    pub fn release_key(&mut self, key: u16) {
        self.state[usize::from(key)].set_released();
    }
}

fn handle_keyboard_event(event: &event::Keyboard, input: &mut Input) {
    input.current_modifiers = event.keysym.mod_.bits();
    match event.ty {
        KeyboardEventType::KeyDown => {
            input.state[event.keysym.symbol.to_usize()].set_just_pressed();
            #[cfg(feature = "gcw0")]
            if input.axis.x != 0 || input.axis.y != 0 {
                input.axis_is_active = true; // 4 GCW-0 ; breaks cursor keys after axis has been active...
            }
        }
        KeyboardEventType::KeyUp => {
            input.state[event.keysym.symbol.to_usize()].set_just_released();
            #[cfg(feature = "gcw0")]
            {
                input.axis_is_active = false;
            }
        }
    }
}

fn handle_joy_axis_event(event: event::JoyAxis, input: &mut Input) {
    let axis = event.axis;
    let get_value = || i32::from(input.joy_sensitivity) * i32::from(event.value);
    if axis == 0 || ((input.joy_num_axes >= 5) && (axis == 3))
    /* x-axis */
    {
        input.axis.x = event.value.into();

        // this is a bit tricky, because we want to allow direction keys
        // to be soft-released. When mapping the joystick->keyboard, we
        // therefore have to make sure that this mapping only occurs when
        // and actual _change_ of the joystick-direction ('digital') occurs
        // so that it behaves like "set"/"release"
        if get_value() > 10000 {
            /* about half tilted */
            input.state[PointerStates::JoyRight as usize].set_just_pressed();
            input.state[PointerStates::JoyLeft as usize].set_released();
        } else if get_value() < -10000 {
            input.state[PointerStates::JoyLeft as usize].set_just_pressed();
            input.state[PointerStates::JoyRight as usize].set_released();
        } else {
            input.state[PointerStates::JoyLeft as usize].set_released();
            input.state[PointerStates::JoyRight as usize].set_released();
        }
    } else if (axis == 1) || ((input.joy_num_axes >= 5) && (axis == 4)) {
        /* y-axis */
        input.axis.y = event.value.into();

        if get_value() > 10000 {
            input.state[PointerStates::JoyDown as usize].set_just_pressed();
            input.state[PointerStates::JoyUp as usize].set_released();
        } else if get_value() < -10000 {
            input.state[PointerStates::JoyUp as usize].set_just_pressed();
            input.state[PointerStates::JoyDown as usize].set_released();
        } else {
            input.state[PointerStates::JoyUp as usize].set_released();
            input.state[PointerStates::JoyDown as usize].set_released();
        }
    }
}

fn handle_joy_button_event(event: event::JoyButton, input: &mut Input) {
    let is_pressed = event.state.is_pressed();
    let input_state_index = match event.button {
        0 => Some(PointerStates::JoyButton1 as usize),
        1 => Some(PointerStates::JoyButton2 as usize),
        2 => Some(PointerStates::JoyButton3 as usize),
        3 => Some(PointerStates::JoyButton4 as usize),
        _ => None,
    };
    if let Some(input_state_index) = input_state_index {
        let input_state = &mut input.state[input_state_index];
        input_state.pressed = is_pressed;
        input_state.fresh = true;
    }
    input.axis_is_active = is_pressed;
}

fn handle_mouse_motion_event(event: event::MouseMotion, input: &mut Input, vars: &Vars, sdl: &Sdl) {
    let user_center = vars.get_user_center();
    input.axis.x = i32::from(event.x) - i32::from(user_center.x()) + 16;
    input.axis.y = i32::from(event.y) - i32::from(user_center.y()) + 16;

    input.last_mouse_event = sdl.ticks_ms();
}

fn handle_mouse_button_event(event: event::MouseButton, input: &mut Input, sdl: &Sdl) {
    const BUTTON_LEFT: u8 = i32_to_u8(SDL_BUTTON_LEFT);
    const BUTTON_RIGHT: u8 = i32_to_u8(SDL_BUTTON_RIGHT);
    const BUTTON_MIDDLE: u8 = i32_to_u8(SDL_BUTTON_MIDDLE);
    const BUTTON_WHEELUP: u8 = i32_to_u8(SDL_BUTTON_WHEELUP);
    const BUTTON_WHEELDOWN: u8 = i32_to_u8(SDL_BUTTON_WHEELDOWN);

    let is_pressed = event.state.is_pressed();
    let input_state_index = match event.button {
        BUTTON_LEFT => {
            input.axis_is_active = is_pressed;
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
        let input_state = &mut input.state[input_state_index];
        input_state.pressed = is_pressed;
        input_state.fresh = true;
    }
    if is_pressed {
        input.last_mouse_event = sdl.ticks_ms();
    }
}
