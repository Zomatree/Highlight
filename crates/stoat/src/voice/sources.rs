use std::process::Stdio;

use async_trait::async_trait;

use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::{Child, ChildStdout, Command},
};

use crate::voice::{CHANNELS, FRAME_SIZE, SAMPLE_RATE};

#[async_trait]
pub trait AudioSource {
    /// Reads 20ms of audio,
    async fn read(&mut self, buffer: &mut [i16]) -> bool;
    async fn close(&mut self) {}
}

pub struct PCMAudio<B> {
    stream: B,
}

impl<B> PCMAudio<B> {
    pub fn new(stream: B) -> Self {
        Self { stream }
    }
}

#[async_trait]
impl<B: AsyncRead + Send + Sync + Unpin> AudioSource for PCMAudio<B> {
    async fn read(&mut self, buffer: &mut [i16]) -> bool {
        for i in 0..FRAME_SIZE {
            if let Ok(data) = self.stream.read_i16_le().await {
                buffer[i] = data
            } else {
                return true;
            };
        }

        false
    }
}

pub struct FFmpegPCMAudioOptions {
    pub executable: String,
    pub before_options: Option<Vec<String>>,
    pub options: Option<Vec<String>>,
    pub blocksize: usize,
}

impl Default for FFmpegPCMAudioOptions {
    fn default() -> Self {
        Self {
            executable: "ffmpeg".to_string(),
            before_options: None,
            options: None,
            blocksize: 1024 * 8,
        }
    }
}

pub struct FFmpegPCMAudio {
    child: Child,
    stdout: ChildStdout,
}

impl FFmpegPCMAudio {
    pub fn new(source: &str) -> Self {
        Self::new_with_options(source, FFmpegPCMAudioOptions::default())
    }

    pub fn new_with_options(source: &str, options: FFmpegPCMAudioOptions) -> Self {
        let mut command = Command::new(options.executable);

        if let Some(before_options) = options.before_options {
            for arg in before_options {
                command.arg(arg);
            }
        };

        command
            .arg("-i")
            .arg(source)
            .arg("-f")
            .arg("s16le")
            .arg("-acodec")
            .arg("pcm_s16le")
            .arg("-ar")
            .arg(SAMPLE_RATE.to_string())
            .arg("-ac")
            .arg(CHANNELS.to_string())
            .arg("-vn")
            .arg("-loglevel")
            .arg("warning")
            .arg("-blocksize")
            .arg(options.blocksize.to_string());

        if let Some(options) = options.options {
            for arg in options {
                command.arg(arg);
            }
        };

        let mut child = command
            .arg("pipe:1")
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let stdout = child.stdout.take().unwrap();

        Self { child, stdout }
    }
}

#[async_trait]
impl AudioSource for FFmpegPCMAudio {
    async fn read(&mut self, buffer: &mut [i16]) -> bool {
        for i in 0..FRAME_SIZE {
            if let Ok(data) = self.stdout.read_i16_le().await {
                buffer[i] = data
            } else {
                return true;
            };
        }

        false
    }

    async fn close(&mut self) {
        let _ = self.child.kill().await;
    }
}
