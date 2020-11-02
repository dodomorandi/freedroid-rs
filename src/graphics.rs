use sdl::video::ll::SDL_Rect;

extern "C" {
    #[no_mangle]
    pub fn MakeGridOnScreen(grid_rectangle: *mut SDL_Rect);
}
