use std::rc::Rc;

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header::HeaderName,
    HttpMessage,
};

use crate::error::Error;

use super::method::Method;

pub struct SqsApi;

impl<S, B> Transform<S, ServiceRequest> for SqsApi
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;

    type Error = actix_web::Error;

    type Transform = SqsApiMiddleware<S>;

    type InitError = ();

    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(SqsApiMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct SqsApiMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for SqsApiMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future =
        std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        Box::pin(async move {
            let method = req
                .headers()
                .get(HeaderName::from_static("x-amz-target"))
                .ok_or_else(|| Error::InvalidHeader {
                    header: "X-Amz-Target".to_owned(),
                })
                .and_then(|header| header.to_str().map_err(|e| Error::internal(e)))
                .and_then(Method::parse)?;

            req.extensions_mut().insert(method);

            service.call(req).await
        })
    }
}
