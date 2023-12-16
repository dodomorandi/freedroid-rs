use crate::{
    defs::{BulletKind, Criticality, SoundType, Themed, BYCOLOR, NUM_COLORS, SOUND_DIR_C},
    global::Global,
    misc::Misc,
    Data, Main, Sdl,
};

use array_init::array_init;
use log::{error, info, warn};
use sdl::{
    mixer::{Chunk, Music, OpenAudio},
    rwops::RwOps,
    Mixer,
};
use std::{
    ffi::CStr,
    os::raw::{c_float, c_int},
};

const MIX_MAX_VOLUME: u8 = 128;

#[inline]
fn mix_load_wav<'a>(mixer: &'a Mixer, file: &CStr) -> Option<Chunk<'a>> {
    use sdl::rwops::{Mode, ReadWriteMode};

    let file = RwOps::from_c_str_path(file, Mode::from(ReadWriteMode::Read))?;
    mixer.load_wav_from_rwops(file)
}

const SOUND_SAMPLE_FILENAMES: [&str; SoundType::All as usize] = [
    "ERRORSOUND_NILL.NOWAV",
    "Blast_Sound_0.wav",
    // "Collision_Sound_0.wav", // replaced by damage-dependent-sounds:  Collision_[Neutral|GotDamaged|DamagedEnemy]
    "Collision_Neutral.wav",
    "Collision_GotDamaged.wav",
    "Collision_DamagedEnemy.wav",
    //"GotIntoBlast_Sound_0.wav", // replaced by GotIntoBlast_Sound_1.wav
    "GotIntoBlast_Sound_1.wav",
    "MoveElevator_Sound_0.wav",
    "Refresh_Sound_0.wav",
    "LeaveElevator_Sound_0.wav",
    "EnterElevator_Sound_0.wav",
    "ThouArtDefeated_Sound_0.wav",
    "Got_Hit_Sound_0.wav",
    "TakeoverSetCapsule_Sound_0.wav",
    "Menu_Item_Selected_Sound_0.wav",
    "Move_Menu_Position_Sound_0.wav",
    "Takeover_Game_Won_Sound_0.wav",
    "Takeover_Game_Deadlock_Sound_0.wav",
    "Takeover_Game_Lost_Sound_0.wav",
    "Fire_Bullet_Pulse_Sound_0.wav",
    "Fire_Bullet_Single_Pulse_Sound_0.wav",
    "Fire_Bullet_Military_Sound_0.wav",
    "Fire_Bullet_Flash_Sound_0.wav",
    "Fire_Bullet_Exterminator_Sound_0.wav",
    "Fire_Bullet_Laser_Rifle_Sound.wav",
    "Cry_Sound_0.wav",
    "Takeover_Sound_0.wav",
    "Countdown_Sound.wav",
    "EndCountdown_Sound.wav",
    "InfluExplosion.wav",
    "WhiteNoise.wav",
    "Alert.wav",
    "Screenshot.wav",
];

const MUSIC_FILES: [&[u8]; NUM_COLORS] = [
    b"AnarchyMenu1.mod",          // RED
    b"starpaws.mod",              // YELLOW
    b"The_Last_V8.mod",           // GREEN
    b"dreamfish-green_beret.mod", // GRAY
    #[cfg(feature = "gcw0")]
    b"dreamfish-green_beret.mod", // GRAY
    #[cfg(not(feature = "gcw0"))]
    b"dreamfish-sanxion.mod", // BLUE // CRASHES the GCW0 ???
    b"kollaps-tron.mod",          // GREENBLUE
    b"dreamfish-uridium2_loader.mod", // DARK
];

