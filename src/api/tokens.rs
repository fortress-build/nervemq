use std::future::{ready, Ready};

use actix_identity::Identity;
use actix_web::{
    delete,
    error::{ErrorInternalServerError, ErrorNotFound, ErrorUnauthorized},
    get, post,
    web::{self, Json},
    FromRequest, HttpRequest, HttpResponse, Responder, Scope,
};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::{
    auth::crypto::{generate_api_key, GeneratedKey},
    error::Error,
    service::Service,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTokenRequest {
    name: String,
    namespace: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTokenResponse {
    name: String,
    namespace: String,
    access_key: String,
    secret_key: String,
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
    let data = data.into_inner();

    let GeneratedKey {
        short_token,
        long_token,
        long_token_hash,
        validation_key,
    } = web::block(|| generate_api_key())
        .await
        .map_err(Error::internal)?
        .map_err(Error::internal)?;

    let namespace_id = service
        .get_namespace_id(&data.namespace, service.db())
        .await
        .map_err(Error::internal)?
        .ok_or(Error::NotFound)?;

    sqlx::query(
        "
        INSERT INTO api_keys (name, user, key_id, hashed_key, validation_key, ns)
        VALUES ($1, (SELECT id FROM users WHERE email = $2), $3, $4, $5, $6)
        ",
    )
    .bind(&data.name)
    .bind(identity.id().map_err(ErrorUnauthorized)?)
    .bind(&short_token)
    .bind(long_token_hash.to_string())
    .bind(validation_key)
    .bind(namespace_id as i64)
    .execute(service.db())
    .await
    .map_err(ErrorInternalServerError)?;

    // Return the plain API key (should be securely sent/stored by the user).
    Ok(web::Json(CreateTokenResponse {
        name: data.name,
        namespace: data.namespace,
        access_key: short_token,
        secret_key: long_token.expose_secret().to_owned(),
    }))
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

pub struct IdentityWrapped(pub actix_identity::Identity);

impl FromRequest for IdentityWrapped {
    type Error = <Identity as FromRequest>::Error;
    type Future = Ready<Result<IdentityWrapped, Self::Error>>;

    fn from_request(req: &HttpRequest, pl: &mut actix_web::dev::Payload) -> Self::Future {
        if let Ok(identity) = Identity::from_request(req, pl).into_inner() {
            return ready(Ok(IdentityWrapped(identity)));
        }

        ready(Err(ErrorUnauthorized("no identity found")))
    }
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
