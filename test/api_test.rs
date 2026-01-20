use actix_web::{test, web, App};
use sqlx::mysql::MySqlPool;
use std::env;

// Import modul dari aplikasi utama (pastikan main.rs mengekspos modulnya)
// Catatan: Biasanya untuk testing yang kompleks, kode di main.rs dipindah ke lib.rs

#[actix_web::test]
async fn test_get_users_endpoint() {
    dotenvy::dotenv().ok();
    let database_url = env::var("DATABASE_URL").unwrap();
    let pool = MySqlPool::connect(&database_url).await.unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(web::scope("").service(crate::controllers::get_users))
    ).await;

    let req = test::TestRequest::get().uri("/users").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
}