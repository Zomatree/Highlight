use crate::{notifiers::Notifiers, GlobalCache, HttpClient};


#[derive(Debug, Clone)]
pub struct Context {
    pub cache: GlobalCache,
    pub http: HttpClient,
    pub notifiers: Notifiers,
}