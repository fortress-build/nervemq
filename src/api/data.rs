use std::collections::HashMap;

use actix_identity::Identity;
use actix_web::{get, web, Scope};

use crate::{
    db::{namespace::NamespaceStatistics, queue::QueueStatistics},
    service::Service,
};

// #[get("/queue/{queue_id}")]
// async fn queue_stats(
//     service: web::Data<Service>,
//     identity: Identity,
//     path: web::Path<String>,
// ) -> actix_web::Result<web::Json<Vec<QueueStatistics>>> {
//     let queue_name = path.into_inner();
//
//     match service.queue_statistics(identity, queue_name).await {
//         Ok(val) => Ok(web::Json(val)),
//         Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
//     }
// }

#[get("/queue")]
async fn queue_stats(
    service: web::Data<Service>,
    identity: Identity,
) -> actix_web::Result<web::Json<HashMap<String, QueueStatistics>>> {
    match service.list_queue_statistics(identity).await {
        Ok(val) => Ok(web::Json(val)),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}

#[get("/ns")]
async fn namespace_stats(
    service: web::Data<Service>,
    identity: Identity,
) -> actix_web::Result<web::Json<Vec<NamespaceStatistics>>> {
    match service.list_namespace_statistics(identity).await {
        Ok(val) => Ok(web::Json(val)),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}

pub fn service() -> Scope {
    web::scope("/stats")
        .service(queue_stats)
        .service(namespace_stats)
}
