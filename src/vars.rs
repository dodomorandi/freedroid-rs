use std::os::raw::{c_char, c_int};

extern "C" {
    pub static mut InfluenceModeNames: [*mut c_char; 25];
    pub static mut ShipEmptyCounter: c_int;
}
