use crate::handler::menu_handler::MenuHandler;
use crate::{require_any_perm, require_token};
use axum::routing::{get, post};
use axum::Router;

pub struct MenuRouter;

/// 菜单路由
impl MenuRouter {
    pub fn init() -> Router {
        Router::new()
            .nest(
                "/menu",
                Router::new()
                    .route(
                        "/add",
                        post(MenuHandler::add).layer(require_any_perm!("menu:add")),
                    )
                    .route(
                        "/remove",
                        post(MenuHandler::remove).layer(require_any_perm!("menu:remove")),
                    )
                    .route(
                        "/edit",
                        post(MenuHandler::edit).layer(require_any_perm!("menu:edit")),
                    )
                    .route(
                        "/info",
                        get(MenuHandler::info).layer(require_any_perm!("menu:info")),
                    ),
            )
            .layer(require_token!())

    }
}
