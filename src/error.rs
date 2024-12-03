use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Unauthorized"))]
    Unauthorized,

    #[snafu(display("Resource not found: {resource}"))]
    NotFound { resource: String },

    #[snafu(display("Internal server error"))]
    InternalServerError {
        #[snafu(source(false))]
        source: Option<eyre::Report>,
    },

    #[snafu(display("Error returned from database"))]
    Sqlx {
        #[snafu(source)]
        source: sqlx::Error,
    },

    #[snafu(display("Error running migrations"))]
    MigrationError {
        #[snafu(source)]
        source: sqlx::migrate::MigrateError,
    },

    #[snafu(display("Identity {key_id} not found"))]
    IdentityNotFound { key_id: String },

    #[snafu(display("Payload too large"))]
    PayloadTooLarge,

    #[snafu(display("Missing header"))]
    MissingHeader { header: String },

    #[snafu(display("Invalid header"))]
    InvalidHeader { header: String },

    #[snafu(whatever, display("{message}"))]
    Whatever {
        message: String,
        #[snafu(source(from(eyre::Report, Some)))]
        source: Option<eyre::Report>,
    },

    #[snafu(display("Invalid parameter: {message}"))]
    InvalidParameter { message: String },

    #[snafu(display("Missing parameter: {message}"))]
    MissingParameter { message: String },
}

impl From<sqlx::Error> for Error {
    fn from(source: sqlx::Error) -> Self {
        Self::Sqlx { source }
    }
}

impl From<eyre::Report> for Error {
    fn from(e: eyre::Report) -> Self {
        Self::InternalServerError { source: Some(e) }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::InternalServerError {
            source: Some(e.into()),
        }
    }
}

impl From<actix_web::Error> for Error {
    fn from(source: actix_web::Error) -> Self {
        Self::InternalServerError {
            source: Some(eyre::eyre!("{source}")),
        }
    }
}

impl From<actix_identity::error::GetIdentityError> for Error {
    fn from(_: actix_identity::error::GetIdentityError) -> Self {
        Self::Unauthorized
    }
}

impl From<sqlx::migrate::MigrateError> for Error {
    fn from(source: sqlx::migrate::MigrateError) -> Self {
        Self::MigrationError { source }
    }
}

impl Error {
    pub fn internal(e: impl Into<eyre::Report>) -> Self {
        Self::InternalServerError {
            source: Some(e.into()),
        }
    }

    pub fn opaque() -> Self {
        Self::InternalServerError { source: None }
    }

    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }

    pub fn invalid_parameter(message: impl Into<String>) -> Self {
        Self::InvalidParameter {
            message: message.into(),
        }
    }

    pub fn missing_parameter(message: impl Into<String>) -> Self {
        Self::MissingParameter {
            message: message.into(),
        }
    }

    pub fn queue_not_found(queue: impl Into<String>, namespace: impl Into<String>) -> Self {
        Self::NotFound {
            resource: format!("queue {} in namespace {}", queue.into(), namespace.into()),
        }
    }

    pub fn namespace_not_found(namespace: impl Into<String>) -> Self {
        Self::NotFound {
            resource: format!("namespace {}", namespace.into()),
        }
    }
}

impl actix_web::ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Self::Unauthorized => actix_web::http::StatusCode::UNAUTHORIZED,
            Self::IdentityNotFound { .. } => actix_web::http::StatusCode::UNAUTHORIZED,
            Self::NotFound { .. } => actix_web::http::StatusCode::NOT_FOUND,

            Self::MissingHeader { .. }
            | Self::MissingParameter { .. }
            | Self::InvalidHeader { .. }
            | Self::InvalidParameter { .. } => actix_web::http::StatusCode::BAD_REQUEST,
            Self::PayloadTooLarge => actix_web::http::StatusCode::PAYLOAD_TOO_LARGE,

            Self::MigrationError { .. }
            | Self::InternalServerError { .. }
            | Self::Sqlx { .. }
            | Self::Whatever { .. } => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
