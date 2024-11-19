use actix_identity::Identity;
use actix_web::{
    delete,
    error::ErrorInternalServerError,
    get, post,
    web::{self, Json},
    HttpResponse, Responder, Scope,
};
use argon2::{
    password_hash::{PasswordHashString, PasswordHasher, SaltString},
    Argon2,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::service::Service;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    email: String,
    password: String,
}

pub async fn hash_password(password: String) -> eyre::Result<PasswordHashString> {
    match web::block(move || {
        let argon2 = Argon2::default();

        let salt = SaltString::generate(&mut rand::thread_rng());

        Ok(argon2
            .hash_password(password.as_bytes(), salt.as_salt())?
            .serialize())
    })
    .await
    {
        Ok(Ok(res)) => Ok(res),
        Ok(Err(e)) => Err(e),
        Err(e) => Err(eyre::eyre!("Failed to join create API key task: {e}")),
    }
}

#[post("/users")]
pub async fn create_user(
    data: web::Json<CreateUserRequest>,
    service: web::Data<Service>,
    // identity: Identity,
) -> actix_web::Result<impl Responder> {
    let data = data.into_inner();
    let hashed_password = hash_password(data.password)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    sqlx::query("INSERT INTO users (email, hashed_pass) VALUES ($1, $2)")
        .bind(&data.email)
        .bind(hashed_password.to_string())
        .execute(service.db())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // Return the plain API key (should be securely sent/stored by the user).
    Ok(HttpResponse::Ok())
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserInfo {
    email: String,
}

#[get("/users")]
pub async fn list_users(
    identity: Identity,
    service: web::Data<Service>,
) -> actix_web::Result<impl Responder> {
    let users: Vec<UserInfo> = sqlx::query_as("SELECT email FROM users")
        .fetch_all(service.db())
        .await
        .map_err(ErrorInternalServerError)?;

    Ok(Json(users))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteUserRequest {
    email: String,
}

#[delete("/users")]
pub async fn delete_user(
    identity: Identity,
    data: web::Json<DeleteUserRequest>,
    service: web::Data<Service>,
) -> actix_web::Result<impl Responder> {
    sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(&data.email)
        .execute(service.db())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok())
}

pub fn service() -> Scope {
    web::scope("/admin")
        .service(create_user)
        .service(delete_user)
        .service(list_users)
}
