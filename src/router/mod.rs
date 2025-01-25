use warp::{reject::Rejection, Filter};

use crate::types::{Connections, RedisClient};

mod filter;
mod ws_route;
mod register_route;

pub fn create(redis_client: RedisClient, connections: Connections) -> impl Filter<Extract = impl warp::Reply, Error = Rejection> + Clone {
	let register = register_route::create(redis_client.clone());
	let ws = ws_route::create(redis_client, connections);
	
	register.or(ws)
}
