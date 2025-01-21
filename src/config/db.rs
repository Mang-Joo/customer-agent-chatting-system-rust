use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub async fn init_db(db_url: String) -> Pool<Postgres> {
    PgPoolOptions::new()
        .connect(db_url.as_ref())
        .await
        .expect("Db Connection Error")
}
