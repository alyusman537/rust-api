use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DbError(#[from] sqlx::Error),

    #[error("User tidak ditemukan")]
    NotFound,

    #[error("Email sudah terdaftar")]
    Conflict,

    #[error("Email atau password salah")]
    Unauthorized,
    #[error("Terjadi kesalahan internal")]
    InternalError,
}

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ErrorResponse {
            message: self.to_string(),
        })
    }
}