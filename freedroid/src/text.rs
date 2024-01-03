use crate::{
    b_font::{char_width, font_height, BFont},
    defs::{Cmds, PointerStates, SHOW_WAIT, TEXT_STRETCH},
    global::Global,
    graphics::Graphics,
    input::Input,
    misc::my_random,
    structs::TextToBeDisplayed,
    vars::Vars,
    FontCellOwner, Sdl,
};

use cstr::cstr;
use log::{error, info, trace};
use sdl::{
    convert::i32_to_u8,
    event::{JoyButtonEventType, KeyboardEventType, MouseButtonEventType},
    rect, Event, Rect, Rgba, Surface,
};
#[cfg(not(feature = "arcade-input"))]
use sdl_sys::SDLKey_SDLK_DELETE;
use sdl_sys::{
    SDLKey_SDLK_BACKSPACE, SDLKey_SDLK_RETURN, SDL_BUTTON_LEFT, SDL_BUTTON_MIDDLE,
    SDL_BUTTON_RIGHT, SDL_BUTTON_WHEELDOWN, SDL_BUTTON_WHEELUP,
};
use std::{
    cell::Cell,
    ffi::CString,
    fmt,
    io::Cursor,
    ops::Not,
    os::raw::{c_int, c_uchar},
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

impl crate::Data<'_> {
    /// Reads a string of "`MaxLen`" from User-input, and echos it
    /// either to stdout or using graphics-text, depending on the
    /// parameter "echo":
    /// * echo=0    no echo
    /// * echo=1    print using printf
    /// * echo=2    print using graphics-text
    ///
    /// values of echo > 2 are ignored and treated like echo=0
    pub fn get_string(&mut self, max_len: c_int, echo: c_int) -> Option<CString> {
        let max_len: usize = max_len.try_into().unwrap();

        if echo == 1 {
            error!("GetString(): sorry, echo=1 currently not implemented!\n");
            return None;
        }

        let x0 = self.text.my_cursor_x;
        let y0 = self.text.my_cursor_y;
        let height = font_height(
            self.b_font
                .current_font
                .as_ref()
                .unwrap()
                .ro(&self.font_owner),
        );

        let mut store = Surface::create_rgb(
            self.vars.screen_rect.width().into(),
            height.try_into().unwrap(),
            self.graphics.vid_bpp.max(0).try_into().unwrap_or(u8::MAX),
            Rgba::default(),
        )
        .unwrap();
        let store_rect = Rect::new(
            x0.try_into().unwrap(),
            y0.try_into().unwrap(),
            self.vars.screen_rect.width(),
            height.try_into().unwrap(),
        );
        self.graphics
            .ne_screen
            .as_mut()
            .unwrap()
            .blit_from(&store_rect, &mut store);

        #[cfg(feature = "arcade-input")]
        let blink_time = 200; // For adjusting fast <->slow blink; in ms
        #[cfg(feature = "arcade-input")]
        let mut inputchar: c_int = 17; // initial char = A
        #[cfg(feature = "arcade-input")]
        let empty_char = b' '; //for "empty" input line / backspace etc...

        #[cfg(not(feature = "arcade-input"))]
        let empty_char = b'.'; //for "empty" input linue / backspace etc...

        let mut input = vec![empty_char; max_len + 5];
        input[max_len] = 0;

        let mut finished = false;
        let mut curpos = 0;

        while !finished {
            let mut tmp_rect = store_rect;
            let mut ne_screen = self.graphics.ne_screen.take().unwrap();
            store.blit_to(&mut ne_screen, &mut tmp_rect);
            let usable_input = {
                let end = input.iter().copied().position(|c| c == b'\0').unwrap();
                &input[..end]
            };
            self.put_string(&mut ne_screen, x0, y0, usable_input);
            assert!(ne_screen.flip());
            self.graphics.ne_screen = Some(ne_screen);

            #[cfg(feature = "arcade-input")]
            {
                if inputchar < 0 {
                    inputchar += i32::try_from(ARCADE_INPUT_CHARS.len()).unwrap();
                }
                if inputchar >= i32::try_from(ARCADE_INPUT_CHARS.len()).unwrap() {
                    inputchar -= i32::try_from(ARCADE_INPUT_CHARS.len()).unwrap();
                }
                let key = ARCADE_INPUT_CHARS[usize::try_from(inputchar).unwrap()];

                let frame_duration = self.sdl.ticks_ms() - self.text.last_frame_time;
                if frame_duration > blink_time / 2 {
                    input[curpos] = key.try_into().unwrap(); // We want to show the currently chosen character
                    if frame_duration > blink_time {
                        self.text.last_frame_time = self.sdl.ticks_ms();
                    } else {
                        input[curpos] = empty_char; // Hmm., how to get character widht? If using '.', or any fill character, we'd need to know
                    }

                    if self.key_is_pressed_r(SDLKey_SDLK_RETURN.try_into().unwrap()) {
                        // For GCW0, maybe we need a prompt to say [PRESS ENTER WHEN FINISHED], or any other key we may choose...
                        input[curpos] = 0; // The last char is currently shown but, not entered into the string...
                                           // 	  input[curpos] = key; // Not sure which one would be expected by most users; the last blinking char is input or not?
                        finished = true;
                    } else if self.up_pressed_r() {
                        /* Currently, the key will work ON RELEASE; we might change this to
                         * ON PRESS and add a counter / delay after which while holding, will
                         * scroll trough the chars */
                        inputchar += 1;
                    } else if self.down_pressed_r() {
                        inputchar -= 1;
                    } else if self.fire_pressed_r() {
                        // ADVANCE CURSOR
                        input[curpos] = key.try_into().unwrap(); // Needed in case character has just blinked out...
                        curpos += 1;
                    } else if self.left_pressed_r() {
                        inputchar -= 5;
                    } else if self.right_pressed_r() {
                        inputchar += 5;
                    } else if self.cmd_is_active_r(Cmds::Activate) {
                        // CAPITAL <-> small
                        if (17..=42).contains(&inputchar) {
                            inputchar = 44 + (inputchar - 17);
                        } else if (44..=69).contains(&inputchar) {
                            inputchar = 17 + (inputchar - 44);
                        }
                    } else if self.key_is_pressed_r(SDLKey_SDLK_BACKSPACE.try_into().unwrap()) {
                        // else if ... other functions to consider: SPACE
                        // Or any othe key we choose for the GCW0!
                        input[curpos] = empty_char;
                        curpos = curpos.saturating_sub(1);
                    }
                }
            }

            #[cfg(not(feature = "arcade-input"))]
            {
                let key = self.getchar_raw();

                #[allow(clippy::cast_sign_loss)]
                if key == SDLKey_SDLK_RETURN.try_into().unwrap() {
                    input[curpos] = 0;
                    finished = true;
                } else if key < SDLKey_SDLK_DELETE.try_into().unwrap()
                    && (u8::try_from(key).map_or(false, |key| key.is_ascii_graphic())
                        || (u8::try_from(key).map_or(false, |key| key.is_ascii_whitespace())))
                    && curpos < max_len
                {
                    /* printable characters are entered in string */
                    input[curpos] = key.try_into().unwrap();
                    curpos += 1;
                } else if key == SDLKey_SDLK_BACKSPACE.try_into().unwrap() {
                    curpos = curpos.saturating_sub(1);
                    input[curpos] = b'.';
                }
            }
        }

        let end_pos = input.iter().copied().position(|c| c == 0).unwrap();
        input.truncate(end_pos + 1);
        let input = CString::from_vec_with_nul(input).unwrap();

        info!(
            "GetString(..): The final string is: {}",
            input.to_string_lossy()
        );

        Some(input)
    }

    /// Should do roughly what getchar() does, but in raw (SLD) keyboard mode.
    ///
    /// Return the `SDLKey` of the next key-pressed event cast to
    pub fn getchar_raw(&mut self) -> c_int {
        let mut return_key = 0;

        loop {
            match self.sdl.wait_event().unwrap() {
                Event::Keyboard(event) if matches!(event.ty, KeyboardEventType::KeyDown) => {
                    /*
                     * here we use the fact that, I cite from SDL_keyboard.h:
                     * "The keyboard syms have been cleverly chosen to map to ASCII"
                     * ... I hope that this design feature is portable, and durable ;)
                     */
                    return_key = event.keysym.symbol as c_int;
                    if event.keysym.mod_.is_shift() {
                        return_key = u8::try_from(event.keysym.symbol as isize)
                            .unwrap()
                            .to_ascii_uppercase()
                            .into();
                    }
                }

                Event::JoyButton(event) if matches!(event.ty, JoyButtonEventType::Down) => {
                    if event.button == 0 {
                        return_key = PointerStates::JoyButton1 as c_int;
                    } else if event.button == 1 {
                        return_key = PointerStates::JoyButton2 as c_int;
                    } else if event.button == 2 {
                        return_key = PointerStates::JoyButton3 as c_int;
                    } else if event.button == 3 {
                        return_key = PointerStates::JoyButton4 as c_int;
                    }
                }

                Event::JoyAxis(event) => {
                    let axis = event.axis;
                    if axis == 0 || ((self.input.joy_num_axes >= 5) && (axis == 3))
                    /* x-axis */
                    {
                        if self.input.joy_sensitivity * i32::from(event.value) > 10000
                        /* about half tilted */
                        {
                            return_key = PointerStates::JoyRight as c_int;
                        } else if self.input.joy_sensitivity * i32::from(event.value) < -10000 {
                            return_key = PointerStates::JoyLeft as c_int;
                        }
                    } else if (axis == 1) || ((self.input.joy_num_axes >= 5) && (axis == 4))
                    /* y-axis */
                    {
                        if self.input.joy_sensitivity * i32::from(event.value) > 10000 {
                            return_key = PointerStates::JoyDown as c_int;
                        } else if self.input.joy_sensitivity * i32::from(event.value) < -10000 {
                            return_key = PointerStates::JoyUp as c_int;
                        }
                    }
                }

                Event::MouseButton(event) if matches!(event.ty, MouseButtonEventType::Down) => {
                    if event.button == i32_to_u8(SDL_BUTTON_LEFT) {
                        return_key = PointerStates::MouseButton1 as c_int;
                    } else if event.button == i32_to_u8(SDL_BUTTON_RIGHT) {
                        return_key = PointerStates::MouseButton2 as c_int;
                    } else if event.button == i32_to_u8(SDL_BUTTON_MIDDLE) {
                        return_key = PointerStates::MouseButton3 as c_int;
                    } else if event.button == i32_to_u8(SDL_BUTTON_WHEELUP) {
                        return_key = PointerStates::MouseWheelup as c_int;
                    } else if event.button == i32_to_u8(SDL_BUTTON_WHEELDOWN) {
                        return_key = PointerStates::MouseWheeldown as c_int;
                    }
                }

                event => {
                    if self.sdl.push_even(&event).not() {
                        error!("Unable to push SDL event back to queue");
                    }
                    self.update_input(); /* and treat it the usual way */
                    continue;
                }
            }

            if return_key != 0 {
                break return_key;
            }
        }
    }

    /// Behaves similarly as `gl_printf`() of svgalib, using the [`BFont`]
    /// print function [`print_string`].
    ///
    ///  sets current position of `MyCursor[XY]`,
    ///     if last char is '\n': to same x, next line y
    ///     to end of string otherwise
    ///
    /// Added functionality to [`print_string`] is:
    ///  o) passing -1 as coord uses previous x and next-line y for printing
    ///  o) Screen is updated immediatly after print, using `SDL_flip`()
    ///
    /// [`print_string`]: crate::Data::print_string
    #[inline]
    pub fn printf_sdl<const F: bool>(
        &mut self,
        screen: &mut sdl::surface::Generic<F>,
        x: c_int,
        y: c_int,
        format_args: fmt::Arguments,
    ) {
        Self::printf_sdl_static::<F>(
            &mut self.text,
            &self.b_font,
            &mut self.font_owner,
            screen,
            x,
            y,
            format_args,
        );
    }

    pub fn printf_sdl_static<const F: bool>(
        text: &mut Text,
        b_font: &BFont,
        font_owner: &mut FontCellOwner,
        screen: &mut sdl::surface::Generic<F>,
        mut x: c_int,
        mut y: c_int,
        format_args: fmt::Arguments,
    ) {
        use std::io::Write;

        if x == -1 {
            x = text.my_cursor_x;
        } else {
            text.my_cursor_x = x;
        }

        if y == -1 {
            y = text.my_cursor_y;
        } else {
            text.my_cursor_y = y;
        }

        let mut cursor = Cursor::new(text.text_buffer.as_mut());
        cursor.write_fmt(format_args).unwrap();
        let cursor_pos = cursor.position();
        let text_buffer = &text.text_buffer[..usize::try_from(cursor_pos).unwrap()];
        let textlen: c_int = text_buffer
            .iter()
            .map(|&c| char_width(b_font.current_font.as_ref().unwrap().ro(font_owner), c))
            .sum();

        Self::put_string_static(b_font, font_owner, screen, x, y, text_buffer);
        let h = font_height(b_font.current_font.as_ref().unwrap().ro(font_owner)) + 2;

        // update the relevant line
        screen.update_rect(&Rect::new(
            x.try_into().unwrap(),
            y.try_into().unwrap(),
            textlen.try_into().unwrap(),
            h.try_into().unwrap(),
        ));

        #[allow(clippy::cast_possible_truncation)]
        if *text_buffer.last().unwrap() == b'\n' {
            text.my_cursor_x = x;
            text.my_cursor_y = (f64::from(y) + 1.1 * f64::from(h)) as c_int;
        } else {
            text.my_cursor_x += textlen;
            text.my_cursor_y = y;
        }
    }

    /// Prints *Text beginning at positions startx/starty,
    /// and respecting the text-borders set by `clip_rect`
    /// -> this includes clipping but also automatic line-breaks
    /// when end-of-line is reached
    ///
    /// if startx/y == -1, write at current position, given by MyCursorX/Y.
    /// if `clip_rect==NULL`, no clipping is performed
    ///
    /// NOTE: the previous clip-rectange is restored before the function returns!
    /// NOTE2: this function _does not_ update the screen
    ///
    /// Return TRUE if some characters where written inside the clip rectangle,
    /// FALSE if not (used by `ScrollText` to know if Text has been scrolled
    /// out of clip-rect completely)
    pub fn display_text(
        &mut self,
        text: &[u8],
        start_x: c_int,
        start_y: c_int,
        clip: Option<Rect>,
    ) -> c_int {
        let Self {
            text: data_text,
            graphics,
            vars,
            b_font,
            font_owner,
            ..
        } = self;

        Displayer {
            data_text,
            graphics,
            vars,
            b_font,
            font_owner,
            text,
            start_x,
            start_y,
            clip,
        }
        .run()
    }

    /// This function displays a char. It uses `Menu_BFont` now
    /// to do this.  `MyCursorX` is  updated to new position.
    pub fn display_char(
        graphics: &mut Graphics,
        text: &mut Text,
        b_font: &BFont,
        font_owner: &mut FontCellOwner,
        c: c_uchar,
    ) {
        // don't accept non-printable characters
        assert!(
            c.is_ascii_graphic() || c.is_ascii_whitespace(),
            "Illegal char passed to DisplayChar(): {c}"
        );

        let mut ne_screen = graphics.ne_screen.take().unwrap();
        Self::put_char(
            b_font,
            font_owner,
            &mut ne_screen,
            text.my_cursor_x,
            text.my_cursor_y,
            c,
        );
        graphics.ne_screen = Some(ne_screen);

        // After the char has been displayed, we must move the cursor to its
        // new position.  That depends of course on the char displayed.
        //
        text.my_cursor_x += char_width(b_font.current_font.as_ref().unwrap().ro(font_owner), c);
    }

    ///  This function checks if the next word still fits in this line
    ///  of text or if we need a linebreak:
    ///  returns TRUE if linebreak is needed, FALSE otherwise
    ///
    ///  NOTE: this function only does something if *textpos is pointing on a space,
    ///  i.e. a word-beginning, otherwise it just returns TRUE
    ///
    ///  rp: added argument clip, which contains the text-window we're writing in
    ///  (formerly known as `TextBorder`)
    pub fn is_linebreak_needed(
        b_font: &BFont,
        font_owner: &FontCellOwner,
        text: &Text,
        textpos: &[u8],
        clip: Rect,
    ) -> bool {
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
            let w = char_width(b_font.current_font.as_ref().unwrap().ro(font_owner), c);
            needed_space += w;
            if text.my_cursor_x + needed_space
                > c_int::from(clip.x()) + c_int::from(clip.width()) - w
            {
                return true;
            }
        }

        false
    }

    pub fn enemy_hit_by_bullet_text(&mut self, enemy: c_int) {
        let robot = &mut self.main.all_enemys[usize::try_from(enemy).unwrap()];

        if self.global.game_config.droid_talk == 0 {
            return;
        }

        robot.text_visible_time = 0.;
        let text = match my_random(4) {
            0 => "Unhandled exception fault.  Press ok to reboot.",
            1 => "System fault. Please buy a newer version.",
            2 => "System error. Might be a virus.",
            3 => "System error. Pleae buy an upgrade from MS.",
            4 => "System error. Press any key to reboot.",
            _ => unreachable!(),
        };
        robot.text_to_be_displayed = text;
    }

    pub fn enemy_influ_collision_text(&mut self, enemy: c_int) {
        let robot = &mut self.main.all_enemys[usize::try_from(enemy).unwrap()];

        if self.global.game_config.droid_talk == 0 {
            return;
        }

        robot.text_visible_time = 0.;
        let text = match my_random(1) {
            0 => "Hey, I'm from MS! Walk outa my way!",
            1 => "Hey, I know the big MS boss! You better go.",
            _ => unreachable!(),
        };
        robot.text_to_be_displayed = text;
    }

    pub fn add_influ_burnt_text(&mut self) {
        if self.global.game_config.droid_talk == 0 {
            return;
        }

        self.vars.me.text_visible_time = 0.;

        let new_text = match my_random(6) {
            0 => {
                cstr!("Aaarrgh, aah, that burnt me!")
            }
            1 => {
                cstr!("Hell, that blast was hot!")
            }
            2 => {
                cstr!("Ghaart, I hate to stain my chassis like that.")
            }
            3 => {
                cstr!("Oh no!  I think I've burnt a cable!")
            }
            4 => {
                cstr!("Oh no, my poor transfer connectors smolder!")
            }
            5 => {
                cstr!("I hope that didn't melt any circuits!")
            }
            6 => {
                cstr!("So that gives some more black scars on me ol' dented chassis!")
            }
            _ => unreachable!(),
        };
        self.vars.me.text_to_be_displayed = TextToBeDisplayed::String(new_text);
    }

    /// Scrolls a given text down inside the given rect
    ///
    /// returns 0 if end of text was scolled out, 1 if user pressed fire
    pub fn scroll_text(&mut self, text: &[u8], rect: &mut Rect) -> c_int {
        let Self {
            sdl,
            b_font,
            text: data_text,
            input,
            global,
            vars,
            graphics,
            quit,
            font_owner,
            ..
        } = self;

        Scroll {
            graphics,
            input,
            sdl,
            vars,
            global,
            data_text,
            b_font,
            font_owner,
            quit,
            text,
            rect,
        }
        .run()
    }
}

