use crate::{
    b_font::{char_width, font_height},
    defs::{Cmds, PointerStates, SHOW_WAIT, TEXT_STRETCH},
    misc::my_random,
    Data,
};

#[cfg(feature = "arcade-input")]
use crate::{
    defs::{down_pressed_r, left_pressed_r, right_pressed_r, up_pressed_r},
    input::{cmd_is_active_r, key_is_pressed_r},
};

use cstr::cstr;
use log::{error, info, trace};
#[cfg(not(feature = "arcade-input"))]
use sdl_sys::SDLKey_SDLK_DELETE;
use sdl_sys::{
    SDLKey_SDLK_BACKSPACE, SDLKey_SDLK_RETURN, SDLMod, SDLMod_KMOD_LSHIFT, SDLMod_KMOD_RSHIFT,
    SDL_CreateRGBSurface, SDL_Delay, SDL_DisplayFormat, SDL_Event, SDL_EventType, SDL_Flip,
    SDL_FreeSurface, SDL_GetClipRect, SDL_GetTicks, SDL_PushEvent, SDL_Rect, SDL_SetClipRect,
    SDL_Surface, SDL_UpdateRect, SDL_UpperBlit, SDL_WaitEvent, SDL_BUTTON_LEFT, SDL_BUTTON_MIDDLE,
    SDL_BUTTON_RIGHT, SDL_BUTTON_WHEELDOWN, SDL_BUTTON_WHEELUP,
};
use std::{
    convert::{TryFrom, TryInto},
    ffi::CStr,
    fmt,
    io::Cursor,
    os::raw::{c_char, c_int, c_uchar},
    ptr::null_mut,
};

#[cfg(feature = "arcade-input")]
const ARCADE_INPUT_CHARS: [c_int; 70] = [
    48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 61, 42, 43, 44, 45, 46, 47, 65, 66, 67, 68, 69, 70, 71,
    72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 32, 97, 98, 99,
    100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118,
    119, 120, 121, 122,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text {
    my_cursor_x: c_int,
    my_cursor_y: c_int,
    text_buffer: [u8; 10000],

    #[cfg(feature = "arcade-input")]
    last_frame_time: u32,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            my_cursor_x: 0,
            my_cursor_y: 0,
            text_buffer: [0; 10000],

            #[cfg(feature = "arcade-input")]
            last_frame_time: 0,
        }
    }
}

