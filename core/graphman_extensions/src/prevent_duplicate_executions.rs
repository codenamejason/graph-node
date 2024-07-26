use std::future::Future;
use std::sync::Arc;

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

error_builder!("PreventDuplicateExecutions");

/// Fails the command execution if there are other executions in progress of the same kind.
pub struct PreventDuplicateExecutions<S> {
    store: Arc<S>,
}

pub struct PreventDuplicateExecutionsExtension<S, C> {
    store: Arc<S>,
    inner: C,
}

impl<S> PreventDuplicateExecutions<S> {
    /// Creates a new duplicate command execution detector.
    pub fn new(store: Arc<S>) -> Self {
        Self { store }
    }
}

impl<S, C> GraphmanLayer<C> for PreventDuplicateExecutions<S> {
    type Outer = PreventDuplicateExecutionsExtension<S, C>;

    fn layer(self, inner: C) -> Self::Outer {
        let Self { store } = self;

        PreventDuplicateExecutionsExtension { store, inner }
    }
}

impl<S, C, Ctx> GraphmanCommand<Ctx> for PreventDuplicateExecutionsExtension<S, C>
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
            let Self { store, inner } = self;

            let kind = ctx
                .get::<CommandKind>()
                .map_err(|err| e!(Context, err.into()))?
                .0;

            let other_executions_in_progress = store
                .any_executions_in_progress(kind.to_owned())
                .map_err(|err| e!(Datastore, err))?;

            if other_executions_in_progress {
                return Err(e!(
                    ExtensionFailed,
                    anyhow!("other executions of kind '{kind}' are in progress")
                ));
            }

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
    async fn allow_execution() {
        let store = Arc::new(TestStore::default());

        store
            .expect_any_executions_in_progress
            .lock()
            .unwrap()
            .push(Ok(false));

        let cmd = TestCommand::new(|| "Ok");
        let ext = PreventDuplicateExecutions::new(store.clone());
        let output = cmd.layer(ext).execute(ctx()).await.unwrap();

        assert_eq!(output, "Ok");

        store.assert_no_expected_calls_left();
    }

    #[tokio::test]
    async fn prevent_execution() {
        let store = Arc::new(TestStore::default());

        store
            .expect_any_executions_in_progress
            .lock()
            .unwrap()
            .push(Ok(true));

        let cmd = TestCommand::new(|| "Ok");
        let ext = PreventDuplicateExecutions::new(store.clone());

        cmd.layer(ext).execute(ctx()).await.unwrap_err();

        store.assert_no_expected_calls_left();
    }
}
