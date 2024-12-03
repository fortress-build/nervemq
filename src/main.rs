use actix_cors::Cors;
use actix_identity::IdentityMiddleware;
use actix_session::config::{CookieContentSecurity, PersistentSession};
use actix_session::SessionMiddleware;
use actix_web::cookie::time::Duration;
use actix_web::middleware::{NormalizePath, TrailingSlash};
use actix_web::web::{FormConfig, JsonConfig};
use actix_web::{web::Data, App, HttpServer};

use chrono::TimeDelta;
use nervemq::api::auth::{self};
use nervemq::api::{admin, data, namespace, queue, tokens};
use nervemq::auth::middleware::api_keys::ApiKeyAuth;
use nervemq::auth::middleware::protected_route::Protected;
use nervemq::auth::session::SqliteSessionStore;
use nervemq::config::Config;
use nervemq::service::Service;
use tracing::level_filters::LevelFilter;
// use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use tracing_actix_web::TracingLogger;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    if cfg!(debug_assertions) {
        FmtSubscriber::builder()
            .pretty()
            .with_env_filter(
                EnvFilter::builder()
                    .with_env_var("NERVEMQ_LOG")
                    .with_default_directive(LevelFilter::INFO.into())
                    .from_env()?,
            )
            .finish()
            .try_init()?;
    } else {
        FmtSubscriber::builder()
            .json()
            .with_env_filter(
                EnvFilter::builder()
                    .with_env_var("NERVEMQ_LOG")
                    .with_default_directive(LevelFilter::INFO.into())
                    .from_env()?,
            )
            .finish()
            .try_init()?;
    }

    let config = Config::load()?;

    let service = Service::connect_with(config).await?;

    let session_store = SqliteSessionStore::new(service.db().clone());
    let secret_key = actix_web::cookie::Key::generate();

    let data = Data::new(service);

    // let ssl_acceptor = {
    //     let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;
    //
    //     builder.set_private_key_file("localhost-key.pem", SslFiletype::PEM)?;
    //     builder.set_certificate_chain_file("localhost.pem")?;
    //
    //     builder
    // };

    const SESSION_EXPIRATION: TimeDelta = chrono::Duration::hours(1);

    let deadline = SESSION_EXPIRATION.to_std().expect("valid duration");
    let session_ttl = Duration::new(SESSION_EXPIRATION.num_seconds(), 0);

    HttpServer::new(move || {
        let session_middleware =
            SessionMiddleware::builder(session_store.clone(), secret_key.clone())
                .cookie_secure(true)
                .cookie_content_security(CookieContentSecurity::Signed)
                .session_lifecycle(PersistentSession::default().session_ttl(session_ttl))
                .cookie_domain(Some("localhost".to_owned()))
                .cookie_path("/".to_owned())
                .cookie_http_only(true)
                .cookie_name("nervemq_session".to_owned())
                .build();

        let identity_middleware = IdentityMiddleware::builder()
            .visit_deadline(Some(deadline))
            .logout_behaviour(actix_identity::config::LogoutBehaviour::PurgeSession)
            .id_key("nervemq_id")
            .build();

        let cors = Cors::default()
            // .send_wildcard()
            .supports_credentials()
            .allow_any_origin()
            .allow_any_header()
            .allow_any_method();

        let json_cfg = JsonConfig::default().content_type_required(false);
        let form_cfg = FormConfig::default();

        App::new()
            .wrap(identity_middleware)
            .wrap(session_middleware)
            .wrap(ApiKeyAuth)
            .wrap(cors)
            .wrap(TracingLogger::default())
            .wrap(NormalizePath::new(TrailingSlash::Trim))
            .service(queue::service().wrap(Protected::authenticated()))
            .service(data::service().wrap(Protected::authenticated()))
            .service(tokens::service().wrap(Protected::authenticated()))
            .service(namespace::service().wrap(Protected::admin_only()))
            .service(admin::service().wrap(Protected::admin_only()))
            .service(auth::service())
            .app_data(data.clone())
            .app_data(json_cfg)
            .app_data(form_cfg)
    })
    // .bind_openssl(("127.0.0.1", 8080), ssl_acceptor)?
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
