use std::{env, net::IpAddr};

use crate::types::ServerConfig;

pub fn create() -> ServerConfig {
	let ws_addr: IpAddr = env::var("WS_ADDR").unwrap_or(String::from("127.0.0.1")).parse().expect("Invalid host address");
	let ws_port: u16 = env::var("WS_PORT").unwrap_or(String::from("3030")).parse().expect("Invalid port address");
	(ws_addr, ws_port)
}
