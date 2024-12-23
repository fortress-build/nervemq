//! Authentication header parsing module.
//!
//! This module provides parsers for authentication headers supporting two schemes:
//! - NerveMqApiV1: Custom API key-based authentication
//! - AWS SigV4: AWS-style request signing authentication
//!
//! The parsing is implemented using the `pom` parser combinator library.

use pom::utf8::{end, list, none_of, one_of, seq, sym, Parser};
use secrecy::SecretString;

use super::{
    credential::{ApiKey, API_KEY_PREFIX},
    protocols::sigv4::SigV4Header,
};

/// Represents supported authentication schemes.
pub enum AuthScheme {
    NerveMqApiV1,
    AWSv4 {
        #[allow(unused)]
        algorithm: String,
    },
}

#[derive(Debug)]
/// Parsed authentication header containing credentials and metadata.
///
/// The lifetime parameter 'a refers to the borrowed string data from the original header.
pub enum AuthHeader<'a> {
    /// NerveMQ API key authentication credentials
    /// AWS Signature Version 4 authentication credentials and metadata
    NerveMqApiV1(ApiKey),
    AWSv4(SigV4Header<'a>),
}

#[allow(unused)]
/// Parser for authentication scheme identifiers.
///
/// Recognizes either "NerveMqApiV1" or AWS-style algorithm strings (e.g., "AWS4-HMAC-SHA256").
pub fn auth_scheme<'a>() -> Parser<'a, AuthScheme> {
    let api = seq("NerveMqApiV1")
        .map(|_| AuthScheme::NerveMqApiV1)
        .name("nervemq api");

    let sqs_algo = ((seq("AWS4") - sym('-')) * (alphanumeric() | sym('-')).repeat(1..).collect())
        .map(|s| AuthScheme::AWSv4 {
            algorithm: s.to_owned(),
        });

    (api | sqs_algo).name("auth scheme")
}

/// Parser for basic tokens consisting of alphanumeric characters.
///
/// Used as a building block for more complex token patterns.
pub fn token<'a>() -> Parser<'a, &'a str> {
    one_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890")
        .repeat(1..)
        .collect()
}

/// Parser for tokens in the format "prefix_short_long".
///
/// Returns a tuple of (prefix, short token, long token).
/// Used primarily for NerveMQ API key parsing.
pub fn prefixed_token<'a>() -> Parser<'a, (&'a str, &'a str, &'a str)> {
    let prefix = token();

    let short = sym('_') * token();
    let long = sym('_') * token();

    (prefix + short + long).map(|((prefix, short), long)| (prefix, short, long))
}

/// Parser for whitespace characters (space, tab, newline, carriage return).
pub fn whitespace<'a>() -> Parser<'a, char> {
    one_of(" \r\n\t")
}

/// Parser for alphabetic characters (a-z, A-Z).
pub fn alpha<'a>() -> Parser<'a, char> {
    one_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ")
}

/// Parser for numeric characters (0-9).
pub fn numeric<'a>() -> Parser<'a, char> {
    one_of("0123456789")
}

/// Parser for alphanumeric characters (combination of alpha and numeric).
pub fn alphanumeric<'a>() -> Parser<'a, char> {
    alpha() | numeric()
}

/// Parser for NerveMQ API v1 authentication headers.
///
/// Expects format: "NerveMqApiV1 prefix_shorttoken_longtoken"
/// Validates the prefix and constructs an ApiKey from the tokens.
fn nervemq_api_v1<'a>() -> Parser<'a, AuthHeader<'a>> {
    let tag = seq("NerveMqApiV1");
    let space = sym(' ').repeat(1..).discard();

    ((tag + space) * prefixed_token() - end())
        .convert(|(prefix, short, long)| {
            if prefix != API_KEY_PREFIX {
                return Err("invalid prefix");
            }

            Ok(AuthHeader::NerveMqApiV1(ApiKey::new(
                short.to_owned(),
                SecretString::from(long),
            )))
        })
        .name("nervemq api v1")
}

