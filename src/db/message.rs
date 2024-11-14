use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, SqliteConnection};

use super::queue::Queue;

#[derive(Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: u64,
    pub queue: String,

    pub delivered_at: u64,

    pub body: Vec<u8>,

    #[sqlx(skip)]
    pub kv: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, FromRow)]
struct MessageNoKv {
    id: u64,
    queue: String,

    delivered_at: u64,

    body: Vec<u8>,
}

impl Message {
    pub async fn insert(
        db: &mut SqliteConnection,
        namespace: &str,
        queue: &str,
        body: &[u8],
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

    pub async fn list(
        db: &mut SqliteConnection,
        namespace: impl AsRef<str>,
        queue: impl AsRef<str>,
    ) -> eyre::Result<Vec<Message>> {
        Ok(sqlx::query_as::<_, Message>(
            "
            SELECT m.*, q.name as queue FROM messages m
            JOIN queues q ON m.queue = q.id
        ",
        )
        .bind(namespace.as_ref())
        .bind(queue.as_ref())
        .fetch_all(db)
        .await?)
    }
}
