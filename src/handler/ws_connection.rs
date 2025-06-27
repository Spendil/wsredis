use std::{collections::HashMap, sync::Arc};

use crate::{pubsub, types::{Connections, RedisClient, RedisConfig}};
use futures::{SinkExt, StreamExt};
use redis::AsyncCommands;
use tokio::sync::Mutex;
use warp;

type Listeners = Arc<Mutex<HashMap<String, tokio::task::JoinHandle<()>>>>;

pub async fn handle(
	ws: warp::ws::WebSocket,
    query: HashMap<String, String>,
    redis_client: RedisClient,
    connections: Connections,
    redis_config: RedisConfig
) {	
    static LISTENERS: once_cell::sync::Lazy<Listeners> = once_cell::sync::Lazy::new(|| {
        Arc::new(Mutex::new(HashMap::new()))
    });

    let session_key = query.get("session").map(String::as_str).unwrap_or("");
    let table_key = query.get("tablename").map(String::as_str).unwrap_or("");
    let channel = table_key.to_string();

    let conn_id = format!("conn_{}_{}", channel, chrono::Utc::now().timestamp_nanos());

    let (mut tx, mut rx) = ws.split();
    let (tx_msg, rx_msg) = tokio::sync::mpsc::unbounded_channel();

    {
        let mut conns = connections.lock().await;
        conns.entry(channel.clone())
            .or_insert_with(Vec::new)
            .push(tx_msg.clone());
    }

    {
        let mut listeners = LISTENERS.lock().await;
        if !listeners.contains_key(&channel) {
            println!("Spawning listener for channel: {}", channel);
            let handle = tokio::spawn(pubsub::listener(
                connections.clone(),
                redis_config.clone(),
                channel.clone()
            ));
            listeners.insert(conn_id.clone(), handle);
        }
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
            }
        }
    }

   {
        let mut conns = connections.lock().await;
        if let Some(channel_conns) = conns.get_mut(&channel) {
            channel_conns.retain(|conn| !std::ptr::eq(conn, &tx_msg));
            println!("Removed connection {} for channel {}, remaining: {}", conn_id, channel, channel_conns.len());
            let mut listeners = LISTENERS.lock().await;
            if let Some(handle) = listeners.remove(&conn_id) {
                handle.abort();
                println!("Listener for connection {} on channel {} aborted", conn_id, channel);
            }
            if channel_conns.is_empty() {
                conns.remove(&channel);
            }
        }
    }
}
