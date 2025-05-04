use crate::handler::token_handler::TokenHandler;
use crate::{require_any_perm, require_token};
use axum::routing::{get, post};
use axum::Router;

pub struct TokenRouter;

/// token路由
impl TokenRouter {
    pub fn init() -> Router {
        Router::new().nest(
            "/token",
            Router::new()
                .route("/register", post(TokenHandler::register))
                .route("/login", post(TokenHandler::login))
                .route(
                    "/logout",
                    post(TokenHandler::logout)
                        .layer(require_any_perm!("token:logout"))
                        .layer(require_token!()),
                )
                .route(
                    "/check",
                    get(TokenHandler::check)
                        .layer(require_any_perm!("token:check"))
                        .layer(require_token!()),
                ),
        )
    }
}
