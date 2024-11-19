use actix_identity::Identity;
use actix_web::{get, web, Scope};

use crate::{db::queue::QueueStatistics, service::Service};

#[get("")]
async fn stats(
    identity: Identity,
    service: web::Data<Service>,
) -> actix_web::Result<web::Json<Vec<QueueStatistics>>> {
    match service.statistics().await {
        Ok(val) => Ok(web::Json(val)),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}

pub fn service() -> Scope {
    web::scope("/stats").service(stats)
}
