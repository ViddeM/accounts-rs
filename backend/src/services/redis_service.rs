use mobc::Pool;
use mobc_redis::{redis::AsyncCommands, RedisConnectionManager};
use rocket::{serde::DeserializeOwned, State};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum RedisError {
    #[error("An internal error has occured")]
    Internal,
    #[error("A redis error occured")]
    Redis,
    #[error("Failed to serialize/deserialize the value to/from the correct json type")]
    Serde,
}

pub async fn redis_get<T>(
    redis_pool: &State<Pool<RedisConnectionManager>>,
    key: String,
) -> Result<T, RedisError>
where
    T: DeserializeOwned,
{
    let mut redis_conn = redis_pool.get().await.map_err(|err| {
        error!("Failed to get redis connection from pool, err {}", err);
        RedisError::Internal
    })?;

    let raw_result = redis_conn.get::<String, String>(key).await.map_err(|err| {
        error!("Failed to get from redis, err {}", err);
        RedisError::Redis
    })?;

    serde_json::from_str::<T>(&raw_result).map_err(|err| {
        error!("Failed to parse json value from redis, err {}", err);
        RedisError::Serde
    })
}

pub async fn redis_get_option<T>(
    redis_pool: &State<Pool<RedisConnectionManager>>,
    key: String,
) -> Result<Option<T>, RedisError>
where
    T: DeserializeOwned,
{
    let mut redis_conn = redis_pool.get().await.map_err(|err| {
        error!("Failed to get redis connection from pool, err {}", err);
        RedisError::Internal
    })?;

    match redis_conn
        .get::<String, Option<String>>(key)
        .await
        .map_err(|err| {
            error!("Failed to get from redis, err {}", err);
            RedisError::Redis
        })? {
        Some(raw_result) => Ok(Some(serde_json::from_str::<T>(&raw_result).map_err(
            |err| {
                error!("Failed to parse json value from redis, err {}", err);
                RedisError::Serde
            },
        )?)),
        None => Ok(None),
    }
}

pub async fn redis_set<T>(
    redis_pool: &State<Pool<RedisConnectionManager>>,
    key: String,
    val: T,
    expiration_seconds: usize,
) -> Result<(), RedisError>
where
    T: Serialize,
{
    let mut redis_conn = redis_pool.get().await.map_err(|err| {
        error!("Failed to get redis connection from pool, err {}", err);
        RedisError::Internal
    })?;

    redis_conn
        .set_ex::<String, String, String>(
            key,
            serde_json::to_string(&val).map_err(|err| {
                error!("Failed to serialize value, err {}", err);
                RedisError::Serde
            })?,
            expiration_seconds,
        )
        .await
        .map_err(|err| {
            error!("Failed to insert value to redis, err {}", err);
            RedisError::Redis
        })?;

    Ok(())
}

pub async fn redis_push(
    redis_pool: &State<Pool<RedisConnectionManager>>,
    key: String,
    value: String,
) -> Result<(), RedisError> {
    let mut redis_conn = redis_pool.get().await.map_err(|err| {
        error!("Failed to get redis connection from pool, err {}", err);
        RedisError::Internal
    })?;

    redis_conn
        .lpush::<String, String, usize>(key, value)
        .await
        .map_err(|err| {
            error!("Failed to push value to redis, err {}", err);
            RedisError::Redis
        })?;

    Ok(())
}

pub async fn redis_del(
    redis_pool: &State<Pool<RedisConnectionManager>>,
    key: String,
) -> Result<(), RedisError> {
    let mut redis_conn = redis_pool.get().await.map_err(|err| {
        error!("Failed to get redis connection from pool, err {}", err);
        RedisError::Internal
    })?;

    redis_conn.del::<String, ()>(key).await.map_err(|err| {
        error!("Failed to delete value from redis, err {}", err);
        RedisError::Redis
    })?;

    Ok(())
}
