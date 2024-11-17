use actix_web::{get, web};

use crate::{db::queue::QueueStatistics, service::Service};

#[get("/stats")]
async fn stats(service: web::Data<Service>) -> actix_web::Result<web::Json<Vec<QueueStatistics>>> {
    match service.statistics().await {
        Ok(val) => Ok(web::Json(val)),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(stats);
}
