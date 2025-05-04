use crate::future::ResFuture;
use axum::extract::Request;
use axum::response::{IntoResponse, Response};
use std::convert::Infallible;
use std::fmt::Debug;
use std::task::{Context, Poll};
use tower_layer::Layer;
use tower_service::Service;

/// 认证
pub trait Authenticator {
    type Err: IntoResponse;
    fn authenticate<Body>(&self, req: &mut Request<Body>) -> Result<(), Self::Err>;
}

#[derive(Clone, Copy, Debug)]
pub struct AuthenticatorLayer<Cxt>
where
    Cxt: Authenticator + Clone,
{
    pub(crate) context: Cxt,
}

impl<Cxt> AuthenticatorLayer<Cxt>
where
Cxt: Authenticator + Clone,
{
    pub fn new(context: Cxt) -> Self {
        Self { context }
    }
}

impl<Svc, Cxt> Layer<Svc> for AuthenticatorLayer<Cxt>
where
    Cxt: Authenticator + Clone,
{
    type Service = AuthenticatorService<Svc, Cxt>;

    fn layer(&self, inner: Svc) -> Self::Service {
        AuthenticatorService {
            inner,
            context: self.context.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AuthenticatorService<Svc, Cxt>
where
    Cxt: Authenticator + Clone,
{
    pub(crate) inner: Svc,
    pub(crate) context: Cxt,
}

impl<Svc, Cxt> AuthenticatorService<Svc, Cxt>
where
    Cxt: Authenticator + Clone,
{
    pub fn new(inner: Svc, context: Cxt) -> Self {
        Self { inner, context }
    }

    pub fn layer(context: Cxt) -> AuthenticatorLayer<Cxt> {
        AuthenticatorLayer::new(context)
    }
}

impl<Body, Svc, Cxt, Err> Service<Request<Body>> for AuthenticatorService<Svc, Cxt>
where
    Svc: Service<Request<Body>, Response = Response, Error = Infallible>,
    Svc::Error: Debug,
    Cxt: Authenticator<Err = Err> + Clone,
    Err: IntoResponse + Clone,
{
    type Response = Response;
    type Error = Svc::Error;
    type Future = ResFuture<Err, Svc::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        if let Err(e) = self.context.authenticate(&mut req) {
            return ResFuture::err(e);
        }

        ResFuture::new(self.inner.call(req))

    }
}
