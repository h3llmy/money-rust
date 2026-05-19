use bcrypt::{hash, verify, DEFAULT_COST};

pub fn hash_password(password: &str) -> Result<String, String> {
    hash(password, DEFAULT_COST).map_err(|e| format!("Password hashing failed: {}", e))
}

pub fn verify_password(password: &str, hashed: &str) -> Result<bool, String> {
    verify(password, hashed).map_err(|e| format!("Password verification failed: {}", e))
}
