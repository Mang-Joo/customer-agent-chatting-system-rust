use axum::{extract::State, Extension, Json};
use serde::Deserialize;
use tower_cookies::{
    cookie::{time::Duration, SameSite},
    Cookie, Cookies,
};

use crate::config::{app_state::ArcAppState, MangJooResult};

use super::{
    service::{UserLogin, UserRegister, UserService},
    user::UserRole,
};

#[tracing::instrument]
pub async fn register_user(
    Extension(user_service): Extension<UserService>,
    Json(request): Json<RegisterUserRequest>,
) -> MangJooResult<()> {
    let user_register = UserRegister::new(
        request.email,
        request.password,
        request.name,
        UserRole::User,
    );
    let _ = user_service.register(user_register).await?;

    Ok(())
}

#[tracing::instrument]
pub async fn register_agent(
    Extension(user_service): Extension<UserService>,
    Json(request): Json<RegisterUserRequest>,
) -> MangJooResult<()> {
    let user_register = UserRegister::new(
        request.email,
        request.password,
        request.name,
        UserRole::Agent,
    );
    let _ = user_service.register(user_register).await?;
    Ok(())
}

pub async fn login_hander(
    State(app_state): State<ArcAppState>,
    Extension(user_service): Extension<UserService>,
    cookies: Cookies,
    Json(request): Json<LoginRequest>,
) -> MangJooResult<()> {
    let user_login = UserLogin::new(request.email, request.password);
    let session = user_service
        .login(user_login, &app_state.session_store)
        .await?;

    let cookie = Cookie::build(("session_id", session.clone()))
        .path("/")
        .secure(false)
        .http_only(true)
        .same_site(SameSite::Strict)
        .max_age(Duration::hours(24));

    cookies.add(cookie.build());

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct RegisterUserRequest {
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}
