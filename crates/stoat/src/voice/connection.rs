use std::{panic::AssertUnwindSafe, sync::Arc, time::Duration};

use crate::{AudioSink, AudioSource, Error, GlobalCache, VideoSource, VoiceEventHandler};
use futures::{FutureExt, StreamExt, future::try_join_all};
use livekit::{
    Room, RoomEvent, RoomOptions,
    options::TrackPublishOptions,
    prelude::{LocalParticipant, Participant, RemoteParticipant, RemoteTrackPublication},
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
use stoat_models::v0::{User, UserVoiceState};
use tokio::{
    sync::mpsc::UnboundedReceiver,
    time::{MissedTickBehavior, interval},
};

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

        cache.insert_voice_connection(this.clone());

        Ok(this)
    }

    pub async fn connect(cache: &GlobalCache, url: &str, token: &str) -> Result<Self, Error> {
        Self::connect_with_options(cache, url, token, RoomOptions::default()).await
    }

    pub fn channel_id(&self) -> String {
        self.room.name()
    }

    pub fn local_participant(&self) -> (User, UserVoiceState, LocalParticipant) {
        let channel_voice_state = self
            .cache
            .get_voice_state(&self.channel_id())
            .expect("no channel voice state");

        let user = self.cache.get_current_user().expect("No local user");
        let voice_state = channel_voice_state
            .participants
            .iter()
            .find(|s| &s.id == &user.id)
            .expect("No local voice state found");

        (user, voice_state.clone(), self.room.local_participant())
    }

    pub fn remote_participants(&self) -> Vec<(User, UserVoiceState, RemoteParticipant)> {
        let mut participants = Vec::new();
        let channel_voice_state = self
            .cache
            .get_voice_state(&self.channel_id())
            .expect("no channel voice state");

        for remote in self.room.remote_participants().values() {
            if let Some(user) = self.cache.get_user(&remote.identity().as_str())
                && let Some(voice_state) = channel_voice_state
                    .participants
                    .iter()
                    .find(|s| &s.id == &user.id)
            {
                participants.push((user, voice_state.clone(), remote.clone()));
            };
        }

        participants
    }

    pub fn participants(&self) -> Vec<(User, UserVoiceState, Participant)> {
        let mut participants = Vec::new();

        let (local_user, local_voice_state, local_participant) = self.local_participant();

        participants.push((
            local_user,
            local_voice_state,
            Participant::Local(local_participant),
        ));

        participants.extend(self.remote_participants().into_iter().map(
            |(user, voice_state, participant)| {
                (user, voice_state, Participant::Remote(participant))
            },
        ));

        participants
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
                        RoomEvent::ParticipantConnected(participant) => {
                            handle_error(
                                &conn,
                                &events,
                                events.participant_connected(&conn, participant),
                            )
                            .await
                        }
                        RoomEvent::ParticipantDisconnected(participant) => {
                            handle_error(
                                &conn,
                                &events,
                                events.participant_disconnected(&conn, participant),
                            )
                            .await
                        }
                        RoomEvent::LocalTrackPublished {
                            publication,
                            track,
                            participant,
                        } => {
                            handle_error(
                                &conn,
                                &events,
                                events.local_track_published(
                                    &conn,
                                    publication,
                                    track,
                                    participant,
                                ),
                            )
                            .await
                        }
                        RoomEvent::LocalTrackUnpublished {
                            publication,
                            participant,
                        } => {
                            handle_error(
                                &conn,
                                &events,
                                events.local_track_unpublished(&conn, publication, participant),
                            )
                            .await
                        }
                        RoomEvent::LocalTrackSubscribed { track } => {
                            handle_error(
                                &conn,
                                &events,
                                events.local_track_subscribed(&conn, track),
                            )
                            .await
                        }
                        RoomEvent::TrackSubscribed {
                            track,
                            publication,
                            participant,
                        } => {
                            handle_error(
                                &conn,
                                &events,
                                events.track_subscribed(&conn, track, publication, participant),
                            )
                            .await
                        }
                        RoomEvent::TrackUnsubscribed {
                            track,
                            publication,
                            participant,
                        } => {
                            handle_error(
                                &conn,
                                &events,
                                events.track_unsubscribed(&conn, track, publication, participant),
                            )
                            .await
                        }
                        RoomEvent::TrackSubscriptionFailed {
                            participant,
                            error,
                            track_sid,
                        } => {
                            handle_error(
                                &conn,
                                &events,
                                events.track_subscription_failed(
                                    &conn,
                                    participant,
                                    error,
                                    track_sid,
                                ),
                            )
                            .await
                        }
                        RoomEvent::TrackPublished {
                            publication,
                            participant,
                        } => {
                            handle_error(
                                &conn,
                                &events,
                                events.track_published(&conn, publication, participant),
                            )
                            .await
                        }
                        RoomEvent::TrackUnpublished {
                            publication,
                            participant,
                        } => {
                            handle_error(
                                &conn,
                                &events,
                                events.track_unpublished(&conn, publication, participant),
                            )
                            .await
                        }
                        RoomEvent::TrackMuted {
                            participant,
                            publication,
                        } => {
                            handle_error(
                                &conn,
                                &events,
                                events.track_muted(&conn, participant, publication),
                            )
                            .await
                        }
                        RoomEvent::TrackUnmuted {
                            participant,
                            publication,
                        } => {
                            handle_error(
                                &conn,
                                &events,
                                events.track_unmuted(&conn, participant, publication),
                            )
                            .await
                        }
                        RoomEvent::RoomMetadataChanged {
                            old_metadata,
                            metadata,
                        } => {
                            handle_error(
                                &conn,
                                &events,
                                events.room_metadata_changed(&conn, old_metadata, metadata),
                            )
                            .await
                        }
                        RoomEvent::ParticipantMetadataChanged {
                            participant,
                            old_metadata,
                            metadata,
                        } => {
                            handle_error(
                                &conn,
                                &events,
                                events.participant_metadata_changed(
                                    &conn,
                                    participant,
                                    old_metadata,
                                    metadata,
                                ),
                            )
                            .await
                        }
                        RoomEvent::ParticipantNameChanged {
                            participant,
                            old_name,
                            name,
                        } => {
                            handle_error(
                                &conn,
                                &events,
                                events.participant_name_changed(&conn, participant, old_name, name),
                            )
                            .await
                        }
                        RoomEvent::ParticipantAttributesChanged {
                            participant,
                            changed_attributes,
                        } => {
                            handle_error(
                                &conn,
                                &events,
                                events.participant_attributes_changed(
                                    &conn,
                                    participant,
                                    changed_attributes,
                                ),
                            )
                            .await
                        }
                        RoomEvent::ParticipantEncryptionStatusChanged {
                            participant,
                            is_encrypted,
                        } => {
                            handle_error(
                                &conn,
                                &events,
                                events.participant_encryption_status_changed(
                                    &conn,
                                    participant,
                                    is_encrypted,
                                ),
                            )
                            .await
                        }
                        RoomEvent::ActiveSpeakersChanged { speakers } => {
                            handle_error(
                                &conn,
                                &events,
                                events.active_speakers_changed(&conn, speakers),
                            )
                            .await
                        }
                        RoomEvent::ConnectionQualityChanged {
                            quality,
                            participant,
                        } => {
                            handle_error(
                                &conn,
                                &events,
                                events.connection_quality_changed(&conn, quality, participant),
                            )
                            .await
                        }
                        RoomEvent::DataReceived {
                            payload,
                            topic,
                            kind,
                            participant,
                        } => {
                            handle_error(
                                &conn,
                                &events,
                                events.data_received(&conn, payload, topic, kind, participant),
                            )
                            .await
                        }
                        RoomEvent::E2eeStateChanged { participant, state } => {
                            handle_error(
                                &conn,
                                &events,
                                events.e2ee_state_changed(&conn, participant, state),
                            )
                            .await
                        }
                        RoomEvent::ConnectionStateChanged(connection_state) => {
                            handle_error(
                                &conn,
                                &events,
                                events.connection_state_changed(&conn, connection_state),
                            )
                            .await
                        }
                        RoomEvent::Disconnected { reason } => {
                            handle_error(&conn, &events, events.disconnected(&conn, reason)).await
                        }
                        RoomEvent::Reconnecting => {
                            handle_error(&conn, &events, events.reconnecting(&conn)).await
                        }
                        RoomEvent::Reconnected => {
                            handle_error(&conn, &events, events.reconnected(&conn)).await
                        }
                        RoomEvent::RoomUpdated { room } => {
                            handle_error(&conn, &events, events.room_updated(&conn, room)).await
                        }
                        RoomEvent::ParticipantsUpdated { participants } => {
                            handle_error(
                                &conn,
                                &events,
                                events.participants_updated(&conn, participants),
                            )
                            .await
                        }
                        event => log::debug!("Unhandled voice event: {event:?}"),
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

            if finished {
                source.close().await;

                break;
            } else {
                native_source.capture_frame(&audio_frame).await?;
            }
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

        let mut interval = source.fps().map(|fps| {
            let mut interval = interval(Duration::from_secs_f32(1.0 / fps));
            interval.set_missed_tick_behavior(MissedTickBehavior::Burst);
            interval
        });

        loop {
            if let Some(interval) = &mut interval {
                interval.tick().await;
            };

            let finished = source.read(video_frame.buffer.data_mut()).await;

            if finished {
                source.close().await;

                break;
            } else {
                native_source.capture_frame(&video_frame);
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

    pub fn inner(&self) -> Arc<Room> {
        self.room.clone()
    }

    pub async fn disconnect(&self) -> Result<(), Error> {
        self.cache.remove_voice_connection(&self.channel_id());

        self.room.close().await?;

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
                rx.close();
                return Some(track);
            }
        }
    }

    None
}