#[derive(Debug)]
pub struct Sound<'a> {
    prev_color: c_int,
    paused: bool,
    loaded_wav_files: [Option<Chunk<'a>>; SoundType::All as usize],
    _opened_audio: OpenAudio<'a>,
    music_songs: [Option<Music<'a>>; NUM_COLORS],
    tmp_mod_file: Option<Music<'a>>,
}

impl Data<'_> {
    pub fn cry_sound(&self) {
        self.play_sound(SoundType::Cry as i32);
    }

    pub fn transfer_sound(&self) {
        self.play_sound(SoundType::Transfer as i32);
    }

    #[inline]
    pub fn play_sound(&self, tune: c_int) {
        Self::play_sound_static(
            self.main.sound_on,
            self.sdl,
            self.sound.as_ref().unwrap(),
            tune,
        );
    }

    pub fn play_sound_static(sound_on: c_int, sdl: &Sdl, sound: &Sound, tune: c_int) {
        if sound_on == 0 {
            return;
        }

        let mixer = sdl.mixer.get().unwrap();
        let tune = usize::try_from(tune).unwrap();
        let newest_sound_channel = mixer.play_channel_timed(
            None,
            sound.loaded_wav_files[tune].as_ref().unwrap(),
            Some(0),
            None,
        );
        if newest_sound_channel.is_none() {
            let err = sdl.get_error();
            warn!(
                "Could not play sound-sample: {} Error: {}.\
                 This usually just means that too many samples where played at the same time",
                SOUND_SAMPLE_FILENAMES[tune],
                err.to_string_lossy(),
            );
        } else {
            info!(
                "Successfully playing file {}.",
                SOUND_SAMPLE_FILENAMES[tune]
            );
        };
    }

    pub fn takeover_set_capsule_sound(&self) {
        if self.main.sound_on == 0 {
            return;
        }

        self.play_sound(SoundType::TakeoverSetCapsule as i32);
    }

    pub fn takeover_game_won_sound(&self) {
        if self.main.sound_on == 0 {
            return;
        }

        self.play_sound(SoundType::TakeoverGameWon as i32);
    }

    pub fn takeover_game_deadlock_sound(&self) {
        if self.main.sound_on == 0 {
            return;
        }

        self.play_sound(SoundType::TakeoverGameDeadlock as i32);
    }

    pub fn takeover_game_lost_sound(&self) {
        if self.main.sound_on == 0 {
            return;
        }

        self.play_sound(SoundType::TakeoverGameLost as i32);
    }

    pub fn collision_got_damaged_sound(&self) {
        if self.main.sound_on == 0 {
            return;
        }

        self.play_sound(SoundType::CollisionGotDamaged as i32);
    }

    pub fn collision_damaged_enemy_sound(&self) {
        if self.main.sound_on == 0 {
            return;
        }

        self.play_sound(SoundType::CollisionDamagedEnemy as i32);
    }

    pub fn bounce_sound(&self) {
        if self.main.sound_on == 0 {
            return;
        }

        self.play_sound(SoundType::Collision as i32);
    }

    pub fn druid_blast_sound(&self) {
        if self.main.sound_on == 0 {
            return;
        }

        self.play_sound(SoundType::Blast as i32);
    }

    pub fn got_hit_sound(&self) {
        if self.main.sound_on == 0 {
            return;
        }

        self.play_sound(SoundType::GotHit as i32);
    }

    pub fn got_into_blast_sound(&self) {
        if self.main.sound_on == 0 {
            return;
        }

        self.play_sound(SoundType::GotIntoBlast as i32);
    }

    pub fn refresh_sound(&self) {
        if self.main.sound_on == 0 {
            return;
        }

        self.play_sound(SoundType::Refresh as i32);
    }

    #[inline]
    pub fn move_lift_sound(&self) {
        Self::move_lift_sound_static(self.main.sound_on, self.sdl, self.sound.as_ref().unwrap());
    }

    pub fn move_lift_sound_static(sound_on: c_int, sdl: &Sdl, sound: &Sound) {
        if sound_on == 0 {
            return;
        }

        Self::play_sound_static(sound_on, sdl, sound, SoundType::MoveElevator as i32);
    }

    pub fn menu_item_selected_sound(&self) {
        if self.main.sound_on == 0 {
            return;
        }

        self.play_sound(SoundType::MenuItemSelected as i32);
    }

    pub fn move_menu_position_sound(&self) {
        if self.main.sound_on == 0 {
            return;
        }

        self.play_sound(SoundType::MoveMenuPosition as i32);
    }

    pub fn thou_art_defeated_sound(&self) {
        if self.main.sound_on == 0 {
            return;
        }

        self.play_sound(SoundType::ThouArtDefeated as i32);
    }

    pub fn enter_lift_sound(&self) {
        if self.main.sound_on == 0 {
            return;
        }

        self.play_sound(SoundType::EnterElevator as i32);
    }

    pub fn leave_lift_sound(&self) {
        if self.main.sound_on == 0 {
            return;
        }

        self.play_sound(SoundType::LeaveElevator as i32);
    }
}

