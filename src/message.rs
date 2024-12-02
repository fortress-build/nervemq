use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Serialize, Deserialize, Debug, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum MessageStatus {
    #[serde(rename = "pending")]
    #[sqlx(rename = "pending")]
    Pending,
    #[serde(rename = "delivered")]
    #[sqlx(rename = "delivered")]
    Delivered,
    #[serde(rename = "failed")]
    #[sqlx(rename = "failed")]
    Failed,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: u64,
    pub queue: String,

    pub delivered_at: Option<u64>,
    pub sent_by: Option<u64>,
    pub body: Vec<u8>,
    pub tries: u64,

    pub status: MessageStatus,

    #[sqlx(skip)]
    pub kv: HashMap<String, String>,
}
