use std::convert::Infallible;
use axum::extract::{Request};
use axum::response::{IntoResponse, Response};
use std::fmt::Debug;
use std::task::{Context, Poll};
use tower_layer::Layer;
use tower_service::Service;
use crate::future::ResFuture;

/// 授权
pub trait Authorizer {
    type Err: IntoResponse;
    fn authorize<Body>(&self, req: &mut Request<Body>) -> Result<(), Self::Err>;
}

#[derive(Clone, Copy, Debug)]
pub struct AuthorizerLayer<Cxt>
where
    Cxt: Authorizer + Clone,
{
    pub(crate) context: Cxt,
}

impl<Cxt> AuthorizerLayer<Cxt>
where
    Cxt: Authorizer + Clone,
{
    pub fn new(context: Cxt) -> Self {
        Self { context }
    }
}

impl<Svc, Cxt> Layer<Svc> for AuthorizerLayer<Cxt>
where
    Cxt: Authorizer + Clone,
{
    type Service = AuthorizerService<Svc, Cxt>;

    fn layer(&self, inner: Svc) -> Self::Service {
        AuthorizerService {
            inner,
            context: self.context.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AuthorizerService<Svc, Cxt>
where
    Cxt: Authorizer + Clone,
{
    pub(crate) inner: Svc,
    pub(crate) context: Cxt,
}

impl<Svc, Cxt> AuthorizerService<Svc, Cxt>
where
    Cxt: Authorizer + Clone,
{
    pub fn new(inner: Svc, context: Cxt) -> Self {
        Self { inner, context }
    }

    pub fn layer(context: Cxt) -> AuthorizerLayer<Cxt> {
        AuthorizerLayer::new(context)
    }
}

impl<Body, Svc, Cxt, Err> Service<Request<Body>> for AuthorizerService<Svc, Cxt>
where
    Svc: Service<Request<Body>, Response = Response, Error = Infallible>,
    Svc:: Error: Debug,
    Cxt: Authorizer<Err = Err> + Clone,
    Err: IntoResponse + Clone,
{
    type Response = Svc::Response;
    type Error = Svc::Error;
    type Future = ResFuture<Err, Svc::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        if let Err(e) = self.context.authorize(&mut req) {
            return ResFuture::err(e);
        }
        ResFuture::new(self.inner.call(req))
    }
}
