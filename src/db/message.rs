use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sqlx::SqliteConnection;

use super::queue::Queue;

#[derive(Serialize, Deserialize)]
pub struct Message {
    id: u64,
    queue: u64,

    delivered_at: u64,

    body: Vec<u8>,

    kv: HashMap<String, String>,
}

impl Message {
    pub async fn insert(
        db: &mut SqliteConnection,
        namespace: impl AsRef<str>,
        queue: impl AsRef<str>,
        body: impl AsRef<[u8]>,
        kv: HashMap<String, String>,
    ) -> eyre::Result<()> {
        let queue_id = Queue::get_id(&mut *db, namespace, queue).await?;

        let msg_id: i64 =
            sqlx::query_scalar("INSERT INTO messages (queue, body) VALUES ($1, $2) RETURNING id")
                .bind(queue_id as i64)
                .bind(body.as_ref())
                .fetch_one(&mut *db)
                .await?;

        for (k, v) in kv.into_iter() {
            sqlx::query("INSERT INTO kv_pairs (message, k, v) VALUES ($1, $2, $3)")
                .bind(msg_id)
                .bind(k)
                .bind(v)
                .execute(&mut *db)
                .await?;
        }

        Ok(())
    }
}
