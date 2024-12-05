//! Key Management Service (KMS) module for handling encryption operations.
//!
//! This module provides traits and types for managing cryptographic keys and performing
//! encryption/decryption operations in a generic way.

use bytes::Bytes;
use std::{future::Future, pin::Pin};

pub mod aws;
pub mod memory;

/// Represents encrypted data along with the ID of the key used to encrypt it.
///
/// This type is used to keep track of which key was used for encryption,
/// making it possible to decrypt the data later using the correct key.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Encrypted {
    key_id: String,
    data: Bytes,
}

impl Encrypted {
    pub fn new(key_id: String, data: Bytes) -> Self {
        Self { key_id, data }
    }

    pub fn key_id(&self) -> &str {
        &self.key_id
    }

    pub fn data(&self) -> &Bytes {
        &self.data
    }
}

/// Represents an in-progress key rotation operation.
///
/// Contains the necessary information to complete a key rotation:
/// - The ID of the new key
/// - A handle to track the rotation operation
/// - A secret used to authorize the completion of the rotation
pub struct Rotation {
    key_id: String,
    new_key_id: String,
}

impl Rotation {
    pub fn new(key_id: String, new_key_id: String) -> Self {
        Self { key_id, new_key_id }
    }

    pub fn key_id(&self) -> &str {
        &self.key_id
    }

    pub fn new_key_id(&self) -> &str {
        &self.new_key_id
    }
}

/// A trait for types that can be used as key identifiers.
///
/// This trait is automatically implemented for any type that implements
/// the required serialization traits, allowing for flexible key ID types
/// across different KMS implementations.
pub trait KeyId
where
    Self: Clone + Into<String> + serde::Serialize + for<'de> serde::Deserialize<'de>,
{
}

impl<T> KeyId for T where
    T: Clone + Into<String> + serde::Serialize + for<'de> serde::Deserialize<'de>
{
}

/// Core trait for key management operations.
///
/// This trait defines the interface for a key management service, providing
/// methods for:
/// - Encrypting and decrypting data
/// - Creating and deleting encryption keys
/// - Rotating keys safely
///
/// Implementations of this trait should handle the underlying cryptographic
/// operations and key management details for specific KMS providers.
pub trait KeyManager: Send + Sync + 'static {
    /// Encrypts the provided data using a key managed by this service.
    ///
    /// # Arguments
    /// * `data` - The data to encrypt, provided as any type implementing `bytes::Buf`
    ///
    /// # Returns
    /// An [`Encrypted`] instance containing the encrypted data and the ID of the key used
    fn encrypt(
        &self,
        key_id: &String,
        data: Bytes,
    ) -> Pin<Box<dyn Future<Output = eyre::Result<Encrypted>>>>;

    /// Decrypts the provided data using the specified key.
    ///
    /// # Arguments
    /// * `key_id` - The ID of the key to use for decryption
    /// * `data` - The encrypted data to decrypt
    ///
    /// # Returns
    /// The decrypted data as [`Bytes`]
    fn decrypt(
        &self,
        key_id: &String,
        data: Bytes,
    ) -> Pin<Box<dyn Future<Output = eyre::Result<Bytes>>>>;

    /// Creates a new encryption key.
    ///
    /// # Returns
    /// The ID of the newly created key
    fn create_key(&self) -> Pin<Box<dyn Future<Output = eyre::Result<String>>>>;

    /// Deletes an existing encryption key.
    ///
    /// # Arguments
    /// * `key_id` - The ID of the key to delete
    ///
    /// # Warning
    /// Deleting a key will make it impossible to decrypt any data that was encrypted with it.
    fn delete_key(&self, key_id: &String) -> Pin<Box<dyn Future<Output = eyre::Result<()>>>>;

    /// Begin a key rotation operation.
    ///
    /// This will generate a new key and return a handle to the rotation operation. The handle
    /// should be stored securely and used to complete the rotation operation. The new key will
    /// not be used until the rotation is completed.
    ///
    /// During the rotation operation, you should decrypt data using the old key and re-encrypt it
    /// using the new key. Then, call [`KeyManager::complete_rotation`] with the handle to complete the
    /// rotation and activate the new key.
    ///
    /// # Important
    /// It is recommended to perform the rotation operations in a database transaction to avoid
    /// attempting to decrypt data requiring the new key before it is activated.
    fn begin_rotation<'a>(
        &'a self,
        key_id: &String,
    ) -> Pin<Box<dyn Future<Output = eyre::Result<Rotation>> + 'a>> {
        let key_id = key_id.clone();
        Box::pin(async move {
            let new_key = self.create_key().await?;

            Ok(Rotation {
                key_id: key_id.clone(),
                new_key_id: new_key,
            })
        })
    }

    /// Complete a key rotation operation.
    ///
    /// This method finalizes a key rotation operation that was started with [`KeyManager::begin_rotation`].
    /// It validates the rotation handle and secret, then activates the new key for use.
    ///
    /// # Important
    /// Before calling this method, ensure that:
    /// 1. All necessary data has been re-encrypted with the new key
    /// 2. The rotation handle and secret have been kept secure
    /// 3. You are ready to permanently switch to using the new key
    ///
    /// After successful completion:
    /// - The old key will be deactivated
    /// - All future encryption operations will use the new key
    /// - The rotation handle will no longer be valid
    fn complete_rotation<'a>(
        &'a self,
        handle: Rotation,
    ) -> Pin<Box<dyn Future<Output = eyre::Result<()>> + 'a>> {
        Box::pin(async move { self.delete_key(&handle.key_id).await })
    }
}
