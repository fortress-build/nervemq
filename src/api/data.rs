use actix_identity::Identity;
use actix_web::{get, web, Scope};

use crate::{
    db::{namespace::NamespaceStatistics, queue::QueueStatistics},
    service::Service,
};

#[get("/queue")]
async fn queue_stats(
    service: web::Data<Service>,
    identity: Identity,
) -> actix_web::Result<web::Json<Vec<QueueStatistics>>> {
    match service.queue_statistics(identity).await {
        Ok(val) => Ok(web::Json(val)),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}

#[get("/ns")]
async fn namespace_stats(
    service: web::Data<Service>,
    identity: Identity,
) -> actix_web::Result<web::Json<Vec<NamespaceStatistics>>> {
    match service.namespace_statistics(identity).await {
        Ok(val) => Ok(web::Json(val)),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}

pub fn service() -> Scope {
    web::scope("/stats")
        .service(queue_stats)
        .service(namespace_stats)
}
