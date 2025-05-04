use axum::extract::multipart::MultipartError;
use crate::core::result::R;
use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use base64::DecodeError;
use derive_builder::{UninitializedFieldError};
use jsonwebtoken::errors::{Error as JwtError, ErrorKind};
use redis::{FromRedisValue, RedisError, RedisResult, Value};
use sea_orm::{DbErr, SqlxError};

/// 错误类型
#[derive(Debug)]
#[allow(unused)]
pub enum AppError {
    Unlogin(&'static str),
    Relogin(&'static str),
    Unauthorized,
    NotFound,
    Other(&'static str),
    Unknown(anyhow::Error),
    JsonRejection(JsonRejection),
    SqlError(SqlxError),
    DbError(DbErr),
    BuilderError(UninitializedFieldError),
    RedisError(RedisError),
    JwtError(JwtError),
    DecodeError(DecodeError),
    MultipartError(MultipartError),
}

/// 实现IntoResponse，可直接返回AppError
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let r = match self {
            AppError::Unlogin(msg) => {
                tracing::error!("未登录：{}", msg);
                R::<String>::new(false,403, None, msg)
            }
            AppError::Relogin(msg) => {
                tracing::error!("需重新登录：{}", msg);
                R::new(false,403, None, msg)
            }
            AppError::Unauthorized => {
                tracing::error!("未授权");
                R::new(false, 401, None, "权限不足")
            }
            AppError::JsonRejection(rejection) => {
                tracing::error!("{:?}", rejection);
                R::<String>::new(
                    false,
                    rejection.status().as_u16() as u32,
                    None,
                    rejection.body_text().as_str(),
                )
            }
            AppError::SqlError(e) => {
                tracing::error!("{}", e);
                let msg = match e {
                    SqlxError::Configuration(_) => "数据库配置异常",
                    SqlxError::Database(_) => "数据库执行异常",
                    SqlxError::Io(_) => "数据库IO异常",
                    SqlxError::Tls(_) => "数据库Tls异常",
                    SqlxError::Protocol(_) => "数据库协议异常",
                    SqlxError::RowNotFound => "错误的行",
                    SqlxError::TypeNotFound { .. } => "错误的类型",
                    SqlxError::ColumnIndexOutOfBounds { .. } => "超出数据长度",
                    SqlxError::ColumnNotFound(_) => "错误的列",
                    SqlxError::ColumnDecode { .. } => "解析列失败",
                    SqlxError::Encode(_) => "加密失败",
                    SqlxError::Decode(_) => "解码失败",
                    SqlxError::AnyDriverError(_) => "数据库驱动异常",
                    SqlxError::PoolTimedOut => "数据库超时",
                    SqlxError::PoolClosed => "数据库连接池关闭",
                    SqlxError::WorkerCrashed => "数据库进程异常",
                    SqlxError::Migrate(_) => "数据库Migrate异常",
                    _ => "数据库异常",
                };
                R::new(false, 500, None, msg)
            }
            AppError::DbError(e) => {
                tracing::error!("{}", e);
                let msg = match e {
                    DbErr::ConnectionAcquire(_) => {
                        "数据库异常"
                    }
                    DbErr::TryIntoErr { .. } => {
                        "数据库异常"
                    }
                    DbErr::Conn(_) => {
                        "数据库异常"
                    }
                    DbErr::Exec(_) => {
                        "数据库异常"
                    }
                    DbErr::Query(_) => {
                        "数据库异常"
                    }
                    DbErr::ConvertFromU64(_) => {
                        "数据库异常"
                    }
                    DbErr::UnpackInsertId => {
                        "数据库异常"
                    }
                    DbErr::UpdateGetPrimaryKey => {
                        "数据库异常"
                    }
                    DbErr::RecordNotFound(_) => {
                        "数据库异常"
                    }
                    DbErr::AttrNotSet(_) => {
                        "数据库异常"
                    }
                    DbErr::Custom(_) => {
                        "数据库异常"
                    }
                    DbErr::Type(_) => {
                        "数据库异常"
                    }
                    DbErr::Json(_) => {
                        "数据库异常"
                    }
                    DbErr::Migration(_) => {
                        "数据库异常"
                    }
                    DbErr::RecordNotInserted => {
                        "数据库异常"
                    }
                    DbErr::RecordNotUpdated => {
                        "数据库异常"
                    }
                };
                R::new(false, 500, None, msg)
            }
            AppError::BuilderError(e) => {
                tracing::error!("{:?}", e);
                R::fail(format!("未初始化字段：{}", e.field_name()).as_str())
            }
            AppError::RedisError(e) => {
                tracing::error!("{:?}", e);
                R::fail("redis异常")
            }
            AppError::JwtError(e) => {
                tracing::error!("{:?}", e);
                let msg = match e.kind() {
                    ErrorKind::InvalidToken => "无效的令牌token",
                    ErrorKind::InvalidSignature => "无效的令牌signature",
                    ErrorKind::InvalidEcdsaKey => "无效的令牌Ecdsa key",
                    ErrorKind::InvalidRsaKey(_) => "无效的令牌Rsa key",
                    ErrorKind::RsaFailedSigning => "令牌Rsa签名失败",
                    ErrorKind::InvalidAlgorithmName => "无效的令牌算法名称",
                    ErrorKind::InvalidKeyFormat => "无效的令牌key格式",
                    ErrorKind::MissingRequiredClaim(_) => "未找到令牌claim",
                    ErrorKind::ExpiredSignature => "令牌已过期",
                    ErrorKind::InvalidIssuer => "无效的令牌签发者",
                    ErrorKind::InvalidAudience => "无效的令牌消费者",
                    ErrorKind::InvalidSubject => "无效的令牌sub",
                    ErrorKind::ImmatureSignature => "解析令牌失败",
                    ErrorKind::InvalidAlgorithm => "无效的令牌算法",
                    ErrorKind::MissingAlgorithm => "未找到牌算法",
                    ErrorKind::Base64(_) => "解析令牌失败",
                    ErrorKind::Json(_) => "解析令牌失败",
                    ErrorKind::Utf8(_) => "解析令牌失败",
                    ErrorKind::Crypto(_) => "解析令牌失败",
                    _ => "解析令牌失败",
                };
                R::new(false, 401, None, msg)
            }
            AppError::DecodeError(e) => {
                tracing::error!("{:?}", e);
                R::fail("Base64解码异常")
            }
            AppError::Other(msg) => {
                tracing::error!("{}", msg);
                R::fail(msg)
            }
            AppError::MultipartError(e) => {
                tracing::error!("{:?}", e);
                R::fail("处理multipart失败")
            }
            AppError::Unknown(e) => {
                tracing::error!("{:?}", e);
                R::new(false,StatusCode::INTERNAL_SERVER_ERROR.as_u16() as u32, None, "服务器异常")
            }
            AppError::NotFound => {
                R::<String>::new(false, 404, None, "未找到请求地址")
            }
        };
        r.into_response()
    }
}

impl FromRedisValue for AppError {
    fn from_redis_value(_: &Value) -> RedisResult<Self> {
        todo!()
    }
}

/// 发生JsonRejection错误时，通过?可以快速将[`JsonRejection`]转换为[`AppError`]
impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        Self::JsonRejection(rejection)
    }
}

