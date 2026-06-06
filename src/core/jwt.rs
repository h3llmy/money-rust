use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id (UUID)
    pub email: String,
    pub exp: usize,
    pub token_type: String,
}

pub fn generate_auth_tokens(user_id: Uuid, email: &str, secret: &str) -> Result<(String, String), String> {
    let now = Utc::now();
    let access_exp = now.checked_add_signed(chrono::Duration::try_hours(1).ok_or("Invalid duration")?)
        .ok_or_else(|| "Failed to calculate token expiration".to_string())?
        .timestamp() as usize;

    let refresh_exp = now.checked_add_signed(chrono::Duration::try_days(7).ok_or("Invalid duration")?)
        .ok_or_else(|| "Failed to calculate token expiration".to_string())?
        .timestamp() as usize;

    let access_claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        exp: access_exp,
        token_type: "access".to_string(),
    };

    let refresh_claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        exp: refresh_exp,
        token_type: "refresh".to_string(),
    };

    let access_token = encode(
        &Header::default(),
        &access_claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    ).map_err(|e| format!("JWT access token generation failed: {}", e))?;

    let refresh_token = encode(
        &Header::default(),
        &refresh_claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    ).map_err(|e| format!("JWT refresh token generation failed: {}", e))?;

    Ok((access_token, refresh_token))
}

pub fn decode_token(token: &str, secret: &str, expected_type: &str) -> Result<Claims, String> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| format!("Invalid token: {}", e))?;

    if token_data.claims.token_type != expected_type {
        return Err(format!("Invalid token type: expected {}", expected_type));
    }

    Ok(token_data.claims)
}
