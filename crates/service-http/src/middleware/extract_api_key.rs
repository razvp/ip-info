use axum::{extract::Request, http::StatusCode, response::Response};
use tower::{Layer, Service};

use super::util::ResponseFuture;
use crate::ApiKey;

#[derive(Clone)]
pub struct ExtractApiKeyLayer;

impl<S> Layer<S> for ExtractApiKeyLayer {
    type Service = ExtractApiKey<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ExtractApiKey { inner }
    }
}

#[derive(Clone)]
pub struct ExtractApiKey<S> {
    inner: S,
}

impl<S> Service<Request> for ExtractApiKey<S>
where
    S: Service<Request, Response = Response>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request) -> Self::Future {
        match set_api_key_extension(&mut req) {
            Ok(_) => ResponseFuture::Continue {
                future: self.inner.call(req),
            },
            Err(response) => ResponseFuture::Stop { response },
        }
    }
}

fn set_api_key_extension(req: &mut Request) -> Result<(), (StatusCode, &'static str)> {
    let header_api_key = req.headers().get("API_KEY").map(|hv| {
        hv.to_str()
            .map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    "API_KEY header contains invisible ASCII chars",
                )
            })
            .map(ToString::to_string)
    });

    let api_key = if let Some(api_key) = header_api_key {
        Some(api_key?)
    } else {
        req.uri().query().and_then(|q| {
            q.split('&')
                .find(|v| v.starts_with("api_key"))
                .and_then(|v| v.split_once('='))
                .map(|spl| spl.1.to_string())
        })
    };
    req.extensions_mut().insert(ApiKey(api_key));

    Ok(())
}
