//! AWS KMS (Key Management Service) implementation.
//!
//! Provides encryption, decryption, and key management operations using
//! AWS KMS as the backend service. Implements the KeyManager trait for
//! seamless integration with the rest of the system.

use std::{future::Future, pin::Pin};

use aws_sdk_kms::operation::encrypt::EncryptOutput;

/// AWS KMS implementation of the KeyManager trait.
///
/// Manages cryptographic operations using AWS KMS service,
/// supporting symmetric encryption/decryption and key lifecycle.
pub struct AwsKeyManager {
    client: aws_sdk_kms::Client,
}

impl AwsKeyManager {
    /// Creates a new AWS KMS manager with the provided client.
    pub fn new(client: aws_sdk_kms::Client) -> Self {
        Self { client }
    }
}

impl super::KeyManager for AwsKeyManager {
    /// Encrypts data using AWS KMS with the specified key.
    ///
    /// Uses symmetric encryption with the default algorithm.
    fn encrypt(
        &self,
        key_id: &String,
        data: Vec<u8>,
    ) -> Pin<Box<dyn Future<Output = eyre::Result<Vec<u8>>>>> {
        let client = self.client.clone();
        let key_id = key_id.clone();

        Box::pin(async move {
            let EncryptOutput {
                ciphertext_blob, ..
            } = client
                .encrypt()
                .key_id(&key_id)
                .plaintext(aws_sdk_kms::primitives::Blob::new(data))
                .encryption_algorithm(aws_sdk_kms::types::EncryptionAlgorithmSpec::SymmetricDefault)
                .send()
                .await?;

            let encrypted = match ciphertext_blob {
                Some(blob) => blob.into_inner(),
                None => {
                    return Err(eyre::eyre!("No ciphertext blob in response"));
                }
            };

            Ok(encrypted)
        })
    }

    /// Decrypts KMS-encrypted data using the specified key.
    fn decrypt(
        &self,
        key_id: &String,
        data: Vec<u8>,
    ) -> Pin<Box<dyn Future<Output = eyre::Result<Vec<u8>>>>> {
        let client = self.client.clone();
        let key_id = key_id.clone();
        Box::pin(async move {
            let decrypted = client
                .decrypt()
                .key_id(key_id)
                .ciphertext_blob(aws_sdk_kms::primitives::Blob::new(data))
                .send()
                .await?
                .plaintext
                .ok_or_else(|| eyre::eyre!("No plaintext in response"))?;

            Ok(decrypted.into_inner())
        })
    }

    /// Creates a new KMS key for encryption/decryption.
    fn create_key(&self) -> Pin<Box<dyn Future<Output = eyre::Result<String>>>> {
        let client = self.client.clone();
        Box::pin(async move {
            let res = client
                .create_key()
                .key_usage(aws_sdk_kms::types::KeyUsageType::EncryptDecrypt)
                .send()
                .await?;

            let Some(key_meta) = res.key_metadata else {
                return Err(eyre::eyre!("No key ID in response"));
            };

            Ok(key_meta.key_id)
        })
    }

    /// Schedules deletion of a KMS key.
    ///
    /// Note: This initiates key deletion with AWS KMS's standard
    /// waiting period before actual deletion.
    fn delete_key(&self, key_id: &String) -> Pin<Box<dyn Future<Output = eyre::Result<()>>>> {
        let client = self.client.clone();
        let key_id = key_id.clone();
        Box::pin(async move {
            client.schedule_key_deletion().key_id(key_id).send().await?;
            Ok(())
        })
    }
}
