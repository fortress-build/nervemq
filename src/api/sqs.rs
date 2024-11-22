use std::rc::Rc;

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    HttpMessage,
};
use pom::utf8::{seq, sym};

const SQS_METHOD_PREFIX: &str = "AmazonSQS";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    SendMessage,
    SendMessageBatch,
    ReceiveMessage,
    DeleteMessage,
    ListQueues,
    GetQueueUrl,
    CreateQueue,
    GetQueueAttributes,
    PurgeQueue,
}

impl Method {
    pub fn parse(method: &str) -> Result<Self, Error> {
        let parser = seq(SQS_METHOD_PREFIX) * sym('.') * seq(method);

        match parser.parse_str(method).map_err(|_| Error::InvalidMethod)? {
            "SendMessage" => Ok(Self::SendMessage),
            "SendMessageBatch" => Ok(Self::SendMessageBatch),
            "ReceiveMessage" => Ok(Self::ReceiveMessage),
            "DeleteMessage" => Ok(Self::DeleteMessage),
            "ListQueues" => Ok(Self::ListQueues),
            "GetQueueUrl" => Ok(Self::GetQueueUrl),
            "CreateQueue" => Ok(Self::CreateQueue),
            "GetQueueAttributes" => Ok(Self::GetQueueAttributes),
            "PurgeQueue" => Ok(Self::PurgeQueue),
            _ => Err(Error::InvalidMethod),
        }
    }
}

pub enum Error {
    InvalidMethod,
}

pub struct SqsApi;

impl<S, B> Transform<S, ServiceRequest> for SqsApi
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;

    type Error = Error;

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
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future =
        std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        Box::pin(async move {
            let method = req
                .headers()
                .get("X-Amz-Target")
                .and_then(|header| header.to_str().ok())
                .ok_or_else(|| Error::InvalidMethod)
                .and_then(Method::parse)?;

            req.extensions_mut().insert(method);

            service.call(req).await
        })
    }
}
