#[cfg(target_os = "android")]
use crate::graphics::NE_SCREEN;
use crate::{
    defs::{Cmds, MenuAction, PointerStates},
    structs::Point,
    Data,
};

use cstr::cstr;
use log::info;
use sdl::Joystick;
#[cfg(feature = "gcw0")]
use sdl_sys::{SDLKey_SDLK_BACKSPACE, SDLKey_SDLK_LALT, SDLKey_SDLK_LCTRL, SDLKey_SDLK_TAB};
use sdl_sys::{
    SDLKey_SDLK_DOWN, SDLKey_SDLK_ESCAPE, SDLKey_SDLK_LEFT, SDLKey_SDLK_RETURN, SDLKey_SDLK_RIGHT,
    SDLKey_SDLK_SPACE, SDLKey_SDLK_UP, SDLMod, SDL_Event, SDL_PollEvent, SDL_BUTTON_LEFT,
    SDL_BUTTON_MIDDLE, SDL_BUTTON_RIGHT, SDL_BUTTON_WHEELDOWN, SDL_BUTTON_WHEELUP,
};
#[cfg(not(feature = "gcw0"))]
use sdl_sys::{SDLKey_SDLK_F12, SDLKey_SDLK_PAUSE, SDLKey_SDLK_RSHIFT};
use std::{
    fmt,
    os::raw::{c_char, c_int},
    ptr::null,
};

pub struct Input {
    pub show_cursor: bool,
    wheel_up_events: c_int,
    wheel_down_events: c_int,
    pub last_mouse_event: u32,
    current_modifiers: SDLMod,
    input_state: [c_int; PointerStates::Last as usize],
    event: SDL_Event,
    pub joy: Option<Joystick>,
    pub joy_sensitivity: c_int,
    // joystick (and mouse) axis values
    pub input_axis: Point,
    // number of joystick axes
    pub joy_num_axes: u16,
    // is firing to use axis-values or not
    pub axis_is_active: i32,
    pub key_cmds: [[c_int; 3]; Cmds::Last as usize],
    pub keystr: [*const c_char; PointerStates::Last as usize],
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
            .field("joy_sensitivity", &self.joy_sensitivity)
            .field("input_axis", &self.input_axis)
            .field("joy_num_axes", &self.joy_num_axes)
            .field("axis_is_active", &self.axis_is_active)
            .field("key_cmds", &self.key_cmds)
            .field("keystr", &self.keystr)
            .finish()
    }
}

