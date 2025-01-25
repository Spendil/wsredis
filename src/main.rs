mod types;
mod config;
mod router;
mod handler;

use types::Connections;
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex;

const MAIN_CHANNEL: &str = "knopki_updates";

#[tokio::main]
async fn main() {

    let (redis_config, server_config) = config::create();

    let client = match redis::Client::open(redis_config.clone()) {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to Redis: {}", e);
            return;
        }
    };

    let connection = client.get_multiplexed_async_connection().await.expect("Failed to connect to Redis");
    let redis_client = Arc::new(Mutex::new(connection));
    let connections: Connections = Arc::new(Mutex::new(Vec::new()));
    let routes = router::create(redis_client, connections.clone());

    tokio::spawn(redis_listener(connections.clone(), redis_config));
    
    warp::serve(routes).run(server_config).await;
}

async fn redis_listener(connections: Connections, redis_config: String) {
    let mut pubsub_conn = {
        let client = redis::Client::open(redis_config).expect("Failed while connecting to redis");
        client.get_async_connection().await.expect("Failed while creating PubSub connection")
    }.into_pubsub();

    pubsub_conn.subscribe(MAIN_CHANNEL).await.expect("Failed while subscribing to the channel");

    let mut pubsub_stream = pubsub_conn.on_message();

    while let Some(msg) = pubsub_stream.next().await {
        if let Ok(payload) = msg.get_payload::<String>() {
            let conns = connections.lock().await;
            for conn in conns.iter() {
                let _ = conn.send(warp::ws::Message::text(payload.clone()));
            }
        }
    }
}
