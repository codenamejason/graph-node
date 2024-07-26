use chrono::{DateTime, Utc};
use diesel::deserialize::FromSql;
use diesel::pg::Pg;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::Varchar;
use serde_json::Value;
use strum::{EnumString, IntoStaticStr};
use uuid::Uuid;

use crate::graphman_store::schema;

#[derive(Clone, Debug, Insertable, Queryable, Selectable)]
#[diesel(table_name = schema::graphman_command_executions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Execution {
    pub id: Uuid,
    pub kind: String,
    pub status: ExecutionStatus,
    pub error_message: Option<String>,
    pub command_output: Option<Value>,
    pub started_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Copy, Debug, AsExpression, FromSqlRow, EnumString, IntoStaticStr)]
#[diesel(sql_type = Varchar)]
#[strum(serialize_all = "snake_case")]
pub enum ExecutionStatus {
    InProgress,
    Failed,
    Succeeded,
}

impl From<Execution> for graphman_extensions::store::Execution {
    fn from(execution: Execution) -> Self {
        let Execution {
            id,
            kind,
            status,
            error_message,
            command_output,
            started_at,
            updated_at,
            completed_at,
        } = execution;

        Self {
            id,
            kind,
            status: status.into(),
            error_message,
            command_output,
            started_at,
            updated_at,
            completed_at,
        }
    }
}

impl From<ExecutionStatus> for graphman_extensions::store::ExecutionStatus {
    fn from(status: ExecutionStatus) -> Self {
        use ExecutionStatus::*;

        match status {
            InProgress => Self::InProgress,
            Failed => Self::Failed,
            Succeeded => Self::Succeeded,
        }
    }
}

impl FromSql<Varchar, Pg> for ExecutionStatus {
    fn from_sql(bytes: diesel::pg::PgValue) -> diesel::deserialize::Result<Self> {
        Ok(std::str::from_utf8(bytes.as_bytes())?.parse()?)
    }
}

impl ToSql<Varchar, Pg> for ExecutionStatus {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
        <str as ToSql<Varchar, Pg>>::to_sql(self.into(), &mut out.reborrow())
    }
}
