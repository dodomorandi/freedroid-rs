use crate::{
    defs::{BulletKind, Criticality, SoundType, Themed, BYCOLOR, NUM_COLORS, SOUND_DIR_C},
    global::Global,
    misc::Misc,
    Data, Main, Sdl,
};

use array_init::array_init;
use cstr::cstr;
use log::{error, info, warn};
use sdl::{
    mixer::{Chunk, Music, OpenAudio},
    rwops::RwOps,
    Mixer,
};
use std::{
    ffi::CStr,
    os::raw::{c_char, c_float, c_int},
};

const MIX_MAX_VOLUME: u8 = 128;

#[inline]
fn mix_load_wav<'a>(mixer: &'a Mixer, file: &CStr) -> Option<Chunk<'a>> {
    use sdl::rwops::{Mode, ReadWriteMode};

    let file = RwOps::from_c_str_path(file, Mode::from(ReadWriteMode::Read))?;
    mixer.load_wav_from_rwops(file)
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
pub struct Sound<'a> {
    prev_color: c_int,
    paused: bool,
    loaded_wav_files: [Option<Chunk<'a>>; SoundType::All as usize],
    _opened_audio: OpenAudio<'a>,
    music_songs: [Option<Music<'a>>; NUM_COLORS],
    tmp_mod_file: Option<Music<'a>>,
}

impl Data<'_> {
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

        let mixer = self.sdl.mixer.get().unwrap();
        let sound = self.sound.as_ref().unwrap();
        let tune = usize::try_from(tune).unwrap();
        let newest_sound_channel = mixer.play_channel_timed(
            None,
            sound.loaded_wav_files[tune].as_ref().unwrap(),
            Some(0),
            None,
        );
        if newest_sound_channel.is_none() {
            sdl::get_error(|err| {
                warn!(
                    "Could not play sound-sample: {} Error: {}.\
This usually just means that too many samples where played at the same time",
                    SOUND_SAMPLE_FILENAMES[tune].to_string_lossy(),
                    err.to_string_lossy(),
                );
            });
        } else {
            info!(
                "Successfully playing file {}.",
                SOUND_SAMPLE_FILENAMES[tune].to_string_lossy()
            );
        };
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

impl<'sdl> Data<'sdl> {
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

        let Self {
            sdl, sound, main, ..
        } = self;

        let mixer = sdl.mixer.get().unwrap();
        let sound = sound.as_mut().unwrap();
        if filename_raw.is_null() {
            mixer.pause_music();
            sound.paused = true;
            return;
        }

        let filename_raw = CStr::from_ptr(filename_raw);

        // New feature: choose background music by level-color:
        // if filename_raw==BYCOLOR then chose bg_music[color]
        // NOTE: if new level-color is the same as before, just resume paused music!
        if filename_raw.to_bytes() == BYCOLOR.to_bytes() {
            if sound.paused && sound.prev_color == (*main.cur_level).color {
                // current level-song was just paused
                mixer.resume_music();
                sound.paused = false;
            } else {
                mixer.play_music(
                    sound.music_songs[usize::try_from((*main.cur_level).color).unwrap()]
                        .as_ref()
                        .unwrap(),
                    None,
                );
                sound.paused = false;
                sound.prev_color = (*main.cur_level).color;
            }
        } else {
            // not using BYCOLOR mechanism: just play specified song
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

            let &mut Self {
                sdl, ref mut sound, ..
            } = self;

            let mixer = sdl.mixer.get().unwrap();
            let sound = sound.as_mut().unwrap();

            sound.tmp_mod_file = mixer.load_music_from_c_str_path(CStr::from_ptr(fpath));
            match sound.tmp_mod_file.as_ref() {
                Some(music) => {
                    mixer.play_music(music, None);
                }
                None => {
                    error!(
                        "SDL Mixer Error: {}. Continuing with sound disabled",
                        sdl.get_error().to_string_lossy(),
                    );
                    return;
                }
            };
        }

        self.sdl.mixer.get().unwrap().replace_music_volume(Some(
            (self.global.game_config.current_bg_music_volume * f32::from(MIX_MAX_VOLUME)) as u32,
        ));
    }

    pub unsafe fn countdown_sound(&self) {
        self.play_sound(SoundType::Countdown as i32);
    }

    pub unsafe fn end_countdown_sound(&self) {
        self.play_sound(SoundType::Endcountdown as i32);
    }

    pub unsafe fn set_bg_music_volume(&self, new_volume: c_float) {
        if self.main.sound_on == 0 {
            return;
        }

        let mixer = self.sdl.mixer.get().unwrap();
        let new_volume =
            (new_volume >= 0.).then(|| (new_volume * f32::from(MIX_MAX_VOLUME)) as u32);
        mixer.replace_music_volume(new_volume);
    }
}

impl<'a> Sound<'a> {
    pub(crate) unsafe fn new(
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

        let mixer = match sdl.init_audio() {
            Some(audio) => audio,
            None => {
                warn!(
                    "SDL Sound subsystem could not be initialized. \
Continuing with sound disabled",
                );
                main.sound_on = false.into();
                return None;
            }
        };
        info!("SDL Audio initialisation successful.");

        // Now that we have initialized the audio SubSystem, we must open
        // an audio channel.  This will be done here (see code from Mixer-Tutorial):
        let opened_audio = match mixer.open_audio().channels(2).open(100) {
            Some(open_audio) => open_audio,
            None => {
                error!("SDL audio channel could not be opened.");
                warn!(
                    "SDL Mixer Error: {}. Continuing with sound disabled",
                    sdl.get_error().to_string_lossy(),
                );
                main.sound_on = false.into();
                return None;
            }
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
                sample_filename.as_ptr(),
                SOUND_DIR_C.as_ptr() as *mut c_char,
                Themed::NoTheme as c_int,
                Criticality::WarnOnly as c_int,
            );

            let loaded_wav_file = &mut loaded_wav_files[sound_file_index];

            if !fpath.is_null() {
                *loaded_wav_file = mix_load_wav(mixer, CStr::from_ptr(fpath));
            }

            if loaded_wav_file.is_none() {
                error!(
                    "Could not load Sound-sample: {}",
                    sample_filename.to_string_lossy()
                );
                warn!(
                    "Continuing with sound disabled. Error = {}",
                    sdl.get_error().to_string_lossy()
                );
                main.sound_on = false.into();
                return None;
            } else {
                info!(
                    "Successfully loaded file {}.",
                    sample_filename.to_string_lossy()
                );
            }
        }

        let mut music_songs: [_; NUM_COLORS] = array_init(|_| None);
        let iter = MUSIC_FILES.iter().copied().enumerate();
        for (music_file_index, music_file) in iter {
            let fpath = Data::find_file_static(
                global,
                misc,
                music_file.as_ptr(),
                SOUND_DIR_C.as_ptr() as *mut c_char,
                Themed::NoTheme as c_int,
                Criticality::WarnOnly as c_int,
            );
            let music_song = &mut music_songs[music_file_index];
            if !fpath.is_null() {
                *music_song = mixer.load_music_from_c_str_path(CStr::from_ptr(fpath));
            }
            if music_song.is_none() {
                error!("Error loading sound-file: {}", music_file.to_string_lossy());
                warn!(
                    "SDL Mixer Error: {}. Continuing with sound disabled",
                    sdl.get_error().to_string_lossy()
                );
                main.sound_on = false.into();
                return None;
            } else {
                info!("Successfully loaded file {}.", music_file.to_string_lossy());
            }
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
                Some((new_volume * f32::from(MIX_MAX_VOLUME)) as u32),
            );
        });
    }
}
