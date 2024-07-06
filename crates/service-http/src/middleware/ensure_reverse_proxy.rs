use std::{future::Future, task::Poll};

use axum::{
    extract::Request,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tower::{Layer, Service};

use crate::HeaderIp;

#[derive(Clone)]
pub struct EnsureReverseProxyLayer;

impl<S> Layer<S> for EnsureReverseProxyLayer {
    type Service = EnsureReverseProxy<S>;

    fn layer(&self, inner: S) -> Self::Service {
        EnsureReverseProxy { inner }
    }
}

#[derive(Clone)]
pub struct EnsureReverseProxy<S> {
    inner: S,
}

impl<S> Service<Request> for EnsureReverseProxy<S>
where
    S: Service<Request, Response = Response>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = EnsureReverseProxyFuture<S::Future>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request) -> Self::Future {
        let ext = req.extensions_mut();
        ext.insert(HeaderIp("teeeest_ip".to_string()));

        EnsureReverseProxyFuture {
            future: self.inner.call(req),
        }
    }
}

pin_project_lite::pin_project! {
    pub struct EnsureReverseProxyFuture<F> {
        #[pin]
        future: F,
    }

}

impl<F, E> Future for EnsureReverseProxyFuture<F>
where
    F: Future<Output = Result<Response, E>>,
{
    type Output = F::Output;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();

        Poll::Ready(Ok(StatusCode::IM_A_TEAPOT.into_response()))
    }
}
