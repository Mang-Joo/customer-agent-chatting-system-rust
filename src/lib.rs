use std::{env, sync::Arc};

use axum::{Extension, Router};
use config::{app_state::AppState, db::init_db, jwt::JwtManager};
use tokio::net::TcpListener;
use user::create_user_router;

pub mod constants {
    use once_cell::sync::Lazy;

    pub static APP_NAME: &str = "MangJoo-axum";
    pub static ENVIRONMENT: Lazy<&'static str> = Lazy::new(|| {
        if cfg!(feature = "dev") {
            "dev"
        } else if cfg!(feature = "live") {
            "live"
        } else {
            "unknown"
        }
    });
}

pub mod chat;
pub mod config;
pub mod user;

pub async fn start_server() {
    dotenv::dotenv().ok();
    let secure = env::var("JWT_SECURE_VALUE").expect("JWT_TOKEN_VALUE must be set");
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_manager = JwtManager::new(secure.as_bytes());
    let db_pool = init_db(db_url).await;

    let app_state = Arc::new(AppState::new(db_pool));

    let chat_router = chat::create_chat_router().await;
    let user_router = create_user_router(Arc::clone(&app_state)).await;

    let router = Router::new()
        .nest("/api", chat_router)
        .with_state(app_state)
        .nest("/api", user_router)
        .layer(Extension(jwt_manager));

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, router).await.unwrap();
}
