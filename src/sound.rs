use crate::{
    defs::{BulletKind, Criticality, Sound, Themed, BYCOLOR, NUM_COLORS, SOUND_DIR_C},
    global::GAME_CONFIG,
    misc::find_file,
    CUR_LEVEL, SOUND_ON,
};

use cstr::cstr;
use log::{error, info, warn};
use sdl::{
    audio::ll::SDL_CloseAudio,
    sdl::{
        get_error,
        ll::{SDL_InitSubSystem, SDL_INIT_AUDIO},
    },
    video::ll::{SDL_RWFromFile, SDL_RWops},
};
use std::{
    convert::TryFrom,
    ffi::CStr,
    os::raw::{c_char, c_float, c_int, c_void},
    ptr::null_mut,
};

mod inner {
    #[repr(C)]
    pub struct MixMusic {
        _private: [u8; 0],
    }
}
use inner::MixMusic;

extern "C" {
    fn Mix_PlayChannelTimed(
        channel: c_int,
        chunk: *mut Mix_Chunk,
        loops: c_int,
        ticks: c_int,
    ) -> c_int;
    fn Mix_FreeChunk(chunk: *mut Mix_Chunk);
    fn Mix_FreeMusic(music: *mut MixMusic);
    fn Mix_PauseMusic();
    fn Mix_ResumeMusic();
    fn Mix_CloseAudio();
    fn Mix_PlayMusic(music: *mut MixMusic, loops: c_int) -> c_int;
    fn Mix_VolumeMusic(volume: c_int) -> c_int;
    fn Mix_LoadMUS(file: *const c_char) -> *mut MixMusic;
    fn Mix_VolumeChunk(chunk: *mut Mix_Chunk, volume: c_int) -> c_int;
    fn Mix_OpenAudio(frequency: c_int, format: u16, channels: c_int, chunksize: c_int) -> c_int;
    fn Mix_AllocateChannels(num_chans: c_int) -> c_int;
    fn Mix_LoadWAV_RW(src: *mut SDL_RWops, freesrc: c_int) -> *mut Mix_Chunk;
}

const MIX_MAX_VOLUME: u8 = 128;
const MIX_DEFAULT_FREQUENCY: i32 = 22050;

#[cfg(target_endian = "little")]
const AUDIO_S16LSB: u16 = 0x8010;
#[cfg(target_endian = "little")]
const MIX_DEFAULT_FORMAT: u16 = AUDIO_S16LSB;

#[cfg(not(target_endian = "little"))]
const AUDIO_S16MSB: u16 = 0x9010;
#[cfg(not(target_endian = "little"))]
const MIX_DEFAULT_FORMAT: u16 = AUDIO_S16MSB;

#[inline]
unsafe fn mix_load_wav(file: *mut c_char) -> *mut Mix_Chunk {
    Mix_LoadWAV_RW(SDL_RWFromFile(file, cstr!("rb").as_ptr() as *mut c_char), 1)
}

const SOUND_SAMPLE_FILENAMES: [&CStr; Sound::All as usize] = [
    cstr!("ERRORSOUND_NILL.NOWAV"),
    cstr!("Blast_Sound_0.wav"),
    // "Collision_Sound_0.wav", // replaced by damage-dependent-sounds:  Collision_[Neutral|GotDamaged|DamagedEnemy]
    cstr!("Collision_Neutral.wav"),
    cstr!("Collision_GotDamaged.wav"),
    cstr!("Collision_DamagedEnemy.wav"),
    //"GotIntoBlast_Sound_0.wav", // replaced by GotIntoBlast_Sound_1.wav
    cstr!("GotIntoBlast_Sound_1.wav"),
    cstr!("MoveElevator_Sound_0.wav"),
    cstr!("Refresh_Sound_0.wav"),
    cstr!("LeaveElevator_Sound_0.wav"),
    cstr!("EnterElevator_Sound_0.wav"),
    cstr!("ThouArtDefeated_Sound_0.wav"),
    cstr!("Got_Hit_Sound_0.wav"),
    cstr!("TakeoverSetCapsule_Sound_0.wav"),
    cstr!("Menu_Item_Selected_Sound_0.wav"),
    cstr!("Move_Menu_Position_Sound_0.wav"),
    cstr!("Takeover_Game_Won_Sound_0.wav"),
    cstr!("Takeover_Game_Deadlock_Sound_0.wav"),
    cstr!("Takeover_Game_Lost_Sound_0.wav"),
    cstr!("Fire_Bullet_Pulse_Sound_0.wav"),
    cstr!("Fire_Bullet_Single_Pulse_Sound_0.wav"),
    cstr!("Fire_Bullet_Military_Sound_0.wav"),
    cstr!("Fire_Bullet_Flash_Sound_0.wav"),
    cstr!("Fire_Bullet_Exterminator_Sound_0.wav"),
    cstr!("Fire_Bullet_Laser_Rifle_Sound.wav"),
    cstr!("Cry_Sound_0.wav"),
    cstr!("Takeover_Sound_0.wav"),
    cstr!("Countdown_Sound.wav"),
    cstr!("EndCountdown_Sound.wav"),
    cstr!("InfluExplosion.wav"),
    cstr!("WhiteNoise.wav"),
    cstr!("Alert.wav"),
    cstr!("Screenshot.wav"),
];

