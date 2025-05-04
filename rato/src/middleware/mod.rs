use crate::core::constant::{APP_NAME, LOGIN_UID};
use crate::core::result::R;
use crate::entity::menu::RequirePermission;
use crate::entity::role::RequireRole;
use crate::state::{AppState, RequestState};
use crate::utils::auth::AuthUtils;
use crate::utils::jwt::JwtUtils;
use axum::extract::Request;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::Arc;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use rato_core::authenticator::Authenticator;
use rato_core::authorizer::Authorizer;
use rato_core::error_handler::ErrorHandler;
use rato_core::redis::RedisPool;

#[derive(Deserialize, PartialEq, Debug, Serialize, Clone, Default)]
pub enum CheckType {
    #[default]
    Or,
    And,
}

#[derive(Deserialize, Debug, Serialize, Clone, Default)]
pub struct AuthState {
    pub perm: Option<RequirePermission>,
    pub role: Option<RequireRole>,
    pub check_type: CheckType,
}

#[allow(unused)]
impl AuthState {
    pub fn all(perm: RequirePermission, role: RequireRole) -> AuthState {
        AuthState {
            perm: Some(perm),
            role: Some(role),
            check_type: CheckType::And,
        }
    }
    pub fn any(perm: RequirePermission, role: RequireRole) -> AuthState {
        AuthState {
            perm: Some(perm),
            role: Some(role),
            check_type: CheckType::Or,
        }
    }
    pub fn perm(perm: RequirePermission) -> AuthState {
        AuthState {
            perm: Some(perm),
            role: None,
            check_type: CheckType::Or,
        }
    }
    pub fn role(role: RequireRole) -> AuthState {
        AuthState {
            perm: None,
            role: Some(role),
            check_type: CheckType::Or,
        }
    }
}

/// 自定义认证实现
impl Authenticator for AuthState {
    type Err = R<String>;

    fn authenticate<Body>(&self, req: &mut Request<Body>) -> Result<(), Self::Err> {
        let token = req
            .headers()
            .get("Authorization")
            .and_then(|header| header.to_str().ok())
            .and_then(|header| {
                if header.starts_with("Bearer ") {
                    Some(header[7..].to_owned())
                } else {
                    None
                }
            })
            .ok_or_else(|| R::fail("未找到令牌"))?;
        let app_state = req.extensions_mut().get::<Arc<AppState>>().unwrap();
        let result = JwtUtils::decode(token.as_str(), app_state.env.jwt_secret.as_str());
        let mut login_user = match result {
            Ok(claims) => claims.login_user,
            Err(e) => return Err(R::fail(format!("{}", e).as_str())),
        };
        login_user.token = Some(token);
        let future = async {
            app_state
                .exists(format!("{}:{}:{}", APP_NAME, LOGIN_UID, login_user.uid))
                .await
                .map_err(|_| return R::new(false, 401, None, "登录已失效，请重新登录"))
        };
        tokio::task::block_in_place(move || tokio::runtime::Handle::current().block_on(future))?;
        req.extensions_mut()
            .insert(Arc::new(RequestState { login_user }));
        Ok(())
    }
}

/// 自定义授权实现
impl Authorizer for AuthState {
    type Err = R<String>;

    fn authorize<Body>(&self, req: &mut Request<Body>) -> Result<(), Self::Err> {
        let request_state = req.extensions_mut().get::<Arc<RequestState>>().unwrap();
        if request_state.login_user.uid != 1 {
            if let false = request_state.login_user.auth(&self) {
                return Err(R::fail("权限不足"));
            }
        }
        Ok(())
    }
}

/// 自定义全局异常返回
impl ErrorHandler for AuthState {
    fn msg(&self) -> impl IntoResponse {
        R::<String>::new(false, StatusCode::INTERNAL_SERVER_ERROR.as_u16() as u32, None, "服务器异常")
    }
}