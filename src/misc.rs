use crate::global::{ne_screen, progress_filler_pic, ProgressBar_Rect, ProgressMeter_Rect};

use sdl::{
    sdl::Rect,
    video::ll::{SDL_UpdateRects, SDL_UpperBlit},
};
use std::os::raw::{c_float, c_int};

extern "C" {
    pub fn Frame_Time() -> c_float;
    pub fn Terminate(ExitCode: c_int);
    pub fn MyRandom(upper_bound: c_int) -> c_int;
    pub fn Pause();

}

#[no_mangle]
pub unsafe extern "C" fn update_progress(percent: c_int) {
    let h = (f64::from(ProgressBar_Rect.h) * f64::from(percent) / 100.) as u16;
    let mut dst = Rect::new(
        ProgressBar_Rect.x + ProgressMeter_Rect.x,
        ProgressBar_Rect.y + ProgressMeter_Rect.y + ProgressBar_Rect.h as i16 - h as i16,
        ProgressBar_Rect.w,
        h,
    );

    let mut src = Rect::new(0, ProgressBar_Rect.h as i16 - dst.h as i16, dst.h, 0);

    SDL_UpperBlit(progress_filler_pic, &mut src, ne_screen, &mut dst);
    SDL_UpdateRects(ne_screen, 1, &mut dst);
}
