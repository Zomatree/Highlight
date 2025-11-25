use std::io::{BufWriter, Seek, Write};

use async_trait::async_trait;
use hound::{self, SampleFormat, WavSpec, WavWriter};
use livekit::{prelude::RemoteParticipant, track::RemoteAudioTrack, webrtc::prelude::AudioFrame};

use crate::voice::{CHANNELS, SAMPLE_RATE};

#[async_trait]
pub trait AudioSink: Sized {
    async fn sink(
        &mut self,
        participant: RemoteParticipant,
        track: RemoteAudioTrack,
        frame: AudioFrame<'_>,
    );

    async fn close(self) {}
}

pub struct WavAudioSink<W: Write + Seek + Send + Sync> {
    writer: WavWriter<BufWriter<W>>,
}

impl<W: Write + Seek + Send + Sync> WavAudioSink<W> {
    pub fn new(writer: W) -> Self {
        let writer = WavWriter::new(
            BufWriter::new(writer),
            WavSpec {
                channels: CHANNELS as u16,
                sample_rate: SAMPLE_RATE as u32,
                bits_per_sample: (size_of::<i16>() * 8) as u16,
                sample_format: SampleFormat::Int,
            },
        )
        .unwrap();

        Self { writer }
    }
}

#[async_trait]
impl<W: Write + Seek + Send + Sync> AudioSink for WavAudioSink<W> {
    async fn sink(
        &mut self,
        _participant: RemoteParticipant,
        _track: RemoteAudioTrack,
        frame: AudioFrame<'_>,
    ) {
        let mut writer = self.writer.get_i16_writer(frame.data.len() as u32);

        for sample in frame.data.iter() {
            writer.write_sample(*sample);
        }

        writer.flush().unwrap();
    }

    async fn close(self) {
        self.writer.finalize().unwrap();
    }
}
