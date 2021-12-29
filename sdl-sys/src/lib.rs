#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(deref_nullptr)]
#![allow(clippy::redundant_static_lifetimes)]
#![allow(clippy::too_many_arguments)]

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
