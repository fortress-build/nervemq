use std::collections::HashMap;

use serde_email::Email;
use sqlx::{
    sqlite::{
        SqliteAutoVacuum, SqliteConnectOptions, SqliteJournalMode, SqliteLockingMode,
        SqlitePoolOptions,
    },
    Acquire, Pool, Sqlite, SqlitePool,
};

use crate::{
    api::{admin::hash_password, auth::Role},
    config::Config,
    db::{
        message::Message,
        namespace::Namespace,
        queue::{Queue, QueueStatistics},
    },
};

pub struct Service {
    db: SqlitePool,
    #[allow(unused)]
    config: crate::config::Config,
}

impl Service {
    pub fn db(&self) -> &Pool<Sqlite> {
        &self.db
    }

    pub async fn connect() -> eyre::Result<Self> {
        Self::connect_with(Config::default()).await
    }

    pub async fn connect_with(config: Config) -> eyre::Result<Self> {
        let opts = SqliteConnectOptions::new()
            .filename(config.db_path())
            .create_if_missing(true)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal)
            .locking_mode(SqliteLockingMode::Normal)
            .optimize_on_close(true, None)
            .auto_vacuum(SqliteAutoVacuum::Full);

        let pool = SqlitePoolOptions::new().connect_with(opts).await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { db: pool, config })
    }

    pub async fn list_namespaces(&self) -> eyre::Result<Vec<Namespace>> {
        let mut db = self.db.acquire().await?;
        Namespace::list(db.acquire().await?).await
    }

    pub async fn create_namespace(&self, name: &str) -> eyre::Result<u64> {
        let mut db = self.db.acquire().await?;
        Namespace::insert(db.acquire().await?, name).await
    }

    pub async fn delete_namespace(&self, name: &str) -> eyre::Result<()> {
        let mut db = self.db.acquire().await?;
        Namespace::delete(db.acquire().await?, name).await
    }

    pub async fn create_queue(&self, namespace: &str, name: &str) -> eyre::Result<()> {
        let mut tx = self.db.begin().await?;

        Queue::insert(tx.acquire().await?, namespace, name).await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn delete_queue(&self, namespace: &str, name: &str) -> eyre::Result<()> {
        let mut tx = self.db.begin().await?;

        Queue::delete(tx.acquire().await?, namespace, name).await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn list_queues(&self, namespace: Option<&str>) -> eyre::Result<Vec<Queue>> {
        let mut conn = self.db.acquire().await?;
        Queue::list(conn.acquire().await?, namespace).await
    }

    pub async fn create_user(
        &self,
        email: Email,
        password: String,
        role: Option<Role>,
    ) -> eyre::Result<()> {
        let hashed_password = hash_password(password).await?;

        sqlx::query("INSERT INTO users (email, hashed_pass, role) VALUES ($1, $2, $3)")
            .bind(email.as_str())
            .bind(hashed_password.to_string())
            .bind(role.unwrap_or(Role::User))
            .execute(self.db())
            .await?;

        Ok(())
    }

    pub async fn send_message(
        &self,
        namespace: &str,
        queue: &str,
        message: &[u8],
        kv: HashMap<String, String>,
    ) -> eyre::Result<()> {
        let mut tx = self.db.begin().await?;

        Message::insert(tx.acquire().await?, namespace, queue, message, kv).await?;

        tx.commit().await?;

        Ok(())
    }

    // pub async fn recv_message(&self, namespace: impl AsRef<str>, queue: impl AsRef<[u8]>) {
    //
    // }

    pub async fn list_messages(&self, namespace: &str, queue: &str) -> eyre::Result<Vec<Message>> {
        let mut db = self.db.acquire().await?;
        Message::list(db.acquire().await?, namespace, queue).await
    }

    pub async fn statistics(&self) -> eyre::Result<Vec<QueueStatistics>> {
        let mut db = self.db.acquire().await?;
        Queue::statistics(db.acquire().await?).await
    }
}
