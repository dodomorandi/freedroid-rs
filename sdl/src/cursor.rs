use std::{marker::PhantomData, os::raw::c_int, ptr::NonNull};

use sdl_sys::{
    SDL_CreateCursor, SDL_Cursor, SDL_DISABLE, SDL_ENABLE, SDL_FreeCursor, SDL_SetCursor,
    SDL_ShowCursor,
};

pub struct Unassociated<'sdl>(PhantomData<&'sdl *const ()>);

#[derive(Debug)]
pub struct Cursor<'sdl, 'data> {
    pointer: NonNull<SDL_Cursor>,
    _marker: PhantomData<(&'sdl *const (), &'data ())>,
}

impl Cursor<'_, '_> {
    pub fn set_active(&self) {
        unsafe { SDL_SetCursor(self.pointer.as_ptr()) }
    }
}

// FIXME: we need const_generic_exprs to take arbitrary widths
const WIDTH: usize = 32 / 8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Data<const HEIGHT: usize> {
    values: [[u8; WIDTH]; HEIGHT],
    mask: [[u8; WIDTH]; HEIGHT],
    upper_left_corner: [c_int; 2],
}

impl<const HEIGHT: usize> Data<HEIGHT> {
    #[must_use]
    pub fn from_draw(draw: &[[u8; WIDTH * 8]; HEIGHT]) -> Self {
        let mut values = [[0; WIDTH]; HEIGHT];
        let mut mask = [[0; WIDTH]; HEIGHT];

        draw.iter()
            .zip(values.iter_mut())
            .zip(mask.iter_mut())
            .flat_map(|((draw_line, values_line), mask_line)| {
                draw_line
                    .chunks_exact(8)
                    .zip(values_line.iter_mut())
                    .zip(mask_line.iter_mut())
            })
            .for_each(|((draw_chunk, value_byte), mask_byte)| {
                let [value, mask] =
                    draw_chunk
                        .iter()
                        .copied()
                        .fold([0, 0], |[acc_value, acc_mask], draw| {
                            let [value, mask] = match draw {
                                b' ' => [0, 0],
                                b'.' => [0, 1],
                                b'X' => [1, 1],
                                c => panic!("invalid drawing character {}", c as char),
                            };

                            [(acc_value << 1) | value, (acc_mask << 1) | mask]
                        });
                *value_byte = value;
                *mask_byte = mask;
            });

        Self {
            values,
            mask,
            upper_left_corner: [0, 0],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum CreationError {
    #[error("height value cannot be converted c_int")]
    InvalidHeight,

    #[error("SDL returned an error while creating cursor")]
    Sdl,
}

impl<'sdl> Unassociated<'sdl> {
    pub(crate) fn new() -> Self {
        Self(PhantomData)
    }

    pub fn from_data<'data, const HEIGHT: usize>(
        &self,
        data: &'data Data<HEIGHT>,
    ) -> Result<Cursor<'sdl, 'data>, CreationError> {
        let height = HEIGHT
            .try_into()
            .map_err(|_| CreationError::InvalidHeight)?;

        // Safety
        // - The created `Cursor` cannot outlive `'sdl` or `'data`.
        // - Array of arrays have linear and contiguous memory.
        // - The pointer cast is safe because the signature of the C function is wrong, and the
        //   pointed data is never changed.
        let pointer = unsafe {
            SDL_CreateCursor(
                data.values[0].as_ptr().cast_mut(),
                data.mask[0].as_ptr().cast_mut(),
                (WIDTH * 8).try_into().unwrap(),
                height,
                data.upper_left_corner[0],
                data.upper_left_corner[1],
            )
        };

        let pointer = NonNull::new(pointer).ok_or(CreationError::Sdl)?;
        Ok(Cursor {
            pointer,
            _marker: PhantomData,
        })
    }

    #[allow(clippy::must_use_candidate)]
    pub fn show(&self) -> bool {
        unsafe { show_cursor(SDL_ENABLE) }
    }

    #[allow(clippy::must_use_candidate)]
    pub fn hide(&self) -> bool {
        unsafe { show_cursor(SDL_DISABLE) }
    }
}

/// Change the visibility of the SDL cursor. Returns a bool indicating whether the curosr was shown
/// before the call.
///
/// # Safety
/// - There must be an active SDL instance.
/// - `value` must be [`SDL_ENABLE`], [`SDL_DISABLE`] or `-1`.
unsafe fn show_cursor(value: c_int) -> bool {
    let previous_state = unsafe { SDL_ShowCursor(value) };
    previous_state == 1
}

impl Drop for Cursor<'_, '_> {
    fn drop(&mut self) {
        unsafe { SDL_FreeCursor(self.pointer.as_ptr()) }
    }
}
