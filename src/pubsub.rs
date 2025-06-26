use futures::StreamExt;

use crate::types::Connections;

pub async fn listener(
    connections: Connections,
    redis_config: String,
    channel: String
) {
    let mut pubsub_conn = {
        let client = redis::Client::open(redis_config).expect("Failed while connecting to redis");
        client.get_async_connection().await.expect("Failed while creating PubSub connection")
    }.into_pubsub();

    let channel_clone = channel.clone();
    pubsub_conn.subscribe(channel_clone).await.expect("Failed while subscribing to the channel");

    {
        let mut pubsub_stream = pubsub_conn.on_message();
        while let Some(msg) = pubsub_stream.next().await {
            if let Ok(payload) = msg.get_payload::<String>() {
                let conns = connections.lock().await;
                if let Some(channel_conns) = conns.get(&channel) {
                    for conn in channel_conns.iter() {
                        let _ = conn.send(warp::ws::Message::text(payload.clone()));
                    }
                }
            }
        }
    }

    {
        if let Err(e) = pubsub_conn.unsubscribe(&channel).await {
            eprintln!("Failed to unsubscribe from channel: {}", e);
        }
    }
}
