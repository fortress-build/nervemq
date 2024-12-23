//! SQLite-based session storage implementation for Actix-web.
//!
//! This module provides a persistent session storage backend using SQLite. It implements
//! the `SessionStore` trait from actix-session and stores session data in two tables:
//! - sessions: Stores session metadata (id, key, TTL)
//! - session_state: Stores key-value pairs for each session
//!
//! The implementation supports all standard session operations including:
//! - Creating new sessions
//! - Loading existing sessions
//! - Updating session data
//! - Managing session TTL
//! - Deleting sessions

use actix_session::storage::{LoadError, SaveError, SessionKey, UpdateError};
use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tokio_stream::StreamExt;

pub use actix_session::storage::SessionStore;

pub type SessionState = serde_json::Map<String, serde_json::Value>;

/// SQLite-based implementation of the session store.
///
/// Provides persistent storage of session data using SQLite as the backend.
/// Each instance maintains a connection pool to the database.
#[derive(Clone)]
pub struct SqliteSessionStore {
    db: SqlitePool,
}

impl SqliteSessionStore {
    /// Creates a new SQLite session store with the provided database connection pool.
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }
}

/// Represents a session in the database.
///
/// Contains the session's unique identifier, key for lookup,
/// and the associated state data.
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Session {
    id: u64,
    session_key: String,

    #[sqlx(skip)]
    state: SessionState,
}

/// Represents a single key-value entry in a session's state.
///
/// Maps to the session_state table in SQLite, where each row
/// contains a reference to its parent session and a key-value pair.
#[derive(Serialize, Deserialize, sqlx::FromRow)]
struct SessionStateEntry {
    session: u64,
    k: String,
    v: serde_json::Value,
}

/// Implementation of the `SessionStore` trait for SQLite-based storage.
///
/// Provides all required session management operations including:
/// - Loading sessions by key
/// - Saving new sessions
/// - Updating existing sessions
/// - Managing session TTL
/// - Deleting sessions
impl SessionStore for SqliteSessionStore {
    /// Loads a session from the database by its key.
    ///
    /// Returns the session state if found, or None if the session doesn't exist.
    /// Also loads all associated key-value pairs from the session_state table.
    fn load(
        &self,
        session_key: &actix_session::storage::SessionKey,
    ) -> impl ::core::future::Future<Output = Result<Option<SessionState>, LoadError>> {
        let db = self.db.clone();
        Box::pin(async move {
            let session: Option<Session> =
                sqlx::query_as("SELECT * from sessions WHERE session_key = $1")
                    .bind(session_key.as_ref())
                    .fetch_optional(&db)
                    .await
                    .map_err(|e| {
                        tracing::error!("Failed to load session: {e}");
                        LoadError::Other(anyhow::Error::new(e))
                    })?;

            let session = match session {
                Some(mut session) => {
                    let mut kv = sqlx::query_as::<_, SessionStateEntry>(
                        "SELECT * FROM session_state WHERE session = $1",
                    )
                    .bind(session.id as i64)
                    .fetch(&db);

                    while let Some(pair) = kv.next().await.transpose().map_err(|e| {
                        tracing::warn!("Load error: {e}");
                        LoadError::Other(anyhow::Error::new(e))
                    })? {
                        session.state.insert(pair.k, pair.v);
                    }

                    session
                }
                None => {
                    return Ok(None);
                }
            };

            tracing::debug!("Loaded session: {}", session.id);

            Ok(Some(session.state))
        })
    }

