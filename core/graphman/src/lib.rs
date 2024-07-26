//! This module contains graphman commands and the utilities needed to make commands possible.
//!
//! This module does not make any decisions about how the commands will be executed
//! (for example, via API or CLI), so the details about these interfaces
//! should not leak to this module.

mod context;
mod error;
mod kind;

pub mod commands;
pub mod deployment_search;

pub use self::context::GraphmanContext;
pub use self::error::GraphmanError;
pub use self::kind::CommandKind;
