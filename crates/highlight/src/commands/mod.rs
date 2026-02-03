use std::time::Duration;

use stoat::{
    Error as StoatError, async_trait,
    commands::{Command, CommandEventHandler, Context, when_mentioned_or},
    error::{StoatHttpError, StoatHttpErrorType},
};

use crate::{Error, State, utils::MessageExt};

mod highlight;
mod info;
mod moderation;
mod starboard;

#[derive(Clone)]
pub struct CommandEvents;

#[async_trait]
impl CommandEventHandler for CommandEvents {
    type Error = Error;
    type State = State;

    async fn get_prefix(&self, ctx: Context<Error, State>) -> Result<Vec<String>, Error> {
        Ok(when_mentioned_or(
            &ctx,
            &[ctx.state.config.bot.prefix.clone()],
        ))
    }

    async fn after_command(&self, ctx: Context<Error, State>) -> Result<(), Error> {
        let Some(command) = ctx.command.as_ref() else {
            return Ok(());
        };

        if command.name == "highlight" || command.parents.get(0).is_some_and(|p| p == "highlight") {
            ctx.message.delete_after(&ctx, Duration::from_secs(5));
        };

        Ok(())
    }

    async fn error(&self, ctx: Context<Error, State>, error: Error) -> Result<(), Error> {
        let msg = match error {
            Error::StoatError(StoatError::NotInServer) => {
                "This command can only be used in a server".to_string()
            }
            Error::StoatError(StoatError::MissingParameter) => "Missing parameter".to_string(),
            Error::StoatError(StoatError::HttpError(StoatHttpError {
                error_type: StoatHttpErrorType::MissingPermission { permission },
                ..
            })) => format!("Bot is missing permission `{permission}`."),
            _ => {
                log::error!("{error:?}");
                return Ok(());
            }
        };

        ctx.send().content(msg).build().await?;

        Ok(())
    }
}

// async fn play(ctx: Context<Error, State>) -> Result<(), Error> {
//     let Some((voice_channel_id, _)) = ctx.get_member().await?.voice(&ctx) else {
//         return Ok(());
//     };

//     let channel = ctx.cache.get_channel(&voice_channel_id).unwrap();

//     let conn = channel
//         .join_call(
//             &ctx,
//             &ctx,
//             Some(ctx.cache.livekit_nodes().first().unwrap().name.clone()),
//         )
//         .await?;

//     // let (a, b) = async_ringbuf::AsyncHeapRb::new(1920 * 5).split();

//     // let sink = RingSink { prod: Mutex::new(a) };
//     // let source = RingSource::new(Mutex::new(b));

//     // let (_, _, p) = conn.remote_participants().into_iter().find(|(user, _, p)| &user.id == &ctx.message.author).unwrap();

//     // let track = p.track_publications().into_values().find(|t| t.kind() == TrackKind::Audio).unwrap();

//     // // pin!(sink);
//     // // pin!(source);

//     // select! {
//     //     e = conn.listen_to_track(track.clone(), p.clone(), sink) => e,
//     //     e = conn.play(source) => e,
//     // }?;

//     conn.play(RNNoiseFFmpegPCMAudio::new("audio.mp3")).await?;

//     Ok(())
// }

// pub struct RNNoiseFFmpegPCMAudio {
//     child: Child,
//     stdout: ChildStdout,
//     denoise: Box<DenoiseState<'static>>,
//     buf: Vec<f32>,
//     output: Vec<f32>,
// }

// impl RNNoiseFFmpegPCMAudio {
//     pub fn new(source: &str) -> Self {
//         let denoise = DenoiseState::new();

//         let mut command = tokio::process::Command::new("ffmpeg");

//         let mut child = command
//             .arg("-i")
//             .arg(source)
//             .arg("-f")
//             .arg("s16le")
//             .arg("-acodec")
//             .arg("pcm_s16le")
//             .arg("-ar")
//             .arg("48000")
//             .arg("-ac")
//             .arg("1")
//             .arg("-vn")
//             .arg("-loglevel")
//             .arg("warning")
//             .arg("-blocksize")
//             .arg("8192")
//             .arg("pipe:1")
//             .stdout(Stdio::piped())
//             .spawn()
//             .unwrap();

//         let stdout = child.stdout.take().unwrap();

//         Self {
//             child,
//             stdout,
//             denoise,
//             buf: vec![0.; 4 * FRAME_SIZE],
//             output: vec![0.; 4 * FRAME_SIZE],
//         }
//     }
// }

// #[async_trait]
// impl AudioSource for RNNoiseFFmpegPCMAudio {
//     fn channels(&self) -> usize {
//         1
//     }
//     fn frame_length_ms(&self) -> usize {
//         20
//     }

//     async fn read(&mut self, buffer: &mut [i16]) -> bool {
//         for i in 0..self.frame_size() {
//             if let Ok(data) = self.stdout.read_i16_le().await {
//                 self.buf[i] = data as f32;
//             } else {
//                 return true;
//             };
//         }

//         for i in 0..(self.frame_length_ms() / 5) {
//             let lower = i * FRAME_SIZE;
//             let upper = lower + FRAME_SIZE;

//             self.denoise
//                 .process_frame(&mut self.output[lower..upper], &self.buf[lower..upper]);
//         }

//         for i in 0..self.frame_size() {
//             buffer[i] = self.output[i] as i16;
//         }

//         false
//     }

//     async fn close(mut self) {
//         let _ = self.child.kill().await;
//     }
// }

// struct RingSink {
//     pub prod: Mutex<AsyncHeapProd<i16>>,
// }

// #[async_trait]
// impl AudioSink for RingSink {
//     fn channels(&self) -> usize {
//         1
//     }

//     async fn sink(
//         &mut self,
//         _participant: RemoteParticipant,
//         _track: RemoteAudioTrack,
//         frame: AudioFrame<'_>,
//     ) {
//         self.prod.lock().await.push_slice(&frame.data);
//     }
// }

// struct RingSource {
//     pub cons: Mutex<AsyncHeapCons<i16>>,
//     denoise: Box<DenoiseState<'static>>,
//     buf: Vec<f32>,
//     output: Vec<f32>,
// }

// impl RingSource {
//     pub fn new(cons: Mutex<AsyncHeapCons<i16>>) -> Self {
//         Self {
//             cons,
//             denoise: DenoiseState::new(),
//             buf: vec![0.; 4 * FRAME_SIZE],
//             output: vec![0.; 4 * FRAME_SIZE],
//         }
//     }
// }

// #[async_trait]
// impl AudioSource for RingSource {
//     fn channels(&self) -> usize {
//         1
//     }

//     async fn read(&mut self, buffer: &mut [i16]) -> bool {
//         let mut cons = self.cons.lock().await;

//         for i in 0..self.frame_size() {
//             self.buf[i] = cons.next().await.unwrap_or_default() as f32;
//         }

//         for i in 0..(self.frame_length_ms() / 5) {
//             let lower = i * FRAME_SIZE;
//             let upper = lower + FRAME_SIZE;

//             self.denoise
//                 .process_frame(&mut self.output[lower..upper], &self.buf[lower..upper]);
//         }

//         for i in 0..self.frame_size() {
//             buffer[i] = self.output[i] as i16;
//         }

//         false
//     }
// }

// async fn close(ctx: Context<Error, State>) -> Result<(), Error> {
//     ctx.events.close()?;

//     Ok(())
// }

pub fn commands() -> Vec<Command<Error, State>> {
    [
        vec![highlight::command(), info::command(), starboard::command()].as_slice(),
        moderation::commands().as_slice(),
    ]
    .concat()
}
