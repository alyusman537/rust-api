use actix_web::{dev::ServiceRequest, Error};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use jsonwebtoken::{decode, DecodingKey, Validation};
use actix_web::HttpMessage; // Trait is now in scope
use serde::{Serialize, Deserialize}; // You might need both

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Email user
    pub exp: usize,
}

pub async fn validator(
    req: ServiceRequest,
    auth: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let decoding_key = DecodingKey::from_secret("secret_banget".as_ref());
    
    // Validasi Token
    match decode::<Claims>(auth.token(), &decoding_key, &Validation::default()) {
        Ok(token_data) => {
            // Masukkan data user (email) ke dalam request extensions 
            // agar bisa diakses di controller jika dibutuhkan
            req.extensions_mut().insert(token_data.claims);
            Ok(req)
        }
        Err(_) => {
            // Jika token tidak valid atau expired
            let err = actix_web::error::ErrorUnauthorized("Token tidak valid atau kadaluarsa");
            Err((err, req))
        }
    }
}