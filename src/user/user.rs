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
