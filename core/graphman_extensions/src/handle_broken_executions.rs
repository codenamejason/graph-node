use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use anyhow::anyhow;
use graphman_primitives::BoxedError;
use graphman_primitives::BoxedFuture;
use graphman_primitives::ExtensibleGraphmanContext;
use graphman_primitives::GraphmanCommand;
use graphman_primitives::GraphmanLayer;

use crate::context_extensions::CommandKind;
use crate::error::error_builder;
use crate::GraphmanExtensionError;
use crate::GraphmanExtensionStore;

error_builder!("HandleBrokenExecutions");

// Command executions will be considered broken if they have
// not received any updates for this duration.
const DEFAULT_MAX_INACTIVE_TIME: Duration = Duration::from_secs(300);

/// Marks command executions as failed if they did not receive any
/// updates for the specified duration.
pub struct HandleBrokenExecutions<S> {
    max_inactive_time: Duration,
    store: Arc<S>,
}

pub struct HandleBrokenExecutionsExtension<S, C> {
    max_inactive_time: Duration,
    store: Arc<S>,
    inner: C,
}

impl<S> HandleBrokenExecutions<S> {
    /// Creates a new handler for broken executions with default parameters.
    pub fn new(store: Arc<S>) -> Self {
        Self {
            max_inactive_time: DEFAULT_MAX_INACTIVE_TIME,
            store,
        }
    }

    /// Adds a custom maximum inactive time to the handler.
    pub fn with_max_inactive_time(self, max_inactive_time: Duration) -> Self {
        let Self {
            max_inactive_time: _,
            store,
        } = self;

        Self {
            max_inactive_time,
            store,
        }
    }
}

impl<S, C> GraphmanLayer<C> for HandleBrokenExecutions<S> {
    type Outer = HandleBrokenExecutionsExtension<S, C>;

    fn layer(self, inner: C) -> Self::Outer {
        let Self {
            max_inactive_time,
            store,
        } = self;

        HandleBrokenExecutionsExtension {
            max_inactive_time,
            store,
            inner,
        }
    }
}

impl<S, C, Ctx> GraphmanCommand<Ctx> for HandleBrokenExecutionsExtension<S, C>
where
    S: GraphmanExtensionStore + Send + Sync + 'static,
    C: GraphmanCommand<Ctx> + Send + 'static,
    C::Error: Into<BoxedError>,
    C::Future: Future<Output = Result<C::Output, C::Error>> + Send + 'static,
    Ctx: ExtensibleGraphmanContext + Send + 'static,
{
    type Output = C::Output;
    type Error = GraphmanExtensionError;
    type Future = BoxedFuture<Self::Output, Self::Error>;

    fn execute(self, ctx: Ctx) -> Self::Future {
        Box::pin(async move {
            let Self {
                max_inactive_time,
                store,
                inner,
            } = self;

            let kind = ctx
                .get::<CommandKind>()
                .map_err(|err| e!(Context, err.into()))?
                .0;

            store
                .handle_broken_executions(kind.to_owned(), max_inactive_time)
                .map_err(|err| e!(Datastore, err))?;

            inner
                .execute(ctx)
                .await
                .map_err(|err| e!(CommandFailed, anyhow!(err.into())))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::command::TestCommand;
    use crate::test_utils::store::TestStore;
    use crate::GraphmanExtensionContext;

    fn ctx() -> GraphmanExtensionContext<()> {
        let mut ctx = GraphmanExtensionContext::new(());

        ctx.extend(CommandKind("TestCommand"));
        ctx
    }

    #[tokio::test]
    async fn forward_handling_to_the_store() {
        let store = Arc::new(TestStore::default());

        store
            .expect_handle_broken_executions
            .lock()
            .unwrap()
            .push(Ok(()));

        let cmd = TestCommand::new(|| "Ok");
        let ext = HandleBrokenExecutions::new(store.clone());
        let output = cmd.layer(ext).execute(ctx()).await.unwrap();

        assert_eq!(output, "Ok");

        store.assert_no_expected_calls_left();
    }
}