impl Default for Input {
    fn default() -> Self {
        #[cfg(feature = "gcw0")]
        let key_cmds = [
            [SDLKey_SDLK_UP as c_int, PointerStates::JoyUp as c_int, 0], // CMD_UP
            [
                SDLKey_SDLK_DOWN as c_int,
                PointerStates::JoyDown as c_int,
                0,
            ], // CMD_DOWN
            [
                SDLKey_SDLK_LEFT as c_int,
                PointerStates::JoyLeft as c_int,
                0,
            ], // CMD_LEFT
            [
                SDLKey_SDLK_RIGHT as c_int,
                PointerStates::JoyRight as c_int,
                0,
            ], // CMD_RIGHT
            [SDLKey_SDLK_SPACE as c_int, SDLKey_SDLK_LCTRL as c_int, 0], // CMD_FIRE
            [
                SDLKey_SDLK_LALT as c_int,
                PointerStates::JoyButton2 as c_int,
                0,
            ], // CMD_ACTIVATE
            [SDLKey_SDLK_BACKSPACE as c_int, SDLKey_SDLK_TAB as c_int, 0], // CMD_TAKEOVER
            [0, 0, 0],                                                   // CMD_QUIT,
            [SDLKey_SDLK_RETURN as c_int, 0, 0],                         // CMD_PAUSE,
            [0, 0, 0],                                                   // CMD_SCREENSHOT
            [0, 0, 0],                                                   // CMD_FULLSCREEN,
            [
                SDLKey_SDLK_ESCAPE as c_int,
                PointerStates::JoyButton4 as c_int,
                0,
            ], // CMD_MENU,
            [
                SDLKey_SDLK_ESCAPE as c_int,
                PointerStates::JoyButton2 as c_int,
                PointerStates::MouseButton2 as c_int,
            ], // CMD_BACK
        ];

        #[cfg(not(feature = "gcw0"))]
        let key_cmds = [
            [
                SDLKey_SDLK_UP as c_int,
                PointerStates::JoyUp as c_int,
                b'w' as c_int,
            ], // CMD_UP
            [
                SDLKey_SDLK_DOWN as c_int,
                PointerStates::JoyDown as c_int,
                b's' as c_int,
            ], // CMD_DOWN
            [
                SDLKey_SDLK_LEFT as c_int,
                PointerStates::JoyLeft as c_int,
                b'a' as c_int,
            ], // CMD_LEFT
            [
                SDLKey_SDLK_RIGHT as c_int,
                PointerStates::JoyRight as c_int,
                b'd' as c_int,
            ], // CMD_RIGHT
            [
                SDLKey_SDLK_SPACE as c_int,
                PointerStates::JoyButton1 as c_int,
                PointerStates::MouseButton1 as c_int,
            ], // CMD_FIRE
            [
                SDLKey_SDLK_RETURN as c_int,
                SDLKey_SDLK_RSHIFT as c_int,
                b'e' as c_int,
            ], // CMD_ACTIVATE
            [
                SDLKey_SDLK_SPACE as c_int,
                PointerStates::JoyButton2 as c_int,
                PointerStates::MouseButton2 as c_int,
            ], // CMD_TAKEOVER
            [b'q' as c_int, 0, 0],                          // CMD_QUIT,
            [SDLKey_SDLK_PAUSE as c_int, b'p' as c_int, 0], // CMD_PAUSE,
            [SDLKey_SDLK_F12 as c_int, 0, 0],               // CMD_SCREENSHOT
            [b'f' as c_int, 0, 0],                          // CMD_FULLSCREEN,
            [
                SDLKey_SDLK_ESCAPE as c_int,
                PointerStates::JoyButton4 as c_int,
                0,
            ], // CMD_MENU,
            [
                SDLKey_SDLK_ESCAPE as c_int,
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
            input_state: [0; PointerStates::Last as usize],
            event: SDL_Event::default(),
            joy: None,
            joy_sensitivity: 0,
            input_axis: Point { x: 0, y: 0 },
            joy_num_axes: 0,
            axis_is_active: 0,
            key_cmds,
            keystr: [null(); PointerStates::Last as usize],
        }
    }
}

pub const CMD_STRINGS: [*const c_char; Cmds::Last as usize] = [
    cstr!("UP").as_ptr(),
    cstr!("DOWN").as_ptr(),
    cstr!("LEFT").as_ptr(),
    cstr!("RIGHT").as_ptr(),
    cstr!("FIRE").as_ptr(),
    cstr!("ACTIVATE").as_ptr(),
    cstr!("TAKEOVER").as_ptr(),
    cstr!("QUIT").as_ptr(),
    cstr!("PAUSE").as_ptr(),
    cstr!("SCREENSHOT").as_ptr(),
    cstr!("FULLSCREEN").as_ptr(),
    cstr!("MENU").as_ptr(),
    cstr!("BACK").as_ptr(),
];

pub const CURSOR_KEEP_VISIBLE: u32 = 3000; // ticks to keep mouse-cursor visible without mouse-input

const FRESH_BIT: c_int = 0x01 << 8;
const PRESSED: c_int = true as c_int | FRESH_BIT;
const RELEASED: c_int = false as c_int | FRESH_BIT;

const fn is_just_pressed(key_flags: c_int) -> bool {
    key_flags & PRESSED == PRESSED
}

fn clear_fresh(key_flags: &mut c_int) {
    *key_flags &= !FRESH_BIT;
}

impl Data<'_> {
    /// Check if any keys have been 'freshly' pressed. If yes, return key-code, otherwise 0.
    pub unsafe fn wait_for_key_pressed(&mut self) -> c_int {
        loop {
            match self.any_key_just_pressed() {
                0 => self.sdl.delay_ms(1),
                key => break key,
            }
        }
    }

    pub unsafe fn any_key_just_pressed(&mut self) -> c_int {
        #[cfg(target_os = "android")]
        assert!(self.graphics.ne_screen.as_mut().unwrap().flip());

        self.update_input();

        let pressed_key = (0..PointerStates::Last as c_int)
            .find(|&key| is_just_pressed(self.input.input_state[key as usize]));

        match pressed_key {
            Some(key) => {
                clear_fresh(&mut self.input.input_state[key as usize]);
                key
            }
            None => 0,
        }
    }