impl Data {
    /// Reads a string of "MaxLen" from User-input, and echos it
    /// either to stdout or using graphics-text, depending on the
    /// parameter "echo":
    /// * echo=0    no echo
    /// * echo=1    print using printf
    /// * echo=2    print using graphics-text
    ///
    /// values of echo > 2 are ignored and treated like echo=0
    pub unsafe fn get_string(&mut self, max_len: c_int, echo: c_int) -> *mut c_char {
        let max_len: usize = max_len.try_into().unwrap();

        if echo == 1 {
            error!("GetString(): sorry, echo=1 currently not implemented!\n");
            return null_mut();
        }

        let x0 = self.text.my_cursor_x;
        let y0 = self.text.my_cursor_y;
        let height = font_height(&*self.b_font.current_font);

        let store = SDL_CreateRGBSurface(
            0,
            self.vars.screen_rect.w.into(),
            height,
            self.graphics.vid_bpp,
            0,
            0,
            0,
            0,
        );
        let mut store_rect = rect!(
            x0.try_into().unwrap(),
            y0.try_into().unwrap(),
            self.vars.screen_rect.w,
            height.try_into().unwrap(),
        );
        SDL_UpperBlit(self.graphics.ne_screen, &mut store_rect, store, null_mut());

        #[cfg(feature = "arcade-input")]
        let blink_time = 200; // For adjusting fast <->slow blink; in ms
        #[cfg(feature = "arcade-input")]
        let mut inputchar: c_int = 17; // initial char = A
        #[cfg(feature = "arcade-input")]
        let empty_char = b' ' as c_char; //for "empty" input line / backspace etc...

        #[cfg(not(feature = "arcade-input"))]
        let empty_char = b'.' as c_char; //for "empty" input linue / backspace etc...

        let mut input = vec![empty_char; max_len + 5].into_boxed_slice();
        input[max_len] = 0;

        let mut finished = false;
        let mut curpos = 0;

        while !finished {
            let mut tmp_rect = store_rect;
            SDL_UpperBlit(store, null_mut(), self.graphics.ne_screen, &mut tmp_rect);
            self.put_string(
                self.graphics.ne_screen,
                x0,
                y0,
                CStr::from_ptr(input.as_ptr()).to_bytes(),
            );
            SDL_Flip(self.graphics.ne_screen);

            #[cfg(feature = "arcade-input")]
            {
                if inputchar < 0 {
                    inputchar += ARCADE_INPUT_CHARS.len() as i32;
                }
                if inputchar >= ARCADE_INPUT_CHARS.len() as i32 {
                    inputchar -= ARCADE_INPUT_CHARS.len() as i32;
                }
                let key = ARCADE_INPUT_CHARS[usize::try_from(inputchar).unwrap()];

                let frame_duration = SDL_GetTicks() - self.text.last_frame_time;
                if frame_duration > blink_time / 2 {
                    input[curpos] = key.try_into().unwrap(); // We want to show the currently chosen character
                    if frame_duration > blink_time {
                        self.text.last_frame_time = SDL_GetTicks();
                    } else {
                        input[curpos] = empty_char; // Hmm., how to get character widht? If using '.', or any fill character, we'd need to know
                    }

                    if KeyIsPressedR(SDLKey_SDLK_RETURN.try_into().unwrap()) {
                        // For GCW0, maybe we need a prompt to say [PRESS ENTER WHEN FINISHED], or any other key we may choose...
                        input[curpos] = 0; // The last char is currently shown but, not entered into the string...
                                           // 	  input[curpos] = key; // Not sure which one would be expected by most users; the last blinking char is input or not?
                        finished = true;
                    } else if UpPressedR() {
                        /* Currently, the key will work ON RELEASE; we might change this to
                         * ON PRESS and add a counter / delay after which while holding, will
                         * scroll trough the chars */
                        inputchar += 1;
                    } else if DownPressedR() {
                        inputchar -= 1;
                    } else if FirePressedR() {
                        // ADVANCE CURSOR
                        input[curpos] = key.try_into().unwrap(); // Needed in case character has just blinked out...
                        curpos += 1;
                    } else if LeftPressedR() {
                        inputchar -= 5;
                    } else if RightPressedR() {
                        inputchar += 5;
                    } else if cmd_is_activeR(Cmds::Activate) {
                        // CAPITAL <-> small
                        if (17..=42).contains(&inputchar) {
                            inputchar = 44 + (inputchar - 17);
                        } else if (44..=69).contains(&inputchar) {
                            inputchar = 17 + (inputchar - 44);
                        }
                    } else if KeyIsPressedR(SDLKey_SDLK_BACKSPACE.try_into().unwrap()) {
                        // else if ... other functions to consider: SPACE
                        // Or any othe key we choose for the GCW0!
                        input[curpos] = empty_char;
                        if curpos > 0 {
                            curpos -= 1
                        };
                    }
                }
            }

            #[cfg(not(feature = "arcade-input"))]
            {
                let key = self.getchar_raw();

                if key == SDLKey_SDLK_RETURN.try_into().unwrap() {
                    input[curpos] = 0;
                    finished = true;
                } else if key < SDLKey_SDLK_DELETE.try_into().unwrap()
                    && ((key as u8).is_ascii_graphic() || (key as u8).is_ascii_whitespace())
                    && curpos < max_len
                {
                    /* printable characters are entered in string */
                    input[curpos] = key.try_into().unwrap();
                    curpos += 1;
                } else if key == SDLKey_SDLK_BACKSPACE.try_into().unwrap() {
                    if curpos > 0 {
                        curpos -= 1
                    };
                    input[curpos] = b'.' as c_char;
                }
            }
        }

        info!(
            "GetString(..): The final string is: {}",
            CStr::from_ptr(input.as_ptr()).to_string_lossy()
        );

        Box::into_raw(input) as *mut c_char
    }