    /// Saves a new session to the database.
    ///
    /// Creates a new session with a randomly generated key and stores all
    /// provided state data. Returns the new session key on success.
    fn save(
        &self,
        session_state: SessionState,
        ttl: &actix_web::cookie::time::Duration,
    ) -> impl ::core::future::Future<Output = Result<actix_session::storage::SessionKey, SaveError>>
    {
        let db = self.db.clone();
        Box::pin(async move {
            let mut tx = db
                .begin()
                .await
                .map_err(|e| SaveError::Other(anyhow::Error::new(e)))?;

            let key: SessionKey = Alphanumeric
                .sample_string(&mut rand::thread_rng(), 64)
                .try_into()
                .expect("generated string should be within the size range for a session key");

            let id: u64 = sqlx::query_scalar(
                "
                INSERT INTO sessions (session_key, ttl)
                VALUES ($1, $2)
                RETURNING id
                ",
            )
            .bind(key.as_ref())
            .bind(ttl.whole_seconds())
            .fetch_one(tx.as_mut())
            .await
            .map_err(|e| SaveError::Other(anyhow::Error::new(e)))?;

            for (k, v) in session_state.into_iter() {
                sqlx::query(
                    "
                    INSERT INTO session_state (session, k, v)
                    VALUES ($1, $2, $3)
                ",
                )
                .bind(id as i64)
                .bind(k)
                .bind(v)
                .execute(tx.as_mut())
                .await
                .map_err(|e| SaveError::Other(anyhow::Error::new(e)))?;
            }

            tx.commit()
                .await
                .map_err(|e| SaveError::Other(anyhow::Error::new(e)))?;

            Ok(key)
        })
    }

    /// Updates an existing session with new state data.
    ///
    /// Modifies both the session's TTL and its state data. Removes any
    /// key-value pairs that are no longer present in the new state.
    fn update(
        &self,
        session_key: actix_session::storage::SessionKey,
        session_state: SessionState,
        ttl: &actix_web::cookie::time::Duration,
    ) -> impl ::core::future::Future<Output = Result<actix_session::storage::SessionKey, UpdateError>>
    {
        let db = self.db.clone();
        Box::pin(async move {
            let mut tx = db
                .begin()
                .await
                .map_err(|e| UpdateError::Other(anyhow::Error::new(e)))?;

            let ttl_query = "
                UPDATE sessions
                SET ttl = $1
                WHERE session_key = $2
                RETURNING id
            ";

            let session_id: u64 = sqlx::query_scalar(ttl_query)
                .bind(ttl.whole_seconds())
                .bind(session_key.as_ref())
                .fetch_one(tx.as_mut())
                .await
                .map_err(|e| UpdateError::Other(anyhow::Error::new(e)))?;

            let keys =
                session_state
                    .keys()
                    .map(|k| format!("'{k}'"))
                    .fold(String::new(), |s, k| {
                        if s.len() == 0 {
                            return k;
                        }
                        format!("{s}, {k}")
                    });

            sqlx::query(&format!(
                "
                DELETE FROM session_state
                WHERE session = $1 AND k NOT IN ({keys})
            ",
            ))
            .bind(session_id as i64)
            .execute(tx.as_mut())
            .await
            .map_err(|e| UpdateError::Other(anyhow::Error::new(e)))?;

            for (k, v) in session_state.iter() {
                sqlx::query(
                    "
                        INSERT OR REPLACE INTO session_state (session, k, v)
                        VALUES ($1, $2, $3)
                    ",
                )
                .bind(session_id as i64)
                .bind(k)
                .bind(v)
                .execute(tx.as_mut())
                .await
                .map_err(|e| UpdateError::Other(anyhow::Error::new(e)))?;
            }

            tx.commit()
                .await
                .map_err(|e| UpdateError::Other(anyhow::Error::new(e)))?;

            Ok(session_key)
        })
    }

    /// Updates only the TTL (time-to-live) of an existing session.
    ///
    /// This is used to extend or shorten a session's lifetime without
    /// modifying its state data.
    fn update_ttl(
        &self,
        session_key: &actix_session::storage::SessionKey,
        ttl: &actix_web::cookie::time::Duration,
    ) -> impl ::core::future::Future<Output = Result<(), anyhow::Error>> {
        let db = self.db.clone();

        Box::pin(async move {
            let query = "
                UPDATE sessions
                SET ttl = $1
                WHERE session_key = $2
            ";
            let mut db = db.acquire().await.map_err(|e| anyhow::Error::new(e))?;

            sqlx::query(query)
                .bind(ttl.whole_seconds())
                .bind(session_key.as_ref())
                .execute(db.as_mut())
                .await
                .map_err(|e| anyhow::Error::new(e))?;

            Ok(())
        })
    }

