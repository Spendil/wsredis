use warp::Filter;

use crate::handler::table_list;
use crate::types::RedisClient;

use super::filter::with_redis;

pub fn create(
    redis_client: RedisClient,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("tables")
        .and(warp::get())
        .and(with_redis(redis_client.clone()))
        .and_then(table_list::handle)
}
