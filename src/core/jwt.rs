use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id (UUID)
    pub email: String,
    pub exp: usize,
}

pub fn generate_token(user_id: Uuid, email: &str, secret: &str) -> Result<String, String> {
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::try_days(7).ok_or("Invalid duration")?)
        .ok_or_else(|| "Failed to calculate token expiration".to_string())?
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| format!("JWT generation failed: {}", e))
}

pub fn decode_token(token: &str, secret: &str) -> Result<Claims, String> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|token_data| token_data.claims)
    .map_err(|e| format!("Invalid token: {}", e))
}