    /// Should do roughly what getchar() does, but in raw (SLD) keyboard mode.
    ///
    /// Return the (SDLKey) of the next key-pressed event cast to
    pub unsafe fn getchar_raw(&mut self) -> c_int {
        let mut event = SDL_Event::default();
        let mut return_key = 0;

        loop {
            SDL_WaitEvent(&mut event); /* wait for next event */

            match SDL_EventType::from(event.type_) {
                sdl_sys::SDL_EventType_SDL_KEYDOWN => {
                    /*
                     * here we use the fact that, I cite from SDL_keyboard.h:
                     * "The keyboard syms have been cleverly chosen to map to ASCII"
                     * ... I hope that this design feature is portable, and durable ;)
                     */
                    let key = event.key;
                    return_key = key.keysym.sym as c_int;
                    const SHIFT: SDLMod =
                        SDLMod_KMOD_LSHIFT as SDLMod | SDLMod_KMOD_RSHIFT as SDLMod;
                    if key.keysym.mod_ & SHIFT != 0 {
                        return_key = u8::try_from(key.keysym.sym as u32)
                            .unwrap()
                            .to_ascii_uppercase()
                            .into();
                    }
                }

                sdl_sys::SDL_EventType_SDL_JOYBUTTONDOWN => {
                    let jbutton = event.jbutton;
                    if jbutton.button == 0 {
                        return_key = PointerStates::JoyButton1 as c_int;
                    } else if jbutton.button == 1 {
                        return_key = PointerStates::JoyButton2 as c_int;
                    } else if jbutton.button == 2 {
                        return_key = PointerStates::JoyButton3 as c_int;
                    } else if jbutton.button == 3 {
                        return_key = PointerStates::JoyButton4 as c_int;
                    }
                }

                sdl_sys::SDL_EventType_SDL_JOYAXISMOTION => {
                    let jaxis = event.jaxis;
                    let axis = jaxis.axis;
                    if axis == 0 || ((self.input.joy_num_axes >= 5) && (axis == 3))
                    /* x-axis */
                    {
                        if self.input.joy_sensitivity * i32::from(jaxis.value) > 10000
                        /* about half tilted */
                        {
                            return_key = PointerStates::JoyRight as c_int;
                        } else if self.input.joy_sensitivity * i32::from(jaxis.value) < -10000 {
                            return_key = PointerStates::JoyLeft as c_int;
                        }
                    } else if (axis == 1) || ((self.input.joy_num_axes >= 5) && (axis == 4))
                    /* y-axis */
                    {
                        if self.input.joy_sensitivity * i32::from(jaxis.value) > 10000 {
                            return_key = PointerStates::JoyDown as c_int;
                        } else if self.input.joy_sensitivity * i32::from(jaxis.value) < -10000 {
                            return_key = PointerStates::JoyUp as c_int;
                        }
                    }
                }

                sdl_sys::SDL_EventType_SDL_MOUSEBUTTONDOWN => {
                    let button = event.button;
                    if button.button == SDL_BUTTON_LEFT as u8 {
                        return_key = PointerStates::MouseButton1 as c_int;
                    } else if button.button == SDL_BUTTON_RIGHT as u8 {
                        return_key = PointerStates::MouseButton2 as c_int;
                    } else if button.button == SDL_BUTTON_MIDDLE as u8 {
                        return_key = PointerStates::MouseButton3 as c_int;
                    } else if button.button == SDL_BUTTON_WHEELUP as u8 {
                        return_key = PointerStates::MouseWheelup as c_int;
                    } else if button.button == SDL_BUTTON_WHEELDOWN as u8 {
                        return_key = PointerStates::MouseWheeldown as c_int;
                    }
                }

                _ => {
                    SDL_PushEvent(&mut event); /* put this event back into the queue */
                    self.update_input(); /* and treat it the usual way */
                    continue;
                }
            }

            if return_key != 0 {
                break return_key;
            }
        }
    }

