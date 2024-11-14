use std::collections::HashMap;

use config::Config;
use serde::{Deserialize, Serialize};
use service::Service;
use sqlx::{
    sqlite::{
        SqliteAutoVacuum, SqliteConnectOptions, SqliteJournalMode, SqliteLockingMode,
        SqlitePoolOptions,
    },
    SqlitePool,
};

mod config;
mod db;
mod service;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let config = Config::load()?;

    let service = Service::connect_with(config).await?;

    Ok(())
}
