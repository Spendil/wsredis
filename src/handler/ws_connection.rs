use std::collections::HashMap;

use crate::types::{Connections, RedisClient};
use futures::{SinkExt, StreamExt};
use redis::AsyncCommands;
use warp;

pub async fn handle(
	ws: warp::ws::WebSocket,
    query: HashMap<String, String>,
    redis_client: RedisClient,
    connections: Connections,
) {	

    let session_key = query.get("session").unwrap();
    let table_key = query.get("tablename").unwrap();

    let (mut tx, mut rx) = ws.split();
    let (tx_msg, rx_msg) = tokio::sync::mpsc::unbounded_channel();

    {
        let mut conns = connections.lock().await;
        conns.push(tx_msg.clone());
    }

    {
        let mut conn = redis_client.lock().await;

        let curr = conn.hget::<_, _, Option<String>>(&table_key, "action").await.ok().flatten();
        let time = conn.hget::<_, _, Option<String>>(&table_key, "time").await.ok().flatten();

        if let Some(mut message) = curr {
            if message == "time" {
                if let Some(t) = time {
                    if t != "0" {
                        message.push(' ');
                        message.push_str(&t);
                    }
                }
            }
            if let Err(_) = tx.send(warp::ws::Message::text(message)).await {
                return;
            }
        }
    }

    tokio::spawn(async move {
        let mut rx_msg = tokio_stream::wrappers::UnboundedReceiverStream::new(rx_msg);
        while let Some(message) = rx_msg.next().await {
            if tx.send(message).await.is_err() {
                break;
            }
        }
    });

    while let Some(Ok(msg)) = rx.next().await {
        if msg.is_text() {
            let original_text = msg.to_str().unwrap_or("").to_string();

            let mut text = original_text.clone();
            let mut conn = redis_client.lock().await;
            
            let session_is_valid = if conn.exists(&session_key).await.unwrap_or(false) {
                let session = conn.get(session_key).await.unwrap_or(String::from(""));
                println!("session: {}", session);
                session == *table_key
            } else {
                false
            };

            if session_is_valid {

                if original_text == "time" {
                    let time_value: i64 = if conn.hexists(&table_key, "time").await.unwrap_or(false) {
                        conn.hincr(&table_key, "time", 1).await.unwrap_or(0)
                    } else {
                        conn.hset(&table_key, "time", 0).await.unwrap_or(());
                        0
                    };
                    if time_value > 0 {
                        text = format!("{} {}", original_text, time_value);
                    }
                }
                if original_text == "no action" {
                    if conn.hexists(&table_key, "time").await.unwrap_or(false) {
                        let _: () = conn.hdel(&table_key, "time").await.unwrap_or(());
                    }
                }
                let _: () = conn.publish(&table_key, text.as_str()).await.unwrap_or(());
                let _: () = conn.hset(&table_key, "action", original_text).await.unwrap_or(());
            } else {
                println!("invalid session trying to send {}", original_text);
                // let _ = tx.send(warp::ws::Message::text("Authorize to have write permisiions.")).await;
            }
        }
    }

    {
        let mut conns = connections.lock().await;
        conns.retain(|conn| !conn.same_channel(&tx_msg));
    }
}
