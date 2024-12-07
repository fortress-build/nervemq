//! AWS SQS-compatible API types and data structures.
//!
//! This module defines the request and response types for implementing
//! an AWS SQS-compatible API interface. It includes all the major SQS
//! operations like:
//!
//! - Queue management (create, delete, list, purge)
//! - Message operations (send, receive, delete)
//! - Queue attribute management
//! - Queue tagging
//! - Batch operations
//!
//! Each operation is organized in its own submodule with corresponding
//! request and response types that match the AWS SQS API specification.
//!
//! # Message Attributes
//!
//! The system supports three types of message attributes:
//! - String values
//! - Number values (stored as strings)
//! - Binary values
//!
//! # API Compatibility
//!
//! The types in this module are designed to be wire-compatible with the
//! AWS SQS API, using the same field names and serialization formats.

use bytes::BufMut;
use std::collections::HashMap;
use url::Url;

/// Types for the SendMessage API operation.
///
/// Handles sending a single message to a queue with optional
/// attributes and delivery delay settings.
pub mod send_message {
    use super::*;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    /// Request for the SendMessage operation.
    pub struct SendMessageRequest {
        pub queue_url: Url,
        pub message_body: String,
        pub delay_seconds: Option<u64>,
        pub message_attributes: HashMap<String, SqsMessageAttribute>,
        pub message_deduplication_id: Option<String>,
        pub message_group_id: Option<String>,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    /// Response for the SendMessage operation.
    pub struct SendMessageResponse {
        pub message_id: u64,

        #[serde(rename = "MD5OfMessageBody")]
        pub md5_of_message_body: String,

        #[serde(rename = "MD5OfMessageAttributes")]
        pub md5_of_message_attributes: String,
        // pub md5_of_message_system_attributes: String,
        // pub sequence_number: Option<String>,
    }
}

/// Types for the GetQueueUrl API operation.
///
/// Retrieves the URL of a queue given its name. The URL is required
/// for most other queue operations.
pub mod get_queue_url {
    use super::*;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    /// Request for the GetQueueUrl operation.
    pub struct GetQueueUrlRequest {
        pub queue_name: String,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    /// Response for the GetQueueUrl operation.
    pub struct GetQueueUrlResponse {
        pub queue_url: Url,
    }
}

/// Types for the CreateQueue API operation.
///
/// Handles queue creation with configurable attributes and tags.
/// Creates a new queue or returns the URL of an existing queue with
/// the same name.
pub mod create_queue {
    use super::*;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    /// Request for the CreateQueue operation.
    pub struct CreateQueueRequest {
        pub queue_name: String,
        #[serde(default)]
        pub attributes: HashMap<String, String>,
        #[serde(default)]
        pub tags: HashMap<String, String>,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    /// Response for the CreateQueue operation.
    pub struct CreateQueueResponse {
        pub queue_url: Url,
    }
}

/// Types for the ListQueues API operation.
///
/// Returns a list of queue URLs, optionally filtered by a name prefix.
/// Useful for discovering existing queues in the system.
pub mod list_queues {
    use super::*;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    /// Request for the ListQueues operation.
    pub struct ListQueuesRequest {
        pub queue_name_prefix: Option<String>,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    /// Response for the ListQueues operation.
    pub struct ListQueuesResponse {
        pub queue_urls: Vec<Url>,
    }
}

/// Types for the DeleteMessage API operation.
///
/// Deletes a specific message from a queue using its receipt handle.
/// The receipt handle is obtained when receiving the message.
pub mod delete_message {
    use super::*;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    /// Request for the DeleteMessage operation.
    pub struct DeleteMessageRequest {
        pub queue_url: Url,
        pub receipt_handle: String,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    /// Empty response for the DeleteMessage operation.
    pub struct DeleteMessageResponse {}
}

/// Types for the DeleteQueue API operation.
///
/// Permanently deletes a queue and all its messages. This operation
/// cannot be undone.
pub mod delete_queue {
    use super::*;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    /// Request for the DeleteQueue operation.
    pub struct DeleteQueueRequest {
        pub queue_url: Url,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    /// Empty response for the DeleteQueue operation.
    pub struct DeleteQueueResponse {}
}

/// Types for the PurgeQueue API operation.
///
/// Deletes all messages from a queue while retaining the queue itself.
/// Useful for clearing a queue without deleting its configuration.
pub mod purge_queue {
    use super::*;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    /// Request for the PurgeQueue operation.
    pub struct PurgeQueueRequest {
        pub queue_url: Url,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    /// Empty response for the PurgeQueue operation.
    ///
    /// Contains a success flag indicating if the operation was successful.
    pub struct PurgeQueueResponse {
        pub success: bool,
    }
}

/// Types for the GetQueueAttributes API operation.
///
/// Retrieves one or more attributes of a queue. Attributes include
/// settings like delay seconds, message retention period, and
/// visibility timeout.
pub mod get_queue_attributes {
    use crate::service::QueueAttributesSer;

