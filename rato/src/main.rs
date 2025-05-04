mod state;
mod core;
mod handler;
mod middleware;
mod router;
mod utils;
mod entity;
mod config;

use dotenv::dotenv;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;
use tracing_subscriber::filter::LevelFilter;
use crate::config::{DbConfig, GlobalConfig, RedisConfig};
use crate::router::AppRouter;
use crate::state::AppState;

/// 入口函数
#[tokio::main]
async fn main() {
    // 日志订阅
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .init();
    // 环境变量解析
    dotenv().ok();
    let config = GlobalConfig::init();
    let server = format!("{}:{}", &config.server_host, &config.server_port);
    tracing::info!("Server starting at {}", server);
    // 监听ip及端口
    let listener = TcpListener::bind(server).await.unwrap();
    // 初始化路由注册并启动
    axum::serve(
        listener,
        AppRouter::init(Arc::new(AppState::new(
            config.clone(),
            DbConfig::init(&config.database_url).await,
            RedisConfig::init(&config.redis_url).await,
        ))),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();

}

/// 优雅关闭
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            println!("received Ctrl+C");
        },
        _ = terminate => {
            println!("received terminate");
        },
    }
}
