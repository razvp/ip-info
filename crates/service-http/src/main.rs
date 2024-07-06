#![allow(unused)]
use axum::error_handling::HandleErrorLayer;
use axum::extract::Extension;
use service_http::HeaderIp;

use std::net::Ipv4Addr;

use axum::async_trait;
use axum::body::Body;
use axum::extract::FromRequestParts;
use axum::extract::Path;
use axum::http::request::Parts;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::response::Response;
use axum::{response::IntoResponse, routing::get, Router};

use service_http::middleware::EnsureReverseProxyLayer;

#[derive(Debug)]
struct ExtractForwardedIP(Ipv4Addr);

#[async_trait]
impl<S> FromRequestParts<S> for ExtractForwardedIP
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let ips: Result<Vec<_>, Self::Rejection> = parts
            .headers
            .get_all("X-Forwarded-For")
            .iter()
            .map(|hv| {
                let ip = hv
                    .to_str()
                    .map_err(|_| (StatusCode::BAD_REQUEST, "test"))?
                    .parse::<Ipv4Addr>()
                    .map_err(|_| (StatusCode::BAD_REQUEST, "test"))?;
                Ok(ip)
            })
            .collect();

        let ips = ips?;
        match ips.len() {
            0 => Err((StatusCode::BAD_REQUEST, "no ip in x-forwarded-ip")),
            1 => Ok(ExtractForwardedIP(ips[0])),
            _ => Err((StatusCode::BAD_REQUEST, "too many forwarded ips")),
        }
    }
}

enum Permit {
    // API_KEY header is not set, but there is free quota for the IP
    Once,
    ApiKey(String),
}

struct PermitMiddleware<S> {
    inner: S,
}

async fn ip4(
    path_ip: Option<Path<String>>,
    headers: HeaderMap,
    ExtractForwardedIP(ip): ExtractForwardedIP,
) -> Response {
    dbg!(&ip);

    (StatusCode::OK, Body::from(ip.to_string())).into_response()
}

async fn test(Extension(ip): Extension<HeaderIp>) -> &'static str {
    dbg!(&ip);

    "test"
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/ip4", get(ip4))
        .route("/ip4/:ip", get(ip4))
        .route("/test", get(test))
        .layer(EnsureReverseProxyLayer);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
