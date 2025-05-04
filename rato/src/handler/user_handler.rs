use crate::entity::prelude::{Menu, Role, RoleMenu, User, UserRole};
use crate::entity::user::{AuthRoleBody, LoginUserBuilder, UserQuery};
use crate::entity::{user_role};
use crate::core::error::AppError;
use crate::core::result::{AppJson, AppQuery, R};
use axum::response::IntoResponse;
use axum::Extension;
use rato_core::database::DbPool;
use sea_orm::{ColumnTrait, EntityTrait, LoaderTrait, ModelTrait, QueryFilter, Set};
use std::sync::Arc;
use crate::state::{AppState, RequestState};
use crate::utils::Utils;

/// 用户handler
pub struct UserHandler;

#[allow(unused)]
impl UserHandler {
    pub async fn me(
        Extension(app_state): Extension<Arc<AppState>>,
        Extension(request_state): Extension<Arc<RequestState>>,
    ) -> Result<impl IntoResponse, AppError> {
        let user = User::find_by_id(request_state.login_user.uid)
            .one(&app_state.db.connection)
            .await?.ok_or_else(||{
            AppError::Other("未找到用户信息")
        })?;
        // 获取用户角色
        let roles = user
            .find_related(Role)
            .all(&app_state.db.connection)
            .await?;
        // 获取用户菜单权限
        let menus = roles
            .load_many_to_many(Menu, RoleMenu, &app_state.db.connection)
            .await?;
        let roles = roles.iter().map(|role| role.value.clone()).collect();
        // 去重
        let menus = Utils::dedup(menus, |menu| menu.value.clone()).await;
        Ok(R::ok(
            LoginUserBuilder::default()
                .uid(user.uid)
                .account(Some(user.account))
                .name(user.name)
                .token(request_state.login_user.token.clone())
                .creator_id(Some(user.creator_id))
                .create_time(Some(user.create_time))
                .updater_id(user.updater_id)
                .update_time(user.update_time)
                .perms(Some(menus))
                .roles(Some(roles))
                .build()?,
        ))
    }

    pub async fn info(
        Extension(app_state): Extension<Arc<AppState>>,
        Extension(_request_state): Extension<Arc<RequestState>>,
        AppQuery(user): AppQuery<UserQuery>,
    ) -> Result<impl IntoResponse, AppError> {
        if user.uid.is_none() {
            return Err(AppError::Other("uid不能为空"));
        }
        let user = User::find_by_id(user.uid.unwrap())
            .one(&app_state.db.connection)
            .await?.ok_or_else(||{
            AppError::Other("未找到用户信息")
        })?;
        Ok(R::ok(user))
    }

    pub async fn auth_role(
        Extension(app_state): Extension<Arc<AppState>>,
        Extension(_request_state): Extension<Arc<RequestState>>,
        AppJson(auth_role): AppJson<AuthRoleBody>,
    ) -> Result<impl IntoResponse, AppError> {
        let user = User::find_by_id(auth_role.user_id)
            .one(&app_state.db.connection)
            .await?.ok_or_else(||{
            AppError::Other("未找到用户信息")
        })?;
        let transaction = app_state.begin().await?;
        UserRole::delete_many()
            .filter(user_role::Column::UserId.eq(user.uid))
            .exec(&transaction)
            .await
            .or_else(|e| {
                tracing::error!("{:?}", e);
                Err(AppError::Other("清空用户角色失败"))
            })?;
        let models = auth_role
            .role_uids
            .iter()
            .map(|role_id| user_role::ActiveModel {
                user_id: Set(user.uid),
                role_id: Set(*role_id),
            })
            .collect::<Vec<_>>();
        UserRole::insert_many(models)
            .exec(&transaction)
            .await
            .or_else(|e| {
                tracing::error!("{:?}", e);
                Err(AppError::Other("分配角色失败"))
            })?;
        transaction.commit().await?;
        Ok(R::ok(auth_role.role_uids.len()))
    }
}
