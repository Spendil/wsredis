use crate::types::{RedisConfig, ServerConfig};

mod redis_config;
mod server_config;

pub fn create() -> (RedisConfig, ServerConfig) {
	let redis_config = redis_config::create();
	let server_config = server_config::create();
	(redis_config, server_config)
}
