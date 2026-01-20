mod auth;
mod controllers;
mod errors;
mod models;
mod services;

use actix_web::{App, HttpServer, middleware::Logger, web};
use actix_web_httpauth::middleware::HttpAuthentication;
use dotenvy::dotenv;
use sqlx::mysql::MySqlPool;
use std::env;
// --- Main Function ---

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    // Inisialisasi logger
    // Default log level diatur ke 'info' jika tidak ada RUST_LOG di .env
    if env::var("RUST_LOG").is_err() {
        // env::set_var("RUST_LOG", "actix_web=info,info").await;
        unsafe {
            env::set_var("RUST_LOG", "actix_web=info,info");
        }
        assert_eq!(env::var("RUST_LOG"), Ok("actix_web=info,info".to_string()));
    }
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = MySqlPool::connect(&database_url)
        .await
        .expect("Failed to connect to MySQL");

    let app_port = env::var("APP_PORT").unwrap_or_else(|_| "8080".to_string());
    let port: u16 = app_port.parse().expect("PORT must be a valid number");
    // let app_port = format!("{}", app_port_int);// app_port_int.to_string();
    println!("Server running at http://127.0.0.1:{}", port);

    HttpServer::new(move || {
        // Inisialisasi middleware auth
        let auth_middleware = HttpAuthentication::bearer(auth::validator);

        App::new()
            .app_data(web::Data::new(pool.clone()))
            // Tambahkan Middleware Logger di sini
            .wrap(Logger::default())
            // Anda juga bisa kustomisasi formatnya:
            // .wrap(Logger::new("%a %r %s %b %T"))
            // Route Publik
            .service(controllers::login)
            .service(controllers::register)
            // Route Privat (Semua di dalam scope ini butuh Token)
            .service(
                web::scope("/api")
                    .wrap(auth_middleware)
                    .service(controllers::get_users)
                    // .service(controllers::create_user)
                    .service(controllers::update_user)
                    .service(controllers::delete_user),
            )
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}

