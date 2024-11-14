use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub db_path: Option<String>,
}

impl Config {
    pub fn load() -> eyre::Result<Self> {
        Ok(envy::prefixed("CREEK_").from_env::<Self>()?)
    }

    pub fn db_path(&self) -> &str {
        self.db_path
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("creek.db")
    }
}

impl Default for Config {
    fn default() -> Self {
        Self { db_path: None }
    }
}
