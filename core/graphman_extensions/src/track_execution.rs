use std::sync::Arc;
use std::time::Duration;

use anyhow::anyhow;
use graphman_primitives::BoxedError;
use graphman_primitives::BoxedFuture;
use graphman_primitives::ExtensibleGraphmanContext;
use graphman_primitives::GraphmanCommand;
use graphman_primitives::GraphmanLayer;
use serde::Serialize;
use uuid::Uuid;

use crate::context_extensions::CommandExecutionId;
use crate::context_extensions::CommandKind;
use crate::error::error_builder;
use crate::GraphmanExtensionError;
use crate::GraphmanExtensionStore;

error_builder!("TrackExecution");

// By default, there is no timeout because every command might have different requirements.
// The API provides a way to specify custom timeouts for each execution.
const DEFAULT_MAX_EXECUTION_TIME: Duration = Duration::MAX;

// At this interval, execution will report that it is still in progress.
const DEFAULT_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(10);

/// Makes a command report its execution status, and stores the execution output
/// or the error message in the persistent storage.
pub struct TrackExecution<S> {
    max_execution_time: Duration,
    heartbeat_interval: Duration,
    store: Arc<S>,
}

pub struct TrackExecutionExtension<S, C> {
    max_execution_time: Duration,
    heartbeat_interval: Duration,
    store: Arc<S>,
    inner: C,
}

impl<S> TrackExecution<S> {
    /// Creates a new execution tracker with default parameters.
    pub fn new(store: Arc<S>) -> Self {
        Self {
            max_execution_time: DEFAULT_MAX_EXECUTION_TIME,
            heartbeat_interval: DEFAULT_HEARTBEAT_INTERVAL,
            store,
        }
    }

    /// Adds a custom maximum execution time to the tracker.
    ///
    /// If a command does not produce a result within this time, its execution will be canceled.
    pub fn with_max_execution_time(self, max_execution_time: Duration) -> Self {
        let Self {
            max_execution_time: _,
            heartbeat_interval,
            store,
        } = self;

        Self {
            max_execution_time,
            heartbeat_interval,
            store,
        }
    }

    /// Adds a custom heartbeat interval to the tracker.
    ///
    /// The interval should be short enough to make sure that other extensions or
    /// modules don't consider a command execution broken.
    pub fn with_heartbeat_interval(self, heartbeat_interval: Duration) -> Self {
        let Self {
            max_execution_time,
            heartbeat_interval: _,
            store,
        } = self;

        Self {
            max_execution_time,
            heartbeat_interval,
            store,
        }
    }
}

impl<S, C> GraphmanLayer<C> for TrackExecution<S> {
    type Outer = TrackExecutionExtension<S, C>;

    fn layer(self, inner: C) -> Self::Outer {
        let Self {
            max_execution_time,
            heartbeat_interval,
            store,
        } = self;

        TrackExecutionExtension {
            max_execution_time,
            heartbeat_interval,
            store,
            inner,
        }
    }
}

impl<S, C> TrackExecutionExtension<S, C>
where
    S: GraphmanExtensionStore,
{
    // Returns a value from the extensible context.
    fn ctx<T>(ctx: &impl ExtensibleGraphmanContext) -> Result<&T, GraphmanExtensionError>
    where
        T: Send + Sync + 'static,
    {
        ctx.get::<T>().map_err(|err| e!(Context, err.into()))
    }

    // Reports that the execution is still in progress.
    async fn heartbeat(store: &S, id: Uuid, interval: Duration) -> GraphmanExtensionError {
        loop {
            tokio::time::sleep(interval).await;

            if let Err(err) = store.execution_in_progress(id) {
                return e!(Datastore, err.context("heartbeat failed"));
            }
        }
    }

    // Saves either the execution output or the error message to the persistent storage.
    fn handle_execution_result<O, E>(
        store: &S,
        id: Uuid,
        result: Result<O, E>,
    ) -> Result<O, GraphmanExtensionError>
    where
        O: Serialize,
        E: Into<BoxedError>,
    {
        match result {
            Ok(output) => {
                let json = serde_json::to_value(&output).map_err(|err| {
                    e!(
                        ExtensionFailed,
                        anyhow!(err).context("failed to convert output")
                    )
                })?;

                store
                    .execution_succeeded(id, Some(json))
                    .map_err(|err| e!(Datastore, err))?;

                Ok(output)
            }
            Err(err) => {
                let err = err.into();

                store
                    .execution_failed(id, err.to_string())
                    .map_err(|err| e!(Datastore, err))?;

                Err(e!(CommandFailed, anyhow!(err)))
            }
        }
    }

    // Reports an execution failure caused by the timeout.
    fn handle_execution_timeout(store: &S, id: Uuid) -> GraphmanExtensionError {
        if let Err(err) = store.execution_failed(id, "Timeout".to_owned()) {
            return e!(Datastore, err);
        }

        e!(ExtensionFailed, anyhow!("Timeout"))
    }
}

