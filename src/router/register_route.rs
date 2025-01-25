use warp::Filter;

use crate::handler::register;
use crate::types::RedisClient;

use super::filter::with_redis;

pub fn create(
    redis_client: RedisClient,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("register")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_redis(redis_client.clone()))
        .and_then(register::handle)
}
