use axum::http::StatusCode;
use axum::response::{IntoResponse};
use axum::routing::get;
use axum::Router;
use rato_core::error_handler::{ErrorHandler, GlobalErrorHandlerLayer};
use std::io::Error;
use tokio::net::TcpListener;

/// 测试panic，全局异常捕获
async fn err() -> Result<impl IntoResponse, Error> {
    let _ = 1 / 0;
    Ok("测试错误")
}

#[derive(Clone)]
pub struct AppState;

/// 实现全局异常捕获trait，返回自定义消息
impl ErrorHandler for AppState {
    fn msg(&self) -> impl IntoResponse {
        (StatusCode::INTERNAL_SERVER_ERROR, "全局异常消息")
    }
}

#[tokio::main]
async fn main() {
    let router = Router::new()
        .route("/test", get(err))
        .layer(GlobalErrorHandlerLayer::new(AppState {}));
    let listener = TcpListener::bind("127.0.0.1:8980").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
