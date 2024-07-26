use graphman_primitives::ExtensibleGraphmanContext;
use graphman_primitives::GraphmanCommand;
use graphman_primitives::GraphmanLayer;
use uuid::Uuid;

use crate::context_extensions::CommandExecutionId;
use crate::context_extensions::CommandKind;

/// Makes a command identifiable by assigning it a kind
/// and a unique command execution ID.
pub struct IdentifyCommand {
    execution_id: CommandExecutionId,
    kind: CommandKind,
}

pub struct IdentifyCommandExtension<C> {
    execution_id: CommandExecutionId,
    kind: CommandKind,
    inner: C,
}

impl IdentifyCommand {
    /// Creates a new command identification.
    pub fn new(kind: &'static str) -> Self {
        Self {
            execution_id: CommandExecutionId(Uuid::new_v4()),
            kind: CommandKind(kind),
        }
    }
}

impl<C> GraphmanLayer<C> for IdentifyCommand {
    type Outer = IdentifyCommandExtension<C>;

    fn layer(self, inner: C) -> Self::Outer {
        let Self { execution_id, kind } = self;

        IdentifyCommandExtension {
            execution_id,
            kind,
            inner,
        }
    }
}

impl<C, Ctx> GraphmanCommand<Ctx> for IdentifyCommandExtension<C>
where
    C: GraphmanCommand<Ctx> + Send + 'static,
    Ctx: ExtensibleGraphmanContext + Send + 'static,
{
    type Output = C::Output;
    type Error = C::Error;
    type Future = C::Future;

    fn execute(self, mut ctx: Ctx) -> Self::Future {
        let Self {
            execution_id,
            kind,
            inner,
        } = self;

        ctx.extend(execution_id);
        ctx.extend(kind);

        inner.execute(ctx)
    }
}
