use std::rc::Rc;

use actix_identity::Identity;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    post,
    web::Data,
    FromRequest, HttpMessage, Responder,
};
use pom::utf8::{seq, sym};
use url::Url;

use crate::error::Error;

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

        match parser.parse_str(method).map_err(|_| Error::InvalidHeader {
            header: "X-Amz-Target".to_owned(),
        })? {
            "SendMessage" => Ok(Self::SendMessage),
            "SendMessageBatch" => Ok(Self::SendMessageBatch),
            "ReceiveMessage" => Ok(Self::ReceiveMessage),
            "DeleteMessage" => Ok(Self::DeleteMessage),
            "ListQueues" => Ok(Self::ListQueues),
            "GetQueueUrl" => Ok(Self::GetQueueUrl),
            "CreateQueue" => Ok(Self::CreateQueue),
            "GetQueueAttributes" => Ok(Self::GetQueueAttributes),
            "PurgeQueue" => Ok(Self::PurgeQueue),
            _ => Err(Error::InvalidHeader {
                header: "X-Amz-Target".to_owned(),
            }),
        }
    }
}

impl FromRequest for Method {
    type Error = Error;

    type Future = std::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        std::future::ready(req.extensions().get::<Method>().cloned().ok_or_else(|| {
            Error::MissingHeader {
                header: "X-Amz-Target".to_owned(),
            }
        }))
    }
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
                .ok_or_else(|| Error::InvalidHeader {
                    header: "X-Amz-Target".to_owned(),
                })
                .and_then(Method::parse)?;

            req.extensions_mut().insert(method);

            service.call(req).await
        })
    }
}

pub mod types {
    use std::collections::HashMap;
    use url::Url;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct SendMessageRequest {
        pub queue_url: Url,
        pub message_body: Vec<u8>,
        pub delay_seconds: Option<u64>,
        pub message_attributes: HashMap<String, String>,
        pub message_deduplication_id: Option<String>,
        pub message_group_id: Option<String>,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct SendMessageResponse {
        pub message_id: u64,
        pub md5_of_message_body: String,
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct GetQueueUrlRequest {
        pub queue_name: String,
        pub queue_owner_aws_account_id: String,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct GetQueueUrlResponse {
        pub queue_url: Url,
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct CreateQueueRequest {
        pub queue_name: String,
        pub attributes: HashMap<String, String>,
        pub tags: HashMap<String, String>,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct CreateQueueResponse {
        pub queue_url: Url,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase", untagged)]
    pub enum SqsResponse {
        SendMessage(SendMessageResponse),
        GetQueueUrl(GetQueueUrlResponse),
        CreateQueue(CreateQueueResponse),
    }
}
use types::*;

fn queue_url(mut host: Url, queue_name: &str, namespace_name: &str) -> Result<url::Url, Error> {
    host.path_segments_mut()
        .map_err(|_| Error::InternalServerError { source: None })?
        .push("sqs")
        .push(namespace_name)
        .push(queue_name);
    Ok(host)
}

#[post("/sqs")]
pub async fn sqs_service(
    service: Data<crate::service::Service>,
    method: Method,
    data: actix_web::web::Json<serde_json::Value>,
    identity: Identity,
) -> Result<impl Responder, Error> {
    let data = data.into_inner();

    match method {
        Method::SendMessage => {
            let request: types::SendMessageRequest = serde_json::from_value(data)?;

            let mut path = request
                .queue_url
                .path_segments()
                .ok_or_else(|| Error::NotFound)?;

            let (queue_name, namespace_name) = path
                .next_back()
                .and_then(|queue_name| path.next_back().map(|ns_name| (queue_name, ns_name)))
                .ok_or_else(|| Error::NotFound)?;

            let ns_id = service
                .get_namespace_id(namespace_name, service.db())
                .await?
                .ok_or_else(|| Error::NotFound)?;

            service
                .check_user_access(&identity, ns_id, service.db())
                .await?;

            let queue_id = service
                .get_queue_id(namespace_name, queue_name, service.db())
                .await?
                .ok_or_else(|| Error::NotFound)?;

            // FIXME: Implement delay_seconds and typed attributes
            let message_id = service
                .send_message(queue_id, &request.message_body, request.message_attributes)
                .await?;

            let digest = hex::encode(md5::compute(&request.message_body).as_ref());

            Ok(actix_web::web::Json(SqsResponse::SendMessage(
                SendMessageResponse {
                    message_id,
                    md5_of_message_body: digest,
                },
            )))
        }
        Method::SendMessageBatch => todo!(),
        Method::ReceiveMessage => todo!(),
        Method::DeleteMessage => todo!(),
        Method::ListQueues => todo!(),
        Method::GetQueueUrl => {
            let request: types::GetQueueUrlRequest = serde_json::from_value(data)?;

            let namespace_id = service
                .get_namespace_id(&request.queue_owner_aws_account_id, service.db())
                .await?
                .ok_or_else(|| Error::NotFound)?;

            service
                .check_user_access(&identity, namespace_id, service.db())
                .await?;

            let url = queue_url(
                service.config().host.clone(),
                &request.queue_name,
                &request.queue_owner_aws_account_id,
            )?;

            Ok(actix_web::web::Json(SqsResponse::GetQueueUrl(
                GetQueueUrlResponse { queue_url: url },
            )))
        }
        Method::CreateQueue => {
            let request: types::CreateQueueRequest = serde_json::from_value(data)?;

            // FIXME: Get namespace from api key
            let namespace_id = service
                .get_namespace_id(&request.queue_name, service.db())
                .await?
                .ok_or_else(|| Error::NotFound)?;

            service
                .check_user_access(&identity, namespace_id, service.db())
                .await?;

            // service.create_queue()

            let url = queue_url(
                service.config().host.clone(),
                &request.queue_name,
                &request.queue_name,
            )?;

            Ok(actix_web::web::Json(SqsResponse::CreateQueue(
                CreateQueueResponse { queue_url: url },
            )))
        }
        Method::GetQueueAttributes => todo!(),
        Method::PurgeQueue => todo!(),
    }
}
