use crate::{
    defs::{Bullet, Criticality, Sound, Themed, BYCOLOR, NUM_COLORS, SOUND_DIR_C},
    global::{sound_on, CurLevel, GameConfig},
    misc::find_file,
};

use log::{error, info, warn};
use sdl::{audio::ll::SDL_CloseAudio, sdl::get_error};
use std::{
    convert::TryFrom,
    ffi::CStr,
    os::raw::{c_char, c_int},
};

extern "C" {
    pub type Mix_Music;
    fn Mix_PlayChannelTimed(
        channel: c_int,
        chunk: *mut Mix_Chunk,
        loops: c_int,
        ticks: c_int,
    ) -> c_int;
    fn Mix_FreeChunk(chunk: *mut Mix_Chunk);
    fn Mix_FreeMusic(music: *mut Mix_Music);
    fn Mix_PauseMusic();
    fn Mix_ResumeMusic();
    fn Mix_CloseAudio();
    fn Mix_PlayMusic(music: *mut Mix_Music, loops: c_int) -> c_int;
    fn Mix_VolumeMusic(volume: c_int) -> c_int;
    fn Mix_LoadMUS(file: *const c_char) -> *mut Mix_Music;

    static mut Loaded_WAV_Files: [*mut Mix_Chunk; Sound::All as usize];
    static SoundSampleFilenames: [*mut c_char; Sound::All as usize];
    static mut MusicSongs: [*mut Mix_Music; NUM_COLORS];
    static mut Tmp_MOD_File: *mut Mix_Music;
}

const MIX_MAX_VOLUME: u8 = 128;

#[repr(C)]
struct Mix_Chunk {
    allocated: c_int,
    abuf: *mut u8,
    alen: u32,
    volume: u8,
}

#[inline]
unsafe fn mix_play_channel(channel: c_int, chunk: *mut Mix_Chunk, loops: c_int) -> c_int {
    Mix_PlayChannelTimed(channel, chunk, loops, -1)
}

#[no_mangle]
pub unsafe extern "C" fn CrySound() {
    Play_Sound(Sound::Cry as i32);
}

#[no_mangle]
pub unsafe extern "C" fn TransferSound() {
    Play_Sound(Sound::Transfer as i32);
}

#[no_mangle]
pub unsafe extern "C" fn Play_Sound(tune: c_int) {
    if sound_on == 0 {
        return;
    }

    let tune = usize::try_from(tune).unwrap();
    let newest_sound_channel = mix_play_channel(-1, Loaded_WAV_Files[tune], 0);
    if newest_sound_channel == -1 {
        warn!(
            "Could not play sound-sample: {} Error: {}.\
             This usually just means that too many samples where played at the same time",
            CStr::from_ptr(SoundSampleFilenames[tune]).to_string_lossy(),
            sdl::get_error(),
        );
    } else {
        info!(
            "Successfully playing file {}.",
            CStr::from_ptr(SoundSampleFilenames[tune]).to_string_lossy()
        );
    }
}

#[no_mangle]
pub unsafe extern "C" fn FreeSounds() {
    Loaded_WAV_Files
        .iter()
        .filter(|file| !file.is_null())
        .for_each(|&file| Mix_FreeChunk(file));

    MusicSongs
        .iter()
        .filter(|song| !song.is_null())
        .for_each(|&song| Mix_FreeMusic(song));

    if !Tmp_MOD_File.is_null() {
        Mix_FreeMusic(Tmp_MOD_File);
    }

    Mix_CloseAudio();
    SDL_CloseAudio();
}

#[no_mangle]
pub unsafe extern "C" fn Takeover_Set_Capsule_Sound() {
    if sound_on == 0 {
        return;
    }

    Play_Sound(Sound::TakeoverSetCapsule as i32);
}

#[no_mangle]
pub unsafe extern "C" fn Takeover_Game_Won_Sound() {
    if sound_on == 0 {
        return;
    }

    Play_Sound(Sound::TakeoverGameWon as i32);
}

#[no_mangle]
pub unsafe extern "C" fn Takeover_Game_Deadlock_Sound() {
    if sound_on == 0 {
        return;
    }

    Play_Sound(Sound::TakeoverGameDeadlock as i32);
}

#[no_mangle]
pub unsafe extern "C" fn Takeover_Game_Lost_Sound() {
    if sound_on == 0 {
        return;
    }

    Play_Sound(Sound::TakeoverGameLost as i32);
}

#[no_mangle]
pub unsafe extern "C" fn CollisionGotDamagedSound() {
    if sound_on == 0 {
        return;
    }

    Play_Sound(Sound::CollisionGotDamaged as i32);
}

#[no_mangle]
pub unsafe extern "C" fn CollisionDamagedEnemySound() {
    if sound_on == 0 {
        return;
    }

    Play_Sound(Sound::CollisionDamagedEnemy as i32);
}

#[no_mangle]
pub unsafe extern "C" fn BounceSound() {
    if sound_on == 0 {
        return;
    }

    Play_Sound(Sound::Collision as i32);
}

#[no_mangle]
pub unsafe extern "C" fn DruidBlastSound() {
    if sound_on == 0 {
        return;
    }

    Play_Sound(Sound::Blast as i32);
}

