use crate::defs::MenuAction;

use std::os::raw::c_char;

extern "C" {
    pub fn handle_QuitGame(action: MenuAction) -> *const c_char;
    pub fn showMainMenu();
    pub fn Cheatmenu();
    pub fn FreeMenuData();
}
