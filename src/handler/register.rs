use argon2::Config;
use redis::AsyncCommands;
use uuid::Uuid;

use crate::{constants::TTL_SECONDS, types::{ApiResponse, RedisClient, RegisterRequest}};

pub async fn handle(req: RegisterRequest, redis_client: RedisClient) -> Result<impl warp::Reply, warp::Rejection>{
    let mut conn = redis_client.lock().await;

    let hashed_password = argon2::hash_encoded(req.password.as_bytes(), b"randomsalt", &Config::default()).unwrap();
    let table_key = format!("table:{}", req.tablename);

    if conn.exists(&table_key).await.unwrap_or(false) {
        return Ok(warp::reply::json(&ApiResponse {
            status: "error".to_string(),
            message: "Table with such name already exists".to_string(),
            token: None
        }));
    }

    let _: () = conn.hset(&table_key, "hashed_password", hashed_password).await.unwrap_or(());
    let _: () = conn.expire(&table_key, TTL_SECONDS).await.unwrap_or(());

    let session_key = format!("session:{}", Uuid::new_v4().to_string());
    let _: () = conn.set(&session_key, table_key).await.unwrap_or(());

    Ok(warp::reply::json(&ApiResponse {
        status: "success".to_string(),
        message: "Table registered".to_string(),
        token: Some(session_key)
    }))
}
