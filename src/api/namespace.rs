use actix_web::{delete, get, post, web, Responder, Scope};
use serde::{Deserialize, Serialize};

use crate::service::Service;

#[get("")]
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

pub fn service() -> Scope {
    web::scope("/ns")
        .service(list_namespaces)
        .service(create_namespace)
        .service(delete_namespace)
}
