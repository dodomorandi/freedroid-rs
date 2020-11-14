use crate::{
    b_font::{CharWidth, FontHeight, GetCurrentFont, PutString},
    defs::{
        Cmds, DownPressedR, FirePressedR, LeftPressedR, PointerStates, RightPressedR, UpPressedR,
        TEXT_STRETCH,
    },
    global::{joy_num_axes, joy_sensitivity, ne_screen, Screen_Rect},
    graphics::vid_bpp,
    input::{cmd_is_activeR, update_input, KeyIsPressedR},
};

use log::{error, info};
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
        SDL_CreateRGBSurface, SDL_Flip, SDL_Rect, SDL_Surface, SDL_UpdateRect, SDL_UpperBlit,
    },
};
use std::{
    convert::{TryFrom, TryInto},
    ffi::{CStr, VaList},
    os::raw::{c_char, c_int, c_uchar},
    ptr::null_mut,
};

extern "C" {
    static mut MyCursorX: c_int;
    static mut MyCursorY: c_int;
    static mut TextBuffer: [c_char; 10000];
    fn SDL_PushEvent(event: *mut SDL_Event) -> c_int;
    fn SDL_GetClipRect(surface: *mut SDL_Surface, rect: *mut SDL_Rect);
    fn SDL_SetClipRect(surface: *mut SDL_Surface, rect: *const SDL_Rect) -> bool;
    fn vsprintf(str: *mut c_char, format: *const c_char, ap: VaList) -> c_int;
    fn linebreak_needed(textpos: *const c_char, clip: *const SDL_Rect) -> bool;
    fn DisplayChar(c: c_uchar);

}

#[cfg(feature = "arcade-input")]
const ARCADE_INPUT_CHARS: [c_int; 70] = [
    48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 61, 42, 43, 44, 45, 46, 47, 65, 66, 67, 68, 69, 70, 71,
    72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 32, 97, 98, 99,
    100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118,
    119, 120, 121, 122,
];

/// Reads a string of "MaxLen" from User-input, and echos it
/// either to stdout or using graphics-text, depending on the
/// parameter "echo":	echo=0    no echo
///                 	echo=1    print using printf
///  			echo=2    print using graphics-text
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
        let mut tmp_rect = store_rect.clone();
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

                if KeyIsPressedR(SDLK_RETURN.try_into().unwrap())
                // For GCW0, maybe we need a prompt to say [PRESS ENTER WHEN FINISHED], or any other key we may choose...
                {
                    input[curpos] = 0; // The last char is currently shown but, not entered into the string...
                                       // 	  input[curpos] = key; // Not sure which one would be expected by most users; the last blinking char is input or not?
                    finished = true;
                } else if UpPressedR()
                // UP
                /* Currently, the key will work ON RELEASE; we might change this to
                	* ON PRESS and add a counter / delay after which while holding, will
                	* scroll trough the chars */
                {
                    inputchar += 1;
                } else if DownPressedR()
                // DOWN
                {
                    inputchar -= 1;
                } else if FirePressedR()
                // FIRE
                {
                    // ADVANCE CURSOR
                    input[curpos] = key.try_into().unwrap(); // Needed in case character has just blinked out...
                    curpos += 1;
                // key=startkey; // Reselect A or not?
                } else if LeftPressedR() {
                    inputchar -= 5;
                } else if RightPressedR() {
                    inputchar += 5;
                } else if cmd_is_activeR(Cmds::Activate)
                // CAPITAL <-> small
                {
                    if inputchar >= 17 && inputchar <= 42 {
                        inputchar = 44 + (inputchar - 17);
                    } else if inputchar >= 44 && inputchar <= 69 {
                        inputchar = 17 + (inputchar - 44);
                    }
                }
                // else if ... other functions to consider: SPACE
                else if KeyIsPressedR(SDLK_BACKSPACE.try_into().unwrap())
                // Or any othe key we choose for the GCW0!
                {
                    input[curpos] = empty_char;
                    if curpos > 0 {
                        curpos -= 1
                    };
                } // (el)ifs Pressed
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
    let mut return_key = 0 as c_int;

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

    let mut tmp = text; /* running text-pointer */

    let clip = &*clip;
    while *tmp != 0 && MyCursorY < c_int::from(clip.y) + c_int::from(clip.h) {
        if *tmp == b'\n' as c_char {
            MyCursorX = clip.x.into();
            MyCursorY += (f64::from(FontHeight(&*GetCurrentFont())) * TEXT_STRETCH) as c_int;
        } else {
            DisplayChar(*tmp as c_uchar);
        }

        tmp = tmp.add(1);

        if linebreak_needed(tmp, clip) {
            tmp = tmp.add(1); // skip the space when doing line-breaks !
            MyCursorX = clip.x.into();
            MyCursorY += (f64::from(FontHeight(&*GetCurrentFont())) * TEXT_STRETCH) as c_int;
        }
    } // while !FensterVoll()

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
