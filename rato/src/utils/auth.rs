use crate::core::error::AppError;
use std::collections::HashSet;
use crate::entity::role::RoleType;
use crate::entity::user::LoginUser;
use crate::middleware::{AuthState, CheckType};

/// 角色权限检验工具
pub trait AuthUtils<Cxt, Err, Role, Perm> {
    fn get_all_roles(&self) -> Result<HashSet<Role>, Err>;

    fn has_all_roles(&self, roles: Vec<Role>) -> bool;

    fn has_any_role(&self, roles: Vec<Role>) -> bool;

    fn get_all_perms(&self) -> Result<HashSet<Perm>, Err>;

    fn has_all_perms(&self, roles: Vec<Perm>) -> bool;

    fn has_any_perm(&self, roles: Vec<Perm>) -> bool;

    fn auth(&self, context: &Cxt) -> bool;
}

impl AuthUtils<AuthState, AppError, RoleType, String> for LoginUser {
    fn get_all_roles(&self) -> Result<HashSet<RoleType>, AppError> {
        if self.roles.is_none() {
            return Err(AppError::Other("无任何角色"));
        }
        Ok(self
            .roles
            .clone()
            .unwrap()
            .iter()
            .map(|x| RoleType::from_str(&x))
            .filter(|r| r.is_some())
            .map(|r| r.unwrap())
            .collect::<HashSet<_>>())
    }

    fn has_all_roles(&self, roles: Vec<RoleType>) -> bool {
        let role_set = match self.get_all_roles() {
            Ok(r) => r,
            Err(_) => return false,
        };
        roles.iter().all(|x| role_set.contains(&x))
    }

    fn has_any_role(&self, roles: Vec<RoleType>) -> bool {
        let role_set = match self.get_all_roles() {
            Ok(r) => r,
            Err(_) => return false,
        };
        roles.iter().any(|x| role_set.contains(&x))
    }

    fn get_all_perms(&self) -> Result<HashSet<String>, AppError> {
        if self.perms.is_none() {
            return Err(AppError::Other("无任何权限"));
        }
        let option = self.perms.clone();
        Ok(option
            .unwrap()
            .iter()
            .map(|x| x.clone())
            .collect::<HashSet<_>>())
    }

    fn has_all_perms(&self, perms: Vec<String>) -> bool {
        let perm_set = match self.get_all_perms() {
            Ok(p) => p,
            Err(_) => return false,
        };
        perms.iter().all(|x| perm_set.contains(x))
    }

    fn has_any_perm(&self, perms: Vec<String>) -> bool {
        let perm_set = match self.get_all_perms() {
            Ok(p) => p,
            Err(_) => return false,
        };
        perms.iter().any(|x| perm_set.contains(x))
    }

    fn auth(&self, auth_state: &AuthState) -> bool {
        if self.uid == 1 {
            return true;
        }
        if auth_state.role.is_some() && auth_state.perm.is_some() {
            let p = auth_state.perm.clone().unwrap();
            let pb = match p.check_type {
                CheckType::And => self.has_all_perms(p.perms),
                CheckType::Or => self.has_any_perm(p.perms),
            };
            let r = auth_state.role.clone().unwrap();
            let pr = match r.check_type {
                CheckType::And => self.has_all_roles(r.roles),
                CheckType::Or => self.has_any_role(r.roles),
            };
            return match auth_state.check_type {
                CheckType::And => pb && pr,
                CheckType::Or => pb || pr,
            };
        }
        if auth_state.perm.is_some() {
            let p = auth_state.perm.clone().unwrap();
            return match p.check_type {
                CheckType::And => self.has_all_perms(p.perms),
                CheckType::Or => self.has_any_perm(p.perms),
            };
        }
        if auth_state.role.is_some() {
            let r = auth_state.role.clone().unwrap();
            return match r.check_type {
                CheckType::And => self.has_all_roles(r.roles),
                CheckType::Or => self.has_any_role(r.roles),
            };
        }
        true
    }
}


#[cfg(test)]
mod tests {
    use crate::utils::auth::AuthUtils;
    use crate::entity::role::{RoleType};
    use crate::entity::user::LoginUserBuilder;

    #[test]
    fn check_perms() {
        let user = LoginUserBuilder::default()
            .uid(1)
            .name("name".to_string())
            .roles(Some(vec![
                RoleType::Admin.to_string(),
                RoleType::User.to_string(),
            ]))
            .perms(Some(vec!["user".to_string(), "admin".to_string()]))
            .build()
            .unwrap();
        assert_eq!(user.has_any_perm(vec!["user:me".to_string()]), true);
        assert_eq!(user.has_any_perm(vec!["user:me1".to_string()]), false);
        assert_eq!(
            user.has_all_perms(vec!["user:me".to_string(), "token:logout".to_string()]),
            true
        );
        assert_eq!(user.has_all_perms(vec!["user:me".to_string()]), true);
        assert_eq!(
            user.has_all_perms(vec!["user:me".to_string(), "token:logout".to_string()]),
            true
        );
    }

    #[test]
    fn check_roles() {
        let user = LoginUserBuilder::default()
            .uid(1)
            .name("name".to_string())
            .roles(Some(vec![
                RoleType::Admin.to_string(),
                RoleType::User.to_string(),
            ]))
            .perms(Some(vec!["user".to_string(), "admin".to_string()]))
            .build()
            .unwrap();
        assert_eq!(user.has_any_role(vec![RoleType::User]), true);
        assert_eq!(user.has_any_role(vec![RoleType::Api]), false);
        assert_eq!(user.has_all_roles(vec![RoleType::Admin]), true);
        assert_eq!(
            user.has_all_roles(vec![RoleType::Admin, RoleType::Api]),
            false
        );
    }

}