static mut LOADED_WAV_FILES: [*mut Mix_Chunk; Sound::All as usize] =
    [null_mut(); Sound::All as usize];

const MUSIC_FILES: [&CStr; NUM_COLORS] = [
    cstr!("AnarchyMenu1.mod"),          // RED
    cstr!("starpaws.mod"),              // YELLOW
    cstr!("The_Last_V8.mod"),           // GREEN
    cstr!("dreamfish-green_beret.mod"), // GRAY
    #[cfg(feature = "gcw0")]
    cstr!("dreamfish-green_beret.mod"), // GRAY
    #[cfg(not(feature = "gcw0"))]
    cstr!("dreamfish-sanxion.mod"), // BLUE // CRASHES the GCW0 ???
    cstr!("kollaps-tron.mod"),          // GREENBLUE
    cstr!("dreamfish-uridium2_loader.mod"), // DARK
];

static mut MUSIC_SONGS: [*mut MixMusic; NUM_COLORS] =
    [null_mut::<c_void>() as *mut MixMusic; NUM_COLORS];
static mut TMP_MOD_FILE: *mut MixMusic = null_mut::<c_void>() as *mut MixMusic;

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

pub unsafe fn cry_sound() {
    play_sound(Sound::Cry as i32);
}

pub unsafe fn transfer_sound() {
    play_sound(Sound::Transfer as i32);
}

pub unsafe fn play_sound(tune: c_int) {
    if SOUND_ON == 0 {
        return;
    }

    let tune = usize::try_from(tune).unwrap();
    let newest_sound_channel = mix_play_channel(-1, LOADED_WAV_FILES[tune], 0);
    if newest_sound_channel == -1 {
        warn!(
            "Could not play sound-sample: {} Error: {}.\
             This usually just means that too many samples where played at the same time",
            SOUND_SAMPLE_FILENAMES[tune].to_string_lossy(),
            sdl::get_error(),
        );
    } else {
        info!(
            "Successfully playing file {}.",
            SOUND_SAMPLE_FILENAMES[tune].to_string_lossy()
        );
    }
}

pub unsafe fn free_sounds() {
    LOADED_WAV_FILES
        .iter()
        .filter(|file| !file.is_null())
        .for_each(|&file| Mix_FreeChunk(file));

    MUSIC_SONGS
        .iter()
        .filter(|song| !song.is_null())
        .for_each(|&song| Mix_FreeMusic(song));

    if !TMP_MOD_FILE.is_null() {
        Mix_FreeMusic(TMP_MOD_FILE);
    }

    Mix_CloseAudio();
    SDL_CloseAudio();
}

pub unsafe fn takeover_set_capsule_sound() {
    if SOUND_ON == 0 {
        return;
    }

    play_sound(Sound::TakeoverSetCapsule as i32);
}

pub unsafe fn takeover_game_won_sound() {
    if SOUND_ON == 0 {
        return;
    }

    play_sound(Sound::TakeoverGameWon as i32);
}

pub unsafe fn takeover_game_deadlock_sound() {
    if SOUND_ON == 0 {
        return;
    }

    play_sound(Sound::TakeoverGameDeadlock as i32);
}

pub unsafe fn takeover_game_lost_sound() {
    if SOUND_ON == 0 {
        return;
    }

    play_sound(Sound::TakeoverGameLost as i32);
}

