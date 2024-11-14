use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, SqliteConnection};
use tokio_stream::StreamExt;

use super::namespace::Namespace;

#[derive(Serialize, Deserialize, FromRow)]
pub struct Queue {
    id: u64,
    ns: String,
    name: String,
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

    pub async fn delete(
        db: &mut SqliteConnection,
        namespace: impl AsRef<str>,
        name: impl AsRef<str>,
    ) -> eyre::Result<()> {
        let namespace = Namespace::ensure(db, namespace).await?;

        sqlx::query("DELETE FROM queues q JOIN namespaces n ON q.ns = n.id WHERE n.name = $1 AND q.name = $2")
            .bind(namespace as i64)
            .bind(name.as_ref())
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
        namespace: impl AsRef<str>,
    ) -> eyre::Result<Vec<Queue>> {
        let mut stream = sqlx::query_as(
            "SELECT q.id, q.name, n.name as ns FROM queues q JOIN namespaces n ON q.ns = n.id WHERE n.name = $1",
        )
        .bind(namespace.as_ref())
        .fetch(db);

        let mut queues = Vec::new();

        while let Some(res) = stream.next().await.transpose()? {
            queues.push(res);
        }

        Ok(queues)
    }

    pub async fn list(
        db: &mut SqliteConnection,
        namespace: Option<impl AsRef<str>>,
    ) -> eyre::Result<Vec<Queue>> {
        match namespace {
            Some(ns) => Self::list_for_namespace(db, ns.as_ref()).await,
            None => Self::list_all(db).await,
        }
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
