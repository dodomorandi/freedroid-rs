use crate::{
    b_font::{CharWidth, FontHeight, GetCurrentFont, PutChar, PutString},
    defs::{
        self, Cmds, DownPressed, FirePressedR, PointerStates, UpPressed, SHOW_WAIT, TEXT_STRETCH,
    },
    global::{AllEnemys, GameConfig, Me, Screen_Rect},
    graphics::{ne_screen, vid_bpp},
    input::{
        any_key_just_pressed, joy_num_axes, joy_sensitivity, key_cmds, update_input,
        wait_for_all_keys_released, SDL_Delay, WheelDownPressed, WheelUpPressed,
    },
    misc::{MyRandom, Terminate},
};

#[cfg(feature = "arcade-input")]
use crate::{
    defs::{DownPressedR, LeftPressedR, RightPressedR, UpPressedR},
    input::{cmd_is_activeR, KeyIsPressedR},
};

use cstr::cstr;
use log::{error, info, trace};
#[cfg(not(feature = "arcade-input"))]
use sdl::keysym::SDLK_DELETE;
use sdl::{
    event::{
        ll::{
            SDLMod, SDL_Event, SDL_WaitEvent, SDL_JOYAXISMOTION, SDL_JOYBUTTONDOWN, SDL_KEYDOWN,
            SDL_MOUSEBUTTONDOWN,
        },
        Mod, MouseState,
    },
    keysym::{SDLK_BACKSPACE, SDLK_RETURN},
    sdl::{ll::SDL_GetTicks, Rect},
    video::ll::{
        SDL_CreateRGBSurface, SDL_DisplayFormat, SDL_Flip, SDL_FreeSurface, SDL_Rect, SDL_Surface,
        SDL_UpdateRect, SDL_UpperBlit,
    },
};
use std::{
    convert::{TryFrom, TryInto},
    ffi::{CStr, VaList},
    os::raw::{c_char, c_int, c_uchar},
    ptr::null_mut,
};

extern "C" {
    fn SDL_PushEvent(event: *mut SDL_Event) -> c_int;
    fn SDL_GetClipRect(surface: *mut SDL_Surface, rect: *mut SDL_Rect);
    fn SDL_SetClipRect(surface: *mut SDL_Surface, rect: *const SDL_Rect) -> bool;
    fn vsprintf(str: *mut c_char, format: *const c_char, ap: VaList) -> c_int;
}

#[cfg(feature = "arcade-input")]
const ARCADE_INPUT_CHARS: [c_int; 70] = [
    48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 61, 42, 43, 44, 45, 46, 47, 65, 66, 67, 68, 69, 70, 71,
    72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 32, 97, 98, 99,
    100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118,
    119, 120, 121, 122,
];

#[no_mangle]
static mut MyCursorX: c_int = 0;
#[no_mangle]
static mut MyCursorY: c_int = 0;
#[no_mangle]
static mut TextBuffer: [c_char; 10000] = [0; 10000];

