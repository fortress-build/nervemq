use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sqlx::{
    sqlite::{
        SqliteAutoVacuum, SqliteConnectOptions, SqliteJournalMode, SqliteLockingMode,
        SqlitePoolOptions,
    },
    SqlitePool,
};

use creek::config::Config;
use creek::service::Service;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let config = Config::load()?;

    let service = Service::connect_with(config).await?;

    Ok(())
}