impl<S, C, Ctx> GraphmanCommand<Ctx> for TrackExecutionExtension<S, C>
where
    S: GraphmanExtensionStore + Send + Sync + 'static,
    C: GraphmanCommand<Ctx> + Send + 'static,
    C::Output: Serialize,
    C::Error: Into<BoxedError>,
    Ctx: ExtensibleGraphmanContext + Send + 'static,
{
    type Output = C::Output;
    type Error = GraphmanExtensionError;
    type Future = BoxedFuture<Self::Output, Self::Error>;

    fn execute(self, ctx: Ctx) -> Self::Future {
        Box::pin(async move {
            let Self {
                max_execution_time,
                heartbeat_interval,
                store,
                inner,
            } = self;

            let id = Self::ctx::<CommandExecutionId>(&ctx)?.0;
            let kind = Self::ctx::<CommandKind>(&ctx)?.0;

            store
                .new_execution(id, kind.to_owned())
                .map_err(|err| e!(Datastore, err))?;

            tokio::select! {
                result = inner.execute(ctx) => {
                    Self::handle_execution_result(&store, id, result)
                },
                _ = tokio::time::sleep(max_execution_time) => {
                    Err(Self::handle_execution_timeout(&store, id))
                },
                error = Self::heartbeat(&store, id, heartbeat_interval) => {
                   Err(error)
                }
            }
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

        ctx.extend(CommandExecutionId(Uuid::nil()));
        ctx.extend(CommandKind("TestCommand"));
        ctx
    }

    fn ms(value: u64) -> Duration {
        Duration::from_millis(value)
    }

    #[tokio::test]
    async fn track_successful_execution() {
        let store = Arc::new(TestStore::default());

        store.expect_new_execution.lock().unwrap().push(Ok(()));

        store
            .expect_execution_succeeded
            .lock()
            .unwrap()
            .push(Ok(()));

        let cmd = TestCommand::new(|| "Ok");
        let ext = TrackExecution::new(store.clone());
        let output = cmd.layer(ext).execute(ctx()).await.unwrap();

        assert_eq!(output, "Ok");

        store.assert_no_expected_calls_left();
    }

    #[tokio::test]
    async fn track_failed_execution() {
        let store = Arc::new(TestStore::default());

        store.expect_new_execution.lock().unwrap().push(Ok(()));
        store.expect_execution_failed.lock().unwrap().push(Ok(()));

        let cmd = TestCommand::new(|| "Ok").with_failure();
        let ext = TrackExecution::new(store.clone());

        cmd.layer(ext).execute(ctx()).await.unwrap_err();

        store.assert_no_expected_calls_left();
    }

    #[tokio::test]
    async fn track_execution_with_heartbeats() {
        let store = Arc::new(TestStore::default());

        store.expect_new_execution.lock().unwrap().push(Ok(()));

        store
            .expect_execution_in_progress
            .lock()
            .unwrap()
            .extend(vec![Ok(()), Ok(()), Ok(())]);

        store
            .expect_execution_succeeded
            .lock()
            .unwrap()
            .push(Ok(()));

        let cmd = TestCommand::new(|| "Ok").with_delay(ms(1700));
        let ext = TrackExecution::new(store.clone()).with_heartbeat_interval(ms(500));
        let output = cmd.layer(ext).execute(ctx()).await.unwrap();

        assert_eq!(output, "Ok");

        store.assert_no_expected_calls_left();
    }

    #[tokio::test]
    async fn track_execution_timeout() {
        let store = Arc::new(TestStore::default());

        store.expect_new_execution.lock().unwrap().push(Ok(()));
        store.expect_execution_failed.lock().unwrap().push(Ok(()));

        let cmd = TestCommand::new(|| "Ok").with_delay(ms(200));
        let ext = TrackExecution::new(store.clone()).with_max_execution_time(ms(100));

        cmd.layer(ext).execute(ctx()).await.unwrap_err();

        store.assert_no_expected_calls_left();
    }

    #[tokio::test]
    async fn track_with_heartbeat_failure() {
        let store = Arc::new(TestStore::default());

        store.expect_new_execution.lock().unwrap().push(Ok(()));

        store
            .expect_execution_in_progress
            .lock()
            .unwrap()
            .push(Err(anyhow!("Error")));

        let cmd = TestCommand::new(|| "Ok").with_delay(ms(200));
        let ext = TrackExecution::new(store.clone()).with_heartbeat_interval(ms(100));

        cmd.layer(ext).execute(ctx()).await.unwrap_err();

        store.assert_no_expected_calls_left();
    }

    #[tokio::test]
    async fn track_with_execution_timeout_failure() {
        let store = Arc::new(TestStore::default());

        store.expect_new_execution.lock().unwrap().push(Ok(()));

        store
            .expect_execution_failed
            .lock()
            .unwrap()
            .push(Err(anyhow!("Error")));

        let cmd = TestCommand::new(|| "Ok").with_delay(ms(200));
        let ext = TrackExecution::new(store.clone()).with_max_execution_time(ms(100));

        cmd.layer(ext).execute(ctx()).await.unwrap_err();

        store.assert_no_expected_calls_left();
    }
}
