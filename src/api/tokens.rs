use actix_identity::Identity;
use actix_web::{
    delete,
    error::{ErrorInternalServerError, ErrorNotFound, ErrorUnauthorized},
    get, post,
    web::{self, Json},
    HttpResponse, Responder, Scope,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::{error::Error, service::Service};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTokenRequest {
    pub name: String,
    pub namespace: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTokenResponse {
    pub name: String,
    pub namespace: String,
    pub access_key: String,
    pub secret_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteTokenRequest {
    name: String,
}

#[post("")]
pub async fn create_token(
    data: web::Json<CreateTokenRequest>,
    service: web::Data<Service>,
    identity: Identity,
) -> Result<Json<CreateTokenResponse>, Error> {
    let CreateTokenRequest { name, namespace } = data.into_inner();

    service
        .create_token(name, namespace, identity)
        .await
        .map(Json)
}

#[delete("")]
pub async fn delete_token(
    service: web::Data<Service>,
    data: web::Json<DeleteTokenRequest>,
    identity: Identity,
) -> actix_web::Result<impl Responder> {
    let res = sqlx::query(
        "
        DELETE FROM api_keys
        WHERE
            name = $1
        AND
            user IN (SELECT id FROM users WHERE email = $2)
    ",
    )
    .bind(&data.name)
    .bind(&identity.id().map_err(ErrorUnauthorized)?)
    .execute(service.db())
    .await
    .map_err(|e| ErrorInternalServerError(e))?;

    if res.rows_affected() == 0 {
        return Err(ErrorNotFound(format!("No such api key {}", data.name)));
    }

    Ok(HttpResponse::Ok())
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct ApiKey {
    name: String,
    namespace: String,
}

#[get("")]
pub async fn list_tokens(
    service: web::Data<Service>,
    identity: Identity,
) -> actix_web::Result<web::Json<Vec<ApiKey>>> {
    let email = match identity.id() {
        Ok(email) => email,
        Err(err) => {
            return Err(ErrorUnauthorized(err));
        }
    };

    let tokens = sqlx::query_as(
        "
        SELECT *, ns.name as namespace FROM users u
        INNER JOIN api_keys k ON u.id = k.user
        JOIN namespaces ns ON k.ns = ns.id
        WHERE u.email = $1
    ",
    )
    .bind(&email)
    .fetch_all(service.db())
    .await
    .map_err(ErrorInternalServerError)?;

    Ok(Json(tokens))
}

pub fn service() -> Scope {
    web::scope("/tokens")
        .service(create_token)
        .service(delete_token)
        .service(list_tokens)
}