    use super::*;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    /// Request for the GetQueueAttributes operation.
    ///
    /// Contains the queue URL and a list of attribute names to retrieve.
    pub struct GetQueueAttributesRequest {
        pub queue_url: Url,
        pub attribute_names: Vec<String>,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    /// Response for the GetQueueAttributes operation.
    ///
    /// Contains the requested attributes for the queue.
    pub struct GetQueueAttributesResponse {
        pub attributes: QueueAttributesSer,
    }
}

/// Types for the ReceiveMessage API operation.
///
/// Handles retrieving one or more messages from a queue with
/// configurable visibility timeout and wait time settings.
pub mod receive_message {
    use super::*;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    /// Request for the ReceiveMessage operation.
    ///
    /// Contains the queue URL and various options for message retrieval.
    pub struct ReceiveMessageRequest {
        pub queue_url: Url,

        #[serde(default)]
        pub attribute_names: Vec<String>,

        #[serde(default)]
        pub message_attribute_names: Vec<String>,

        pub max_number_of_messages: Option<u64>,
        pub visibility_timeout: Option<u64>,
        pub wait_time_seconds: Option<u64>,
        pub receive_request_attempt_id: Option<String>,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    /// Response for the ReceiveMessage operation.
    ///
    /// Contains a list of messages retrieved from the queue.
    pub struct ReceiveMessageResponse {
        pub messages: Vec<SqsMessage>,
    }
}

/// Types for the SendMessageBatch API operation.
///
/// Sends multiple messages to a queue in a single request.
/// More efficient than sending messages individually for bulk operations.
/// Supports up to 10 messages per request.
pub mod send_message_batch {
    use super::*;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    /// Request for a batch message send operation.
    ///
    /// Contains the queue URL and a list of message entries to send.
    pub struct SendMessageBatchRequest {
        pub queue_url: Url,
        pub entries: Vec<SendMessageBatchRequestEntry>,
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    /// Entry for a batch message send request.
    ///
    /// Each entry represents a single message to be sent as part of
    /// a batch operation, with its own ID and attributes.
    pub struct SendMessageBatchRequestEntry {
        pub id: String,
        pub message_body: String,
        pub delay_seconds: Option<u64>,
        pub message_attributes: HashMap<String, SqsMessageAttribute>,
        pub message_deduplication_id: Option<String>,
        pub message_group_id: Option<String>,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    /// Successful result entry for a batch message send operation.
    ///
    /// Contains the ID of the successfully sent message along with
    /// its message ID and MD5 hash for verification.
    pub struct SendMessageBatchResultEntry {
        pub id: String,
        pub message_id: String,
        #[serde(rename = "MD5OfMessageBody")]
        pub md5_of_message_body: String,
        // pub md5_of_message_attributes: String,
        // pub md5_of_message_system_attributes: String,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    /// Error result entry for a batch message send operation.
    ///
    /// Contains details about why a particular message in the batch
    /// failed to be sent, including error code and message.
    pub struct SendMessageBatchResultErrorEntry {
        pub id: String,
        pub sender_fault: bool,
        pub code: String,
        pub message: Option<String>,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    /// Response for a batch message send operation.
    ///
    /// Contains lists of successful and failed messages.
    pub struct SendMessageBatchResponse {
        pub successful: Vec<SendMessageBatchResultEntry>,
        pub failed: Vec<SendMessageBatchResultErrorEntry>,
    }
}

/// Types for the ListQueueTags API operation.
///
/// Lists all tags associated with a queue. Tags are key-value pairs
/// that can be used to categorize and organize queues.
pub mod list_queue_tags {
    use super::*;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    /// Request for listing tags on a queue.
    pub struct ListQueueTagsRequest {
        pub queue_url: Url,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    /// Response for listing tags on a queue.
    pub struct ListQueueTagsResponse {
        pub tags: HashMap<String, String>,
    }
}

/// Types for the TagQueue API operation.
///
/// Adds or updates tags on a queue. Tags are metadata that can be
/// attached to queues for organization and billing purposes.
pub mod tag_queue {
    use super::*;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    /// Request for adding tags to a queue
    pub struct TagQueueRequest {
        pub queue_url: Url,
        pub tags: HashMap<String, String>,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    /// Empty response for the TagQueue operation.
    pub struct TagQueueResponse {}
}

/// Types for the UntagQueue API operation.
///
/// Removes specified tags from a queue. Only the tag keys need to
/// be provided to remove the corresponding tags.
pub mod untag_queue {
    use super::*;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    /// Request for removing tags from a queue.
    pub struct UntagQueueRequest {
        pub queue_url: Url,
        pub tag_keys: Vec<String>,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    /// Empty response for the UntagQueue operation.
    pub struct UntagQueueResponse {}
}

/// Types for the SetQueueAttributes API operation.
///
/// Sets one or more attributes of a queue. Can modify settings like
/// message retention period, visibility timeout, and dead-letter queue
/// configuration.
pub mod set_queue_attributes {
    use crate::service::QueueAttributesSer;

