use std::collections::HashSet;

use actix_identity::Identity;
use actix_web::{post, web::Data, Responder, Scope};
use futures_util::TryStreamExt as _;
use method::Method;
use tokio_serde::{formats::SymmetricalJson, SymmetricallyFramed};
use tokio_stream::StreamExt;
use tokio_util::{
    codec::{BytesCodec, FramedRead},
    io::StreamReader,
};
use tracing::instrument;
use types::{
    create_queue::{CreateQueueRequest, CreateQueueResponse},
    delete_message::{DeleteMessageRequest, DeleteMessageResponse},
    delete_queue::{DeleteQueueRequest, DeleteQueueResponse},
    get_queue_attributes::{GetQueueAttributesRequest, GetQueueAttributesResponse},
    get_queue_url::{GetQueueUrlRequest, GetQueueUrlResponse},
    list_queues::{ListQueuesRequest, ListQueuesResponse},
    purge_queue::{PurgeQueueRequest, PurgeQueueResponse},
    receive_message::{ReceiveMessageRequest, ReceiveMessageResponse},
    send_message::SendMessageRequest,
    send_message_batch::SendMessageBatchRequest,
    set_queue_attributes::{SetQueueAttributesRequest, SetQueueAttributesResponse},
    SqsResponse,
};
use url::Url;

use crate::{auth::credential::AuthorizedNamespace, error::Error};

pub mod method;
pub mod service;
pub mod types;

fn queue_url(mut host: Url, queue_name: &str, namespace_name: &str) -> Result<url::Url, Error> {
    host.path_segments_mut()
        .map_err(|_| Error::InternalServerError { source: None })?
        .push("sqs")
        .push(namespace_name)
        .push(queue_name);
    Ok(host)
}

#[instrument(skip(service, identity))]
async fn send_message(
    service: Data<crate::service::Service>,
    identity: Identity,
    namespace: AuthorizedNamespace,
    request: SendMessageRequest,
) -> Result<SqsResponse, Error> {
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

    let res = service.sqs_send(queue_id, request).await?;

    Ok(SqsResponse::SendMessage(res))
}

#[instrument(skip(service, identity))]
async fn send_message_batch(
    service: Data<crate::service::Service>,
    identity: Identity,
    namespace: AuthorizedNamespace,
    request: SendMessageBatchRequest,
) -> Result<SqsResponse, Error> {
    let queue_url = request.queue_url.clone();

    let mut path = queue_url
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

    let res = service
        .sqs_send_batch(namespace_name, queue_name, request)
        .await?;

    Ok(SqsResponse::SendMessageBatch(res))
}

#[instrument(skip(service, identity))]
async fn receive_message(
    service: Data<crate::service::Service>,
    identity: Identity,
    namespace: AuthorizedNamespace,
    request: ReceiveMessageRequest,
) -> Result<SqsResponse, Error> {
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

    let messages = service
        .sqs_recv_batch(
            namespace_name,
            queue_name,
            request.max_number_of_messages.unwrap_or(1) as u64,
            HashSet::from_iter(request.attribute_names.into_iter()),
        )
        .await?;

    Ok(SqsResponse::ReceiveMessage(ReceiveMessageResponse {
        messages,
    }))
}

#[instrument(skip(service, identity))]
async fn delete_message(
    service: Data<crate::service::Service>,
    identity: Identity,
    namespace: AuthorizedNamespace,
    request: DeleteMessageRequest,
) -> Result<SqsResponse, Error> {
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

    Ok(SqsResponse::DeleteMessage(DeleteMessageResponse {}))
}

