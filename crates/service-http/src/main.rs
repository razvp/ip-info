use axum::extract::Extension;
use service_http::{ApiKey, ClientIp};

use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::Response;
use axum::{response::IntoResponse, routing::get, Router};

use service_http::middleware::EnsureReverseProxyLayer;
use service_http::middleware::ExtractApiKeyLayer;

async fn ip4(Extension(client_ip): Extension<ClientIp>, path_ip: Option<Path<String>>) -> Response {
    (StatusCode::OK, client_ip.0.to_string()).into_response()
}

async fn test(
    Extension(client_ip): Extension<ClientIp>,
    Extension(api_key): Extension<ApiKey>,
) -> Response {
    dbg!(&api_key);
    (StatusCode::OK, client_ip.0.to_string()).into_response()
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/ip4", get(ip4))
        .route("/ip4/:ip", get(ip4))
        .route("/test", get(test))
        // We assume the service is behind a reverse proxy
        .layer(ExtractApiKeyLayer)
        .layer(EnsureReverseProxyLayer::new("X-Forwarded-For"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