    use super::*;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    /// Request for setting queue attributes.
    pub struct SetQueueAttributesRequest {
        pub queue_url: Url,
        pub attributes: QueueAttributesSer,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    /// Empty response for the SetQueueAttributes operation.
    pub struct SetQueueAttributesResponse {}
}

/// Types for the DeleteMessageBatch API operation.
///
/// Deletes multiple messages from a queue in a single request.
/// More efficient than deleting messages individually when processing
/// multiple messages. Supports up to 10 deletions per request.
pub mod delete_message_batch {
    use super::*;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    /// Entry for a batch message delete request.
    ///
    /// Each entry identifies a message to be deleted using its
    /// receipt handle and a client-provided ID for tracking.
    pub struct DeleteMessageBatchRequestEntry {
        pub id: String,
        pub receipt_handle: String,
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    /// Request for a batch message delete operation.
    ///
    /// Contains the queue URL and a list of message entries to delete.
    pub struct DeleteMessageBatchRequest {
        pub queue_url: Url,
        pub entries: Vec<DeleteMessageBatchRequestEntry>,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    /// Successful result entry for a batch message delete operation.
    ///
    /// Contains the ID of the successfully deleted message for correlation
    /// with the original request.
    pub struct DeleteMessageBatchResultSuccess {
        pub id: String,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    /// Error result entry for a batch message delete operation.
    ///
    /// Contains details about why a particular message in the batch
    /// failed to be deleted, including error code and message.
    pub struct DeleteMessageBatchResultError {
        pub code: String,
        pub id: String,
        pub message: String,
        pub sender_fault: bool,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    /// Response for a batch message delete operation.
    /// Contains lists of successful and failed messages.
    pub struct DeleteMessageBatchResponse {
        pub failed: Vec<DeleteMessageBatchResultError>,
        pub successful: Vec<DeleteMessageBatchResultSuccess>,
    }
}

/// Represents a message attribute in SQS format.
///
/// Message attributes can be one of three types:
/// - String: Text data
/// - Number: Numeric values stored as strings
/// - Binary: Raw binary data
///
/// This matches the AWS SQS message attribute format exactly for compatibility.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase", tag = "DataType")]
pub enum SqsMessageAttribute {
    String {
        #[serde(rename = "StringValue")]
        string_value: String,
    },
    Number {
        #[serde(rename = "StringValue")]
        string_value: String,
    },
    Binary {
        #[serde(rename = "BinaryValue")]
        binary_value: Vec<u8>,
    },
}

impl SqsMessageAttribute {
    pub fn data_type(&self) -> &'static str {
        match self {
            SqsMessageAttribute::String { .. } => "String",
            SqsMessageAttribute::Number { .. } => "Number",
            SqsMessageAttribute::Binary { .. } => "Binary",
        }
    }

    /// Serializes the attributes in the expected binary format for SQS attributes.
    ///
    /// [key length (4 bytes)][key bytes][type (1 byte)][value length (4 bytes)][value bytes]
    pub fn serialize(&self, key: &str) -> Vec<u8> {
        let mut buf = Vec::new();
        self.serialize_into(key, &mut buf);
        buf
    }

