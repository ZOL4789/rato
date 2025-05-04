use axum::body::Body;
use axum::extract::Request;
use axum::response::{IntoResponse, Response};
use futures_util::future::BoxFuture;
use std::fmt::Debug;
use std::panic;
use std::panic::AssertUnwindSafe;
use std::task::{Context, Poll};
use tower_layer::Layer;
use tower_service::Service;

/// 全局异常处理
pub trait ErrorHandler {
    // TODO 尚未找到req该如何实现Copy Trait
    // fn msg<Body>(&self, req: &Request<Body>) -> impl IntoResponse;
    fn msg(&self) -> impl IntoResponse;
}

#[derive(Clone)]
pub struct GlobalErrorHandlerLayer<T> {
    pub context: T,
}

impl<T> GlobalErrorHandlerLayer<T> {
    pub fn new(context: T) -> Self {
        Self { context }
    }
}

impl<S, T> Layer<S> for GlobalErrorHandlerLayer<T>
where
    T: ErrorHandler + Send + Clone,
{
    type Service = GlobalErrorHandlerService<S, T>;

    fn layer(&self, inner: S) -> Self::Service {
        GlobalErrorHandlerService {
            inner,
            context: self.context.clone(),
        }
    }
}

#[derive(Clone)]
pub struct GlobalErrorHandlerService<S, T> {
    inner: S,
    pub context: T,
}

impl<S, B, T> Service<Request<B>> for GlobalErrorHandlerService<S, T>
where
    S: Service<Request<B>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    B: Send + 'static,
    S::Error: Debug,
    T: ErrorHandler + Send + Clone + 'static,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        let msg = self.context.msg().into_response();
        Box::pin(async move {
            let fut = AssertUnwindSafe(inner.call(req));
            match panic::catch_unwind(|| {
                tokio::task::block_in_place(move || tokio::runtime::Handle::current().block_on(fut))
            }) {
                Ok(res) => Ok(res.unwrap()),
                Err(_) => Ok(msg),
            }
        })
    }
}