    /// Behaves similarly as gl_printf() of svgalib, using the BFont
    /// print function PrintString().
    ///
    ///  sets current position of MyCursor[XY],
    ///     if last char is '\n': to same x, next line y
    ///     to end of string otherwise
    ///
    /// Added functionality to PrintString() is:
    ///  o) passing -1 as coord uses previous x and next-line y for printing
    ///  o) Screen is updated immediatly after print, using SDL_flip()
    pub unsafe fn printf_sdl(
        &mut self,
        screen: *mut SDL_Surface,
        mut x: c_int,
        mut y: c_int,
        format_args: fmt::Arguments,
    ) {
        use std::io::Write;

        if x == -1 {
            x = self.text.my_cursor_x;
        } else {
            self.text.my_cursor_x = x;
        }

        if y == -1 {
            y = self.text.my_cursor_y;
        } else {
            self.text.my_cursor_y = y;
        }

        let mut cursor = Cursor::new(self.text.text_buffer.as_mut());
        cursor.write_fmt(format_args).unwrap();
        let cursor_pos = cursor.position();
        let text_buffer = &self.text.text_buffer[..usize::try_from(cursor_pos).unwrap()];
        let textlen: c_int = text_buffer
            .iter()
            .map(|&c| char_width(&*self.b_font.current_font, c))
            .sum();

        self.put_string(screen, x, y, text_buffer);
        let h = font_height(&*self.b_font.current_font) + 2;

        SDL_UpdateRect(
            screen,
            x,
            y,
            textlen.try_into().unwrap(),
            h.try_into().unwrap(),
        ); // update the relevant line

        if *text_buffer.last().unwrap() == b'\n' {
            self.text.my_cursor_x = x;
            self.text.my_cursor_y = (f64::from(y) + 1.1 * f64::from(h)) as c_int;
        } else {
            self.text.my_cursor_x += textlen;
            self.text.my_cursor_y = y;
        }
    }

    /// Prints *Text beginning at positions startx/starty,
    /// and respecting the text-borders set by clip_rect
    /// -> this includes clipping but also automatic line-breaks
    /// when end-of-line is reached
    ///
    /// if startx/y == -1, write at current position, given by MyCursorX/Y.
    /// if clip_rect==NULL, no clipping is performed
    ///
    /// NOTE: the previous clip-rectange is restored before the function returns!
    /// NOTE2: this function _does not_ update the screen
    ///
    /// Return TRUE if some characters where written inside the clip rectangle,
    /// FALSE if not (used by ScrollText to know if Text has been scrolled
    /// out of clip-rect completely)
    pub unsafe fn display_text(
        &mut self,
        text: *const c_char,
        startx: c_int,
        starty: c_int,
        mut clip: *const SDL_Rect,
    ) -> c_int {
        if text.is_null() {
            return false as c_int;
        }

        if startx != -1 {
            self.text.my_cursor_x = startx;
        }
        if starty != -1 {
            self.text.my_cursor_y = starty;
        }

        let mut store_clip = rect!(0, 0, 0, 0);
        let mut temp_clipping_rect;
        SDL_GetClipRect(self.graphics.ne_screen, &mut store_clip); /* store previous clip-rect */
        if !clip.is_null() {
            SDL_SetClipRect(self.graphics.ne_screen, clip);
        } else {
            temp_clipping_rect = rect!(0, 0, self.vars.screen_rect.w, self.vars.screen_rect.h);
            clip = &mut temp_clipping_rect;
        }

        let mut text = CStr::from_ptr(text).to_bytes();

        let clip = &*clip;
        while let Some((&first, rest)) = text.split_first() {
            if self.text.my_cursor_y >= c_int::from(clip.y) + c_int::from(clip.h) {
                break;
            }

            if first == b'\n' {
                self.text.my_cursor_x = clip.x.into();
                self.text.my_cursor_y +=
                    (f64::from(font_height(&*self.b_font.current_font)) * TEXT_STRETCH) as c_int;
            } else {
                self.display_char(first as c_uchar);
            }

            text = rest;
            if self.is_linebreak_needed(text, clip) {
                text = &text[1..];
                self.text.my_cursor_x = clip.x.into();
                self.text.my_cursor_y +=
                    (f64::from(font_height(&*self.b_font.current_font)) * TEXT_STRETCH) as c_int;
            }
        }

        SDL_SetClipRect(self.graphics.ne_screen, &store_clip); /* restore previous clip-rect */

        /*
         * ScrollText() wants to know if we still wrote something inside the
         * clip-rectangle, of if the Text has been scrolled out
         */
        if self.text.my_cursor_y < clip.y.into()
            || starty > c_int::from(clip.y) + c_int::from(clip.h)
        {
            false as c_int
        } else {
            true as c_int
        }
    }

