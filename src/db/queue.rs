use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, SqliteConnection};
use tokio_stream::StreamExt;

use super::namespace::Namespace;

#[derive(Serialize, Deserialize, FromRow, Debug)]
pub struct Queue {
    pub id: u64,
    pub ns: String,
    pub name: String,
}

impl PartialEq for Queue {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Serialize, Deserialize, FromRow, Debug)]
pub struct QueueStatistics {
    id: u64,
    ns: String,
    name: String,
    message_count: u64,
}

impl Queue {
    pub async fn insert(
        db: &mut SqliteConnection,
        namespace: &str,
        name: &str,
    ) -> eyre::Result<()> {
        let namespace = Namespace::ensure(db, namespace).await?;

        sqlx::query("INSERT INTO queues (ns, name) VALUES ($1, $2)")
            .bind(namespace as i64)
            .bind(name)
            .execute(db)
            .await?;

        Ok(())
    }

    pub async fn delete(
        db: &mut SqliteConnection,
        namespace: &str,
        name: &str,
    ) -> eyre::Result<()> {
        let id = Queue::get_id(db, namespace, name).await?;

        sqlx::query("DELETE FROM queues WHERE id = $1")
            .bind(id as i64)
            .execute(db)
            .await?;

        Ok(())
    }

    async fn list_all(db: &mut SqliteConnection) -> eyre::Result<Vec<Queue>> {
        let mut stream = sqlx::query_as(
            "SELECT q.id, q.name, n.name as ns FROM queues q JOIN namespaces n ON q.ns = n.id",
        )
        .fetch(db);

        let mut queues = Vec::new();

        while let Some(res) = stream.next().await.transpose()? {
            queues.push(res);
        }

        Ok(queues)
    }

    async fn list_for_namespace(
        db: &mut SqliteConnection,
        namespace: &str,
    ) -> eyre::Result<Vec<Queue>> {
        let mut stream = sqlx::query_as(
            "SELECT q.id, q.name, n.name as ns FROM queues q JOIN namespaces n ON q.ns = n.id WHERE n.name = $1",
        )
        .bind(namespace)
        .fetch(db);

        let mut queues = Vec::new();

        while let Some(res) = stream.next().await.transpose()? {
            queues.push(res);
        }

        Ok(queues)
    }

    pub async fn statistics(db: &mut SqliteConnection) -> eyre::Result<Vec<QueueStatistics>> {
        let res = sqlx::query_as(
            "
            SELECT
                q.id   AS id,
                n.name as ns,
                q.name AS name,
                COUNT(m.id) AS message_count
            FROM queues q
            LEFT JOIN messages m ON q.id = m.queue
            LEFT JOIN namespaces n ON q.ns = n.id
            GROUP BY q.id, q.name;
        ",
        )
        .fetch_all(db)
        .await?;

        Ok(res)
    }

    pub async fn list(
        db: &mut SqliteConnection,
        namespace: Option<&str>,
    ) -> eyre::Result<Vec<Queue>> {
        match namespace {
            Some(ns) => Self::list_for_namespace(db, ns).await,
            None => Self::list_all(db).await,
        }
    }

    pub async fn get_id(
        db: &mut SqliteConnection,
        namespace: &str,
        queue: &str,
    ) -> eyre::Result<u64> {
        Ok(sqlx::query_scalar("SELECT q.id FROM namespaces AS n INNER JOIN queues as q ON q.ns = n.id WHERE n.name = $1 AND q.name = $2")
            .bind(namespace)
            .bind(queue)
            .fetch_one(db)
        .await?)
    }
}
