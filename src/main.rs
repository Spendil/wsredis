mod types;
mod config;
mod router;
mod handler;
mod pubsub;
mod constants;

use types::Connections;
use std::sync::Arc;
use tokio::sync::Mutex;

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

    tokio::spawn(pubsub::listener(connections.clone(), redis_config));

    warp::serve(routes).run(server_config).await;
}
