use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub password: String,
    pub name: String,
}

impl User {
    pub fn new(id: i64, email: String, password: String, name: String) -> Self {
        Self {
            id,
            email,
            password,
            name,
        }
    }
}

// 권한을 나타내는 열거형
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum UserRole {
    Agent,
    User,
}

impl UserRole {
    pub fn is_user(&self) -> bool {
        self == &UserRole::User
    }

    pub fn is_agent(&self) -> bool {
        self == &UserRole::Agent
    }
}
