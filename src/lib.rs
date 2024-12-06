use std::future::Future;

use actix_cors::Cors;
use actix_identity::IdentityMiddleware;
use actix_session::{
    config::{CookieContentSecurity, PersistentSession},
    SessionMiddleware,
};
use actix_web::{
    middleware::{NormalizePath, TrailingSlash},
    web::{Data, FormConfig, JsonConfig},
    App, HttpServer,
};
use auth::{
    middleware::{api_keys::ApiKeyAuth, protected_route::Protected},
    session::SqliteSessionStore,
};
use chrono::TimeDelta;
use config::Config;
use error::Error;
use kms::KeyManager;
use sqlx::SqlitePool;
use sqs::service::SqsApi;
use tracing::level_filters::LevelFilter;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{util::SubscriberInitExt, EnvFilter, FmtSubscriber};

pub mod api;
pub mod auth;
pub mod config;
pub mod error;
pub mod kms;
pub mod message;
pub mod namespace;
pub mod queue;
pub mod service;
pub mod sqs;
pub mod utils;

/// Returns a builder for the main application.
#[bon::builder(finish_fn = start)]
pub async fn run<K, F, R>(kms_factory: K) -> eyre::Result<()>
where
    K: FnOnce(SqlitePool) -> F,
    F: Future<Output = Result<R, Error>>,
    R: KeyManager,
{
    #[cfg(debug_assertions)]
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

    #[cfg(not(debug_assertions))]
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

    let config = Config::load()?;

    let service = service::Service::connect_with()
        .config(config)
        .kms_factory(kms_factory)
        .call()
        .await?;

    let session_store = SqliteSessionStore::new(service.db().clone());

    // FIXME: This should be generated on first run and stored in a file, or pulled from config
    let secret_key = actix_web::cookie::Key::generate();

    let data = Data::new(service);

    const SESSION_EXPIRATION: TimeDelta = chrono::Duration::hours(1);

    let deadline = SESSION_EXPIRATION.to_std().expect("valid duration");
    let session_ttl = actix_web::cookie::time::Duration::new(SESSION_EXPIRATION.num_seconds(), 0);

    HttpServer::new(move || {
        let session_middleware =
            SessionMiddleware::builder(session_store.clone(), secret_key.clone())
                .cookie_secure(true)
                .cookie_content_security(CookieContentSecurity::Signed)
                .session_lifecycle(PersistentSession::default().session_ttl(session_ttl))
                .cookie_http_only(true)
                .cookie_name("nervemq_session".to_owned())
                .build();

        let identity_middleware = IdentityMiddleware::builder()
            .visit_deadline(Some(deadline))
            .logout_behaviour(actix_identity::config::LogoutBehaviour::PurgeSession)
            .id_key("nervemq_id")
            .build();

        let cors = Cors::default()
            .supports_credentials()
            .allow_any_origin()
            .allow_any_header()
            .allow_any_method();

        let json_cfg = JsonConfig::default().content_type_required(false);
        let form_cfg = FormConfig::default();

        App::new()
            .wrap(TracingLogger::default())
            .wrap(ApiKeyAuth)
            .wrap(identity_middleware)
            .wrap(session_middleware)
            .wrap(NormalizePath::new(TrailingSlash::Trim))
            .wrap(cors)
            .service(api::queue::service().wrap(Protected::authenticated()))
            .service(api::data::service().wrap(Protected::authenticated()))
            .service(api::tokens::service().wrap(Protected::authenticated()))
            .service(sqs::service().wrap(Protected::authenticated()).wrap(SqsApi))
            .service(api::namespace::service().wrap(Protected::admin_only()))
            .service(api::admin::service().wrap(Protected::admin_only()))
            .service(api::auth::service())
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