impl<'sdl> Data<'sdl> {
    pub fn fire_bullet_sound(&self, bullet_type: c_int) {
        use BulletKind as K;

        if self.main.sound_on == 0 {
            return;
        }

        let bullet_type = match bullet_type {
            0 => K::Pulse,
            1 => K::SinglePulse,
            2 => K::Military,
            3 => K::Flash,
            4 => K::Exterminator,
            5 => K::LaserRifle,
            _ => panic!("invalid bullet type {bullet_type}"),
        };

        match bullet_type {
            K::Pulse => self.play_sound(SoundType::FireBulletPulse as i32),
            K::SinglePulse => self.play_sound(SoundType::FireBulletSinglePulse as i32),
            K::Military => self.play_sound(SoundType::FireBulletMilitary as i32),
            K::Flash => self.play_sound(SoundType::FireBulletFlash as i32),
            K::Exterminator => self.play_sound(SoundType::FireBulletExterminator as i32),
            K::LaserRifle => self.play_sound(SoundType::FireBulletLaserRifle as i32),
        }
    }

    pub fn switch_background_music_to(&mut self, filename_raw: Option<&[u8]>) {
        let Self {
            sdl,
            sound,
            misc,
            global,
            main,
            ..
        } = self;

        let sound = sound.as_mut().unwrap();
        Self::switch_background_music_to_static(sound, main, global, misc, sdl, filename_raw);
    }

    pub fn switch_background_music_to_static<'a>(
        sound: &mut Sound<'a>,
        main: &Main,
        global: &Global,
        misc: &mut Misc,
        sdl: &'a Sdl,
        filename_raw: Option<&[u8]>,
    ) {
        if main.sound_on == 0 {
            return;
        }

        let mixer = sdl.mixer.get().unwrap();
        let Some(filename_raw) = filename_raw else {
            mixer.pause_music();
            sound.paused = true;
            return;
        };

        // New feature: choose background music by level-color:
        // if filename_raw==BYCOLOR then chose bg_music[color]
        // NOTE: if new level-color is the same as before, just resume paused music!
        if filename_raw == BYCOLOR {
            if sound.paused && sound.prev_color == main.cur_level().color {
                // current level-song was just paused
                mixer.resume_music();
                sound.paused = false;
            } else {
                mixer.play_music(
                    sound.music_songs[usize::try_from(main.cur_level().color).unwrap()]
                        .as_ref()
                        .unwrap(),
                    None,
                );
                sound.paused = false;
                sound.prev_color = main.cur_level().color;
            }
        } else {
            // not using BYCOLOR mechanism: just play specified song
            let Some(fpath) = Self::find_file_static(
                global,
                misc,
                filename_raw,
                Some(SOUND_DIR_C),
                Themed::NoTheme as c_int,
                Criticality::WarnOnly as c_int,
            ) else {
                error!(
                    "Error loading sound-file: {}",
                    String::from_utf8_lossy(filename_raw)
                );
                return;
            };

            let mixer = sdl.mixer.get().unwrap();

            sound.tmp_mod_file = mixer.load_music_from_c_str_path(fpath);
            let Some(music) = sound.tmp_mod_file.as_ref() else {
                error!(
                    "SDL Mixer Error: {}. Continuing with sound disabled",
                    sdl.get_error().to_string_lossy(),
                );
                return;
            };
            mixer.play_music(music, None);
        }

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        {
            sdl.mixer.get().unwrap().replace_music_volume(Some(
                (global.game_config.current_bg_music_volume * f32::from(MIX_MAX_VOLUME)) as u32,
            ));
        }
    }

    pub fn countdown_sound(&self) {
        self.play_sound(SoundType::Countdown as i32);
    }

    pub fn end_countdown_sound(&self) {
        self.play_sound(SoundType::Endcountdown as i32);
    }

    pub fn set_bg_music_volume(&self, new_volume: c_float) {
        if self.main.sound_on == 0 {
            return;
        }

        let mixer = self.sdl.mixer.get().unwrap();
        let new_volume = (new_volume >= 0.).then(
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            || (new_volume * f32::from(MIX_MAX_VOLUME)) as u32,
        );
        mixer.replace_music_volume(new_volume);
    }
}

