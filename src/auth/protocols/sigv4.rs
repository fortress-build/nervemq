//! AWS Signature Version 4 (SigV4) authentication implementation.
//!
//! This module provides functionality to authenticate requests using the AWS SigV4 protocol.
//! It verifies request signatures created using AWS-style credentials, following the same
//! signing process as AWS services.
//!
//! # Protocol Overview
//! SigV4 authentication involves:
//! 1. Creating a canonical request from the HTTP request
//! 2. Creating a string to sign using the canonical request
//! 3. Calculating the signature using a signing key
//! 4. Comparing the calculated signature with the provided signature
//!
//! For more details, see [AWS Signature Version 4 signing process](https://docs.aws.amazon.com/general/latest/gr/signature-version-4.html)

use std::{pin::Pin, time::SystemTime};

use actix_web::{
    dev::ServiceRequest,
    web::{self},
    HttpMessage,
};
use aws_sigv4::sign::v4::generate_signing_key;
use bytes::BytesMut;
use futures_util::TryStreamExt;
use hmac::{digest::FixedOutput, Mac};
use itertools::Itertools;
use sha2::Sha256;
use tracing::instrument;

use crate::{
    api::auth::User,
    auth::{credential::AuthorizedNamespace, crypto::sha256_hex},
    error::Error,
};

/// Represents the parsed components of an AWS SigV4 Authorization header.
///
/// This struct contains all the necessary information extracted from the
/// Authorization header required to verify the request signature.
#[derive(Debug)]
pub struct SigV4Header<'a> {
    /// The signing algorithm (typically "AWS4-HMAC-SHA256")
    pub algorithm: &'a str,
    /// The access key ID used to sign the request
    pub key_id: &'a str,
    /// The date when the signature was created (YYYYMMDD format)
    pub date: &'a str,
    /// List of headers included in the signature
    pub signed_headers: Vec<&'a str>,
    /// The request signature to verify
    pub signature: &'a str,
    /// AWS region name used in signing
    pub region: &'a str,
    /// AWS service name used in signing
    pub service: &'a str,
}

