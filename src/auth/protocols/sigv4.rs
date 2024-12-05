use std::{pin::Pin, time::SystemTime};

use actix_web::{
    dev::ServiceRequest,
    web::{self},
    HttpMessage,
};
use aws_sigv4::sign::v4::generate_signing_key;
use bytes::BytesMut;
use futures_util::TryStreamExt;

use crate::{api::auth::User, auth::credential::AuthorizedNamespace, error::Error};

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
    service: web::Data<crate::service::Service>,
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

    let kms_key_id = service.get_key_id(user_email).await?;

    let signing_key = generate_signing_key(
        std::str::from_utf8(&service.kms().decrypt(&kms_key_id, encrypted_key).await?)
            .expect("kms key is not utf8"),
        SystemTime::now(),
        "us-west-1",
        "sqs",
    );

    match scratchstack_aws_signature::sigv4_validate_request(
        &mut req,
        payload,
        "us-west-1",
        "sqs",
        signing_key.as_ref(),
        chrono::Utc::now(),
        &scratchstack_aws_signature::NO_ADDITIONAL_SIGNED_HEADERS,
        scratchstack_aws_signature::SignatureOptions {
            s3: false,
            url_encode_form: false,
        },
    )
    .await
    {
        Ok(_) => {
            tracing::debug!("Request validated");
        }
        Err(err) => {
            tracing::error!("Error validating request: {}", err);
            return Err(Error::internal(eyre::eyre!("{err}")));
        }
    }

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
