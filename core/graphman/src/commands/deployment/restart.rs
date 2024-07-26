use std::time::Duration;

use graphman_primitives::BoxedFuture;
use graphman_primitives::ExtensibleGraphmanCommand;
use graphman_primitives::GraphmanCommand;

use crate::commands::deployment::pause::PauseDeploymentCommand;
use crate::commands::deployment::resume::ResumeDeploymentCommand;
use crate::deployment_search::DeploymentSelector;
use crate::GraphmanContext;
use crate::GraphmanError;

#[derive(Clone, Debug)]
pub struct RestartDeploymentCommand {
    pub deployment: DeploymentSelector,
    pub delay: Duration,
}

impl<Ctx> GraphmanCommand<Ctx> for RestartDeploymentCommand
where
    Ctx: AsRef<GraphmanContext> + Send + 'static,
{
    type Output = ();
    type Error = GraphmanError;
    type Future = BoxedFuture<Self::Output, Self::Error>;

    fn execute(self, ctx: Ctx) -> Self::Future {
        Box::pin(async move {
            PauseDeploymentCommand {
                deployment: self.deployment.clone(),
            }
            .execute(ctx.as_ref().to_owned())
            .await?;

            tokio::time::sleep(self.delay).await;

            ResumeDeploymentCommand {
                deployment: self.deployment,
            }
            .execute(ctx)
            .await?;

            Ok(())
        })
    }
}

impl ExtensibleGraphmanCommand for RestartDeploymentCommand {}
