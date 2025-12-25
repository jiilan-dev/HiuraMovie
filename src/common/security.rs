use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use anyhow::{anyhow, Result};

pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow!(e.to_string()))?
        .to_string();
    Ok(password_hash)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| anyhow!(e.to_string()))?;
    
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map(|_| true)
        .map_err(|_| anyhow!("Invalid password"))
}
