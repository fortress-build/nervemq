use actix_identity::Identity;
use actix_web::{post, web::Data, Responder, Scope};
use bytes::Bytes;
use futures_util::{stream::MapErr, TryStreamExt as _};
use method::Method;
use tokio_serde::{formats::SymmetricalJson, Framed};
use tokio_stream::StreamExt;
use tokio_util::{
    codec::{BytesCodec, FramedRead},
    io::StreamReader,
};
use types::{
    create_queue::{CreateQueueRequest, CreateQueueResponse},
    delete_message::{DeleteMessageRequest, DeleteMessageResponse},
    delete_queue::{DeleteQueueRequest, DeleteQueueResponse},
    get_queue_attributes::{GetQueueAttributesRequest, GetQueueAttributesResponse},
    get_queue_url::{GetQueueUrlRequest, GetQueueUrlResponse},
    list_queues::{ListQueuesRequest, ListQueuesResponse},
    purge_queue::{PurgeQueueRequest, PurgeQueueResponse},
    receive_message::{ReceiveMessageRequest, ReceiveMessageResponse},
    send_message::{SendMessageRequest, SendMessageResponse},
    send_message_batch::{
        SendMessageBatchRequest, SendMessageBatchResponse, SendMessageBatchResultEntry,
        SendMessageBatchResultErrorEntry,
    },
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

type Stream<M> = Framed<
    FramedRead<
        StreamReader<
            MapErr<
                actix_web::web::Payload,
                Box<dyn FnMut(actix_web::error::PayloadError) -> std::io::Error>,
            >,
            Bytes,
        >,
        BytesCodec,
    >,
    M,
    M,
    SymmetricalJson<M>,
>;

async fn send_message(
    service: Data<crate::service::Service>,
    identity: Identity,
    namespace: AuthorizedNamespace,
    mut stream: Stream<SendMessageRequest>,
) -> Result<SqsResponse, Error> {
    let request = stream
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

    Ok(SqsResponse::SendMessage(SendMessageResponse {
        message_id,
        md5_of_message_body: digest,
    }))
}

async fn send_message_batch(
    service: Data<crate::service::Service>,
    identity: Identity,
    namespace: AuthorizedNamespace,
    mut stream: Stream<SendMessageBatchRequest>,
) -> Result<SqsResponse, Error> {
    let request = stream
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

    let mut successful = Vec::new();
    let mut failed = Vec::new();

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
                    sender_fault: true,
                    code: "InternalError".to_string(),
                    message: e.to_string(),
                });
            }
        }
    }

    Ok(if !failed.is_empty() {
        SqsResponse::SendMessageBatch(SendMessageBatchResponse::Failed { failed })
    } else {
        SqsResponse::SendMessageBatch(SendMessageBatchResponse::Successful { successful })
    })
}

async fn receive_message(
    service: Data<crate::service::Service>,
    identity: Identity,
    namespace: AuthorizedNamespace,
    mut stream: Stream<ReceiveMessageRequest>,
) -> Result<SqsResponse, Error> {
    let request = stream
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

    let messages = service
        .sqs_recv_batch(
            namespace_name,
            queue_name,
            request.max_number_of_messages as u64,
        )
        .await?;

    Ok(SqsResponse::ReceiveMessage(ReceiveMessageResponse {
        messages,
    }))
}

