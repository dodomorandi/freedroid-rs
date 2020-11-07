mod b_font;
mod defs;
mod global;
mod graphics;
mod highscore;
mod input;
mod structs;
mod text;
mod view;

use std::{
    env,
    ffi::CString,
    os::raw::{c_char, c_float, c_int},
    process,
};

use defs::MAX_INFLU_POSITION_HISTORY;

extern "C" {
    fn c_main(argc: c_int, argv: *const *const c_char) -> c_int;
}

fn main() {
    env_logger::init();

    let args: Vec<_> = env::args().map(|arg| CString::new(arg).unwrap()).collect();
    let c_args: Vec<_> = args.iter().map(|arg| arg.as_ptr()).collect();

    let retval = unsafe { c_main(c_args.len() as c_int, c_args.as_ptr()) };
    if retval != 0 {
        process::exit(retval);
    }
}

#[repr(C)]
#[derive(Clone, Default, PartialEq)]
struct FinePoint {
    x: c_float,
    y: c_float,
}

#[repr(C)]
#[derive(Clone, Default, PartialEq)]
struct Gps {
    x: c_float,
    y: c_float,
    z: c_int,
}

#[repr(C)]
struct Influence {
    ty: c_int,
    status: c_int,
    speed: FinePoint,
    pos: FinePoint,
    health: c_float,
    energy: c_float,
    firewait: c_float,
    phase: c_float,
    timer: c_float,
    last_crysound_time: c_float,
    last_transfer_sound_time: c_float,
    text_visible_time: c_float,
    text_to_be_displayed: *mut c_char,
    position_history_eing_buffer: [Gps; MAX_INFLU_POSITION_HISTORY],
}

#[repr(C)]
enum InfluenceStatus {
    Mobile,
    TransferMode,
    Weapon,
    Captured,
    Complete,
    Rejected,
    Console,
    Debriefing,
    Terminated,
    Pause,
    Cheese,
    Elevator,
    Briefing,
    Menu,
    Victory,
    Activate,
    Out,
}
