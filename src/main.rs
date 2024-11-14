use actix_web::web::{self};
use actix_web::{get, Responder};
use actix_web::{web::Data, App, HttpServer};

use creek::service::Service;
use creek::{api, config::Config};
use serde::ser;
use tracing_actix_web::TracingLogger;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::load()?;

    let service = Data::new(Service::connect_with(config).await?);

    HttpServer::new(move || {
        App::new()
            .service(
                web::scope("/ns")
                    .service(api::list_namespaces)
                    .service(api::create_namespace)
                    .service(api::delete_namespace),
            )
            .service(
                web::scope("/queue")
                    .service(api::list_all_queues)
                    .service(api::list_ns_queues)
                    .service(api::create_queue)
                    .service(api::delete_queue),
            )
            .service(api::stats)
            .wrap(TracingLogger::default())
            .wrap(actix_web::middleware::Logger::default())
            .app_data(service.clone())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
