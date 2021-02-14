use std::os::raw::c_int;

extern "C" {
    pub fn DeleteBullet(num: c_int);
    pub fn CheckBulletCollisions(num: c_int);
    pub fn MoveBullets();
    pub fn ExplodeBlasts();
}