pub struct Scroll<'a, 'sdl: 'a> {
    pub graphics: &'a mut Graphics<'sdl>,
    pub input: &'a mut Input,
    pub sdl: &'sdl Sdl,
    pub vars: &'a Vars<'sdl>,
    pub global: &'a Global<'sdl>,
    pub data_text: &'a mut Text,
    pub b_font: &'a BFont<'sdl>,
    pub font_owner: &'a mut FontCellOwner,
    pub quit: &'a Cell<bool>,
    pub text: &'a [u8],
    pub rect: &'a mut Rect,
}

impl Scroll<'_, '_> {
    pub fn run(self) -> c_int {
        const MAX_SPEED: c_int = 150;

        let Self {
            graphics,
            input,
            sdl,
            vars,
            global,
            data_text,
            b_font,
            font_owner,
            quit,
            text,
            rect,
        } = self;

        let mut insert_line: f32 = rect.y().into();
        let mut speed = 30; // in pixel / sec
        let mut just_started = true;

        let mut background = graphics
            .ne_screen
            .as_mut()
            .unwrap()
            .display_format()
            .unwrap();

        crate::Data::wait_for_all_keys_released_static(
            input,
            sdl,
            #[cfg(not(target_os = "android"))]
            vars,
            #[cfg(not(target_os = "android"))]
            quit,
            #[cfg(target_os = "android")]
            graphics,
        );
        let ret_val;
        loop {
            let mut prev_tick = sdl.ticks_ms();
            background.blit(graphics.ne_screen.as_mut().unwrap());
            if (Displayer {
                data_text,
                graphics,
                vars,
                b_font,
                font_owner,
                text,
                start_x: rect.x().into(),
                #[allow(clippy::cast_possible_truncation)]
                start_y: insert_line as c_int,
                clip: Some(*rect),
            }
            .run())
                == 0
            {
                ret_val = 0; /* Text has been scrolled outside Rect */
                break;
            }
            assert!(graphics.ne_screen.as_mut().unwrap().flip());

            if global.game_config.hog_cpu != 0 {
                sdl.delay_ms(1);
            }

            if just_started {
                just_started = false;
                let now = sdl.ticks_ms();
                let mut key;
                loop {
                    key = crate::Data::any_key_just_pressed_static(
                        sdl,
                        input,
                        vars,
                        quit,
                        #[cfg(target_os = "android")]
                        graphics,
                    );
                    if key == 0 && (sdl.ticks_ms() - now < SHOW_WAIT) {
                        sdl.delay_ms(1); // wait before starting auto-scroll
                    } else {
                        break;
                    }
                }

                if (key == input.key_cmds[Cmds::Fire as usize][0])
                    || (key == input.key_cmds[Cmds::Fire as usize][1])
                    || (key == input.key_cmds[Cmds::Fire as usize][2])
                {
                    trace!("in just_started: Fire registered");
                    ret_val = 1;
                    break;
                }
                prev_tick = sdl.ticks_ms();
            }

            if crate::Data::fire_pressed_r_static(sdl, input, vars, quit) {
                trace!("outside just_started: Fire registered");
                ret_val = 1;
                break;
            }

            if quit.get() {
                return 1;
            }

            if crate::Data::up_pressed_static(sdl, input, vars, quit)
                || crate::Data::wheel_up_pressed_static(sdl, input, vars, quit)
            {
                speed -= 5;
                if speed < -MAX_SPEED {
                    speed = -MAX_SPEED;
                }
            }
            if crate::Data::down_pressed_static(sdl, input, vars, quit)
                || crate::Data::wheel_down_pressed_static(sdl, input, vars, quit)
            {
                speed += 5;
                if speed > MAX_SPEED {
                    speed = MAX_SPEED;
                }
            }

            #[allow(clippy::cast_possible_truncation)]
            {
                insert_line -=
                    (f64::from(sdl.ticks_ms() - prev_tick) * f64::from(speed) / 1000.0) as f32;
            }

            if insert_line > f32::from(rect.y()) + f32::from(rect.height()) {
                insert_line = f32::from(rect.y()) + f32::from(rect.height());
                if speed < 0 {
                    speed = 0;
                }
            }
        }

        background.blit(graphics.ne_screen.as_mut().unwrap());
        assert!(graphics.ne_screen.as_mut().unwrap().flip());

        ret_val
    }
}

