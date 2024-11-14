use serde::{Deserialize, Serialize};
use sqlx::SqliteConnection;

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
