use actix_web::web;
use argon2::{
    password_hash::{PasswordHashString, SaltString},
    Argon2, PasswordHasher, PasswordVerifier,
};
use prefixed_api_key::{PrefixedApiKey, PrefixedApiKeyController};
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

pub const API_KEY_PREFIX: &str = "nervemq";

pub async fn authenticate_api_key(pool: &SqlitePool, token: PrefixedApiKey) -> eyre::Result<()> {
    if token.prefix() != API_KEY_PREFIX {
        return Err(Error::Unauthorized.into());
    }

    let key_id = token.short_token();

    let Some(hashed_key) =
        sqlx::query_scalar::<_, String>("SELECT hashed_key FROM api_keys WHERE key_id = $1")
            .bind(key_id)
            .fetch_optional(pool)
            .await?
    else {
        return Err(Error::IdentityNotFound {
            key_id: key_id.to_string(),
        }
        .into());
    };

    let Ok(hashed_key) = PasswordHashString::new(&hashed_key) else {
        return Err(Error::InternalError.into());
    };

    if let Err(e) = verify_api_key(token.long_token().to_owned(), hashed_key).await {
        tracing::warn!("Failed to authenticate key id {}: {}", key_id, e);
        return Err(Error::Unauthorized.into());
    }

    return Ok(());
}

pub type ApiKeyGenerator = PrefixedApiKeyController<rand::rngs::OsRng, sha2::Sha256>;
pub type ApiKey = PrefixedApiKey;

pub struct GeneratedKey {
    pub api_key: PrefixedApiKey,
    pub short_token: String,
    pub long_token_hash: PasswordHashString,
}

pub async fn gen_api_key(controller: web::Data<ApiKeyGenerator>) -> eyre::Result<GeneratedKey> {
    match web::block(move || {
        let api_key = controller.generate_key();

        let short_token = api_key.short_token().to_owned();
        let long_token = api_key.long_token();

        // Hash the API key using Argon2
        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut rand::thread_rng());

        let long_token_hash = argon2
            .hash_password(long_token.as_bytes(), salt.as_salt())?
            .serialize();

        Ok(GeneratedKey {
            api_key,
            short_token,
            long_token_hash,
        })
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
