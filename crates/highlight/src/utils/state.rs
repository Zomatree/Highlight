use futures::{TryStreamExt, lock::Mutex};
use lru::LruCache;
use regex::Regex;
use sqlx::PgPool;
use std::{
    collections::{HashMap, HashSet},
    num::NonZero,
    sync::Arc,
};
use tokio::sync::RwLock;

use crate::{Config, Error, create_highlight_regex};

#[derive(Clone, Debug)]
pub struct State {
    pub config: Arc<Config>,
    pub pool: PgPool,
    pub cached_keywords: Arc<Mutex<LruCache<String, HashMap<String, (Vec<String>, Regex)>>>>,
    pub cached_blocked: Arc<Mutex<LruCache<String, HashSet<String>>>>,
    pub known_not_in_server: Arc<RwLock<HashMap<String, HashSet<String>>>>,
}

impl State {
    pub async fn new() -> Self {
        let config = Arc::new(
            toml::from_str::<Config>(&std::fs::read_to_string("Highlight.toml").unwrap()).unwrap(),
        );

        let pool = PgPool::connect(&config.database.url).await.unwrap();

        let cached_keywords = Arc::new(Mutex::new(LruCache::new(NonZero::new(1000).unwrap())));
        let cached_blocked = Arc::new(Mutex::new(LruCache::new(NonZero::new(1000).unwrap())));
        let known_not_in_server = Arc::new(RwLock::new(HashMap::new()));

        Self {
            pool,
            config,
            cached_keywords,
            cached_blocked,
            known_not_in_server,
        }
    }

    pub async fn ensure_db(&self) {
        sqlx::raw_sql(include_str!("../../schema.psql"))
            .execute(&self.pool)
            .await
            .unwrap();
    }

    pub async fn fetch_keywords_for_user(
        &self,
        user_id: &str,
        server_id: &str,
    ) -> Result<Vec<String>, Error> {
        sqlx::query_scalar("select keyword from highlights where user_id=$1 and server_id=$2")
            .bind(&user_id)
            .bind(&server_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.into())
    }

    pub async fn fetch_keywords_for_server(
        &self,
        server_id: &str,
    ) -> Result<HashMap<String, (Vec<String>, Regex)>, Error> {
        let mut iter = sqlx::query_as::<_, (String, String)>(
            "select user_id, keyword from highlights where server_id=$1",
        )
        .bind(&server_id)
        .fetch(&self.pool);

        let mut mapping = HashMap::<String, Vec<String>>::new();

        while let Some((user_id, keyword)) = iter.try_next().await? {
            mapping.entry(user_id).or_default().push(keyword)
        }

        let mut regexes = HashMap::new();

        for (user_id, keywords) in mapping {
            let regex = create_highlight_regex(&keywords)?;

            regexes.insert(user_id, (keywords, regex));
        }

        Ok(regexes)
    }

    pub async fn get_keywords(
        &self,
        server_id: String,
    ) -> Result<HashMap<String, (Vec<String>, Regex)>, Error> {
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
        let mut tx = self.pool.begin().await?;

        sqlx::query("insert into highlights (user_id, server_id, keyword) values ($1, $2, $3)")
            .bind(&user_id)
            .bind(&server_id)
            .bind(&keyword)
            .execute(&mut *tx)
            .await?;

        let mut lock = self.cached_keywords.lock().await;

        if let Some(values) = lock.get_mut(&server_id) {
            if let Some((keywords, regex)) = values.get_mut(&user_id) {
                *regex = create_highlight_regex(keywords)?;

                keywords.push(keyword);
            } else {
                let regex = create_highlight_regex(&[keyword.clone()])?;
                values.insert(user_id.clone(), (vec![keyword], regex));
            }
        };

        tx.commit().await?;

        Ok(())
    }

    pub async fn remove_keyword(
        &self,
        user_id: String,
        server_id: String,
        keyword: String,
    ) -> Result<bool, Error> {
        let mut tx = self.pool.begin().await?;

        let row_count = sqlx::query(
            "delete from highlights where user_id=$1 and server_id=$2 and keyword=$3 returning *",
        )
        .bind(&user_id)
        .bind(&server_id)
        .bind(&keyword)
        .execute(&mut *tx)
        .await?
        .rows_affected();

        if row_count == 0 {
            return Ok(false);
        }

        let mut lock = self.cached_keywords.lock().await;

        if let Some(values) = lock.get_mut(&server_id) {
            if let Some((keywords, regex)) = values.get_mut(&user_id) {
                keywords.remove(
                    keywords
                        .iter()
                        .enumerate()
                        .find(|&(_, kw)| kw == &keyword)
                        .unwrap()
                        .0,
                );

                if keywords.is_empty() {
                    values.remove(&user_id);
                } else {
                    *regex = create_highlight_regex(&keywords)?;
                }
            };
        };

        Ok(true)
    }

    pub async fn clear_keywords(&self, user_id: &str, server_id: &str) -> Result<Vec<String>, Error> {
        let keywords = sqlx::query_scalar("delete from highlights where user_id=$1 and server_id=$2")
            .bind(user_id)
            .bind(server_id)
            .fetch_all(&self.pool)
            .await?;

        if let Some(server_keywords) = self.cached_keywords.lock().await.get_mut(server_id) {
            server_keywords.remove(user_id);
        };

        Ok(keywords)
    }

    pub async fn block_user(&self, user_id: String, blocked_user: String) -> Result<(), Error> {
        sqlx::query("insert into blocks(user_id, blocked_user) values($1, $2)")
            .bind(&user_id)
            .bind(&blocked_user)
            .execute(&self.pool)
            .await?;

        let mut lock = self.cached_blocked.lock().await;

        if let Some(blocked) = lock.get_mut(&user_id) {
            blocked.insert(blocked_user);
        };

        Ok(())
    }

    pub async fn unblock_user(&self, user_id: String, blocked_user: String) -> Result<(), Error> {
        sqlx::query("delete from blocks where user_id=$1 and blocked_user=$2")
            .bind(&user_id)
            .bind(&blocked_user)
            .execute(&self.pool)
            .await?;

        let mut lock = self.cached_blocked.lock().await;

        if let Some(blocked) = lock.get_mut(&user_id) {
            blocked.remove(&blocked_user);
        };

        Ok(())
    }

    pub async fn fetch_blocked_users(&self, user_id: String) -> Result<HashSet<String>, Error> {
        let mut lock = self.cached_blocked.lock().await;

        if let Some(blocked) = lock.get(&user_id) {
            Ok(blocked.clone())
        } else {
            let blocked =
                sqlx::query_scalar::<_, String>("select blocked_user from blocks where user_id=$1")
                    .bind(&user_id)
                    .fetch_all(&self.pool)
                    .await?;

            let set = HashSet::from_iter(blocked.into_iter());
            lock.put(user_id, set.clone());

            Ok(set)
        }
    }

    pub async fn get_total_keyword_count(&self) -> Result<i64, Error> {
        sqlx::query_scalar::<_, i64>("select count(keyword) from highlights")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.into())
    }
}
