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

pub struct ApiKeyAuth;

impl<S: 'static, B> Transform<S, ServiceRequest> for ApiKeyAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ApiKeyAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> <Self as Transform<S, ServiceRequest>>::Future {
        std::future::ready(Ok(ApiKeyAuthMiddleware {
            service: Arc::new(service),
        }))
    }
}

pub struct ApiKeyAuthMiddleware<S> {
    service: Arc<S>,
}

impl<S, B> Service<ServiceRequest> for ApiKeyAuthMiddleware<S>
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

            let user = match auth_header {
                AuthHeader::NerveMqApiV1(token) => {
                    match authenticate_api_key(api.db(), token).await {
                        Ok(user) => user,
                        Err(e) => return Err(ErrorUnauthorized(e)),
                    }
                }
                AuthHeader::AWSv4(header) => match authenticate_sigv4(&mut req, header).await {
                    Ok(user) => user,
                    Err(e) => return Err(ErrorUnauthorized(e)),
                },
                #[allow(unreachable_patterns)]
                _ => return Err(ErrorUnauthorized("unimplemented")),
            };

            match Identity::login(&req.extensions(), user.email.clone()) {
                Ok(_) => {}
                Err(e) => return Err(ErrorUnauthorized(e)),
            }

            svc.call(req).await
        })
    }
}