/// Parser for AWS Signature Version 4 authentication headers.
///
/// Expects format: "AWS4-HMAC-SHA256 Credential=...,SignedHeaders=...,Signature=..."
/// Parses and validates all required components of the SigV4 authentication scheme.
fn sigv4<'a>() -> Parser<'a, AuthHeader<'a>> {
    let tag = seq("AWS4-HMAC-SHA256");

    let space = sym(' ');

    // TODO: Switch to nom, chumsky or another parser library with an API not based on operator overloads. This is incomprehensible.

    let access_key_id = token();
    let yyyymmdd = numeric().repeat(8).collect();
    let region = (alphanumeric() | sym('-')).repeat(1..).collect();
    let service = (alphanumeric() | sym('-')).repeat(1..).collect();

    let credential_parser = (((access_key_id - sym('/'))
        + (yyyymmdd - sym('/'))
        + (region - sym('/'))
        + (service - sym('/')))
        - seq("aws4_request"))
    .map(|(((access_key_id, yyyymmdd), region), service)| {
        (access_key_id, yyyymmdd, region, service)
    });

    let signed_headers_parser = list((alphanumeric() | sym('-')).repeat(1..).collect(), sym(';'));

    let kv = list(
        (((alphanumeric() | sym('-')).repeat(1..).collect()) - sym('='))
            + none_of(",").repeat(1..).collect(),
        (whitespace().repeat(0..) * sym(',')) - whitespace().repeat(0..),
    )
    .convert(move |items| {
        let (mut creds, mut signed_headers, mut signature) = (None, None, None);
        for (k, v) in items {
            match k {
                "Credential" => creds = Some(credential_parser.parse_str(v)),
                "SignedHeaders" => signed_headers = Some(signed_headers_parser.parse_str(v)),
                "Signature" => signature = Some(v),
                _ => (),
            }
        }
        let (Some(creds), Some(signed_headers), Some(signature)) =
            (creds.clone(), signed_headers.clone(), signature)
        else {
            return Err(format!(
                "missing required parameters: {}",
                vec![
                    creds.map(|_| "creds ok").unwrap_or("creds"),
                    signed_headers.map(|_| "headers ok").unwrap_or("headers"),
                    signature.map(|_| "signature ok").unwrap_or("signature"),
                ]
                .join(", ")
            ));
        };
        Ok((creds, signed_headers, signature))
    });

    (((tag - space.repeat(1..)) + kv) - end())
        .convert(|(tag, (credential, signed_headers, signature))| {
            let (key_id, yyyymmdd, region, service) = credential?;
            let signed_headers = signed_headers?;

            Result::<_, pom::Error>::Ok(AuthHeader::AWSv4(SigV4Header {
                algorithm: tag,
                key_id,
                date: yyyymmdd,
                signed_headers,
                signature,
                region,
                service,
            }))
        })
        .name("sqs api credential")
}

/// Main parser for authentication headers.
///
/// Attempts to parse either a NerveMQ API v1 or AWS SigV4 authentication header.
/// Returns the parsed authentication information in an AuthHeader enum.
pub fn auth_header<'a>() -> Parser<'a, AuthHeader<'a>> {
    (nervemq_api_v1() | sigv4()).name("auth header")
}

#[cfg(test)]
mod tests {
    use secrecy::ExposeSecret;

    use super::*;

    #[test]
    fn test_nervemq_api_v1_valid() {
        let input = "NerveMqApiV1 nervemq_abcABC12_abcabcabcabcabcABCABC234";
        let result = auth_header().parse(input.as_bytes());

        assert!(result.is_ok(), "{:?}", result.err());

        if let Ok(AuthHeader::NerveMqApiV1(token)) = result {
            assert_eq!(token.short_token, "abcABC12");
            assert_eq!(token.short_token.len(), 8);

            assert_eq!(token.long_token.expose_secret(), "abcabcabcabcabcABCABC234");
            assert_eq!(token.long_token.expose_secret().len(), 24);
        } else {
            panic!("Expected NerveMqApiV1 variant");
        }
    }