#[no_mangle]
pub unsafe extern "C" fn GotHitSound() {
    if sound_on == 0 {
        return;
    }

    Play_Sound(Sound::GotHit as i32);
}

#[no_mangle]
pub unsafe extern "C" fn GotIntoBlastSound() {
    if sound_on == 0 {
        return;
    }

    Play_Sound(Sound::GotIntoBlast as i32);
}

#[no_mangle]
pub unsafe extern "C" fn RefreshSound() {
    if sound_on == 0 {
        return;
    }

    Play_Sound(Sound::Refresh as i32);
}

#[no_mangle]
pub unsafe extern "C" fn MoveLiftSound() {
    if sound_on == 0 {
        return;
    }

    Play_Sound(Sound::MoveElevator as i32);
}

#[no_mangle]
pub unsafe extern "C" fn MenuItemSelectedSound() {
    if sound_on == 0 {
        return;
    }

    Play_Sound(Sound::MenuItemSelected as i32);
}

#[no_mangle]
pub unsafe extern "C" fn MoveMenuPositionSound() {
    if sound_on == 0 {
        return;
    }

    Play_Sound(Sound::MoveMenuPosition as i32);
}

#[no_mangle]
pub unsafe extern "C" fn ThouArtDefeatedSound() {
    if sound_on == 0 {
        return;
    }
    Play_Sound(Sound::ThouArtDefeated as i32);
}

#[no_mangle]
pub unsafe extern "C" fn EnterLiftSound() {
    if sound_on == 0 {
        return;
    }

    Play_Sound(Sound::EnterElevator as i32);
}

#[no_mangle]
pub unsafe extern "C" fn LeaveLiftSound() {
    if sound_on == 0 {
        return;
    }

    Play_Sound(Sound::LeaveElevator as i32);
}

#[no_mangle]
pub unsafe extern "C" fn Fire_Bullet_Sound(bullet_type: c_int) {
    if sound_on == 0 {
        return;
    }

    use Bullet::*;
    let bullet_type = match bullet_type {
        0 => Pulse,
        1 => SinglePulse,
        2 => Military,
        3 => Flash,
        4 => Exterminator,
        5 => LaserRifle,
        _ => panic!("invalid bullet type {}", bullet_type),
    };

    match bullet_type {
        Pulse => Play_Sound(Sound::FireBulletPulse as i32),
        SinglePulse => Play_Sound(Sound::FireBulletSinglePulse as i32),
        Military => Play_Sound(Sound::FireBulletMilitary as i32),
        Flash => Play_Sound(Sound::FireBulletFlash as i32),
        Exterminator => Play_Sound(Sound::FireBulletExterminator as i32),
        LaserRifle => Play_Sound(Sound::FireBulletLaserRifle as i32),
    }
}

#[no_mangle]
pub unsafe extern "C" fn Switch_Background_Music_To(filename_raw: *const c_char) {
    static mut PREV_COLOR: c_int = -1;
    static mut PAUSED: bool = false;

    if sound_on == 0 {
        return;
    }

    if filename_raw.is_null() {
        Mix_PauseMusic(); // pause currently played background music
        PAUSED = true;
        return;
    }

    let filename_raw = CStr::from_ptr(filename_raw);

    // New feature: choose background music by level-color:
    // if filename_raw==BYCOLOR then chose bg_music[color]
    // NOTE: if new level-color is the same as before, just resume paused music!
    if filename_raw.to_bytes() == BYCOLOR.as_bytes() {
        if PAUSED && PREV_COLOR == (*CurLevel).color {
            // current level-song was just paused
            Mix_ResumeMusic();
            PAUSED = false;
        } else {
            Mix_PlayMusic(MusicSongs[usize::try_from((*CurLevel).color).unwrap()], -1);
            PAUSED = false;
            PREV_COLOR = (*CurLevel).color;
        }
    } else {
        // not using BYCOLOR mechanism: just play specified song
        if !Tmp_MOD_File.is_null() {
            Mix_FreeMusic(Tmp_MOD_File);
        }
        let fpath = find_file(
            filename_raw.as_ptr() as *const c_char,
            SOUND_DIR_C.as_ptr() as *mut c_char,
            Themed::NoTheme as c_int,
            Criticality::WarnOnly as c_int,
        );
        if fpath.is_null() {
            error!(
                "Error loading sound-file: {}",
                filename_raw.to_string_lossy()
            );
            return;
        }
        Tmp_MOD_File = Mix_LoadMUS(fpath);
        if Tmp_MOD_File.is_null() {
            error!(
                "SDL Mixer Error: {}. Continuing with sound disabled",
                get_error(),
            );
            return;
        }
        Mix_PlayMusic(Tmp_MOD_File, -1);
    }

    Mix_VolumeMusic((GameConfig.Current_BG_Music_Volume * f32::from(MIX_MAX_VOLUME)) as c_int);
}

#[no_mangle]
pub unsafe extern "C" fn CountdownSound() {
    Play_Sound(Sound::Countdown as i32);
}

#[no_mangle]
pub unsafe extern "C" fn EndCountdownSound() {
    Play_Sound(Sound::Endcountdown as i32);
}
