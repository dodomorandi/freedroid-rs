use std::{ffi::CStr, marker::PhantomData, os::raw::c_int, ptr::NonNull};

use sdl_sys::{
    Mix_AllocateChannels, Mix_Chunk, Mix_CloseAudio, Mix_FreeChunk, Mix_FreeMusic, Mix_HaltMusic,
    Mix_LoadMUS, Mix_LoadWAV_RW, Mix_Music, Mix_OpenAudio, Mix_PauseMusic, Mix_PlayChannelTimed,
    Mix_PlayMusic, Mix_ResumeMusic, Mix_VolumeChunk, Mix_VolumeMusic, MIX_DEFAULT_CHANNELS,
    MIX_DEFAULT_FORMAT, MIX_DEFAULT_FREQUENCY,
};

use crate::rwops::RwOps;

#[derive(Debug)]
pub struct Mixer;

impl Mixer {
    #[must_use]
    pub fn open_audio(&self) -> OpenAudioBuilder {
        OpenAudioBuilder::default()
    }

    #[must_use]
    pub fn allocate_channels(&self, channels: u32) -> Option<u32> {
        unsafe {
            Mix_AllocateChannels(channels.try_into().unwrap())
                .try_into()
                .ok()
        }
    }

    #[must_use]
    pub fn load_music_from_c_str_path<'a>(&'a self, path: &CStr) -> Option<Music<'a>> {
        let music_ptr = unsafe { Mix_LoadMUS(path.as_ptr()) };
        NonNull::new(music_ptr).map(|inner| Music {
            inner,
            _marker: PhantomData,
        })
    }

    #[must_use]
    pub fn load_wav_from_rwops(&self, rwops: RwOps) -> Option<Chunk> {
        let ret = unsafe { Mix_LoadWAV_RW(rwops.into_inner().as_ptr(), 1) };
        NonNull::new(ret).map(|inner| Chunk {
            inner,
            _marker: PhantomData,
        })
    }

    pub fn pause_music(&self) {
        unsafe { Mix_PauseMusic() }
    }

    pub fn resume_music(&self) {
        unsafe { Mix_ResumeMusic() }
    }

    pub fn play_music(&self, music: &Music, loops: Option<u32>) -> bool {
        let ret = unsafe {
            Mix_PlayMusic(
                music.inner.as_ptr(),
                loops.map(|loops| loops.try_into().unwrap()).unwrap_or(-1),
            )
        };
        ret == 0
    }

    pub fn replace_music_volume(&self, volume: Option<u32>) -> Option<u32> {
        let volume = volume
            .map(|volume| volume.try_into().unwrap())
            .unwrap_or(-1);
        let old_volume = unsafe { Mix_VolumeMusic(volume) };
        old_volume.try_into().ok()
    }

    pub fn replace_chunk_volume(&self, chunk: &Chunk, volume: Option<u32>) -> Option<u32> {
        let volume = volume
            .map(|volume| volume.try_into().unwrap())
            .unwrap_or(-1);
        let old_volume = unsafe { Mix_VolumeChunk(chunk.inner.as_ptr(), volume) };
        old_volume.try_into().ok()
    }

    pub fn play_channel_timed(
        &self,
        channel: Option<u8>,
        chunk: &Chunk,
        loops: Option<u32>,
        ticks: Option<u32>,
    ) -> Option<u32> {
        let channel = channel.map(i32::from).unwrap_or(-1);
        let loops = loops.map(|loops| loops.try_into().unwrap()).unwrap_or(-1);
        let ticks = ticks.map(|ticks| ticks.try_into().unwrap()).unwrap_or(-1);
        let channel = unsafe { Mix_PlayChannelTimed(channel, chunk.inner.as_ptr(), loops, ticks) };

        channel.try_into().ok()
    }

    pub fn halt_music(&self) {
        // TODO: Checks whether the abstraction related to callbacks set by `Mix_HookMusicFinished` is safe.
        unsafe {
            // Always returns zero.
            Mix_HaltMusic();
        }
    }
}

#[derive(Debug, Clone)]
pub struct OpenAudioBuilder<'a> {
    frequency: u32,
    format: u16,
    channels: u8,
    _marker: PhantomData<&'a ()>,
}

impl Default for OpenAudioBuilder<'_> {
    fn default() -> Self {
        Self {
            frequency: MIX_DEFAULT_FREQUENCY,
            format: MIX_DEFAULT_FORMAT,
            channels: MIX_DEFAULT_CHANNELS,
            _marker: PhantomData,
        }
    }
}

impl<'a> OpenAudioBuilder<'a> {
    #[must_use]
    pub fn frequency(&mut self, frequency: u32) -> &mut Self {
        self.frequency = frequency;
        self
    }

    #[must_use]
    pub fn channels(&mut self, channels: u8) -> &mut Self {
        self.channels = channels;
        self
    }

    #[must_use]
    pub fn open(&self, chunk_size: c_int) -> Option<OpenAudio<'a>> {
        unsafe {
            (Mix_OpenAudio(
                self.frequency.try_into().unwrap(),
                self.format,
                self.channels.into(),
                chunk_size,
            ) == 0)
                .then(|| OpenAudio(PhantomData))
        }
    }
}

#[derive(Debug)]
pub struct OpenAudio<'a>(PhantomData<&'a ()>);

impl Drop for OpenAudio<'_> {
    fn drop(&mut self) {
        unsafe { Mix_CloseAudio() }
    }
}

#[derive(Debug)]
pub struct Music<'a> {
    inner: NonNull<Mix_Music>,
    _marker: PhantomData<&'a ()>,
}

impl Drop for Music<'_> {
    fn drop(&mut self) {
        unsafe {
            Mix_FreeMusic(self.inner.as_ptr());
        }
    }
}

#[derive(Debug)]
pub struct Chunk<'a> {
    inner: NonNull<Mix_Chunk>,
    _marker: PhantomData<&'a ()>,
}

impl Drop for Chunk<'_> {
    fn drop(&mut self) {
        unsafe {
            Mix_FreeChunk(self.inner.as_ptr());
        }
    }
}
