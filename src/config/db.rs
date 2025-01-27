use std::future::Future;

use async_redis_session::RedisSessionStore;
use futures::future::BoxFuture;
use sqlx::{postgres::PgPoolOptions, PgPool, PgTransaction, Pool, Postgres, Transaction};

use super::{error::AppError, MangJooResult};

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

pub async fn transaction<'a, F, R>(db_pool: &PgPool, operation: F) -> MangJooResult<R>
where
    F: for<'b> FnOnce(&'b mut PgTransaction<'_>) -> BoxFuture<'b, MangJooResult<R>>,
{
    let mut tx = db_pool
        .begin()
        .await
        .map_err(|err| AppError::DatabaseError(format!("Failed start transaction. {}", err)))?;

    match operation(&mut tx).await {
        Ok(result) => {
            let _ = tx.commit().await;
            Ok(result)
        }
        Err(err) => {
            let _ = tx.rollback().await;
            Err(err)
        }
    }
}
