use std::{collections::HashMap, hash::Hash};

use serde::{Deserialize, Serialize};
use sqlx::{Acquire, Database, Executor, Sqlite, SqliteConnection, SqlitePool, Statement};

#[derive(Serialize, Deserialize)]
pub struct Namespace {
    id: u64,
    name: String,
}

impl Namespace {
    pub async fn get_id<'a>(
        db: &mut SqliteConnection,
        name: impl AsRef<str>,
    ) -> eyre::Result<Option<u64>> {
        sqlx::query_scalar("SELECT id FROM namespaces WHERE name = $1")
            .bind(name.as_ref())
            .fetch_optional(db)
            .await
            .map_err(|e| eyre::eyre!(e))
    }

    pub async fn ensure<'a>(db: &mut SqliteConnection, name: impl AsRef<str>) -> eyre::Result<u64> {
        if let Some(ns) = Self::get_id(db, &name).await? {
            return Ok(ns);
        }

        sqlx::query("INSERT INTO namespaces(name) VALUES ($1)")
            .bind(name.as_ref())
            .execute(&mut *db)
            .await
            .map_err(|e| eyre::eyre!(e))?;

        sqlx::query_scalar("SELECT id FROM namespaces WHERE name = $1")
            .bind(name.as_ref())
            .fetch_one(&mut *db)
            .await
            .map_err(|e| eyre::eyre!(e))
    }
}

#[derive(Serialize, Deserialize)]
pub struct Queue {
    id: u64,
    ns: u64,
    name: String,
    messages: Vec<Message>,
}

impl Queue {
    pub async fn insert(
        db: &mut SqliteConnection,
        namespace: impl AsRef<str>,
        name: impl AsRef<str>,
    ) -> eyre::Result<()> {
        let namespace = Namespace::ensure(db, namespace).await?;

        sqlx::query("INSERT INTO queues (ns, name) VALUES ($1, $2)")
            .bind(namespace as i64)
            .bind(name.as_ref())
            .execute(db)
            .await?;

        Ok(())
    }

    pub async fn get_id(
        db: &mut SqliteConnection,
        namespace: impl AsRef<str>,
        queue: impl AsRef<str>,
    ) -> eyre::Result<u64> {
        Ok(sqlx::query_scalar("SELECT q.id FROM namespaces AS n INNER JOIN queues as q ON q.ns = n.id WHERE n.name = $1 AND q.name = $2")
            .bind(namespace.as_ref())
            .bind(queue.as_ref())
            .fetch_one(db)
        .await?)
    }
}

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
