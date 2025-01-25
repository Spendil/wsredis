use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, mpsc::UnboundedSender};
use redis::aio::MultiplexedConnection;
use warp::ws::Message;
use std::sync::Arc;

pub type RedisClient = Arc<Mutex<MultiplexedConnection>>;
pub type Connections = Arc<Mutex<Vec<UnboundedSender<Message>>>>;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub tablename: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct AuthRequest {
    pub tablename: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct ApiResponse {
    pub status: String,
    pub message: String,
}
