use futures::StreamExt;

use crate::{constants::MAIN_CHANNEL, types::Connections};

pub async fn listener(connections: Connections, redis_config: String) {
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
