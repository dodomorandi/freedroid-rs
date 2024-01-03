use std::mem;

#[must_use]
pub const fn u32_to_u16(value: u32) -> u16 {
    assert!(
        value <= u16::MAX as u32,
        "u32 too big to being converted to u16"
    );

    // Checked above
    #[allow(clippy::cast_possible_truncation)]
    let value = value as u16;
    value
}

#[must_use]
pub const fn u32_to_u8(value: u32) -> u8 {
    assert!(
        value <= u8::MAX as u32,
        "u32 too big to being converted to u8"
    );

    // Checked above
    #[allow(clippy::cast_possible_truncation)]
    let value = value as u8;
    value
}

#[must_use]
pub const fn i32_to_u8(value: i32) -> u8 {
    assert!(
        value >= 0 && value <= u8::MAX as i32,
        "i32 less than 0 or too big to being converted to u8"
    );

    // Checked above
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let value = value as u8;
    value
}

#[must_use]
pub const fn i32_to_u32(value: i32) -> u32 {
    assert!(value >= 0, "i32 less than 0");

    // Checked above
    #[allow(clippy::cast_sign_loss)]
    let value = value as u32;
    value
}

#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub const fn u32_to_isize(value: u32) -> isize {
    if mem::size_of::<isize>() <= 4 {
        assert!(
            value <= isize::MAX as u32,
            "u32 too big to being converted to isize"
        );
    }

    // Checked above
    #[allow(clippy::cast_possible_wrap)]
    let value = value as isize;
    value
}

#[must_use]
pub const fn u32_to_usize(value: u32) -> usize {
    #[allow(clippy::cast_possible_truncation)]
    if mem::size_of::<usize>() <= 4 {
        assert!(
            value <= usize::MAX as u32,
            "u32 too big to being converted to usize"
        );
    }

    value as usize
}

#[must_use]
pub const fn i64_to_u32(value: i64) -> u32 {
    assert!(
        value >= 0 && value <= (u32::MAX as i64),
        "i64 is not a valid u32"
    );

    // Checked above
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let value = value as u32;
    value
}

#[must_use]
pub const fn u32_to_i32(value: u32) -> i32 {
    assert!(
        value <= i32::MAX as u32,
        "u32 too big to being converted to i32"
    );

    // Checked above
    #[allow(clippy::cast_possible_wrap)]
    let value = value as i32;
    value
}

#[must_use]
pub const fn u8_to_usize(value: u8) -> usize {
    value as usize
}
