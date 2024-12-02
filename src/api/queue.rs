use actix_identity::Identity;
use actix_web::{
    delete,
    error::{ErrorInternalServerError, ErrorUnauthorized},
    get, post, web, Responder, Scope,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::{
    message::Message,
    queue::Queue,
    service::{Error, Service},
};

#[derive(Serialize, Deserialize)]
pub struct ListQueuesResponse {
    queues: Vec<Queue>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct QueueStats {
    pub pending: u64,
    pub delivered: u64,
    pub failed: u64,
}

#[get("")]
async fn list_all_queues(
    service: web::Data<Service>,
    identity: Identity,
) -> actix_web::Result<impl Responder> {
    let queues = match service.list_all_queues(identity).await {
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
    let queues = match service.list_queues_for_namespace(&*path).await {
        Ok(q) => q,
        Err(e) => return Err(actix_web::error::ErrorInternalServerError(e)),
    };

    Ok(web::Json(ListQueuesResponse { queues }))
}

#[delete("/{ns_name}/{queue_name}")]
async fn delete_queue(
    service: web::Data<Service>,
    path: web::Path<(String, String)>,
    identity: Identity,
) -> actix_web::Result<impl Responder> {
    let (namespace, name) = &*path;
    if let Err(e) = service.delete_queue(namespace, name, identity).await {
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

#[get("/{ns_name}/{queue_name}")]
async fn queue_stats(
    service: web::Data<Service>,
    path: web::Path<(String, String)>,
    identity: Identity,
) -> actix_web::Result<impl Responder> {
    let (namespace, name) = &*path;

    match service.queue_statistics(identity, namespace, name).await {
        Ok(stats) => Ok(web::Json(stats)),
        Err(Error::Unauthorized) => Err(ErrorUnauthorized("Unauthorized")),
        Err(e) => Err(ErrorInternalServerError(e)),
    }
}

#[get("/{ns_name}/{queue_name}/messages")]
async fn list_messages(
    service: web::Data<Service>,
    path: web::Path<(String, String)>,
    identity: Identity,
) -> actix_web::Result<web::Json<Vec<Message>>> {
    let (namespace, name) = &*path;

    let ns_id = match service.get_namespace_id(namespace, service.db()).await {
        Ok(Some(id)) => id,
        Ok(None) => return Err(ErrorInternalServerError("Namespace not found")),
        Err(e) => return Err(ErrorInternalServerError(e)),
    };

    match service
        .check_user_access(&identity, ns_id, service.db())
        .await
    {
        Ok(_) => {}
        Err(e) => return Err(ErrorUnauthorized(e)),
    }

    match service.list_messages(namespace, name).await {
        Ok(messages) => Ok(web::Json(messages)),
        Err(e) => Err(ErrorInternalServerError(e)),
    }
}

pub fn service() -> Scope {
    web::scope("/queue")
        .service(list_all_queues)
        .service(list_ns_queues)
        .service(create_queue)
        .service(delete_queue)
        .service(queue_stats)
        .service(list_messages)
}
