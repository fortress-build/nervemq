use actix_identity::Identity;
use actix_web::{
    error::{ErrorInternalServerError, ErrorUnauthorized},
    get, post,
    web::{self, Json, ServiceConfig},
    HttpMessage, HttpRequest, HttpResponse, Responder,
};
use argon2::{password_hash::PasswordHashString, Argon2, PasswordVerifier};
use serde::{Deserialize, Serialize};

use crate::service::Service;

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    email: String,
    password: zeroize::Zeroizing<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum SessionResponse {
    Valid { email: String },
    Invalid,
}

#[post("/login")]
pub async fn login(
    request: HttpRequest,
    form: Json<LoginRequest>,
    service: web::Data<Service>,
) -> actix_web::Result<impl Responder> {
    let form = form.into_inner();

    let Some(hashed_key) =
        sqlx::query_scalar::<_, String>("SELECT hashed_pass FROM users WHERE email = $1")
            .bind(&form.email)
            .fetch_optional(service.db())
            .await
            .map_err(|e| ErrorInternalServerError(e))?
    else {
        return Err(ErrorUnauthorized("identity not found"));
    };

    match web::block(move || {
        let pass_hash = PasswordHashString::new(&hashed_key)?;

        Argon2::default().verify_password(form.password.as_bytes(), &pass_hash.password_hash())
    })
    .await
    {
        Ok(Err(e)) => return Err(ErrorUnauthorized(e)),
        Err(e) => {
            return Err(ErrorInternalServerError(eyre::eyre!(
                "Failed to join API key verify task: {e}"
            )))
        }
        Ok(Ok(_)) => {}
    };

    Identity::login(&request.extensions(), form.email.clone())
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(web::Json(SessionResponse::Valid { email: form.email }))
}

#[post("/logout")]
pub async fn logout(user: Identity) -> actix_web::Result<impl Responder> {
    user.logout();

    Ok(HttpResponse::Ok())
}

#[get("/session")]
pub async fn get_session(identity: Option<Identity>) -> actix_web::Result<impl Responder> {
    match identity {
        Some(identity) => {
            let email = identity
                .id()
                .map_err(actix_web::error::ErrorInternalServerError)?;
            Ok(web::Json(SessionResponse::Valid { email }))
        }
        None => Ok(web::Json(SessionResponse::Invalid)),
    }
}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .service(login)
            .service(logout)
            .service(get_session),
    );
}
