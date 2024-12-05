use std::{pin::Pin, time::SystemTime};

use actix_web::{
    dev::ServiceRequest,
    web::{self},
    HttpMessage,
};
use argon2::password_hash::PasswordHashString;
use aws_sigv4::sign::v4::generate_signing_key;
use bytes::BytesMut;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use futures_util::TryStreamExt;
use hmac::Mac;
use sha2::Sha256;

use crate::{
    api::auth::User,
    auth::{credential::AuthorizedNamespace, crypto::sha256_hex, kms::KeyManager},
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

pub async fn authenticate_sigv4(
    mut req: &mut ServiceRequest,
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

        let payload = actix_web::dev::Payload::Stream {
            payload: Box::pin(futures_util::stream::once(std::future::ready(Ok(
                bytes.clone()
            ))))
                as Pin<
                    Box<dyn futures_util::Stream<Item = Result<_, actix_web::error::PayloadError>>>,
                >,
        };

        req.set_payload(payload);

        bytes
    };

    let pool = req
        .app_data::<web::Data<crate::service::Service>>()
        .expect("SQLite pool not found. This is a bug.")
        .db()
        .clone();

    let Some((validation_key, namespace)) = sqlx::query_as::<_, (Vec<u8>, String)>(
        "
            SELECT k.validation_key, ns.name FROM api_keys k
            JOIN namespaces ns ON ns.id = k.ns
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

    let validation_key = <PasswordHashString as std::str::FromStr>::from_str(
        std::str::from_utf8(&validation_key).unwrap(),
    )
    .unwrap()
    .password_hash()
    .hash
    .unwrap();

    // let date = req
    //     .headers()
    //     .get("X-Amz-Date")
    //     .ok_or_else(|| Error::MissingHeader {
    //         header: "X-Amz-Date".to_string(),
    //     })?;
    // let date = DateTime::from_utc

    let date_str = header.date;
    let year = date_str[0..4].parse::<i32>().unwrap();
    let month = date_str[4..6].parse::<u32>().unwrap();
    let day = date_str[6..8].parse::<u32>().unwrap();

    let date = NaiveDate::from_ymd_opt(year, month, day).ok_or_else(|| Error::InvalidHeader {
        header: "X-Amz-Date".to_string(),
    })?;
    let timestamp = NaiveDateTime::new(date, NaiveTime::from_hms(0, 0, 0));

    let timestamp = Utc.from_local_datetime(&timestamp).unwrap();

    // let timestamp = SystemTime::now();
    let signing_key = generate_signing_key(
        hex::encode(validation_key.as_ref()).as_str(),
        timestamp.into(),
        "us-west-1",
        "sqs",
    );

    match scratchstack_aws_signature::sigv4_validate_request(
        &mut req,
        payload,
        "us-west-1",
        "sqs",
        signing_key.as_ref(),
        timestamp.into(),
        &scratchstack_aws_signature::NO_ADDITIONAL_SIGNED_HEADERS,
        scratchstack_aws_signature::SignatureOptions {
            s3: false,
            url_encode_form: false,
        },
    )
    .await
    {
        Ok(_) => {}
        Err(err) => {
            tracing::error!("Error validating request: {}", err);
            return Err(Error::internal(eyre::eyre!("{err}")));
        }
    }

    // let canonical_headers = header
    //     .signed_headers
    //     .iter()
    //     .map(|header| {
    //         let value = req
    //             .headers()
    //             .get(*header)
    //             .ok_or_else(|| Error::MissingHeader {
    //                 header: header.to_string(),
    //             })?
    //             .to_str()
    //             .map_err(|e| {
    //                 tracing::error!("Invalid header value: {}", e);
    //
    //                 Error::InvalidHeader {
    //                     header: header.to_string(),
    //                 }
    //             })?;
    //         Ok(format!("{}:{}", header, value))
    //     })
    //     .collect::<Result<Vec<String>, Error>>()?
    //     .join("\n");
    //
    // let signed_headers = header.signed_headers.join(";");
    //
    // let canonical_request = format!(
    //     "{}\n{}\n{}\n{}\n{}\n{}",
    //     req.method(),
    //     req.uri(),
    //     "us-west-1", // FIXME: This should be the actual region
    //     canonical_headers,
    //     signed_headers,
    //     payload_hash,
    // );
    // let canonical_request_hash = sha256_hex(canonical_request.as_bytes());
    //
    // let credential_scope = format!(
    //     "{}/{}/{}/aws4_request",
    //     header.date, header.region, header.service
    // );
    //
    // let string_to_sign = format!(
    //     "{}\n{}\n{}\n{}",
    //     header.algorithm, header.date, credential_scope, canonical_request_hash
    // );

    // let Ok(hashed_key) = PasswordHashString::new(&verify.hashed_key) else {
    //     return Err(Error::InternalError.into());
    // };
    //
    // let hash = hashed_key.password_hash();
    // let Some(salt) = hash.salt else {
    //     tracing::error!("No salt found in hashed key - this is probably a bug");
    //     return Err(Error::InternalError);
    // };

    // let generated_signature = {
    //     let sk = scratchstack_aws_signature::SigningKey {
    //         kind: scratchstack_aws_signature::SigningKeyKind::KSecret,
    //         key: validation_key.clone(),
    //     };
    //
    //     let date = Utc::now().naive_local();
    //     let sk = sk.derive(
    //         scratchstack_aws_signature::SigningKeyKind::KSigning,
    //         &date.date(),
    //         "us-west-1",
    //         "sqs",
    //     );
    //
    //     let parts = req.into_parts().0.uri().into_parts();
    //     // let parts = http::uri::Parts::from(parts);
    //     scratchstack_aws_signature::Request::from_http_request_parts(&mut req, None);
    //
    //     let signing_key = aws_sigv4::sign::v4::generate_signing_key(
    //         &hex::encode(validation_key),
    //         SystemTime::now(),
    //         "us-west-1",
    //         "sqs",
    //     );
    //
    //     let mut mac = hmac::Hmac::<Sha256>::new_from_slice(signing_key.as_ref())
    //         .map_err(|e| Error::internal(e))?;
    //
    //     mac.update(string_to_sign.as_bytes());
    //
    //     hex::encode(mac.finalize().into_bytes())
    // };
    //
    // if header.signature != generated_signature {
    //     tracing::error!(
    //         "Invalid signature: {} != {}",
    //         header.signature,
    //         generated_signature
    //     );
    //
    //     // return Err(Error::Unauthorized);
    // } else {
    //     tracing::info!("Signature is valid");
    // }

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