    #[test]
    fn test_nervemq_api_v1_invalid() {
        // Missing space after scheme
        let input = "NerveMqApiV1abcdef1234567890abcdef1234567890.abcdef1234567890abcdef1234567890";
        assert!(auth_header().parse(input.as_bytes()).is_err());

        // Missing dot separator
        let input = "NerveMqApiV1 abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890";
        assert!(auth_header().parse(input.as_bytes()).is_err());

        // Invalid characters
        let input = "NerveMqApiV1 abcdef!@#$%^&*.abcdef1234567890abcdef1234567890";
        assert!(auth_header().parse(input.as_bytes()).is_err());
    }

    #[test]
    fn test_auth_scheme_parser() {
        assert!(matches!(
            auth_scheme().parse(b"NerveMqApiV1"),
            Ok(AuthScheme::NerveMqApiV1)
        ));

        assert!(auth_scheme().parse(b"Invalid").is_err());
    }

    #[test]
    fn test_token_parser() {
        assert_eq!(token().parse(b"abc123ABC"), Ok("abc123ABC"));

        assert!((token() - end()).parse(b"").is_err());
        assert!((token() - end()).parse(b"abc!@#").is_err());
    }

    #[test]
    fn test_aws_v4_valid() {
        let input = "AWS4-HMAC-SHA256 Credential=AKIAIOSFODNN7EXAMPLE/20230815/us-east-1/sqs/aws4_request,SignedHeaders=content-type;host;x-amz-date,Signature=e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        let result = auth_header().parse(input.as_bytes());

        assert!(result.is_ok(), "{:?}", result.err());

        if let Ok(AuthHeader::AWSv4(SigV4Header {
            algorithm,
            key_id: access_key,
            date,
            signed_headers,
            signature,
            region,
            service,
        })) = result
        {
            assert_eq!(algorithm, "AWS4-HMAC-SHA256");
            assert_eq!(access_key, "AKIAIOSFODNN7EXAMPLE");
            assert_eq!(date, "20230815");
            assert_eq!(signed_headers, vec!["content-type", "host", "x-amz-date"]);
            assert_eq!(
                signature,
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
            );
            assert_eq!(region, "us-east-1");
            assert_eq!(service, "sqs");
        } else {
            panic!("Expected AWSv4 variant");
        }
    }

    #[test]
    fn test_aws_v4_invalid() {
        // Missing required Credential parameter
        let input = "AWS4-HMAC-SHA256 SignedHeaders=content-type;host;x-amz-date;Signature=abc123";
        assert!(auth_header().parse(input.as_bytes()).is_err());

        // Invalid credential format
        let input = "AWS4-HMAC-SHA256 Credential=INVALID_FORMAT;SignedHeaders=content-type;Signature=abc123";
        assert!(auth_header().parse(input.as_bytes()).is_err());

        // Missing SignedHeaders
        let input = "AWS4-HMAC-SHA256 Credential=AKIAIOSFODNN7EXAMPLE/20230815/us-east-1/sqs/aws4_request;Signature=abc123";
        assert!(auth_header().parse(input.as_bytes()).is_err());

        // Missing Signature
        let input = "AWS4-HMAC-SHA256 Credential=AKIAIOSFODNN7EXAMPLE/20230815/us-east-1/sqs/aws4_request;SignedHeaders=content-type";
        assert!(auth_header().parse(input.as_bytes()).is_err());

        // Invalid algorithm
        let input = "AWS4-INVALID Credential=AKIAIOSFODNN7EXAMPLE/20230815/us-east-1/sqs/aws4_request;SignedHeaders=content-type;Signature=abc123";
        assert!(auth_header().parse(input.as_bytes()).is_err());
    }
}
