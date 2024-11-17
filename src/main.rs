use actix_identity::IdentityMiddleware;
use actix_session::config::PersistentSession;
use actix_session::SessionMiddleware;
use actix_web::cookie::time::Duration;
use actix_web::web;
use actix_web::{web::Data, App, HttpServer};

use creek::auth;
use creek::auth::middleware::ApiKeyAuth;
use creek::auth::session::SqliteSessionStore;
use creek::service::Service;
use creek::{api, config::Config};
use tracing_actix_web::TracingLogger;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt::init();

    let config = Config::load()?;

    let service = Service::connect_with(config).await?;

    let session_store = SqliteSessionStore::new(service.db().clone());
    let secret_key = actix_web::cookie::Key::generate();

    let data = Data::new(service);

    // let ssl_acceptor = {
    //     let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;
    //
    //     builder.set_private_key_file("key.pem", SslFiletype::PEM)?;
    //     builder.set_certificate_chain_file("cert.pem")?;
    //
    //     builder
    // };

    // let casbin_middleware = CasbinService::new();

    const SESSION_EXPIRATION: u64 = 24 * 60 * 60;
    let deadline = std::time::Duration::from_secs(SESSION_EXPIRATION);
    let session_ttl = Duration::new(SESSION_EXPIRATION as i64, 0);

    HttpServer::new(move || {
        let identity_middleware = IdentityMiddleware::builder()
            .visit_deadline(Some(deadline))
            .logout_behaviour(actix_identity::config::LogoutBehaviour::PurgeSession)
            .build();

        let session_middleware =
            SessionMiddleware::builder(session_store.clone(), secret_key.clone())
                .session_lifecycle(PersistentSession::default().session_ttl(session_ttl))
                .build();

        App::new()
            .wrap(ApiKeyAuth)
            .wrap(identity_middleware)
            .wrap(session_middleware)
            .wrap(TracingLogger::default())
            // .wrap(actix_web::middleware::Logger::default())
            .app_data(data.clone())
            .configure(creek::api::namespace::config)
            .configure(creek::api::queue::config)
            .configure(creek::api::auth::config)
            .configure(creek::api::data::config)
            .configure(creek::api::admin::config)
    })
    // .bind_openssl(("127.0.0.1", 443), ssl_acceptor)?
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
