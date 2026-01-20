use crate::auth::Claims;
use crate::errors::AppError;
use crate::models::{AuthResponse, CreateUserDto, LoginDto, UpdateUserDto};
use crate::services::UserService;
use actix_web::{HttpResponse, HttpMessage, Responder, delete, get, post, put, web};
use sqlx::MySqlPool;


#[get("/users")]
pub async fn get_users(
    req: actix_web::HttpRequest, // Tambahkan request
    pool: web::Data<MySqlPool>,
) -> Result<impl Responder, AppError> {
    // Ambil claims dari ekstensi request yang diisi oleh middleware tadi
    if let Some(claims) = req.extensions().get::<Claims>() {
        log::info!("User {} sedang mengakses data", claims.sub);
    }
    let users = UserService::fetch_all(pool.get_ref()).await?;
    Ok(web::Json(users))
}

#[put("/users/{id}")]
pub async fn update_user(
    pool: web::Data<MySqlPool>,
    id: web::Path<String>,
    body: web::Json<UpdateUserDto>,
) -> Result<impl Responder, AppError> {
    let user_id = id.into_inner();

    // Kita panggil service untuk update
    let rows_affected = UserService::update(pool.get_ref(), user_id, body.into_inner()).await?;

    // Jika tidak ada baris yang berubah, berarti ID tidak ditemukan
    if rows_affected == false {
        return Err(AppError::NotFound);
    }

    Ok(HttpResponse::Ok().json("User berhasil diperbarui"))
}

// --- DELETE ---
#[delete("/users/{id}")]
pub async fn delete_user(
    pool: web::Data<MySqlPool>,
    id: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let user_id = id.into_inner();

    let rows_affected = UserService::delete(pool.get_ref(), user_id).await?;

    if rows_affected == 0 {
        return Err(AppError::NotFound);
    }

    // 204 No Content adalah standar untuk delete yang berhasil tanpa mengembalikan body
    Ok(HttpResponse::NoContent().finish())
    // Ok(HttpResponse::Ok().json("User {user_id} berhasil di delete"))
}

#[post("/login")]
pub async fn login(
    pool: web::Data<MySqlPool>,
    body: web::Json<LoginDto>,
) -> Result<impl Responder, AppError> {
    let token = UserService::login(pool.get_ref(), body.into_inner()).await?;
    Ok(web::Json(AuthResponse { token }))
}

#[post("/register")]
pub async fn register(
    req: actix_web::HttpRequest, // Tambahkan request
    pool: web::Data<MySqlPool>,
    body: web::Json<CreateUserDto>,
) -> Result<impl Responder, AppError> {
    if let Some(claims) = req.extensions().get::<Claims>() {
        log::info!("User {} sedang menambahkan data", claims.sub);
    }
    // Memanggil service. Jika error, operator '?' akan melempar AppError
    UserService::register(pool.get_ref(), body.into_inner()).await?;

    // Mengembalikan 201 Created
    Ok(HttpResponse::Created().json("User berhasil dibuat"))
}
