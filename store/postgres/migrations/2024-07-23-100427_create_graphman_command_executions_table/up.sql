create table public.graphman_command_executions
(
    id             uuid primary key,
    kind           varchar                  not null,
    status         varchar                  not null,
    error_message  varchar                  default null,
    command_output jsonb                    default null,
    started_at     timestamp with time zone not null,
    updated_at     timestamp with time zone default null,
    completed_at   timestamp with time zone default null
);

create index graphman_command_executions_kind_idx
    on public.graphman_command_executions using hash (kind);

create index graphman_command_executions_status_idx
    on public.graphman_command_executions using hash (status);

create index graphman_command_executions_updated_at_idx
    on public.graphman_command_executions using btree (updated_at);
