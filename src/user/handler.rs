use axum::{Extension, Json};
use serde::Deserialize;

use crate::config::{error::AppError, jwt::JwtManager};

use super::service::{UserLogin, UserRegister, UserService};

#[tracing::instrument]
pub async fn register_user(
    Extension(user_service): Extension<UserService>,
    Extension(jwt_manager): Extension<JwtManager>,
    Json(request): Json<RegisterUserRequest>,
) -> Result<Json<String>, AppError> {
    let user_register = UserRegister::new(request.email, request.password, request.name);
    let token = user_service.register(user_register, &jwt_manager).await?;

    Ok(Json(String::from(token)))
}

pub async fn login_hander(
    Extension(user_service): Extension<UserService>,
    Extension(jwt_manager): Extension<JwtManager>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<String>, AppError> {
    let user_login = UserLogin::new(request.email, request.password);
    let token = user_service.login(user_login, &jwt_manager).await?;

    Ok(Json(String::from(token)))
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
