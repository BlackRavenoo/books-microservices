use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};

#[derive(Debug, thiserror::Error)]
pub enum PasswordError {
    #[error("Invalid password")]
    InvalidPassword,
    #[error("Failed to hash password: {0}")]
    HashingError(String),
    #[error("Failed to verify password: {0}")]
    VerificationError(String),
}

pub fn hash_password(password: String) -> Result<String, PasswordError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| PasswordError::HashingError(e.to_string()))
}

pub fn verify_password(
    password: String,
    stored_hash: &str,
) -> Result<(), PasswordError> {
    let parsed_hash = PasswordHash::new(stored_hash)
        .map_err(|e| PasswordError::VerificationError(e.to_string()))?;
    
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| PasswordError::InvalidPassword)
}