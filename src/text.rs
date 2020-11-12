use crate::{
    b_font::{FontHeight, GetCurrentFont, PutString},
    defs::{Cmds, DownPressedR, FirePressedR, LeftPressedR, RightPressedR, UpPressedR},
    global::{ne_screen, Screen_Rect},
    graphics::vid_bpp,
    input::{cmd_is_activeR, KeyIsPressedR},
};

use log::{error, info};
#[cfg(not(feature = "arcade-input"))]
use sdl::keysym::SDLK_DELETE;
use sdl::{
    keysym::{SDLK_BACKSPACE, SDLK_RETURN},
    sdl::{ll::SDL_GetTicks, Rect},
    video::ll::{SDL_CreateRGBSurface, SDL_Flip, SDL_Rect, SDL_Surface, SDL_UpperBlit},
};
use std::{
    convert::{TryFrom, TryInto},
    ffi::CStr,
    os::raw::{c_char, c_int},
    ptr::null_mut,
};

extern "C" {
    #[no_mangle]
    pub fn DisplayText(
        text: *const c_char,
        startx: c_int,
        starty: c_int,
        clip: *const SDL_Rect,
    ) -> c_int;

    #[no_mangle]
    pub fn printf_SDL(screen: *mut SDL_Surface, x: c_int, y: c_int, fmt: *mut c_char, ...);

    #[no_mangle]
    static mut MyCursorX: c_int;

    #[no_mangle]
    static mut MyCursorY: c_int;

    #[no_mangle]
    pub fn getchar_raw() -> c_int;
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
    static mut last_frame_time: u32 = 0; //  = SDL_GetTicks();
    #[cfg(feature = "arcade-input")]
    let mut inputchar = 17; // initial char = A
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
                inputchar += ARCADE_INPUT_CHARS.len();
            }
            if inputchar >= ARCADE_INPUT_CHARS.len() {
                inputchar -= ARCADE_INPUT_CHARS.len();
            }
            let key = ARCADE_INPUT_CHARS[usize::try_from(inputchar).unwrap()];

            let frame_duration = SDL_GetTicks() - last_frame_time;
            if frame_duration > blink_time / 2 {
                input[curpos] = key.try_into().unwrap(); // We want to show the currently chosen character
                if frame_duration > blink_time {
                    last_frame_time = SDL_GetTicks();
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
