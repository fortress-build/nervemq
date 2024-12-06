//! SQLite-based implementation of the Key Management Service (KMS).
//!
//! This module provides a simple key management solution that stores encryption keys
//! in a SQLite database. It implements the [`KeyManager`] trait using AES-GCM-SIV
//! for encryption operations.
//!
//! # Security Considerations
//! This implementation stores encryption keys directly in the database. While suitable
//! for development or testing, production environments should consider using a more
//! secure key management solution like AWS KMS.

use std::{future::Future, pin::Pin};

use aes_gcm_siv::{aead::Aead, Aes256GcmSiv, KeyInit, Nonce};
use sqlx::SqlitePool;

use crate::{auth::crypto::generate_token, error::Error};

use super::KeyManager;

/// A Key Management Service implementation that stores encryption keys in SQLite.
///
/// This implementation:
/// - Uses AES-256-GCM-SIV for encryption/decryption
/// - Stores keys in a dedicated SQLite table
/// - Generates unique key IDs automatically
/// - Performs cryptographic operations in a separate blocking thread pool
#[derive(Clone)]
pub struct SqliteKeyManager {
    pool: SqlitePool,
}

/// Represents an encryption key stored in the SQLite database.
///
/// # Fields
/// * `key_id` - Unique identifier for the key
/// * `key` - The raw key material as bytes
#[derive(Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct EncryptionKey {
    key_id: String,
    key: Vec<u8>,
}

impl SqliteKeyManager {
    /// Creates a new SQLite-based key manager.
    ///
    /// # Arguments
    /// * `pool` - A connection pool to the SQLite database
    ///
    /// # Returns
    /// A new instance of [`SqliteKeyManager`]
    ///
    /// This method will create the required database table if it doesn't exist.
    pub async fn new(pool: SqlitePool) -> Result<Self, Error> {
        // Since we're not necessarily using the sqlite key manager, we can't
        // include this code in the main NerveMQ migrations. The `sqlite_kms_keys` table
        // should only be created if the sqlite key manager is used.
        sqlx::query(
            "
            CREATE TABLE IF NOT EXISTS nervemq_sqlite_kms_keys (
                key_id TEXT UNIQUE NOT NULL,
                key BLOB NOT NULL,

                PRIMARY KEY (key_id)
            )
            ",
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    /// Checks if a key with the given ID exists in the database.
    ///
    /// # Arguments
    /// * `key_id` - The ID of the key to check
    ///
    /// # Returns
    /// `true` if the key exists, `false` otherwise
    pub async fn key_exists(&self, key_id: &str) -> Result<bool, Error> {
        let exists = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM nervemq_sqlite_kms_keys WHERE key_id = $1)",
        )
        .bind(key_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(exists)
    }

    // let key = aes_gcm_siv::Key::<Aes256GcmSiv>::from_slice(&key);
    /// Retrieves an encryption key from the database.
    ///
    /// # Arguments
    /// * `key_id` - The ID of the key to retrieve
    ///
    /// # Returns
    /// The AES-256-GCM-SIV key for encryption/decryption operations
    ///
    /// # Errors
    /// Returns an error if the key doesn't exist or cannot be loaded
    pub async fn get_key(&self, key_id: &str) -> Result<aes_gcm_siv::Key<Aes256GcmSiv>, Error> {
        let key: Vec<u8> =
            sqlx::query_scalar("SELECT key FROM nervemq_sqlite_kms_keys WHERE key_id = $1")
                .bind(key_id)
                .fetch_one(&self.pool)
                .await?;
        Ok(aes_gcm_siv::Key::<Aes256GcmSiv>::from_slice(&key).to_owned())
    }
}

impl KeyManager for SqliteKeyManager {
    /// Encrypts data using AES-256-GCM-SIV.
    ///
    /// # Arguments
    /// * `key_id` - ID of the key to use for encryption
    /// * `data` - The data to encrypt
    ///
    /// # Implementation Details
    /// - Uses the key ID as the nonce (initialization vector)
    /// - Performs encryption in a separate blocking thread pool
    /// - Uses AES-256-GCM-SIV which provides both confidentiality and authenticity
    fn encrypt(
        &self,
        key_id: &String,
        data: Vec<u8>,
    ) -> Pin<Box<dyn Future<Output = eyre::Result<Vec<u8>>>>> {
        let self_clone = self.clone();
        let key_id = key_id.clone();
        Box::pin(async move {
            let key = self_clone.get_key(&key_id).await?;

            let encrypted = tokio::task::spawn_blocking({
                let key_id = key_id.clone();
                move || {
                    let nonce = Nonce::from_iter(key_id.bytes().cycle());

                    let cipher = Aes256GcmSiv::new(&key);

                    let encrypted = cipher
                        .encrypt(&nonce, data.as_ref())
                        .map_err(|e| eyre::eyre!("Error encrypting data: {e}"))?;

                    Result::<_, eyre::Report>::Ok(encrypted)
                }
            })
            .await??;

            Ok(encrypted.into())
        })
    }

