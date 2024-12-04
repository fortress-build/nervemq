use std::collections::HashMap;
use url::Url;

pub mod send_message {
    use super::*;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
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
    pub struct SendMessageResponse {
        pub message_id: u64,
        pub md5_of_message_body: String,
        // pub md5_of_message_attributes: String,
        // pub sequence_number: Option<String>,
    }
}

pub mod get_queue_url {
    use super::*;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct GetQueueUrlRequest {
        pub queue_name: String,
        // pub queue_owner_aws_account_id: String,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct GetQueueUrlResponse {
        pub queue_url: Url,
    }
}

pub mod create_queue {
    use super::*;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct CreateQueueRequest {
        pub queue_name: String,
        #[serde(default)]
        pub attributes: HashMap<String, String>,
        #[serde(default)]
        pub tags: HashMap<String, String>,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct CreateQueueResponse {
        pub queue_url: Url,
    }
}

pub mod list_queues {
    use super::*;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct ListQueuesRequest {
        pub queue_name_prefix: Option<String>,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct ListQueuesResponse {
        pub queue_urls: Vec<Url>,
    }
}

pub mod delete_message {
    use super::*;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct DeleteMessageRequest {
        pub queue_url: Url,
        pub receipt_handle: String,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct DeleteMessageResponse {}
}

pub mod delete_queue {
    use super::*;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct DeleteQueueRequest {
        pub queue_url: Url,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct DeleteQueueResponse {}
}

pub mod purge_queue {
    use super::*;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct PurgeQueueRequest {
        pub queue_url: Url,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct PurgeQueueResponse {
        pub success: bool,
    }
}

pub mod get_queue_attributes {
    use super::*;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct GetQueueAttributesRequest {
        pub queue_url: Url,
        pub attribute_names: Vec<String>,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct GetQueueAttributesResponse {
        pub attributes: HashMap<String, String>,
    }
}

pub mod receive_message {
    use super::*;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct ReceiveMessageRequest {
        pub queue_url: Url,
        pub attribute_names: Vec<String>,
        pub message_attribute_names: Vec<String>,
        pub max_number_of_messages: u64,
        pub visibility_timeout: u64,
        pub wait_time_seconds: u64,
        pub receive_request_attempt_id: String,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct ReceiveMessageResponse {
        pub messages: Vec<SqsMessage>,
    }
}

pub mod send_message_batch {
    use super::*;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct SendMessageBatchRequest {
        pub queue_url: Url,
        pub entries: Vec<SendMessageBatchRequestEntry>,
    }

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct SendMessageBatchRequestEntry {
        pub id: String,
        pub message_body: String,
        pub delay_seconds: u64,
        pub message_attributes: HashMap<String, SqsMessageAttribute>,
        pub message_deduplication_id: String,
        pub message_group_id: String,
    }

    #[derive(Debug, serde::Serialize)]
    pub struct SendMessageBatchResultEntry {
        pub id: String,
        pub message_id: String,
        pub md5_of_message_body: String,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct SendMessageBatchResultErrorEntry {
        pub id: String,
        pub sender_fault: bool,
        pub code: String,
        pub message: String,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase", untagged)]
    pub enum SendMessageBatchResponse {
        Successful {
            successful: Vec<SendMessageBatchResultEntry>,
        },
        Failed {
            failed: Vec<SendMessageBatchResultErrorEntry>,
        },
    }
}

pub mod list_queue_tags {
    use super::*;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct ListQueueTagsRequest {
        pub queue_url: Url,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct ListQueueTagsResponse {
        pub tags: HashMap<String, String>,
    }
}

pub mod tag_queue {
    use super::*;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct TagQueueRequest {
        pub queue_url: Url,
        pub tags: HashMap<String, String>,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct TagQueueResponse {}
}

pub mod untag_queue {
    use super::*;

    #[derive(Debug, serde::Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct UntagQueueRequest {
        pub queue_url: Url,
        pub tag_keys: Vec<String>,
    }

    #[derive(Debug, serde::Serialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct UntagQueueResponse {}
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "PascalCase", tag = "DataType")]
pub enum SqsMessageAttribute {
    #[serde(rename_all = "PascalCase")]
    String { string_value: String },
    #[serde(rename_all = "PascalCase")]
    Number { string_value: String },
    #[serde(rename_all = "PascalCase")]
    Binary { binary_value: Vec<u8> },
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SqsMessage {
    pub message_id: String,
    pub md5_of_body: String,
    pub body: String,
}

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
}
