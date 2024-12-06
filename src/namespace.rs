use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Represents a namespace that contains queues.
///
/// A namespace is a logical grouping of queues that helps organize and control access
/// to queue resources. Each namespace has a unique ID, name, and tracks who created it.
#[derive(Serialize, Deserialize, FromRow, Debug)]
pub struct Namespace {
    /// Unique identifier for the namespace
    pub id: u64,
    /// Human-readable name of the namespace
    pub name: String,
    /// Email/identifier of the user who created the namespace
    pub created_by: String,
}

/// Implements equality comparison for Namespace based only on ID.
/// Two namespaces are considered equal if they have the same ID,
/// regardless of other fields.
impl PartialEq for Namespace {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// Statistics and metadata about a namespace.
///
/// This struct extends the base Namespace information with additional
/// statistical data about the queues contained within it.
#[derive(Serialize, Deserialize, FromRow, PartialEq, Debug)]
pub struct NamespaceStatistics {
    #[serde(flatten)]
    #[sqlx(flatten)]
    /// The base namespace information
    pub namespace: Namespace,
    /// Total number of queues in this namespace
    pub queue_count: u64,
}
