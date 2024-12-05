use std::{future::Future, pin::Pin};

use aws_sdk_kms::operation::encrypt::EncryptOutput;
use bytes::Bytes;

pub struct AwsKeyManager {
    client: aws_sdk_kms::Client,
}

impl AwsKeyManager {
    pub fn new(client: aws_sdk_kms::Client) -> Self {
        Self { client }
    }
}

impl super::KeyManager for AwsKeyManager {
    fn encrypt(
        &self,
        key_id: &String,
        data: Bytes,
    ) -> Pin<Box<dyn Future<Output = eyre::Result<super::Encrypted>>>> {
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

            Ok(super::Encrypted {
                key_id,
                data: Bytes::from(encrypted),
            })
        })
    }

    fn decrypt(
        &self,
        key_id: &String,
        data: Bytes,
    ) -> Pin<Box<dyn Future<Output = eyre::Result<bytes::Bytes>>>> {
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

            Ok(Bytes::from(decrypted.into_inner()))
        })
    }

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

    fn delete_key(&self, key_id: &String) -> Pin<Box<dyn Future<Output = eyre::Result<()>>>> {
        let client = self.client.clone();
        let key_id = key_id.clone();
        Box::pin(async move {
            client.schedule_key_deletion().key_id(key_id).send().await?;
            Ok(())
        })
    }
}
