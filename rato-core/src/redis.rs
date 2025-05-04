use std::fmt::Display;
use async_trait::async_trait;
use bb8::{ManageConnection, Pool};
use redis::{FromRedisValue, ToRedisArgs};

/// 简易使用Redis
#[async_trait]
pub trait RedisPool {
    type P: ManageConnection;
    type E;

    fn get_redis_pool(&self) -> &Pool<Self::P>;

    async fn set<K, V>(&self, k: K, v: V) -> Result<(), Self::E>
    where
        K: ToRedisArgs + Sync + Send,
        V: ToRedisArgs + Sync + Send;

    async fn set_ex<K, V>(&self, k: K, v: V, expire: u64) -> Result<(), Self::E>
    where
        K: ToRedisArgs + Sync + Send,
        V: ToRedisArgs + Sync + Send;

    async fn get<K, V>(&self, k: K) -> Result<V, Self::E>
    where
        K: ToRedisArgs + Sync + Send,
        V: Sync + Send + Display + FromRedisValue;

    async fn del<K>(&self, k: K) -> Result<(), Self::E>
    where
        K: ToRedisArgs + Sync + Send;

    async fn exists<K>(&self, k: K) -> Result<(), Self::E>
    where
        K: ToRedisArgs + Sync + Send;

}