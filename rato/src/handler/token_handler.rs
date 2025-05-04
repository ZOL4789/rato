use crate::core::error::AppError;
use crate::core::result::{AppJson, R};
use crate::utils::jwt::JwtUtils;
use axum::response::IntoResponse;
use axum::{Extension};
use std::sync::Arc;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, LoaderTrait, ModelTrait, QueryFilter, Set};
use rato_core::database::DbPool;
use rato_core::redis::RedisPool;
use crate::entity::prelude::{Menu, Role, RoleMenu, User};
use crate::entity::{user};
use crate::entity::user::{LoginUserBuilder, UserBody};
use crate::core::constant::{APP_NAME, LOGIN_UID};
use crate::state::{AppState, RequestState};
use crate::utils::Utils;

/// token handler
pub struct TokenHandler;

#[allow(unused)]
impl TokenHandler {

    pub async fn register(
        Extension(app_state): Extension<Arc<AppState>>,
        AppJson(register): AppJson<UserBody>,
    ) -> Result<impl IntoResponse, AppError> {
        let transaction = app_state.begin().await?;
        let filter_users = User::find().filter(user::Column::Account.eq(&register.account)).all(&app_state.db.connection).await?;
        if !filter_users.is_empty() {
            return Err(AppError::Other("账号已存在，注册失败"));
        }
        let user = user::ActiveModel {
            account: Set(register.account),
            name: Set(register.name),
            password: Set(register.password),
            creator_id: Set(-1),
            create_time: Set(Utc::now()),
            ..Default::default()
        }.insert(&transaction).await?;
        let update_user = user::ActiveModel {
            uid: Set(user.uid),
            creator_id: Set(user.uid),
            ..Default::default()
        };
        // 更新创建人id
        let mut user = update_user.update(&transaction).await?;
        user.password = "".to_string();
        transaction.commit().await?;
        Ok(R::ok(user))
    }

    pub async fn login(
        Extension(app_state): Extension<Arc<AppState>>,
        AppJson(login): AppJson<UserBody>,
    ) -> Result<impl IntoResponse, AppError> {
        let transaction = app_state.begin().await?;
        let filter_users = User::find().filter(user::Column::Account.eq(login.account)).all(&app_state.db.connection).await?;
        if filter_users.is_empty() {
            return Err(AppError::Other("账号或密码错误"))
        }
        let user = filter_users[0].clone();
        // 获取用户角色
        let roles = user.find_related(Role).all(&app_state.db.connection).await?;
        // 获取用户菜单权限
        let menus = roles.load_many_to_many(Menu, RoleMenu, &app_state.db.connection).await?;
        let roles = roles.iter().map(|role| {
            role.value.clone()
        }).collect();
        // 菜单权限去重
        let menus = Utils::dedup(menus, |menu| menu.value.clone()).await;
        if user.password != login.password {
            return Err(AppError::Other("账号或密码错误"));
        }
        let mut login_user = LoginUserBuilder::default()
            .uid(user.uid)
            .account(Some(user.account.clone()))
            .name(user.name.clone())
            .creator_id(Some(user.creator_id))
            .create_time(Some(user.create_time))
            .updater_id(user.updater_id)
            .update_time(user.update_time)
            .perms(Some(menus))
            .roles(Some(roles))
            .build()?;
        let token = JwtUtils::create(login_user.clone(), app_state.env.jwt_secret.as_str(), 36000)
            .or_else(|e| {
                Err(AppError::Other("生成令牌失败"))
            })?;
        login_user.token = Some(token.to_string());
        app_state
            .set_ex(format!("{}:{}:{}", APP_NAME, LOGIN_UID, login_user.uid), login_user, 36000)
            .await?;
        transaction.commit().await?;
        Ok(R::ok(token))
    }

    pub async fn logout(
        Extension(app_state): Extension<Arc<AppState>>,
        Extension(request_state): Extension<Arc<RequestState>>,
    ) -> Result<impl IntoResponse, AppError> {
        let transaction = app_state.begin().await?;
        app_state
            .del(format!("{}:{}:{}", APP_NAME, LOGIN_UID, request_state.login_user.uid))
            .await?;
        transaction.commit().await?;
        Ok(R::ok(true))
    }

    pub async fn check(
        Extension(app_state): Extension<Arc<AppState>>,
        Extension(request_state): Extension<Arc<RequestState>>,
    ) -> Result<impl IntoResponse, AppError> {
        app_state
            .exists(format!("{}:{}:{}", APP_NAME, LOGIN_UID, request_state.login_user.uid))
            .await.map_err(|e| {
            AppError::Relogin("登录已失效")
        })?;
        Ok(R::ok(true))
    }

}
