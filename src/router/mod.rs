use tokio_tungstenite::tungstenite::http::Method;
use warp::{reject::Rejection, Filter};

use crate::types::{Connections, RedisClient, RedisConfig};

mod filter;
mod ws_route;
mod register_route;
mod auth_route;
mod table_list_route;

pub fn create(redis_client: RedisClient, connections: Connections, redis_config: RedisConfig) -> impl Filter<Extract = impl warp::Reply, Error = Rejection> + Clone {
	let cors = warp::cors().allow_any_origin()
		.allow_methods(&[Method::GET, Method::POST, Method::OPTIONS])
		.allow_headers(vec!["Content-Type", "Authorization"])
		.allow_credentials(true);

	let auth = auth_route::create(redis_client.clone()).with(cors.clone());
	let register = register_route::create(redis_client.clone()).with(cors.clone());
	let table_list = table_list_route::create(redis_client.clone());
	let ws = ws_route::create(redis_client, connections, redis_config);
	
	register.or(auth).or(table_list).or(ws)
}
