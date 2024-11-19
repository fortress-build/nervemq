use actix_identity::Identity;
use actix_session::SessionExt;
use actix_web::{
    error::ErrorUnauthorized, get, http::StatusCode, post, web, HttpMessage, HttpRequest,
    HttpResponse, Responder, ResponseError, Scope,
};
use argon2::{password_hash::PasswordHashString, Argon2, PasswordVerifier};
use serde::{Deserialize, Serialize};
use snafu::Snafu;

use crate::service::Service;

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionResponse {
    email: String,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Identity not found"))]
    IdentityNotFound,
    #[snafu(display("Unauthorized"))]
    Unauthorized,
    #[snafu(display("Internal server error"))]
    InternalError { source: eyre::Report },
}

impl ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Error::IdentityNotFound => StatusCode::UNAUTHORIZED,
            Error::Unauthorized => StatusCode::UNAUTHORIZED,
            Error::InternalError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub async fn login(
    request: HttpRequest,
    form: web::Json<LoginRequest>,
    service: web::Data<Service>,
) -> Result<impl Responder, Error> {
    let form = form.into_inner();

    let Ok(Some(hashed_key)) =
        sqlx::query_scalar::<_, String>("SELECT hashed_pass FROM users WHERE email = $1")
            .bind(&form.email)
            .fetch_optional(service.db())
            .await
    else {
        return Err(Error::IdentityNotFound);
    };

    match web::block(move || {
        let pass_hash = PasswordHashString::new(&hashed_key)?;

        Argon2::default().verify_password(form.password.as_bytes(), &pass_hash.password_hash())
    })
    .await
    {
        Ok(Err(e)) => {
            tracing::error!("{e}");
            return Err(Error::Unauthorized);
        }
        Err(e) => {
            tracing::error!("{e}");
            return Err(Error::InternalError {
                source: eyre::eyre!(e),
            });
        }
        Ok(Ok(_)) => {}
    };

    let session = request.get_session();

    match Identity::login(&request.extensions(), form.email.clone()) {
        Ok(id) => {
            session
                .insert::<String>("creek_user_id", id.id().expect("identifier").to_string())
                .ok();
        }
        Err(e) => {
            tracing::error!("Failed to login: {e}");
            return Err(Error::InternalError {
                source: eyre::eyre!(e),
            });
        }
    }

    Ok(HttpResponse::Ok().json(SessionResponse { email: form.email }))
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
            Ok(web::Json(SessionResponse { email }))
        }
        None => Err(ErrorUnauthorized("Unauthorized")),
    }
}

pub fn service() -> Scope {
    web::scope("/auth")
        .service(web::resource("/login").post(login))
        .service(logout)
        .service(get_session)
}
