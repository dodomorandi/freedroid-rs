use std::os::raw::c_float;

extern "C" {
    #[no_mangle]
    pub fn Frame_Time() -> c_float;
}
