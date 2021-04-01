use crate::vars::Druidmap;

use std::{convert::TryFrom, os::raw::c_int};

extern "C" {
    pub fn AnimateEnemys();
    pub fn ShuffleEnemys();
    pub fn MoveEnemys();
    pub fn ClearEnemys();
}

#[no_mangle]
pub unsafe extern "C" fn ClassOfDruid(druid_type: c_int) -> c_int {
    /* first digit is class */
    let class_char = (*Druidmap.add(usize::try_from(druid_type).unwrap())).druidname[0] as u8;
    match class_char {
        b'0'..=b'9' => (class_char - b'0').into(),
        _ => 0,
    }
}
