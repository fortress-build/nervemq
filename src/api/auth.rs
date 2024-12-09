use actix_identity::Identity;
use actix_session::SessionExt;
use actix_web::{post, web, HttpMessage, HttpRequest, HttpResponse, Responder, Scope};
use argon2::{password_hash::PasswordHashString, Argon2, PasswordVerifier};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::{error::Error, service::Service};

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionResponse {
    email: String,
    role: Role,
}

#[derive(
    Debug, Clone, Serialize, Deserialize, Default, sqlx::Type, PartialEq, Eq, PartialOrd, Ord,
)]
#[sqlx(type_name = "text")]
pub enum Role {
    #[default]
    #[serde(rename = "user")]
    #[sqlx(rename = "user")]
    User = 0,
    #[serde(rename = "admin")]
    #[sqlx(rename = "admin")]
    Admin = 1,
}

#[derive(Debug, Clone, Deserialize, FromRow)]
pub struct Permission {
    #[allow(unused)]
    pub id: u64,
    pub user: u64,
    #[allow(unused)]
    pub namespace: u64,
    pub can_delete_ns: bool,
}

#[derive(Deserialize, FromRow)]
struct LoginData {
    hashed_pass: String,
    role: Role,
}

#[post("/login")]
pub async fn login(
    request: HttpRequest,
    form: web::Json<LoginRequest>,
    service: web::Data<Service>,
) -> Result<web::Json<SessionResponse>, Error> {
    let form = form.into_inner();

    let Ok(Some(user_data)) =
        sqlx::query_as::<_, LoginData>("SELECT hashed_pass, role FROM users WHERE email = $1")
            .bind(&form.email)
            .fetch_optional(service.db())
            .await
    else {
        return Err(Error::UserNotFound { email: form.email });
    };

    match tokio::task::spawn_blocking(move || {
        let pass_hash = PasswordHashString::new(&user_data.hashed_pass)?;

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
            return Err(Error::InternalServerError {
                source: Some(eyre::eyre!(e)),
            });
        }
        Ok(Ok(_)) => {}
    };

    let session = request.get_session();

    match Identity::login(&request.extensions(), form.email.clone()) {
        Ok(id) => {
            session
                .insert::<String>("nervemq_id", id.id().expect("identifier").to_string())
                .ok();
        }
        Err(e) => {
            tracing::error!("Failed to login: {e}");
            return Err(Error::InternalServerError {
                source: Some(eyre::eyre!(e)),
            });
        }
    }

    Ok(web::Json(SessionResponse {
        email: form.email,
        role: user_data.role,
    }))
}

#[post("/logout")]
pub async fn logout(user: Identity) -> actix_web::Result<impl Responder> {
    user.logout();

    Ok(HttpResponse::Ok())
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: u64,
    pub email: String,
    pub role: Role,
}

#[post("/verify")]
pub async fn verify(
    identity: Option<Identity>,
    service: web::Data<Service>,
) -> Result<web::Json<SessionResponse>, Error> {
    match identity {
        Some(identity) => {
            let email = identity.id().map_err(Error::internal)?;

            let User { email, role, .. } = sqlx::query_as("SELECT * FROM users WHERE email = $1")
                .bind(&email)
                .fetch_optional(service.db())
                .await
                .map_err(Error::internal)?
                .ok_or_else(|| Error::Unauthorized)?;

            Ok(web::Json(SessionResponse { email, role }))
        }
        None => Err(Error::Unauthorized),
    }
}

pub fn service() -> Scope {
    web::scope("/auth")
        .service(login)
        .service(logout)
        .service(verify)
}
