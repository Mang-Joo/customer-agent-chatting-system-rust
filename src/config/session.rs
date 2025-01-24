use async_redis_session::RedisSessionStore;
use async_session::{Session, SessionStore};
use axum::extract::{FromRef, FromRequestParts};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::user::user::{User, UserRole};

use super::{app_state::ArcAppState, error::AppError, MangJooResult};

const SESSION_KEY: &str = "user_session";

// 세션 관리를 위한 래퍼 구조체
#[derive(Clone, Debug)]
pub struct SessionManager {
    store: RedisSessionStore,
}

impl SessionManager {
    pub fn new(redis_session_store: RedisSessionStore) -> Self {
        Self {
            store: redis_session_store.with_prefix("user:session:"),
        }
    }

    pub async fn create_user_session(&self, user_session: UserSession) -> MangJooResult<String> {
        let mut session = Session::new();
        let result = session.insert(SESSION_KEY, &user_session);
        if result.is_err() {
            return Err(super::error::AppError::InternalError(
                "Create Session Failed.".to_string(),
            ));
        }

        // 세션 만료 시간 설정 (예: 24시간)
        session.expire_in(std::time::Duration::from_secs(24 * 60 * 60));

        let cookie_value = self
            .store
            .store_session(session)
            .await
            .map_err(|err| {
                AppError::InternalError(format!("Session insert error {}", err.to_string()))
            })?
            .unwrap();

        Ok(cookie_value)
    }

    pub async fn get_user_session(&self, session_id: &str) -> MangJooResult<UserSession> {
        let session = self
            .store
            .load_session(session_id.to_string())
            .await
            .map_err(|err| AppError::InternalError(format!("{}", err)))?;

        return if let Some(session) = session {
            Ok(session.get::<UserSession>("user_session").unwrap())
        } else {
            Err(AppError::Unauthorized("Session invaild.".to_string()))
        };
    }

    pub async fn renewal_user_session(&self, session_id: &str) -> MangJooResult<()> {
        let session = self
            .store
            .load_session(session_id.to_string())
            .await
            .map_err(|_err| AppError::Unauthorized("Session invaild.".to_string()))?;

        let mut session =
            session.ok_or_else(|| AppError::Unauthorized("Session invaild.".to_string()))?;

        session.expire_in(std::time::Duration::from_secs(24 * 60 * 60));

        let _ = self.store.store_session(session).await.map_err(|err| {
            AppError::InternalError(format!("Session insert error {}", err.to_string()))
        })?;

        Ok(())
    }
}

// 메인 세션 구조체
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserSession {
    pub user_id: i64,
    email: String,
    name: String,
    pub role: UserRole,
    last_login: DateTime<Utc>,
}

impl UserSession {
    pub fn new(user: &User) -> Self {
        Self {
            user_id: user.user_id,
            email: user.email.clone(),
            name: user.name.clone(),
            role: user.role.clone(),
            last_login: Utc::now(),
        }
    }

    pub fn is_agent(&self) -> bool {
        self.role == UserRole::Agent
    }

    pub fn is_user(&self) -> bool {
        self.role == UserRole::User
    }
}

#[derive(Debug, Clone)]
pub struct AuthUser(pub UserSession);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    ArcAppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> MangJooResult<Self> {
        let state = ArcAppState::from_ref(state);

        let session = parts
            .headers
            .get(http::header::COOKIE)
            .and_then(|value| value.to_str().ok())
            .and_then(|cookie_str| {
                println!("cookie {:?}", cookie_str);
                cookie_str
                    .split(';')
                    .find(|s| s.trim().starts_with("session_id="))
                    .map(|s| s.trim()[11..].to_string())
            })
            .ok_or_else(|| AppError::Unauthorized("Session Required.".to_string()))?;

        let session_manager = &state.session_store;
        let _ = session_manager.renewal_user_session(&session).await?;
        let user_session = session_manager.get_user_session(&session).await?;
        Ok(AuthUser(user_session))
    }
}

#[derive(Debug, Clone)]
pub struct RequiredAgent(pub UserSession);

impl<S> FromRequestParts<S> for RequiredAgent
where
    S: Send + Sync,
    ArcAppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        state: &S,
    ) -> MangJooResult<Self> {
        let session = AuthUser::from_request_parts(parts, state).await?.0;

        return if session.is_agent() {
            Ok(RequiredAgent(session))
        } else {
            Err(AppError::Unauthorized("Only agent".to_string()))
        };
    }
}

#[derive(Debug, Clone)]
pub struct RequiredUser(pub UserSession);

impl<S> FromRequestParts<S> for RequiredUser
where
    S: Send + Sync,
    ArcAppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        state: &S,
    ) -> MangJooResult<Self> {
        let session = AuthUser::from_request_parts(parts, state).await?.0;

        return if session.is_user() {
            Ok(RequiredUser(session))
        } else {
            Err(AppError::Unauthorized("Only User".to_string()))
        };
    }
}
