use crate::{defs::Sound, global::sound_on};

use log::{info, warn};
use std::{
    convert::TryFrom,
    ffi::CStr,
    os::raw::{c_char, c_int},
};

extern "C" {
    fn Mix_PlayChannelTimed(
        channel: c_int,
        chunk: *mut Mix_Chunk,
        loops: c_int,
        ticks: c_int,
    ) -> c_int;
    static mut Loaded_WAV_Files: [*mut Mix_Chunk; Sound::All as usize];
    static SoundSampleFilenames: [*mut c_char; Sound::All as usize];
    pub fn FreeSounds();
}

#[repr(C)]
struct Mix_Chunk {
    allocated: c_int,
    abuf: *mut u8,
    alen: u32,
    volume: u8,
}

#[inline]
unsafe fn mix_play_channel(channel: c_int, chunk: *mut Mix_Chunk, loops: c_int) -> c_int {
    Mix_PlayChannelTimed(channel, chunk, loops, -1)
}

#[no_mangle]
pub unsafe extern "C" fn CrySound() {
    Play_Sound(Sound::Cry as i32);
}

#[no_mangle]
pub unsafe extern "C" fn TransferSound() {
    Play_Sound(Sound::Transfer as i32);
}

#[no_mangle]
pub unsafe extern "C" fn Play_Sound(tune: c_int) {
    if sound_on == 0 {
        return;
    }

    let tune = usize::try_from(tune).unwrap();
    let newest_sound_channel = mix_play_channel(-1, Loaded_WAV_Files[tune], 0);
    if newest_sound_channel == -1 {
        warn!(
            "Could not play sound-sample: {} Error: {}.\
             This usually just means that too many samples where played at the same time",
            CStr::from_ptr(SoundSampleFilenames[tune]).to_string_lossy(),
            sdl::get_error(),
        );
    } else {
        info!(
            "Successfully playing file {}.",
            CStr::from_ptr(SoundSampleFilenames[tune]).to_string_lossy()
        );
    }
}
