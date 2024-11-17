use actix_identity::Identity;
use actix_web::{post, HttpMessage, HttpRequest, HttpResponse, Responder};

#[post("/login")]
pub async fn login(request: HttpRequest) -> actix_web::Result<impl Responder> {
    let user_id = "user1".to_owned();

    Identity::login(&request.extensions(), user_id)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok())
}

#[post("/logout")]
pub async fn logout(user: Identity) -> actix_web::Result<impl Responder> {
    user.logout();

    Ok(HttpResponse::Ok())
}
