use crate::{misc::Frame_Time, sound::RefreshSound, GameConfig, LastRefreshSound, Me, RealScore};

use cstr::cstr;
use std::os::raw::{c_char, c_int};

extern "C" {
    pub fn AnimateInfluence();
    pub fn CheckInfluenceEnemyCollision();
    pub fn CheckInfluenceWallCollisions();
    pub fn MoveInfluence();
    pub fn InitInfluPositionHistory();
    pub fn ExplodeInfluencer();
}

const REFRESH_ENERGY: f32 = 3.;

/// Refresh fields can be used to regain energy
/// lost due to bullets or collisions, but not energy lost due to permanent
/// loss of health in PermanentLoseEnergy.
///
/// This function now takes into account the framerates.
#[no_mangle]
pub unsafe extern "C" fn RefreshInfluencer() {
    static mut TIME_COUNTER: c_int = 3; /* to slow down healing process */

    TIME_COUNTER -= 1;
    if TIME_COUNTER != 0 {
        return;
    }
    TIME_COUNTER = 3;

    if Me.energy < Me.health {
        Me.energy += REFRESH_ENERGY * Frame_Time() * 5.;
        RealScore -= REFRESH_ENERGY * Frame_Time() * 10.;

        if RealScore < 0. {
            // don't go negative...
            RealScore = 0.;
        }

        if Me.energy > Me.health {
            Me.energy = Me.health;
        }

        if LastRefreshSound > 0.6 {
            RefreshSound();
            LastRefreshSound = 0.;
        }

        // since robots like the refresh, the influencer might also say so...
        if GameConfig.Droid_Talk != 0 {
            Me.TextToBeDisplayed = cstr!("Ahhh, that feels so good...").as_ptr() as *mut c_char;
            Me.TextVisibleTime = 0.;
        }
    } else {
        // If nothing more is to be had, the influencer might also say so...
        if GameConfig.Droid_Talk != 0 {
            Me.TextToBeDisplayed = cstr!("Oh, it seems that was it again.").as_ptr() as *mut c_char;
            Me.TextVisibleTime = 0.;
        }
    }
}
