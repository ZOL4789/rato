
use crate::handler::user_handler::UserHandler;
use axum::routing::{get, post};
use axum::Router;
use crate::{require_any_perm, require_token};

pub struct UserRouter;

/// 用户路由
impl UserRouter {
    pub fn init() -> Router {
        Router::new()
            .nest(
                "/user",
                Router::new()
                    .route(
                        "/me",
                        get(UserHandler::me).layer(require_any_perm!("user:me")),
                    )
                    .route(
                        "/info",
                        get(UserHandler::info).layer(require_any_perm!("user:info")),
                    )
                    .route(
                        "/authrole",
                        post(UserHandler::auth_role).layer(require_any_perm!("user:authrole")),
                    ),
            )
            .layer(require_token!())
    }
}
