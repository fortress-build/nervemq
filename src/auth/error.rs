use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Unauthorized"))]
    Unauthorized,
    #[snafu(display("Internal server error"))]
    InternalError,
    #[snafu(display("Identity {key_id} not found"))]
    IdentityNotFound { key_id: String },
    #[snafu(display("Payload too large"))]
    PayloadTooLarge,
    #[snafu(display("Missing header"))]
    MissingHeader { header: String },
    #[snafu(display("Missing header"))]
    InvalidHeader { header: String },
}

impl actix_web::ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Self::Unauthorized => actix_web::http::StatusCode::UNAUTHORIZED,
            Self::InternalError => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            Self::IdentityNotFound { .. } => actix_web::http::StatusCode::UNAUTHORIZED,
            Self::MissingHeader { .. } | Self::InvalidHeader { .. } => {
                actix_web::http::StatusCode::BAD_REQUEST
            }
            Self::PayloadTooLarge => actix_web::http::StatusCode::PAYLOAD_TOO_LARGE,
        }
    }
}
