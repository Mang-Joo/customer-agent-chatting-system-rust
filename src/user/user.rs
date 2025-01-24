use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct User {
    pub user_id: i64,
    pub email: String,
    pub password: String,
    pub name: String,
    pub role: UserRole,
}

impl User {
    pub fn new(id: i64, email: String, password: String, name: String, role: UserRole) -> Self {
        Self {
            user_id: id,
            email,
            password,
            name,
            role,
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

impl From<String> for UserRole {
    fn from(value: String) -> Self {
        if value == String::from("agent") {
            Self::Agent
        } else if value == String::from("user") {
            Self::User
        } else {
            Self::User
        }
    }
}

impl ToString for UserRole {
    fn to_string(&self) -> String {
        match self {
            UserRole::Agent => String::from("agent"),
            UserRole::User => String::from("user"),
        }
    }
}
