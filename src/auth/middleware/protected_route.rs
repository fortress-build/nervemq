use std::future::{Future, Ready};
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

use actix_identity::IdentityExt;
use actix_web::dev::{Service, Transform};
use actix_web::error::ErrorUnauthorized;
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error};

pub struct Protected;

impl<S: 'static, B> Transform<S, ServiceRequest> for Protected
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ProtectedRouteMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> <Self as Transform<S, ServiceRequest>>::Future {
        std::future::ready(Ok(ProtectedRouteMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct ProtectedRouteMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for ProtectedRouteMiddleware<S>
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
        let svc = Rc::clone(&self.service);

        Box::pin(async move {
            match req.get_identity() {
                Ok(_) => {
                    // TODO: RBAC
                    svc.call(req).await
                }
                Err(e) => Err(ErrorUnauthorized(e)),
            }
        })
    }
}