    /// Decrypts data using AES-256-GCM-SIV.
    ///
    /// # Arguments
    /// * `key_id` - ID of the key used for encryption
    /// * `data` - The encrypted data to decrypt
    ///
    /// # Implementation Details
    /// - Uses the key ID as the nonce (must match encryption)
    /// - Performs decryption in a separate blocking thread pool
    /// - Verifies data authenticity during decryption
    fn decrypt(
        &self,
        key_id: &String,
        data: Vec<u8>,
    ) -> Pin<Box<dyn std::future::Future<Output = eyre::Result<Vec<u8>>>>> {
        let self_clone = self.clone();
        let key_id = key_id.clone();
        Box::pin(async move {
            let key = self_clone.get_key(&key_id).await?;

            let decrypted = tokio::task::spawn_blocking({
                let key_id = key_id.clone();
                move || {
                    let nonce = Nonce::from_iter(key_id.bytes().cycle());

                    let cipher = Aes256GcmSiv::new(&key);
                    let decrypted = cipher
                        .decrypt(&nonce, data.as_ref())
                        .map_err(|e| eyre::eyre!("Error decrypting data: {e}"))?;
                    Result::<_, eyre::Report>::Ok(decrypted)
                }
            })
            .await??;

            Ok(decrypted.into())
        })
    }

    /// Creates a new encryption key and stores it in the database.
    ///
    /// # Implementation Details
    /// - Generates a cryptographically secure random key
    /// - Creates a unique key ID using a 16-byte random token
    /// - Retries key ID generation if a collision occurs
    /// - Stores both the key and its ID in the SQLite database
    ///
    /// # Returns
    /// The ID of the newly created key
    fn create_key(&self) -> Pin<Box<dyn std::future::Future<Output = eyre::Result<String>>>> {
        let self_clone = self.clone();
        Box::pin(async move {
            let mut rng = rand::thread_rng();

            let mut key_buf = [0u8; 24];
            rand::RngCore::try_fill_bytes(&mut rng, &mut key_buf)?;

            let key = Aes256GcmSiv::generate_key(&mut rng);

            let key_id = loop {
                let key_id = generate_token::<16>(&mut rng)?;
                if !self_clone.key_exists(&key_id).await? {
                    break key_id;
                }
            };

            sqlx::query(
                "
                INSERT INTO nervemq_sqlite_kms_keys (key_id, key)
                VALUES ($1, $2)
                ",
            )
            .bind(&key_id)
            .bind(&key.as_slice())
            .execute(&self_clone.pool)
            .await?;

            Ok(key_id)
        })
    }

    /// Permanently deletes a key from the database.
    ///
    /// # Arguments
    /// * `key_id` - ID of the key to delete
    ///
    /// # Warning
    /// This operation is irreversible. Any data encrypted with this key
    /// will no longer be decryptable after the key is deleted.
    fn delete_key(
        &self,
        key_id: &String,
    ) -> Pin<Box<dyn std::future::Future<Output = eyre::Result<()>>>> {
        let self_clone = self.clone();
        let key_id = key_id.clone();
        Box::pin(async move {
            sqlx::query(
                "
                DELETE FROM nervemq_sqlite_kms_keys
                WHERE key_id = $1
                ",
            )
            .bind(key_id)
            .execute(&self_clone.pool)
            .await?;

            Ok(())
        })
    }
}
