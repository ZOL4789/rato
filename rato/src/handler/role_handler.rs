use crate::entity::prelude::{Role, RoleMenu};
use crate::entity::role::{AuthPermBody, RoleBody, RoleQuery};
use crate::entity::{role, role_menu};
use crate::core::error::AppError;
use crate::core::result::{AppJson, AppQuery, R};
use axum::response::IntoResponse;
use axum::Extension;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, EntityTrait, IntoActiveModel, QueryFilter, Set, ColumnTrait, QuerySelect};
use std::sync::Arc;
use rato_core::database::DbPool;
use crate::state::{AppState, RequestState};

/// 角色handler
pub struct RoleHandler;

#[allow(unused)]
impl RoleHandler {
    pub async fn add(
        Extension(app_state): Extension<Arc<AppState>>,
        Extension(request_state): Extension<Arc<RequestState>>,
        AppJson(role): AppJson<RoleBody>,
    ) -> Result<impl IntoResponse, AppError> {
        let transaction = app_state.begin().await?;
        let role = role::ActiveModel {
            name: Set(role.name),
            value: Set(role.value),
            creator_id: Set(request_state.login_user.uid),
            create_time: Set(Utc::now()),
            ..Default::default()
        };
        let role = role.insert(&transaction).await.or_else(|e|{
            tracing::error!("{:?}", e);
            Err(AppError::Other("添加角色失败"))
        })?;
        transaction.commit().await?;
        Ok(R::ok(role))
    }

    pub async fn remove(
        Extension(app_state): Extension<Arc<AppState>>,
        AppJson(role): AppJson<RoleBody>,
    ) -> Result<impl IntoResponse, AppError> {
        let role = Role::find_by_id(role.uid)
            .one(&app_state.db.connection)
            .await?.ok_or_else(|| {
            AppError::Other("未找到角色信息")
        })?;
        let transaction = app_state.begin().await?;
        Role::delete_by_id(role.uid).exec(&transaction).await.or_else(|e|{
            tracing::error!("{:?}", e);
            Err(AppError::Other("删除角色失败"))
        })?;
        transaction.commit().await?;
        Ok(R::ok(role))
    }

    pub async fn edit(
        Extension(app_state): Extension<Arc<AppState>>,
        Extension(request_state): Extension<Arc<RequestState>>,
        AppJson(update_role): AppJson<RoleBody>,
    ) -> Result<impl IntoResponse, AppError> {
        let mut role = Role::find_by_id(update_role.uid)
            .one(&app_state.db.connection)
            .await?.ok_or_else(||{
            AppError::Other("未找到角色信息")
        })?;
        let transaction = app_state.begin().await?;
        role.value = update_role.value;
        role.name = update_role.name;
        role.updater_id = Some(request_state.login_user.uid);
        role.update_time = Some(Utc::now());
        role.clone()
            .into_active_model()
            .update(&transaction)
            .await
            .or_else(|e| {
                tracing::error!("{:?}", e);
                Err(AppError::Other("更新角色失败"))
            })?;
        transaction.commit().await?;
        Ok(R::ok(role))
    }

    pub async fn info(
        Extension(app_state): Extension<Arc<AppState>>,
        Extension(_request_state): Extension<Arc<RequestState>>,
        AppQuery(role): AppQuery<RoleQuery>,
    ) -> Result<impl IntoResponse, AppError> {
        if role.uid.is_none() {
            return Err(AppError::Other("uid不能为空"));
        }
        let role = Role::find_by_id(role.uid.unwrap())
            .one(&app_state.db.connection)
            .await?.ok_or_else(||{
            AppError::Other("未找到角色信息")
        })?;
        Ok(R::ok(role))
    }

    pub async fn auth_perm(
        Extension(app_state): Extension<Arc<AppState>>,
        Extension(_request_state): Extension<Arc<RequestState>>,
        AppJson(auth_perm): AppJson<AuthPermBody>,
    ) -> Result<impl IntoResponse, AppError> {
        let role = Role::find_by_id(auth_perm.role_id)
            .select_only()
            .column(role::Column::Uid)
            .one(&app_state.db.connection)
            .await?.ok_or_else(||{
            AppError::Other("未找到角色信息")
        })?;
        let transaction = app_state.begin().await?;
        RoleMenu::delete_many()
            .filter(role_menu::Column::RoleId.eq(role.uid))
            .exec(&transaction)
            .await
            .or_else(|e| {
                tracing::error!("{:?}", e);
                Err(AppError::Other("清空角色菜单权限失败"))
            })?;
        let models = auth_perm
            .perm_uids
            .iter()
            .map(|menu_id| role_menu::ActiveModel {
                menu_id: Set(*menu_id),
                role_id: Set(role.uid),
            })
            .collect::<Vec<_>>();
        RoleMenu::insert_many(models)
            .exec(&transaction)
            .await
            .or_else(|e| {
                tracing::error!("{:?}", e);
                Err(AppError::Other("分配角色菜单权限失败"))
            })?;
        transaction.commit().await?;
        Ok(R::ok(auth_perm.perm_uids.len()))
    }
}
