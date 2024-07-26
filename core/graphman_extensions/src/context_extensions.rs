//! This module contains all the custom types that can be added to the context.
//! Types in this module can be used by any extensions.

use uuid::Uuid;

pub struct CommandExecutionId(pub Uuid);

pub struct CommandKind(pub &'static str);
