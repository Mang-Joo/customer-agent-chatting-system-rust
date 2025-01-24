use std::sync::Arc;

use axum::{routing::post, Extension, Router};
use handler::{login_hander, register_user};
use repository::UserRepository;
use service::UserService;
use tower_cookies::CookieManagerLayer;

use crate::config::app_state::ArcAppState;

pub mod handler;
pub mod repository;
pub mod service;
pub mod user;

pub async fn create_user_router(app_state: ArcAppState) -> Router {
    Router::new()
        .route("/register-user", post(register_user))
        .route("/login", post(login_hander))
        .layer(Extension(UserService::new(UserRepository::new(
            app_state.db_pool.clone(),
        ))))
        .layer(CookieManagerLayer::new())
        .with_state(Arc::clone(&app_state))
}
