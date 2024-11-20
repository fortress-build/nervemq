use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqliteConnection};

#[derive(Serialize, Deserialize, FromRow, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Namespace {
    pub id: u64,
    pub name: String,
    pub created_by: String,
}

#[derive(Serialize, Deserialize, FromRow, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NamespaceStatistics {
    pub id: u64,
    pub name: String,
    pub created_by: String,
    pub queue_count: u64,
}

impl Namespace {
    pub async fn get_id(db: &mut SqliteConnection, name: &str) -> eyre::Result<Option<u64>> {
        sqlx::query_scalar(
            "
            SELECT id FROM namespaces WHERE name = $1
        ",
        )
        .bind(name)
        .fetch_optional(db)
        .await
        .map_err(|e| eyre::eyre!(e))
    }

    pub async fn delete(db: &mut SqliteConnection, name: &str) -> eyre::Result<()> {
        sqlx::query(
            "
            DELETE FROM namespaces WHERE name = $1
        ",
        )
        .bind(name)
        .execute(&mut *db)
        .await
        .map(|_| ())
        .map_err(|e| eyre::eyre!(e))
    }

    pub async fn list(db: &mut SqliteConnection) -> eyre::Result<Vec<Namespace>> {
        Ok(sqlx::query_as(
            "
            SELECT ns.*, count(q.id) as queue_count FROM namespaces ns
            LEFT JOIN queues q ON q.ns = ns.id
        ",
        )
        .fetch_all(db)
        .await?)
    }
}
