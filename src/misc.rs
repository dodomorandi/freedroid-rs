use std::os::raw::{c_float, c_int};

extern "C" {
    #[no_mangle]
    pub fn Frame_Time() -> c_float;

    #[no_mangle]
    pub fn Terminate(ExitCode: c_int);
}
