use std::future::{ready, Ready};

use actix_identity::Identity;
use actix_web::{
    delete,
    error::{ErrorInternalServerError, ErrorNotFound, ErrorUnauthorized},
    get,
    // middleware::Identity,
    post,
    web::{self, Json},
    FromRequest,
    HttpRequest,
    HttpResponse,
    Responder,
    Scope,
};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use sqlx::FromRow;

use crate::{auth::data::gen_api_key, service::Service};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTokenRequest {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTokenResponse {
    secret: String,
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
) -> actix_web::Result<Json<CreateTokenResponse>> {
    let (hashed_key, raw_key) = gen_api_key().await.map_err(ErrorInternalServerError)?;

    sqlx::query(
        "
        INSERT INTO api_keys (name, user, hashed_key)
        VALUES ($1, (SELECT id FROM users WHERE email = $2), $3)
        ",
    )
    .bind(&data.name)
    .bind(identity.id().map_err(ErrorUnauthorized)?)
    .bind(hashed_key.to_string())
    .execute(service.db())
    .await
    .map_err(ErrorInternalServerError)?;

    // Return the plain API key (should be securely sent/stored by the user).
    Ok(web::Json(CreateTokenResponse { secret: raw_key }))
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
        SELECT * FROM users u
        INNER JOIN api_keys k ON u.id = k.user
        WHERE u.email = $1
    ",
    )
    .bind(&email)
    .fetch_all(service.db())
    .await
    .map_err(ErrorInternalServerError)?;

    Ok(Json(tokens))
}

#[get("/client_id")]
pub async fn client_id(
    service: web::Data<Service>,
    identity: Identity,
) -> actix_web::Result<impl Responder> {
    let email = identity.id().map_err(|e| ErrorUnauthorized(e))?;

    // use std::hash::Hash;
    // use std::hash::Hasher;
    //
    // let mut hasher = Sha256::new();

    Ok("")
}

pub fn service() -> Scope {
    web::scope("/tokens")
        .service(create_token)
        .service(delete_token)
        .service(list_tokens)
}
