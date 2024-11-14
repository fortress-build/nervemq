use actix_web::{delete, get, post, web, Responder};
use serde::{Deserialize, Serialize};

use crate::{
    db::queue::{Queue, QueueStatistics},
    service::Service,
};

#[get("/")]
async fn list_namespaces(service: web::Data<Service>) -> actix_web::Result<impl Responder> {
    let data = match service.list_namespaces().await {
        Ok(data) => data,
        Err(e) => return Err(actix_web::error::ErrorInternalServerError(e)),
    };

    Ok(web::Json(data))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateNamespaceResponse {
    id: u64,
}

#[post("/{ns_name}")]
async fn create_namespace(
    service: web::Data<Service>,
    path: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    let id = match service.create_namespace(&*path).await {
        Ok(id) => id,
        Err(e) => return Err(actix_web::error::ErrorInternalServerError(e)),
    };

    Ok(web::Json(CreateNamespaceResponse { id }))
}

#[delete("/{ns_name}")]
async fn delete_namespace(
    service: web::Data<Service>,
    path: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    if let Err(e) = service.delete_namespace(&*path).await {
        return Err(actix_web::error::ErrorInternalServerError(e));
    }

    Ok("OK")
}

#[derive(Serialize, Deserialize)]
pub struct ListQueuesResponse {
    queues: Vec<Queue>,
}

#[get("/")]
async fn list_all_queues(service: web::Data<Service>) -> actix_web::Result<impl Responder> {
    let queues = match service.list_queues(None).await {
        Ok(q) => q,
        Err(e) => return Err(actix_web::error::ErrorInternalServerError(e)),
    };

    Ok(web::Json(ListQueuesResponse { queues }))
}

#[get("/{ns_name}")]
async fn list_ns_queues(
    service: web::Data<Service>,
    path: web::Path<String>,
) -> actix_web::Result<impl Responder> {
    let queues = match service.list_queues(Some(&*path)).await {
        Ok(q) => q,
        Err(e) => return Err(actix_web::error::ErrorInternalServerError(e)),
    };

    Ok(web::Json(ListQueuesResponse { queues }))
}

#[delete("/{ns_name}/{queue_name}")]
async fn delete_queue(
    service: web::Data<Service>,
    path: web::Path<(String, String)>,
) -> actix_web::Result<impl Responder> {
    let (namespace, name) = &*path;
    if let Err(e) = service.delete_queue(namespace, name).await {
        return Err(actix_web::error::ErrorInternalServerError(e));
    }

    Ok("OK")
}

#[post("/{ns_name}/{queue_name}")]
async fn create_queue(
    service: web::Data<Service>,
    path: web::Path<(String, String)>,
) -> actix_web::Result<impl Responder> {
    let (namespace, name) = &*path;
    if let Err(e) = service.create_queue(namespace, name).await {
        return Err(actix_web::error::ErrorInternalServerError(e));
    }

    Ok("OK")
}

#[get("/stats")]
async fn stats(service: web::Data<Service>) -> actix_web::Result<web::Json<Vec<QueueStatistics>>> {
    match service.statistics().await {
        Ok(val) => Ok(web::Json(val)),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}