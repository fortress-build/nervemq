//! Queue definitions and statistics tracking.
//!
//! This module defines the core queue types and their associated statistics.
//! Each queue belongs to a namespace and maintains counters for message states
//! and performance metrics.
//!
//! # Queue Identification
//! Queues are uniquely identified by:
//! - A numeric ID (for internal use)
//! - A namespace + name combination (for API use)
//!
//! # Statistics Tracking
//! The system tracks per-queue statistics including:
//! - Total message count
//! - Average message size
//! - Count of messages in each state (pending/delivered/failed)

use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

/// Represents a message queue in the system.
///
/// Each queue exists within a namespace and is created by a specific user.
/// Queues are the primary containers for messages and maintain their own
/// configuration and statistics.
#[derive(Serialize, Deserialize, FromRow, Debug)]
pub struct Queue {
    /// Unique numeric identifier for the queue
    pub id: u64,
    /// Namespace the queue belongs to
    pub ns: String,
    /// Human-readable queue name
    pub name: String,
    /// ID of the user who created the queue
    pub created_by: String,
}

impl PartialEq for Queue {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// Statistics and metrics for a queue.
///
/// Tracks various operational metrics including message counts by status
/// and size statistics. These metrics are used for monitoring queue health
/// and performance.
#[derive(Serialize, Deserialize, FromRow, Debug)]
pub struct QueueStatistics {
    #[serde(flatten)]
    #[sqlx(flatten)]
    /// The base queue information this statistics belongs to
    pub queue: Queue,
    /// Total number of messages ever sent to the queue
    pub message_count: u64,
    /// Average size of messages in bytes
    pub avg_size_bytes: f64,
    /// Number of messages waiting to be processed
    pub pending: u64,
    /// Number of successfully processed messages
    pub delivered: u64,
    /// Number of messages that failed processing
    pub failed: u64,
}
