use std::os::raw::c_int;

extern "C" {
    #[no_mangle]
    pub fn Assemble_Combat_Picture(mask: c_int);
}
