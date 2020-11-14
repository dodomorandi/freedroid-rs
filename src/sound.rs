use crate::defs::Sound;

use std::os::raw::c_int;

extern "C" {
    pub fn Play_Sound(tune: c_int);
}

#[no_mangle]
pub unsafe extern "C" fn CrySound() {
    Play_Sound(Sound::Cry as i32);
}

#[no_mangle]
pub unsafe extern "C" fn TransferSound() {
    Play_Sound(Sound::Transfer as i32);
}
