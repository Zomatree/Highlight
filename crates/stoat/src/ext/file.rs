use std::time::SystemTime;

use async_trait::async_trait;
use bytes::Bytes;
use stoat_models::v0::File;

use crate::{HttpClient, Identifiable, Result, created_at};

#[async_trait]
pub trait FileExt {
    fn url(&self, http: impl AsRef<HttpClient>, preview: bool) -> String;
    async fn bytes(&self, http: impl AsRef<HttpClient> + Send, preview: bool) -> Result<Bytes>;
}

#[async_trait]
impl FileExt for File {
    fn url(&self, http: impl AsRef<HttpClient>, preview: bool) -> String {
        http.as_ref().format_file_url(&self.tag, &self.id, preview.then_some(&self.filename))
    }

    async fn bytes(&self, http: impl AsRef<HttpClient> + Send, preview: bool) -> Result<Bytes> {
        if preview {
            http.as_ref().fetch_image_preview(&self.tag, &self.id).await
        } else {
            http.as_ref().fetch_image(&self.tag, &self.id, &self.filename).await
        }
    }
}

impl Identifiable for File {
    fn created_at(&self) -> SystemTime {
        created_at(&self.id)
    }
}