pub unsafe fn collision_got_damaged_sound() {
    if SOUND_ON == 0 {
        return;
    }

    play_sound(Sound::CollisionGotDamaged as i32);
}

pub unsafe fn collision_damaged_enemy_sound() {
    if SOUND_ON == 0 {
        return;
    }

    play_sound(Sound::CollisionDamagedEnemy as i32);
}

pub unsafe fn bounce_sound() {
    if SOUND_ON == 0 {
        return;
    }

    play_sound(Sound::Collision as i32);
}

pub unsafe fn druid_blast_sound() {
    if SOUND_ON == 0 {
        return;
    }

    play_sound(Sound::Blast as i32);
}

pub unsafe fn got_hit_sound() {
    if SOUND_ON == 0 {
        return;
    }

    play_sound(Sound::GotHit as i32);
}

pub unsafe fn got_into_blast_sound() {
    if SOUND_ON == 0 {
        return;
    }

    play_sound(Sound::GotIntoBlast as i32);
}

pub unsafe fn refresh_sound() {
    if SOUND_ON == 0 {
        return;
    }

    play_sound(Sound::Refresh as i32);
}

pub unsafe fn move_lift_sound() {
    if SOUND_ON == 0 {
        return;
    }

    play_sound(Sound::MoveElevator as i32);
}

pub unsafe fn menu_item_selected_sound() {
    if SOUND_ON == 0 {
        return;
    }

    play_sound(Sound::MenuItemSelected as i32);
}

pub unsafe fn move_menu_position_sound() {
    if SOUND_ON == 0 {
        return;
    }

    play_sound(Sound::MoveMenuPosition as i32);
}

pub unsafe fn thou_art_defeated_sound() {
    if SOUND_ON == 0 {
        return;
    }
    play_sound(Sound::ThouArtDefeated as i32);
}

pub unsafe fn enter_lift_sound() {
    if SOUND_ON == 0 {
        return;
    }

    play_sound(Sound::EnterElevator as i32);
}

pub unsafe fn leave_lift_sound() {
    if SOUND_ON == 0 {
        return;
    }

    play_sound(Sound::LeaveElevator as i32);
}

pub unsafe fn fire_bullet_sound(bullet_type: c_int) {
    if SOUND_ON == 0 {
        return;
    }

    use BulletKind::*;
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
        Pulse => play_sound(Sound::FireBulletPulse as i32),
        SinglePulse => play_sound(Sound::FireBulletSinglePulse as i32),
        Military => play_sound(Sound::FireBulletMilitary as i32),
        Flash => play_sound(Sound::FireBulletFlash as i32),
        Exterminator => play_sound(Sound::FireBulletExterminator as i32),
        LaserRifle => play_sound(Sound::FireBulletLaserRifle as i32),
    }
}

