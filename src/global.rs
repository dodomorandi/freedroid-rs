use sdl::video::ll::SDL_Surface;

extern "C" {
    #[no_mangle]
    pub static mut ne_screen: *mut SDL_Surface;
}
