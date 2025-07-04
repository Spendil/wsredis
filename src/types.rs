use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, mpsc::UnboundedSender};
use redis::aio::MultiplexedConnection;
use warp::ws::Message;
use std::{net::IpAddr, sync::Arc};
use std::collections::HashMap;

pub type RedisClient = Arc<Mutex<MultiplexedConnection>>;
pub type Connections = Arc<Mutex<HashMap<String, Vec<UnboundedSender<Message>>>>>;

pub type RedisConfig = String;
pub type ServerConfig = (IpAddr, u16);

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub tablename: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct ApiResponse {
    pub status: String,
    pub message: String,
    pub token: Option<String>
}
