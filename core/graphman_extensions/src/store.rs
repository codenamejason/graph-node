use std::time::Duration;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

/// Describes a persistent storage that is used by extensions
/// to store and retrieve details about command executions.
pub trait GraphmanExtensionStore {
    /// Registers a new command execution.
    fn new_execution(&self, id: Uuid, kind: String) -> Result<()>;

    /// Returns all available information about a command execution.
    fn get_execution(&self, id: Uuid) -> Result<Option<Execution>>;

    /// Returns true if there are any executions in progress of the specified kind.
    fn any_executions_in_progress(&self, kind: String) -> Result<bool>;

    /// Marks a command execution as in progress.
    ///
    /// It can be used multiple times at an interval to confirm that
    /// the execution is still in progress.
    fn execution_in_progress(&self, id: Uuid) -> Result<()>;

    /// Marks a command execution as failed.
    fn execution_failed(&self, id: Uuid, error_message: String) -> Result<()>;

    /// Marks a command execution as succeeded.
    fn execution_succeeded(&self, id: Uuid, command_output: Option<Value>) -> Result<()>;

    /// Sometimes, executions might fail, or the process might be killed and leave execution
    /// status as in progress forever, possibly preventing other commands from starting
    /// their execution. This method is used to mark such executions as timed out.
    fn handle_broken_executions(&self, kind: String, max_inactive_time: Duration) -> Result<()>;
}

#[derive(Clone, Debug)]
pub struct Execution {
    pub id: Uuid,
    pub kind: String,
    pub status: ExecutionStatus,
    pub error_message: Option<String>,
    pub command_output: Option<Value>,
    pub started_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionStatus {
    InProgress,
    Failed,
    Succeeded,
}
