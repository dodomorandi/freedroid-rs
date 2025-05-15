#![expect(non_upper_case_globals)]
#![expect(non_camel_case_types)]
#![expect(non_snake_case)]
#![expect(clippy::too_many_arguments)]
#![expect(clippy::useless_transmute)]
#![expect(clippy::missing_safety_doc)]
#![expect(clippy::ptr_offset_with_cast)]
#![expect(unsafe_op_in_unsafe_fn)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub const MIX_DEFAULT_FREQUENCY: u32 = 22050;

pub const AUDIO_U16LSB: u16 = 0x0010;
pub const AUDIO_S16LSB: u16 = 0x8010;
pub const AUDIO_U16MSB: u16 = 0x1010;
pub const AUDIO_S16MSB: u16 = 0x9010;

#[cfg(target_endian = "little")]
pub const MIX_DEFAULT_FORMAT: u16 = AUDIO_S16LSB;
#[cfg(target_endian = "big")]
pub const MIX_DEFAULT_FORMAT: u16 = AUDIO_S16MSB;

pub const MIX_DEFAULT_CHANNELS: u8 = 2;
