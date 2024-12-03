use std::collections::HashMap;

use actix_identity::Identity;
use actix_web::{
    delete,
    error::{ErrorInternalServerError, ErrorUnauthorized},
    get, post, web, HttpResponse, Responder, Scope,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::{
    error::Error,
    message::Message,
    queue::Queue,
    service::{QueueConfig, Service},
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

#[derive(Debug, Deserialize)]
struct CreateQueueRequest {
    attributes: HashMap<String, String>,
    tags: HashMap<String, String>,
}

#[post("/{ns_name}/{queue_name}")]
async fn create_queue(
    service: web::Data<Service>,
    path: web::Path<(String, String)>,
    data: web::Json<CreateQueueRequest>,
    identity: Identity,
) -> actix_web::Result<impl Responder> {
    let (namespace, name) = &*path;
    let data = data.into_inner();

    match service
        .create_queue(namespace, name, data.attributes, data.tags, identity)
        .await
    {
        Ok(_) => {}
        Err(Error::Unauthorized) => return Err(ErrorUnauthorized("Unauthorized")),
        Err(e) => return Err(ErrorInternalServerError(e)),
    }

    Ok(actix_web::HttpResponse::Ok())
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

#[get("/{ns_name}/{queue_name}/config")]
async fn get_queue_config(
    service: web::Data<Service>,
    path: web::Path<(String, String)>,
    identity: Identity,
) -> Result<web::Json<QueueConfig>, Error> {
    let (namespace, name) = &*path;

    let ns_id = match service.get_namespace_id(namespace, service.db()).await {
        Ok(Some(id)) => id,
        Ok(None) => return Err(Error::namespace_not_found(namespace)),
        Err(e) => return Err(e),
    };

    service
        .check_user_access(&identity, ns_id, service.db())
        .await?;

    let queue_id = match service.get_queue_id(namespace, name, service.db()).await? {
        Some(id) => id,
        None => return Err(Error::queue_not_found(name, namespace)),
    };

    let config = service.get_queue_configuration(queue_id).await?;

    Ok(web::Json(config))
}

#[derive(Debug, Deserialize)]
struct UpdateQueueConfigRequest {
    max_retries: u64,
    dead_letter_queue: Option<String>,
}

#[post("/{ns_name}/{queue_name}/config")]
async fn update_queue_config(
    service: web::Data<Service>,
    path: web::Path<(String, String)>,
    updates: web::Json<UpdateQueueConfigRequest>,
    identity: Identity,
) -> Result<impl Responder, Error> {
    let (namespace, name) = &*path;

    let ns_id = match service.get_namespace_id(namespace, service.db()).await {
        Ok(Some(id)) => id,
        Ok(None) => return Err(Error::namespace_not_found(namespace)),
        Err(e) => return Err(e),
    };

    service
        .check_user_access(&identity, ns_id, service.db())
        .await?;

    let queue_id = match service.get_queue_id(namespace, name, service.db()).await? {
        Some(id) => id,
        None => return Err(Error::queue_not_found(name, namespace)),
    };

    let dead_letter_queue = match &updates.dead_letter_queue {
        Some(dlq) => match service.get_queue_id(namespace, dlq, service.db()).await? {
            Some(id) => Some(id),
            None => return Err(Error::queue_not_found(dlq, namespace)),
        },
        None => None,
    };

    let new_config = QueueConfig {
        queue: queue_id,
        max_retries: updates.max_retries,
        dead_letter_queue,
    };

    service
        .update_queue_configuration(queue_id, new_config)
        .await?;

    Ok(HttpResponse::Ok())
}

pub fn service() -> Scope {
    web::scope("/queue")
        .service(list_all_queues)
        .service(list_ns_queues)
        .service(create_queue)
        .service(delete_queue)
        .service(queue_stats)
        .service(list_messages)
        .service(get_queue_config)
        .service(update_queue_config)
}
