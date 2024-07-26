use std::sync::Arc;

use anyhow::anyhow;
use async_graphql::Context;
use async_graphql::Object;
use async_graphql::Result;
use graph_store_postgres::graphman_store::GraphmanStore;
use graphman_extensions::store::GraphmanExtensionStore;
use uuid::Uuid;

use crate::entities::Execution;

pub struct ExecutionQuery;

#[Object]
/// Queries related to command executions.
impl ExecutionQuery {
    /// Returns all the details about a command execution.
    pub async fn info(&self, ctx: &Context<'_>, id: Uuid) -> Result<Execution> {
        let store = ctx.data::<Arc<GraphmanStore>>()?.to_owned();

        let execution = store
            .get_execution(id)?
            .ok_or_else(|| anyhow!("execution '{id}' was not found"))?;

        Ok(execution.try_into()?)
    }
}
