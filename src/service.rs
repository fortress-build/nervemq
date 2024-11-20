use std::collections::HashMap;

use actix_identity::{error::GetIdentityError, Identity};
use serde_email::Email;
use snafu::Snafu;
use sqlx::{
    sqlite::{
        SqliteAutoVacuum, SqliteConnectOptions, SqliteJournalMode, SqliteLockingMode,
        SqlitePoolOptions,
    },
    Acquire, Executor, Pool, Sqlite, SqlitePool,
};

use crate::{
    api::{
        admin::hash_password,
        auth::{Permission, Role, User},
    },
    config::Config,
    db::{
        message::Message,
        namespace::{Namespace, NamespaceStatistics},
        queue::{Queue, QueueStatistics},
    },
};

#[derive(Debug, Snafu)]
pub enum Error {
    Unauthorized,

    Sqlx {
        source: sqlx::Error,
    },

    #[snafu(whatever, display("{message}"))]
    Whatever {
        message: String,
        #[snafu(source(from(eyre::Report, Some)))]
        source: Option<eyre::Report>,
    },
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Self::Sqlx { source: value }
    }
}

impl From<eyre::Report> for Error {
    fn from(value: eyre::Report) -> Self {
        Error::Whatever {
            message: format!("{value}"),
            source: Some(value),
        }
    }
}

impl From<GetIdentityError> for Error {
    fn from(_: GetIdentityError) -> Self {
        Self::Unauthorized
    }
}