// // FIXME: Finish implementing this
//
// async fn delete_message_batch(
//     service: Data<crate::service::Service>,
//     identity: Identity,
//     namespace: AuthorizedNamespace,
//     mut stream: Stream<DeleteMessageBatchRequest>,
// ) -> Result<DeleteMessageBatchResponse, Error> {
//     let request = stream
//         .next()
//         .await
//         .transpose()
//         .map_err(|e| Error::internal(e))?
//         .ok_or_else(|| Error::missing_parameter("missing request body"))?;
//
//     let mut path = request
//         .queue_url
//         .path_segments()
//         .ok_or_else(|| Error::missing_parameter("queue name"))?;
//
//     let (queue_name, namespace_name) = path
//         .next_back()
//         .and_then(|queue_name| path.next_back().map(|ns_name| (queue_name, ns_name)))
//         .ok_or_else(|| Error::missing_parameter("namespace name"))?;
//
//     let ns_id = service
//         .get_namespace_id(namespace_name, service.db())
//         .await?
//         .ok_or_else(|| Error::namespace_not_found(namespace_name))?;
//
//     service
//         .check_user_access(&identity, ns_id, service.db())
//         .await?;
//
//     if namespace_name != namespace.0 {
//         return Err(Error::Unauthorized);
//     }
//
//     let message_id = request
//         .receipt_handle
//         .parse::<u64>()
//         .map_err(|e| Error::invalid_parameter(format!("ReceiptHandle: {e}")))?;
//
//     let (successful, failed) = service
//         .delete_message_batch(namespace_name, queue_name, message_id, identity)
//         .await
//         .map(|(successful, failed)| {
//             (
//                 successful
//                     .into_iter()
//                     .map(|id| DeleteMessageBatchResultSuccess { id: id.to_string() })
//                     .collect(),
//                 failed
//                     .into_iter()
//                     .map(|(id, err)| DeleteMessageBatchResultError {
//                         id: id.to_string(),
//                         code: "InternalError".to_string(),
//                         message: err.to_string(),
//                         sender_fault: true,
//                     })
//                     .collect(),
//             )
//         })?;
//
//     Ok(DeleteMessageBatchResponse { failed, successful })
// }

#[instrument(skip(service, identity))]
async fn list_queues(
    service: Data<crate::service::Service>,
    identity: Identity,
    namespace: AuthorizedNamespace,
    request: ListQueuesRequest,
) -> Result<SqsResponse, Error> {
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

    Ok(SqsResponse::ListQueues(ListQueuesResponse {
        queue_urls: urls,
    }))
}

#[instrument(skip(service, identity))]
async fn get_queue_url(
    service: Data<crate::service::Service>,
    identity: Identity,
    namespace: AuthorizedNamespace,
    request: GetQueueUrlRequest,
) -> Result<SqsResponse, Error> {
    let namespace_id = service
        .get_namespace_id(&namespace.0, service.db())
        .await?
        .ok_or_else(|| Error::namespace_not_found(&namespace.0))?;

    service
        .check_user_access(&identity, namespace_id, service.db())
        .await?;

    service
        .get_queue_id(&namespace.0, &request.queue_name, service.db())
        .await?
        .ok_or_else(|| Error::queue_not_found(&request.queue_name, &namespace.0))?;

    let url = queue_url(service.config().host(), &request.queue_name, &namespace.0)?;

    Ok(SqsResponse::GetQueueUrl(GetQueueUrlResponse {
        queue_url: url,
    }))
}

#[instrument(skip(service, identity))]
async fn create_queue(
    service: Data<crate::service::Service>,
    identity: Identity,
    namespace: AuthorizedNamespace,
    request: CreateQueueRequest,
) -> Result<SqsResponse, Error> {
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

    Ok(SqsResponse::CreateQueue(CreateQueueResponse {
        queue_url: url,
    }))
}

#[instrument(skip(service, identity))]
async fn set_queue_attributes(
    service: Data<crate::service::Service>,
    identity: Identity,
    namespace: AuthorizedNamespace,
    request: SetQueueAttributesRequest,
) -> Result<SqsResponse, Error> {
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

    service
        .set_queue_attributes(namespace_name, queue_name, request.attributes, identity)
        .await?;

    Ok(SqsResponse::SetQueueAttributes(
        SetQueueAttributesResponse {},
    ))
}

#[instrument(skip(service, identity))]
async fn get_queue_attributes(
    service: Data<crate::service::Service>,
    identity: Identity,
    namespace: AuthorizedNamespace,
    request: GetQueueAttributesRequest,
) -> Result<SqsResponse, Error> {
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

    Ok(SqsResponse::GetQueueAttributes(
        GetQueueAttributesResponse { attributes },
    ))
}

#[instrument(skip(service, identity))]
async fn purge_queue(
    service: Data<crate::service::Service>,
    identity: Identity,
    _namespace: AuthorizedNamespace,
    request: PurgeQueueRequest,
) -> Result<SqsResponse, Error> {
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

    let success = service
        .purge_queue(namespace_name, queue_name, identity)
        .await
        .is_ok();

    Ok(SqsResponse::PurgeQueue(PurgeQueueResponse { success }))
}

