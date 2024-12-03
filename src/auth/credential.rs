use actix_web::{FromRequest, HttpMessage};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
pub struct AuthorizedNamespace(pub String);

impl FromRequest for AuthorizedNamespace {
    type Error = Error;

    type Future = std::future::Ready<Result<AuthorizedNamespace, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        std::future::ready(
            req.extensions()
                .get::<AuthorizedNamespace>()
                .cloned()
                .ok_or(Error::Unauthorized),
        )
    }
}

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
