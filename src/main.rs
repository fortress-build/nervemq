use nervemq::kms::sqlite::SqliteKeyManager;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    nervemq::run()
        .kms_factory(|db| SqliteKeyManager::new(db))
        .start()
        .await
}
