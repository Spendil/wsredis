use redis::RedisResult;
use warp::{reject::Rejection, reply::Reply};

use crate::types::RedisClient;

#[derive(Debug)]
struct RedisRejection;

impl warp::reject::Reject for RedisRejection {}

pub async fn handle(redis_client: RedisClient) -> Result<impl Reply, Rejection> {
    match get_all_hash_keys(redis_client).await {
        Ok(hash_keys) => {
            println!("Hash maps: {:?}", hash_keys);
            Ok(warp::reply::json(&hash_keys)) // Возвращаем JSON-ответ с ключами
        }
        Err(err) => {
            eprintln!("Error fetching hash keys: {:?}", err);
            Err(warp::reject::custom(RedisRejection)) // Генерируем ошибку, если Redis-запрос не удался
        }
    }
}

async fn get_all_hash_keys(client: RedisClient) -> RedisResult<Vec<String>> {
    let mut cursor: u64 = 0; // Инициализируем курсор
    let mut hash_keys = Vec::new();

    loop {
        // Блокируем Redis-клиент для работы с соединением
        let mut con = client.lock().await;

        // Выполняем SCAN команду для поиска ключей
        let (next_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
            .cursor_arg(cursor) // Передаём текущий курсор
            .arg("MATCH")
            .arg("table:*") // Шаблон для поиска
            .query_async(&mut *con)
            .await?;

        // Проверяем тип каждого ключа
        for key in keys {
            let key_type: String = redis::cmd("TYPE").arg(&key).query_async(&mut *con).await?;
            if key_type == "hash" {
                hash_keys.push(key);
            }
        }

        cursor = next_cursor;

        // Если курсор вернулся в 0, завершить цикл
        if cursor == 0 {
            break;
        }
    }

    Ok(hash_keys)
}
