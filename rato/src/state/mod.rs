use std::fmt::Display;
use async_trait::async_trait;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use redis::{AsyncCommands, FromRedisValue, ToRedisArgs};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use rato_core::database::DbPool;
use rato_core::redis::RedisPool;
use crate::config::{DbConfig, GlobalConfig, RedisConfig};
use crate::core::error::AppError;
use crate::entity::user::LoginUser;

/// 全局共享变量
pub struct AppState {
    pub env: GlobalConfig,
    pub db: DbConfig,
    pub redis: RedisConfig
}

impl AppState {
    /// 初始化
    pub fn new(env: GlobalConfig, db: DbConfig, redis: RedisConfig) -> Self {
        AppState {
            env,
            db,
            redis
        }
    }
}

/// 实现自定义快捷操作数据库trait
#[async_trait]
#[allow(unused)]
impl DbPool for AppState {
    fn get_db_pool(&self) -> &DatabaseConnection {
        &self.db.connection
    }
}

/// 实现自定义快捷操作redis trait
#[async_trait]
#[allow(unused)]
impl RedisPool for AppState {
    type P = RedisConnectionManager;
    type E = AppError;

    fn get_redis_pool(&self) -> &Pool<Self::P> {
        &self.redis.pool
    }

    async fn set<K, V>(&self, k: K, v: V) -> Result<(), Self::E>
    where
        K: ToRedisArgs + Sync + Send,
        V: ToRedisArgs + Sync + Send,
    {
        let mut connection = self.get_redis_pool().get().await.map_err(|e| {
            tracing::error!("{:?}", e);
            AppError::Other("连接Redis异常")
        })?;
        connection.set::<K, V, String>(k, v).await?;
        Ok(())
    }

    async fn set_ex<K, V>(&self, k: K, v: V, expire: u64) -> Result<(), Self::E>
    where
        K: ToRedisArgs + Sync + Send,
        V: ToRedisArgs + Sync + Send,
    {
        let mut connection = self.get_redis_pool().get().await.map_err(|e| {
            tracing::error!("{:?}", e);
            AppError::Other("连接Redis异常")
        })?;
        connection.set_ex::<K, V, String>(k, v, expire).await?;
        Ok(())
    }

    async fn get<K, V>(&self, k: K) -> Result<V, Self::E>
    where
        K: ToRedisArgs + Sync + Send,
        V: Sync + Send + Display + FromRedisValue,
    {
        let mut connection = self.get_redis_pool().get().await.map_err(|e| {
            tracing::error!("{:?}", e);
            AppError::Other("连接Redis异常")
        })?;
        Ok(connection.get::<K, V>(k).await?)
    }

    async fn del<K>(&self, k: K) -> Result<(), Self::E>
    where
        K: ToRedisArgs + Sync + Send,
    {
        let mut connection = self.get_redis_pool().get().await.map_err(|e| {
            tracing::error!("{:?}", e);
            AppError::Other("连接Redis异常")
        })?;
        let b: bool = connection.del(k).await?;
        if !b {
            tracing::error!("{}", "删除缓存失败");
            return Err(AppError::Other("删除缓存失败"));
        }
        Ok(())
    }

    async fn exists<K>(&self, k: K) -> Result<(), Self::E>
    where
        K: ToRedisArgs + Sync + Send,
    {
        let mut connection = self.get_redis_pool().get().await.map_err(|e| {
            tracing::error!("{:?}", e);
            AppError::Other("连接Redis异常")
        })?;
        let b: bool = connection.exists(k).await?;
        if !b {
            tracing::error!("{}", "缓存不存在或已失效");
            return Err(AppError::Other("缓存不存在或已失效"));
        }
        Ok(())
    }

}

/// 认证成功后的请求变量，保存在单次请求中。不需要认证的handler无法获取
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestState {
    pub login_user: LoginUser,
}

