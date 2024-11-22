use actix_web::web;
use argon2::password_hash::PasswordHashString;
use sqlx::SqlitePool;

use crate::auth::{credential::ApiKey, crypto::verify_secret, error::Error};

pub async fn authenticate_api_key(pool: &SqlitePool, token: ApiKey) -> eyre::Result<()> {
    let key_id = token.short_token;

    let Some(hashed_key) =
        sqlx::query_scalar::<_, String>("SELECT hashed_key FROM api_keys WHERE key_id = $1")
            .bind(&key_id)
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

    match web::block(move || verify_secret(token.long_token, hashed_key))
        .await
        .map_err(|e| e.into())
        .and_then(|res| res)
    {
        Ok(_) => {}
        Err(err) => {
            tracing::warn!("Failed to authenticate key id {}: {}", key_id, err);
            return Err(err);
        }
    }

    return Ok(());
}
