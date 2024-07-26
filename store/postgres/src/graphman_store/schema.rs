diesel::table! {
    public.graphman_command_executions {
        id -> Uuid,
        kind -> Varchar,
        status -> Varchar,
        error_message -> Nullable<Varchar>,
        command_output -> Nullable<Jsonb>,
        started_at -> Timestamptz,
        updated_at -> Nullable<Timestamptz>,
        completed_at -> Nullable<Timestamptz>,
    }
}