#[derive(Clone)]
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

    pub async fn list_namespaces(&self, identity: Identity) -> Result<Vec<Namespace>, Error> {
        let email = identity.id()?;

        Ok(sqlx::query_as(
            "
            SELECT ns.id, ns.name, nu.email as created_by FROM namespaces ns
            JOIN user_permissions p ON p.namespace = ns.id
            JOIN users u ON p.user = u.id
            JOIN users nu ON ns.created_by = nu.id
            WHERE u.email = $1
        ",
        )
        .bind(email)
        .fetch_all(&self.db)
        .await?)
    }

    pub async fn create_namespace(&self, name: &str, identity: Identity) -> Result<u64, Error> {
        let mut tx = self.db.begin().await?;

        let user_email = identity.id()?;
        let user: User = sqlx::query_as("SELECT * FROM users WHERE email = $1")
            .bind(&user_email)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| Error::Unauthorized)?;

        if user.role != Role::Admin {
            return Err(Error::Unauthorized);
        }

        let ns_id: u64 = sqlx::query_scalar(
            "INSERT INTO namespaces(name, created_by) VALUES ($1, $2) RETURNING id",
        )
        .bind(name)
        .bind(user.id as i64)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            "
            INSERT INTO user_permissions (user, namespace, can_delete_ns)
            VALUES ($1, $2, true)
        ",
        )
        .bind(user.id as i64)
        .bind(ns_id as i64)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(user.id)
    }

    pub async fn delete_namespace(&self, name: &str, identity: Identity) -> Result<(), Error> {
        let mut tx = self.db.begin().await?;

        let namespace = Namespace::get_id(tx.acquire().await?, name)
            .await?
            .ok_or_else(|| eyre::eyre!("Namespace {name} does not exist"))?;

        let (_user_id, can_delete) = self
            .check_user_access(&identity, namespace, tx.acquire().await?)
            .await?;

        if !can_delete {
            return Err(Error::Unauthorized);
        }

        Namespace::delete(tx.acquire().await?, name).await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn check_user_access<'a>(
        &self,
        identity: &Identity,
        ns: u64,
        executor: impl Executor<'a, Database = Sqlite>,
    ) -> Result<(u64, bool), Error> {
        let email = identity.id()?;

        let res: Option<Permission> = sqlx::query_as(
            "
            SELECT p.* FROM user_permissions p
            JOIN users u ON p.user = u.id
            WHERE u.email = $1 AND p.namespace = $2
        ",
        )
        .bind(email)
        .bind(ns as i64)
        .fetch_optional(executor)
        .await?;

        match res {
            Some(permission) => Ok((permission.user, permission.can_delete_ns)),
            None => Err(Error::Unauthorized),
        }
    }

    pub async fn get_queue_id<'a>(&self, namespace: &str, name: &str) -> Result<u64, Error> {
        Ok(sqlx::query_scalar(
            "
            SELECT q.id FROM queues q
            JOIN namespaces n ON q.ns = n.id
            WHERE n.name = $1 AND q.name = $2
            ",
        )
        .bind(namespace)
        .bind(name)
        .fetch_one(&self.db)
        .await?)
    }

    pub async fn create_queue(
        &self,
        namespace: &str,
        name: &str,
        identity: Identity,
    ) -> Result<(), Error> {
        let mut tx = self.db.begin().await?;

        let namespace = Namespace::get_id(tx.acquire().await?, namespace)
            .await?
            .ok_or_else(|| eyre::eyre!("Namespace {namespace} does not exist"))?;

        let (user_id, _) = self
            .check_user_access(&identity, namespace, &mut *tx)
            .await?;

        sqlx::query("INSERT INTO queues (ns, name, created_by) VALUES ($1, $2, $3)")
            .bind(namespace as i64)
            .bind(name)
            .bind(user_id as i64)
            .execute(tx.acquire().await?)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn delete_queue(
        &self,
        namespace: &str,
        name: &str,
        identity: Identity,
    ) -> eyre::Result<()> {
        let mut tx = self.db.begin().await?;

        let namespace_id = Namespace::get_id(tx.acquire().await?, namespace)
            .await?
            .ok_or_else(|| eyre::eyre!("Namespace {namespace} does not exist"))?;

        self.check_user_access(&identity, namespace_id, &mut *tx)
            .await?;

        Queue::delete(tx.acquire().await?, namespace, name).await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn list_queues(
        &self,
        namespace: Option<&str>,
        identity: Identity,
    ) -> Result<Vec<Queue>, Error> {
        let mut conn = self.db.acquire().await?;

        if let Some(namespace) = namespace {
            let namespace_id = Namespace::get_id(conn.acquire().await?, namespace)
                .await?
                .ok_or_else(|| eyre::eyre!("Namespace {namespace} does not exist"))?;

            self.check_user_access(&identity, namespace_id, &mut *conn)
                .await?;
        }

        Queue::list(conn.acquire().await?, namespace, identity).await
    }

    pub async fn create_user(
        &self,
        email: Email,
        password: String,
        role: Option<Role>,
        namespaces: Vec<String>,
    ) -> eyre::Result<()> {
        let hashed_password = hash_password(password).await?;

        let mut tx = self.db.begin().await?;

        let user_id: u64 = sqlx::query_scalar(
            "
            INSERT INTO users (email, hashed_pass, role)
            VALUES ($1, $2, $3)
            RETURNING id
        ",
        )
        .bind(email.as_str())
        .bind(hashed_password.to_string())
        .bind(role.unwrap_or(Role::User))
        .fetch_one(tx.acquire().await?)
        .await?;

        for namespace in namespaces {
            sqlx::query(
                "
                INSERT INTO user_permissions (user, namespace, can_delete_ns)
                VALUES ($1, (SELECT id FROM namespaces WHERE name = $2), false)
            ",
            )
            .bind(user_id as i64)
            .bind(namespace)
            .execute(tx.acquire().await?)
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    pub async fn send_message(
        &self,
        queue: u64,
        message: &[u8],
        kv: HashMap<String, String>,
    ) -> eyre::Result<u64> {
        let mut tx = self.db.begin().await?;

        let msg_id = Message::insert(tx.acquire().await?, queue, message, kv).await?;

        tx.commit().await?;

        Ok(msg_id)
    }

    pub async fn send_batch(
        &self,
        namespace: &str,
        queue: &str,
        messages: Vec<(Vec<u8>, HashMap<String, String>)>,
    ) -> eyre::Result<Vec<u64>> {
        let mut tx = self.db.begin().await?;

        // Get queue ID once for all messages
        let queue_id: u64 = sqlx::query_scalar(
            r#"
            SELECT q.id FROM queues q
            JOIN namespaces n ON q.ns = n.id
            WHERE n.name = $1 AND q.name = $2
            "#,
        )
        .bind(namespace)
        .bind(queue)
        .fetch_one(&mut *tx)
        .await?;

        let mut message_ids = Vec::with_capacity(messages.len());

        // Insert all messages in the batch
        for (message, kv) in messages {
            message_ids.push(Message::insert(tx.acquire().await?, queue_id, &message, kv).await?);
        }

        tx.commit().await?;

        Ok(message_ids)
    }

    pub async fn recv(
        &self,
        namespace: impl AsRef<str>,
        queue: impl AsRef<str>,
    ) -> eyre::Result<Option<Message>> {
        let mut tx = self.db.begin().await?;

        // Get the first undelivered message and mark it as delivered in one atomic operation
        let message: Option<Message> = sqlx::query_as(
            r#"
            WITH next_message AS (
                SELECT
                    m.id,
                    m.body,
                    m.delivered_at,
                    m.sent_by,
                    q.name as queue
                FROM messages m
                JOIN queues q ON m.queue = q.id
                JOIN namespaces n ON q.ns = n.id
                WHERE n.name = $1
                AND q.name = $2
                AND m.delivered_at IS NULL
                ORDER BY m.id ASC
                LIMIT 1
            )
            UPDATE messages
            SET delivered_at = unixepoch('now')
            WHERE id IN (SELECT id FROM next_message)
            RETURNING *
            "#,
        )
        .bind(namespace.as_ref())
        .bind(queue.as_ref())
        .fetch_optional(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(message)
    }

    pub async fn recv_batch(
        &self,
        namespace: &str,
        queue: &str,
        max_messages: usize,
    ) -> eyre::Result<Vec<Message>> {
        let mut tx = self.db.begin().await?;

        // Get multiple undelivered messages and mark them as delivered in one atomic operation
        let messages: Vec<Message> = sqlx::query_as(
            r#"
            WITH next_messages AS (
                SELECT
                    m.id,
                    m.body,
                    m.delivered_at,
                    m.sent_by,
                    q.name as queue_name
                FROM messages m
                JOIN queues q ON m.queue = q.id
                JOIN namespaces n ON q.ns = n.id
                WHERE n.name = $1
                AND q.name = $2
                AND m.delivered_at IS NULL
                ORDER BY m.id ASC
                LIMIT $3
            )
            UPDATE messages
            SET delivered_at = unixepoch('now')
            WHERE id IN (SELECT id FROM next_messages)
            RETURNING *, (SELECT queue_name FROM next_messages WHERE next_messages.id = messages.id) as queue
            "#,
        )
        .bind(namespace)
        .bind(queue)
        .bind(max_messages as i64)
        .fetch_all(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(messages)
    }

    pub async fn peek(&self, namespace: &str, queue: &str, message_id: u64) -> Option<Message> {
        let mut db = self.db.acquire().await.ok()?;

        sqlx::query_as(
            r#"
            SELECT m.* FROM messages m
            JOIN queues q ON m.queue = q.id
            JOIN namespaces n ON q.ns = n.id
            WHERE n.name = $1 AND q.name = $2 AND m.id = $3
            "#,
        )
        .bind(namespace)
        .bind(queue)
        .bind(message_id as i64)
        .fetch_optional(&mut *db)
        .await
        .ok()?
    }

    pub async fn list_messages(&self, namespace: &str, queue: &str) -> eyre::Result<Vec<Message>> {
        let mut db = self.db.acquire().await?;
        Message::list(db.acquire().await?, namespace, queue).await
    }

    pub async fn queue_statistics(
        &self,
        identity: Identity,
    ) -> Result<Vec<QueueStatistics>, Error> {
        let mut db = self.db.acquire().await?;
        Queue::statistics(db.acquire().await?, identity).await
    }

    pub async fn namespace_statistics(
        &self,
        identity: Identity,
    ) -> Result<Vec<NamespaceStatistics>, Error> {
        let email = identity.id()?;

        Ok(sqlx::query_as(
            "
            -- SELECT ns.*, nu.email as created_by, count(q.id) as queue_count FROM namespaces ns
            -- JOIN user_permissions p ON p.namespace = ns.id
            -- LEFT JOIN queues q ON q.ns = ns.id
            -- JOIN users u ON p.user = u.id
            -- JOIN users nu ON ns.created_by = nu.id
            -- WHERE u.email = $1
            SELECT 
                ns.*,
                nu.email as created_by,
                COUNT(q.id) as queue_count 
            FROM namespaces ns
            JOIN user_permissions p ON p.namespace = ns.id
            JOIN users u ON p.user = u.id
            JOIN users nu ON ns.created_by = nu.id
            LEFT JOIN queues q ON q.ns = ns.id
            WHERE u.email = 'admin@fortress.build'
            GROUP BY ns.id, nu.email
        ",
        )
        .bind(email)
        .fetch_all(&self.db)
        .await?)
    }
}
