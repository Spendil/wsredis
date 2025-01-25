use std::env;

use crate::types::RedisConfig;

pub fn create() -> RedisConfig {
	let redis_addr = env::var("REDIS_ADDR").unwrap_or(String::from("127.0.0.1"));
    let redis_port = env::var("REDIS_PORT").unwrap_or(String::from("6379"));

	format!("redis://{}:{}", &redis_addr, &redis_port)
}
