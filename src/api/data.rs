use actix_identity::Identity;
use actix_web::{get, web, Scope};

use crate::{db::queue::QueueStatistics, service::Service};

#[get("")]
async fn stats(
    service: web::Data<Service>,
    identity: Identity,
) -> actix_web::Result<web::Json<Vec<QueueStatistics>>> {
    match service.statistics(identity).await {
        Ok(val) => Ok(web::Json(val)),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}

pub fn service() -> Scope {
    web::scope("/stats").service(stats)
}
