//! Configuration management for NerveMQ.
//!
//! Handles loading and accessing configuration values from environment
//! variables with fallback to default values.

use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use url::Url;

/// Default configuration values used when not specified in environment.
pub mod defaults {
    pub const DB_PATH: &str = "nervemq.db";
    pub const MAX_RETRIES: usize = 10;

    pub const HOST: &str = "http://localhost:8080";

    pub const ROOT_EMAIL: &str = "admin@example.com";
    pub const ROOT_PASSWORD: &str = "password";
}

#[derive(Clone, Deserialize)]
/// Application configuration loaded from environment variables.
///
/// All fields are optional and fall back to values in `defaults` module.
/// Environment variables are prefixed with `NERVEMQ_` when loading.
///
/// # Fields
/// * `db_path` - Path to the SQLite database file
/// * `default_max_retries` - Maximum number of retry attempts for failed messages
/// * `host` - Base URL for the server
/// * `root_email` - Email address for the root admin user
/// * `root_password` - Password for the root admin user (stored securely)
///
/// # Environment Variables
/// * `NERVEMQ_DB_PATH`             - Database file path
/// * `NERVEMQ_DEFAULT_MAX_RETRIES` - Default retry limit
/// * `NERVEMQ_HOST`                - Server host URL (for UI access)
/// * `NERVEMQ_ROOT_EMAIL`          - Root admin email
/// * `NERVEMQ_ROOT_PASSWORD`       - Root admin password
pub struct Config {
    db_path: Option<String>,
    default_max_retries: Option<usize>,

    host: Option<Url>,

    root_email: Option<String>,
    root_password: Option<SecretString>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            db_path: None,
            default_max_retries: None,
            host: None,
            root_email: None,
            root_password: None,
        }
    }
}

impl Config {
    /// Loads configuration from environment variables.
    ///
    /// Reads variables prefixed with `NERVEMQ_` and constructs a Config instance.
    /// Falls back to default values for any unspecified settings.
    ///
    /// # Returns
    /// * `Ok(Config)` - Successfully loaded configuration
    /// * `Err` - If environment variables cannot be parsed
    ///
    /// # Warning
    /// Logs a warning if no root email is provided, as using the default is unsafe
    /// in production environments.
    pub fn load() -> eyre::Result<Self> {
        let config = envy::prefixed("NERVEMQ_").from_env::<Self>()?;

        if config.root_email.is_none() {
            tracing::warn!("No root email provided, using default - don't do this in production!");
        }

        Ok(config)
    }

    /// Gets the configured server host URL.
    ///
    /// # Returns
    /// The configured host URL or the default if not specified
    pub fn host(&self) -> Url {
        self.host
            .clone()
            .unwrap_or(defaults::HOST.try_into().expect("valid default url"))
    }

    /// Gets the database file path.
    ///
    /// # Returns
    /// The configured database path or the default if not specified
    pub fn db_path(&self) -> &str {
        self.db_path
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or(defaults::DB_PATH)
    }

    /// Gets the maximum number of retry attempts for failed messages.
    ///
    /// # Returns
    /// The configured retry limit or the default if not specified
    pub fn default_max_retries(&self) -> usize {
        self.default_max_retries.unwrap_or(defaults::MAX_RETRIES)
    }

    /// Gets the root administrator email address.
    ///
    /// # Returns
    /// The configured root email or the default if not specified
    pub fn root_email(&self) -> &str {
        self.root_email
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or(defaults::ROOT_EMAIL)
    }

    /// Gets the root administrator password.
    ///
    /// # Returns
    /// The configured root password or the default if not specified
    ///
    /// # Security
    /// The password is stored as a SecretString but must be exposed
    /// for authentication. Care should be taken when using this value.
    pub fn root_password(&self) -> &str {
        self.root_password
            .as_ref()
            .map(|s| s.expose_secret())
            .unwrap_or(defaults::ROOT_PASSWORD)
    }
}
