use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: u64,
    pub queue: String,

    pub delivered_at: Option<u64>,
    pub sent_by: Option<u64>,
    pub body: Vec<u8>,

    #[sqlx(skip)]
    pub kv: HashMap<String, String>,
}
