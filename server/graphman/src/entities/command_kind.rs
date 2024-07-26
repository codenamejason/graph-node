use async_graphql::Enum;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
#[graphql(remote = "graphman::CommandKind")]
/// Lists all the supported graphman commands.
pub enum CommandKind {
    DeploymentInfo,
    PauseDeployment,
    ResumeDeployment,
    RestartDeployment,
}
