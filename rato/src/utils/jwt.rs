use crate::entity::user::LoginUser;
use chrono::{Duration, Utc};
use jsonwebtoken::errors::Error;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

/// Jwt Claims。签发token保存的数据
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Claims {
    pub exp: usize,
    pub iat: usize,
    pub sub: String,
    pub login_user: LoginUser,
}

/// Jwt工具类
pub struct JwtUtils;

impl JwtUtils {
    /// 签发token
    pub fn create(login_user: LoginUser, secret: &str, expiration: i64) -> Result<String, Error> {
        let now = Utc::now();
        let iat = now.timestamp() as usize;
        let exp = (now + Duration::minutes(expiration)).timestamp() as usize;
        let claims = Claims {
            iat,
            exp,
            sub: login_user.uid.to_string(),
            login_user,
        };
        jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .or_else(|e| {
            tracing::error!("{:?}", e);
            Err(e)
        })
    }

    /// 根据token和secret解析Jwt Claims
    pub fn decode(token: &str, secret: &str) -> Result<Claims, Error> {
        Ok(jsonwebtoken::decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        )
        .map(|claims| claims.claims)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::entity::user::LoginUserBuilder;
    use crate::entity::user::{LoginUser, LoginUserBuilder};
    use crate::utils::jwt::JwtUtils;

    #[test]
    fn jwt_test() {
        let secret = "123456";
        let result = JwtUtils::create(
            LoginUserBuilder::default()
                .uid(1)
                .name("name".to_string())
                .roles(Some(vec!["user".to_string(), "admin".to_string()]))
                .perms(Some(vec!["user".to_string(), "admin".to_string()]))
                .build()
                .unwrap(),
            secret,
            3600,
        )
        .unwrap();
        let user = JwtUtils::decode(result.as_str(), secret)
            .unwrap()
            .login_user;
        tracing::error!("{:#?}", user);
        assert_eq!(user.name, "name");
        assert_eq!(user.account, Some("account".to_string()));
    }
}
