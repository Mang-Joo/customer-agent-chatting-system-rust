use crate::config::{
    error::AppError,
    hash::{hash, verify},
    session::{SessionManager, UserSession},
    MangJooResult,
};

use super::{repository::UserRepository, user::UserRole};

#[derive(Debug, Clone)]
pub struct UserService {
    user_repository: UserRepository,
}

impl UserService {
    pub fn new(user_repository: UserRepository) -> Self {
        Self { user_repository }
    }

    pub async fn register(&self, user_register: UserRegister) -> MangJooResult<()> {
        let _ = self
            .user_repository
            .register(user_register.hash_password().await?)
            .await?;

        Ok(())
    }

    pub async fn login(
        &self,
        login: UserLogin,
        session_manager: &SessionManager,
    ) -> MangJooResult<String> {
        let user = self.user_repository.find_by_email(login.email).await?;

        let verify_password = verify(&login.password, &user.password).await;

        match verify_password {
            true => Ok(session_manager
                .create_user_session(UserSession::new(&user))
                .await?),
            false => Err(AppError::Unauthorized("Invalid Password".to_string())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserRegister {
    pub email: String,
    pub password: String,
    pub name: String,
    pub role: UserRole,
}

impl UserRegister {
    pub fn new(email: String, password: String, name: String, role: UserRole) -> Self {
        Self {
            email,
            password,
            name,
            role,
        }
    }

    pub async fn hash_password(self) -> MangJooResult<Self> {
        let hash_password = hash(&self.password).await?;
        Ok(Self {
            password: hash_password,
            ..self
        })
    }
}

#[derive(Debug)]
pub struct UserLogin {
    pub email: String,
    pub password: String,
}

impl UserLogin {
    pub fn new(email: String, password: String) -> Self {
        Self { email, password }
    }
}