#[instrument(skip(service, identity))]
async fn delete_queue(
    service: Data<crate::service::Service>,
    identity: Identity,
    _namespace: AuthorizedNamespace,
    request: DeleteQueueRequest,
) -> Result<SqsResponse, Error> {
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

    service
        .delete_queue(namespace_name, queue_name, identity)
        .await?;

    Ok(SqsResponse::DeleteQueue(DeleteQueueResponse {}))
}

#[instrument(skip(service, identity))]
async fn list_queue_tags(
    service: Data<crate::service::Service>,
    identity: Identity,
    namespace: AuthorizedNamespace,
    request: types::list_queue_tags::ListQueueTagsRequest,
) -> Result<SqsResponse, Error> {
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

    let tags = service
        .get_queue_tags(namespace_name, queue_name, identity)
        .await?;

    Ok(SqsResponse::ListQueueTags(
        types::list_queue_tags::ListQueueTagsResponse { tags },
    ))
}

#[instrument(skip(service, identity))]
async fn tag_queue(
    service: Data<crate::service::Service>,
    identity: Identity,
    namespace: AuthorizedNamespace,
    request: types::tag_queue::TagQueueRequest,
) -> Result<SqsResponse, Error> {
    let mut path = request
        .queue_url
        .path_segments()
        .ok_or_else(|| Error::missing_parameter("queue name"))?;

    let (queue_name, namespace_name) = path
        .next_back()
        .and_then(|queue_name| path.next_back().map(|ns_name| (queue_name, ns_name)))
        .ok_or_else(|| Error::missing_parameter("namespace name"))?;

    if namespace_name != namespace.0 {
        return Err(Error::Unauthorized);
    }

    service
        .tag_queue(namespace_name, queue_name, request.tags, identity)
        .await?;

    Ok(SqsResponse::TagQueue(types::tag_queue::TagQueueResponse {}))
}