/// 发生SqlxError错误时，通过?可以快速将[`SqlxError`]转换为[`AppError`]
impl From<SqlxError> for AppError {
    fn from(error: SqlxError) -> Self {
        Self::SqlError(error)
    }
}

/// 发生DbErr错误时，通过?可以快速将[`DbErr`]转换为[`AppError`]
impl From<DbErr> for AppError {
    fn from(error: DbErr) -> Self {
        Self::DbError(error)
    }
}

/// 发生RedisError错误时，通过?可以快速将[`RedisError`]转换为[`AppError`]
impl From<RedisError> for AppError {
    fn from(error: RedisError) -> Self {
        Self::RedisError(error)
    }
}

/// 发生JwtError错误时，通过?可以快速将[`JwtError`]转换为[`AppError`]
impl From<JwtError> for AppError {
    fn from(error: JwtError) -> Self {
        Self::JwtError(error)
    }
}

/// 发生DecodeError错误时，通过?可以快速将[`DecodeError`]转换为[`AppError`]
impl From<DecodeError> for AppError {
    fn from(error: DecodeError) -> Self {
        Self::DecodeError(error)
    }
}

/// 发生UninitializedFieldError错误时，通过?可以快速将[`UninitializedFieldError`]转换为[`AppError`]
impl From<UninitializedFieldError> for AppError {
    fn from(error: UninitializedFieldError) -> Self {
        Self::BuilderError(error)
    }
}

/// 发生MultipartError错误时，通过?可以快速将[`MultipartError`]转换为[`AppError`]
impl From<MultipartError> for AppError {
    fn from(error: MultipartError) -> Self {
        Self::MultipartError(error)
    }
}

/// 发生anyhow::Error错误时，通过?可以快速将[`anyhow::Error`]转换为[`AppError`]
impl From<anyhow::Error> for AppError {
    fn from(error: anyhow::Error) -> Self {
        Self::Unknown(error)
    }
}
