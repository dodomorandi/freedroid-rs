use std::os::raw::c_int;

extern "C" {
    pub fn AnimateEnemys();
    pub fn ShuffleEnemys();
    pub fn ClassOfDruid(druid_type: c_int) -> c_int;
}