    /// Deletes a session and all its associated state data.
    ///
    /// Removes the session and relies on foreign key cascading to
    /// clean up related session_state entries.
    fn delete(
        &self,
        session_key: &actix_session::storage::SessionKey,
    ) -> impl ::core::future::Future<Output = Result<(), anyhow::Error>> {
        let db = self.db.clone();
        Box::pin(async move {
            let mut db = db
                .acquire()
                .await
                .map_err(|e| LoadError::Other(anyhow::Error::new(e)))?;

            sqlx::query("DELETE FROM sessions WHERE session_key = $1")
                .bind(session_key.as_ref())
                .execute(db.as_mut())
                .await
                .map_err(|e| LoadError::Other(anyhow::Error::new(e)))?;

            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::cookie::time::Duration;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_db() -> SqlitePool {
        let db = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY,
                session_key TEXT NOT NULL UNIQUE,
                ttl INTEGER NOT NULL
            )
            "#,
        )
        .execute(&db)
        .await
        .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS session_state (
                session INTEGER NOT NULL,
                k TEXT NOT NULL,
                v TEXT NOT NULL,
                PRIMARY KEY (session, k),
                FOREIGN KEY (session) REFERENCES sessions(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&db)
        .await
        .unwrap();

        db
    }

    fn create_test_state() -> SessionState {
        let mut state = SessionState::new();
        state.insert(
            "user_id".to_string(),
            serde_json::Value::String("123".to_string()),
        );
        state.insert(
            "username".to_string(),
            serde_json::Value::String("test_user".to_string()),
        );
        state
    }

    #[tokio::test]
    async fn test_save_and_load_session() {
        let db = setup_db().await;
        let store = SqliteSessionStore::new(db);
        let state = create_test_state();
        let ttl = Duration::minutes(30);

        // Save session
        let session_key = store.save(state.clone(), &ttl).await.unwrap();

        // Load and verify session
        let loaded_state = store.load(&session_key).await.unwrap().unwrap();
        assert_eq!(
            loaded_state.get("user_id").unwrap().as_str().unwrap(),
            "123"
        );
        assert_eq!(
            loaded_state.get("username").unwrap().as_str().unwrap(),
            "test_user"
        );
    }

    #[tokio::test]
    async fn test_update_session() {
        let db = setup_db().await;
        let store = SqliteSessionStore::new(db);
        let initial_state = create_test_state();
        let ttl = Duration::minutes(30);

        // Create initial session
        let session_key = store.save(initial_state, &ttl).await.unwrap();

        // Update session with new state
        let mut new_state = SessionState::new();
        new_state.insert(
            "user_id".to_string(),
            serde_json::Value::String("456".to_string()),
        );

        store
            .update(session_key.clone(), new_state, &ttl)
            .await
            .unwrap();

        // Verify updated state
        let loaded_state = store.load(&session_key).await.unwrap().unwrap();
        assert_eq!(
            loaded_state.get("user_id").unwrap().as_str().unwrap(),
            "456"
        );
        assert!(loaded_state.get("username").is_none());
    }

    #[tokio::test]
    async fn test_delete_session() {
        let db = setup_db().await;
        let store = SqliteSessionStore::new(db);
        let state = create_test_state();
        let ttl = Duration::minutes(30);

        // Create session
        let session_key = store.save(state, &ttl).await.unwrap();

        // Delete session
        store.delete(&session_key).await.unwrap();

        // Verify session is deleted
        let loaded_state = store.load(&session_key).await.unwrap();
        assert!(loaded_state.is_none());
    }

    #[tokio::test]
    async fn test_update_ttl() {
        let db = setup_db().await;
        let store = SqliteSessionStore::new(db.clone());
        let state = create_test_state();
        let initial_ttl = Duration::minutes(30);

        // Create session
        let session_key = store.save(state, &initial_ttl).await.unwrap();

        // Update TTL
        let new_ttl = Duration::minutes(60);
        store.update_ttl(&session_key, &new_ttl).await.unwrap();

        // Verify TTL was updated
        let updated_ttl: i64 = sqlx::query_scalar("SELECT ttl FROM sessions WHERE session_key = ?")
            .bind(session_key.as_ref())
            .fetch_one(&db)
            .await
            .unwrap();

        assert_eq!(updated_ttl, new_ttl.whole_seconds());
    }
}
