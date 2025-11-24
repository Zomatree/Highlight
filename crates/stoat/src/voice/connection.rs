use std::sync::Arc;

use crate::{
    AudioSource, Error, GlobalCache,
    voice::{CHANNELS, FRAME_LENGTH_MS, FRAME_SIZE, SAMPLE_RATE},
};
use futures::lock::Mutex;
use livekit::{
    Room, RoomEvent, RoomOptions,
    options::TrackPublishOptions,
    track::{LocalAudioTrack, LocalTrack, TrackSource},
    webrtc::{
        audio_source::native::NativeAudioSource,
        prelude::{AudioFrame, AudioSourceOptions, RtcAudioSource},
    },
};
use tokio::{io::AsyncReadExt, sync::mpsc::UnboundedReceiver};

#[derive(Debug, Clone)]
pub struct VoiceConnection {
    room: Arc<Room>,
    events: Arc<Mutex<UnboundedReceiver<RoomEvent>>>,
    cache: GlobalCache,
}

impl VoiceConnection {
    pub async fn connect_with_options(
        cache: &GlobalCache,
        url: &str,
        token: &str,
        options: RoomOptions,
    ) -> Result<Self, Error> {
        let (room, events) = Room::connect(url, token, options).await?;

        let this = Self {
            cache: cache.clone(),
            room: Arc::new(room),
            events: Arc::new(Mutex::new(events)),
        };

        cache.insert_voice_connection(this.clone()).await;

        Ok(this)
    }

    pub async fn connect(cache: &GlobalCache, url: &str, token: &str) -> Result<Self, Error> {
        Self::connect_with_options(cache, url, token, RoomOptions::default()).await
    }

    pub fn channel_id(&self) -> String {
        self.room.name()
    }

    pub async fn play<S: AudioSource + Send + Sync + 'static>(
        &self,
        mut source: S,
    ) -> Result<(), Error> {
        let native_source = NativeAudioSource::new(
            AudioSourceOptions::default(),
            SAMPLE_RATE as u32,
            CHANNELS as u32,
            FRAME_LENGTH_MS as u32 * 5,
        );
        let track = LocalAudioTrack::create_audio_track(
            "audio",
            RtcAudioSource::Native(native_source.clone()),
        );
        let options = TrackPublishOptions {
            source: TrackSource::Microphone,
            ..Default::default()
        };

        self.room
            .local_participant()
            .publish_track(LocalTrack::Audio(track), options)
            .await?;

        loop {
            let mut audio_frame = AudioFrame {
                data: vec![0i16; FRAME_SIZE].into(),
                sample_rate: SAMPLE_RATE as u32,
                num_channels: CHANNELS as u32,
                samples_per_channel: (FRAME_SIZE / CHANNELS) as u32,
            };

            let finished = source.read(audio_frame.data.to_mut()).await;

            native_source.capture_frame(&audio_frame).await.unwrap();

            if finished {
                source.close().await;

                break;
            };
        }

        Ok(())
    }

    pub async fn disconnect(&self) -> Result<(), Error> {
        self.room.close().await?;

        self.cache.remove_voice_connection(&self.channel_id()).await;

        Ok(())
    }
}
