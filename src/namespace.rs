use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow, Debug)]
pub struct Namespace {
    pub id: u64,
    pub name: String,
    pub created_by: String,
}

impl PartialEq for Namespace {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Serialize, Deserialize, FromRow, PartialEq, Debug)]
pub struct NamespaceStatistics {
    #[serde(flatten)]
    #[sqlx(flatten)]
    pub namespace: Namespace,
    pub queue_count: u64,
}
