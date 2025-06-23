use redis::AsyncCommands;
use uuid::Uuid;
use warp::{reject::Rejection, reply::Reply};

use crate::types::{ApiResponse, RedisClient, RegisterRequest};

pub async fn handle(req: RegisterRequest, redis_client: RedisClient) -> Result<impl Reply, Rejection> {
	let mut conn = redis_client.lock().await;

	let table_key = format!("table:{}", req.tablename);

	let valid_password = if conn.exists(&table_key).await.unwrap_or(false) {
		let hashed_password = conn.hget(&table_key, "hashed_password").await.unwrap_or(String::from(""));
		argon2::verify_encoded(&hashed_password, req.password.as_bytes()).unwrap_or(false)
	} else {
		false
	};

	let token = if valid_password {
		let session_key = format!("session:{}", Uuid::new_v4().to_string());
    	let _: () = conn.set(&session_key, table_key).await.unwrap_or(());
		session_key
	} else {
		String::from("")
	};

	if !token.is_empty() {
		Ok(warp::reply::json(&ApiResponse {
			status: "success".to_string(),
			message: "Authorized".to_string(),
			token: Some(token)
		}))
	} else {
		Ok(warp::reply::json(&ApiResponse {
			status: "error".to_string(),
			message: "Password is not valid".to_string(),
			token: None
		}))
	}
}