    pub unsafe fn update_input(&mut self) -> c_int {
        // switch mouse-cursor visibility as a function of time of last activity
        if self.sdl.ticks_ms() - self.input.last_mouse_event > CURSOR_KEEP_VISIBLE {
            self.input.show_cursor = false;
        } else {
            self.input.show_cursor = true;
        }

        while SDL_PollEvent(&mut self.input.event) != 0 {
            match self.input.event.type_.into() {
                sdl_sys::SDL_EventType_SDL_QUIT => {
                    info!("User requested termination, terminating.");
                    self.quit_successfully();
                }

                sdl_sys::SDL_EventType_SDL_KEYDOWN => {
                    let key = self.input.event.key;
                    self.input.current_modifiers = key.keysym.mod_;
                    self.input.input_state[usize::try_from(key.keysym.sym).unwrap()] = PRESSED;
                    #[cfg(feature = "gcw0")]
                    if input_axis.x != 0 || input_axis.y != 0 {
                        axis_is_active = true.into(); // 4 GCW-0 ; breaks cursor keys after axis has been active...
                    }
                }
                sdl_sys::SDL_EventType_SDL_KEYUP => {
                    let key = self.input.event.key;
                    self.input.current_modifiers = key.keysym.mod_;
                    self.input.input_state[usize::try_from(key.keysym.sym).unwrap()] = RELEASED;
                    #[cfg(feature = "gcw0")]
                    {
                        axis_is_active = false.into();
                    }
                }

                sdl_sys::SDL_EventType_SDL_JOYAXISMOTION => {
                    let jaxis = self.input.event.jaxis;
                    let axis = jaxis.axis;
                    if axis == 0 || ((self.input.joy_num_axes >= 5) && (axis == 3))
                    /* x-axis */
                    {
                        self.input.input_axis.x = jaxis.value.into();

                        // this is a bit tricky, because we want to allow direction keys
                        // to be soft-released. When mapping the joystick->keyboard, we
                        // therefore have to make sure that this mapping only occurs when
                        // and actual _change_ of the joystick-direction ('digital') occurs
                        // so that it behaves like "set"/"release"
                        if self.input.joy_sensitivity * i32::from(jaxis.value) > 10000 {
                            /* about half tilted */
                            self.input.input_state[PointerStates::JoyRight as usize] = PRESSED;
                            self.input.input_state[PointerStates::JoyLeft as usize] = false.into();
                        } else if self.input.joy_sensitivity * i32::from(jaxis.value) < -10000 {
                            self.input.input_state[PointerStates::JoyLeft as usize] = PRESSED;
                            self.input.input_state[PointerStates::JoyRight as usize] = false.into();
                        } else {
                            self.input.input_state[PointerStates::JoyLeft as usize] = false.into();
                            self.input.input_state[PointerStates::JoyRight as usize] = false.into();
                        }
                    } else if (axis == 1) || ((self.input.joy_num_axes >= 5) && (axis == 4)) {
                        /* y-axis */
                        self.input.input_axis.y = jaxis.value.into();

                        if self.input.joy_sensitivity * i32::from(jaxis.value) > 10000 {
                            self.input.input_state[PointerStates::JoyDown as usize] = PRESSED;
                            self.input.input_state[PointerStates::JoyUp as usize] = false.into();
                        } else if self.input.joy_sensitivity * i32::from(jaxis.value) < -10000 {
                            self.input.input_state[PointerStates::JoyUp as usize] = PRESSED;
                            self.input.input_state[PointerStates::JoyDown as usize] = false.into();
                        } else {
                            self.input.input_state[PointerStates::JoyUp as usize] = false.into();
                            self.input.input_state[PointerStates::JoyDown as usize] = false.into();
                        }
                    }
                }

                sdl_sys::SDL_EventType_SDL_JOYBUTTONDOWN => {
                    let jbutton = self.input.event.jbutton;
                    // first button
                    if jbutton.button == 0 {
                        self.input.input_state[PointerStates::JoyButton1 as usize] = PRESSED;
                    }
                    // second button
                    else if jbutton.button == 1 {
                        self.input.input_state[PointerStates::JoyButton2 as usize] = PRESSED;
                    }
                    // and third button
                    else if jbutton.button == 2 {
                        self.input.input_state[PointerStates::JoyButton3 as usize] = PRESSED;
                    }
                    // and fourth button
                    else if jbutton.button == 3 {
                        self.input.input_state[PointerStates::JoyButton4 as usize] = PRESSED;
                    }

                    self.input.axis_is_active = true.into();
                }

                sdl_sys::SDL_EventType_SDL_JOYBUTTONUP => {
                    let jbutton = self.input.event.jbutton;
                    // first button
                    if jbutton.button == 0 {
                        self.input.input_state[PointerStates::JoyButton1 as usize] = false.into();
                    }
                    // second button
                    else if jbutton.button == 1 {
                        self.input.input_state[PointerStates::JoyButton2 as usize] = false.into();
                    }
                    // and third button
                    else if jbutton.button == 2 {
                        self.input.input_state[PointerStates::JoyButton3 as usize] = false.into();
                    }
                    // and fourth button
                    else if jbutton.button == 3 {
                        self.input.input_state[PointerStates::JoyButton4 as usize] = false.into();
                    }

                    self.input.axis_is_active = false.into();
                }

                sdl_sys::SDL_EventType_SDL_MOUSEMOTION => {
                    let button = self.input.event.button;
                    let user_center = self.vars.get_user_center();
                    self.input.input_axis.x = i32::from(button.x) - i32::from(user_center.x()) + 16;
                    self.input.input_axis.y = i32::from(button.y) - i32::from(user_center.y()) + 16;

                    self.input.last_mouse_event = self.sdl.ticks_ms();
                }

                /* Mouse control */
                sdl_sys::SDL_EventType_SDL_MOUSEBUTTONDOWN => {
                    let button = self.input.event.button;
                    if button.button == SDL_BUTTON_LEFT as u8 {
                        self.input.input_state[PointerStates::MouseButton1 as usize] = PRESSED;
                        self.input.axis_is_active = true.into();
                    }

                    if button.button == SDL_BUTTON_RIGHT as u8 {
                        self.input.input_state[PointerStates::MouseButton2 as usize] = PRESSED;
                    }

                    if button.button == SDL_BUTTON_MIDDLE as u8 {
                        self.input.input_state[PointerStates::MouseButton3 as usize] = PRESSED;
                    }

                    // wheel events are immediately released, so we rather
                    // count the number of not yet read-out events
                    if button.button == SDL_BUTTON_WHEELUP as u8 {
                        self.input.wheel_up_events += 1;
                    }

                    if button.button == SDL_BUTTON_WHEELDOWN as u8 {
                        self.input.wheel_down_events += 1;
                    }

                    self.input.last_mouse_event = self.sdl.ticks_ms();
                }

                sdl_sys::SDL_EventType_SDL_MOUSEBUTTONUP => {
                    let button = self.input.event.button;
                    if button.button == SDL_BUTTON_LEFT as u8 {
                        self.input.input_state[PointerStates::MouseButton1 as usize] = false.into();
                        self.input.axis_is_active = false.into();
                    }

                    if button.button == SDL_BUTTON_RIGHT as u8 {
                        self.input.input_state[PointerStates::MouseButton2 as usize] = false.into();
                    }

                    if button.button == SDL_BUTTON_MIDDLE as u8 {
                        self.input.input_state[PointerStates::MouseButton3 as usize] = false.into();
                    }
                }

                _ => break,
            }
        }

        0
    }

