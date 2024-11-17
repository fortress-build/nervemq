use actix_identity::Identity;
use actix_web::{
    get, post,
    web::{self, Json, ServiceConfig},
    HttpMessage, HttpRequest, HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};

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
) -> actix_web::Result<impl Responder> {
    let form = form.into_inner();

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
