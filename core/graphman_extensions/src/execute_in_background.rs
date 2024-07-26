use graphman_primitives::BoxedFuture;
use graphman_primitives::ExtensibleGraphmanContext;
use graphman_primitives::GraphmanCommand;
use graphman_primitives::GraphmanLayer;
use uuid::Uuid;

use crate::context_extensions::CommandExecutionId;
use crate::error::error_builder;
use crate::GraphmanExtensionError;

error_builder!("ExecuteInBackground");

/// Executes a command in the background and returns the execution ID as the output.
pub struct ExecuteInBackground;

pub struct ExecuteInBackgroundExtension<C> {
    inner: C,
}

impl ExecuteInBackground {
    /// Creates a new background executor.
    pub fn new() -> Self {
        Self {}
    }
}

impl<C> GraphmanLayer<C> for ExecuteInBackground {
    type Outer = ExecuteInBackgroundExtension<C>;

    fn layer(self, inner: C) -> Self::Outer {
        let Self {} = self;

        ExecuteInBackgroundExtension { inner }
    }
}

impl<C, Ctx> GraphmanCommand<Ctx> for ExecuteInBackgroundExtension<C>
where
    C: GraphmanCommand<Ctx> + Send + 'static,
    C::Output: Send + 'static,
    C::Error: Send + 'static,
    Ctx: ExtensibleGraphmanContext + Send + 'static,
{
    type Output = Uuid;
    type Error = GraphmanExtensionError;
    type Future = BoxedFuture<Self::Output, Self::Error>;

    fn execute(self, ctx: Ctx) -> Self::Future {
        Box::pin(async move {
            let Self { inner } = self;

            let id = ctx
                .get::<CommandExecutionId>()
                .map_err(|err| e!(Context, err.into()))?
                .0;

            // The current API does not provide a way to cancel executions on demand.
            let _handle = tokio::spawn(async move { inner.execute(ctx).await });

            Ok(id)
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicU8;
    use std::sync::atomic::Ordering;
    use std::sync::Arc;
    use std::time::Duration;

    use super::*;
    use crate::context_extensions::CommandExecutionId;
    use crate::test_utils::command::TestCommand;
    use crate::GraphmanExtensionContext;

    fn ctx() -> GraphmanExtensionContext<()> {
        let mut ctx = GraphmanExtensionContext::new(());

        ctx.extend(CommandExecutionId(Uuid::from_u128(1)));
        ctx
    }

    fn ms(value: u64) -> Duration {
        Duration::from_millis(value)
    }

    #[tokio::test]
    async fn command_is_executed_in_background() {
        use Ordering::SeqCst;

        let counter = Arc::new(AtomicU8::new(0));
        let counter_clone = counter.clone();

        let cmd = TestCommand::new(move || {
            counter_clone.store(100, SeqCst);
        })
        .with_delay(ms(500));

        let ext = ExecuteInBackground::new();
        let output = cmd.layer(ext).execute(ctx()).await.unwrap();

        assert_eq!(output, Uuid::from_u128(1));

        tokio::time::sleep(ms(100)).await;

        assert_eq!(counter.load(SeqCst), 0);

        tokio::time::sleep(ms(800)).await;

        assert_eq!(counter.load(SeqCst), 100);
    }
}
