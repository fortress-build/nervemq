//! API Key authentication middleware for Actix-web.
//!
//! Provides middleware that authenticates requests using either NerveMQ API keys
//! or AWS SigV4 signatures. Successful authentication creates an Identity session
//! and injects the authorized namespace into request extensions.

use std::future::{Future, Ready};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use actix_identity::Identity;
use actix_web::dev::{Service, Transform};
use actix_web::error::{ErrorInternalServerError, ErrorUnauthorized};
use actix_web::http::header::{self};
use actix_web::web::Data;
use actix_web::HttpMessage;
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error};

use crate::auth::header::AuthHeader;
use crate::auth::protocols::nervemq::authenticate_api_key;
use crate::auth::protocols::sigv4::authenticate_sigv4;

/// Transform factory for API key authentication middleware.
///
/// Used by Actix-web to create the authentication middleware that processes
/// requests with API keys or AWS SigV4 signatures.
pub struct Authentication;

impl<S: 'static, B> Transform<S, ServiceRequest> for Authentication
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> <Self as Transform<S, ServiceRequest>>::Future {
        std::future::ready(Ok(AuthMiddleware {
            service: Arc::new(service),
        }))
    }
}

/// Middleware that performs API key authentication.
///
/// Intercepts requests to:
/// 1. Check for Authorization header
/// 2. Parse and validate API keys or AWS SigV4 signatures
/// 3. Create user session on successful authentication
/// 4. Inject authorized namespace into request extensions
pub struct AuthMiddleware<S> {
    service: Arc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &self,
        cx: &mut Context,
    ) -> Poll<Result<(), <Self as Service<ServiceRequest>>::Error>> {
        self.service.poll_ready(cx)
    }

    /// Processes each request to authenticate API keys.
    ///
    /// If no Authorization header is present, allows the request to pass through
    /// for potential cookie-based authentication later. Otherwise validates the
    /// provided credentials and establishes the user session.
    fn call(&self, mut req: ServiceRequest) -> <Self as Service<ServiceRequest>>::Future {
        let svc = Arc::clone(&self.service);

        Box::pin(async move {
            let api = req
                .app_data::<Data<crate::service::Service>>()
                .expect("SQLite pool not found. This is a bug.")
                .clone();

            let auth_req = {
                let Some(auth_header) = req.headers().get(header::AUTHORIZATION) else {
                    // If there's no auth header, allow the request to pass through.
                    // Authorization will be enforced past this point by the identity system.
                    //
                    // This is necessary for user authentication, since it is checked later based
                    // on cookies.
                    return svc.call(req).await;
                };

                match auth_header.to_str() {
                    Ok(str) => str.to_owned(),
                    Err(e) => return Err(ErrorInternalServerError(e)),
                }
            };

            let auth_header = crate::auth::header::auth_header()
                .parse_str(&auth_req)
                .map_err(|e| ErrorInternalServerError(e))?;

            let (user, authed_namespace) = match auth_header {
                AuthHeader::NerveMqApiV1(token) => {
                    match authenticate_api_key(api.db(), token).await {
                        Ok(user) => user,
                        Err(e) => return Err(ErrorUnauthorized(e)),
                    }
                }
                AuthHeader::AWSv4(header) => {
                    match authenticate_sigv4(api, &mut req, header).await {
                        Ok(user) => user,
                        Err(e) => {
                            tracing::error!("Error authenticating AWSv4: {:?}", e);
                            return Err(ErrorUnauthorized(e));
                        }
                    }
                }
                #[allow(unreachable_patterns)]
                _ => return Err(ErrorUnauthorized("unimplemented")),
            };

            tracing::debug!(email = user.email, "Authenticated user");

            match Identity::login(&req.extensions(), user.email.clone()) {
                Ok(_) => {
                    tracing::debug!("User session established");
                }
                Err(e) => return Err(ErrorUnauthorized(e)),
            }

            req.extensions_mut().insert(authed_namespace);

            svc.call(req).await
        })
    }
}
