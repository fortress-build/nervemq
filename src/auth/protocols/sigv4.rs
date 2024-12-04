use actix_web::{
    dev::ServiceRequest,
    web::{self},
    HttpMessage,
};
use futures_util::StreamExt;
// use actix_web_lab::util::fork_request_payload;
use hmac::Mac;
use secrecy::ExposeSecret;
use sha2::Sha256;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::{
    api::auth::User,
    auth::{
        credential::AuthorizedNamespace,
        crypto::{gen_signature_key, sha256_hex},
    },
    error::Error,
};

#[derive(Debug)]
pub struct SigV4Header<'a> {
    pub algorithm: &'a str,
    pub key_id: &'a str,
    pub date: &'a str,
    pub signed_headers: Vec<&'a str>,
    pub signature: &'a str,
    pub region: &'a str,
    pub service: &'a str,
}

/// NOTE: This function was lifted from the `actix-web-lab` crate since I don't need anything else
/// from the crate.
///
/// Returns an effectively cloned payload that supports streaming efficiently.
///
/// The cloned payload:
/// - yields identical chunks;
/// - does not poll ahead of the original;
/// - does not poll significantly slower than the original;
/// - receives an error signal if the original errors, but details are opaque to the copy.
///
/// If the payload is forked in one of the extractors used in a handler, then the original _must_ be
/// read in another extractor or else the request will hang.
fn fork_request_payload(orig_payload: &mut actix_web::dev::Payload) -> actix_web::dev::Payload {
    const TARGET: &str = concat!(module_path!(), "::fork_request_payload");

    let payload = orig_payload.take();

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    let proxy_stream: actix_http::BoxedPayloadStream =
        Box::pin(payload.inspect(move |res| match res {
            Ok(chunk) => {
                tracing::trace!(target: TARGET, "yielding {} byte chunk", chunk.len());
                tx.send(Ok(chunk.clone())).ok();
            }
            Err(err) => {
                tx.send(Err(actix_web::error::PayloadError::Io(
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("error from original stream: {err}"),
                    ),
                )))
                .ok();
            }
        }));

    tracing::trace!(target: TARGET, "creating proxy payload");
    *orig_payload = actix_web::dev::Payload::from(proxy_stream);

    actix_web::dev::Payload::Stream {
        payload: Box::pin(UnboundedReceiverStream::new(rx)),
    }
}

pub async fn authenticate_sigv4(
    req: &mut ServiceRequest,
    header: SigV4Header<'_>,
) -> Result<(User, AuthorizedNamespace), Error> {
    let payload_hash = {
        let mut original_payload = req.take_payload();

        let mut payload = fork_request_payload(&mut original_payload);

        req.set_payload(original_payload);

        let mut bytes = Vec::new();
        while let Some(chunk) = payload
            .next()
            .await
            .transpose()
            // temporary
            .ok()
            .flatten()
        // .map_err(|e| Error::internal(e))?
        {
            if chunk.is_empty() {
                break;
            }
            bytes.extend_from_slice(&chunk);
        }

        sha256_hex(&[])
    };

    let canonical_headers = header
        .signed_headers
        .iter()
        .map(|header| {
            let header = header.to_lowercase();
            let value = req
                .headers()
                .get(&header)
                .ok_or_else(|| Error::MissingHeader {
                    header: header.clone(),
                })?
                .to_str()
                .map_err(|e| {
                    tracing::error!("Invalid header value: {}", e);

                    Error::InvalidHeader {
                        header: header.clone(),
                    }
                })?;
            Ok(format!("{}:{}", header, value))
        })
        .collect::<Result<Vec<String>, Error>>()?
        .join("\n");

    let signed_headers = header.signed_headers.join(";");

    let canonical_request = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        req.method(),
        req.uri(),
        "",
        canonical_headers,
        signed_headers,
        payload_hash,
    );
    let canonical_request_hash = sha256_hex(canonical_request.as_bytes());

    let credential_scope = format!(
        "{}/{}/{}/aws4_request",
        header.date, header.region, header.service
    );

    let string_to_sign = format!(
        "{}\n{}\n{}\n{}",
        header.algorithm, header.date, credential_scope, canonical_request_hash
    );

    let pool = req
        .app_data::<web::Data<crate::service::Service>>()
        .expect("SQLite pool not found. This is a bug.")
        .db();

    let Some((validation_key, namespace)) = sqlx::query_as::<_, (Vec<u8>, String)>(
        "
            SELECT k.validation_key, ns.name FROM api_keys k
            JOIN namespaces ns ON ns.id = k.ns
            WHERE key_id = $1
            ",
    )
    .bind(&header.key_id)
    .fetch_optional(pool)
    .await?
    else {
        return Err(Error::IdentityNotFound {
            key_id: header.key_id.to_string(),
        }
        .into());
    };

    // let Ok(hashed_key) = PasswordHashString::new(&verify.hashed_key) else {
    //     return Err(Error::InternalError.into());
    // };
    //
    // let hash = hashed_key.password_hash();
    // let Some(salt) = hash.salt else {
    //     tracing::error!("No salt found in hashed key - this is probably a bug");
    //     return Err(Error::InternalError);
    // };

    let generated_signature = {
        let signing_key = gen_signature_key(
            &validation_key,
            &header.date,
            &header.region,
            &header.service,
        );

        let mut mac = hmac::Hmac::<Sha256>::new_from_slice(signing_key.expose_secret())
            .map_err(|e| Error::internal(e))?;

        mac.update(string_to_sign.as_bytes());

        hex::encode(mac.finalize().into_bytes())
    };

    if header.signature != generated_signature {
        tracing::error!(
            "Invalid signature: {} != {}",
            header.signature,
            generated_signature
        );

        // return Err(Error::Unauthorized);
    }

    let user: User = sqlx::query_as(
        "
        SELECT u.* FROM api_keys k
        JOIN users u ON u.id = k.user
        WHERE k.key_id = $1
        ",
    )
    .bind(&header.key_id)
    .fetch_one(pool)
    .await?;

    Ok((user, AuthorizedNamespace(namespace)))
}