pub struct Displayer<'a, 'sdl: 'a> {
    pub data_text: &'a mut Text,
    pub graphics: &'a mut Graphics<'sdl>,
    pub vars: &'a Vars<'sdl>,
    pub b_font: &'a BFont<'sdl>,
    pub font_owner: &'a mut FontCellOwner,
    pub text: &'a [u8],
    pub start_x: c_int,
    pub start_y: c_int,
    pub clip: Option<Rect>,
}

impl Displayer<'_, '_> {
    pub fn run(self) -> c_int {
        let Self {
            data_text,
            graphics,
            vars,
            b_font,
            font_owner,
            mut text,
            start_x,
            start_y,
            clip,
        } = self;

        if start_x != -1 {
            data_text.my_cursor_x = start_x;
        }
        if start_y != -1 {
            data_text.my_cursor_y = start_y;
        }

        // store previous clip-rect
        let store_clip = graphics.ne_screen.as_ref().unwrap().get_clip_rect();
        let clip = match clip {
            Some(clip) => {
                graphics
                    .ne_screen
                    .as_mut()
                    .unwrap()
                    .set_clip_rect(rect::Ref::from(&clip));

                clip
            }
            None => Rect::new(0, 0, vars.screen_rect.width(), vars.screen_rect.height()),
        };

        while let Some((&first, rest)) = text.split_first() {
            if data_text.my_cursor_y >= c_int::from(clip.y()) + c_int::from(clip.height()) {
                break;
            }

            #[allow(clippy::cast_possible_truncation)]
            if first == b'\n' {
                data_text.my_cursor_x = clip.x().into();
                data_text.my_cursor_y += (f64::from(font_height(
                    b_font.current_font.as_ref().unwrap().ro(font_owner),
                )) * TEXT_STRETCH) as c_int;
            } else {
                crate::Data::display_char(
                    graphics,
                    data_text,
                    b_font,
                    font_owner,
                    first as c_uchar,
                );
            }

            text = rest;
            #[allow(clippy::cast_possible_truncation)]
            if crate::Data::is_linebreak_needed(b_font, font_owner, data_text, text, clip) {
                text = &text[1..];
                data_text.my_cursor_x = clip.x().into();
                data_text.my_cursor_y += (f64::from(font_height(
                    b_font.current_font.as_ref().unwrap().ro(font_owner),
                )) * TEXT_STRETCH) as c_int;
            }
        }

        graphics
            .ne_screen
            .as_mut()
            .unwrap()
            .set_clip_rect(&store_clip); /* restore previous clip-rect */

        /*
         * ScrollText() wants to know if we still wrote something inside the
         * clip-rectangle, of if the Text has been scrolled out
         */
        if data_text.my_cursor_y < clip.y().into()
            || start_y > c_int::from(clip.y()) + c_int::from(clip.height())
        {
            i32::from(false)
        } else {
            i32::from(true)
        }
    }
}
