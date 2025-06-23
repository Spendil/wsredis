use warp::{reject::Rejection, reply::Reply, Filter};

use crate::types::RedisClient;
use crate::handler::auth;

use super::filter::with_redis;

pub fn create(
	redis_client: RedisClient
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
	warp::path("auth")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_redis(redis_client.clone()))
        .and_then(auth::handle)
}
