use std::os::raw::c_char;

extern "C" {
    pub static mut InfluenceModeNames: [*mut c_char; 25];
}
