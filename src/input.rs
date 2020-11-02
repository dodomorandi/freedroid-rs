use std::os::raw::c_int;

extern "C" {
    #[no_mangle]
    pub fn wait_for_key_pressed() -> c_int;
}
