use std::path::Path;
use tokio::fs::File;

use reqwest::Body;

pub struct LocalFile {
    pub name: String,
    pub body: Body,
}

impl LocalFile {
    pub fn new<B: Into<Body>>(name: String, body: B) -> Self {
        Self {
            name,
            body: body.into(),
        }
    }

    pub async fn from_path<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();

        let filename = path
            .file_name()
            .expect("File not found.")
            .to_str()
            .expect("Invalid filename")
            .to_string();
        let file = File::open(path).await.expect("Failed to open file.");

        Self::new(filename, file)
    }

    pub fn spoiler(mut self) -> Self {
        if !self.is_spoiler() {
            self.name = format!("SPOILER_{}", &self.name);
        };

        self
    }

    pub fn is_spoiler(&self) -> bool {
        self.name.starts_with("SPOILER_")
    }
}
