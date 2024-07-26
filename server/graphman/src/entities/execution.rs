//! Even though not every command is meant to store its execution data and make it available by ID,
//! this shouldn't be a limitation of the API. Every command and its output should be parsed
//! and made available to the users.

use async_graphql::Enum;
use async_graphql::Result;
use async_graphql::SimpleObject;
use async_graphql::Union;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::entities::deployment_info_command::DeploymentInfo;
use crate::entities::CommandKind;

#[derive(Clone, Debug, SimpleObject)]
/// Contains all the available information about a command execution.
pub struct Execution {
    pub id: Uuid,
    pub kind: CommandKind,
    pub status: ExecutionStatus,
    pub error_message: Option<String>,
    pub command_output: Option<CommandOutput>,
    pub started_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum)]
#[graphql(remote = "graphman_extensions::store::ExecutionStatus")]
/// Represents all the possible states of a command execution.
pub enum ExecutionStatus {
    InProgress,
    Failed,
    Succeeded,
}

#[derive(Clone, Debug, Union)]
/// Contains all the possible outputs of command executions.
pub enum CommandOutput {
    DeploymentInfo(DeploymentInfo),
    PauseDeployment(PauseDeployment),
    ResumeDeployment(ResumeDeployment),
    RestartDeployment(RestartDeployment),
}

#[derive(Clone, Debug, SimpleObject)]
/// Contains the output of a deployment pause execution.
pub struct PauseDeployment {
    success: bool,
}

#[derive(Clone, Debug, SimpleObject)]
/// Contains the output of a deployment resume execution.
pub struct ResumeDeployment {
    success: bool,
}

#[derive(Clone, Debug, SimpleObject)]
/// Contains the output of a deployment restart execution.
pub struct RestartDeployment {
    success: bool,
}

impl TryFrom<graphman_extensions::store::Execution> for Execution {
    type Error = async_graphql::Error;

    fn try_from(execution: graphman_extensions::store::Execution) -> Result<Self> {
        let graphman_extensions::store::Execution {
            id,
            kind,
            status,
            error_message,
            command_output,
            started_at,
            updated_at,
            completed_at,
        } = execution;

        let kind = kind.parse::<graphman::CommandKind>()?.into();

        let command_output = command_output
            .map(|value| parse_command_output(kind, value))
            .transpose()?;

        Ok(Self {
            id,
            kind,
            status: status.into(),
            error_message,
            command_output,
            started_at,
            updated_at,
            completed_at,
        })
    }
}

fn parse_command_output(kind: CommandKind, value: serde_json::Value) -> Result<CommandOutput> {
    use graphman::commands::deployment::info::DeploymentInfo;

    let parsed = match kind {
        CommandKind::DeploymentInfo => {
            CommandOutput::DeploymentInfo(serde_json::from_value::<DeploymentInfo>(value)?.into())
        }
        CommandKind::PauseDeployment => CommandOutput::PauseDeployment(PauseDeployment {
            success: serde_json::from_value(value)?,
        }),
        CommandKind::ResumeDeployment => CommandOutput::ResumeDeployment(ResumeDeployment {
            success: serde_json::from_value(value)?,
        }),
        CommandKind::RestartDeployment => {
            CommandOutput::RestartDeployment(RestartDeployment { success: true })
        }
    };

    Ok(parsed)
}
