mod connection;
mod events;
mod sink;
mod source;

pub use connection::VoiceConnection;
pub use events::VoiceEventHandler;
pub use livekit;
pub use sink::*;
pub use source::*;

const SAMPLE_RATE: usize = 48000;
const CHANNELS: usize = 2;
const FRAME_LENGTH_MS: usize = 50;
const SAMPLE_SIZE: usize = size_of::<i16>() * CHANNELS;
const SAMPLES_PER_FRAME: usize = SAMPLE_RATE / 1000 * FRAME_LENGTH_MS;
const FRAME_SIZE: usize = SAMPLES_PER_FRAME * SAMPLE_SIZE;
