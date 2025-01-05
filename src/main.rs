use futures::{SinkExt, StreamExt};
use redis::{aio::MultiplexedConnection, AsyncCommands};
use std::{net::IpAddr, sync::Arc};
use tokio::sync::Mutex;
use warp::Filter;
use std::env;

type RedisClient = Arc<Mutex<MultiplexedConnection>>;
type Connections = Arc<Mutex<Vec<tokio::sync::mpsc::UnboundedSender<warp::ws::Message>>>>;

const MAIN_CHANNEL: &str = "knopki_updates";
const TIME_KEY: &str = "knopki_time";

#[tokio::main]
async fn main() {
    let redis_addr = env::var("REDIS_ADDR").unwrap_or(String::from("127.0.0.1"));
    let redis_port = env::var("REDIS_PORT").unwrap_or(String::from("6379"));

    let redis_config = format!("redis://{}:{}", &redis_addr, &redis_port);

    // Подключение к Redis
    let client = match redis::Client::open(redis_config.clone()) {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Failed to connect to Redis: {}", e);
            return;
        }
    };
    let connection = client.get_multiplexed_async_connection().await.expect("Failed to connect to Redis");
    let redis_client = Arc::new(Mutex::new(connection));

    // Хранилище всех активных соединений WebSocket
    let connections: Connections = Arc::new(Mutex::new(Vec::new()));

    // Создаем маршрут для веб-сокетов
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(with_redis(redis_client.clone()))
        .and(with_connections(connections.clone()))
        .map(|ws: warp::ws::Ws, redis_client, connections| {
            ws.on_upgrade(move |socket| handle_connection(socket, redis_client, connections))
        });
    let ws_addr: IpAddr = env::var("WS_ADDR").unwrap_or(String::from("127.0.0.1")).parse().expect("Invalid host address");
    let ws_port: u16 = env::var("WS_PORT").unwrap_or(String::from("3030")).parse().expect("Invalid port address");

    // Запускаем Redis Pub/Sub listener
    tokio::spawn(redis_listener(connections.clone(), redis_config));

    let ws_config = (ws_addr, ws_port);
    warp::serve(ws_route).run(ws_config).await;
}

fn with_redis(
    redis_client: RedisClient,
) -> impl Filter<Extract = (RedisClient,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || redis_client.clone())
}

// Передача списка соединений
fn with_connections(
    connections: Connections,
) -> impl Filter<Extract = (Connections,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || connections.clone())
}

// Обработка каждого подключения
async fn handle_connection(
    ws: warp::ws::WebSocket,
    redis_client: RedisClient,
    connections: Connections,
) {
    let (mut tx, mut rx) = ws.split();
    let (tx_msg, rx_msg) = tokio::sync::mpsc::unbounded_channel();

    // Добавление соединения в список
    {
        let mut conns = connections.lock().await;
        conns.push(tx_msg.clone());
    }

    // Попытка получить последнее сообщение из Redis и отправить его через WebSocket
    {
        let mut conn = redis_client.lock().await;

        let curr = conn.get::<_, Option<String>>(MAIN_CHANNEL).await.ok().flatten();
        let time = conn.get::<_, Option<String>>(TIME_KEY).await.ok().flatten();

        if let Some(mut message) = curr {
            if message == "time" {
                if let Some(t) = time {
                    if t != "0" {
                        message.push(' ');
                        message.push_str(&t);
                    }
                }
            }
            if let Err(_) = tx.send(warp::ws::Message::text(message)).await {
                return; // Если не удалось отправить, прекращаем обработку
            }
        }
    }

    // Запуск задачи для отправки сообщений
    tokio::spawn(async move {
        let mut rx_msg = tokio_stream::wrappers::UnboundedReceiverStream::new(rx_msg);
        while let Some(message) = rx_msg.next().await {
            if tx.send(message).await.is_err() {
                break;
            }
        }
    });

    // Обработка входящих сообщений
    while let Some(Ok(msg)) = rx.next().await {
        if msg.is_text() {
            let original_text = msg.to_str().unwrap_or("").to_string();
            let mut text = original_text.clone();
            let mut conn = redis_client.lock().await;

            // Публикация в Redis канал
            
            
            if original_text == "time" {
                let time_value: i64 = if conn.exists(TIME_KEY).await.unwrap_or(false) {
                    conn.incr(TIME_KEY, 1).await.unwrap_or(0)
                } else {
                    conn.set(TIME_KEY, 0).await.unwrap_or(());
                    0
                };
                if time_value > 0 {
                    text = format!("{} {}", original_text, time_value);
                }
            }
            if original_text == "no action" {
                if let Ok(Some(msg)) = conn.get::<_, Option<String>>(MAIN_CHANNEL).await {
                    if msg == "time" {
                        let _: () = conn.del(TIME_KEY).await.unwrap_or(());
                    }
                }
            }
            let _: () = conn.publish(MAIN_CHANNEL, text.as_str()).await.unwrap_or(());
            let _: () = conn.set(MAIN_CHANNEL, original_text).await.unwrap_or(());
        }
    }

    // Удаление соединения из списка
    {
        let mut conns = connections.lock().await;
        conns.retain(|conn| !conn.same_channel(&tx_msg));
    }
}

// Listener для Redis Pub/Sub
async fn redis_listener(connections: Connections, redis_config: String) {
    // Создаем отдельное PubSub соединение
    let mut pubsub_conn = {
        let client = redis::Client::open(redis_config).expect("Failed while connecting to redis");
        client.get_async_connection().await.expect("Failed while creating PubSub connection")
    }.into_pubsub();

    // Подписка на канал
    pubsub_conn.subscribe(MAIN_CHANNEL).await.expect("Failed while subscribing to the channel");

    let mut pubsub_stream = pubsub_conn.on_message();

    while let Some(msg) = pubsub_stream.next().await {
        if let Ok(payload) = msg.get_payload::<String>() {
            let conns = connections.lock().await;
            for conn in conns.iter() {
                let _ = conn.send(warp::ws::Message::text(payload.clone()));
            }
        }
    }
}
