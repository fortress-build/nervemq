//! In-memory implementation of the Key Management Service.
//!
//! Provides a thread-safe, in-memory KMS using AES-GCM-SIV encryption.
//! Primarily used for testing and development environments.

use std::{future::Future, pin::Pin, sync::Arc};

use aes_gcm_siv::{aead::Aead, Aes256GcmSiv, KeyInit, Nonce};

use crate::auth::crypto::generate_token;

use super::KeyManager;

#[derive(Clone)]
/// Thread-safe in-memory key manager implementation.
///
/// Stores encryption keys in memory using a concurrent hash map.
pub struct InMemoryKeyManager {
    keys: Arc<papaya::HashMap<String, Arc<aes_gcm_siv::Key<Aes256GcmSiv>>>>,
}

impl InMemoryKeyManager {
    /// Creates a new empty key manager instance.
    pub fn new() -> Self {
        Self {
            keys: Arc::new(papaya::HashMap::new()),
        }
    }
}

impl KeyManager for InMemoryKeyManager {
    /// Encrypts data using AES-GCM-SIV with the specified key.
    fn encrypt(
        &self,
        key_id: &String,
        data: Vec<u8>,
    ) -> Pin<Box<dyn Future<Output = eyre::Result<Vec<u8>>>>> {
        let self_clone = self.clone();
        let key_id = key_id.clone();
        Box::pin(async move {
            let key = {
                let guard = self_clone.keys.guard();
                match self_clone.keys.get(&key_id, &guard).cloned() {
                    Some(key) => key,
                    None => return Err(eyre::eyre!("Key not found")),
                }
            };

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

    /// Decrypts AES-GCM-SIV encrypted data using the specified key.
    fn decrypt(
        &self,
        key_id: &String,
        data: Vec<u8>,
    ) -> Pin<Box<dyn std::future::Future<Output = eyre::Result<Vec<u8>>>>> {
        let self_clone = self.clone();
        let key_id = key_id.clone();
        Box::pin(async move {
            let key = {
                let guard = self_clone.keys.guard();
                match self_clone.keys.get(&key_id, &guard).cloned() {
                    Some(key) => key,
                    None => return Err(eyre::eyre!("Key not found")),
                }
            };

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

    /// Generates a new random encryption key with unique ID.
    fn create_key(&self) -> Pin<Box<dyn std::future::Future<Output = eyre::Result<String>>>> {
        let self_clone = self.clone();
        Box::pin(async move {
            let mut rng = rand::thread_rng();

            let mut key_buf = [0u8; 24];
            rand::RngCore::try_fill_bytes(&mut rng, &mut key_buf)?;

            let key = Aes256GcmSiv::generate_key(&mut rng);

            let key_id = loop {
                let key_id = generate_token::<16>(&mut rng)?;
                let guard = self_clone.keys.guard();
                if !self_clone.keys.contains_key(&key_id, &guard) {
                    break key_id;
                }
            };

            let guard = self_clone.keys.guard();
            self_clone
                .keys
                .insert(key_id.clone(), Arc::new(key), &guard);

            Ok(key_id)
        })
    }

    /// Removes a key from the in-memory store.
    fn delete_key(
        &self,
        key_id: &String,
    ) -> Pin<Box<dyn std::future::Future<Output = eyre::Result<()>>>> {
        let self_clone = self.clone();
        let key_id = key_id.clone();
        Box::pin(async move {
            self_clone.keys.pin().remove(&key_id);
            Ok(())
        })
    }
}
