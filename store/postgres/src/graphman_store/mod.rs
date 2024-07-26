mod models;
mod schema;

use std::time::Duration;

use anyhow::Result;
use chrono::Utc;
use diesel::dsl::sql;
use diesel::prelude::*;
use serde_json::Value;
use uuid::Uuid;

use crate::connection_pool::ConnectionPool;

use self::models::Execution;
use self::models::ExecutionStatus;

#[derive(Clone)]
/// Used to save and retrieve details about graphman commands and their executions.
pub struct GraphmanStore {
    pool: ConnectionPool,
}

impl GraphmanStore {
    /// Returns a new graphman store.
    pub fn new(pool: ConnectionPool) -> Self {
        Self { pool }
    }
}

impl graphman_extensions::GraphmanExtensionStore for GraphmanStore {
    fn new_execution(&self, id: Uuid, kind: String) -> Result<()> {
        use schema::graphman_command_executions as gce;

        let model = Execution {
            id,
            kind: kind.to_owned(),
            status: ExecutionStatus::InProgress,
            error_message: None,
            command_output: None,
            started_at: Utc::now(),
            updated_at: None,
            completed_at: None,
        };

        let mut conn = self.pool.get()?;

        let _resp = diesel::insert_into(gce::table)
            .values(model)
            .execute(&mut conn)?;

        Ok(())
    }

    fn get_execution(&self, id: Uuid) -> Result<Option<graphman_extensions::store::Execution>> {
        use schema::graphman_command_executions as gce;

        let mut conn = self.pool.get()?;

        let resp = gce::table
            .find(id)
            .first::<Execution>(&mut conn)
            .optional()?;

        Ok(resp.map(Into::into))
    }

    fn any_executions_in_progress(&self, kind: String) -> Result<bool> {
        use schema::graphman_command_executions as gce;

        let mut conn = self.pool.get()?;

        let query = gce::table
            .select(sql::<diesel::sql_types::Integer>("1"))
            .filter(gce::kind.eq(kind))
            .filter(gce::status.eq(ExecutionStatus::InProgress));

        let resp = diesel::select(diesel::dsl::exists(query)).get_result::<bool>(&mut conn)?;

        Ok(resp)
    }

    fn execution_in_progress(&self, id: Uuid) -> Result<()> {
        use schema::graphman_command_executions as gce;

        let mut conn = self.pool.get()?;

        let _resp = diesel::update(gce::table)
            .set((
                gce::status.eq(ExecutionStatus::InProgress),
                gce::updated_at.eq(Utc::now()),
            ))
            .filter(gce::id.eq(id))
            .filter(gce::completed_at.is_null())
            .execute(&mut conn)?;

        Ok(())
    }

    fn execution_failed(&self, id: Uuid, error_message: String) -> Result<()> {
        use schema::graphman_command_executions as gce;

        let mut conn = self.pool.get()?;

        let _resp = diesel::update(gce::table)
            .set((
                gce::status.eq(ExecutionStatus::Failed),
                gce::error_message.eq(error_message),
                gce::completed_at.eq(Utc::now()),
            ))
            .filter(gce::id.eq(id))
            .execute(&mut conn)?;

        Ok(())
    }

    fn execution_succeeded(&self, id: Uuid, command_output: Option<Value>) -> Result<()> {
        use schema::graphman_command_executions as gce;

        let mut conn = self.pool.get()?;

        let _resp = diesel::update(gce::table)
            .set((
                gce::status.eq(ExecutionStatus::Succeeded),
                gce::command_output.eq(command_output),
                gce::completed_at.eq(Utc::now()),
            ))
            .filter(gce::id.eq(id))
            .execute(&mut conn)?;

        Ok(())
    }

    fn handle_broken_executions(&self, kind: String, max_inactive_time: Duration) -> Result<()> {
        use schema::graphman_command_executions as gce;

        let min_inactive_timestamp = Utc::now() - max_inactive_time;

        let mut conn = self.pool.get()?;

        let _resp = diesel::update(gce::table)
            .set((
                gce::status.eq(ExecutionStatus::Failed),
                gce::error_message.eq("Timeout"),
                gce::completed_at.eq(Utc::now()),
            ))
            .filter(gce::kind.eq(kind))
            .filter(gce::status.eq(ExecutionStatus::InProgress))
            .filter(
                gce::updated_at
                    .is_null()
                    .and(gce::started_at.lt(min_inactive_timestamp))
                    .or(gce::updated_at
                        .is_not_null()
                        .and(gce::updated_at.lt(min_inactive_timestamp))),
            )
            .execute(&mut conn)?;

        Ok(())
    }
}
