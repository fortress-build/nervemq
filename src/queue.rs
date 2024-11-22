use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Serialize, Deserialize, FromRow, Debug)]
pub struct Queue {
    pub id: u64,
    pub ns: String,
    pub name: String,
    pub created_by: String,
}

impl PartialEq for Queue {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Serialize, Deserialize, FromRow, Debug)]
pub struct QueueStatistics {
    #[serde(flatten)]
    #[sqlx(flatten)]
    pub queue: Queue,
    pub message_count: u64,
    pub avg_size_bytes: f64,
}
