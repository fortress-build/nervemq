use actix_web::{
    delete,
    error::{ErrorBadRequest, ErrorInternalServerError},
    get, post, put,
    web::{self, Json},
    HttpResponse, Responder, Scope,
};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use serde_email::Email;
use sqlx::FromRow;

use crate::service::Service;

use super::auth::Role;

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    email: String,
    password: SecretString,
    role: Role,
    namespaces: Vec<String>,
}

#[post("/users")]
pub async fn create_user(
    data: web::Json<CreateUserRequest>,
    service: web::Data<Service>,
) -> actix_web::Result<impl Responder> {
    let data = data.into_inner();

    let email = Email::from_str(&data.email).map_err(|e| ErrorBadRequest(e))?;

    service
        .create_user(email, data.password, Some(data.role), data.namespaces)
        .await
        .map_err(|e| ErrorInternalServerError(e))?;

    // Return the plain API key (should be securely sent/stored by the user).
    Ok(HttpResponse::Ok())
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserInfo {
    email: String,
}

#[get("/users")]
pub async fn list_users(service: web::Data<Service>) -> actix_web::Result<impl Responder> {
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
    data: web::Json<DeleteUserRequest>,
    service: web::Data<Service>,
) -> actix_web::Result<impl Responder> {
    sqlx::query("DELETE FROM users WHERE email = $1")
        .bind(data.email.as_str())
        .execute(service.db())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok())
}

#[get("/users/{email}/permissions")]
pub async fn list_user_permissions(
    service: web::Data<Service>,
    email: web::Path<String>,
) -> actix_web::Result<web::Json<Vec<String>>> {
    let email = email.into_inner();

    let permissions: Vec<String> = sqlx::query_scalar(
        "
            SELECT ns.name FROM user_permissions p
            JOIN namespaces ns ON p.namespace = ns.id
            JOIN users u ON u.id = p.user
            WHERE u.email = $1
        ",
    )
    .bind(&email)
    .fetch_all(service.db())
    .await
    .map_err(ErrorInternalServerError)?;

    Ok(Json(permissions))
}

#[put("/users/{email}/permissions")]
pub async fn grant_user_permissions(
    service: web::Data<Service>,
    email: web::Path<String>,
    data: Json<Vec<String>>,
) -> actix_web::Result<impl Responder> {
    let email = email.into_inner();
    let mut tx = service
        .db()
        .begin()
        .await
        .map_err(ErrorInternalServerError)?;
    for namespace in data.iter() {
        sqlx::query(
            "
            INSERT INTO user_permissions (user, namespace)
            VALUES ((SELECT id FROM users WHERE email = $1), (SELECT id FROM namespaces WHERE name = $2))
            ON CONFLICT DO UPDATE
            ",
        )
        .bind(&email)
        .bind(namespace)
        .execute(&mut *tx)
        .await
        .map_err(ErrorInternalServerError)?;
    }
    tx.commit().await.map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok())
}

#[delete("/users/{email}/permissions")]
pub async fn revoke_user_permissions(
    service: web::Data<Service>,
    email: web::Path<String>,
    data: Json<Vec<String>>,
) -> actix_web::Result<impl Responder> {
    let email = email.into_inner();
    let mut tx = service
        .db()
        .begin()
        .await
        .map_err(ErrorInternalServerError)?;
    for namespace in data.iter() {
        sqlx::query(
            "
            DELETE FROM user_permissions
            WHERE user = (SELECT id FROM users WHERE email = $1)
            AND namespace = (SELECT id FROM namespaces WHERE name = $2)
            ",
        )
        .bind(&email)
        .bind(namespace)
        .execute(&mut *tx)
        .await
        .map_err(ErrorInternalServerError)?;
    }
    tx.commit().await.map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok())
}

#[post("/users/{email}/permissions")]
pub async fn update_user_permissions(
    service: web::Data<Service>,
    email: web::Path<String>,
    data: Json<Vec<String>>,
) -> actix_web::Result<impl Responder> {
    let email = email.into_inner();

    let mut tx = service
        .db()
        .begin()
        .await
        .map_err(ErrorInternalServerError)?;

    // Revoke all existing permissions.
    sqlx::query(
        "
        DELETE FROM user_permissions
        WHERE user = (SELECT id FROM users WHERE email = $1)
        ",
    )
    .bind(&email)
    .execute(&mut *tx)
    .await
    .map_err(ErrorInternalServerError)?;

    for namespace in data.iter() {
        sqlx::query(
            "
            INSERT INTO user_permissions (user, namespace)
            VALUES ((SELECT id FROM users WHERE email = $1), (SELECT id FROM namespaces WHERE name = $2))
            ON CONFLICT DO NOTHING
            ",
        )
        .bind(&email)
        .bind(namespace)
        .execute(&mut *tx)
        .await
        .map_err(ErrorInternalServerError)?;
    }

    tx.commit().await.map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok())
}

#[get("/users/{email}/role")]
async fn get_user_role(
    service: web::Data<Service>,
    email: web::Path<String>,
) -> actix_web::Result<web::Json<Role>> {
    let email = email.into_inner();
    let role: Role = sqlx::query_scalar(
        "
            SELECT role FROM users
            WHERE email = $1
        ",
    )
    .bind(&email)
    .fetch_one(service.db())
    .await
    .map_err(ErrorInternalServerError)?;
    Ok(Json(role))
}

#[post("/users/{email}/role")]
async fn set_user_role(
    service: web::Data<Service>,
    email: web::Path<String>,
    role: web::Json<Role>,
) -> actix_web::Result<impl Responder> {
    let email = email.into_inner();
    sqlx::query(
        "
            UPDATE users
            SET role = $2
            WHERE email = $1
        ",
    )
    .bind(&email)
    .bind(role.into_inner())
    .execute(service.db())
    .await
    .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok())
}

pub fn service() -> Scope {
    web::scope("/admin")
        .service(create_user)
        .service(delete_user)
        .service(list_users)
        .service(list_user_permissions)
        .service(grant_user_permissions)
        .service(revoke_user_permissions)
        .service(update_user_permissions)
        .service(get_user_role)
        .service(set_user_role)
}
