use actix_identity::Identity;
use actix_web::{
    delete,
    error::{ErrorInternalServerError, ErrorUnauthorized},
    get, post, web, Responder, Scope,
};
use serde::{Deserialize, Serialize};

use crate::{
    db::queue::Queue,
    service::{Error, Service},
};

#[derive(Serialize, Deserialize)]
pub struct ListQueuesResponse {
    queues: Vec<Queue>,
}

#[get("")]
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
    identity: Identity,
) -> actix_web::Result<impl Responder> {
    let (namespace, name) = &*path;

    match service.create_queue(namespace, name, identity).await {
        Ok(_) => {}
        Err(Error::Unauthorized) => return Err(ErrorUnauthorized("Unauthorized")),
        Err(e) => return Err(ErrorInternalServerError(e)),
    }

    Ok("OK")
}

pub fn service() -> Scope {
    web::scope("/queue")
        .service(list_all_queues)
        .service(list_ns_queues)
        .service(create_queue)
        .service(delete_queue)
}
