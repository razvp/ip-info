#![allow(unused)] // For example code.

use reqwest::header::ACCEPT;

#[tokio::main]
async fn main() {
    println!("Hello world x");
    let resp = reqwest::get("http://localhost:3000/myip").await.unwrap();
    dbg!(&resp.text().await);

    let client = reqwest::Client::new();
    let resp = client
        .get("http://localhost:3000/myip")
        .header(ACCEPT, "application/json")
        .send()
        .await;

    dbg!(&resp);
}
