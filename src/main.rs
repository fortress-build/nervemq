use actix_cors::Cors;
use actix_identity::IdentityMiddleware;
use actix_session::config::{CookieContentSecurity, PersistentSession};
use actix_session::SessionMiddleware;
use actix_web::cookie::time::Duration;
use actix_web::web::{FormConfig, Html, JsonConfig};
use actix_web::{web::Data, App, HttpServer};

use chrono::TimeDelta;
use creek::auth::session::SqliteSessionStore;
use creek::config::Config;
use creek::service::Service;
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

        builder.set_private_key_file("localhost-key.pem", SslFiletype::PEM)?;
        builder.set_certificate_chain_file("localhost.pem")?;

        builder
    };

    const SESSION_EXPIRATION: TimeDelta = chrono::Duration::hours(1);

    let deadline = SESSION_EXPIRATION.to_std().expect("valid duration");
    let session_ttl = Duration::new(SESSION_EXPIRATION.num_seconds(), 0);

    HttpServer::new(move || {
        let session_middleware =
            SessionMiddleware::builder(session_store.clone(), secret_key.clone())
                .cookie_secure(false)
                .cookie_content_security(CookieContentSecurity::Signed)
                .session_lifecycle(PersistentSession::default().session_ttl(session_ttl))
                .cookie_domain(Some("localhost".to_owned()))
                .cookie_path("/".to_owned())
                .cookie_http_only(true)
                .cookie_name("creek-session".to_owned())
                .build();

        let identity_middleware = IdentityMiddleware::builder()
            .visit_deadline(Some(deadline))
            .logout_behaviour(actix_identity::config::LogoutBehaviour::PurgeSession)
            .id_key("creek_user_id")
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
            // .wrap(ApiKeyAuth)
            // .wrap(actix_web::middleware::Logger::default())
            .wrap(identity_middleware)
            .wrap(session_middleware)
            .wrap(cors)
            .wrap(TracingLogger::default())
            .service(creek::api::namespace::service())
            .service(creek::api::queue::service())
            .service(creek::api::data::service())
            .service(creek::api::admin::service())
            .service(creek::api::tokens::service())
            .service(creek::api::auth::service())
            .app_data(json_cfg)
            .app_data(data.clone())
            .app_data(form_cfg)
    })
    // .bind_openssl(("127.0.0.1", 8080), ssl_acceptor)?
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}

#[actix_web::get("/")]
async fn index(user: Option<actix_identity::Identity>) -> Html {
    if let Some(user) = user {
        let id = user.id().unwrap();

        Html::new(format!(
            r#"
            <html>
                <body>
                    <h1>Welcome! {id}</h1>

                    <form action="https://localhost:8081/logout" method="post" target="_blank">
                         <input type="submit" value="Submit" />
                    </form>
                </body>
            </html>
            "#,
        ))
    } else {
        Html::new(format!(
            r#"
            <html>
                <body>
                    <script>
                        async function onSubmit() {{
                            const data = {{
                                email: document.getElementById("emailEntry").value,
                                password: document.getElementById("passwordEntry").value
                            }};
                            const res = await fetch("http://localhost:8080/auth/login", {{
                                method: "POST",
                                body: JSON.stringify(data),
                                headers: {{
                                    "Content-Type": "application/json"
                                }}
                            }})
                            console.log(res);
                        }}
                    </script>
                    <h1>Welcome Anonymous!</h1>

                    <form>
                         <input type="text" id="emailEntry" name="email" value="e@e.e" /><br />
                         <input type="text" id="passwordEntry" name="password" value="eeeeeeee" /><br />
                         <input type="button" value="Submit" onclick="onSubmit()" />
                    </form>
                </body>
            </html>
            "#,
        ))
    }
}
