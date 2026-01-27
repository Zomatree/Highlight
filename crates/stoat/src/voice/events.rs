use std::{collections::HashMap, fmt::Debug, sync::Arc};

use async_trait::async_trait;
use livekit::{
    ConnectionState, DataPacketKind, DisconnectReason, RoomEvent, RoomInfo,
    id::TrackSid,
    prelude::{
        ConnectionQuality, LocalParticipant, LocalTrackPublication, Participant, RemoteParticipant,
        RemoteTrackPublication, TrackPublication,
    },
    track::{self, LocalTrack, RemoteTrack},
    webrtc::native::frame_cryptor::EncryptionState,
};

use crate::{Error, VoiceConnection};

#[async_trait]
#[allow(unused)]
pub trait VoiceEventHandler: Sized {
    type Error: From<Error> + Debug + Send + Sync + 'static;

    async fn event(
        &self,
        connection: &VoiceConnection,
        event: RoomEvent,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn connected(
        &self,
        connection: &VoiceConnection,
        tracks: Vec<(RemoteParticipant, Vec<RemoteTrackPublication>)>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn participant_connected(
        &self,
        connection: &VoiceConnection,
        participant: RemoteParticipant,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn participant_disconnected(
        &self,
        connection: &VoiceConnection,
        participant: RemoteParticipant,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn local_track_published(
        &self,
        connection: &VoiceConnection,
        publication: LocalTrackPublication,
        track: LocalTrack,
        participant: LocalParticipant,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn local_track_unpublished(
        &self,
        connection: &VoiceConnection,
        publication: LocalTrackPublication,
        participant: LocalParticipant,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn local_track_subscribed(
        &self,
        connection: &VoiceConnection,
        track: LocalTrack,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn track_subscribed(
        &self,
        connection: &VoiceConnection,
        track: RemoteTrack,
        publication: RemoteTrackPublication,
        participant: RemoteParticipant,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn track_unsubscribed(
        &self,
        connection: &VoiceConnection,
        track: RemoteTrack,
        publication: RemoteTrackPublication,
        participant: RemoteParticipant,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn track_subscription_failed(
        &self,
        connection: &VoiceConnection,
        participant: RemoteParticipant,
        error: track::TrackError,
        track_sid: TrackSid,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn track_published(
        &self,
        connection: &VoiceConnection,
        publication: RemoteTrackPublication,
        participant: RemoteParticipant,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn track_unpublished(
        &self,
        connection: &VoiceConnection,
        publication: RemoteTrackPublication,
        participant: RemoteParticipant,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn track_muted(
        &self,
        connection: &VoiceConnection,
        participant: Participant,
        publication: TrackPublication,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn track_unmuted(
        &self,
        connection: &VoiceConnection,
        participant: Participant,
        publication: TrackPublication,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn room_metadata_changed(
        &self,
        connection: &VoiceConnection,
        old_metadata: String,
        metadata: String,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn participant_metadata_changed(
        &self,
        connection: &VoiceConnection,
        participant: Participant,
        old_metadata: String,
        metadata: String,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn participant_name_changed(
        &self,
        connection: &VoiceConnection,
        participant: Participant,
        old_name: String,
        name: String,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn participant_attributes_changed(
        &self,
        connection: &VoiceConnection,
        participant: Participant,
        changed_attributes: HashMap<String, String>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn participant_encryption_status_changed(
        &self,
        connection: &VoiceConnection,
        participant: Participant,
        is_encrypted: bool,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn active_speakers_changed(
        &self,
        connection: &VoiceConnection,
        speakers: Vec<Participant>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn connection_quality_changed(
        &self,
        connection: &VoiceConnection,
        quality: ConnectionQuality,
        participant: Participant,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn data_received(
        &self,
        connection: &VoiceConnection,
        payload: Arc<Vec<u8>>,
        topic: Option<String>,
        kind: DataPacketKind,
        participant: Option<RemoteParticipant>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn e2ee_state_changed(
        &self,
        connection: &VoiceConnection,
        participant: Participant,
        state: EncryptionState,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn connection_state_changed(
        &self,
        connection: &VoiceConnection,
        connection_state: ConnectionState,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn disconnected(
        &self,
        connection: &VoiceConnection,
        reason: DisconnectReason,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn reconnecting(&self, connection: &VoiceConnection) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn reconnected(&self, connection: &VoiceConnection) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn room_updated(
        &self,
        connection: &VoiceConnection,
        room: RoomInfo,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn participants_updated(
        &self,
        connection: &VoiceConnection,
        participants: Vec<Participant>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn error(
        &self,
        connection: &VoiceConnection,
        error: Self::Error,
    ) -> Result<(), Self::Error> {
        log::error!("{error:?}");

        Ok(())
    }
}
