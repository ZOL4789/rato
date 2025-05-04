/// 多权限检验中间件宏。提供权限字符串，如果都满足，则通过；反之抛出异常
#[macro_export]
macro_rules! require_all_perm {
    ($($perm:expr),+) => {{
        use $crate::entity::menu::RequirePermission;
        use $crate::middleware::common::AuthState;
        use $crate::require_auth;
        let state = AuthState::perm(RequirePermission::all(vec![$($perm),*]));
        require_auth![state]
    }};
}

/// 任一权限校验中间件宏。提供权限字符串，如果有一个满足，则通过；反之抛出异常
#[macro_export]
macro_rules! require_any_perm {
    ($($perm:expr),+) => {{
        use $crate::entity::menu::RequirePermission;
        use $crate::middleware::AuthState;
        use $crate::require_auth;
        let state = AuthState::perm(RequirePermission::any(vec![$($perm),*]));
        require_auth![state]
    }};
}

/// 多角色校验中间件宏。提供角色类型[`crate::entity::role::RoleType`]，如果都满足，则通过；反之抛出异常
#[macro_export]
macro_rules! require_all_role {
    ($($role:expr),+) => {{
        use $crate::entity::role::RequireRole;
        use $crate::middleware::{AuthState,CheckType};
        use $crate::require_auth;
        let state = AuthState::role(RequireRole::all(vec![$($role),*]))
        require_auth![state]
    }};
}

/// 任一角色校验中间件宏。提供角色类型[`crate::entity::role::RoleType`]，如果有一个满足，则通过；反之抛出异常
#[macro_export]
macro_rules! require_any_role {
    ($($role:expr),+) => {{
        use $crate::entity::role::RequireRole;
        use $crate::middleware::{AuthState,CheckType};
        use $crate::require_auth;
        let state = AuthState::role(RequireRole::any(vec![$($role),*]))
        require_auth![state]
    }};
}

/// 权限校验中间件宏
#[macro_export]
macro_rules! require_auth {
    ($state:expr) => {{
        use rato_core::authorizer::AuthorizerLayer;
        AuthorizerLayer::new($state)
    }};
}

/// token检验中间件宏
#[macro_export]
macro_rules! require_token {
    () => {{
        use rato_core::authenticator::AuthenticatorLayer;
        use crate::middleware::AuthState;
        AuthenticatorLayer::new(AuthState::default())
    }};
}

/// 全局异常中间件宏
#[macro_export]
macro_rules! global_error_handler {
    () => {{
        use rato_core::error_handler::GlobalErrorHandlerLayer;
        use crate::middleware::AuthState;
        GlobalErrorHandlerLayer::new(AuthState::default())
    }};
}

/// to_redis_args宏
#[macro_export]
macro_rules! to_redis_args {
    ($name:ident) => {
        #[doc(hidden)]
        impl ToRedisArgs for $name {
            // we have local variables named T1 as dummies and those
            // variables are unused.
            #[allow(non_snake_case, unused_variables)]
            fn write_redis_args<W>(&self, out: &mut W)
            where
                W: ?Sized + RedisWrite,
            {
                out.write_arg(serde_json::to_string(self).unwrap().as_ref());
            }
        }
    };
}
