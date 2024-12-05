//! SQS API method parsing and validation.
//!
//! This module handles the parsing and validation of AWS SQS API methods
//! from HTTP requests. It supports the standard SQS method format:
//! `AmazonSQS.{MethodName}` (e.g., "AmazonSQS.SendMessage").
//!
//! The module implements:
//! - Method enumeration and parsing
//! - Request extraction for Actix-web
//! - Validation of SQS method formats
//!
//! # Format
//! All valid SQS methods must:
//! 1. Start with the prefix "AmazonSQS."
//! 2. Be followed by a valid method name
//! 3. Match exactly one of the supported operations

use std::str::FromStr;

use actix_web::{FromRequest, HttpMessage};
use pom::utf8::{end, seq, sym};
use strum::EnumString;

use crate::{error::Error, utils::to_pom_error};

/// Standard prefix for all SQS API method names.
///
/// All valid SQS method strings must start with this prefix followed by a dot.
/// Example: "AmazonSQS.SendMessage"
pub const SQS_METHOD_PREFIX: &str = "AmazonSQS";

/// Represents an SQS API method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString)]
pub enum Method {
    // AddPermission,                // TODO: Implement
    // CancelMessageMoveTask,        // TODO: Implement
    // ChangeMessageVisibility,      // TODO: Implement
    // ChangeMessageVisibilityBatch, // TODO: Implement
    CreateQueue,
    DeleteMessage,
    DeleteMessageBatch,
    DeleteQueue,
    GetQueueAttributes,
    GetQueueUrl,
    // ListDeadLetterSourceQueues,   // TODO: Implement
    // ListMessageMoveTasks,         // TODO: Implement
    ListQueues,
    ListQueueTags,
    PurgeQueue,
    ReceiveMessage,
    // RemovePermission,             // TODO: Implement
    SendMessage,
    SendMessageBatch,
    SetQueueAttributes,
    // StartMessageMoveTask,         // TODO: Implement
    TagQueue,
    UntagQueue,
}

impl Method {
    /// Parses an SQS API method from a string.
    pub fn parse(input: &str) -> Result<Self, Error> {
        let method = pom::utf8::Parser::new(|bytes, position| {
            let (method, consumed) = std::str::from_utf8(&bytes[position..])
                .map_err(|e| to_pom_error(e, position, "Invalid UTF-8"))
                .and_then(|s| {
                    Method::from_str(s)
                        .map_err(|e| to_pom_error(e, position, "Invalid method"))
                        .map(|m| (m, s.len()))
                })?;
            Ok((method, position + consumed))
        });

        let parser = seq(SQS_METHOD_PREFIX) * sym('.') * method - end();

        parser.parse_str(input).map_err(|e| Error::InvalidMethod {
            message: format!("{e}"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_method_valid() {
        let test_cases = vec![
            ("AmazonSQS.SendMessage", Method::SendMessage),
            ("AmazonSQS.SendMessageBatch", Method::SendMessageBatch),
            ("AmazonSQS.ReceiveMessage", Method::ReceiveMessage),
            ("AmazonSQS.DeleteMessage", Method::DeleteMessage),
            ("AmazonSQS.ListQueues", Method::ListQueues),
            ("AmazonSQS.GetQueueUrl", Method::GetQueueUrl),
            ("AmazonSQS.CreateQueue", Method::CreateQueue),
            ("AmazonSQS.GetQueueAttributes", Method::GetQueueAttributes),
            ("AmazonSQS.PurgeQueue", Method::PurgeQueue),
        ];

        for (input, expected) in test_cases {
            let result = Method::parse(input);
            assert!(
                result.is_ok(),
                "Failed to parse valid method: {} ({})",
                input,
                result.unwrap_err()
            );
            assert_eq!(
                result.unwrap(),
                expected,
                "Method mismatch for input: {}",
                input
            );
        }
    }

    #[test]
    fn test_parse_method_invalid() {
        let invalid_inputs = vec![
            "SendMessage",             // Missing prefix
            "AmazonSQS",               // Missing method
            "AmazonSQS.",              // Empty method
            "AmazonSQS.InvalidMethod", // Non-existent method
            "Amazon.SendMessage",      // Wrong prefix
            "",                        // Empty string
        ];

        for input in invalid_inputs {
            let result = Method::parse(input);
            assert!(
                result.is_err(),
                "Expected error for invalid input: {}",
                input
            );

            match result {
                Err(Error::InvalidMethod { .. }) => {}
                _ => panic!("Expected InvalidHeader error for input: {}", input),
            }
        }
    }
}

impl FromRequest for Method {
    type Error = Error;

    type Future = std::future::Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        std::future::ready(req.extensions().get::<Method>().cloned().ok_or_else(|| {
            Error::MissingHeader {
                header: "X-Amz-Target".to_owned(),
            }
        }))
    }
}
