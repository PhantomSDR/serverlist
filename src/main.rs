mod api;
mod servers;

use std::net::SocketAddr;

use axum::{
    http::{HeaderValue, Method}, routing::{get, post}, Router
};
use axum_client_ip::SecureClientIpSource;
use tower_http::{add_extension::AddExtensionLayer, cors::CorsLayer};

use api::{add_server, get_all_servers};
use servers::SharedServerList;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Only allow phantomsdr.github.io to access the API
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin("https://phantomsdr.github.io".parse::<HeaderValue>().unwrap());
    
    let app = Router::new()
        .route("/api/v1/ping", post(add_server))
        .route("/api/v1/all", get(get_all_servers))
        .layer(SecureClientIpSource::ConnectInfo.into_extension())
        .layer(cors)
        .layer(AddExtensionLayer::new(SharedServerList::default()));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
