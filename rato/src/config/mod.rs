use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use config::{Config, Environment};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use serde::Deserialize;

/// 全局变量配置
#[derive(Debug, Clone, Deserialize)]
pub struct GlobalConfig {
    // 端口
    pub server_port: u32,
    // host
    pub server_host: String,
    // jwt秘钥
    pub jwt_secret: String,
    // 数据库url
    pub database_url: String,
    // redis url
    pub redis_url: String
}

impl GlobalConfig {
    /// 从系统环境变量及.env文件中解析
    pub fn init() -> Self {
        let cfg = Config::builder()
            .add_source(Environment::default())
            .build()
            .expect("初始化环境变量失败");
        cfg.try_deserialize::<GlobalConfig>().expect("反序列化环境变量失败")
    }
}

/// Redis配置
pub struct RedisConfig {
    pub pool: Pool<RedisConnectionManager>,
}

impl RedisConfig {
    /// 根据url初始化redis连接
    pub async fn init(url: &str) -> Self {
        tracing::info!("Connected to redis：{}", &url);
        let manager = RedisConnectionManager::new(url).unwrap_or_else(|e| {
            tracing::error!("{:?}", e);
            std::process::exit(1);
        });
        let pool = match Pool::builder().build(manager).await {
            Ok(pool) => pool,
            Err(e) => {
                tracing::error!("{:?}", e);
                std::process::exit(1);
            }
        };

        RedisConfig { pool }
    }
}

/// 数据库配置
#[derive(Debug, Clone)]
pub struct DbConfig {
    pub connection: DatabaseConnection,
}

impl DbConfig {
    /// 根据url初始化数据库配置
    pub async fn init(url: &str) -> Self {
        let mut options = ConnectOptions::new(url);
        options.max_connections(20);
        let connection = match Database::connect(options).await {
            Ok(connection) => {
                tracing::info!("Connected to {}", &url);
                connection
            }
            Err(e) => {
                tracing::error!("{:?}", e);
                panic!("Failed to connect to database {}", &url);
            }
        };
        DbConfig { connection }
    }
}