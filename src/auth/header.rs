use pom::utf8::{any, end, one_of, seq, sym, Parser};
use prefixed_api_key::PrefixedApiKey;

pub enum AuthScheme {
    NerveMqApiV1,
    Bearer,
}

pub enum AuthHeader<'a> {
    NerveMqApiV1 { token: PrefixedApiKey },
    Bearer { token: &'a str },
}

#[allow(unused)]
pub fn auth_scheme<'a>() -> Parser<'a, AuthScheme> {
    let api = seq("NerveMqApiV1")
        .map(|_| AuthScheme::NerveMqApiV1)
        .name("nervemq api");

    let bearer = seq("Bearer").map(|_| AuthScheme::Bearer).name("bearer");

    (api | bearer).name("auth scheme")
}

pub fn token<'a>() -> Parser<'a, &'a str> {
    one_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890")
        .repeat(1..)
        .collect()
}

pub fn prefixed_token<'a>() -> Parser<'a, (&'a str, &'a str, &'a str)> {
    let prefix = token();

    let short = sym('_') * token();
    let long = sym('_') * token();

    (prefix + short + long).map(|((prefix, short), long)| (prefix, short, long))
}

pub fn whitespace<'a>() -> Parser<'a, &'a str> {
    sym(' ').repeat(1..).collect()
}

fn nervemq_api_v1<'a>() -> Parser<'a, AuthHeader<'a>> {
    let tag = seq("NerveMqApiV1");
    let space = sym(' ').repeat(1..);

    ((tag + space) * prefixed_token() - end())
        .map(|(prefix, short, long)| AuthHeader::NerveMqApiV1 {
            token: PrefixedApiKey::new(prefix.to_owned(), short.to_owned(), long.to_owned()),
        })
        .name("nervemq api v1")
}

fn bearer<'a>() -> Parser<'a, AuthHeader<'a>> {
    let tag = seq("Bearer");
    let space = sym(' ').repeat(1..);
    let rest = any().repeat(1..).collect();

    ((tag + space) * rest - end())
        .map(|token| AuthHeader::Bearer { token })
        .name("bearer auth")
}

pub fn auth_header<'a>() -> Parser<'a, AuthHeader<'a>> {
    (nervemq_api_v1() | bearer()).name("auth header")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nervemq_api_v1_valid() {
        let input = "NerveMqApiV1 nervemq_abcABC12_abcabcabcabcabcABCABC234";
        let result = auth_header().parse(input.as_bytes());

        assert!(result.is_ok(), "{:?}", result.err());

        if let Ok(AuthHeader::NerveMqApiV1 { token }) = result {
            assert_eq!(token.prefix(), "nervemq");

            assert_eq!(token.short_token(), "abcABC12");
            assert_eq!(token.short_token().len(), 8);

            assert_eq!(token.long_token(), "abcabcabcabcabcABCABC234");
            assert_eq!(token.long_token().len(), 24);
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
    fn test_bearer_valid() {
        let input = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0";
        let result = auth_header().parse(input.as_bytes());
        assert!(result.is_ok());

        if let Ok(AuthHeader::Bearer { token }) = result {
            assert_eq!(
                token,
                "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0"
            );
        } else {
            panic!("Expected Bearer variant");
        }
    }

    #[test]
    fn test_bearer_invalid() {
        // Missing space after Bearer
        let input = "BearereyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        assert!(auth_header().parse(input.as_bytes()).is_err());

        // Empty token
        let input = "Bearer ";
        assert!(auth_header().parse(input.as_bytes()).is_err());
    }

    #[test]
    fn test_auth_scheme_parser() {
        assert!(matches!(
            auth_scheme().parse(b"NerveMqApiV1"),
            Ok(AuthScheme::NerveMqApiV1)
        ));

        assert!(matches!(
            auth_scheme().parse(b"Bearer"),
            Ok(AuthScheme::Bearer)
        ));

        assert!(auth_scheme().parse(b"Invalid").is_err());
    }

    #[test]
    fn test_token_parser() {
        assert_eq!(token().parse(b"abc123ABC"), Ok("abc123ABC"));

        assert!((token() - end()).parse(b"").is_err());
        assert!((token() - end()).parse(b"abc!@#").is_err());
    }
}
