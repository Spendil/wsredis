use warp::Filter;

use crate::types::{Connections, RedisClient};

pub fn with_redis(
    redis_client: RedisClient,
) -> impl Filter<Extract = (RedisClient,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || redis_client.clone())
}

pub fn with_connections(
    connections: Connections,
) -> impl Filter<Extract = (Connections,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || connections.clone())
}
