use futures::{TryStreamExt, lock::Mutex};
use lru::LruCache;
use regex::Regex;
use sqlx::{PgPool, postgres::PgConnectOptions};
use std::{collections::HashMap, num::NonZero, sync::Arc};

use crate::{Config, Error};

#[derive(Clone, Debug)]
pub struct State {
    pub config: Arc<Config>,
    pub pool: PgPool,
    pub cached_keywords: Arc<Mutex<LruCache<String, HashMap<String, Regex>>>>,
    pub cached_blocked: Arc<Mutex<LruCache<String, Vec<String>>>>,
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
        let cached_blocked = Arc::new(Mutex::new(LruCache::new(NonZero::new(1000).unwrap())));

        Self {
            pool,
            config,
            cached_keywords,
            cached_blocked,
        }
    }

    pub async fn fetch_keywords_for_user(
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

    pub async fn fetch_keywords_for_server(
        &self,
        server_id: &str,
    ) -> Result<HashMap<String, Regex>, Error> {
        let mut iter = sqlx::query_as::<_, (String, String)>("select user_id, keyword from highlights where server_id=$1")
            .bind(&server_id)
            .fetch(&self.pool);

        let mut mapping = HashMap::<String, Vec<String>>::new();

        while let Some((user_id, keyword)) = iter.try_next().await? {
            mapping.entry(user_id).or_default().push(keyword)
        };

        Ok(mapping
            .into_iter()
            .map(|(user_id, keywords)| {
                let regex = Regex::new(&keywords.join("|")).unwrap();

                (user_id, regex)
            })
            .collect()
        )
    }

    pub async fn get_keywords(
        &self,
        server_id: String,
    ) -> Result<HashMap<String, Regex>, Error> {
        let mut lock = self.cached_keywords.lock().await;

        if let Some(value) = lock.get(&server_id) {
            return Ok(value.clone());
        } else {
            let keywords = self.fetch_keywords_for_server(&server_id).await?;

            lock.put(server_id, keywords.clone());

            Ok(keywords)
        }
    }

    pub async fn add_keyword(
        &self,
        user_id: String,
        server_id: String,
        keyword: String,
    ) -> Result<(), Error> {
        sqlx::query("insert into highlights (user_id, server_id, keyword) values ($1, $2, $3)")
            .bind(&user_id)
            .bind(&server_id)
            .bind(&keyword)
            .execute(&self.pool)
            .await?;

        let mut lock = self.cached_keywords.lock().await;

        if let Some(values) = lock.get_mut(&server_id) {
            if let Some(regex) = values.get_mut(&user_id) {
                *regex = Regex::new(&format!("{}|{keyword}", regex.as_str())).unwrap()
            }
        };

        Ok(())
    }
}
