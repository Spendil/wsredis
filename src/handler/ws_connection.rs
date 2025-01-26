use std::collections::HashMap;

use crate::{constants::{MAIN_CHANNEL, TIME_KEY}, types::{Connections, RedisClient}};
use futures::{SinkExt, StreamExt};
use redis::AsyncCommands;
use warp;

pub async fn handle(
	ws: warp::ws::WebSocket,
    query: HashMap<String, String>,
    redis_client: RedisClient,
    connections: Connections,
) {	

    let session_key = if let Some(session_key) = query.get("session") {
        session_key
    } else {
        &String::from("")
    };
    // let mut conn = redis_client.lock().await;
    // let session_is_valid = conn.exists(session_key).await.unwrap_or(false);

    let (mut tx, mut rx) = ws.split();
    let (tx_msg, rx_msg) = tokio::sync::mpsc::unbounded_channel();

    {
        let mut conns = connections.lock().await;
        conns.push(tx_msg.clone());
    }

    {
        let mut conn = redis_client.lock().await;

        let curr = conn.get::<_, Option<String>>(MAIN_CHANNEL).await.ok().flatten();
        let time = conn.get::<_, Option<String>>(TIME_KEY).await.ok().flatten();

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
            
            let session_is_valid = conn.exists(session_key).await.unwrap_or(false);
            if session_is_valid {
    
                if original_text == "time" {
                    let time_value: i64 = if conn.exists(TIME_KEY).await.unwrap_or(false) {
                        conn.incr(TIME_KEY, 1).await.unwrap_or(0)
                    } else {
                        conn.set(TIME_KEY, 0).await.unwrap_or(());
                        0
                    };
                    if time_value > 0 {
                        text = format!("{} {}", original_text, time_value);
                    }
                }
                if original_text == "no action" {
                    if let Ok(Some(msg)) = conn.get::<_, Option<String>>(MAIN_CHANNEL).await {
                        if msg == "time" {
                            let _: () = conn.del(TIME_KEY).await.unwrap_or(());
                        }
                    }
                }
                let _: () = conn.publish(MAIN_CHANNEL, text.as_str()).await.unwrap_or(());
                let _: () = conn.set(MAIN_CHANNEL, original_text).await.unwrap_or(());
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
