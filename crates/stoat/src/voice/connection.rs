use std::{
    collections::HashMap,
    panic::AssertUnwindSafe,
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{AudioSink, AudioSource, Error, GlobalCache, VideoSource, VoiceEventHandler};
use futures::{FutureExt, StreamExt, future::try_join_all};
use livekit::{
    Room, RoomEvent, RoomOptions,
    id::ParticipantIdentity,
    options::TrackPublishOptions,
    prelude::{RemoteParticipant, RemoteTrackPublication},
    track::{LocalAudioTrack, LocalTrack, LocalVideoTrack, RemoteTrack, TrackKind, TrackSource},
    webrtc::{
        audio_source::native::NativeAudioSource,
        audio_stream::native::NativeAudioStream,
        prelude::{
            AudioFrame, AudioSourceOptions, I420Buffer, RtcAudioSource, RtcVideoSource, VideoFrame,
            VideoResolution, VideoRotation,
        },
        video_source::native::NativeVideoSource,
    },
};
use tokio::{sync::mpsc::UnboundedReceiver, time::sleep};

#[derive(Debug, Clone)]
pub struct VoiceConnection {
    room: Arc<Room>,
    cache: GlobalCache,
}

impl VoiceConnection {
    pub async fn connect_with_options(
        cache: &GlobalCache,
        url: &str,
        token: &str,
        options: RoomOptions,
    ) -> Result<Self, Error> {
        let (room, _) = Room::connect(url, token, options).await?;

        let this = Self {
            cache: cache.clone(),
            room: Arc::new(room),
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

    pub fn register<E: VoiceEventHandler + Send + Sync + 'static>(&self, events: E) {
        let weak = Arc::downgrade(&self.room);
        let cache = self.cache.clone();

        let mut rx = self.room.subscribe();

        tokio::spawn({
            async move {
                while let Some(event) = rx.recv().await {
                    let room = weak.upgrade()?;
                    let conn = Self {
                        room,
                        cache: cache.clone(),
                    };

                    match event {
                        RoomEvent::Connected {
                            participants_with_tracks,
                        } => {
                            handle_error(
                                &conn,
                                &events,
                                events.connected(&conn, participants_with_tracks),
                            )
                            .await
                        }
                        event => log::warn!("Unhandled voice event: {event:?}"),
                    };
                }

                Some(())
            }
        });
    }

    pub async fn play<S: AudioSource + Send + Sync + 'static>(
        &self,
        mut source: S,
    ) -> Result<(), Error> {
        let native_source = NativeAudioSource::new(
            AudioSourceOptions::default(),
            source.sample_rate() as u32,
            source.channels() as u32,
            source.frame_length_ms() as u32 * 5,
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

        let mut audio_frame = AudioFrame {
            data: vec![0i16; source.frame_size()].into(),
            sample_rate: source.sample_rate() as u32,
            num_channels: source.channels() as u32,
            samples_per_channel: (source.frame_size() / source.channels()) as u32,
        };

        loop {
            let finished = source.read(audio_frame.data.to_mut()).await;

            native_source.capture_frame(&audio_frame).await.unwrap();

            if finished {
                source.close().await;

                break;
            };
        }

        Ok(())
    }

    pub async fn play_video<S: VideoSource + Send + Sync + 'static>(
        &self,
        mut source: S,
    ) -> Result<(), Error> {
        let (width, height) = source.resolution();
        let native_source = NativeVideoSource::new(VideoResolution { width, height });
        let track = LocalVideoTrack::create_video_track(
            "video",
            RtcVideoSource::Native(native_source.clone()),
        );
        let options = TrackPublishOptions {
            source: TrackSource::Microphone,
            ..Default::default()
        };

        self.room
            .local_participant()
            .publish_track(LocalTrack::Video(track), options)
            .await?;

        let mut video_frame = VideoFrame {
            rotation: VideoRotation::VideoRotation0,
            buffer: I420Buffer::new(width, height),
            timestamp_us: 0,
        };

        loop {
            let time = Instant::now();
            let finished = source.read(video_frame.buffer.data_mut()).await;

            native_source.capture_frame(&video_frame);

            if finished {
                source.close().await;

                break;
            };

            let elapsed = time.elapsed();

            if let Some(fps) = source.fps() {
                sleep(
                    Duration::from_secs_f32(1.0 / fps)
                        .checked_sub(elapsed)
                        .unwrap_or_default(),
                )
                .await;
            };
        }

        Ok(())
    }

    pub async fn listen_to_track<S: AudioSink>(
        &self,
        publication: RemoteTrackPublication,
        participant: RemoteParticipant,
        mut sink: S,
    ) -> Result<(), Error> {
        if publication.kind() != TrackKind::Audio {
            return Err(Error::NotAudioTrack);
        };

        let track = match publication.track() {
            Some(track) => track,
            None => {
                let rx = self.room.subscribe();
                publication.set_subscribed(true);

                let res = tokio::select!(
                    track = wait_for_track_subscribe(rx, &publication) => { track },
                    _ = tokio::time::sleep(Duration::from_secs(5)) => { publication.track() }
                );

                match res {
                    Some(track) => track,
                    None => return Err(Error::Timeout),
                }
            }
        };

        let RemoteTrack::Audio(track) = track else {
            unreachable!()
        };

        let mut stream = NativeAudioStream::new(
            track.rtc_track(),
            sink.sample_rate() as i32,
            sink.channels() as i32,
        );

        while let Some(frame) = stream.next().await {
            sink.sink(participant.clone(), track.clone(), frame).await;
        }

        Ok(())
    }

    pub async fn listen_to_partipant<S: AudioSink + Send + Clone>(
        &self,
        participant: RemoteParticipant,
        sink: S,
    ) -> Result<(), Error> {
        let futs = participant
            .track_publications()
            .values()
            .filter(|track| track.kind() == TrackKind::Audio)
            .map(|publication| {
                self.listen_to_track(publication.clone(), participant.clone(), sink.clone())
                    .boxed()
            })
            .collect::<Vec<_>>();

        try_join_all(futs).await?;

        Ok(())
    }

    pub async fn listen<S: AudioSink + Send + Clone>(&self, sink: S) -> Result<(), Error> {
        let futs = self
            .room
            .remote_participants()
            .values()
            .map(|participant| {
                self.listen_to_partipant(participant.clone(), sink.clone())
                    .boxed()
            })
            .collect::<Vec<_>>();

        try_join_all(futs).await?;

        Ok(())
    }

    pub fn remote_participants(&self) -> HashMap<ParticipantIdentity, RemoteParticipant> {
        self.room.remote_participants()
    }

    pub fn inner(&self) -> Arc<Room> {
        self.room.clone()
    }

    pub async fn disconnect(&self) -> Result<(), Error> {
        self.room.close().await?;

        self.cache.remove_voice_connection(&self.channel_id()).await;

        Ok(())
    }
}

async fn handle_error<
    T,
    E,
    Fut: Future<Output = Result<T, E>>,
    Ev: VoiceEventHandler<Error = E> + Send + Sync + 'static,
>(
    conn: &VoiceConnection,
    events: &Ev,
    fut: Fut,
) {
    let wrapper = AssertUnwindSafe(fut).catch_unwind();

    match wrapper.await {
        Ok(Ok(_)) => (),
        Ok(Err(e)) => {
            if let Err(e) = AssertUnwindSafe(events.error(conn, e)).catch_unwind().await {
                log::error!("{e:?}")
            }
        }
        Err(e) => {
            log::error!("{e:?}")
        }
    }
}

async fn wait_for_track_subscribe(
    mut rx: UnboundedReceiver<RoomEvent>,
    publication: &RemoteTrackPublication,
) -> Option<RemoteTrack> {
    while let Some(event) = rx.recv().await {
        if let RoomEvent::TrackSubscribed {
            track,
            publication: remote_publication,
            ..
        } = event
        {
            if publication.sid() == remote_publication.sid() {
                return Some(track);
            }
        }
    }

    None
}
