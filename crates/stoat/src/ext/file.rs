use std::time::SystemTime;

use stoat_models::v0::File;

use crate::{HttpClient, Identifiable, created_at};

pub trait FileExt {
    fn url(&self, http: &HttpClient) -> String;
}

impl FileExt for File {
    fn url(&self, http: &HttpClient) -> String {
        http.format_file_url(&self.tag, &self.id)
    }
}

impl Identifiable for File {
    fn created_at(&self) -> SystemTime {
        created_at(&self.id)
    }
}
