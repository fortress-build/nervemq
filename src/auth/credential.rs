use secrecy::SecretString;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ApiKeyRequest {
    api_key: String,
}

pub const API_KEY_PREFIX: &str = "nervemq";

#[derive(Debug)]
pub struct ApiKey {
    /// For AWS Sigv4, this is the access key ID
    pub short_token: String,
    /// For AWS Sigv4, this is the secret access key
    pub long_token: SecretString,
}

impl ApiKey {
    pub fn new(short_token: String, long_token: SecretString) -> Self {
        Self {
            short_token,
            long_token,
        }
    }

    pub fn short_token(&self) -> &str {
        &self.short_token
    }

    pub fn long_token(&self) -> &SecretString {
        &self.long_token
    }
}
