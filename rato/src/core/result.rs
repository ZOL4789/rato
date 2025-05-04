use crate::core::error::AppError;
use axum::extract::{FromRequest, FromRequestParts};
use axum::http::request::Parts;
use axum::http::{StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::{Debug, Display, Formatter};

/// 响应结果
#[derive(Debug, Clone, Serialize)]
pub struct R<T> {
    pub status: bool,
    pub code: u32,
    pub data: Option<T>,
    pub msg: String,
}

impl<T> R<T> {
    pub fn new(status: bool, code: u32, data: Option<T>, msg: &str) -> Self {
        R {
            status,
            code,
            data,
            msg: msg.to_owned(),
        }
    }

    pub fn ok(data: T) -> Self {
        R {
            status: true,
            code: 200,
            data: Some(data),
            msg: "成功".to_string(),
        }
    }

    pub fn fail(msg: &str) -> Self {
        R {
            status: false,
            code: 400,
            data: None,
            msg: msg.to_owned(),
        }
    }
}

/// 响应格式
impl<T> IntoResponse for R<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        (StatusCode::from_u16(self.code as u16).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR), Json(self)).into_response()
    }
}

impl<T> Display for R<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Json(self).to_string())
    }
}

/// 自定义请求体解析器
#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(crate::core::error::AppError))]
pub struct AppJson<T>(pub T);

/// 自定义请求参数解析器
#[derive(Debug, Clone, Copy, Default)]
pub struct AppQuery<T>(pub T);

/// 自定义请求参数解析实现，实现空字符串不解析
impl<T> AppQuery<T>
where
    T: DeserializeOwned,
{
    pub fn try_from_uri(value: &Uri) -> Result<Self, AppError> {
        let raw_query = value.query().unwrap_or_default();
        // 过滤掉空值的参数
        let query = raw_query
            .split("&")
            .filter(|q| {
                let l = (&q).split("=").last().clone();
                l.is_some() && l.unwrap() != ""
            })
            .collect::<Vec<_>>()
            .join("&");
        let deserializer =
            serde_urlencoded::Deserializer::new(form_urlencoded::parse(query.as_str().as_bytes()));
        let params = serde_path_to_error::deserialize(deserializer)
            .map_err(|e| {
                tracing::error!("{}", e);
                AppError::Other("解析参数失败")
            })?;
        Ok(AppQuery(params))
    }
}

/// 自定义请求参数解析实现
impl<T, S> FromRequestParts<S> for AppQuery<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Self::try_from_uri(&parts.uri)
    }
}
