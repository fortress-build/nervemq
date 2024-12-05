use actix_cors::Cors;
use actix_identity::IdentityMiddleware;
use actix_session::{
    config::{CookieContentSecurity, PersistentSession},
    SessionMiddleware,
};
use actix_web::{
    cookie::time::Duration,
    middleware::{NormalizePath, TrailingSlash},
    web::{Data, FormConfig, JsonConfig},
    App, HttpServer,
};

use chrono::TimeDelta;
use nervemq::{
    api::{
        admin,
        auth::{self},
        data, namespace, queue, tokens,
    },
    auth::{
        kms::memory::InMemoryKeyManager,
        middleware::{api_keys::ApiKeyAuth, protected_route::Protected},
        session::SqliteSessionStore,
    },
    config::Config,
    service::Service,
    sqs::{self, service::SqsApi},
};
use tracing::level_filters::LevelFilter;
// use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{util::SubscriberInitExt, EnvFilter, FmtSubscriber};

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

    let kms = InMemoryKeyManager::new();

    let service = Service::connect_with(kms, config).await?;

    let session_store = SqliteSessionStore::new(service.db().clone());

    // FIXME: This should be generated on first run and stored in a file, or pulled from config
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
            .service(queue::service().wrap(Protected::authenticated()))
            .service(data::service().wrap(Protected::authenticated()))
            .service(tokens::service().wrap(Protected::authenticated()))
            .service(sqs::service().wrap(Protected::authenticated()).wrap(SqsApi))
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
