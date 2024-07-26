//! This module adds additional features to the way graphman commands are executed.
//! Each new feature is called an extension.
//! Extensions can depend directly or indirectly on each other or on the output of commands.
//! Layering extensions on top of each other can add multiple capabilities to the commands.

mod context;
mod context_extensions;
mod error;

pub mod execute_in_background;
pub mod handle_broken_executions;
pub mod identify_command;
pub mod prevent_duplicate_executions;
pub mod store;
pub mod track_execution;

pub use self::context::GraphmanExtensionContext;
pub use self::error::GraphmanExtensionError;
pub use self::execute_in_background::ExecuteInBackground;
pub use self::handle_broken_executions::HandleBrokenExecutions;
pub use self::identify_command::IdentifyCommand;
pub use self::prevent_duplicate_executions::PreventDuplicateExecutions;
pub use self::store::GraphmanExtensionStore;
pub use self::track_execution::TrackExecution;

#[cfg(test)]
mod test_utils;
