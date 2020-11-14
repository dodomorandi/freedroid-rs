use std::os::raw::{c_float, c_int};

extern "C" {
    pub fn Frame_Time() -> c_float;
    pub fn Terminate(ExitCode: c_int);
}
