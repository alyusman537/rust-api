use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Vec<u8>,
    pub name: String,
    pub email: String,
    pub password: String,
}

// #[derive(Deserialize, FromRow)]
// pub struct UserResponse {
//     pub id: String,
//     pub name: String,
//     pub email: String,
//     pub password: String,
// }

#[derive(Deserialize)]
pub struct CreateUserDto {
    pub name: String,
    pub email: String,
    pub password: String
}

#[derive(Deserialize)]
pub struct UpdateUserDto {
    pub name: String,
    pub email: String
}

#[derive(Serialize, Deserialize)]
pub struct LoginDto {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
}