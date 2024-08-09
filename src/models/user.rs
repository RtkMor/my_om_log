use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub email: String,
    pub password: String,
    pub is_admin: bool
}