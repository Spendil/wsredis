use tokio_tungstenite::tungstenite::http::Method;
use warp::{reject::Rejection, Filter};

use crate::types::{Connections, RedisClient};

mod filter;
mod ws_route;
mod register_route;

pub fn create(redis_client: RedisClient, connections: Connections) -> impl Filter<Extract = impl warp::Reply, Error = Rejection> + Clone {
	let cors = warp::cors().allow_any_origin() // Разрешаем все источники (для тестирования)
		.allow_methods(&[Method::GET, Method::POST, Method::OPTIONS]) // Разрешаем методы
		.allow_headers(vec!["Content-Type", "Authorization"])
		.allow_credentials(true);

	let register = register_route::create(redis_client.clone()).with(cors.clone());
	let ws = ws_route::create(redis_client, connections);
	
	register.or(ws)
}
