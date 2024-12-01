use std::{collections::HashMap, sync::Arc};

use actix_identity::{error::GetIdentityError, Identity};
use actix_web::web;
use secrecy::SecretString;
use serde_email::Email;
use snafu::Snafu;
use sqlx::{
    sqlite::{
        SqliteAutoVacuum, SqliteConnectOptions, SqliteJournalMode, SqliteLockingMode,
        SqlitePoolOptions,
    },
    Acquire, Sqlite, SqlitePool,
};
use tokio_stream::StreamExt as _;

use crate::{
    api::auth::{Permission, Role, User},
    auth::crypto::hash_secret,
    config::Config,
    message::Message,
    namespace::{Namespace, NamespaceStatistics},
    queue::{Queue, QueueStatistics},
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
    config: Arc<crate::config::Config>,
}

impl Service {
    pub fn db(&self) -> &SqlitePool {
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

        Ok(Self {
            db: pool,
            config: Arc::new(config),
        })
    }

    pub async fn get_queue_id(
        &self,
        namespace: &str,
        name: &str,
        exec: impl Acquire<'_, Database = Sqlite>,
    ) -> Result<u64, Error> {
        Ok(sqlx::query_scalar(
            "
            SELECT q.id FROM queues q
            JOIN namespaces n ON q.ns = n.id
            WHERE n.name = $1 AND q.name = $2
            ",
        )
        .bind(namespace)
        .bind(name)
        .fetch_one(&mut *exec.acquire().await?)
        .await?)
    }

