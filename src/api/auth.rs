use actix_identity::Identity;
use actix_web::{
    post,
    web::{self, Json, ServiceConfig},
    HttpMessage, HttpRequest, HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    email: String,
    password: zeroize::Zeroizing<String>,
}

#[post("/login")]
pub async fn login(
    request: HttpRequest,
    form: Json<LoginRequest>,
) -> actix_web::Result<impl Responder> {
    let form = form.into_inner();

    Identity::login(&request.extensions(), form.email)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok())
}

#[post("/logout")]
pub async fn logout(user: Identity) -> actix_web::Result<impl Responder> {
    user.logout();

    Ok(HttpResponse::Ok())
}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(web::scope("/auth").service(login).service(logout));
}