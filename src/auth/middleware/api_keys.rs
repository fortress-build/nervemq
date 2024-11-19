use std::future::{Future, Ready};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use actix_web::dev::{Service, Transform};
use actix_web::error::ErrorUnauthorized;
use actix_web::http::header::{self};
use actix_web::web::Data;
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error, HttpMessage};

use actix_casbin_auth::CasbinVals;

use crate::auth::data::authenticate_api_key;
use crate::auth::header::AuthHeader;

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

    fn call(&self, req: ServiceRequest) -> <Self as Service<ServiceRequest>>::Future {
        let svc = Arc::clone(&self.service);

        Box::pin(async move {
            let api = req
                .app_data::<Data<crate::service::Service>>()
                .expect("SQLite pool not found. This is a bug.")
                .clone();

            let Some(auth_header) = req.headers().get(header::AUTHORIZATION) else {
                return Err(ErrorUnauthorized("API Key not provided"));
            };

            let auth_req = match auth_header.to_str() {
                Ok(str) => str,
                Err(_) => return Err(ErrorUnauthorized("Invalid authorization")),
            };

            match {
                let parser = crate::auth::header::auth_header();

                parser
                    .parse_str(auth_req)
                    .map_err(|_| ErrorUnauthorized("Invalid authorization"))?
            } {
                AuthHeader::NerveMqApiV1 { token } => {
                    match authenticate_api_key(api.db(), token).await {
                        Ok(_) => {
                            let vals = CasbinVals {
                                subject: String::from("alice"),
                                domain: None,
                            };
                            req.extensions_mut().insert(vals);
                            svc.call(req).await
                        }
                        Err(e) => Err(ErrorUnauthorized(e)),
                    }
                }
                AuthHeader::Bearer { .. } => Err(ErrorUnauthorized("unimplemented")),
            }
        })
    }
}
