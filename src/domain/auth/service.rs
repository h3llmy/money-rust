use crate::domain::auth::entity::User;
use crate::domain::auth::AuthRepository;
use crate::domain::auth::dto::{RegisterRequest, LoginRequest, AuthResponse, UserResponse};
use std::sync::Arc;
use crate::core::security::{hash_password, verify_password};
use crate::core::jwt::{generate_auth_tokens, decode_token};

pub struct AuthService {
    repo: Arc<dyn AuthRepository>,
    jwt_secret: String,
}

impl AuthService {
    pub fn new(repo: Arc<dyn AuthRepository>, jwt_secret: String) -> Self {
        Self { repo, jwt_secret }
    }

    pub async fn register(&self, req: RegisterRequest) -> Result<AuthResponse, String> {
        // 1. Check if user already exists
        if let Some(_) = self.repo.find_by_email(&req.email).await? {
            return Err("Email already in use".to_string());
        }

        // 2. Hash the password
        let hashed = hash_password(&req.password)?;

        // 3. Create the user
        let user = self.repo.create(&req.email, &req.username, &hashed).await?;

        // 4. Generate JWT tokens
        let (token, refresh_token) = generate_auth_tokens(user.id, &user.email, &self.jwt_secret)?;

        Ok(AuthResponse {
            token,
            refresh_token,
            user: UserResponse::from(user),
        })
    }

    pub async fn login(&self, req: LoginRequest) -> Result<AuthResponse, String> {
        // 1. Find user by email
        let user = self.repo.find_by_email(&req.email).await?
            .ok_or_else(|| "Invalid email or password".to_string())?;

        // 2. Verify password
        let valid = verify_password(&req.password, &user.password_hash)?;

        if !valid {
            return Err("Invalid email or password".to_string());
        }

        // 3. Generate tokens
        let (token, refresh_token) = generate_auth_tokens(user.id, &user.email, &self.jwt_secret)?;

        Ok(AuthResponse {
            token,
            refresh_token,
            user: UserResponse::from(user),
        })
    }

    pub async fn refresh(&self, refresh_token: &str) -> Result<AuthResponse, String> {
        let claims = decode_token(refresh_token, &self.jwt_secret, "refresh")?;

        let user_id = uuid::Uuid::parse_str(&claims.sub)
            .map_err(|_| "Invalid user ID in token".to_string())?;

        let user = self.repo.find_by_id(user_id).await?
            .ok_or_else(|| "User not found".to_string())?;

        let (token, new_refresh_token) = generate_auth_tokens(user.id, &user.email, &self.jwt_secret)?;

        Ok(AuthResponse {
            token,
            refresh_token: new_refresh_token,
            user: UserResponse::from(user),
        })
    }

    pub async fn get_user_by_id(&self, id: uuid::Uuid) -> Result<Option<User>, String> {
        self.repo.find_by_id(id).await
    }
}


