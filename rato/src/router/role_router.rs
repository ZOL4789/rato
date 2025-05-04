
use crate::handler::role_handler::RoleHandler;
use crate::{require_any_perm, require_token};
use axum::routing::{get, post};
use axum::Router;

pub struct RoleRouter;

/// 角色路由
impl RoleRouter {
    pub fn init() -> Router {
        Router::new()
            .nest(
                "/role",
                Router::new()
                    .route(
                        "/add",
                        post(RoleHandler::add).layer(require_any_perm!("role:add")),
                    )
                    .route(
                        "/remove",
                        post(RoleHandler::remove).layer(require_any_perm!("role:remove")),
                    )
                    .route(
                        "/edit",
                        post(RoleHandler::edit).layer(require_any_perm!("role:edit")),
                    )
                    .route(
                        "/info",
                        get(RoleHandler::info).layer(require_any_perm!("role:info")),
                    )
                    .route(
                        "/authperm",
                        post(RoleHandler::auth_perm).layer(require_any_perm!("role:authperm")),
                    ),
            )
            .layer(require_token!())

    }
}