/// Authenticates a request using AWS Signature Version 4.
///
/// This function verifies the signature of an incoming request and returns the associated
/// user and namespace if authentication succeeds.
///
/// # Arguments
/// * `service` - Application service instance containing KMS and other configurations
/// * `req` - The incoming service request to authenticate
/// * `header` - Parsed SigV4 authorization header components
///
/// # Returns
/// * `Ok((User, AuthorizedNamespace))` - The authenticated user and their authorized namespace
/// * `Err(Error)` - If authentication fails for any reason
///
/// # Authentication Process
/// 1. Retrieves and validates the API key from the database
/// 2. Decrypts the signing key using KMS
/// 3. Reconstructs the canonical request
/// 4. Generates the signature using the same process as the client
/// 5. Compares the generated signature with the provided signature
///
/// # Errors
/// * `Error::IdentityNotFound` - If the provided key ID doesn't exist
/// * `Error::MissingHeader` - If a required header is missing
/// * `Error::InvalidHeader` - If a header value is invalid
/// * `Error::Unauthorized` - If the signature verification fails
///
///
/// For implementation details, see [The AWS Signature Version 4 Signing Process](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_sigv-create-signed-request.html)
#[instrument(skip(service, req))]
pub async fn authenticate_sigv4(
    service: web::Data<crate::service::Service>,
    req: &mut ServiceRequest,
    header: SigV4Header<'_>,
) -> Result<(User, AuthorizedNamespace), Error> {
    let payload = {
        let payload = req.take_payload();

        let bytes = payload
            .try_fold(BytesMut::new(), |mut acc, chunk| async move {
                acc.extend_from_slice(&chunk);
                Ok(acc)
            })
            .await
            .map_err(|e| {
                tracing::error!("Error reading request payload: {}", e);
                Error::internal(e)
            })?
            .freeze();

        bytes
    };

    let pool = req
        .app_data::<web::Data<crate::service::Service>>()
        .expect("SQLite pool not found. This is a bug.")
        .db()
        .clone();

    let Some((encrypted_key, namespace, user_email)) =
        sqlx::query_as::<_, (Vec<u8>, String, String)>(
            "
            SELECT k.encrypted_key, ns.name, u.email FROM api_keys k
            JOIN namespaces ns ON ns.id = k.ns
            JOIN users u ON u.id = k.user
            WHERE key_id = $1
            ",
        )
        .bind(&header.key_id)
        .fetch_optional(&pool)
        .await?
    else {
        return Err(Error::IdentityNotFound {
            key_id: header.key_id.to_string(),
        }
        .into());
    };

    let kms_key_id = service.get_key_id(&user_email).await?;

    let x_amz_date = req
        .headers()
        .get("x-amz-date")
        .ok_or_else(|| Error::MissingHeader {
            header: "x-amz-date".to_string(),
        })?
        .to_str()
        .map_err(Error::internal)?;

    let payload_hash = sha256_hex(&payload);

    let signing_key = generate_signing_key(
        std::str::from_utf8(&service.kms().decrypt(&kms_key_id, encrypted_key).await?)
            .expect("kms key is not utf8"),
        // time.into(),
        SystemTime::now(),
        header.region,
        header.service,
    );

    // The URL path, url-encoded, with the leading slash left in place (not encoded)
    let canonical_uri = req.uri().path();

    // Alphabetically-sorted query string parameters, url-encoded.
    //
    // Query parameters without values should be included with an equal sign (e.g., `key=` for `/?key`).
    let canonical_query = req
        .query_string()
        .split('&')
        .filter(|param| !param.is_empty())
        .map(|param| {
            let mut parts = param.split('=');
            let key = parts.next().unwrap_or("");
            let value = parts.next().unwrap_or("");
            (urlencoding::encode(key), urlencoding::encode(value))
        })
        .sorted_by_key(|(k, _)| k.to_string())
        .map(|(k, v)| format!("{}={}", k, v))
        .join("&");

    // Alphabetically sort list of included headers
    let sorted_signed_headers = header
        .signed_headers
        .into_iter()
        .sorted()
        .map(|h| h.to_lowercase())
        .collect_vec();

    // Alphabetically-sorted headers, with the header name in lowercase, followed by a colon and
    // then the header value, separated by a newline.
    let canonical_headers = sorted_signed_headers
        .iter()
        .map(|header| {
            let value = req
                .headers()
                .get(header)
                .ok_or_else(|| Error::MissingHeader {
                    header: header.to_string(),
                })?
                .to_str()
                .map_err(|e| {
                    tracing::error!("Invalid header value: {}", e);

                    Error::InvalidHeader {
                        header: header.to_string(),
                    }
                })?;

            let canonical_value = value.trim().split_whitespace().join(" ");

            Ok(format!("{}:{}\n", header, canonical_value))
        })
        .collect::<Result<Vec<String>, Error>>()?
        .join("");

    // The list of included headers, separated by semicolon
    let signed_headers = sorted_signed_headers.join(";");

    let canonical_request = [
        &req.method().to_string(),
        &*canonical_uri,
        &canonical_query,
        &canonical_headers,
        &signed_headers,
        &payload_hash,
    ]
    .join("\n");

    let canonical_request_hash = sha256_hex(canonical_request.as_bytes());

    let credential_scope = [header.date, header.region, header.service, "aws4_request"].join("/");

    // Final string to sign; signature = HEX(HMAC-SHA256(string_to_sign))
    let string_to_sign = [
        header.algorithm,
        x_amz_date,
        &credential_scope,
        &canonical_request_hash,
    ]
    .join("\n");

    let generated_signature = {
        let mut mac = hmac::Hmac::<Sha256>::new_from_slice(signing_key.as_ref())
            .map_err(|e| Error::internal(e))?;

        mac.update(string_to_sign.as_bytes());

        hex::encode(mac.finalize_fixed())
    };

    // IMPORTANT: We must duplicate the payload and return it to the request,
    // since it may be needed by route handlers or other middleware.
    //
    // We probably don't need this if authorization fails, but return it to the request before
    // validating the hash just for consistency/sanity.
    req.set_payload(actix_web::dev::Payload::Stream {
        payload: Box::pin(futures_util::stream::once(std::future::ready(Ok(payload))))
            as Pin<Box<dyn futures_util::Stream<Item = Result<_, actix_web::error::PayloadError>>>>,
    });

    if header.signature != generated_signature {
        tracing::debug!(
            provided = header.signature,
            generated = generated_signature,
            "Invalid signature for request",
        );

        return Err(Error::Unauthorized);
    }

    tracing::debug!(
        key_id = header.key_id,
        namespace = namespace,
        user_email = user_email,
        "Request authenticated successfully"
    );

    let user: User = sqlx::query_as(
        "
        SELECT u.* FROM api_keys k
        JOIN users u ON u.id = k.user
        WHERE k.key_id = $1
        ",
    )
    .bind(&header.key_id)
    .fetch_one(&pool)
    .await?;

    Ok((user, AuthorizedNamespace(namespace)))
}
