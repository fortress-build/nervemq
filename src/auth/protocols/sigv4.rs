use actix_web::{dev::ServiceRequest, web};
use hmac::Mac;
use secrecy::ExposeSecret;
use serde::Deserialize;
use sha2::Sha256;
use sqlx::FromRow;

use crate::auth::{
    crypto::{gen_signature_key, sha256_hex},
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

#[derive(Deserialize, FromRow)]
struct VerificationData {
    #[allow(unused)]
    hashed_key: String,
    validation_key: Vec<u8>,
}

pub async fn authenticate_sigv4(
    req: &mut ServiceRequest,
    header: SigV4Header<'_>,
) -> Result<(), Error> {
    let payload_hash = {
        let payload: actix_web::web::Payload = req
            .extract::<actix_web::web::Payload>()
            .await
            .map_err(|_| Error::InternalError)?;

        let bytes = payload
            .to_bytes_limited(8192)
            .await
            .map_err(|_| Error::PayloadTooLarge)?
            .map_err(|_| Error::InternalError)?;

        sha256_hex(&bytes)
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
                .map_err(|_| Error::InvalidHeader {
                    header: header.clone(),
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

    let Some(verify) = sqlx::query_as::<_, VerificationData>(
        "SELECT hashed_key, validation_key FROM api_keys WHERE key_id = $1",
    )
    .bind(&header.key_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch identity: {}", e);
        Error::InternalError
    })?
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
            &verify.validation_key,
            &header.date,
            &header.region,
            &header.service,
        );

        let mut mac = hmac::Hmac::<Sha256>::new_from_slice(signing_key.expose_secret())
            .map_err(|_| Error::InternalError)?;

        mac.update(string_to_sign.as_bytes());

        hex::encode(mac.finalize().into_bytes())
    };

    if header.signature != generated_signature {
        return Err(Error::Unauthorized);
    }

    Ok(())
}
