use actix_web::web;
use argon2::password_hash::PasswordHashString;
use sqlx::SqlitePool;

use crate::{
    api::auth::User,
    auth::{credential::ApiKey, crypto::verify_secret},
    error::Error,
};

pub async fn authenticate_api_key(pool: &SqlitePool, token: ApiKey) -> Result<User, Error> {
    let key_id = token.short_token;

    let Some((hashed_key, email)) = sqlx::query_as::<_, (String, String)>(
        "
        SELECT k.hashed_key, u.email FROM api_keys k
        JOIN users u ON u.id = k.user
        WHERE key_id = $1
        ",
    )
    .bind(&key_id)
    .fetch_optional(pool)
    .await?
    else {
        return Err(Error::IdentityNotFound {
            key_id: key_id.to_string(),
        });
    };

    let Ok(hashed_key) = PasswordHashString::new(&hashed_key) else {
        return Err(Error::InternalServerError { source: None });
    };

    match web::block(move || verify_secret(token.long_token, hashed_key))
        .await
        .map_err(|e| e.into())
        .and_then(|res| res)
    {
        Ok(_) => {}
        Err(err) => {
            tracing::warn!("Failed to authenticate key id {}: {}", key_id, err);
            return Err(err.into());
        }
    }

    let user = sqlx::query_as::<_, User>(
        "
        SELECT * FROM users
        WHERE email = $1
        ",
    )
    .bind(&email)
    .fetch_one(pool)
    .await?;

    return Ok(user);
}
