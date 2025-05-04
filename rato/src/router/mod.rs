use crate::core::error::AppError;
use crate::global_error_handler;
use crate::router::menu_router::MenuRouter;
use crate::router::role_router::RoleRouter;
use crate::router::token_router::TokenRouter;
use crate::router::user_router::UserRouter;
use crate::state::AppState;
use axum::extract::DefaultBodyLimit;
use axum::http::Method;
use axum::response::IntoResponse;
use axum::{Extension, Router};
use std::sync::Arc;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{AllowHeaders, AllowMethods, AllowOrigin, CorsLayer, ExposeHeaders};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;

mod menu_router;
mod role_router;
mod token_router;
mod user_router;

/// 未找到对应的请求地址，返回[`AppError::NotFound`]
async fn not_found() -> impl IntoResponse {
    AppError::NotFound
}

/// 全局路由配置及中间件配置
pub struct AppRouter;

impl AppRouter {
    /// 初始化全局路由及中间件配置
    pub fn init(app_state: Arc<AppState>) -> Router {
        Router::new()
            .nest(
                "/api",
                Router::new()
                    // token路由
                    .merge(TokenRouter::init())
                    // user路由
                    .merge(UserRouter::init())
                    // role路由
                    .merge(RoleRouter::init())
                    // menu路由
                    .merge(MenuRouter::init())
                    // 全局共享状态
                    .layer(Extension(app_state))
                    // 全局异常处理
                    .layer(global_error_handler!()),
            )
            // 未找到请求地址处理
            .fallback(not_found)
            // 开启压缩
            .layer(CompressionLayer::new())
            // 单次请求最大10M
            .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024))
            // 禁用请求大小默认限制
            .layer(DefaultBodyLimit::disable())
            // 跨域设置
            .layer(CorsCfg::init())
            // 日志追踪
            .layer(TraceLayer::new_for_http())
    }
}

/// 跨域配置
pub struct CorsCfg;

impl CorsCfg {
    /// 初始化跨域配置
    pub fn init() -> CorsLayer {
        CorsLayer::new()
            // 允许任意网站访问
            .allow_origin(AllowOrigin::any())
            // 仅允许GET、POST请求方法
            .allow_methods(AllowMethods::from(vec![Method::GET, Method::POST]))
            // 允许任意请求头
            .allow_headers(AllowHeaders::any())
            // 暴露任意请求头给客户端
            .expose_headers(ExposeHeaders::any())
    }
}
