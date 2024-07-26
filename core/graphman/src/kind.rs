use strum::{Display, EnumString, IntoStaticStr};

/// Lists all the supported graphman commands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Display, IntoStaticStr, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum CommandKind {
    DeploymentInfo,
    PauseDeployment,
    ResumeDeployment,
    RestartDeployment,
}
