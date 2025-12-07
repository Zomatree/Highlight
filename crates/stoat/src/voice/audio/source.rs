use std::process::Stdio;

use async_trait::async_trait;

use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::{Child, ChildStdout, Command},
};

#[async_trait]
pub trait AudioSource: Sized {
    #[inline]
    fn sample_rate(&self) -> usize {
        48000
    }
    #[inline]
    fn channels(&self) -> usize {
        2
    }
    #[inline]
    fn frame_length_ms(&self) -> usize {
        20
    }
    #[inline]
    fn sample_size(&self) -> usize {
        size_of::<i16>() * self.channels()
    }
    #[inline]
    fn samples_per_frame(&self) -> usize {
        self.sample_rate() / 1000 * self.frame_length_ms()
    }
    #[inline]
    fn frame_size(&self) -> usize {
        self.samples_per_frame() * self.sample_size()
    }

    /// Reads frame_length_ms of audio, default amount is 20ms
    async fn read(&mut self, buffer: &mut [i16]) -> bool;
    async fn close(self) {}
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
        for i in 0..self.frame_size() {
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
    pub before_options: Vec<String>,
    pub options: Vec<String>,
    pub blocksize: usize,
}

impl Default for FFmpegPCMAudioOptions {
    fn default() -> Self {
        Self {
            executable: "ffmpeg".to_string(),
            before_options: Vec::new(),
            options: Vec::new(),
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
        let mut command = Command::new(&options.executable);

        for arg in &options.before_options {
            command.arg(arg);
        }

        command
            .arg("-i")
            .arg(source)
            .arg("-f")
            .arg("s16le")
            .arg("-acodec")
            .arg("pcm_s16le")
            .arg("-ar")
            .arg("48000")
            .arg("-ac")
            .arg("2")
            .arg("-vn")
            .arg("-loglevel")
            .arg("warning")
            .arg("-blocksize")
            .arg(options.blocksize.to_string());

        for arg in &options.options {
            command.arg(arg);
        }

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
        for i in 0..self.frame_size() {
            if let Ok(data) = self.stdout.read_i16_le().await {
                buffer[i] = data
            } else {
                return true;
            };
        }

        false
    }

    async fn close(mut self) {
        let _ = self.child.kill().await;
    }
}
