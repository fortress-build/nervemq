use actix_web::{delete, get, post, web, Responder, Scope};
use serde::{Deserialize, Serialize};

use crate::service::Service;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTokenRequest {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTokenResponse {
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteTokenRequest {
    name: String,
}

#[post("")]
pub async fn create_token(
    data: web::Json<CreateTokenRequest>,
    service: web::Data<Service>,
) -> actix_web::Result<impl Responder> {
    Ok("")
}

#[delete("")]
pub async fn delete_token(
    data: web::Json<DeleteTokenRequest>,
    service: web::Data<Service>,
) -> actix_web::Result<impl Responder> {
    Ok("")
}

#[get("")]
pub async fn list_tokens(service: web::Data<Service>) -> actix_web::Result<impl Responder> {
    Ok("")
}

pub fn service() -> Scope {
    web::scope("/tokens")
        .service(create_token)
        .service(delete_token)
        .service(list_tokens)
}
