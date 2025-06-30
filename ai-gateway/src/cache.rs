use http_cache::{CacheManager, HttpResponse, MokaManager, Result};
use http_cache_semantics::CachePolicy;
use r2d2::Pool;
use redis::{Client, Commands};
use serde::{Deserialize, Serialize};

use crate::error::init::InitError;

#[derive(Debug, Clone)]
pub enum CacheClient {
    Redis(RedisCacheManager),
    Moka(MokaManager),
}

#[derive(Debug, Clone)]
pub struct RedisCacheManager {
    pool: Pool<Client>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Store {
    response: HttpResponse,
    policy: CachePolicy,
}

impl RedisCacheManager {
    pub fn new(url: url::Url) -> std::result::Result<Self, InitError> {
        let client = Client::open(url)?;
        let pool = Pool::builder().build(client)?;
        Ok(Self { pool })
    }
}

#[async_trait::async_trait]
impl CacheManager for RedisCacheManager {
    async fn get(
        &self,
        cache_key: &str,
    ) -> Result<Option<(HttpResponse, CachePolicy)>> {
        let mut conn = self.pool.get()?;
        let value: String = conn.get(cache_key)?;
        let store: Store = serde_json::from_str(&value)?;
        Ok(Some((store.response, store.policy)))
    }

    async fn put(
        &self,
        cache_key: String,
        response: HttpResponse,
        policy: CachePolicy,
    ) -> Result<HttpResponse> {
        let mut conn = self.pool.get()?;
        let store = Store {
            response: response.clone(),
            policy,
        };
        let serialized = serde_json::to_string(&store)?;
        let _: () = conn.set(cache_key, serialized)?;
        Ok(response)
    }

    async fn delete(&self, cache_key: &str) -> Result<()> {
        let mut conn = self.pool.get()?;
        let _: () = conn.del(cache_key)?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl CacheManager for CacheClient {
    async fn get(
        &self,
        cache_key: &str,
    ) -> Result<Option<(HttpResponse, CachePolicy)>> {
        match self {
            CacheClient::Redis(redis) => redis.get(cache_key).await,
            CacheClient::Moka(moka) => moka.get(cache_key).await,
        }
    }

    async fn put(
        &self,
        cache_key: String,
        response: HttpResponse,
        policy: CachePolicy,
    ) -> Result<HttpResponse> {
        match self {
            CacheClient::Redis(redis) => {
                redis.put(cache_key, response, policy).await
            }
            CacheClient::Moka(moka) => {
                moka.put(cache_key, response, policy).await
            }
        }
    }

    async fn delete(&self, cache_key: &str) -> Result<()> {
        match self {
            CacheClient::Redis(redis) => redis.delete(cache_key).await,
            CacheClient::Moka(moka) => moka.delete(cache_key).await,
        }
    }
}