    /// This function displays a char. It uses Menu_BFont now
    /// to do this.  MyCursorX is  updated to new position.
    pub unsafe fn display_char(&mut self, c: c_uchar) {
        // don't accept non-printable characters
        if !(c.is_ascii_graphic() || c.is_ascii_whitespace()) {
            panic!("Illegal char passed to DisplayChar(): {}", c);
        }

        self.put_char(
            self.graphics.ne_screen,
            self.text.my_cursor_x,
            self.text.my_cursor_y,
            c,
        );

        // After the char has been displayed, we must move the cursor to its
        // new position.  That depends of course on the char displayed.
        //
        self.text.my_cursor_x += char_width(&*self.b_font.current_font, c);
    }

    ///  This function checks if the next word still fits in this line
    ///  of text or if we need a linebreak:
    ///  returns TRUE if linebreak is needed, FALSE otherwise
    ///
    ///  NOTE: this function only does something if *textpos is pointing on a space,
    ///  i.e. a word-beginning, otherwise it just returns TRUE
    ///
    ///  rp: added argument clip, which contains the text-window we're writing in
    ///  (formerly known as "TextBorder")
    pub unsafe fn is_linebreak_needed(&self, textpos: &[u8], clip: &SDL_Rect) -> bool {
        // only relevant if we're at the beginning of a word
        let textpos = match textpos.split_first() {
            Some((&c, _)) if c != b' ' => return false,
            Some((_, rest)) => rest,
            None => textpos,
        };

        let mut needed_space = 0;
        let iter = textpos
            .iter()
            .copied()
            .take_while(|&c| c != b' ' && c != b'\n');
        for c in iter {
            let w = char_width(&*self.b_font.current_font, c);
            needed_space += w;
            if self.text.my_cursor_x + needed_space > c_int::from(clip.x) + c_int::from(clip.w) - w
            {
                return true;
            }
        }

        false
    }

    pub unsafe fn enemy_hit_by_bullet_text(&mut self, enemy: c_int) {
        let robot = &mut self.main.all_enemys[usize::try_from(enemy).unwrap()];

        if self.global.game_config.droid_talk == 0 {
            return;
        }

        robot.text_visible_time = 0.;
        match my_random(4) {
            0 => {
                robot.text_to_be_displayed =
                    cstr!("Unhandled exception fault.  Press ok to reboot.").as_ptr()
                        as *mut c_char;
            }
            1 => {
                robot.text_to_be_displayed =
                    cstr!("System fault. Please buy a newer version.").as_ptr() as *mut c_char;
            }
            2 => {
                robot.text_to_be_displayed =
                    cstr!("System error. Might be a virus.").as_ptr() as *mut c_char;
            }
            3 => {
                robot.text_to_be_displayed =
                    cstr!("System error. Pleae buy an upgrade from MS.").as_ptr() as *mut c_char;
            }
            4 => {
                robot.text_to_be_displayed =
                    cstr!("System error. Press any key to reboot.").as_ptr() as *mut c_char;
            }
            _ => unreachable!(),
        }
    }

    pub unsafe fn enemy_influ_collision_text(&mut self, enemy: c_int) {
        let robot = &mut self.main.all_enemys[usize::try_from(enemy).unwrap()];

        if self.global.game_config.droid_talk == 0 {
            return;
        }

        robot.text_visible_time = 0.;
        match my_random(1) {
            0 => {
                robot.text_to_be_displayed =
                    cstr!("Hey, I'm from MS! Walk outa my way!").as_ptr() as *mut c_char;
            }
            1 => {
                robot.text_to_be_displayed =
                    cstr!("Hey, I know the big MS boss! You better go.").as_ptr() as *mut c_char;
            }
            _ => unreachable!(),
        }
    }