async fn delete_message(
    service: Data<crate::service::Service>,
    identity: Identity,
    namespace: AuthorizedNamespace,
    mut stream: Stream<DeleteMessageRequest>,
) -> Result<SqsResponse, Error> {
    let request = stream
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

async fn list_queues(
    service: Data<crate::service::Service>,
    identity: Identity,
    namespace: AuthorizedNamespace,
    mut stream: Stream<ListQueuesRequest>,
) -> Result<SqsResponse, Error> {
    let request = stream
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

    Ok(SqsResponse::ListQueues(ListQueuesResponse {
        queue_urls: urls,
    }))
}

async fn get_queue_url(
    service: Data<crate::service::Service>,
    identity: Identity,
    namespace: AuthorizedNamespace,
    mut stream: Stream<GetQueueUrlRequest>,
) -> Result<SqsResponse, Error> {
    let request = stream
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
        .get_queue_id(&namespace.0, &request.queue_name, service.db())
        .await?
        .ok_or_else(|| Error::queue_not_found(&request.queue_name, &namespace.0))?;

    let url = queue_url(service.config().host(), &request.queue_name, &namespace.0)?;

    Ok(SqsResponse::GetQueueUrl(GetQueueUrlResponse {
        queue_url: url,
    }))
}

async fn create_queue(
    service: Data<crate::service::Service>,
    identity: Identity,
    namespace: AuthorizedNamespace,
    mut stream: Stream<CreateQueueRequest>,
) -> Result<SqsResponse, Error> {
    let request = stream
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

    Ok(SqsResponse::CreateQueue(CreateQueueResponse {
        queue_url: url,
    }))
}

async fn set_queue_attributes(
    service: Data<crate::service::Service>,
    identity: Identity,
    namespace: AuthorizedNamespace,
    mut stream: Stream<SetQueueAttributesRequest>,
) -> Result<SqsResponse, Error> {
    let request = stream
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

    service
        .set_queue_attributes(namespace_name, queue_name, request.attributes, identity)
        .await?;

    Ok(SqsResponse::SetQueueAttributes(
        SetQueueAttributesResponse {},
    ))
}

async fn get_queue_attributes(
    service: Data<crate::service::Service>,
    identity: Identity,
    namespace: AuthorizedNamespace,
    mut stream: Stream<GetQueueAttributesRequest>,
) -> Result<SqsResponse, Error> {
    let request = stream
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

    Ok(SqsResponse::GetQueueAttributes(
        GetQueueAttributesResponse { attributes },
    ))
}

async fn purge_queue(
    service: Data<crate::service::Service>,
    identity: Identity,
    _namespace: AuthorizedNamespace,
    mut stream: Stream<PurgeQueueRequest>,
) -> Result<SqsResponse, Error> {
    let request = stream
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

    let success = service
        .purge_queue(namespace_name, queue_name, identity)
        .await
        .is_ok();

    Ok(SqsResponse::PurgeQueue(PurgeQueueResponse { success }))
}

async fn delete_queue(
    service: Data<crate::service::Service>,
    identity: Identity,
    _namespace: AuthorizedNamespace,
    mut stream: Stream<DeleteQueueRequest>,
) -> Result<SqsResponse, Error> {
    let request = stream
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

    service
        .delete_queue(namespace_name, queue_name, identity)
        .await?;

    Ok(SqsResponse::DeleteQueue(DeleteQueueResponse {}))
}

async fn list_queue_tags(
    service: Data<crate::service::Service>,
    identity: Identity,
    namespace: AuthorizedNamespace,
    mut stream: Stream<types::list_queue_tags::ListQueueTagsRequest>,
) -> Result<SqsResponse, Error> {
    let request = stream
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

    let tags = service
        .get_queue_tags(namespace_name, queue_name, identity)
        .await?;

    Ok(SqsResponse::ListQueueTags(
        types::list_queue_tags::ListQueueTagsResponse { tags },
    ))
}

async fn tag_queue(
    service: Data<crate::service::Service>,
    identity: Identity,
    namespace: AuthorizedNamespace,
    mut stream: Stream<types::tag_queue::TagQueueRequest>,
) -> Result<SqsResponse, Error> {
    let request = stream
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

    if namespace_name != namespace.0 {
        return Err(Error::Unauthorized);
    }

    service
        .tag_queue(namespace_name, queue_name, request.tags, identity)
        .await?;

    Ok(SqsResponse::TagQueue(types::tag_queue::TagQueueResponse {}))
}

async fn untag_queue(
    service: Data<crate::service::Service>,
    identity: Identity,
    namespace: AuthorizedNamespace,
    mut stream: Stream<types::untag_queue::UntagQueueRequest>,
) -> Result<SqsResponse, Error> {
    let request = stream
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
                Framed::new(stream, SymmetricalJson::default()),
            )
            .await?
        }
        Method::TagQueue => {
            tag_queue(
                service,
                identity,
                namespace,
                Framed::new(stream, SymmetricalJson::default()),
            )
            .await?
        }
        Method::UntagQueue => {
            untag_queue(
                service,
                identity,
                namespace,
                Framed::new(stream, SymmetricalJson::default()),
            )
            .await?
        }
        Method::ListQueueTags => {
            list_queue_tags(
                service,
                identity,
                namespace,
                Framed::new(stream, SymmetricalJson::default()),
            )
            .await?
        }
        Method::DeleteQueue => {
            delete_queue(
                service,
                identity,
                namespace,
                Framed::new(stream, SymmetricalJson::default()),
            )
            .await?
        }
        Method::SendMessage => {
            send_message(
                service,
                identity,
                namespace,
                Framed::new(stream, SymmetricalJson::default()),
            )
            .await?
        }
        Method::SendMessageBatch => {
            send_message_batch(
                service,
                identity,
                namespace,
                Framed::new(stream, SymmetricalJson::default()),
            )
            .await?
        }
        Method::ReceiveMessage => {
            receive_message(
                service,
                identity,
                namespace,
                Framed::new(stream, SymmetricalJson::default()),
            )
            .await?
        }
        Method::DeleteMessage => {
            delete_message(
                service,
                identity,
                namespace,
                Framed::new(stream, SymmetricalJson::default()),
            )
            .await?
        }
        Method::ListQueues => {
            list_queues(
                service,
                identity,
                namespace,
                Framed::new(stream, SymmetricalJson::default()),
            )
            .await?
        }
        Method::GetQueueUrl => {
            get_queue_url(
                service,
                identity,
                namespace,
                Framed::new(stream, SymmetricalJson::default()),
            )
            .await?
        }
        Method::CreateQueue => {
            create_queue(
                service,
                identity,
                namespace,
                Framed::new(stream, SymmetricalJson::default()),
            )
            .await?
        }
        Method::GetQueueAttributes => {
            get_queue_attributes(
                service,
                identity,
                namespace,
                Framed::new(stream, SymmetricalJson::default()),
            )
            .await?
        }
        Method::PurgeQueue => {
            purge_queue(
                service,
                identity,
                namespace,
                Framed::new(stream, SymmetricalJson::default()),
            )
            .await?
        }
    };

    Ok(actix_web::web::Json(res))
}

pub fn service() -> Scope {
    actix_web::web::scope("/sqs").service(sqs_service)
}
