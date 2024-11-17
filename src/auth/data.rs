use actix_web::{web, Responder};
use argon2::{
    password_hash::{PasswordHashString, Salt},
    Argon2, PasswordHasher, PasswordVerifier,
};
use base64::{prelude::BASE64_STANDARD, Engine};
use rand::Rng;
use serde::{Deserialize, Serialize};
use snafu::Snafu;
use sqlx::SqlitePool;

#[derive(Serialize, Deserialize)]
pub struct ApiKeyRequest {
    key_id: String,
    api_key: String,
}

pub async fn create_api_key(pool: web::Data<SqlitePool>) -> actix_web::Result<impl Responder> {
    let Ok((hashed_key, raw_key, key_id)) = gen_api_key().await else {
        return Err(actix_web::error::ErrorInternalServerError("Auth failed"));
    };

    sqlx::query("INSERT INTO api_keys (key_id, hashed_key) VALUES ($1, $2)")
        .bind(&key_id)
        .bind(hashed_key.to_string())
        .execute(pool.get_ref())
        .await
        .expect("Failed to insert API key");

    // Return the plain API key (should be securely sent/stored by the user).
    Ok(web::Json(serde_json::json!({
        "client_id": key_id,
        "secret": raw_key
    })))
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

pub async fn gen_api_key() -> eyre::Result<(PasswordHashString, String, String)> {
    match web::block(|| {
        // Generate a random API key id
        let key_id: String = (0..32)
            .map(|_| rand::thread_rng().gen_range(33..127) as u8 as char)
            .collect();

        // Generate a random API key
        let api_key: String = (0..32)
            .map(|_| rand::thread_rng().gen_range(33..127) as u8 as char)
            .collect();

        // Hash the API key using Argon2
        let argon2 = Argon2::default();
        let salt = {
            let mut rng = rand::thread_rng();
            rng.gen::<[u8; 16]>()
        };

        Ok((
            argon2
                .hash_password(
                    api_key.as_bytes(),
                    Salt::from_b64(BASE64_STANDARD.encode(salt).as_str())?,
                )?
                .serialize(),
            api_key,
            key_id,
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