    pub unsafe fn key_is_pressed(&mut self, key: c_int) -> bool {
        self.update_input();

        (self.input.input_state[usize::try_from(key).unwrap()] & PRESSED) == PRESSED
    }

    /// Does the same as KeyIsPressed, but automatically releases the key as well..
    pub unsafe fn key_is_pressed_r(&mut self, key: c_int) -> bool {
        let ret = self.key_is_pressed(key);

        self.release_key(key);
        ret
    }

    pub unsafe fn release_key(&mut self, key: c_int) {
        self.input.input_state[usize::try_from(key).unwrap()] = false.into();
    }

    pub unsafe fn wheel_up_pressed(&mut self) -> bool {
        self.update_input();
        if self.input.wheel_up_events != 0 {
            self.input.wheel_up_events -= 1;
            true
        } else {
            false
        }
    }

    pub unsafe fn wheel_down_pressed(&mut self) -> bool {
        self.update_input();
        if self.input.wheel_down_events != 0 {
            self.input.wheel_down_events -= 1;
            true
        } else {
            false
        }
    }

    pub unsafe fn cmd_is_active(&mut self, cmd: Cmds) -> bool {
        let cmd = cmd as usize;
        self.key_is_pressed(self.input.key_cmds[cmd][0])
            || self.key_is_pressed(self.input.key_cmds[cmd][1])
            || self.key_is_pressed(self.input.key_cmds[cmd][2])
    }

    /// the same but release the keys: use only for menus!
    pub unsafe fn cmd_is_active_r(&mut self, cmd: Cmds) -> bool {
        let cmd = cmd as usize;

        let c1 = self.key_is_pressed_r(self.input.key_cmds[cmd][0]);
        let c2 = self.key_is_pressed_r(self.input.key_cmds[cmd][1]);
        let c3 = self.key_is_pressed_r(self.input.key_cmds[cmd][2]);

        c1 || c2 || c3
    }

    pub unsafe fn wait_for_all_keys_released(&mut self) {
        while self.any_key_is_pressed_r() {
            self.sdl.delay_ms(1);
        }
        self.reset_mouse_wheel();
    }

    pub unsafe fn any_key_is_pressed_r(&mut self) -> bool {
        #[cfg(target_os = "android")]
        assert!(self.graphics.ne_screen.as_mut().unwrap().flip());

        #[cfg(not(target_os = "android"))]
        self.update_input();

        for state in &mut self.input.input_state {
            if (*state & PRESSED) != 0 {
                *state = 0;
                return true;
            }
        }
        false
    }

    // forget the wheel-counters
    pub unsafe fn reset_mouse_wheel(&mut self) {
        self.input.wheel_up_events = 0;
        self.input.wheel_down_events = 0;
    }

