use std::{collections::HashMap, future::Future};

use actix_session::storage::{
    generate_session_key, LoadError, SaveError, SessionStore, UpdateError,
};
use serde::{Deserialize, Serialize};
use sqlx::{Acquire, SqlitePool};
use tokio_stream::StreamExt;

#[derive(Clone)]
pub struct SqliteSessionStore {
    db: SqlitePool,
}

impl SqliteSessionStore {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }
}

pub type SessionState = HashMap<String, String>;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Session {
    id: u64,
    session_key: String,

    #[sqlx(skip)]
    state: SessionState,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
struct SessionStateEntry {
    session: u64,
    k: String,
    v: String,
}

impl SessionStore for SqliteSessionStore {
    fn load(
        &self,
        session_key: &actix_session::storage::SessionKey,
    ) -> impl Future<Output = Result<Option<SessionState>, actix_session::storage::LoadError>> {
        let db = self.db.clone();
        async move {
            tracing::info!("Loading session");
            let mut tx = db
                .begin()
                .await
                .map_err(|e| LoadError::Other(anyhow::Error::new(e)))?;

            let session: Option<Session> =
                sqlx::query_as("SELECT * from sessions WHERE session_key = $1")
                    .bind(session_key.as_ref())
                    .fetch_optional(
                        tx.acquire()
                            .await
                            .map_err(|e| LoadError::Other(anyhow::Error::new(e)))?,
                    )
                    .await
                    .map_err(|e| LoadError::Other(anyhow::Error::new(e)))?;

            tracing::info!("Loaded session: {session:?}");

            let mut session = match session {
                Some(session) => session,
                None => return Ok(None),
            };

            let state = {
                let mut kv = sqlx::query_as::<_, SessionStateEntry>(
                    "SELECT * FROM session_state WHERE session = $1",
                )
                .bind(session.id as i64)
                .fetch(tx.as_mut());

                while let Some(pair) = kv
                    .next()
                    .await
                    .transpose()
                    .map_err(|e| LoadError::Other(anyhow::Error::new(e)))?
                {
                    session.state.insert(pair.k, pair.v);
                }

                session.state
            };

            tx.commit()
                .await
                .map_err(|e| LoadError::Other(anyhow::Error::new(e)))?;

            Ok(Some(state))
        }
    }

    fn save(
        &self,
        session_state: SessionState,
        ttl: &actix_web::cookie::time::Duration,
    ) -> impl std::future::Future<
        Output = Result<actix_session::storage::SessionKey, actix_session::storage::SaveError>,
    > {
        let db = self.db.clone();
        async move {
            let mut tx = db
                .begin()
                .await
                .map_err(|e| SaveError::Other(anyhow::Error::new(e)))?;

            let key = generate_session_key();

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
        }
    }

    fn update(
        &self,
        session_key: actix_session::storage::SessionKey,
        session_state: SessionState,
        ttl: &actix_web::cookie::time::Duration,
    ) -> impl std::future::Future<
        Output = Result<actix_session::storage::SessionKey, actix_session::storage::UpdateError>,
    > {
        let db = self.db.clone();

        async move {
            tracing::info!("Updating session");
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
                .bind(session_key.as_ref())
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
        }
    }

    fn update_ttl(
        &self,
        session_key: &actix_session::storage::SessionKey,
        ttl: &actix_web::cookie::time::Duration,
    ) -> impl std::future::Future<Output = Result<(), anyhow::Error>> {
        let db = self.db.clone();

        async move {
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
        }
    }

    fn delete(
        &self,
        session_key: &actix_session::storage::SessionKey,
    ) -> impl std::future::Future<Output = Result<(), anyhow::Error>> {
        let db = self.db.clone();
        async move {
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
        }
    }
}
