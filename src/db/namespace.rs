use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqliteConnection};

#[derive(Serialize, Deserialize, FromRow, PartialEq, Debug)]
pub struct Namespace {
    pub id: u64,
    pub name: String,
}

impl Namespace {
    pub async fn get_id(db: &mut SqliteConnection, name: &str) -> eyre::Result<Option<u64>> {
        sqlx::query_scalar("SELECT id FROM namespaces WHERE name = $1")
            .bind(name)
            .fetch_optional(db)
            .await
            .map_err(|e| eyre::eyre!(e))
    }

    pub async fn insert(db: &mut SqliteConnection, name: &str) -> eyre::Result<u64> {
        sqlx::query_scalar("INSERT INTO namespaces(name) VALUES ($1) RETURNING id")
            .bind(name)
            .fetch_one(&mut *db)
            .await
            .map_err(|e| eyre::eyre!(e))
    }

    pub async fn delete(db: &mut SqliteConnection, name: &str) -> eyre::Result<()> {
        sqlx::query("DELETE FROM namespaces WHERE name = $1")
            .bind(name)
            .execute(&mut *db)
            .await
            .map(|_| ())
            .map_err(|e| eyre::eyre!(e))
    }

    pub async fn ensure(db: &mut SqliteConnection, name: &str) -> eyre::Result<u64> {
        if let Some(ns) = Self::get_id(db, &name).await? {
            return Ok(ns);
        }

        Self::insert(db, name).await
    }

    pub async fn list(db: &mut SqliteConnection) -> eyre::Result<Vec<Namespace>> {
        Ok(sqlx::query_as("SELECT * FROM namespaces")
            .fetch_all(db)
            .await?)
    }
}