/// Reads a string of "MaxLen" from User-input, and echos it
/// either to stdout or using graphics-text, depending on the
/// parameter "echo":
/// * echo=0    no echo
/// * echo=1    print using printf
/// * echo=2    print using graphics-text
///
/// values of echo > 2 are ignored and treated like echo=0
#[no_mangle]
pub unsafe extern "C" fn GetString(max_len: c_int, echo: c_int) -> *mut c_char {
    let max_len: usize = max_len.try_into().unwrap();

    if echo == 1 {
        error!("GetString(): sorry, echo=1 currently not implemented!\n");
        return null_mut();
    }

    let x0 = MyCursorX;
    let y0 = MyCursorY;
    let height = FontHeight(&*GetCurrentFont());

    let store = SDL_CreateRGBSurface(0, Screen_Rect.w.into(), height, vid_bpp, 0, 0, 0, 0);
    let mut store_rect = Rect::new(
        x0.try_into().unwrap(),
        y0.try_into().unwrap(),
        Screen_Rect.w,
        height.try_into().unwrap(),
    );
    SDL_UpperBlit(ne_screen, &mut store_rect, store, null_mut());

    #[cfg(feature = "arcade-input")]
    let blink_time = 200; // For adjusting fast <->slow blink; in ms
    #[cfg(feature = "arcade-input")]
    static mut LAST_FRAME_TIME: u32 = 0; //  = SDL_GetTicks();
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
        SDL_UpperBlit(store, null_mut(), ne_screen, &mut tmp_rect);
        PutString(ne_screen, x0, y0, input.as_mut_ptr());
        SDL_Flip(ne_screen);

        #[cfg(feature = "arcade-input")]
        {
            if inputchar < 0 {
                inputchar += ARCADE_INPUT_CHARS.len() as i32;
            }
            if inputchar >= ARCADE_INPUT_CHARS.len() as i32 {
                inputchar -= ARCADE_INPUT_CHARS.len() as i32;
            }
            let key = ARCADE_INPUT_CHARS[usize::try_from(inputchar).unwrap()];

            let frame_duration = SDL_GetTicks() - LAST_FRAME_TIME;
            if frame_duration > blink_time / 2 {
                input[curpos] = key.try_into().unwrap(); // We want to show the currently chosen character
                if frame_duration > blink_time {
                    LAST_FRAME_TIME = SDL_GetTicks();
                } else {
                    input[curpos] = empty_char; // Hmm., how to get character widht? If using '.', or any fill character, we'd need to know
                }

                if KeyIsPressedR(SDLK_RETURN.try_into().unwrap()) {
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
                } else if KeyIsPressedR(SDLK_BACKSPACE.try_into().unwrap()) {
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
            let key = getchar_raw();

            if key == SDLK_RETURN.try_into().unwrap() {
                input[curpos] = 0;
                finished = true;
            } else if key < SDLK_DELETE.try_into().unwrap()
                && ((key as u8).is_ascii_graphic() || (key as u8).is_ascii_whitespace())
                && curpos < max_len
            {
                /* printable characters are entered in string */
                input[curpos] = key.try_into().unwrap();
                curpos += 1;
            } else if key == SDLK_BACKSPACE.try_into().unwrap() {
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
#[no_mangle]
pub unsafe extern "C" fn getchar_raw() -> c_int {
    let mut event = SDL_Event {
        data: Default::default(),
    };
    let mut return_key = 0;

    loop {
        SDL_WaitEvent(&mut event); /* wait for next event */

        match u32::from(*event._type()) {
            SDL_KEYDOWN => {
                /*
                 * here we use the fact that, I cite from SDL_keyboard.h:
                 * "The keyboard syms have been cleverly chosen to map to ASCII"
                 * ... I hope that this design feature is portable, and durable ;)
                 */
                let key = &*event.key();
                return_key = key.keysym.sym as c_int;
                const SHIFT: SDLMod = Mod::LShift as SDLMod | Mod::RShift as SDLMod;
                if key.keysym._mod & SHIFT != 0 {
                    return_key = u8::try_from(key.keysym.sym as u32)
                        .unwrap()
                        .to_ascii_uppercase()
                        .into();
                }
            }

            SDL_JOYBUTTONDOWN => {
                let jbutton = &*event.jbutton();
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

            SDL_JOYAXISMOTION => {
                let jaxis = &*event.jaxis();
                let axis = jaxis.axis;
                if axis == 0 || ((joy_num_axes >= 5) && (axis == 3))
                /* x-axis */
                {
                    if joy_sensitivity * i32::from(jaxis.value) > 10000
                    /* about half tilted */
                    {
                        return_key = PointerStates::JoyRight as c_int;
                    } else if joy_sensitivity * i32::from(jaxis.value) < -10000 {
                        return_key = PointerStates::JoyLeft as c_int;
                    }
                } else if (axis == 1) || ((joy_num_axes >= 5) && (axis == 4))
                /* y-axis */
                {
                    if joy_sensitivity * i32::from(jaxis.value) > 10000 {
                        return_key = PointerStates::JoyDown as c_int;
                    } else if joy_sensitivity * i32::from(jaxis.value) < -10000 {
                        return_key = PointerStates::JoyUp as c_int;
                    }
                }
            }

            SDL_MOUSEBUTTONDOWN => {
                let button = &*event.button();
                if button.button == MouseState::Left as u8 {
                    return_key = PointerStates::MouseButton1 as c_int;
                } else if button.button == MouseState::Right as u8 {
                    return_key = PointerStates::MouseButton2 as c_int;
                } else if button.button == MouseState::Middle as u8 {
                    return_key = PointerStates::MouseButton3 as c_int;
                } else if button.button == MouseState::WheelUp as u8 {
                    return_key = PointerStates::MouseWheelup as c_int;
                } else if button.button == MouseState::WheelDown as u8 {
                    return_key = PointerStates::MouseWheeldown as c_int;
                }
            }

            _ => {
                SDL_PushEvent(&mut event); /* put this event back into the queue */
                update_input(); /* and treat it the usual way */
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
#[no_mangle]
pub unsafe extern "C" fn printf_SDL(
    screen: *mut SDL_Surface,
    mut x: c_int,
    mut y: c_int,
    fmt: *mut c_char,
    args: ...
) {
    let mut args = args.clone();
    if x == -1 {
        x = MyCursorX;
    } else {
        MyCursorX = x;
    }

    if y == -1 {
        y = MyCursorY;
    } else {
        MyCursorY = y;
    }

    assert!(vsprintf(TextBuffer.as_mut_ptr(), fmt, args.as_va_list()) >= 0);
    let text_buffer = CStr::from_ptr(TextBuffer.as_mut_ptr()).to_bytes();
    let textlen: c_int = text_buffer
        .iter()
        .map(|&c| CharWidth(&*GetCurrentFont(), c.into()))
        .sum();

    PutString(screen, x, y, TextBuffer.as_mut_ptr());
    let h = FontHeight(&*GetCurrentFont()) + 2;

    SDL_UpdateRect(
        screen,
        x,
        y,
        textlen.try_into().unwrap(),
        h.try_into().unwrap(),
    ); // update the relevant line

    if *text_buffer.last().unwrap() == b'\n' {
        MyCursorX = x;
        MyCursorY = (f64::from(y) + 1.1 * f64::from(h)) as c_int;
    } else {
        MyCursorX += textlen;
        MyCursorY = y;
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
#[no_mangle]
pub unsafe extern "C" fn DisplayText(
    text: *const c_char,
    startx: c_int,
    starty: c_int,
    mut clip: *const SDL_Rect,
) -> c_int {
    if text.is_null() {
        return false as c_int;
    }

    if startx != -1 {
        MyCursorX = startx;
    }
    if starty != -1 {
        MyCursorY = starty;
    }

    let mut store_clip = Rect::new(0, 0, 0, 0);
    let mut temp_clipping_rect;
    SDL_GetClipRect(ne_screen, &mut store_clip); /* store previous clip-rect */
    if !clip.is_null() {
        SDL_SetClipRect(ne_screen, clip);
    } else {
        temp_clipping_rect = Rect::new(0, 0, Screen_Rect.w, Screen_Rect.h);
        clip = &mut temp_clipping_rect;
    }

    let mut text = CStr::from_ptr(text).to_bytes();

    let clip = &*clip;
    while let Some((&first, rest)) = text.split_first() {
        if MyCursorY >= c_int::from(clip.y) + c_int::from(clip.h) {
            break;
        }

        if first == b'\n' {
            MyCursorX = clip.x.into();
            MyCursorY += (f64::from(FontHeight(&*GetCurrentFont())) * TEXT_STRETCH) as c_int;
        } else {
            DisplayChar(first as c_uchar);
        }

        text = rest;
        if is_linebreak_needed(text, clip) {
            text = &text[1..];
            MyCursorX = clip.x.into();
            MyCursorY += (f64::from(FontHeight(&*GetCurrentFont())) * TEXT_STRETCH) as c_int;
        }
    }

    SDL_SetClipRect(ne_screen, &store_clip); /* restore previous clip-rect */

    /*
     * ScrollText() wants to know if we still wrote something inside the
     * clip-rectangle, of if the Text has been scrolled out
     */
    if MyCursorY < clip.y.into() || starty > c_int::from(clip.y) + c_int::from(clip.h) {
        false as c_int
    } else {
        true as c_int
    }
}

/// This function displays a char. It uses Menu_BFont now
/// to do this.  MyCursorX is  updated to new position.
#[no_mangle]
pub unsafe extern "C" fn DisplayChar(c: c_uchar) {
    // don't accept non-printable characters
    if !(c.is_ascii_graphic() || c.is_ascii_whitespace()) {
        println!("Illegal char passed to DisplayChar(): {}", c);
        Terminate(defs::ERR.into());
    }

    PutChar(ne_screen, MyCursorX, MyCursorY, c.into());

    // After the char has been displayed, we must move the cursor to its
    // new position.  That depends of course on the char displayed.
    //
    MyCursorX += CharWidth(&*GetCurrentFont(), c.into());
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
pub unsafe fn is_linebreak_needed(textpos: &[u8], clip: &Rect) -> bool {
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
        let w = CharWidth(&*GetCurrentFont(), c.into());
        needed_space += w;
        if MyCursorX + needed_space > c_int::from(clip.x) + c_int::from(clip.w) - w {
            return true;
        }
    }

    false
}

#[no_mangle]
pub unsafe extern "C" fn EnemyHitByBulletText(enemy: c_int) {
    let robot = &mut AllEnemys[usize::try_from(enemy).unwrap()];

    if GameConfig.Droid_Talk == 0 {
        return;
    }

    robot.TextVisibleTime = 0.;
    match MyRandom(4) {
        0 => {
            robot.TextToBeDisplayed =
                cstr!("Unhandled exception fault.  Press ok to reboot.").as_ptr() as *mut c_char;
        }
        1 => {
            robot.TextToBeDisplayed =
                cstr!("System fault. Please buy a newer version.").as_ptr() as *mut c_char;
        }
        2 => {
            robot.TextToBeDisplayed =
                cstr!("System error. Might be a virus.").as_ptr() as *mut c_char;
        }
        3 => {
            robot.TextToBeDisplayed =
                cstr!("System error. Pleae buy an upgrade from MS.").as_ptr() as *mut c_char;
        }
        4 => {
            robot.TextToBeDisplayed =
                cstr!("System error. Press any key to reboot.").as_ptr() as *mut c_char;
        }
        _ => unreachable!(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn EnemyInfluCollisionText(enemy: c_int) {
    let robot = &mut AllEnemys[usize::try_from(enemy).unwrap()];

    if GameConfig.Droid_Talk == 0 {
        return;
    }

    robot.TextVisibleTime = 0.;
    match MyRandom(1) {
        0 => {
            robot.TextToBeDisplayed =
                cstr!("Hey, I'm from MS! Walk outa my way!").as_ptr() as *mut c_char;
        }
        1 => {
            robot.TextToBeDisplayed =
                cstr!("Hey, I know the big MS boss! You better go.").as_ptr() as *mut c_char;
        }
        _ => unreachable!(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn AddStandingAndAimingText(enemy: c_int) {
    let robot = &mut AllEnemys[usize::try_from(enemy).unwrap()];

    if GameConfig.Droid_Talk == 0 {
        return;
    }

    robot.TextVisibleTime = 0.;

    if Me.speed.x.abs() < 1. && Me.speed.y.abs() < 1. {
        robot.TextToBeDisplayed = cstr!("Yeah, stay like that, haha.").as_ptr() as *mut c_char;
    } else {
        robot.TextToBeDisplayed = cstr!("Stand still while I aim at you.").as_ptr() as *mut c_char;
    }
}

#[no_mangle]
pub unsafe extern "C" fn AddInfluBurntText() {
    if GameConfig.Droid_Talk == 0 {
        return;
    }

    Me.TextVisibleTime = 0.;

    match MyRandom(6) {
        0 => Me.TextToBeDisplayed = cstr!("Aaarrgh, aah, that burnt me!").as_ptr() as *mut c_char,
        1 => Me.TextToBeDisplayed = cstr!("Hell, that blast was hot!").as_ptr() as *mut c_char,
        2 => {
            Me.TextToBeDisplayed =
                cstr!("Ghaart, I hate to stain my chassis like that.").as_ptr() as *mut c_char
        }
        3 => {
            Me.TextToBeDisplayed =
                cstr!("Oh no!  I think I've burnt a cable!").as_ptr() as *mut c_char
        }
        4 => {
            Me.TextToBeDisplayed =
                cstr!("Oh no, my poor transfer connectors smolder!").as_ptr() as *mut c_char
        }
        5 => {
            Me.TextToBeDisplayed =
                cstr!("I hope that didn't melt any circuits!").as_ptr() as *mut c_char
        }
        6 => {
            Me.TextToBeDisplayed =
                cstr!("So that gives some more black scars on me ol' dented chassis!").as_ptr()
                    as *mut c_char
        }
        _ => unreachable!(),
    }
}

/// Similar to putchar(), using SDL via the BFont-fct PutChar().
///
/// sets MyCursor[XY], and allows passing (-1,-1) as coords to indicate
/// using the current cursor position.
#[no_mangle]
pub unsafe extern "C" fn putchar_SDL(
    surface: *mut SDL_Surface,
    mut x: c_int,
    mut y: c_int,
    c: c_int,
) -> c_int {
    if x == -1 {
        x = MyCursorX;
    }
    if y == -1 {
        y = MyCursorY;
    }

    MyCursorX = x + CharWidth(&*GetCurrentFont(), c);
    MyCursorY = y;

    let ret = PutChar(surface, x, y, c);

    SDL_Flip(surface);

    ret
}

/// Scrolls a given text down inside the given rect
///
/// returns 0 if end of text was scolled out, 1 if user pressed fire
#[no_mangle]
pub unsafe extern "C" fn ScrollText(
    text: *mut c_char,
    rect: &mut SDL_Rect,
    _seconds_minimum_duration: c_int,
) -> c_int {
    let mut insert_line: f32 = rect.y.into();
    let mut speed = 30; // in pixel / sec
    const MAX_SPEED: c_int = 150;
    let mut just_started = true;

    let background = SDL_DisplayFormat(ne_screen);

    wait_for_all_keys_released();
    let ret;
    loop {
        let mut prev_tick = SDL_GetTicks();
        SDL_UpperBlit(background, null_mut(), ne_screen, null_mut());
        if DisplayText(text, rect.x.into(), insert_line as c_int, rect) == 0 {
            ret = 0; /* Text has been scrolled outside Rect */
            break;
        }
        SDL_Flip(ne_screen);

        if GameConfig.HogCPU != 0 {
            SDL_Delay(1);
        }

        if just_started {
            just_started = false;
            let now = SDL_GetTicks();
            let mut key;
            loop {
                key = any_key_just_pressed();
                if key == 0 && (SDL_GetTicks() - now < SHOW_WAIT as u32) {
                    SDL_Delay(1); // wait before starting auto-scroll
                } else {
                    break;
                }
            }

            if (key == key_cmds[Cmds::Fire as usize][0])
                || (key == key_cmds[Cmds::Fire as usize][1])
                || (key == key_cmds[Cmds::Fire as usize][2])
            {
                trace!("in just_started: Fire registered");
                ret = 1;
                break;
            }
            prev_tick = SDL_GetTicks();
        }

        if FirePressedR() {
            trace!("outside just_started: Fire registered");
            ret = 1;
            break;
        }

        if UpPressed() || WheelUpPressed() {
            speed -= 5;
            if speed < -MAX_SPEED {
                speed = -MAX_SPEED;
            }
        }
        if DownPressed() || WheelDownPressed() {
            speed += 5;
            if speed > MAX_SPEED {
                speed = MAX_SPEED;
            }
        }

        insert_line -= (f64::from(SDL_GetTicks() - prev_tick) * f64::from(speed) / 1000.0) as f32;

        if insert_line > f32::from(rect.y) + f32::from(rect.h) {
            insert_line = f32::from(rect.y) + f32::from(rect.h);
            if speed < 0 {
                speed = 0;
            }
        }
    }

    SDL_UpperBlit(background, null_mut(), ne_screen, null_mut());
    SDL_Flip(ne_screen);
    SDL_FreeSurface(background);

    ret
}
