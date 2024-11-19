use pom::utf8::{any, end, one_of, seq, sym, Parser};

pub enum AuthScheme {
    NerveMqApiV1,
    Bearer,
}

pub enum AuthHeader<'a> {
    NerveMqApiV1 { key_id: &'a str, secret: &'a str },
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

fn nervemq_api_v1<'a>() -> Parser<'a, AuthHeader<'a>> {
    let tag = seq("NerveMqApiV1");
    let space = sym(' ').repeat(1..);
    let rest = one_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890.")
        .repeat(1..)
        .collect();

    ((tag + space) * rest - end())
        .convert(|value| {
            let mut split = value.split('.');
            let Some(key_id) = split.next() else {
                return Err(eyre::eyre!("Expected api key id"));
            };
            let Some(secret) = split.next() else {
                return Err(eyre::eyre!("Expected api key"));
            };

            Ok(AuthHeader::NerveMqApiV1 { key_id, secret })
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
