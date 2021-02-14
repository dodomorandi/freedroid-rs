use std::os::raw::{c_char, c_int};

extern "C" {
    pub fn FreeGameMem();
    pub fn InitFreedroid(argc: c_int, argv: *mut *const c_char);
    pub fn InitNewMission(mission_name: *mut c_char);
    pub fn CheckIfMissionIsComplete();
}
