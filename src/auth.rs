
use actix_web::{dev::ServiceRequest, Error, HttpMessage};
use actix_web_httpauth::extractors::{
    bearer::{BearerAuth, Config},
    AuthenticationError,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject (user id)
    pub exp: usize,  // Expiration time
    pub iat: usize,  // Issued at
}

pub fn generate_token(user_id: &str, secret: &[u8]) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
        + 24 * 3600; // 24 hours from now
    
    let issued_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;
    
    let claims = Claims {
        sub: user_id.to_owned(),
        exp: expiration,
        iat: issued_at,
    };
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )
}

pub async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, Error> {
    let config = req
        .app_data::<Config>()
        .cloned()
        .unwrap_or_else(Default::default);
    
    // Get the JWT secret from app data
    let secret = req.app_data::<actix_web::web::Data<String>>()
        .expect("JWT secret not found in app data")
        .get_ref()
        .as_bytes();
    
    match validate_token(credentials.token(), secret) {
        Ok(claims) => {
            // Store user info in request extensions for handlers to access
            req.extensions_mut().insert(claims);
            Ok(req)
        }
        Err(_) => {
            Err(AuthenticationError::from(config).into())
        }
    }
}

fn validate_token(token: &str, secret: &[u8]) -> Result<Claims, jsonwebtoken::errors::Error> {
    let validation = Validation::default();
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret),
        &validation,
    )?;
    Ok(token_data.claims)
}