    pub async fn get_namespace_id<'a>(
        &self,
        name: &str,
        ex: impl Acquire<'a, Database = Sqlite>,
    ) -> eyre::Result<Option<u64>> {
        sqlx::query_scalar(
            "
                    SELECT id FROM namespaces WHERE name = $1
                ",
        )
        .bind(name)
        .fetch_optional(&mut *ex.acquire().await?)
        .await
        .map_err(|e| eyre::eyre!(e))
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
        .fetch_all(&mut *self.db.acquire().await?)
        .await?)
    }

    pub async fn create_namespace(&self, name: &str, identity: Identity) -> Result<u64, Error> {
        let mut tx = self.db().begin().await?;

        let user_email = identity.id()?;

        let user: User = sqlx::query_as("SELECT * FROM users WHERE email = $1")
            .bind(&user_email)
            .fetch_optional(&mut *tx.acquire().await?)
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
        .fetch_one(&mut *tx.as_mut().acquire().await?)
        .await?;

        sqlx::query(
            "
            INSERT INTO user_permissions (user, namespace, can_delete_ns)
            VALUES ($1, $2, true)
        ",
        )
        .bind(user.id as i64)
        .bind(ns_id as i64)
        .execute(&mut *tx.as_mut().acquire().await?)
        .await?;

        tx.commit().await?;

        Ok(user.id)
    }

    pub async fn delete_namespace(&self, name: &str, identity: Identity) -> Result<(), Error> {
        let mut tx = self.db().begin().await?;

        let namespace = self
            .get_namespace_id(name, &mut tx)
            .await?
            .ok_or_else(|| eyre::eyre!("Namespace {name} does not exist"))?;

        let (_user_id, can_delete) = self
            .check_user_access(&identity, namespace, &mut tx)
            .await?;

        if !can_delete {
            return Err(Error::Unauthorized);
        }

        sqlx::query(
            "
            DELETE FROM namespaces WHERE name = $1
        ",
        )
        .bind(name)
        .execute(&mut *tx)
        .await
        .map(|_| ())?;

        tx.commit().await?;

        Ok(())
    }

    async fn check_user_access<'a>(
        &self,
        identity: &Identity,
        ns: u64,
        exec: impl Acquire<'_, Database = Sqlite>,
    ) -> Result<(u64, bool), Error> {
        let email = identity.id()?;
        let mut db = exec.acquire().await?;

        let res: Option<Permission> = sqlx::query_as(
            "
            SELECT p.* FROM user_permissions p
            JOIN users u ON p.user = u.id
            WHERE u.email = $1 AND p.namespace = $2
        ",
        )
        .bind(email)
        .bind(ns as i64)
        .fetch_optional(&mut *db)
        .await?;

        match res {
            Some(permission) => Ok((permission.user, permission.can_delete_ns)),
            None => Err(Error::Unauthorized),
        }
    }

    pub async fn create_queue(
        &self,
        namespace: &str,
        name: &str,
        identity: Identity,
    ) -> Result<(), Error> {
        let mut tx = self.db().begin().await?;

        let namespace = self
            .get_namespace_id(namespace, &mut tx)
            .await?
            .ok_or_else(|| eyre::eyre!("Namespace {namespace} does not exist"))?;

        let (user_id, _) = self
            .check_user_access(&identity, namespace, &mut tx)
            .await?;

        let queue_id: u64 = sqlx::query_scalar(
            "
            INSERT INTO queues (ns, name, created_by)
            VALUES ($1, $2, $3)
            RETURNING id
        ",
        )
        .bind(namespace as i64)
        .bind(name)
        .bind(user_id as i64)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            "
            INSERT INTO queue_configurations (queue, max_retries)
            VALUES ($1, $2)
        ",
        )
        .bind(queue_id as i64)
        .bind(self.config.default_max_retries() as i64)
        .execute(&mut *tx)
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
        let mut tx = self.db().begin().await?;

        let namespace_id = self
            .get_namespace_id(namespace, &mut tx)
            .await?
            .ok_or_else(|| eyre::eyre!("Namespace {namespace} does not exist"))?;

        self.check_user_access(&identity, namespace_id, &mut tx)
            .await?;

        let id = self.get_queue_id(namespace, name, &mut tx).await?;

        sqlx::query("DELETE FROM queues WHERE id = $1")
            .bind(id as i64)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn list_queues(
        &self,
        namespace: Option<&str>,
        identity: Identity,
    ) -> Result<Vec<Queue>, Error> {
        let mut conn = self.db().acquire().await?;

        if let Some(namespace) = namespace {
            let namespace_id = self
                .get_namespace_id(namespace, &mut *conn)
                .await?
                .ok_or_else(|| eyre::eyre!("Namespace {namespace} does not exist"))?;

            self.check_user_access(&identity, namespace_id, &mut *conn)
                .await?;
        }

        // Queue::list(conn.acquire().await?, namespace, identity).await

        match namespace {
            Some(ns) => self.list_queues_for_namespace(ns).await,
            None => self.list_all_queues(identity).await,
        }
    }

    pub async fn list_queues_for_namespace(&self, namespace: &str) -> Result<Vec<Queue>, Error> {
        let mut db = self.db().acquire().await?;
        let mut stream = sqlx::query_as(
            "
            SELECT q.id, q.name, n.name as ns, u.email as created_by FROM queues q
            JOIN namespaces n ON q.ns = n.id
            JOIN users u on q.created_by = u.id
            WHERE n.name = $1",
        )
        .bind(namespace)
        .fetch(&mut *db);

        let mut queues = Vec::new();

        while let Some(res) = stream.next().await.transpose()? {
            queues.push(res);
        }

        Ok(queues)
    }

    pub async fn list_all_queues(&self, identity: Identity) -> Result<Vec<Queue>, Error> {
        let email = identity.id()?;

        let queues = sqlx::query_as(
            "
            SELECT q.id, q.name, qu.email as created_by, n.name as ns FROM queues q
            JOIN user_permissions p ON p.namespace = q.ns
            JOIN namespaces n ON n.id = q.ns
            JOIN users u ON u.id = p.user
            JOIN users qu ON q.id = q.created_by
            WHERE u.email = $1
            ",
        )
        .bind(email)
        .fetch_all(&mut *self.db().acquire().await?)
        .await?;

        Ok(queues)
    }

    pub async fn create_user(
        &self,
        email: Email,
        password: SecretString,
        role: Option<Role>,
        namespaces: Vec<String>,
    ) -> eyre::Result<()> {
        let hashed_password = web::block(move || hash_secret(password)).await??;

        let mut tx = self.db().begin().await?;

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
        .fetch_one(&mut *tx.acquire().await?)
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
        let mut tx = self.db().begin().await?;

        let msg_id: u64 =
            sqlx::query_scalar("INSERT INTO messages (queue, body) VALUES ($1, $2) RETURNING id")
                .bind(queue as i64)
                .bind(message)
                .fetch_one(&mut *tx)
                .await?;

        for (k, v) in kv.into_iter() {
            sqlx::query("INSERT INTO kv_pairs (message, k, v) VALUES ($1, $2, $3)")
                .bind(msg_id as i64)
                .bind(k)
                .bind(v)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;

        Ok(msg_id)
    }

    pub async fn send_batch(
        &self,
        namespace: &str,
        queue: &str,
        messages: Vec<(Vec<u8>, HashMap<String, String>)>,
    ) -> eyre::Result<Vec<u64>> {
        let mut tx = self.db().begin().await?;

        // Get queue ID once for all messages
        let queue_id: u64 = sqlx::query_scalar(
            "
            SELECT q.id FROM queues q
            JOIN namespaces n ON q.ns = n.id
            WHERE n.name = $1 AND q.name = $2
            ",
        )
        .bind(namespace)
        .bind(queue)
        .fetch_one(&mut *tx)
        .await?;

        let mut message_ids = Vec::with_capacity(messages.len());

        // Insert all messages in the batch
        for (message, kv) in messages {
            let msg_id: u64 = sqlx::query_scalar(
                "INSERT INTO messages (queue, body) VALUES ($1, $2) RETURNING id",
            )
            .bind(queue_id as i64)
            .bind(&message)
            .fetch_one(&mut *tx)
            .await?;

            for (k, v) in kv.into_iter() {
                sqlx::query("INSERT INTO kv_pairs (message, k, v) VALUES ($1, $2, $3)")
                    .bind(msg_id as i64)
                    .bind(k)
                    .bind(v)
                    .execute(&mut *tx)
                    .await?;
            }

            message_ids.push(msg_id);
        }

        tx.commit().await?;

        Ok(message_ids)
    }

    pub async fn recv(
        &self,
        namespace: impl AsRef<str>,
        queue: impl AsRef<str>,
    ) -> eyre::Result<Option<Message>> {
        let mut tx = self.db().begin().await?;

        // Get the first undelivered message and mark it as delivered in one atomic operation
        let message: Option<Message> = sqlx::query_as(
            "
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
            ",
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
        let mut tx = self.db().begin().await?;

        // Get multiple undelivered messages and mark them as delivered in one atomic operation
        let messages: Vec<Message> = sqlx::query_as(
            "
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
            ",
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
        let mut db = self.db().acquire().await.ok()?;

        sqlx::query_as(
            "
            SELECT m.* FROM messages m
            JOIN queues q ON m.queue = q.id
            JOIN namespaces n ON q.ns = n.id
            WHERE n.name = $1 AND q.name = $2 AND m.id = $3
            ",
        )
        .bind(namespace)
        .bind(queue)
        .bind(message_id as i64)
        .fetch_optional(&mut *db)
        .await
        .ok()?
    }

    pub async fn list_messages(&self, namespace: &str, queue: &str) -> eyre::Result<Vec<Message>> {
        let mut db = self.db().acquire().await?;

        Ok(sqlx::query_as::<_, Message>(
            "
            SELECT m.*, q.name as queue FROM messages m
            JOIN queues q ON m.queue = q.id
        ",
        )
        .bind(namespace)
        .bind(queue)
        .fetch_all(&mut *db)
        .await?)
    }

    pub async fn queue_statistics(
        &self,
        identity: Identity,
        namespace: &str,
        queue: &str,
    ) -> Result<QueueStatistics, Error> {
        let mut db = self.db().acquire().await?;
        let email = identity.id()?;

        Ok(sqlx::query_as(
            "
            SELECT
                q.id,
                q.name,
                qu.email as created_by,
                n.name as ns,
                COUNT(m.id) AS message_count,
                SUM(LENGTH(m.body)) / CAST(COUNT(m.id) AS FLOAT) as avg_size_bytes
            FROM queues q
            LEFT JOIN messages m ON q.id = m.queue
            JOIN user_permissions p ON p.namespace = q.ns
            JOIN namespaces n ON n.id = q.ns
            JOIN users u ON u.id = p.user
            JOIN users qu ON q.created_by = qu.id
            WHERE u.email = $1 AND n.name = $2 AND q.name = $3
        ",
        )
        .bind(email)
        .bind(namespace)
        .bind(queue)
        .fetch_one(&mut *db)
        .await?)
    }

    pub async fn global_queue_statistics(
        &self,
        identity: Identity,
    ) -> Result<HashMap<String, QueueStatistics>, Error> {
        let mut db = self.db().acquire().await?;
        let email = identity.id()?;

        let res = sqlx::query_as(
            "
            SELECT
                q.id,
                q.name,
                qu.email as created_by,
                n.name as ns,
                COUNT(m.id) AS message_count,
                SUM(LENGTH(m.body)) / CAST(COUNT(m.id) AS FLOAT) as avg_size_bytes
            FROM queues q
            LEFT JOIN messages m ON q.id = m.queue
            JOIN user_permissions p ON p.namespace = q.ns
            JOIN namespaces n ON n.id = q.ns
            JOIN users u ON u.id = p.user
            JOIN users qu ON q.created_by = qu.id
            WHERE u.email = $1
            GROUP BY q.id, q.name
        ",
        )
        .bind(email)
        .fetch_all(&mut *db)
        .await?
        .into_iter()
        .map(|row: QueueStatistics| (row.queue.name.clone(), row))
        .collect::<HashMap<_, _>>();

        Ok(res)
    }

    pub async fn list_namespace_statistics(
        &self,
        identity: Identity,
    ) -> Result<Vec<NamespaceStatistics>, Error> {
        let email = identity.id()?;

        Ok(sqlx::query_as(
            "
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
        .fetch_all(&mut *self.db().acquire().await?)
        .await?)
    }
}
