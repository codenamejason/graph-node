use std::sync::Arc;
use std::time::Duration;

use async_graphql::Context;
use async_graphql::Object;
use async_graphql::Result;
use graph_store_postgres::graphman_store::GraphmanStore;
use graphman::commands::deployment::pause::PauseDeploymentCommand;
use graphman::commands::deployment::restart::RestartDeploymentCommand;
use graphman::commands::deployment::resume::ResumeDeploymentCommand;
use graphman::CommandKind;
use graphman_extensions::ExecuteInBackground;
use graphman_extensions::GraphmanExtensionContext;
use graphman_extensions::HandleBrokenExecutions;
use graphman_extensions::IdentifyCommand;
use graphman_extensions::PreventDuplicateExecutions;
use graphman_extensions::TrackExecution;
use graphman_primitives::GraphmanCommand;
use graphman_primitives::GraphmanLayer;
use uuid::Uuid;

use crate::entities::DeploymentSelector;
use crate::resolvers::context::make_graphman_context;

pub struct DeploymentMutation;

#[Object]
/// Mutations related to one or multiple deployments.
impl DeploymentMutation {
    /// Pauses a deployment that is not already paused.
    pub async fn pause(&self, ctx: &Context<'_>, deployment: DeploymentSelector) -> Result<bool> {
        let command = PauseDeploymentCommand {
            deployment: deployment.into(),
        };

        let ctx = make_graphman_context(ctx)?;

        Ok(command.execute(ctx).await?)
    }

    /// Resumes a deployment that has been previously paused.
    pub async fn resume(&self, ctx: &Context<'_>, deployment: DeploymentSelector) -> Result<bool> {
        let command = ResumeDeploymentCommand {
            deployment: deployment.into(),
        };

        let ctx = make_graphman_context(ctx)?;

        Ok(command.execute(ctx).await?)
    }

    /// Pauses a deployment and resumes it after a delay.
    pub async fn restart(
        &self,
        ctx: &Context<'_>,
        deployment: DeploymentSelector,
        #[graphql(
            default = 20,
            desc = "The number of seconds to wait before resuming the deployment.
                    When not specified, it defaults to 20 seconds."
        )]
        delay_seconds: u64,
    ) -> Result<Uuid> {
        let command = RestartDeploymentCommand {
            deployment: deployment.into(),
            delay: Duration::from_secs(delay_seconds),
        };

        let store = ctx.data::<Arc<GraphmanStore>>()?.to_owned();

        let ctx = GraphmanExtensionContext::new(make_graphman_context(ctx)?);

        let output = command
            .layer(IdentifyCommand::new(CommandKind::RestartDeployment.into()))
            .layer(HandleBrokenExecutions::new(store.clone()))
            .layer(PreventDuplicateExecutions::new(store.clone()))
            .layer(ExecuteInBackground::new())
            .layer(TrackExecution::new(store))
            .execute(ctx)
            .await?;

        Ok(output)
    }
}
