use actix_web::{web::Data, App, HttpServer};

use creek::service::Service;
use creek::{api, config::Config};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let config = Config::load()?;

    let service = Data::new(Service::connect_with(config).await?);

    HttpServer::new(move || {
        App::new()
            .app_data(service.clone())
            .service(api::list_namespaces)
            .service(api::create_namespace)
            .service(api::delete_namespace)
            .service(api::list_all_queues)
            .service(api::list_ns_queues)
            .service(api::create_queue)
            .service(api::delete_queue)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
