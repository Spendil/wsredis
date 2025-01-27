use std::collections::HashMap;

use warp::Filter;

use crate::types::{Connections, RedisClient, RedisConfig};
use super::filter::{with_redis, with_connections};
use crate::handler::ws_connection;

pub fn create(
	redis_client: RedisClient, connections: Connections, redis_config: RedisConfig
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
	warp::path("ws")
        .and(warp::ws())
        .and(warp::query::<HashMap<String, String>>())
        .and(with_redis(redis_client.clone()))
        .and(with_connections(connections.clone()))
        .and(warp::any().map(move || redis_config.clone()))
        .map(|ws: warp::ws::Ws, query, redis_client, connections, redis_config| {
            ws.on_upgrade(move |socket| ws_connection::handle(socket, query, redis_client, connections, redis_config))
        })
} 
