use crate::errors::AppError;
use crate::models::LoginDto;
use crate::models::{CreateUserDto, UpdateUserDto, User};
use bcrypt::{DEFAULT_COST, hash, verify};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub struct UserService;

// Struktur untuk isi JWT
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String, // email user
    exp: usize,  // waktu kadaluarsa
}
impl UserService {
    pub async fn fetch_all(pool: &MySqlPool) -> Result<Vec<User>, AppError> {
        let users = sqlx::query_as::<_, User>("SELECT id, name, email, password FROM users")
            .fetch_all(pool)
            .await?; // Operator '?' otomatis mengubah sqlx::Error jadi AppError::DbError
        // Ok(users)
        Ok(users)
    }

    pub async fn update(
        pool: &MySqlPool,
        id: String,
        data: UpdateUserDto,
    ) -> Result<bool, AppError> {
        let result = sqlx::query("UPDATE users SET name = ?, email = ? WHERE id = ?")
            .bind(&data.name)
            .bind(&data.email)
            .bind(id)
            .execute(pool)
            .await;
        // Ok(result.rows_affected())
        match result {
            Ok(_) => {
                log::info!("User baru berhasil dibuat: {}", data.email);
                Ok(true)
            }
            Err(e) => {
                log::error!("Gagal menyimpan user ke database: {:?}", e); // Log error internal
                if let Some(db_error) = e.as_database_error() {
                    // Cek jika ada error duplicate entry (MySQL code 1062)
                    if db_error
                        .code()
                        .map_or(false, |code| code == "23000" || code == "1062")
                    {
                        return Err(AppError::Conflict);
                    }
                }
                Err(AppError::DbError(e))
            }
        }
    }
    pub async fn delete(pool: &MySqlPool, id: String) -> Result<u64, AppError> {
        let result = sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?; // sqlx::Error otomatis jadi AppError::DbError

        Ok(result.rows_affected())
    }
    pub async fn register(pool: &MySqlPool, data: CreateUserDto) -> Result<(), AppError> {
        let uuid_v4 = Uuid::new_v4();
        println!("{}", uuid_v4);
        // 1. Hash password sebelum simpan
        let hashed_password =
            hash(data.password, DEFAULT_COST).map_err(|_| AppError::InternalError)?;

        sqlx::query("INSERT INTO users (id, name, email, password) VALUES (?, ?, ?, ?)")
            .bind(uuid_v4.to_string())
            .bind(&data.name)
            .bind(&data.email)
            .bind(hashed_password)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn login(pool: &MySqlPool, data: LoginDto) -> Result<String, AppError> {
        // 1. Cari user berdasarkan email
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
            .bind(&data.email)
            .fetch_optional(pool)
            .await?
            .ok_or(AppError::NotFound)?; // User tidak ketemu

        // 2. Verifikasi password
        let is_valid =
            verify(data.password, &user.password).map_err(|_| AppError::InternalError)?;
        if !is_valid {
            return Err(AppError::Unauthorized); // Password salah
        }

        // 3. Generate JWT
        const EXP_TOKEN: u64 = 5000;
        let exp: u64 = env::var("EXP_TOKEN_IN_MINUTE")
            // Check if the env var was successfully retrieved (Result)
            .ok()
            // Attempt to parse the string into a u16 if it exists (Option)
            .and_then(|port_str| port_str.parse::<u64>().ok())
            // Use the default value if either step failed
            .unwrap_or(EXP_TOKEN);
        let expiration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + exp; // Berlaku 1 jam

        let klem = Claims {
            sub: user.email,
            exp: expiration as usize,
        };
        let token = encode(
            &Header::default(),
            &klem,
            &EncodingKey::from_secret("secret_banget".as_ref()),
        )
        .map_err(|_| AppError::InternalError)?;
        Ok(token)
    }
}
// ... kode UserService sebelumnya ...

#[cfg(test)]
mod tests {
    use super::*;
    use dotenvy::dotenv;
    use sqlx::mysql::MySqlPoolOptions;
    use std::env;

    // Fungsi pembantu untuk koneksi database khusus testing
    async fn setup_test_db() -> MySqlPool {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL harus ada");
        MySqlPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("Gagal konek database test")
    }

    #[tokio::test]
    async fn test_create_user_success() {
        let pool = setup_test_db().await;

        // Data dummy
        let dto = CreateUserDto {
            name: "Test User".to_string(),
            email: format!("test_{}@example.com", chrono::Utc::now().timestamp_millis()),
            password: "sdf sdlkfjsdlkfjsdf".to_string(),
        };

        // Jalankan fungsi create
        let result = UserService::register(&pool, dto).await;

        // Assert: Pastikan hasilnya Ok
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fetch_all_users() {
        let pool = setup_test_db().await;

        let result = UserService::fetch_all(&pool).await;

        // Assert: Pastikan returnnya Result yang berisi Vec
        assert!(result.is_ok());
    }
}
