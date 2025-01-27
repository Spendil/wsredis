use redis::RedisResult;
use warp::{reject::Rejection, reply::Reply};

use crate::types::RedisClient;

pub async fn handle(redis_client: RedisClient) -> Result<impl Reply, Rejection> {

    let hash_keys = get_all_hash_keys(redis_client).await;
    println!("Hash maps: {:?}", hash_keys);

    Ok(Reply::into_response("resp"))
}

async fn get_all_hash_keys(client: RedisClient) -> RedisResult<Vec<String>> {
    let mut cursor;
    let mut hash_keys = Vec::new();

    loop {
        let mut con = client.lock().await;
        let (next_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
            .cursor_arg(0)
            .arg("MATCH")
            .arg("table:*")
            .query_async(&mut *con)
            .await?;
       
        for key in keys {
            let key_type: String = redis::cmd("TYPE").arg(&key).query_async(&mut *con).await?;

            if key_type == "hash" {
                hash_keys.push(key);
            }
        }

        cursor = next_cursor;

        if cursor == 0 {
            break;
        }
    }

    Ok(hash_keys)
}
