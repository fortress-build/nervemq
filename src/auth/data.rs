use actix_web::web;
use argon2::{
    password_hash::{PasswordHashString, SaltString},
    Argon2, PasswordHasher, PasswordVerifier,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use snafu::Snafu;
use sqlx::SqlitePool;

#[derive(Serialize, Deserialize)]
pub struct ApiKeyRequest {
    api_key: String,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Unauthorized"))]
    Unauthorized,
    #[snafu(display("Internal server error"))]
    InternalError,
    #[snafu(display("Identity {key_id} not found"))]
    IdentityNotFound { key_id: String },
}

pub async fn authenticate_api_key(
    pool: &SqlitePool,
    key_id: String,
    api_key: String,
) -> eyre::Result<()> {
    let Some(hashed_key) =
        sqlx::query_scalar::<_, String>("SELECT hashed_key FROM api_keys WHERE key_id = $1")
            .bind(&key_id)
            .fetch_optional(pool)
            .await?
    else {
        return Err(Error::IdentityNotFound { key_id }.into());
    };

    let Ok(hashed_key) = PasswordHashString::new(&hashed_key) else {
        return Err(Error::InternalError.into());
    };

    if let Err(e) = verify_api_key(api_key, hashed_key).await {
        tracing::warn!("Failed to authenticate key id {}: {}", key_id, e);
        return Err(Error::Unauthorized.into());
    }

    return Ok(());
}

pub async fn gen_api_key() -> eyre::Result<(PasswordHashString, String)> {
    match web::block(|| {
        // Generate a random API key
        let api_key: String = (0..32)
            .map(|_| rand::thread_rng().gen_range(33..127) as u8 as char)
            .collect();

        // Hash the API key using Argon2
        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut rand::thread_rng());

        Ok((
            argon2
                .hash_password(api_key.as_bytes(), salt.as_salt())?
                .serialize(),
            api_key,
        ))
    })
    .await
    {
        Ok(Ok(res)) => Ok(res),
        Ok(Err(e)) => Err(e),
        Err(e) => Err(eyre::eyre!("Failed to join create API key task: {e}")),
    }
}

pub async fn verify_api_key(api_key: String, hashed_key: PasswordHashString) -> eyre::Result<()> {
    match web::block(move || {
        Argon2::default()
            .verify_password(api_key.as_bytes(), &hashed_key.password_hash())
            .map_err(|e| eyre::eyre!(e))
    })
    .await
    {
        Ok(Ok(res)) => Ok(res),
        Ok(Err(e)) => Err(e),
        Err(e) => Err(eyre::eyre!(
            "Failed to join create API key verify task: {e}"
        )),
    }
}
