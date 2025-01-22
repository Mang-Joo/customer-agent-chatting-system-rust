use crate::config::{
    error::AppError,
    hash::{hash, verify},
    jwt::JwtManager,
    MangJooResult,
};

use super::repository::UserRepository;

#[derive(Debug, Clone)]
pub struct UserService {
    user_repository: UserRepository,
}

impl UserService {
    pub fn new(user_repository: UserRepository) -> Self {
        Self { user_repository }
    }

    pub async fn register(
        &self,
        user_register: UserRegister,
        jwt_manager: &JwtManager,
    ) -> MangJooResult<String> {
        let register = self
            .user_repository
            .register(user_register.hash_password().await?)
            .await?;

        let token = jwt_manager.generate_token(register.id, "User")?;

        Ok(token)
    }

    pub async fn login(&self, login: UserLogin, jwt_manager: &JwtManager) -> MangJooResult<String> {
        let user = self.user_repository.find_by_email(login.email).await?;

        let verify_password = verify(&login.password, &user.password).await;

        match verify_password {
            true => Ok(jwt_manager.generate_token(user.id, "User")?),
            false => Err(AppError::Unauthorized("Invalid Password".to_string())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UserRegister {
    pub email: String,
    pub password: String,
    pub name: String,
}

impl UserRegister {
    pub fn new(email: String, password: String, name: String) -> Self {
        Self {
            email,
            password,
            name,
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
