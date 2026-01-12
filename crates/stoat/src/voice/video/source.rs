use std::process::Stdio;

use async_trait::async_trait;

use serde::Deserialize;
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    process::{Child, ChildStdout, Command},
};

#[async_trait]
pub trait VideoSource: Sized {
    #[inline]
    fn resolution(&self) -> (u32, u32) {
        (1280, 720)
    }
    #[inline]
    fn fps(&self) -> Option<f32> {
        None
    }

    async fn read(&mut self, buffer: (&mut [u8], &mut [u8], &mut [u8])) -> bool;
    async fn close(self) {}
}

pub struct YUVVideo<B> {
    buffer: Vec<u8>,
    y_size: usize,
    uv_size: usize,
    frame_size: usize,
    fps: Option<f32>,
    stream: B,
}

impl<B> YUVVideo<B> {
    pub fn new(stream: B, fps: Option<f32>) -> Self {
        let y_size = 1280 * 720;
        let uv_size = y_size / 4;
        let frame_size = y_size + 2 * uv_size;

        Self {
            stream,
            y_size,
            uv_size,
            frame_size,
            fps,
            buffer: vec![0; frame_size],
        }
    }
}

#[async_trait]
impl<B: AsyncRead + Send + Sync + Unpin> VideoSource for YUVVideo<B> {
    fn fps(&self) -> Option<f32> {
        self.fps
    }

    async fn read(&mut self, buffer: (&mut [u8], &mut [u8], &mut [u8])) -> bool {
        let mut bytes_read = 0;

        while bytes_read < self.frame_size {
            if let Ok(n) = self.stream.read(&mut self.buffer[bytes_read..]).await {
                if n == 0 {
                    return true;
                };

                bytes_read += n;
            } else {
                return true;
            }
        }

        buffer.0.copy_from_slice(&self.buffer[0..self.y_size]);
        buffer
            .1
            .copy_from_slice(&self.buffer[self.y_size..self.y_size + self.uv_size]);
        buffer
            .2
            .copy_from_slice(&self.buffer[self.y_size + self.uv_size..self.frame_size]);

        false
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct VideoMetadata {
    pub width: usize,
    pub height: usize,
    pub fps: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FFmpegYUVVideoOptions {
    pub executable: String,
    pub ffprobe: String,
    pub before_options: Vec<String>,
    pub options: Vec<String>,
    pub metadata: Option<VideoMetadata>,
}

impl Default for FFmpegYUVVideoOptions {
    fn default() -> Self {
        Self {
            executable: "ffmpeg".to_string(),
            ffprobe: "ffprobe".to_string(),
            before_options: Vec::new(),
            options: Vec::new(),
            metadata: None,
        }
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct FFprobeStream {
    width: usize,
    height: usize,
    r_frame_rate: String,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct FFprobeStats {
    streams: Vec<FFprobeStream>,
}

pub struct FFmpegYUVVideo {
    metadata: VideoMetadata,
    y_size: usize,
    uv_size: usize,
    frame_size: usize,
    child: Child,
    stdout: ChildStdout,
    buffer: Vec<u8>,
}

impl FFmpegYUVVideo {
    pub async fn new(source: &str) -> Self {
        Self::new_with_options(source, FFmpegYUVVideoOptions::default()).await
    }

    pub async fn new_with_options(source: &str, options: FFmpegYUVVideoOptions) -> Self {
        let metadata = if let Some(metadata) = options.metadata {
            metadata
        } else {
            let output = Command::new(&options.ffprobe)
                .arg("-v")
                .arg("quiet")
                .arg("-print_format")
                .arg("json")
                .arg("-show_streams")
                .arg("-select_streams")
                .arg("v:0")
                .arg(source)
                .output()
                .await
                .expect("Failed to execute ffprobe")
                .stdout;

            let stats = serde_json::from_slice::<FFprobeStats>(&output)
                .expect("Failed to parse ffprobe output");
            let stream = stats.streams.into_iter().next().expect("No video streams");

            let (frames, per) = stream
                .r_frame_rate
                .split_once('/')
                .expect("Malformed framerate");

            let fps = frames.parse::<f32>().unwrap() / per.parse::<f32>().unwrap();

            VideoMetadata {
                width: stream.width,
                height: stream.height,
                fps,
            }
        };

        let mut command = Command::new(&options.executable);

        for arg in &options.before_options {
            command.arg(arg);
        }

        command
            .arg("-i")
            .arg(source)
            .arg("-c:v")
            .arg("rawvideo")
            .arg("-pix_fmt")
            .arg("yuv420p")
            .arg("-f")
            .arg("rawvideo")
            .arg("-an")
            .arg("-loglevel")
            .arg("warning");

        for arg in &options.options {
            command.arg(arg);
        }

        let mut child = command
            .arg("pipe:1")
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let stdout = child.stdout.take().unwrap();

        let y_size = metadata.width * metadata.height;
        let uv_size = y_size / 4;
        let frame_size = y_size + 2 * uv_size;
        let buffer = vec![0; frame_size as usize];

        Self {
            metadata,
            y_size,
            uv_size,
            frame_size,
            child,
            stdout,
            buffer,
        }
    }
}

#[async_trait]
impl VideoSource for FFmpegYUVVideo {
    fn resolution(&self) -> (u32, u32) {
        (self.metadata.width as u32, self.metadata.height as u32)
    }

    fn fps(&self) -> Option<f32> {
        Some(self.metadata.fps)
    }

    async fn read(&mut self, buffer: (&mut [u8], &mut [u8], &mut [u8])) -> bool {
        let mut bytes_read = 0;

        while bytes_read < self.frame_size {
            if let Ok(n) = self.stdout.read(&mut self.buffer[bytes_read..]).await {
                if n == 0 {
                    return true;
                };

                bytes_read += n;
            } else {
                return true;
            }
        }

        buffer.0.copy_from_slice(&self.buffer[0..self.y_size]);
        buffer
            .1
            .copy_from_slice(&self.buffer[self.y_size..self.y_size + self.uv_size]);
        buffer
            .2
            .copy_from_slice(&self.buffer[self.y_size + self.uv_size..self.frame_size]);

        false
    }

    async fn close(mut self) {
        let _ = self.child.kill().await;
    }
}