    pub unsafe fn add_influ_burnt_text(&mut self) {
        if self.global.game_config.droid_talk == 0 {
            return;
        }

        self.vars.me.text_visible_time = 0.;

        match my_random(6) {
            0 => {
                self.vars.me.text_to_be_displayed =
                    cstr!("Aaarrgh, aah, that burnt me!").as_ptr() as *mut c_char
            }
            1 => {
                self.vars.me.text_to_be_displayed =
                    cstr!("Hell, that blast was hot!").as_ptr() as *mut c_char
            }
            2 => {
                self.vars.me.text_to_be_displayed =
                    cstr!("Ghaart, I hate to stain my chassis like that.").as_ptr() as *mut c_char
            }
            3 => {
                self.vars.me.text_to_be_displayed =
                    cstr!("Oh no!  I think I've burnt a cable!").as_ptr() as *mut c_char
            }
            4 => {
                self.vars.me.text_to_be_displayed =
                    cstr!("Oh no, my poor transfer connectors smolder!").as_ptr() as *mut c_char
            }
            5 => {
                self.vars.me.text_to_be_displayed =
                    cstr!("I hope that didn't melt any circuits!").as_ptr() as *mut c_char
            }
            6 => {
                self.vars.me.text_to_be_displayed =
                    cstr!("So that gives some more black scars on me ol' dented chassis!").as_ptr()
                        as *mut c_char
            }
            _ => unreachable!(),
        }
    }

    /// Scrolls a given text down inside the given rect
    ///
    /// returns 0 if end of text was scolled out, 1 if user pressed fire
    pub unsafe fn scroll_text(
        &mut self,
        text: *mut c_char,
        rect: &mut SDL_Rect,
        _seconds_minimum_duration: c_int,
    ) -> c_int {
        let mut insert_line: f32 = rect.y.into();
        let mut speed = 30; // in pixel / sec
        const MAX_SPEED: c_int = 150;
        let mut just_started = true;

        let background = SDL_DisplayFormat(self.graphics.ne_screen);

        self.wait_for_all_keys_released();
        let ret;
        loop {
            let mut prev_tick = SDL_GetTicks();
            SDL_UpperBlit(background, null_mut(), self.graphics.ne_screen, null_mut());
            if self.display_text(text, rect.x.into(), insert_line as c_int, rect) == 0 {
                ret = 0; /* Text has been scrolled outside SDL_Rect */
                break;
            }
            SDL_Flip(self.graphics.ne_screen);

            if self.global.game_config.hog_cpu != 0 {
                SDL_Delay(1);
            }

            if just_started {
                just_started = false;
                let now = SDL_GetTicks();
                let mut key;
                loop {
                    key = self.any_key_just_pressed();
                    if key == 0 && (SDL_GetTicks() - now < SHOW_WAIT as u32) {
                        SDL_Delay(1); // wait before starting auto-scroll
                    } else {
                        break;
                    }
                }

                if (key == self.input.key_cmds[Cmds::Fire as usize][0])
                    || (key == self.input.key_cmds[Cmds::Fire as usize][1])
                    || (key == self.input.key_cmds[Cmds::Fire as usize][2])
                {
                    trace!("in just_started: Fire registered");
                    ret = 1;
                    break;
                }
                prev_tick = SDL_GetTicks();
            }

            if self.fire_pressed_r() {
                trace!("outside just_started: Fire registered");
                ret = 1;
                break;
            }

            if self.up_pressed() || self.wheel_up_pressed() {
                speed -= 5;
                if speed < -MAX_SPEED {
                    speed = -MAX_SPEED;
                }
            }
            if self.down_pressed() || self.wheel_down_pressed() {
                speed += 5;
                if speed > MAX_SPEED {
                    speed = MAX_SPEED;
                }
            }

            insert_line -=
                (f64::from(SDL_GetTicks() - prev_tick) * f64::from(speed) / 1000.0) as f32;

            if insert_line > f32::from(rect.y) + f32::from(rect.h) {
                insert_line = f32::from(rect.y) + f32::from(rect.h);
                if speed < 0 {
                    speed = 0;
                }
            }
        }

        SDL_UpperBlit(background, null_mut(), self.graphics.ne_screen, null_mut());
        SDL_Flip(self.graphics.ne_screen);
        SDL_FreeSurface(background);

        ret
    }
}