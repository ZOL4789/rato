use axum::response::{IntoResponse, Response};
use pin_project_lite::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::task::{ready, Context, Poll};

pin_project! {
    #[project = ResFutureProj]
    pub struct ResFuture<E, F> {
        #[pin]
        inner: FutureState<E, F>,
    }
}

impl<Err1, Fut, Err2> Future for ResFuture<Err1, Fut>
where
    Err1: IntoResponse + Clone,
    Fut: Future<Output = Result<Response, Err2>>,
{
    type Output = Result<Response, Err2>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let pin = self.project().inner.project();
        let poll = match pin {
            FutureStatProj::Error { e } =>  {
                e.clone().into_response()
            },
            FutureStatProj::Success { future } => ready!(future.poll(cx))?,
        };
        Poll::Ready(Ok(poll))
    }
}

pin_project! {
    #[project = FutureStatProj]
    enum FutureState<Err, Fut> {
      Error{
            e: Err
        },
      Success {
            #[pin]
            future: Fut
        }
    }
}

impl<Err, Fut> ResFuture<Err, Fut> {
    pub(crate) fn err(e: Err) -> Self {
        Self {
            inner: FutureState::Error { e },
        }
    }

    pub(crate) fn new(future: Fut) -> Self {
        Self {
            inner: FutureState::Success { future },
        }
    }
}
