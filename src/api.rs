use std::time::SystemTime;

use axum::{debug_handler, http::StatusCode, Extension, Json};
use axum_client_ip::InsecureClientIp;
use serde::{Deserialize, Serialize};

use crate::servers::{Server, SharedServerList};

#[debug_handler]
pub async fn add_server(
    ip: InsecureClientIp,
    Extension(state): Extension<SharedServerList>,
    Json(payload): Json<ServerPing>,
) -> (StatusCode, Json<PingReply>) {
    if payload.port.is_none() && payload.url.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(PingReply {
                error: Some("Port or URL must be provided".to_string()),
                success: None,
                server: None,
            }),
        );
    }
    let url = payload.url.unwrap_or_else(|| {
        let port = payload.port.unwrap();
        match ip.0 {
            std::net::IpAddr::V4(ip) => format!("http://{}:{}", ip.to_string(), port),
            std::net::IpAddr::V6(ip) => format!("http://[{}]:{}", ip.to_string(), port),
        }
    });

    let user = Server {
        name: payload.name,
        hardware: payload.hardware.unwrap_or("".to_string()),
        antenna: payload.antenna.unwrap_or("".to_string()),
        bandwidth: payload.bandwidth,
        users: payload.users.unwrap_or(0),
        remarks: payload.remarks.unwrap_or("".to_string()),
        description: payload.description.unwrap_or("".to_string()),
        base_frequency: payload.base_frequency,
        url: url,
        last_update: SystemTime::now(),
    };

    state.write().await.add_server(payload.id, user.clone());

    (
        StatusCode::CREATED,
        Json(PingReply {
            error: None,
            success: Some(true),
            server: Some(user),
        }),
    )
}

pub async fn get_all_servers(
    Extension(state): Extension<SharedServerList>,
) -> (StatusCode, Json<Vec<Server>>) {
    state.write().await.remove_old_servers();
    (StatusCode::OK, Json(state.read().await.get_all_servers()))
}

#[derive(Serialize)]
pub struct PingReply {
    error: Option<String>,
    success: Option<bool>,
    server: Option<Server>,
}

#[derive(Deserialize)]
pub struct ServerPing {
    id: String,
    name: String,
    hardware: Option<String>,
    antenna: Option<String>,
    bandwidth: f64,
    users: Option<i32>,
    remarks: Option<String>,
    description: Option<String>,
    base_frequency: f64,
    port: Option<i32>,
    url: Option<String>,
}
