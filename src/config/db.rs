use async_redis_session::RedisSessionStore;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub async fn init_db(db_url: String) -> Pool<Postgres> {
    PgPoolOptions::new()
        .connect(db_url.as_ref())
        .await
        .expect("Db Connection Error")
}

pub fn init_redis_session_store(redis_url: String) -> RedisSessionStore {
    let redis_session_store =
        RedisSessionStore::new(redis_url).expect("Redis Session Store Connection Failed.");

    redis_session_store
}
