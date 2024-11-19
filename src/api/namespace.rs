use actix_web::{web, Responder, Scope};
use serde::{Deserialize, Serialize};

use crate::service::Service;

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
        .route("", web::get().to(list_namespaces))
        .service(
            web::resource("/{ns_name}")
                .post(create_namespace)
                .delete(delete_namespace),
        )
}
