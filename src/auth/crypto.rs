//! Cryptographic utilities for authentication.
//!
//! Provides functions for hashing passwords, generating API keys,
//! and verifying credentials using Argon2 and SHA-256.

use argon2::{
    password_hash::{PasswordHashString, PasswordHasher, SaltString},
    Argon2, PasswordVerifier,
};
use rand::Rng;
use secrecy::{ExposeSecret, SecretString};
use sha2::{Digest, Sha256};

/// Computes SHA-256 hash of data and returns it as a hex string.
pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Hashes a secret using Argon2 with a random salt.
pub fn hash_secret(secret: String) -> eyre::Result<PasswordHashString> {
    let argon2 = Argon2::default();

    let salt = SaltString::generate(&mut rand::thread_rng());

    Ok(argon2
        .hash_password(secret.as_bytes(), salt.as_salt())?
        .serialize())
}

/// Verifies a secret against its Argon2 hash.
pub fn verify_secret(secret: SecretString, hash: PasswordHashString) -> eyre::Result<()> {
    Ok(Argon2::default()
        .verify_password(secret.expose_secret().as_bytes(), &hash.password_hash())?)
}

/// Represents a generated API key with its components and hash.
pub struct GeneratedKey {
    /// The identifier part of the API key
    pub short_token: String,
    /// The secret part of the API key
    pub long_token: String,
    /// The hashed secret part of the API key
    pub long_token_hash: PasswordHashString,
}

/// Generates a random token of size N bytes, encoded in base58.
pub fn generate_token<const N: usize>(mut rng: impl Rng) -> eyre::Result<String> {
    let mut token = [0u8; N];
    rng.try_fill_bytes(&mut token)?;
    Ok(bs58::encode(token).into_string())
}

/// Generates a new API key with short identifier and long secret components.
pub fn generate_api_key() -> eyre::Result<GeneratedKey> {
    let mut rng = rand::thread_rng();
    let short_token = generate_token::<8>(&mut rng)?;
    let long_token = generate_token::<24>(&mut rng)?;

    // Hash the API key using Argon2
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut rand::thread_rng());

    let long_token_hash = argon2
        .hash_password(long_token.as_bytes(), salt.as_salt())?
        .serialize();

    Ok(GeneratedKey {
        short_token,
        long_token,
        long_token_hash,
    })
}
