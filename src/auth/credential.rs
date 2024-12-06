use actix_web::{FromRequest, HttpMessage};
use secrecy::SecretString;
use serde::{Deserialize, Serialize};

use crate::error::Error;

/// Namespace authorized for the request.
///
/// Included in request-local extension data once authorized.
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

/// Request to create a new API key.
#[derive(Serialize, Deserialize)]
pub struct ApiKeyRequest {
    api_key: String,
}

/// Prefix for API keys for identification.
pub const API_KEY_PREFIX: &str = "nervemq";

/// Request to delete an API key.
#[derive(Debug)]
pub struct ApiKey {
    /// For AWS Sigv4, this is the access key ID
    pub short_token: String,
    /// For AWS Sigv4, this is the secret access key
    pub long_token: SecretString,
}

impl ApiKey {
    /// Creates a new API key with the specified short and long tokens.
    pub fn new(short_token: String, long_token: SecretString) -> Self {
        Self {
            short_token,
            long_token,
        }
    }
}