    pub unsafe fn mod_is_pressed(&mut self, sdl_mod: SDLMod) -> bool {
        self.update_input();
        (self.input.current_modifiers & sdl_mod) != 0
    }

    pub unsafe fn no_direction_pressed(&mut self) -> bool {
        !((self.input.axis_is_active != 0
            && (self.input.input_axis.x != 0 || self.input.input_axis.y != 0))
            || self.down_pressed()
            || self.up_pressed()
            || self.left_pressed()
            || self.right_pressed())
    }

    pub unsafe fn react_to_special_keys(&mut self) {
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

    pub unsafe fn init_joy(&mut self) {
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

    pub fn init_keystr(&mut self) {
        use sdl_sys::*;

        self.input.keystr[0] = cstr!("NONE").as_ptr(); // Empty bind will otherwise crash on some platforms - also, we choose "NONE" as a placeholder...
        #[cfg(feature = "gcw0")]
        {
            // The GCW0 may change to joystick input altogether in the future - which will make these ifdefs unnecessary, I hope...
            self.input.keystr[SDLKey_SDLK_BACKSPACE as usize] = cstr!("RSldr").as_ptr();
            self.input.keystr[SDLKey_SDLK_TAB as usize] = cstr!("LSldr").as_ptr();
            self.input.keystr[SDLKey_SDLK_RETURN as usize] = cstr!("Start").as_ptr();
            self.input.keystr[SDLKey_SDLK_SPACE as usize] = cstr!("Y").as_ptr();
            self.input.keystr[SDLKey_SDLK_ESCAPE as usize] = cstr!("Select").as_ptr();
        }

        #[cfg(not(feature = "gcw0"))]
        {
            self.input.keystr[SDLKey_SDLK_BACKSPACE as usize] = cstr!("BS").as_ptr();
            self.input.keystr[SDLKey_SDLK_TAB as usize] = cstr!("Tab").as_ptr();
            self.input.keystr[SDLKey_SDLK_RETURN as usize] = cstr!("Return").as_ptr();
            self.input.keystr[SDLKey_SDLK_SPACE as usize] = cstr!("Space").as_ptr();
            self.input.keystr[SDLKey_SDLK_ESCAPE as usize] = cstr!("Esc").as_ptr();
        }

        self.input.keystr[SDLKey_SDLK_CLEAR as usize] = cstr!("Clear").as_ptr();
        self.input.keystr[SDLKey_SDLK_PAUSE as usize] = cstr!("Pause").as_ptr();
        self.input.keystr[SDLKey_SDLK_EXCLAIM as usize] = cstr!("!").as_ptr();
        self.input.keystr[SDLKey_SDLK_QUOTEDBL as usize] = cstr!("\"").as_ptr();
        self.input.keystr[SDLKey_SDLK_HASH as usize] = cstr!("#").as_ptr();
        self.input.keystr[SDLKey_SDLK_DOLLAR as usize] = cstr!("$").as_ptr();
        self.input.keystr[SDLKey_SDLK_AMPERSAND as usize] = cstr!("&").as_ptr();
        self.input.keystr[SDLKey_SDLK_QUOTE as usize] = cstr!("'").as_ptr();
        self.input.keystr[SDLKey_SDLK_LEFTPAREN as usize] = cstr!("(").as_ptr();
        self.input.keystr[SDLKey_SDLK_RIGHTPAREN as usize] = cstr!(")").as_ptr();
        self.input.keystr[SDLKey_SDLK_ASTERISK as usize] = cstr!("*").as_ptr();
        self.input.keystr[SDLKey_SDLK_PLUS as usize] = cstr!("+").as_ptr();
        self.input.keystr[SDLKey_SDLK_COMMA as usize] = cstr!(",").as_ptr();
        self.input.keystr[SDLKey_SDLK_MINUS as usize] = cstr!("-").as_ptr();
        self.input.keystr[SDLKey_SDLK_PERIOD as usize] = cstr!(".").as_ptr();
        self.input.keystr[SDLKey_SDLK_SLASH as usize] = cstr!("/").as_ptr();
        self.input.keystr[SDLKey_SDLK_0 as usize] = cstr!("0").as_ptr();
        self.input.keystr[SDLKey_SDLK_1 as usize] = cstr!("1").as_ptr();
        self.input.keystr[SDLKey_SDLK_2 as usize] = cstr!("2").as_ptr();
        self.input.keystr[SDLKey_SDLK_3 as usize] = cstr!("3").as_ptr();
        self.input.keystr[SDLKey_SDLK_4 as usize] = cstr!("4").as_ptr();
        self.input.keystr[SDLKey_SDLK_5 as usize] = cstr!("5").as_ptr();
        self.input.keystr[SDLKey_SDLK_6 as usize] = cstr!("6").as_ptr();
        self.input.keystr[SDLKey_SDLK_7 as usize] = cstr!("7").as_ptr();
        self.input.keystr[SDLKey_SDLK_8 as usize] = cstr!("8").as_ptr();
        self.input.keystr[SDLKey_SDLK_9 as usize] = cstr!("9").as_ptr();
        self.input.keystr[SDLKey_SDLK_COLON as usize] = cstr!(":").as_ptr();
        self.input.keystr[SDLKey_SDLK_SEMICOLON as usize] = cstr!(";").as_ptr();
        self.input.keystr[SDLKey_SDLK_LESS as usize] = cstr!("<").as_ptr();
        self.input.keystr[SDLKey_SDLK_EQUALS as usize] = cstr!("=").as_ptr();
        self.input.keystr[SDLKey_SDLK_GREATER as usize] = cstr!(">").as_ptr();
        self.input.keystr[SDLKey_SDLK_QUESTION as usize] = cstr!("?").as_ptr();
        self.input.keystr[SDLKey_SDLK_AT as usize] = cstr!("@").as_ptr();
        self.input.keystr[SDLKey_SDLK_LEFTBRACKET as usize] = cstr!("[").as_ptr();
        self.input.keystr[SDLKey_SDLK_BACKSLASH as usize] = cstr!("\\").as_ptr();
        self.input.keystr[SDLKey_SDLK_RIGHTBRACKET as usize] = cstr!(" as usize]").as_ptr();
        self.input.keystr[SDLKey_SDLK_CARET as usize] = cstr!("^").as_ptr();
        self.input.keystr[SDLKey_SDLK_UNDERSCORE as usize] = cstr!("_").as_ptr();
        self.input.keystr[SDLKey_SDLK_BACKQUOTE as usize] = cstr!("`").as_ptr();
        self.input.keystr[SDLKey_SDLK_a as usize] = cstr!("a").as_ptr();
        self.input.keystr[SDLKey_SDLK_b as usize] = cstr!("b").as_ptr();
        self.input.keystr[SDLKey_SDLK_c as usize] = cstr!("c").as_ptr();
        self.input.keystr[SDLKey_SDLK_d as usize] = cstr!("d").as_ptr();
        self.input.keystr[SDLKey_SDLK_e as usize] = cstr!("e").as_ptr();
        self.input.keystr[SDLKey_SDLK_f as usize] = cstr!("f").as_ptr();
        self.input.keystr[SDLKey_SDLK_g as usize] = cstr!("g").as_ptr();
        self.input.keystr[SDLKey_SDLK_h as usize] = cstr!("h").as_ptr();
        self.input.keystr[SDLKey_SDLK_i as usize] = cstr!("i").as_ptr();
        self.input.keystr[SDLKey_SDLK_j as usize] = cstr!("j").as_ptr();
        self.input.keystr[SDLKey_SDLK_k as usize] = cstr!("k").as_ptr();
        self.input.keystr[SDLKey_SDLK_l as usize] = cstr!("l").as_ptr();
        self.input.keystr[SDLKey_SDLK_m as usize] = cstr!("m").as_ptr();
        self.input.keystr[SDLKey_SDLK_n as usize] = cstr!("n").as_ptr();
        self.input.keystr[SDLKey_SDLK_o as usize] = cstr!("o").as_ptr();
        self.input.keystr[SDLKey_SDLK_p as usize] = cstr!("p").as_ptr();
        self.input.keystr[SDLKey_SDLK_q as usize] = cstr!("q").as_ptr();
        self.input.keystr[SDLKey_SDLK_r as usize] = cstr!("r").as_ptr();
        self.input.keystr[SDLKey_SDLK_s as usize] = cstr!("s").as_ptr();
        self.input.keystr[SDLKey_SDLK_t as usize] = cstr!("t").as_ptr();
        self.input.keystr[SDLKey_SDLK_u as usize] = cstr!("u").as_ptr();
        self.input.keystr[SDLKey_SDLK_v as usize] = cstr!("v").as_ptr();
        self.input.keystr[SDLKey_SDLK_w as usize] = cstr!("w").as_ptr();
        self.input.keystr[SDLKey_SDLK_x as usize] = cstr!("x").as_ptr();
        self.input.keystr[SDLKey_SDLK_y as usize] = cstr!("y").as_ptr();
        self.input.keystr[SDLKey_SDLK_z as usize] = cstr!("z").as_ptr();
        self.input.keystr[SDLKey_SDLK_DELETE as usize] = cstr!("Del").as_ptr();

        /* Numeric keypad */
        self.input.keystr[SDLKey_SDLK_KP0 as usize] = cstr!("Num[0 as usize]").as_ptr();
        self.input.keystr[SDLKey_SDLK_KP1 as usize] = cstr!("Num[1 as usize]").as_ptr();
        self.input.keystr[SDLKey_SDLK_KP2 as usize] = cstr!("Num[2 as usize]").as_ptr();
        self.input.keystr[SDLKey_SDLK_KP3 as usize] = cstr!("Num[3 as usize]").as_ptr();
        self.input.keystr[SDLKey_SDLK_KP4 as usize] = cstr!("Num[4 as usize]").as_ptr();
        self.input.keystr[SDLKey_SDLK_KP5 as usize] = cstr!("Num[5 as usize]").as_ptr();
        self.input.keystr[SDLKey_SDLK_KP6 as usize] = cstr!("Num[6 as usize]").as_ptr();
        self.input.keystr[SDLKey_SDLK_KP7 as usize] = cstr!("Num[7 as usize]").as_ptr();
        self.input.keystr[SDLKey_SDLK_KP8 as usize] = cstr!("Num[8 as usize]").as_ptr();
        self.input.keystr[SDLKey_SDLK_KP9 as usize] = cstr!("Num[9 as usize]").as_ptr();
        self.input.keystr[SDLKey_SDLK_KP_PERIOD as usize] = cstr!("Num[. as usize]").as_ptr();
        self.input.keystr[SDLKey_SDLK_KP_DIVIDE as usize] = cstr!("Num[/ as usize]").as_ptr();
        self.input.keystr[SDLKey_SDLK_KP_MULTIPLY as usize] = cstr!("Num[* as usize]").as_ptr();
        self.input.keystr[SDLKey_SDLK_KP_MINUS as usize] = cstr!("Num[- as usize]").as_ptr();
        self.input.keystr[SDLKey_SDLK_KP_PLUS as usize] = cstr!("Num[+ as usize]").as_ptr();
        self.input.keystr[SDLKey_SDLK_KP_ENTER as usize] = cstr!("Num[Enter as usize]").as_ptr();
        self.input.keystr[SDLKey_SDLK_KP_EQUALS as usize] = cstr!("Num[= as usize]").as_ptr();

        /* Arrows + Home/End pad */
        self.input.keystr[SDLKey_SDLK_UP as usize] = cstr!("Up").as_ptr();
        self.input.keystr[SDLKey_SDLK_DOWN as usize] = cstr!("Down").as_ptr();
        self.input.keystr[SDLKey_SDLK_RIGHT as usize] = cstr!("Right").as_ptr();
        self.input.keystr[SDLKey_SDLK_LEFT as usize] = cstr!("Left").as_ptr();
        self.input.keystr[SDLKey_SDLK_INSERT as usize] = cstr!("Insert").as_ptr();
        self.input.keystr[SDLKey_SDLK_HOME as usize] = cstr!("Home").as_ptr();
        self.input.keystr[SDLKey_SDLK_END as usize] = cstr!("End").as_ptr();
        self.input.keystr[SDLKey_SDLK_PAGEUP as usize] = cstr!("PageUp").as_ptr();
        self.input.keystr[SDLKey_SDLK_PAGEDOWN as usize] = cstr!("PageDown").as_ptr();

        /* Function keys */
        self.input.keystr[SDLKey_SDLK_F1 as usize] = cstr!("F1").as_ptr();
        self.input.keystr[SDLKey_SDLK_F2 as usize] = cstr!("F2").as_ptr();
        self.input.keystr[SDLKey_SDLK_F3 as usize] = cstr!("F3").as_ptr();
        self.input.keystr[SDLKey_SDLK_F4 as usize] = cstr!("F4").as_ptr();
        self.input.keystr[SDLKey_SDLK_F5 as usize] = cstr!("F5").as_ptr();
        self.input.keystr[SDLKey_SDLK_F6 as usize] = cstr!("F6").as_ptr();
        self.input.keystr[SDLKey_SDLK_F7 as usize] = cstr!("F7").as_ptr();
        self.input.keystr[SDLKey_SDLK_F8 as usize] = cstr!("F8").as_ptr();
        self.input.keystr[SDLKey_SDLK_F9 as usize] = cstr!("F9").as_ptr();
        self.input.keystr[SDLKey_SDLK_F10 as usize] = cstr!("F10").as_ptr();
        self.input.keystr[SDLKey_SDLK_F11 as usize] = cstr!("F11").as_ptr();
        self.input.keystr[SDLKey_SDLK_F12 as usize] = cstr!("F12").as_ptr();
        self.input.keystr[SDLKey_SDLK_F13 as usize] = cstr!("F13").as_ptr();
        self.input.keystr[SDLKey_SDLK_F14 as usize] = cstr!("F14").as_ptr();
        self.input.keystr[SDLKey_SDLK_F15 as usize] = cstr!("F15").as_ptr();

        /* Key state modifier keys */
        self.input.keystr[SDLKey_SDLK_NUMLOCK as usize] = cstr!("NumLock").as_ptr();
        self.input.keystr[SDLKey_SDLK_CAPSLOCK as usize] = cstr!("CapsLock").as_ptr();
        self.input.keystr[SDLKey_SDLK_SCROLLOCK as usize] = cstr!("ScrlLock").as_ptr();
        #[cfg(feature = "gcw0")]
        {
            keystr[SDLKey_SDLK_LSHIFT as usize] = cstr!("X").as_ptr();
            keystr[SDLKey_SDLK_LCTRL as usize] = cstr!("A").as_ptr();
            keystr[SDLKey_SDLK_LALT as usize] = cstr!("B").as_ptr();
        }

        #[cfg(not(feature = "gcw0"))]
        {
            self.input.keystr[SDLKey_SDLK_LSHIFT as usize] = cstr!("LShift").as_ptr();
            self.input.keystr[SDLKey_SDLK_LCTRL as usize] = cstr!("LCtrl").as_ptr();
            self.input.keystr[SDLKey_SDLK_LALT as usize] = cstr!("LAlt").as_ptr();
        }

        self.input.keystr[SDLKey_SDLK_RSHIFT as usize] = cstr!("RShift").as_ptr();
        self.input.keystr[SDLKey_SDLK_RCTRL as usize] = cstr!("RCtrl").as_ptr();
        self.input.keystr[SDLKey_SDLK_RALT as usize] = cstr!("RAlt").as_ptr();
        self.input.keystr[SDLKey_SDLK_RMETA as usize] = cstr!("RMeta").as_ptr();
        self.input.keystr[SDLKey_SDLK_LMETA as usize] = cstr!("LMeta").as_ptr();
        self.input.keystr[SDLKey_SDLK_LSUPER as usize] = cstr!("LSuper").as_ptr();
        self.input.keystr[SDLKey_SDLK_RSUPER as usize] = cstr!("RSuper").as_ptr();
        self.input.keystr[SDLKey_SDLK_MODE as usize] = cstr!("Mode").as_ptr();
        self.input.keystr[SDLKey_SDLK_COMPOSE as usize] = cstr!("Compose").as_ptr();

        /* Miscellaneous function keys */
        self.input.keystr[SDLKey_SDLK_HELP as usize] = cstr!("Help").as_ptr();
        self.input.keystr[SDLKey_SDLK_PRINT as usize] = cstr!("Print").as_ptr();
        self.input.keystr[SDLKey_SDLK_SYSREQ as usize] = cstr!("SysReq").as_ptr();
        self.input.keystr[SDLKey_SDLK_BREAK as usize] = cstr!("Break").as_ptr();
        self.input.keystr[SDLKey_SDLK_MENU as usize] = cstr!("Menu").as_ptr();
        self.input.keystr[SDLKey_SDLK_POWER as usize] = cstr!("Power").as_ptr();
        self.input.keystr[SDLKey_SDLK_EURO as usize] = cstr!("Euro").as_ptr();
        self.input.keystr[SDLKey_SDLK_UNDO as usize] = cstr!("Undo").as_ptr();

        /* Mouse und Joy buttons */
        self.input.keystr[PointerStates::MouseButton1 as usize] = cstr!("Mouse1").as_ptr();
        self.input.keystr[PointerStates::MouseButton2 as usize] = cstr!("Mouse2").as_ptr();
        self.input.keystr[PointerStates::MouseButton3 as usize] = cstr!("Mouse3").as_ptr();
        self.input.keystr[PointerStates::MouseWheelup as usize] = cstr!("WheelUp").as_ptr();
        self.input.keystr[PointerStates::MouseWheeldown as usize] = cstr!("WheelDown").as_ptr();

        self.input.keystr[PointerStates::JoyUp as usize] = cstr!("JoyUp").as_ptr();
        self.input.keystr[PointerStates::JoyDown as usize] = cstr!("JoyDown").as_ptr();
        self.input.keystr[PointerStates::JoyLeft as usize] = cstr!("JoyLeft").as_ptr();
        self.input.keystr[PointerStates::JoyRight as usize] = cstr!("JoyRight").as_ptr();
        self.input.keystr[PointerStates::JoyButton1 as usize] = cstr!("Joy-A").as_ptr();
        self.input.keystr[PointerStates::JoyButton2 as usize] = cstr!("Joy-B").as_ptr();
        self.input.keystr[PointerStates::JoyButton3 as usize] = cstr!("Joy-X").as_ptr();
        self.input.keystr[PointerStates::JoyButton4 as usize] = cstr!("Joy-Y").as_ptr();
    }
}
