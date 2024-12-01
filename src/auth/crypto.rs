use argon2::{
    password_hash::{PasswordHashString, PasswordHasher, SaltString},
    Argon2, PasswordVerifier,
};
use hmac::{Hmac, Mac};
use rand::Rng;
use secrecy::{ExposeSecret, SecretBox, SecretSlice, SecretString};
use sha2::{Digest, Sha256};

pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

pub fn gen_signature_key(key: &str, date: &str, region: &str, service: &str) -> SecretSlice<u8> {
    let key_secret = format!("AWS4{}", key);

    let sign = |msg: &[u8], key: &[u8]| -> Vec<u8> {
        let mut mac =
            hmac::Hmac::<Sha256>::new_from_slice(key).expect("HMAC can take key of any size");
        mac.update(msg);
        mac.finalize().into_bytes().to_vec()
    };

    let date_key = sign(key_secret.as_bytes(), date.as_bytes());
    let date_region_key = sign(&date_key, region.as_bytes());
    let date_region_service_key = sign(&date_region_key, service.as_bytes());
    sign(&date_region_service_key, b"aws4_request").into()
}

pub fn hash_secret(secret: SecretString) -> eyre::Result<PasswordHashString> {
    let argon2 = Argon2::default();

    let salt = SaltString::generate(&mut rand::thread_rng());

    Ok(argon2
        .hash_password(secret.expose_secret().as_bytes(), salt.as_salt())?
        .serialize())
}

pub fn verify_secret(secret: SecretString, hash: PasswordHashString) -> eyre::Result<()> {
    Ok(Argon2::default()
        .verify_password(secret.expose_secret().as_bytes(), &hash.password_hash())?)
}

pub struct GeneratedKey {
    /// The identifier part of the API key
    pub short_token: String,
    /// The secret part of the API key
    pub long_token: SecretString,
    /// The hashed secret part of the API key
    pub long_token_hash: PasswordHashString,
    /// The validation key derived from the secret key before hashing
    pub validation_key: Vec<u8>,
}

fn generate_token<const N: usize>(mut rng: impl Rng) -> eyre::Result<String> {
    let mut token = [0u8; N];
    rng.try_fill_bytes(&mut token)?;
    Ok(bs58::encode(token).into_string())
}

#[test]
fn test_gen_token() {
    let mut rng = rand::thread_rng();

    let token = generate_token::<8>(&mut rng).unwrap();

    assert_eq!(token.len(), 8);

    let token = generate_token::<12>(&mut rng).unwrap();

    assert_eq!(token.len(), 12);
}

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

    let validation_key = {
        let mut mac = Hmac::<Sha256>::new_from_slice(salt.as_str().as_bytes())?;

        mac.update(long_token.as_bytes());

        mac.finalize().into_bytes().to_vec()
    };

    Ok(GeneratedKey {
        short_token,
        long_token: SecretBox::new(long_token.into()),
        long_token_hash,
        validation_key,
    })
}
