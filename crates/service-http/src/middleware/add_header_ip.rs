use std::future::Ready;

use crate::HeaderIp;
use axum::extract::Request;
use reqwest::StatusCode;
use tower::{Layer, Service};

#[derive(Clone)]
pub struct GetIPMiddleware<S> {
    inner: S,
}

#[derive(Clone)]
pub struct GetIPLayer;

impl<S> Layer<S> for GetIPLayer {
    type Service = GetIPMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        GetIPMiddleware { inner }
    }
}

impl<S> Service<Request> for GetIPMiddleware<S>
where
    S: Service<Request>,
{
    type Response = S::Response;
    type Error = S::Error;
    // type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;
    type Future = S::Future;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request) -> Self::Future {
        let ext = req.extensions_mut();
        ext.insert(HeaderIp("teeeest_ip".to_string()));

        self.inner.call(req)
    }
}
