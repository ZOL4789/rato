use crate::core::error::AppError;
use crate::core::result::{AppJson, AppQuery, R};
use crate::entity::menu;
use crate::entity::menu::{MenuBody, MenuQuery};
use crate::entity::prelude::{Menu, Role};
use crate::state::{AppState, RequestState};
use axum::response::IntoResponse;
use axum::Extension;
use chrono::Utc;
use rato_core::database::DbPool;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, ModelTrait, Set, TryIntoModel};
use std::sync::Arc;

/// 菜单handler
pub struct MenuHandler;

#[allow(unused)]
impl MenuHandler {
    /// 添加菜单
    pub async fn add(
        Extension(app_state): Extension<Arc<AppState>>,
        Extension(request_state): Extension<Arc<RequestState>>,
        AppJson(menu): AppJson<MenuBody>,
    ) -> Result<impl IntoResponse, AppError> {
        let transaction = app_state.begin().await?;
        let menu = menu::ActiveModel {
            name: Set(menu.name),
            value: Set(menu.value),
            creator_id: Set(request_state.login_user.uid),
            create_time: Set(Utc::now()),
            ..Default::default()
        };
        Menu::insert(menu.clone())
            .exec(&transaction)
            .await
            .or_else(|e| {
                tracing::error!("{:?}", e);
                Err(AppError::Other("添加菜单失败"))
            })?;
        transaction.commit().await?;
        Ok(R::ok(menu.try_into_model()?))
    }

    /// 删除菜单
    pub async fn remove(
        Extension(app_state): Extension<Arc<AppState>>,
        Extension(_request_state): Extension<Arc<RequestState>>,
        AppJson(menu): AppJson<MenuBody>,
    ) -> Result<impl IntoResponse, AppError> {
        let menu = Role::find_by_id(menu.uid)
            .one(&app_state.db.connection)
            .await?
            .ok_or_else(|| AppError::Other("未找到菜单信息"))?;
        let transaction = app_state.begin().await?;
        menu.clone().delete(&transaction).await.or_else(|e| {
            tracing::error!("{:?}", e);
            Err(AppError::Other("删除菜单失败"))
        })?;
        transaction.commit().await?;
        Ok(R::ok(menu.try_into_model()?))
    }

    /// 编辑菜单
    pub async fn edit(
        Extension(app_state): Extension<Arc<AppState>>,
        Extension(request_state): Extension<Arc<RequestState>>,
        AppJson(update_menu): AppJson<MenuBody>,
    ) -> Result<impl IntoResponse, AppError> {
        let mut menu = Role::find_by_id(update_menu.uid)
            .one(&app_state.db.connection)
            .await?
            .ok_or_else(|| AppError::Other("未找到菜单信息"))?;
        let transaction = app_state.begin().await?;
        menu.value = update_menu.value;
        menu.name = update_menu.name;
        menu.updater_id = Some(request_state.login_user.uid);
        menu.update_time = Some(Utc::now());
        menu.clone()
            .into_active_model()
            .update(&transaction)
            .await
            .or_else(|e| {
                tracing::error!("{:?}", e);
                Err(AppError::Other("更新菜单失败"))
            })?;
        transaction.commit().await?;
        Ok(R::ok(menu))
    }

    /// 查询菜单
    pub async fn info(
        Extension(app_state): Extension<Arc<AppState>>,
        Extension(_request_state): Extension<Arc<RequestState>>,
        AppQuery(menu): AppQuery<MenuQuery>,
    ) -> Result<impl IntoResponse, AppError> {
        if menu.uid.is_none() {
            return Err(AppError::Other("uid不能为空"));
        }
        let menu = Role::find_by_id(menu.uid.unwrap())
            .one(&app_state.db.connection)
            .await?
            .ok_or_else(|| AppError::Other("未找到菜单信息"))?;
        Ok(R::ok(menu))
    }
}
