use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use actix_identity::Identity;
use actix_web::web;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use serde_email::Email;
use sqlx::{
    sqlite::{
        SqliteAutoVacuum, SqliteConnectOptions, SqliteJournalMode, SqliteLockingMode,
        SqlitePoolOptions, SqliteRow,
    },
    Acquire, FromRow, Sqlite, SqlitePool,
};
use tokio::task::JoinSet;
use tokio_stream::StreamExt as _;

use crate::{
    api::auth::{Permission, Role, User},
    auth::crypto::hash_secret,
    config::Config,
    error::Error,
    message::Message,
    namespace::{Namespace, NamespaceStatistics},
    queue::{Queue, QueueStatistics},
    sqs::types::{SqsMessage, SqsMessageAttribute},
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedrivePolicy {
    /// The field is named ARN, but for NerveMQ we use the format `namespace:queue`
    dead_letter_target_arn: String,
    max_receive_count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct QueueAttributes {
    delay_seconds: Option<u64>,
    max_message_size: Option<u64>,
    message_retention_period: Option<u64>,
    receive_message_wait_time_seconds: Option<u64>,
    visibility_timeout: Option<u64>,

    // TODO: RedrivePolicy, RedriveAllowPolicy
    redrive_policy: Option<RedrivePolicy /* Must be JSON serialized to a string */>,

    #[serde(flatten)]
    other: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct QueueAttributesSer {
    delay_seconds: Option<u64>,
    max_message_size: Option<u64>,
    message_retention_period: Option<u64>,
    receive_message_wait_time_seconds: Option<u64>,
    visibility_timeout: Option<u64>,

    // TODO: RedrivePolicy, RedriveAllowPolicy
    redrive_policy: Option<String /* Must be JSON serialized to a string */>,

    #[serde(flatten)]
    other: HashMap<String, String>,
}

impl QueueAttributes {
    pub fn ser(self) -> Result<QueueAttributesSer, Error> {
        Ok(QueueAttributesSer {
            delay_seconds: self.delay_seconds,
            max_message_size: self.max_message_size,
            message_retention_period: self.message_retention_period,
            receive_message_wait_time_seconds: self.receive_message_wait_time_seconds,
            visibility_timeout: self.visibility_timeout,
            redrive_policy: self
                .redrive_policy
                .map(|rp| serde_json::to_string(&rp))
                .transpose()?,
            other: self.other,
        })
    }
}

pub trait QueueAttribute {
    type Value;

    fn name(&self) -> &str;
}

pub mod queue_attributes {
    use super::QueueAttribute;

    pub struct DelaySeconds;

    impl QueueAttribute for DelaySeconds {
        type Value = u64;

        fn name(&self) -> &str {
            "delay_seconds"
        }
    }

    pub struct MaxMessageSize;

    impl QueueAttribute for MaxMessageSize {
        type Value = u64;
        fn name(&self) -> &str {
            "max_message_size"
        }
    }

    pub struct MessageRetentionPeriod;

    impl QueueAttribute for MessageRetentionPeriod {
        type Value = u64;
        fn name(&self) -> &str {
            "message_retention_period"
        }
    }

    pub struct ReceiveMessageWaitTimeSeconds;

    impl QueueAttribute for ReceiveMessageWaitTimeSeconds {
        type Value = u64;
        fn name(&self) -> &str {
            "receive_message_wait_time_seconds"
        }
    }

    pub struct VisibilityTimeout;

    impl QueueAttribute for VisibilityTimeout {
        type Value = u64;
        fn name(&self) -> &str {
            "visibility_timeout"
        }
    }

    pub struct RedrivePolicy;

    impl QueueAttribute for RedrivePolicy {
        type Value = String;

        fn name(&self) -> &str {
            "redrive_policy"
        }
    }

    pub struct Other(String);

    impl QueueAttribute for Other {
        type Value = String;

        fn name(&self) -> &str {
            &self.0
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct QueueConfig {
    pub queue: u64,
    pub max_retries: u64,
    pub dead_letter_queue: Option<u64>,
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

    pub async fn connect() -> Result<Self, Error> {
        Self::connect_with(Config::default()).await
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub async fn connect_with(config: Config) -> Result<Self, Error> {
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

        let svc = Self {
            db: pool,
            config: Arc::new(config),
        };

        match svc
            .create_user(
                Email::from_str(svc.config.root_email()).map_err(Error::internal)?,
                svc.config().root_password().to_owned().into(),
                Some(Role::Admin),
                vec![],
            )
            .await
        {
            Ok(_) => {
                tracing::info!("Root user created");
            }
            Err(e) => match e {
                Error::Sqlx { source } => match source {
                    sqlx::Error::Database(db_err) => match db_err.kind() {
                        sqlx::error::ErrorKind::UniqueViolation => {
                            tracing::info!("Root user already exists");
                        }
                        _ => tracing::warn!("{db_err}"),
                    },
                    other => tracing::warn!("{other}"),
                },
                other => tracing::warn!("{other}"),
            },
        };

        Ok(svc)
    }

    pub async fn get_queue_id(
        &self,
        namespace: &str,
        name: &str,
        exec: impl Acquire<'_, Database = Sqlite>,
    ) -> Result<Option<u64>, Error> {
        Ok(sqlx::query_scalar(
            "
            SELECT q.id FROM queues q
            JOIN namespaces n ON q.ns = n.id
            WHERE n.name = $1 AND q.name = $2
            ",
        )
        .bind(namespace)
        .bind(name)
        .fetch_optional(&mut *exec.acquire().await?)
        .await?)
    }

    pub async fn get_namespace_id<'a>(
        &self,
        name: &str,
        ex: impl Acquire<'a, Database = Sqlite>,
    ) -> Result<Option<u64>, Error> {
        Ok(sqlx::query_scalar(
            "
            SELECT id FROM namespaces WHERE name = $1
            ",
        )
        .bind(name)
        .fetch_optional(&mut *ex.acquire().await?)
        .await?)
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

    pub async fn check_user_role(&self, identity: Identity, role: Role) -> Result<(), Error> {
        let email = identity.id()?;
        let user: User = sqlx::query_as("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_one(&mut *self.db.acquire().await?)
            .await?;
        if user.role < role {
            return Err(Error::Unauthorized);
        }

        return Ok(());
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

    pub async fn check_user_access<'a>(
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
        attributes: HashMap<String, String>,
        tags: HashMap<String, String>,
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

        for (k, v) in attributes.into_iter() {
            sqlx::query(
                "
                INSERT INTO queue_attributes (queue, k, v)
                VALUES ($1, $2, $3)
                ",
            )
            .bind(queue_id as i64)
            .bind(k)
            .bind(v)
            .execute(&mut *tx)
            .await?;
        }

        for (k, v) in tags.into_iter() {
            sqlx::query(
                "
                INSERT INTO queue_tags (queue, k, v)
                VALUES ($1, $2, $3)
                ",
            )
            .bind(queue_id as i64)
            .bind(k)
            .bind(v)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(())
    }

    // TODO: Improve type safety for attributes

    pub async fn set_queue_attributes(
        &self,
        ns: &str,
        queue: &str,
        attributes: HashMap<String, String>,
        identity: Identity,
    ) -> Result<(), Error> {
        let mut db = self.db().acquire().await?;

        let ns_id = self
            .get_namespace_id(ns, &mut *db)
            .await?
            .ok_or(Error::namespace_not_found(ns))?;

        self.check_user_access(&identity, ns_id, &mut *db).await?;

        let queue_id = self
            .get_queue_id(ns, queue, &mut *db)
            .await?
            .ok_or(Error::queue_not_found(queue, ns))?;

        for (k, v) in attributes.into_iter() {
            sqlx::query(
                "
                INSERT INTO queue_attributes (queue, k, v)
                VALUES ($1, $2, $3)
                ON CONFLICT (queue, k) DO UPDATE SET v = $3
                ",
            )
            .bind(queue_id as i64)
            .bind(k)
            .bind(v)
            .execute(&mut *db)
            .await?;
        }

        Ok(())
    }

    pub async fn get_queue_attribute<A>(
        &self,
        ns: &str,
        queue: &str,
        attribute: A,
        identity: Identity,
    ) -> Result<Option<A::Value>, Error>
    where
        A: QueueAttribute,
        A::Value: for<'a> sqlx::FromRow<'a, SqliteRow> + std::marker::Unpin + Send,
    {
        let mut db = self.db().acquire().await?;

        let ns_id = self
            .get_namespace_id(ns, &mut *db)
            .await?
            .ok_or(Error::namespace_not_found(ns))?;

        self.check_user_access(&identity, ns_id, &mut *db).await?;

        let queue_id = self
            .get_queue_id(ns, queue, &mut *db)
            .await?
            .ok_or(Error::queue_not_found(queue, ns))?;

        let res = sqlx::query_as::<_, A::Value>(
            "
            SELECT k, v FROM queue_attributes WHERE queue = $1 AND k = $2
            ",
        )
        .bind(queue_id as i64)
        .bind(attribute.name())
        .fetch_optional(&mut *db)
        .await?;

        Ok(res)
    }

    pub async fn set_queue_attribute<A>(
        &self,
        ns: &str,
        queue: &str,
        attribute: A,
        value: A::Value,
        identity: Identity,
    ) -> Result<(), Error>
    where
        A: QueueAttribute,
        A::Value: sqlx::Type<Sqlite> + for<'a> sqlx::Encode<'a, Sqlite> + std::marker::Unpin + Send,
    {
        let mut tx = self.db().begin().await?;

        let ns_id = self
            .get_namespace_id(ns, &mut *tx)
            .await?
            .ok_or(Error::namespace_not_found(ns))?;

        self.check_user_access(&identity, ns_id, &mut *tx).await?;

        let queue_id = self
            .get_queue_id(ns, queue, &mut *tx)
            .await?
            .ok_or(Error::queue_not_found(queue, ns))?;

        sqlx::query(
            "
            INSERT INTO queue_attributes (queue, k, v)
            VALUES ($1, $2, $3)
            ON CONFLICT (queue, k) DO UPDATE SET v = $3
            ",
        )
        .bind(queue_id as i64)
        .bind(attribute.name())
        .bind(value)
        .execute(&mut *tx)
        .await?;

        Ok(())
    }

    pub async fn get_queue_attributes(
        &self,
        ns: &str,
        queue: &str,
        names: &[String],
        identity: Identity,
    ) -> Result<HashMap<String, String>, Error> {
        let mut db = self.db().acquire().await?;

        let ns_id = self
            .get_namespace_id(ns, &mut *db)
            .await?
            .ok_or(Error::namespace_not_found(ns))?;

        self.check_user_access(&identity, ns_id, &mut *db).await?;

        let queue_id = self
            .get_queue_id(ns, queue, &mut *db)
            .await?
            .ok_or(Error::queue_not_found(queue, ns))?;

        let set = names.iter().collect::<HashSet<_>>();

        let res = sqlx::query_as(
            "
            SELECT k, v FROM queue_attributes WHERE queue = $1
            ",
        )
        .bind(queue_id as i64)
        .fetch_all(&mut *db)
        .await?;

        Ok(res.into_iter().filter(|(k, _)| set.contains(k)).collect())
    }

    pub async fn tag_queue(
        &self,
        ns: &str,
        queue: &str,
        tags: HashMap<String, String>,
        identity: Identity,
    ) -> Result<(), Error> {
        let mut db = self.db().acquire().await?;
        let ns_id = self
            .get_namespace_id(ns, &mut *db)
            .await?
            .ok_or(Error::namespace_not_found(ns))?;

        self.check_user_access(&identity, ns_id, &mut *db).await?;

        let queue_id = self
            .get_queue_id(ns, queue, &mut *db)
            .await?
            .ok_or(Error::queue_not_found(queue, ns))?;

        for (k, v) in tags.into_iter() {
            sqlx::query(
                "
                INSERT INTO queue_tags (queue, k, v)
                VALUES ($1, $2, $3)
                ON CONFLICT (queue, k) DO UPDATE SET v
                ",
            )
            .bind(queue_id as i64)
            .bind(k)
            .bind(v)
            .execute(&mut *db)
            .await?;
        }

        Ok(())
    }

    pub async fn untag_queue(
        &self,
        ns: &str,
        queue: &str,
        tags: Vec<String>,
        identity: Identity,
    ) -> Result<(), Error> {
        let mut db = self.db().acquire().await?;
        let ns_id = self
            .get_namespace_id(ns, &mut *db)
            .await?
            .ok_or(Error::namespace_not_found(ns))?;

        self.check_user_access(&identity, ns_id, &mut *db).await?;

        let queue_id = self
            .get_queue_id(ns, queue, &mut *db)
            .await?
            .ok_or(Error::queue_not_found(queue, ns))?;

        for tag in tags {
            sqlx::query(
                "
                DELETE FROM queue_tags WHERE queue = $1 AND k = $2
                ",
            )
            .bind(queue_id as i64)
            .bind(tag)
            .execute(&mut *db)
            .await?;
        }

        Ok(())
    }

    pub async fn get_queue_tags(
        &self,
        ns: &str,
        queue: &str,
        identity: Identity,
    ) -> Result<HashMap<String, String>, Error> {
        let mut db = self.db().acquire().await?;

        let ns_id = self
            .get_namespace_id(ns, &mut *db)
            .await?
            .ok_or(Error::namespace_not_found(ns))?;

        self.check_user_access(&identity, ns_id, &mut *db).await?;

        let queue_id = self
            .get_queue_id(ns, queue, &mut *db)
            .await?
            .ok_or(Error::queue_not_found(queue, ns))?;

        let res = sqlx::query_as(
            "
            SELECT k, v FROM queue_tags WHERE queue = $1
            ",
        )
        .bind(queue_id as i64)
        .fetch_all(&mut *db)
        .await?;

        Ok(res.into_iter().collect())
    }

    pub async fn delete_queue(
        &self,
        namespace: &str,
        name: &str,
        identity: Identity,
    ) -> Result<(), Error> {
        let mut tx = self.db().begin().await?;

        let namespace_id = self
            .get_namespace_id(namespace, &mut tx)
            .await?
            .ok_or_else(|| eyre::eyre!("Namespace {namespace} does not exist"))?;

        self.check_user_access(&identity, namespace_id, &mut tx)
            .await?;

        let id = self
            .get_queue_id(namespace, name, &mut tx)
            .await?
            .ok_or_else(|| eyre::eyre!("Queue {name} does not exist"))?;

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
    ) -> Result<(), Error> {
        let hashed_password = web::block(move || hash_secret(password))
            .await
            .map_err(|e| Error::internal(e))??;

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

    pub async fn sqs_send(
        &self,
        queue: u64,
        message: &String,
        kv: HashMap<String, SqsMessageAttribute>,
    ) -> Result<u64, Error> {
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
                .bind(bincode::serialize(&v).map_err(Error::internal)?)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;

        Ok(msg_id)
    }

    pub async fn sqs_send_batch(
        &self,
        namespace: &str,
        queue: &str,
        messages: Vec<(Vec<u8>, HashMap<String, SqsMessageAttribute>)>,
    ) -> Result<Vec<u64>, Error> {
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
                    .bind(bincode::serialize(&v).map_err(Error::internal)?)
                    .execute(&mut *tx)
                    .await?;
            }

            message_ids.push(msg_id);
        }

        tx.commit().await?;

        Ok(message_ids)
    }

    pub async fn sqs_recv(
        &self,
        namespace: impl AsRef<str>,
        queue: impl AsRef<str>,
    ) -> Result<Option<SqsMessage>, Error> {
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
                    q.name as queue,
                    (CASE
                        WHEN m.delivered_at IS NULL AND m.tries < conf.max_retries THEN 'pending'
                        WHEN m.delivered_at IS NULL AND m.tries >= conf.max_retries THEN 'failed'
                        ELSE 'delivered'
                    END) as status
                FROM messages m
                JOIN queues q ON m.queue = q.id
                JOIN queue_configurations conf ON q.id = conf.queue
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

        let message = if let Some(message) = message {
            let mut message_attributes = HashMap::new();

            let mut kv = sqlx::query_as::<_, (String, Vec<u8>)>(
                "
                SELECT k, v FROM kv_pairs WHERE message = $1
                ",
            )
            .bind(message.id as i64)
            .fetch(&mut *tx);

            while let Some((k, v)) = kv.next().await.transpose()? {
                let v: SqsMessageAttribute = bincode::deserialize(&v).map_err(Error::internal)?;
                message_attributes.insert(k, v);
            }

            let sqs_message = SqsMessage {
                message_id: message.id.to_string(),
                md5_of_body: hex::encode(md5::compute(&message.body).as_slice()),
                body: message.body,
                message_attributes,
                attributes: HashMap::new(),
                receipt_handle: "TODO".to_owned(),
            };

            Some(sqs_message)
        } else {
            None
        };

        tx.commit().await?;

        Ok(message)
    }

    pub async fn sqs_recv_batch(
        &self,
        namespace: &str,
        queue: &str,
        max_messages: u64,
    ) -> Result<Vec<SqsMessage>, Error> {
        let mut tx = self.db().begin().await?;

        // Get multiple undelivered messages and mark them as delivered in one atomic operation
        let mut stream = sqlx::query_as::<_, Message>(
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
                JOIN queue_configurations conf ON q.id = conf.queue
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
            RETURNING
                *,
                (SELECT queue_name FROM next_messages WHERE next_messages.id = messages.id) as queue,
                (CASE
                    WHEN messages.delivered_at IS NULL AND messages.tries < (SELECT max_retries FROM queue_configurations WHERE queue = messages.queue) THEN 'pending'
                    WHEN messages.delivered_at IS NULL AND messages.tries >= (SELECT max_retries FROM queue_configurations WHERE queue = messages.queue) THEN 'failed'
                    ELSE 'delivered'
                END) as status
            ",
        )
        .bind(namespace)
        .bind(queue)
        .bind(max_messages as i64)
        .fetch(&mut *tx);
        // .await
        //     .map_err(|e| {
        //         tracing::error!("Failed to fetch messages {e}");
        //         e
        //     })
        //     ?
        // .into_iter()
        // .map(|message: Message| SqsMessage {
        //     message_id: message.id.to_string(),
        //     md5_of_body: hex::encode(md5::compute(&message.body).as_slice()),
        //     body: message.body,
        // })
        // .collect();

        let mut messages = vec![];
        while let Some(message) = stream.next().await.transpose()? {
            let mut message_attributes = HashMap::new();
            let mut kv = sqlx::query_as::<_, (String, Vec<u8>)>(
                "
                SELECT k, v FROM kv_pairs WHERE message = $1
                ",
            )
            .bind(message.id as i64)
            .fetch(self.db());

            while let Some((k, v)) = kv.next().await.transpose()? {
                let v: SqsMessageAttribute = bincode::deserialize(&v).map_err(Error::internal)?;
                message_attributes.insert(k, v);
            }

            let sqs_message = SqsMessage {
                message_id: message.id.to_string(),
                md5_of_body: hex::encode(md5::compute(&message.body).as_slice()),
                body: message.body,
                message_attributes,
                attributes: HashMap::new(),
                receipt_handle: "TODO".to_owned(),
            };
            messages.push(sqs_message);
        }

        drop(stream);

        tx.commit().await?;

        Ok(messages)
    }

    // pub async fn sqs_peek(
    //     &self,
    //     namespace: &str,
    //     queue: &str,
    //     message_id: u64,
    // ) -> Result<Option<SqsMessage>, Error> {
    //     let mut db = self.db().acquire().await?;
    //
    //     let msg: Option<Message> = sqlx::query_as(
    //         "
    //         SELECT
    //             m.*,
    //             q.name as queue,
    //             (CASE
    //                 WHEN m.delivered_at IS NULL AND m.tries < conf.max_retries THEN 'pending'
    //                 WHEN m.delivered_at IS NULL AND m.tries >= conf.max_retries THEN 'failed'
    //                 ELSE 'delivered'
    //             END) as status
    //         FROM messages m
    //         JOIN queues q ON m.queue = q.id
    //         JOIN queue_configurations conf ON q.id = conf.queue
    //         JOIN namespaces n ON q.ns = n.id
    //         WHERE n.name = $1 AND q.name = $2 AND m.id = $3
    //         ",
    //     )
    //     .bind(namespace)
    //     .bind(queue)
    //     .bind(message_id as i64)
    //     .fetch_optional(&mut *db)
    //     .await?;
    //
    //     let msg = match msg {
    //         Some(msg) => SqsMessage {
    //             message_id: msg.id.to_string(),
    //             md5_of_body: hex::encode(md5::compute(&msg.body).as_slice()),
    //             body: msg.body,
    //         },
    //         None => return Ok(None),
    //     };
    //
    //     // let mut kv = sqlx::query_as::<_, (String, Vec<u8>)>(
    //     //     "
    //     //     SELECT k, v FROM kv_pairs WHERE message = $1
    //     //     ",
    //     // )
    //     // .bind(msg.id as i64)
    //     // .fetch(&mut *db);
    //     //
    //     // while let Some((k, v)) = kv.next().await.transpose()? {
    //     //     msg.kv
    //     //         .insert(k, bincode::deserialize(&v).map_err(Error::internal)?);
    //     // }
    //
    //     Ok(Some(msg))
    // }

    pub async fn list_messages(&self, namespace: &str, queue: &str) -> Result<Vec<Message>, Error> {
        let mut db = self.db().acquire().await?;

        let mut messages = sqlx::query_as::<_, Message>(
            "
            SELECT
                m.*,
                q.name as queue,
                (CASE
                    WHEN m.delivered_at IS NULL AND m.tries < conf.max_retries THEN 'pending'
                    WHEN m.delivered_at IS NULL AND m.tries >= conf.max_retries THEN 'failed'
                    ELSE 'delivered'
                END) as status
            FROM messages m
            JOIN queues q ON m.queue = q.id
            JOIN queue_configurations conf ON q.id = conf.queue
            WHERE q.ns = (SELECT id FROM namespaces WHERE name = $1) AND q.name = $2
        ",
        )
        .bind(namespace)
        .bind(queue)
        .fetch(&mut *db);

        let mut join_set = JoinSet::new();
        while let Some(mut message) = messages.next().await.transpose()? {
            let db = self.db().clone();
            join_set.spawn_local(async move {
                let mut conn = db.acquire().await?;
                let mut kv_pairs = sqlx::query_as::<_, (String, Vec<u8>)>(
                    "
                    SELECT k, v FROM kv_pairs WHERE message = $1
                ",
                )
                .bind(message.id as i64)
                .fetch(&mut *conn);

                while let Some((k, v)) = kv_pairs.next().await.transpose()? {
                    message
                        .kv
                        .insert(k, bincode::deserialize(&v).map_err(Error::internal)?);
                }

                Result::<_, Error>::Ok(message)
            });
        }

        let mut messages = Vec::new();

        while let Some(result) = join_set
            .join_next()
            .await
            .transpose()
            .map_err(Error::internal)?
            .transpose()?
        {
            messages.push(result);
        }

        Ok(messages)
    }

    pub async fn get_queue_configuration(&self, queue: u64) -> Result<QueueConfig, Error> {
        let mut db = self.db().acquire().await?;
        Ok(sqlx::query_as(
            "
            SELECT * FROM queue_configurations WHERE queue = $1
            ",
        )
        .bind(queue as i64)
        .fetch_one(&mut *db)
        .await?)
    }

    pub async fn update_queue_configuration(
        &self,
        queue: u64,
        new_config: QueueConfig,
    ) -> Result<(), Error> {
        let mut db = self.db().acquire().await?;

        sqlx::query(
            "
            UPDATE queue_configurations
            SET max_retries = $1, dead_letter_queue = $2
            WHERE queue = $3
            ",
        )
        .bind(new_config.max_retries as i64)
        .bind(new_config.dead_letter_queue.map(|id| id as i64))
        .bind(queue as i64)
        .execute(&mut *db)
        .await?;

        Ok(())
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
                IFNULL(AVG(LENGTH(m.body)), 0.0) as avg_size_bytes,
                COUNT(CASE WHEN m.delivered_at IS NULL AND m.tries < conf.max_retries THEN 1 END) as pending,
                COUNT(CASE WHEN m.delivered_at IS NOT NULL THEN 1 END) as delivered,
                COUNT(CASE WHEN m.delivered_at IS NULL AND m.tries >= conf.max_retries THEN 1 END) as failed
            FROM queues q
            JOIN queue_configurations conf ON q.id = conf.queue
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
                IFNULL(AVG(LENGTH(m.body)), 0.0) as avg_size_bytes,
                COUNT(CASE WHEN m.delivered_at IS NULL AND m.tries < conf.max_retries THEN 1 END) as pending,
                COUNT(CASE WHEN m.delivered_at IS NOT NULL  THEN 1 END) as delivered,
                COUNT(CASE WHEN m.delivered_at IS NULL AND m.tries >= conf.max_retries THEN 1 END) as failed
            FROM queues q
            JOIN queue_configurations conf ON q.id = conf.queue
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

    pub async fn delete_message_batch(
        &self,
        namespace: &str,
        queue: &str,
        message_ids: Vec<u64>,
        identity: Identity,
    ) -> Result<
        (
            Vec<u64>,          // Successfully deleted message IDs
            Vec<(u64, Error)>, // Failed message IDs
        ),
        Error,
    > {
        let mut tx = self.db().begin().await?;
        // Verify namespace exists and user has access
        let namespace_id = self
            .get_namespace_id(namespace, &mut tx)
            .await?
            .ok_or_else(|| Error::namespace_not_found(namespace))?;
        self.check_user_access(&identity, namespace_id, &mut tx)
            .await?;
        // Verify queue exists
        let queue_id = self
            .get_queue_id(namespace, queue, &mut tx)
            .await?
            .ok_or_else(|| Error::queue_not_found(queue, namespace))?;

        let mut success = Vec::new();
        let mut failure = Vec::new();

        for message_id in message_ids {
            match sqlx::query(
                "
                DELETE FROM messages
                WHERE id = $1 AND queue = $2
                ",
            )
            .bind(message_id as i64)
            .bind(queue_id as i64)
            .execute(&mut *tx)
            .await
            {
                Ok(res) => {
                    if res.rows_affected() == 0 {
                        failure.push((
                            message_id,
                            Error::not_found(format!("{message_id} in queue {queue}")),
                        ));
                    } else {
                        success.push(message_id);
                    }
                }
                Err(err) => failure.push((message_id, err.into())),
            };
        }

        Ok((success, failure))
    }

    pub async fn delete_message(
        &self,
        namespace: &str,
        queue: &str,
        message_id: u64,
        identity: Identity,
    ) -> Result<(), Error> {
        let mut tx = self.db().begin().await?;

        // Verify namespace exists and user has access
        let namespace_id = self
            .get_namespace_id(namespace, &mut tx)
            .await?
            .ok_or_else(|| Error::namespace_not_found(namespace))?;

        self.check_user_access(&identity, namespace_id, &mut tx)
            .await?;

        // Verify queue exists
        let queue_id = self
            .get_queue_id(namespace, queue, &mut tx)
            .await?
            .ok_or_else(|| Error::queue_not_found(queue, namespace))?;

        // Delete the message if it exists in this queue
        let result = sqlx::query(
            "
            DELETE FROM messages
            WHERE id = $1 AND queue = $2
            ",
        )
        .bind(message_id as i64)
        .bind(queue_id as i64)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            return Err(Error::not_found(format!("{message_id} in queue {queue}")));
        }

        tx.commit().await?;

        Ok(())
    }

    pub async fn purge_queue(
        &self,
        namespace: &str,
        queue: &str,
        identity: Identity,
    ) -> Result<(), Error> {
        let mut tx = self.db().begin().await?;

        // Verify namespace exists and user has access
        let namespace_id = self
            .get_namespace_id(namespace, &mut tx)
            .await?
            .ok_or_else(|| Error::namespace_not_found(namespace))?;

        self.check_user_access(&identity, namespace_id, &mut tx)
            .await?;

        // Verify queue exists
        let queue_id = self
            .get_queue_id(namespace, queue, &mut tx)
            .await?
            .ok_or_else(|| Error::queue_not_found(queue, namespace))?;

        // Delete all messages from the queue
        sqlx::query(
            "
            DELETE FROM messages
            WHERE queue = $1
            ",
        )
        .bind(queue_id as i64)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok(())
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
            WHERE u.email = $1
            GROUP BY ns.id, nu.email
        ",
        )
        .bind(email)
        .fetch_all(&mut *self.db().acquire().await?)
        .await?)
    }
}
