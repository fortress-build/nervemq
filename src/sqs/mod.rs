use actix_identity::Identity;
use actix_web::{post, web::Data, Responder, Scope};
use futures_util::TryStreamExt as _;
use method::Method;
use tokio_serde::{formats::SymmetricalJson, Framed};
use tokio_stream::StreamExt;
use tokio_util::{
    codec::{BytesCodec, FramedRead},
    io::StreamReader,
};
use url::Url;

use crate::{auth::credential::AuthorizedNamespace, error::Error};

pub mod method;
pub mod service;
pub mod types;

use types::*;

fn queue_url(mut host: Url, queue_name: &str, namespace_name: &str) -> Result<url::Url, Error> {
    host.path_segments_mut()
        .map_err(|_| Error::InternalServerError { source: None })?
        .push("sqs")
        .push(namespace_name)
        .push(queue_name);
    Ok(host)
}

#[post("")]
pub async fn sqs_service(
    service: Data<crate::service::Service>,
    method: Method,
    payload: actix_web::web::Payload,
    // payload: actix_web::web::Bytes,
    identity: Identity,
    namespace: AuthorizedNamespace,
) -> Result<impl Responder, Error> {
    let stream =
        StreamReader::new(payload.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)));

    let stream = FramedRead::new(stream, BytesCodec::new());

    match method {
        Method::TagQueue => todo!(),
        Method::UntagQueue => todo!(),
        Method::ListQueueTags => todo!(),
        Method::DeleteQueue => todo!(),
        Method::SendMessage => {
            let request: types::SendMessageRequest =
                Framed::<_, _, types::SendMessageRequest, _>::new(
                    stream,
                    SymmetricalJson::default(),
                )
                .next()
                .await
                .transpose()
                .map_err(|e| Error::internal(e))?
                .ok_or_else(|| Error::missing_parameter("missing request body"))?;

            let mut path = request
                .queue_url
                .path_segments()
                .ok_or_else(|| Error::missing_parameter("queue name"))?;

            let (queue_name, namespace_name) = path
                .next_back()
                .and_then(|queue_name| path.next_back().map(|ns_name| (queue_name, ns_name)))
                .ok_or_else(|| Error::missing_parameter("namespace name"))?;

            let ns_id = service
                .get_namespace_id(namespace_name, service.db())
                .await?
                .ok_or_else(|| Error::namespace_not_found(namespace_name))?;

            service
                .check_user_access(&identity, ns_id, service.db())
                .await?;

            if namespace_name != namespace.0 {
                return Err(Error::Unauthorized);
            }

            let queue_id = service
                .get_queue_id(namespace_name, queue_name, service.db())
                .await?
                .ok_or_else(|| Error::queue_not_found(queue_name, namespace_name))?;

            // FIXME: Implement delay_seconds
            let message_id = service
                .sqs_send(queue_id, &request.message_body, request.message_attributes)
                .await?;

            let digest = hex::encode(md5::compute(&request.message_body).as_ref());

            Ok(actix_web::web::Json(SqsResponse::SendMessage(
                SendMessageResponse {
                    message_id,
                    md5_of_message_body: digest,
                },
            )))
        }
        Method::SendMessageBatch => {
            let request: types::SendMessageBatchRequest =
                Framed::<_, _, types::SendMessageBatchRequest, _>::new(
                    stream,
                    tokio_serde::formats::SymmetricalJson::default(),
                )
                .next()
                .await
                .transpose()
                .map_err(|e| Error::internal(e))?
                .ok_or_else(|| Error::missing_parameter("missing request body"))?;

            // Parse queue URL to get namespace and queue name
            let mut path = request
                .queue_url
                .path_segments()
                .ok_or_else(|| Error::missing_parameter("queue name"))?;

            let (queue_name, namespace_name) = path
                .next_back()
                .and_then(|queue_name| path.next_back().map(|ns_name| (queue_name, ns_name)))
                .ok_or_else(|| Error::missing_parameter("namespace name"))?;

            // Verify namespace access
            let ns_id = service
                .get_namespace_id(namespace_name, service.db())
                .await?
                .ok_or_else(|| Error::namespace_not_found(namespace_name))?;

            service
                .check_user_access(&identity, ns_id, service.db())
                .await?;

            if namespace_name != namespace.0 {
                return Err(Error::Unauthorized);
            }

            // Get queue ID
            let queue_id = service
                .get_queue_id(namespace_name, queue_name, service.db())
                .await?
                .ok_or_else(|| Error::queue_not_found(queue_name, namespace_name))?;

            let mut successful = Vec::new();
            let mut failed = Vec::new();

            // Process each message in the batch
            for entry in request.entries {
                let message_attributes = entry.message_attributes;
                let message_body = entry.message_body;

                match service
                    .sqs_send(queue_id, &message_body, message_attributes)
                    .await
                {
                    Ok(message_id) => {
                        let digest = hex::encode(md5::compute(&message_body).as_ref());
                        successful.push(SendMessageBatchResultEntry {
                            id: entry.id,
                            message_id: message_id.to_string(),
                            md5_of_message_body: digest,
                        });
                    }
                    Err(e) => {
                        failed.push(SendMessageBatchResultErrorEntry {
                            id: entry.id,
                            sender_fault: true, // Set to true for client-side errors
                            code: "InternalError".to_string(),
                            message: e.to_string(),
                        });
                    }
                }
            }

            // Return successful or failed response based on results
            let response = if !failed.is_empty() {
                SqsResponse::SendMessageBatch(SendMessageBatchResponse::Failed { failed })
            } else {
                SqsResponse::SendMessageBatch(SendMessageBatchResponse::Successful { successful })
            };

            Ok(actix_web::web::Json(response))
        }
        Method::ReceiveMessage => {
            let request: types::ReceiveMessageRequest =
                Framed::<_, _, types::ReceiveMessageRequest, _>::new(
                    stream,
                    tokio_serde::formats::SymmetricalJson::default(),
                )
                .next()
                .await
                .transpose()
                .map_err(|e| Error::internal(e))?
                .ok_or_else(|| Error::missing_parameter("missing request body"))?;

            // Parse queue URL to get namespace and queue name
            let mut path = request
                .queue_url
                .path_segments()
                .ok_or_else(|| Error::missing_parameter("queue name"))?;

            let (queue_name, namespace_name) = path
                .next_back()
                .and_then(|queue_name| path.next_back().map(|ns_name| (queue_name, ns_name)))
                .ok_or_else(|| Error::missing_parameter("namespace name"))?;

            // Verify namespace access
            let ns_id = service
                .get_namespace_id(namespace_name, service.db())
                .await?
                .ok_or_else(|| Error::namespace_not_found(namespace_name))?;

            service
                .check_user_access(&identity, ns_id, service.db())
                .await?;

            if namespace_name != namespace.0 {
                return Err(Error::Unauthorized);
            }

            // Get messages from the queue
            let messages = service
                .sqs_recv_batch(
                    namespace_name,
                    queue_name,
                    request.max_number_of_messages as u64,
                    // request.visibility_timeout,
                    // request.wait_time_seconds,
                    // identity,
                )
                .await?
                .into_iter()
                .map(|msg| SqsMessage {
                    message_id: msg.message_id.to_string(),
                    md5_of_body: hex::encode(md5::compute(&msg.body).as_ref()),
                    body: msg.body,
                })
                .collect();

            Ok(actix_web::web::Json(SqsResponse::ReceiveMessage(
                ReceiveMessageResponse { messages },
            )))
        }
        Method::DeleteMessage => {
            let request: types::DeleteMessageRequest =
                Framed::<_, _, types::DeleteMessageRequest, _>::new(
                    stream,
                    tokio_serde::formats::SymmetricalJson::default(),
                )
                .next()
                .await
                .transpose()
                .map_err(|e| Error::internal(e))?
                .ok_or_else(|| Error::missing_parameter("missing request body"))?;

            let mut path = request
                .queue_url
                .path_segments()
                .ok_or_else(|| Error::missing_parameter("queue name"))?;

            let (queue_name, namespace_name) = path
                .next_back()
                .and_then(|queue_name| path.next_back().map(|ns_name| (queue_name, ns_name)))
                .ok_or_else(|| Error::missing_parameter("namespace name"))?;

            let ns_id = service
                .get_namespace_id(namespace_name, service.db())
                .await?
                .ok_or_else(|| Error::namespace_not_found(namespace_name))?;

            service
                .check_user_access(&identity, ns_id, service.db())
                .await?;

            if namespace_name != namespace.0 {
                return Err(Error::Unauthorized);
            }

            let message_id = request
                .receipt_handle
                .parse::<u64>()
                .map_err(|e| Error::invalid_parameter(format!("ReceiptHandle: {e}")))?;

            service
                .delete_message(namespace_name, queue_name, message_id, identity)
                .await?;

            Ok(actix_web::web::Json(SqsResponse::DeleteMessage(
                DeleteMessageResponse {},
            )))
        }
        Method::ListQueues => {
            let request: types::ListQueuesRequest =
                Framed::<_, _, types::ListQueuesRequest, _>::new(
                    stream,
                    tokio_serde::formats::SymmetricalJson::default(),
                )
                .next()
                .await
                .transpose()
                .map_err(|e| Error::internal(e))?
                .ok_or_else(|| Error::missing_parameter("missing request body"))?;

            let namespace_id = service
                .get_namespace_id(&namespace.0, service.db())
                .await?
                .ok_or_else(|| Error::namespace_not_found(&namespace.0))?;

            service
                .check_user_access(&identity, namespace_id, service.db())
                .await?;

            let queues = service
                .list_queues(Some(&namespace.0), identity)
                .await?
                .into_iter()
                .filter(|queue| {
                    if let Some(prefix) = &request.queue_name_prefix {
                        queue.name.starts_with(prefix)
                    } else {
                        true
                    }
                });

            let mut urls = Vec::new();

            for queue in queues {
                urls.push(queue_url(
                    service.config().host(),
                    &queue.name,
                    &namespace.0,
                )?);
            }

            Ok(actix_web::web::Json(SqsResponse::ListQueues(
                ListQueuesResponse { queue_urls: urls },
            )))
        }
        Method::GetQueueUrl => {
            let request: types::GetQueueUrlRequest =
                Framed::<_, _, types::GetQueueUrlRequest, _>::new(
                    stream,
                    tokio_serde::formats::SymmetricalJson::default(),
                )
                .next()
                .await
                .transpose()
                .map_err(|e| Error::internal(e))?
                .ok_or_else(|| Error::missing_parameter("missing request body"))?;

            let namespace_id = service
                .get_namespace_id(&namespace.0, service.db())
                .await?
                .ok_or_else(|| Error::namespace_not_found(&namespace.0))?;

            service
                .check_user_access(&identity, namespace_id, service.db())
                .await?;

            // We don't need the id, but we need to ensure the queue exists
            service
                .get_queue_id(&namespace.0, &request.queue_name, service.db())
                .await?
                .ok_or_else(|| Error::queue_not_found(&request.queue_name, &namespace.0))?;

            let url = queue_url(service.config().host(), &request.queue_name, &namespace.0)?;

            Ok(actix_web::web::Json(SqsResponse::GetQueueUrl(
                GetQueueUrlResponse { queue_url: url },
            )))
        }
        Method::CreateQueue => {
            let request: types::CreateQueueRequest =
                Framed::<_, _, types::GetQueueUrlRequest, _>::new(
                    stream,
                    tokio_serde::formats::SymmetricalJson::default(),
                )
                .next()
                .await
                .transpose()
                .map_err(|e| Error::internal(e))?
                .ok_or_else(|| Error::missing_parameter("missing request body"))?;

            let namespace_id = service
                .get_namespace_id(&namespace.0, service.db())
                .await?
                .ok_or_else(|| Error::namespace_not_found(&namespace.0))?;

            service
                .check_user_access(&identity, namespace_id, service.db())
                .await?;

            service
                .create_queue(
                    &namespace.0,
                    &request.queue_name,
                    request.attributes,
                    request.tags,
                    identity,
                )
                .await?;

            let url = queue_url(service.config().host(), &request.queue_name, &namespace.0)?;

            Ok(actix_web::web::Json(SqsResponse::CreateQueue(
                CreateQueueResponse { queue_url: url },
            )))
        }
        Method::GetQueueAttributes => {
            let request: types::GetQueueAttributesRequest =
                Framed::<_, _, types::GetQueueUrlRequest, _>::new(
                    stream,
                    tokio_serde::formats::SymmetricalJson::default(),
                )
                .next()
                .await
                .transpose()
                .map_err(|e| Error::internal(e))?
                .ok_or_else(|| Error::missing_parameter("missing request body"))?;

            let mut path = request
                .queue_url
                .path_segments()
                .ok_or_else(|| Error::missing_parameter("queue name"))?;

            let (queue_name, namespace_name) = path
                .next_back()
                .and_then(|queue_name| path.next_back().map(|ns_name| (queue_name, ns_name)))
                .ok_or_else(|| Error::missing_parameter("namespace name"))?;

            let ns_id = service
                .get_namespace_id(namespace_name, service.db())
                .await?
                .ok_or_else(|| Error::namespace_not_found(namespace_name))?;

            service
                .check_user_access(&identity, ns_id, service.db())
                .await?;

            if namespace_name != namespace.0 {
                return Err(Error::Unauthorized);
            }

            let attributes = service
                .get_queue_attributes(
                    namespace_name,
                    queue_name,
                    &request.attribute_names,
                    identity,
                )
                .await?;

            Ok(actix_web::web::Json(SqsResponse::GetQueueAttributes(
                GetQueueAttributesResponse { attributes },
            )))
        }
        Method::PurgeQueue => {
            let request: types::PurgeQueueRequest =
                Framed::<_, _, types::GetQueueUrlRequest, _>::new(
                    stream,
                    tokio_serde::formats::SymmetricalJson::default(),
                )
                .next()
                .await
                .transpose()
                .map_err(|e| Error::internal(e))?
                .ok_or_else(|| Error::missing_parameter("missing request body"))?;

            // Parse queue URL to get namespace and queue name
            let mut path = request
                .queue_url
                .path_segments()
                .ok_or_else(|| Error::missing_parameter("queue name"))?;

            let (queue_name, namespace_name) = path
                .next_back()
                .and_then(|queue_name| path.next_back().map(|ns_name| (queue_name, ns_name)))
                .ok_or_else(|| Error::missing_parameter("namespace name"))?;

            // Verify namespace access
            let ns_id = service
                .get_namespace_id(namespace_name, service.db())
                .await?
                .ok_or_else(|| Error::namespace_not_found(namespace_name))?;

            service
                .check_user_access(&identity, ns_id, service.db())
                .await?;

            // Purge all messages from the queue
            let success = service
                .purge_queue(namespace_name, queue_name, identity)
                .await
                .is_ok();

            Ok(actix_web::web::Json(SqsResponse::PurgeQueue(
                PurgeQueueResponse { success },
            )))
        }
    }
}

pub fn service() -> Scope {
    actix_web::web::scope("/sqs").service(sqs_service)
}
