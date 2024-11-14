use std::collections::HashMap;

use sqlx::{
    sqlite::{
        SqliteAutoVacuum, SqliteConnectOptions, SqliteJournalMode, SqliteLockingMode,
        SqlitePoolOptions,
    },
    Acquire, SqlitePool,
};

use crate::{
    config::Config,
    db::{message::Message, queue::Queue},
};

pub struct Service {
    db: SqlitePool,
    config: crate::config::Config,
}

impl Service {
    pub async fn connect() -> eyre::Result<Self> {
        Self::connect_with(Config::default()).await
    }

    pub async fn connect_with(config: Config) -> eyre::Result<Self> {
        let opts = if let Some(path) = &config.db_path() {
            SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(true)
        } else {
            SqliteConnectOptions::new().in_memory(true)
        }
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .locking_mode(SqliteLockingMode::Normal)
        .optimize_on_close(true, None)
        .auto_vacuum(SqliteAutoVacuum::Full);

        let pool = SqlitePoolOptions::new().connect_with(opts).await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { db: pool, config })
    }

    pub async fn enqueue(
        &self,
        namespace: impl AsRef<str>,
        queue: impl AsRef<str>,
        message: impl AsRef<[u8]>,
        kv: HashMap<String, String>,
    ) -> eyre::Result<()> {
        let mut tx = self.db.begin().await?;

        Message::insert(tx.acquire().await?, namespace, queue, message, kv).await?;

        tx.commit().await?;

        Ok(())
    }

    // pub async fn dequeue(&self, namespace: impl AsRef<str>, queue: impl AsRef<[u8]>) {
    //
    // }

    pub async fn create_queue(
        &self,
        namespace: impl AsRef<str>,
        name: impl AsRef<str>,
    ) -> eyre::Result<()> {
        let mut tx = self.db.begin().await?;

        Queue::insert(tx.acquire().await?, namespace, name).await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn delete_queue(
        &self,
        namespace: impl AsRef<str>,
        name: impl AsRef<str>,
    ) -> eyre::Result<()> {
        let mut tx = self.db.begin().await?;

        Queue::delete(tx.acquire().await?, namespace, name).await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn list_queues(
        &self,
        namespace: Option<impl AsRef<str>>,
    ) -> eyre::Result<Vec<Queue>> {
        let mut conn = self.db.acquire().await?;
        Queue::list(conn.acquire().await?, namespace).await
    }
}