pub unsafe fn switch_background_music_to(filename_raw: *const c_char) {
    static mut PREV_COLOR: c_int = -1;
    static mut PAUSED: bool = false;

    if SOUND_ON == 0 {
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
    if filename_raw.to_bytes() == BYCOLOR.to_bytes() {
        if PAUSED && PREV_COLOR == (*CUR_LEVEL).color {
            // current level-song was just paused
            Mix_ResumeMusic();
            PAUSED = false;
        } else {
            Mix_PlayMusic(
                MUSIC_SONGS[usize::try_from((*CUR_LEVEL).color).unwrap()],
                -1,
            );
            PAUSED = false;
            PREV_COLOR = (*CUR_LEVEL).color;
        }
    } else {
        // not using BYCOLOR mechanism: just play specified song
        if !TMP_MOD_FILE.is_null() {
            Mix_FreeMusic(TMP_MOD_FILE);
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
        TMP_MOD_FILE = Mix_LoadMUS(fpath);
        if TMP_MOD_FILE.is_null() {
            error!(
                "SDL Mixer Error: {}. Continuing with sound disabled",
                get_error(),
            );
            return;
        }
        Mix_PlayMusic(TMP_MOD_FILE, -1);
    }

    Mix_VolumeMusic((GAME_CONFIG.current_bg_music_volume * f32::from(MIX_MAX_VOLUME)) as c_int);
}

pub unsafe fn countdown_sound() {
    play_sound(Sound::Countdown as i32);
}

pub unsafe fn end_countdown_sound() {
    play_sound(Sound::Endcountdown as i32);
}

pub unsafe fn set_sound_f_x_volume(new_volume: c_float) {
    if SOUND_ON == 0 {
        return;
    }

    // Set the volume IN the loaded files, if SDL is used...
    // This is done here for the Files 1,2,3 and 4, since these
    // are background music files.
    LOADED_WAV_FILES.iter().skip(1).for_each(|&file| {
        Mix_VolumeChunk(file, (new_volume * f32::from(MIX_MAX_VOLUME)) as c_int);
    });
}

pub unsafe fn set_bg_music_volume(new_volume: c_float) {
    if SOUND_ON == 0 {
        return;
    }

    Mix_VolumeMusic((new_volume * f32::from(MIX_MAX_VOLUME)) as c_int);
}

pub unsafe fn init_audio() {
    info!("Initializing SDL Audio Systems");

    if SOUND_ON == 0 {
        return;
    }

    // Now SDL_AUDIO is initialized here:

    if SDL_InitSubSystem(SDL_INIT_AUDIO) == -1 {
        warn!(
            "SDL Sound subsystem could not be initialized. \
             Continuing with sound disabled",
        );
        SOUND_ON = false.into();
        return;
    } else {
        info!("SDL Audio initialisation successful.");
    }

    // Now that we have initialized the audio SubSystem, we must open
    // an audio channel.  This will be done here (see code from Mixer-Tutorial):

    if Mix_OpenAudio(MIX_DEFAULT_FREQUENCY, MIX_DEFAULT_FORMAT, 2, 100) != 0 {
        error!("SDL audio channel could not be opened.");
        warn!(
            "SDL Mixer Error: {}. Continuing with sound disabled",
            get_error(),
        );
        SOUND_ON = false.into();
        return;
    } else {
        warn!("Successfully opened SDL audio channel.");
    }

    if Mix_AllocateChannels(20) != 20 {
        warn!("WARNING: could not get all 20 mixer-channels I asked for...");
    }

    // Now that the audio channel is opend, its time to load all the
    // WAV files into memory, something we NEVER did while using the yiff,
    // because the yiff did all the loading, analyzing and playing...

    LOADED_WAV_FILES[0] = null_mut();
    let iter = SOUND_SAMPLE_FILENAMES
        .iter()
        .copied()
        .zip(LOADED_WAV_FILES.iter_mut())
        .skip(1);
    for (sample_filename, loaded_wav_file) in iter {
        let fpath = find_file(
            sample_filename.as_ptr(),
            SOUND_DIR_C.as_ptr() as *mut c_char,
            Themed::NoTheme as c_int,
            Criticality::WarnOnly as c_int,
        );
        if !fpath.is_null() {
            *loaded_wav_file = mix_load_wav(fpath);
        }

        if loaded_wav_file.is_null() {
            error!(
                "Could not load Sound-sample: {}",
                sample_filename.to_string_lossy()
            );
            warn!("Continuing with sound disabled. Error = {}", get_error());
            SOUND_ON = false.into();
            return;
        } else {
            info!(
                "Successfully loaded file {}.",
                sample_filename.to_string_lossy()
            );
        }
    }

    let iter = MUSIC_FILES.iter().copied().zip(MUSIC_SONGS.iter_mut());
    for (music_file, music_song) in iter {
        let fpath = find_file(
            music_file.as_ptr(),
            SOUND_DIR_C.as_ptr() as *mut c_char,
            Themed::NoTheme as c_int,
            Criticality::WarnOnly as c_int,
        );
        if !fpath.is_null() {
            *music_song = Mix_LoadMUS(fpath);
        }
        if music_song.is_null() {
            error!("Error loading sound-file: {}", music_file.to_string_lossy());
            warn!(
                "SDL Mixer Error: {}. Continuing with sound disabled",
                get_error()
            );
            SOUND_ON = false.into();
            return;
        } else {
            info!("Successfully loaded file {}.", music_file.to_string_lossy());
        }
    }

    //--------------------
    // Now that the music files have been loaded successfully, it's time to set
    // the music and sound volumes accoridingly, i.e. as specifies by the users
    // configuration.
    //
    set_sound_f_x_volume(GAME_CONFIG.current_sound_fx_volume);
}
