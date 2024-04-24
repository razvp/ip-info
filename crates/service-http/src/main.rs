use std::net::SocketAddr;

use axum::http::header::ACCEPT;
use axum::{extract::ConnectInfo, http::HeaderMap, response::IntoResponse, routing::get, Router};

async fn myip_get(
    ConnectInfo(ip): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let accept_value = headers.get(ACCEPT);

    match accept_value {
        Some(value) => match value.as_bytes() {
            b"*/*" => {
                println!("received ALL");
            }
            _ => {
                println!("no value");
            }
        },
        None => {
            // send text
        }
    }
    dbg!(&accept_value, &ip);
    "Hello from myip"
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "Hello, world!" }))
        .route("/myip", get(myip_get));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