impl<'a> Sound<'a> {
    pub(crate) fn new(
        main: &mut Main,
        sdl: &'a Sdl,
        global: &Global,
        misc: &mut Misc,
    ) -> Option<Self> {
        info!("Initializing SDL Audio Systems");

        if main.sound_on == 0 {
            return None;
        }

        // Now SDL_AUDIO is initialized here:

        let Some(mixer) = sdl.init_audio() else {
            warn!(
                "SDL Sound subsystem could not be initialized. \
                Continuing with sound disabled",
            );
            main.sound_on = false.into();
            return None;
        };
        info!("SDL Audio initialisation successful.");

        // Now that we have initialized the audio SubSystem, we must open
        // an audio channel.  This will be done here (see code from Mixer-Tutorial):
        let Some(opened_audio) = mixer.open_audio().channels(2).open(100) else {
            error!("SDL audio channel could not be opened.");
            warn!(
                "SDL Mixer Error: {}. Continuing with sound disabled",
                sdl.get_error().to_string_lossy(),
            );
            main.sound_on = false.into();
            return None;
        };
        info!("Successfully opened SDL audio channel.");

        if mixer.allocate_channels(20) != Some(20) {
            warn!("WARNING: could not get all 20 mixer-channels I asked for...");
        }

        // Now that the audio channel is opend, its time to load all the
        // WAV files into memory, something we NEVER did while using the yiff,
        // because the yiff did all the loading, analyzing and playing...

        let mut loaded_wav_files: [_; SoundType::All as usize] = array_init(|_| None);

        let iter = SOUND_SAMPLE_FILENAMES.iter().copied().enumerate().skip(1);
        for (sound_file_index, sample_filename) in iter {
            let fpath = Data::find_file_static(
                global,
                misc,
                sample_filename.as_bytes(),
                Some(SOUND_DIR_C),
                Themed::NoTheme as c_int,
                Criticality::WarnOnly as c_int,
            );

            let loaded_wav_file = &mut loaded_wav_files[sound_file_index];

            if let Some(fpath) = fpath {
                *loaded_wav_file = mix_load_wav(mixer, fpath);
            }

            if loaded_wav_file.is_none() {
                error!("Could not load Sound-sample: {}", sample_filename);
                warn!(
                    "Continuing with sound disabled. Error = {}",
                    sdl.get_error().to_string_lossy()
                );
                main.sound_on = false.into();
                return None;
            }

            info!("Successfully loaded file {}.", sample_filename);
        }

        let mut music_songs: [_; NUM_COLORS] = array_init(|_| None);
        let iter = MUSIC_FILES.iter().copied().enumerate();
        for (music_file_index, music_file) in iter {
            let fpath = Data::find_file_static(
                global,
                misc,
                music_file,
                Some(SOUND_DIR_C),
                Themed::NoTheme as c_int,
                Criticality::WarnOnly as c_int,
            );
            let music_song = &mut music_songs[music_file_index];
            if let Some(fpath) = fpath {
                *music_song = mixer.load_music_from_c_str_path(fpath);
            }
            if music_song.is_none() {
                error!(
                    "Error loading sound-file: {}",
                    String::from_utf8_lossy(music_file)
                );
                warn!(
                    "SDL Mixer Error: {}. Continuing with sound disabled",
                    sdl.get_error().to_string_lossy()
                );
                main.sound_on = false.into();
                return None;
            }

            info!(
                "Successfully loaded file {}.",
                String::from_utf8_lossy(music_file)
            );
        }

        let sound = Self {
            prev_color: -1,
            paused: false,
            loaded_wav_files,
            _opened_audio: opened_audio,
            music_songs,
            tmp_mod_file: None,
        };

        //--------------------
        // Now that the music files have been loaded successfully, it's time to set
        // the music and sound volumes accoridingly, i.e. as specifies by the users
        // configuration.
        //
        let mixer = sdl.mixer.get().unwrap();
        sound.set_sound_f_x_volume(main, mixer, global.game_config.current_sound_fx_volume);

        Some(sound)
    }

    pub(crate) fn set_sound_f_x_volume(&self, main: &Main, mixer: &Mixer, new_volume: c_float) {
        if main.sound_on == 0 {
            return;
        }

        // Set the volume IN the loaded files, if SDL is used...
        // This is done here for the Files 1,2,3 and 4, since these
        // are background music files.
        self.loaded_wav_files.iter().skip(1).for_each(|file| {
            mixer.replace_chunk_volume(
                file.as_ref().unwrap(),
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                Some((new_volume * f32::from(MIX_MAX_VOLUME)) as u32),
            );
        });
    }
}
