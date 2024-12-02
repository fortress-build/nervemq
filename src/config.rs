use secrecy::SecretString;
use serde::Deserialize;
use url::Url;

pub mod defaults {
    pub const DB_PATH: &str = "nervemq.db";
    pub const MAX_RETRIES: usize = 3;
    pub const ROOT_EMAIL: &str = "admin@example.com";
    pub const ROOT_PASSWORD: &str = "password";
}

#[derive(Clone, Deserialize)]
pub struct Config {
    pub db_path: Option<String>,
    pub default_max_retries: Option<usize>,

    pub host: Url,

    // TODO: Warn on launch if defaults are used for admin credentials
    pub root_email: Option<String>,
    #[allow(unused)]
    pub root_password: Option<SecretString>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            db_path: None,
            default_max_retries: None,
            host: Url::parse("http://localhost:8080").expect("valid URL"),
            root_email: None,
            root_password: None,
        }
    }
}

impl Config {
    pub fn load() -> eyre::Result<Self> {
        Ok(envy::prefixed("NERVEMQ_").from_env::<Self>()?)
    }

    pub fn db_path(&self) -> &str {
        self.db_path
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or(defaults::DB_PATH)
    }

    pub fn default_max_retries(&self) -> usize {
        self.default_max_retries.unwrap_or(defaults::MAX_RETRIES)
    }

    pub fn root_email(&self) -> &str {
        self.root_email
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or(defaults::ROOT_EMAIL)
    }
}
