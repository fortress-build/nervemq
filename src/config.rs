use secrecy::SecretString;
use serde::Deserialize;

pub mod defaults {
    pub const DB_PATH: &str = "nervemq.db";
    pub const MAX_RETRIES: usize = 3;
    pub const ROOT_EMAIL: &str = "admin@example.com";
    pub const ROOT_PASSWORD: &str = "password";
}

#[derive(Clone, Deserialize, Default)]
pub struct Config {
    pub db_path: Option<String>,
    pub default_max_retries: Option<usize>,

    // TODO: Warn on launch if defaults are used for admin credentials
    pub root_email: Option<String>,
    #[allow(unused)]
    pub root_password: Option<SecretString>,
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