    /// Serializes the attributes in the expected binary format for SQS attributes, writing the
    /// results to the buffer specified in `buf`.
    ///
    /// [key length (4 bytes)][key bytes][type (1 byte)][value length (4 bytes)][value bytes]
    pub fn serialize_into(&self, key: &str, buf: &mut Vec<u8>) {
        let k_bytes = key.as_bytes();

        buf.put_u32(k_bytes.len() as u32);
        buf.put_slice(k_bytes);

        let t_bytes = self.data_type().as_bytes();
        buf.put_u32(t_bytes.len() as u32);
        buf.put_slice(t_bytes);

        match self {
            SqsMessageAttribute::String { string_value }
            | SqsMessageAttribute::Number { string_value } => {
                let v_bytes = string_value.as_bytes();
                buf.put_u8(1); // Type 1 is string (or number)

                buf.put_u32(v_bytes.len() as u32);
                buf.put_slice(v_bytes);
            }
            SqsMessageAttribute::Binary { binary_value } => {
                let v_bytes = binary_value.as_slice();
                buf.put_u8(2); // Type 2 is binary

                buf.put_u32(v_bytes.len() as u32);
                buf.put_slice(v_bytes);
            }
        };
    }
}

#[test]
fn test_sqs_message_attribute() {
    let attr = SqsMessageAttribute::String {
        string_value: "hello".to_string(),
    };
    let json = serde_json::to_string(&attr).unwrap();
    assert_eq!(json, r#"{"DataType":"String","StringValue":"hello"}"#);
    let attr = SqsMessageAttribute::Number {
        string_value: "123".to_string(),
    };
    let json = serde_json::to_string(&attr).unwrap();
    assert_eq!(json, r#"{"DataType":"Number","StringValue":"123"}"#);
    let attr = SqsMessageAttribute::Binary {
        binary_value: b"TEST".to_vec(),
    };
    let json = serde_json::to_string(&attr).unwrap();
    assert_eq!(json, r#"{"DataType":"Binary","BinaryValue":[84,69,83,84]}"#);

    let attr: SqsMessageAttribute =
        serde_json::from_str(r#"{"DataType":"String","StringValue":"hello"}"#).unwrap();
    assert!(matches!(attr, SqsMessageAttribute::String { .. }),);
}

/// Represents a message in SQS format.
///
/// Contains all the standard SQS message fields including:
/// - Message ID and receipt handle for tracking
/// - Message body and MD5 hash
/// - Standard attributes
/// - Custom message attributes
///
/// This structure is used when returning messages to clients in the
/// SQS-compatible API format.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SqsMessage {
    pub message_id: String,
    // pub receipt_handle: String,
    #[serde(rename = "MD5OfBody")]
    pub md5_of_body: String,
    pub body: String,

    // pub md5_of_system_attributes: String,
    pub attributes: HashMap<String, String>,

    #[serde(rename = "MD5OfMessageAttributes")]
    pub md5_of_message_attributes: String,
    pub message_attributes: HashMap<String, SqsMessageAttribute>,
}

/// Represents all possible SQS API response types.
///
/// This enum encompasses every possible response type that can be
/// returned from an SQS API operation. The serialization is untagged
/// to match the AWS SQS wire format.
///
/// Each variant corresponds to a specific API operation response,
/// maintaining compatibility with the AWS SQS API specification.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "PascalCase", untagged)]
pub enum SqsResponse {
    SendMessage(send_message::SendMessageResponse),
    GetQueueUrl(get_queue_url::GetQueueUrlResponse),
    CreateQueue(create_queue::CreateQueueResponse),
    ListQueues(list_queues::ListQueuesResponse),
    DeleteMessage(delete_message::DeleteMessageResponse),
    PurgeQueue(purge_queue::PurgeQueueResponse),
    DeleteQueue(delete_queue::DeleteQueueResponse),
    GetQueueAttributes(get_queue_attributes::GetQueueAttributesResponse),
    ReceiveMessage(receive_message::ReceiveMessageResponse),
    SendMessageBatch(send_message_batch::SendMessageBatchResponse),
    ListQueueTags(list_queue_tags::ListQueueTagsResponse),
    TagQueue(tag_queue::TagQueueResponse),
    UntagQueue(untag_queue::UntagQueueResponse),
    SetQueueAttributes(set_queue_attributes::SetQueueAttributesResponse),
    DeleteMessageBatch(delete_message_batch::DeleteMessageBatchResponse),
}
