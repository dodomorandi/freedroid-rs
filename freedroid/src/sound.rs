use crate::{
    defs::{BulletKind, Criticality, SoundType, Themed, BYCOLOR, NUM_COLORS, SOUND_DIR_C},
    Data,
};

use cstr::cstr;
use log::{error, info, warn};
use sdl_sys::{
    Mix_AllocateChannels, Mix_Chunk, Mix_CloseAudio, Mix_FreeChunk, Mix_FreeMusic, Mix_LoadMUS,
    Mix_LoadWAV_RW, Mix_Music, Mix_OpenAudio, Mix_PauseMusic, Mix_PlayChannelTimed, Mix_PlayMusic,
    Mix_ResumeMusic, Mix_VolumeChunk, Mix_VolumeMusic, SDL_CloseAudio, SDL_GetError,
    SDL_InitSubSystem, SDL_RWFromFile, SDL_INIT_AUDIO,
};
use std::{
    convert::TryFrom,
    ffi::CStr,
    os::raw::{c_char, c_float, c_int},
    ptr::null_mut,
};

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

const SOUND_SAMPLE_FILENAMES: [&CStr; SoundType::All as usize] = [
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

#[derive(Debug)]
pub struct Sound {
    prev_color: c_int,
    paused: bool,
    loaded_wav_files: [*mut Mix_Chunk; SoundType::All as usize],
    music_songs: [*mut Mix_Music; NUM_COLORS],
    tmp_mod_file: *mut Mix_Music,
}

impl Default for Sound {
    fn default() -> Self {
        Self {
            prev_color: -1,
            paused: false,
            loaded_wav_files: [null_mut(); SoundType::All as usize],
            music_songs: [null_mut(); NUM_COLORS],
            tmp_mod_file: null_mut(),
        }
    }
}

#[inline]
unsafe fn mix_play_channel(channel: c_int, chunk: *mut Mix_Chunk, loops: c_int) -> c_int {
    Mix_PlayChannelTimed(channel, chunk, loops, -1)
}

impl Data {
    pub unsafe fn cry_sound(&self) {
        self.play_sound(SoundType::Cry as i32);
    }

    pub unsafe fn transfer_sound(&self) {
        self.play_sound(SoundType::Transfer as i32);
    }

    pub unsafe fn play_sound(&self, tune: c_int) {
        if self.main.sound_on == 0 {
            return;
        }

        let tune = usize::try_from(tune).unwrap();
        let newest_sound_channel = mix_play_channel(-1, self.sound.loaded_wav_files[tune], 0);
        if newest_sound_channel == -1 {
            warn!(
                "Could not play sound-sample: {} Error: {}.\
             This usually just means that too many samples where played at the same time",
                SOUND_SAMPLE_FILENAMES[tune].to_string_lossy(),
                CStr::from_ptr(SDL_GetError()).to_string_lossy(),
            );
        } else {
            info!(
                "Successfully playing file {}.",
                SOUND_SAMPLE_FILENAMES[tune].to_string_lossy()
            );
        }
    }

    pub unsafe fn free_sounds(&self) {
        self.sound
            .loaded_wav_files
            .iter()
            .filter(|file| !file.is_null())
            .for_each(|&file| Mix_FreeChunk(file));

        self.sound
            .music_songs
            .iter()
            .filter(|song| !song.is_null())
            .for_each(|&song| Mix_FreeMusic(song));

        if !self.sound.tmp_mod_file.is_null() {
            Mix_FreeMusic(self.sound.tmp_mod_file);
        }

        Mix_CloseAudio();
        SDL_CloseAudio();
    }

    pub unsafe fn takeover_set_capsule_sound(&self) {
        if self.main.sound_on == 0 {
            return;
        }

        self.play_sound(SoundType::TakeoverSetCapsule as i32);
    }

    pub unsafe fn takeover_game_won_sound(&self) {
        if self.main.sound_on == 0 {
            return;
        }

        self.play_sound(SoundType::TakeoverGameWon as i32);
    }

    pub unsafe fn takeover_game_deadlock_sound(&self) {
        if self.main.sound_on == 0 {
            return;
        }

        self.play_sound(SoundType::TakeoverGameDeadlock as i32);
    }

    pub unsafe fn takeover_game_lost_sound(&self) {
        if self.main.sound_on == 0 {
            return;
        }

        self.play_sound(SoundType::TakeoverGameLost as i32);
    }

    pub unsafe fn collision_got_damaged_sound(&self) {
        if self.main.sound_on == 0 {
            return;
        }

        self.play_sound(SoundType::CollisionGotDamaged as i32);
    }

    pub unsafe fn collision_damaged_enemy_sound(&self) {
        if self.main.sound_on == 0 {
            return;
        }

        self.play_sound(SoundType::CollisionDamagedEnemy as i32);
    }

    pub unsafe fn bounce_sound(&self) {
        if self.main.sound_on == 0 {
            return;
        }

        self.play_sound(SoundType::Collision as i32);
    }

    pub unsafe fn druid_blast_sound(&self) {
        if self.main.sound_on == 0 {
            return;
        }

        self.play_sound(SoundType::Blast as i32);
    }

    pub unsafe fn got_hit_sound(&self) {
        if self.main.sound_on == 0 {
            return;
        }

        self.play_sound(SoundType::GotHit as i32);
    }

    pub unsafe fn got_into_blast_sound(&self) {
        if self.main.sound_on == 0 {
            return;
        }

        self.play_sound(SoundType::GotIntoBlast as i32);
    }

    pub unsafe fn refresh_sound(&self) {
        if self.main.sound_on == 0 {
            return;
        }

        self.play_sound(SoundType::Refresh as i32);
    }

    pub unsafe fn move_lift_sound(&self) {
        if self.main.sound_on == 0 {
            return;
        }

        self.play_sound(SoundType::MoveElevator as i32);
    }

    pub unsafe fn menu_item_selected_sound(&self) {
        if self.main.sound_on == 0 {
            return;
        }

        self.play_sound(SoundType::MenuItemSelected as i32);
    }

    pub unsafe fn move_menu_position_sound(&self) {
        if self.main.sound_on == 0 {
            return;
        }

        self.play_sound(SoundType::MoveMenuPosition as i32);
    }

    pub unsafe fn thou_art_defeated_sound(&self) {
        if self.main.sound_on == 0 {
            return;
        }

        self.play_sound(SoundType::ThouArtDefeated as i32);
    }

    pub unsafe fn enter_lift_sound(&self) {
        if self.main.sound_on == 0 {
            return;
        }

        self.play_sound(SoundType::EnterElevator as i32);
    }

    pub unsafe fn leave_lift_sound(&self) {
        if self.main.sound_on == 0 {
            return;
        }

        self.play_sound(SoundType::LeaveElevator as i32);
    }
}

impl Data {
    pub unsafe fn fire_bullet_sound(&self, bullet_type: c_int) {
        if self.main.sound_on == 0 {
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
            Pulse => self.play_sound(SoundType::FireBulletPulse as i32),
            SinglePulse => self.play_sound(SoundType::FireBulletSinglePulse as i32),
            Military => self.play_sound(SoundType::FireBulletMilitary as i32),
            Flash => self.play_sound(SoundType::FireBulletFlash as i32),
            Exterminator => self.play_sound(SoundType::FireBulletExterminator as i32),
            LaserRifle => self.play_sound(SoundType::FireBulletLaserRifle as i32),
        }
    }

    pub unsafe fn switch_background_music_to(&mut self, filename_raw: *const c_char) {
        if self.main.sound_on == 0 {
            return;
        }

        if filename_raw.is_null() {
            Mix_PauseMusic(); // pause currently played background music
            self.sound.paused = true;
            return;
        }

        let filename_raw = CStr::from_ptr(filename_raw);

        // New feature: choose background music by level-color:
        // if filename_raw==BYCOLOR then chose bg_music[color]
        // NOTE: if new level-color is the same as before, just resume paused music!
        if filename_raw.to_bytes() == BYCOLOR.to_bytes() {
            if self.sound.paused && self.sound.prev_color == (*self.main.cur_level).color {
                // current level-song was just paused
                Mix_ResumeMusic();
                self.sound.paused = false;
            } else {
                Mix_PlayMusic(
                    self.sound.music_songs[usize::try_from((*self.main.cur_level).color).unwrap()],
                    -1,
                );
                self.sound.paused = false;
                self.sound.prev_color = (*self.main.cur_level).color;
            }
        } else {
            // not using BYCOLOR mechanism: just play specified song
            if !self.sound.tmp_mod_file.is_null() {
                Mix_FreeMusic(self.sound.tmp_mod_file);
            }
            let fpath = self.find_file(
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
            self.sound.tmp_mod_file = Mix_LoadMUS(fpath);
            if self.sound.tmp_mod_file.is_null() {
                error!(
                    "SDL Mixer Error: {}. Continuing with sound disabled",
                    CStr::from_ptr(SDL_GetError()).to_string_lossy(),
                );
                return;
            }
            Mix_PlayMusic(self.sound.tmp_mod_file, -1);
        }

        Mix_VolumeMusic(
            (self.global.game_config.current_bg_music_volume * f32::from(MIX_MAX_VOLUME)) as c_int,
        );
    }

    pub unsafe fn countdown_sound(&self) {
        self.play_sound(SoundType::Countdown as i32);
    }

    pub unsafe fn end_countdown_sound(&self) {
        self.play_sound(SoundType::Endcountdown as i32);
    }

    pub unsafe fn set_sound_f_x_volume(&self, new_volume: c_float) {
        if self.main.sound_on == 0 {
            return;
        }

        // Set the volume IN the loaded files, if SDL is used...
        // This is done here for the Files 1,2,3 and 4, since these
        // are background music files.
        self.sound
            .loaded_wav_files
            .iter()
            .skip(1)
            .for_each(|&file| {
                Mix_VolumeChunk(file, (new_volume * f32::from(MIX_MAX_VOLUME)) as c_int);
            });
    }

    pub unsafe fn set_bg_music_volume(&self, new_volume: c_float) {
        if self.main.sound_on == 0 {
            return;
        }

        Mix_VolumeMusic((new_volume * f32::from(MIX_MAX_VOLUME)) as c_int);
    }

    pub unsafe fn init_audio(&mut self) {
        info!("Initializing SDL Audio Systems");

        if self.main.sound_on == 0 {
            return;
        }

        // Now SDL_AUDIO is initialized here:

        if SDL_InitSubSystem(SDL_INIT_AUDIO as u32) == -1 {
            warn!(
                "SDL Sound subsystem could not be initialized. \
             Continuing with sound disabled",
            );
            self.main.sound_on = false.into();
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
                CStr::from_ptr(SDL_GetError()).to_string_lossy(),
            );
            self.main.sound_on = false.into();
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

        self.sound.loaded_wav_files[0] = null_mut();
        let iter = SOUND_SAMPLE_FILENAMES.iter().copied().enumerate().skip(1);
        for (sound_file_index, sample_filename) in iter {
            let fpath = self.find_file(
                sample_filename.as_ptr(),
                SOUND_DIR_C.as_ptr() as *mut c_char,
                Themed::NoTheme as c_int,
                Criticality::WarnOnly as c_int,
            );

            let loaded_wav_file = &mut self.sound.loaded_wav_files[sound_file_index];

            if !fpath.is_null() {
                *loaded_wav_file = mix_load_wav(fpath);
            }

            if loaded_wav_file.is_null() {
                error!(
                    "Could not load Sound-sample: {}",
                    sample_filename.to_string_lossy()
                );
                warn!(
                    "Continuing with sound disabled. Error = {}",
                    CStr::from_ptr(SDL_GetError()).to_string_lossy()
                );
                self.main.sound_on = false.into();
                return;
            } else {
                info!(
                    "Successfully loaded file {}.",
                    sample_filename.to_string_lossy()
                );
            }
        }

        let iter = MUSIC_FILES.iter().copied().enumerate();
        for (music_file_index, music_file) in iter {
            let fpath = self.find_file(
                music_file.as_ptr(),
                SOUND_DIR_C.as_ptr() as *mut c_char,
                Themed::NoTheme as c_int,
                Criticality::WarnOnly as c_int,
            );
            let music_song = &mut self.sound.music_songs[music_file_index];
            if !fpath.is_null() {
                *music_song = Mix_LoadMUS(fpath);
            }
            if music_song.is_null() {
                error!("Error loading sound-file: {}", music_file.to_string_lossy());
                warn!(
                    "SDL Mixer Error: {}. Continuing with sound disabled",
                    CStr::from_ptr(SDL_GetError()).to_string_lossy()
                );
                self.main.sound_on = false.into();
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
        self.set_sound_f_x_volume(self.global.game_config.current_sound_fx_volume);
    }
}
