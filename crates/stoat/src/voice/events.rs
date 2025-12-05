use std::fmt::Debug;

use async_trait::async_trait;
use livekit::prelude::{RemoteParticipant, RemoteTrackPublication};

use crate::{Error, VoiceConnection};

#[async_trait]
#[allow(unused)]
pub trait VoiceEventHandler: Sized {
    type Error: From<Error> + Debug + Send + Sync + 'static;

    async fn connected(
        &self,
        connection: &VoiceConnection,
        tracks: Vec<(RemoteParticipant, Vec<RemoteTrackPublication>)>,
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
