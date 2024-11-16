use actix_identity::IdentityMiddleware;
use actix_session::config::PersistentSession;
use actix_session::SessionMiddleware;
use actix_web::cookie::time::Duration;
use actix_web::web::{self};
use actix_web::{web::Data, App, HttpServer};

use creek::auth;
use creek::auth::middleware::ApiKeyAuth;
use creek::auth::session::SqliteSessionStore;
use creek::service::Service;
use creek::{api, config::Config};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use tracing_actix_web::TracingLogger;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::load()?;

    let service = Service::connect_with(config).await?;

    let session_store = SqliteSessionStore::new(service.db().clone());
    let secret_key = actix_web::cookie::Key::generate();

    let data = Data::new(service);

    let ssl_acceptor = {
        let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;

        builder.set_private_key_file("key.pem", SslFiletype::PEM)?;
        builder.set_certificate_chain_file("cert.pem")?;

        builder
    };

    // let casbin_middleware = CasbinService::new();

    const SESSION_EXPIRATION: u64 = 24 * 60 * 60;
    let deadline = std::time::Duration::from_secs(SESSION_EXPIRATION);
    let session_ttl = Duration::new(SESSION_EXPIRATION as i64, 0);

    HttpServer::new(move || {
        App::new()
            .wrap(ApiKeyAuth)
            .wrap(
                IdentityMiddleware::builder()
                    .visit_deadline(Some(deadline))
                    .build(),
            )
            .wrap(
                SessionMiddleware::builder(session_store.clone(), secret_key.clone())
                    .session_lifecycle(PersistentSession::default().session_ttl(session_ttl))
                    .build(),
            )
            .wrap(TracingLogger::default())
            // .wrap(actix_web::middleware::Logger::default())
            .app_data(data.clone())
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
            .service(
                web::scope("/auth")
                    .service(auth::api::login)
                    .service(auth::api::logout),
            )
    })
    .bind_openssl(("127.0.0.1", 443), ssl_acceptor)?
    // .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
