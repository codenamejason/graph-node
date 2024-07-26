use std::marker::PhantomData;
use std::time::Duration;

use anyhow::anyhow;
use graphman_primitives::BoxedError;
use graphman_primitives::BoxedFuture;
use graphman_primitives::ExtensibleGraphmanCommand;
use graphman_primitives::GraphmanCommand;

#[derive(Clone, Debug)]
pub struct TestCommand<F, O> {
    ok: bool,
    delay: Duration,
    operation: F,
    _marker: PhantomData<O>,
}

impl<F, O> TestCommand<F, O> {
    pub fn new(operation: F) -> Self {
        Self {
            ok: true,
            delay: Duration::from_millis(10),
            operation,
            _marker: PhantomData,
        }
    }

    pub fn with_failure(self) -> Self {
        let Self {
            ok: _,
            delay,
            operation,
            _marker,
        } = self;

        Self {
            ok: false,
            delay,
            operation,
            _marker,
        }
    }

    pub fn with_delay(self, delay: Duration) -> Self {
        let Self {
            ok,
            delay: _,
            operation,
            _marker,
        } = self;

        Self {
            ok,
            delay,
            operation,
            _marker,
        }
    }
}

impl<F, O, Ctx> GraphmanCommand<Ctx> for TestCommand<F, O>
where
    F: FnOnce() -> O + Send + 'static,
    O: Send + Sync + 'static,
{
    type Output = O;
    type Error = BoxedError;
    type Future = BoxedFuture<Self::Output, Self::Error>;

    fn execute(self, _ctx: Ctx) -> Self::Future {
        Box::pin(async move {
            let Self {
                ok,
                delay,
                operation,
                _marker,
            } = self;

            tokio::time::sleep(delay).await;

            if !ok {
                return Err(anyhow!("Error").into());
            }

            let output = operation();

            Ok(output)
        })
    }
}

impl<F, O> ExtensibleGraphmanCommand for TestCommand<F, O> {}