#[instrument(skip(service, identity))]
async fn untag_queue(
    service: Data<crate::service::Service>,
    identity: Identity,
    namespace: AuthorizedNamespace,
    request: types::untag_queue::UntagQueueRequest,
) -> Result<SqsResponse, Error> {
    let mut path = request
        .queue_url
        .path_segments()
        .ok_or_else(|| Error::missing_parameter("queue name"))?;

    let (queue_name, namespace_name) = path
        .next_back()
        .and_then(|queue_name| path.next_back().map(|ns_name| (queue_name, ns_name)))
        .ok_or_else(|| Error::missing_parameter("namespace name"))?;

    if namespace_name != namespace.0 {
        return Err(Error::Unauthorized);
    }

    service
        .untag_queue(namespace_name, queue_name, request.tag_keys, identity)
        .await?;

    Ok(SqsResponse::UntagQueue(
        types::untag_queue::UntagQueueResponse {},
    ))
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
    let stream = StreamReader::new(payload.map_err(Box::new(move |e| {
        std::io::Error::new(std::io::ErrorKind::Other, e)
    }) as Box<dyn FnMut(_) -> _>));

    let stream = FramedRead::new(stream, BytesCodec::new());

    let res = match method {
        Method::DeleteMessageBatch => todo!(),
        Method::SetQueueAttributes => {
            set_queue_attributes(
                service,
                identity,
                namespace,
                SymmetricallyFramed::new(stream, SymmetricalJson::default())
                    .next()
                    .await
                    .transpose()
                    .map_err(|e| Error::internal(e))?
                    .ok_or_else(|| Error::missing_parameter("missing request body"))?,
            )
            .await?
        }
        Method::TagQueue => {
            tag_queue(
                service,
                identity,
                namespace,
                SymmetricallyFramed::new(stream, SymmetricalJson::default())
                    .next()
                    .await
                    .transpose()
                    .map_err(|e| Error::internal(e))?
                    .ok_or_else(|| Error::missing_parameter("missing request body"))?,
            )
            .await?
        }
        Method::UntagQueue => {
            untag_queue(
                service,
                identity,
                namespace,
                SymmetricallyFramed::new(stream, SymmetricalJson::default())
                    .next()
                    .await
                    .transpose()
                    .map_err(|e| Error::internal(e))?
                    .ok_or_else(|| Error::missing_parameter("missing request body"))?,
            )
            .await?
        }
        Method::ListQueueTags => {
            list_queue_tags(
                service,
                identity,
                namespace,
                SymmetricallyFramed::new(stream, SymmetricalJson::default())
                    .next()
                    .await
                    .transpose()
                    .map_err(|e| Error::internal(e))?
                    .ok_or_else(|| Error::missing_parameter("missing request body"))?,
            )
            .await?
        }
        Method::DeleteQueue => {
            delete_queue(
                service,
                identity,
                namespace,
                SymmetricallyFramed::new(stream, SymmetricalJson::default())
                    .next()
                    .await
                    .transpose()
                    .map_err(|e| Error::internal(e))?
                    .ok_or_else(|| Error::missing_parameter("missing request body"))?,
            )
            .await?
        }
        Method::SendMessage => {
            send_message(
                service,
                identity,
                namespace,
                SymmetricallyFramed::new(stream, SymmetricalJson::default())
                    .next()
                    .await
                    .transpose()
                    .map_err(|e| Error::internal(e))?
                    .ok_or_else(|| Error::missing_parameter("missing request body"))?,
            )
            .await?
        }
        Method::SendMessageBatch => {
            send_message_batch(
                service,
                identity,
                namespace,
                SymmetricallyFramed::new(stream, SymmetricalJson::default())
                    .next()
                    .await
                    .transpose()
                    .map_err(|e| Error::internal(e))?
                    .ok_or_else(|| Error::missing_parameter("missing request body"))?,
            )
            .await?
        }
        Method::ReceiveMessage => {
            receive_message(
                service,
                identity,
                namespace,
                SymmetricallyFramed::new(stream, SymmetricalJson::default())
                    .next()
                    .await
                    .transpose()
                    .map_err(|e| Error::internal(e))?
                    .ok_or_else(|| Error::missing_parameter("missing request body"))?,
            )
            .await?
        }
        Method::DeleteMessage => {
            delete_message(
                service,
                identity,
                namespace,
                SymmetricallyFramed::new(stream, SymmetricalJson::default())
                    .next()
                    .await
                    .transpose()
                    .map_err(|e| Error::internal(e))?
                    .ok_or_else(|| Error::missing_parameter("missing request body"))?,
            )
            .await?
        }
        Method::ListQueues => {
            list_queues(
                service,
                identity,
                namespace,
                SymmetricallyFramed::new(stream, SymmetricalJson::default())
                    .next()
                    .await
                    .transpose()
                    .map_err(|e| Error::internal(e))?
                    .ok_or_else(|| Error::missing_parameter("missing request body"))?,
            )
            .await?
        }
        Method::GetQueueUrl => {
            get_queue_url(
                service,
                identity,
                namespace,
                SymmetricallyFramed::new(stream, SymmetricalJson::default())
                    .next()
                    .await
                    .transpose()
                    .map_err(|e| Error::internal(e))?
                    .ok_or_else(|| Error::missing_parameter("missing request body"))?,
            )
            .await?
        }
        Method::CreateQueue => {
            create_queue(
                service,
                identity,
                namespace,
                SymmetricallyFramed::new(stream, SymmetricalJson::default())
                    .next()
                    .await
                    .transpose()
                    .map_err(|e| Error::internal(e))?
                    .ok_or_else(|| Error::missing_parameter("missing request body"))?,
            )
            .await?
        }
        Method::GetQueueAttributes => {
            get_queue_attributes(
                service,
                identity,
                namespace,
                SymmetricallyFramed::new(stream, SymmetricalJson::default())
                    .next()
                    .await
                    .transpose()
                    .map_err(|e| Error::internal(e))?
                    .ok_or_else(|| Error::missing_parameter("missing request body"))?,
            )
            .await?
        }
        Method::PurgeQueue => {
            purge_queue(
                service,
                identity,
                namespace,
                SymmetricallyFramed::new(stream, SymmetricalJson::default())
                    .next()
                    .await
                    .transpose()
                    .map_err(|e| Error::internal(e))?
                    .ok_or_else(|| Error::missing_parameter("missing request body"))?,
            )
            .await?
        }
    };

    Ok(actix_web::web::Json(res))
}

pub fn service() -> Scope {
    actix_web::web::scope("/sqs").service(sqs_service)
}
