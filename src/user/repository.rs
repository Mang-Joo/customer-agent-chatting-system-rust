use chrono::NaiveDateTime;
use sqlx::PgPool;

use crate::config::{error::AppError, MangJooResult};

use super::{service::UserRegister, user::User};

#[derive(Debug, Clone)]
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn register(&self, user: UserRegister) -> MangJooResult<User> {
        let result = sqlx::query_as!(
            UserEntity,
            "INSERT INTO users (email, password, name)
            VALUES ($1, $2, $3)
            RETURNING * 
            ",
            user.email,
            user.password,
            user.name
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| AppError::DatabaseError(format!("DB Error {}", err.to_string())))?;

        Ok(result.to_user())
    }

    pub async fn find_by_email(&self, email: String) -> MangJooResult<User> {
        let user_entity = sqlx::query_as!(
            UserEntity,
            "SELECT * 
            FROM users
            WHERE email = ($1)
            ",
            email
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|err| AppError::DatabaseError(format!("DB Error {}", err.to_string())))?;

        match user_entity {
            Some(user_entity) => Ok(user_entity.to_user()),
            None => Err(AppError::InvalidRequest("Invalid email".to_string())),
        }
    }
}

#[derive(Debug)]
pub struct UserEntity {
    user_id: i64,
    email: String,
    password: String,
    name: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
    deleted: bool,
}
impl UserEntity {
    fn to_user(self) -> User {
        User::new(self.user_id, self.email, self.password, self.name)
    }
}
