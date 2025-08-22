use futures::{TryStreamExt, lock::Mutex};
use lru::LruCache;
use sqlx::{PgPool, postgres::PgConnectOptions};
use std::{num::NonZero, sync::Arc};

use crate::{Config, Error};

#[derive(Clone, Debug)]
pub struct State {
    pub config: Arc<Config>,
    pub pool: PgPool,
    pub cached_keywords: Arc<Mutex<LruCache<(String, String), Vec<String>>>>,
}

impl State {
    pub async fn new() -> Self {
        let config = Arc::new(
            toml::from_str::<Config>(&std::fs::read_to_string("Highlight.toml").unwrap()).unwrap(),
        );

        let pool = PgPool::connect_with(
            PgConnectOptions::new_without_pgpass()
                .host("localhost")
                .database("highlight"),
        )
        .await
        .unwrap();

        let cached_keywords = Arc::new(Mutex::new(LruCache::new(NonZero::new(1000).unwrap())));

        Self {
            pool,
            config,
            cached_keywords,
        }
    }

    pub async fn fetch_keywords(
        &self,
        user_id: &str,
        server_id: &str,
    ) -> Result<Vec<String>, Error> {
        sqlx::query_scalar("select keyword from highlights where user_id=$1 and server_id=$2")
            .bind(&user_id)
            .bind(&server_id)
            .fetch(&self.pool)
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| e.into())
    }

    pub async fn get_keywords(
        &self,
        user_id: String,
        server_id: String,
    ) -> Result<Vec<String>, Error> {
        let mut lock = self.cached_keywords.lock().await;

        if let Some(value) = lock.get(&(user_id.clone(), server_id.clone())) {
            return Ok(value.clone());
        } else {
            let highlights = self.fetch_keywords(&user_id, &server_id).await?;

            lock.put((user_id, server_id), highlights.clone());

            Ok(highlights)
        }
    }

    pub async fn add_keyword(
        &self,
        user_id: String,
        server_id: String,
        keyword: String,
    ) -> Result<(), Error> {
        let mut lock = self.cached_keywords.lock().await;

        if let Some(values) = lock.get_mut(&(user_id.clone(), server_id.clone())) {
            values.push(keyword);
        } else {
            let mut highlights = self.fetch_keywords(&user_id, &server_id).await?;

            highlights.push(keyword);

            lock.put((user_id, server_id), highlights.clone());
        }

        Ok(())
    }
}
