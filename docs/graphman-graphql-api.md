# Graphman GraphQL API

When starting the `graph-node` optionally, a graphman GraphQL server can be started, which makes it possible to execute
graphman commands via GraphQL.

To start the graphman GraphQL server, the `GRAPHMAN_SERVER_AUTH_TOKEN` environment variable should be set. It is
required that the authentication token be exactly 64 characters long. This is equivalent to an SHA-256 hash represented
as a hex string. The token is used to authenticate graphman GraphQL requests.

Other environment variables that can be set:

- `GRAPHMAN_PORT` - The port for the graphman GraphQL server (Defaults to `8050`)
- `GRAPHMAN_SERVER_QUERY_MAX_DEPTH` - The maximum query depth for the graphman GraphQL server (Defaults to `255`)
- `GRAPHMAN_SERVER_QUERY_MAX_COMPLEXITY` - The maximum query complexity for the graphman GraphQL server (Defaults
  to `1,000,000`)

## GraphQL playground

When the graphman GraphQL server is running the GraphQL playground is available at the following
address: http://127.0.0.1:8050

**Note:** The port might be different.

Please make sure to set the authorization header to be able to use the playground:

```json
{
  "Authorization": "Bearer GRAPHMAN_SERVER_AUTH_TOKEN"
}
```

**Note:** There is a headers section at the bottom of the playground page.

## Supported commands

The playground is the best place to see the full schema, the latest available queries and mutations, and their
documentation. Below, we will briefly describe some supported commands and example queries.

At the time of writing, the following graphman commands are available via the GraphQL API:

### Deployment Info

Returns the available information about one, multiple, or all deployments.

**Example query:**

```text
query {
    deployment {
        info(deployment: { ipfs { hash: "Qm..." } }) {
            status {
                isPaused
            }
        }
    }
}
```

**Example response:**

```json
{
  "data": {
    "deployment": {
      "info": [
        {
          "status": {
            "isPaused": false
          }
        }
      ]
    }
  }
}
```

### Pause Deployment

Pauses a deployment that is not already paused.

**Example query:**

```text
mutation {
    deployment {
        pause(deployment: { ipfs { hash: "Qm..." } })
    }
}
```

**Example response:**

```json
{
  "data": {
    "deployment": {
      "pause": true
    }
  }
}
```

### Resume Deployment

Resumes a deployment that has been previously paused.

**Example query:**

```text
mutation {
    deployment {
        resume(deployment: { ipfs { hash: "Qm..." } })
    }
}
```

**Example response:**

```json
{
  "data": {
    "deployment": {
      "resume": true
    }
  }
}
```

### Restart Deployment

Pauses a deployment and resumes it after a delay.

**Example query:**

```text
mutation {
    deployment {
        restart(deployment: { ipfs { hash: "Qm..." } })
    }
}
```

**Example response:**

```json
{
  "data": {
    "deployment": {
      "restart": "UNIQUE_EXECUTION_ID"
    }
  }
}
```

This is a long-running command because the default delay before resuming the deployment is 20 seconds. For long-running
commands, the GraphQL API will return a unique execution ID, which is a UUID.

The UUID can be used to query the execution status and the output of the command:

```text
query {
  execution {
      info(id: "UNIQUE_EXECUTION_ID") {
          status
          errorMessage
          commandOutput {
              ... on RestartDeployment {
                  success
              }
          }
      }
  }
}
```

**Example response when execution is in-progress:**

```json
{
  "data": {
    "execution": {
      "info": {
        "status": "IN_PROGRESS",
        "errorMessage": null,
        "commandOutput": null
      }
    }
  }
}
```

**Example response when execution is completed:**

```json
{
  "data": {
    "execution": {
      "info": {
        "status": "SUCCEEDED",
        "errorMessage": null,
        "commandOutput": {
          "success": true
        }
      }
    }
  }
}
```

## Other commands

GraphQL support for other graphman commands will be added over time, so please make sure to check the GraphQL playground
for the full schema and the latest available queries and mutations.
