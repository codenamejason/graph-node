# Extending graphman GraphQL API

This is a step-by-step guide on how to extend the graphman GraphQL API with new commands. To make the guide easier to
understand, we will implement an abstract command that will return the chain version.

**Table of contents:**

1. [Define the command](#1-define-the-command)
2. [Extend `CommandKind` type](#2-extend-commandkind-type)
3. [**OPTIONAL** Create new extensions](#3-optional-create-new-extensions)
4. [**OPTIONAL** Create new GraphQL types](#4-optional-create-new-graphql-types)
5. [Create a new GraphQL resolver](#5-create-a-new-graphql-resolver)
6. [Extend GraphQL `CommandOutput` type](#6-extend-graphql-commandoutput-type)
7. [Test the new GraphQL query](#7-test-the-new-graphql-query)
8. [Extend the graphman CLI with the command](#8-extend-the-graphman-cli-with-the-command)
9. [A note on long-running commands in the CLI](#9-a-note-on-long-running-commands-in-the-cli)
10. [A note on naming conventions](#10-a-note-on-naming-conventions)
11. [A note on similar types](#11-a-note-on-similar-types)

## 1. Define the command

The code for graphman commands, which is used by the GraphQL API and the CLI, is located in
the [`core/graphman`][graphman] crate.

Every command has its own module, and related commands are grouped together. For example, commands related to
deployments are located in the [`commands/deployment`][deployment] module, and the deployment info command is located
in the [`commands/deployment/info.rs`][deployment-info] module. This pattern should be used for all the commands to
make it easy to understand which commands are available and how they are related. All the utilities and functionality
that are shared by multiple commands should be kept outside [`commands`][commands] modules.

We will start by defining the new command:

```rust
// $PROJECT_ROOT/core/graphman/src/commands/chain/version.rs

use graphman_primitives::BoxedFuture;
use graphman_primitives::ExtensibleGraphmanCommand;
use graphman_primitives::GraphmanCommand;

use crate::GraphmanContext;
use crate::GraphmanError;

#[derive(Clone, Debug)]
pub struct ChainVersionCommand {
    pub chain_id: String,
}

impl<Ctx> GraphmanCommand<Ctx> for ChainVersionCommand
where
    Ctx: AsRef<GraphmanContext> + Send + 'static,
{
    type Output = String;
    type Error = GraphmanError;
    type Future = BoxedFuture<Self::Output, Self::Error>;

    fn execute(self, ctx: Ctx) -> Self::Future {
        Box::pin(async move {
            let Self { chain_id } = self;
            let ctx = ctx.as_ref();

            // We will skip the part where `chain_id` and `ctx` is used to determine the real version ...

            Ok("v0.0.1".to_owned())
        })
    }
}

impl ExtensibleGraphmanCommand for ChainVersionCommand {}
```

Now, let's explain the command module:

- By convention, every command is a struct, and the type name contains a `*Command` suffix. The fields describe all the
  accepted arguments. **Please note that the fields do not contain a database connection or a store, as they are meant
  to be provided in the context**. The meaning of the command struct is to describe a command, not the things it needs
  to execute properly.
- The [`core/graphman_primitives`][graphman-primitives] crate contains the base types and traits used by all the
  commands and extensions, and allows for the creation of a flexible and extensible way of executing commands.
- Implementing the [`GraphmanCommand`][command] trait is required for every command. That allows commands to be wrapped
  by other types that can add additional features to the command executions.
- The context is generic because if a command is wrapped by an extension, the extension is free to have its own context.
  The only requirement is that when a context reaches the command, it should provide a reference to the specific context
  expected by the command. Again, this adds a lot of flexibility.
- Implementing the [`ExtensibleGraphmanCommand`][extensible-command] trait is optional, but it adds support for
  intuitive layering, which is extremely useful. Intuitive layering will be explained later.

## 2. Extend `CommandKind` type

The [`CommandKind`][command-kind] enum contains a list of all supported commands, so every time a new command is added,
it is expected that a new variant is added to the enum.

In our case, we will extend the enum as follows:

```rust
// $PROJECT_ROOT/core/graphman/src/kind.rs

pub enum CommandKind {
    // Other variants ...

    ChainVersion,
}
```

_Please note that there is no need to add the `*Command` suffix._

This change will probably break the compilation because the new enum variant should also be added to the GraphQL
presentation type [here][graphql-command-kind].

At this point, the [`server/graphman`][graphman-server] crate will still not compile, but we will get to that later.

## 3. **OPTIONAL** Create new extensions

In graphman, extensions are a powerful concept that allow reusing commands and mixing features when executing commands
in different environments. For example, the GraphQL API has different execution requirements when compared to the CLI.
One big difference is the execution of long-running commands. The CLI can just wait for a long-running command to
complete its execution, while the GraphQL API is expected to return a result within a second. With extensions, this
difference in execution requirements can be expressed with just one line of code.

At the time of writing, the following extensions are available:

- [`IdentifyCommand`][identify-command] - Makes a command identifiable by assigning it a kind and a unique command
  execution ID.
- [`HandleBrokenExecutions`][handle-broken-executions] - Marks command executions as failed if they did not receive any
  updates for the specified duration.
- [`PreventDuplicateExecutions`][prevent-duplicate-executions] - Fails the command execution if there are other
  executions in progress of the same kind.
- [`ExecuteInBackground`][execute-in-background] - Executes a command in the background and returns the execution ID as
  the output.
- [`TrackExecution`][track-execution] - Makes a command report its execution status, and stores the execution output or
  the error message in the persistent storage.

Extensions can indirectly depend one on another, by expecting some data in the context, but that is not a strict rule.
On the other hand, the order in which extensions are executed is important, and depending on the order, some features
may entirely change their behavior.

For example, running [`ExecuteInBackground`][execute-in-background] and then [`TrackExecution`][track-execution] will
ensure that when a command is executed in the background, its execution is tracked and the output will be recorded in
the database. But if we change the order so that [`TrackExecution`][track-execution] is executed first and
then [`ExecuteInBackground`][execute-in-background], the tracker will immediately report that the command completed its
execution, which is probably not the expected behavior.

To make it easier to understand the order of execution of command extensions, [`IntuitiveLayering`][intuitive-layering]
was introduced. Its goal is to execute the extensions in the order they were applied.

For example, given the following code:

```text
let command = some_command
  .layer(identify_command)
  .layer(handle_broken_executions)
  .layer(prevent_duplicate_executions)
  .layer(execute_in_background)
  .layer(track_execution)
  .execute(ctx)
  .await;
```

The order of execution of the extensions and the command is the following:

```text
1. identify_command
2. handle_broken_executions
3. prevent_duplicate_executions
4. execute_in_background
5. track_execution
6. some_command
```

Intuitive layering is automatically implemented for commands that implement
the [`ExtensibleGraphmanCommand`][extensible-command] trait.

When a command requires some features that are not currently provided by existing extensions, new extensions can be
created, and the process is similar to how commands are created. Please visit
the [`core/graphman_extensions`][graphman-extensions] crate for
more details.

## 4. **OPTIONAL** Create new GraphQL types

All the GraphQL types are created in the [`server/graphman/src/entities`][entities] module. Our command is simple enough
and does not need a custom GraphQL type, but for exercise, we will create a custom presentation type in Rust:

```rust
// $PROJECT_ROOT/server/graphman/src/entities/chain_version.rs

use async_graphql::SimpleObject;

#[derive(Clone, Debug, SimpleObject)]
pub struct ChainVersion {
    pub version: String,
}
```

This will generate the following GraphQL type:

```graphql
type ChainVersion {
    version: String!
}
```

We are using the [`async-graphql`][async-graphql] crate to create the GraphQL schema in Rust and to execute queries, so
please consult the crate [documentation][async-graphql-docs] for more details and available options.

## 5. Create a new GraphQL resolver

All the GraphQL resolvers are defined in the [`server/graphman/src/resolvers`][resolvers] module. We need to create a
GraphQL resolver to make the command available in the API. In our case, the command does not perform any mutations, so
we define the following query resolver:

```rust
// $PROJECT_ROOT/server/graphman/src/resolvers/chain_query.rs

use async_graphql::Context;
use async_graphql::Object;
use async_graphql::Result;
use graphman::commands::chain::version::ChainVersionCommand;
use graphman_primitives::GraphmanCommand;

use crate::entities::chain_version::ChainVersion;
use crate::resolvers::context::make_graphman_context;

pub struct ChainQuery;

#[Object]
impl ChainQuery {
    pub async fn version(&self, ctx: &Context<'_>, chain_id: String) -> Result<ChainVersion> {
        let command = ChainVersionCommand { chain_id };
        let ctx = make_graphman_context(ctx)?;
        let version = command.execute(ctx).await?;

        Ok(ChainVersion { version })
    }
}
```

In our case, the command is simple enough, so we do not need additional features to execute it, but if we really want to
add some features, like tracking executions, the resolver would look like the following:

```rust
// $PROJECT_ROOT/server/graphman/src/resolvers/chain_query.rs

use std::sync::Arc;

use async_graphql::Context;
use async_graphql::Object;
use async_graphql::Result;
use graph_store_postgres::graphman_store::GraphmanStore;
use graphman::commands::chain::version::ChainVersionCommand;
use graphman::CommandKind;
use graphman_extensions::GraphmanExtensionContext;
use graphman_extensions::IdentifyCommand;
use graphman_extensions::TrackExecution;
use graphman_primitives::GraphmanCommand;
use graphman_primitives::GraphmanLayer;

use crate::entities::chain_version::ChainVersion;
use crate::resolvers::context::make_graphman_context;

pub struct ChainQuery;

#[Object]
impl ChainQuery {
    pub async fn version(&self, ctx: &Context<'_>, chain_id: String) -> Result<ChainVersion> {
        let command = ChainVersionCommand { chain_id };
        let store = ctx.data::<Arc<GraphmanStore>>()?.to_owned();
        let ctx = GraphmanExtensionContext::new(make_graphman_context(ctx)?);
        let version = command
            .layer(IdentifyCommand::new(CommandKind::ChainVersion.into()))
            .layer(TrackExecution::new(store))
            .execute(ctx)
            .await?;

        Ok(ChainVersion { version })
    }
}
```

Next, we need to add the new resolver to the [`QueryRoot`][query-root]:

```rust
// $PROJECT_ROOT/server/graphman/src/resolvers/query_root.rs

// QueryRoot definition ...

#[Object]
impl QueryRoot {
    // Other resolvers ...

    pub async fn chain(&self) -> ChainQuery {
        ChainQuery {}
    }
}
```

That's it. Now, when we will run the graphman GraphQL server, we will be able to query the chain version with the
following query:

```text
query {
  chain {
    version(chainId: "chain-id") {
      version
    }
  }
}
```

**Note:** There is a separate guide on how to start the graphman GraphQL server.

## 6. Extend GraphQL `CommandOutput` type

The graphman GraphQL API makes it possible to query details about command executions by their IDs. The primary goal of
this feature is to make it possible to get the execution status and the output of long-running commands.

The execution details that are available via the GraphQL API are defined in
the [`server/graphman/src/entities/execution.rs`][execution] module. This module also contains the `CommandOutput` type,
a Rust enum that represents a GraphQL union of all possible command outputs.

While not every command will store its execution data and make it available by ID, we should not introduce this
limitation in the API. If, for any reason, a command execution will be delayed or sent to the background, the API should
be ready to allow the users to query the execution details for that command execution. To make this possible, every time
a new command is created, the [`CommandOutput`][execution] should be extended with the output type of the new command.

In our case, the process is the following:

- Extend the [`CommandOutput`][execution] enum:
  ```rust
  // $PROJECT_ROOT/server/graphman/src/entities/execution.rs
  
  // Other imports ...
  
  use crate::entities::chain_version::ChainVersion;
  
  // Other types ...
  
  pub enum CommandOutput {
    // Other outputs ...
    
    ChainVersion(ChainVersion),
  }
  
  // Other code ...
  ```
- Extend the [`parse_command_output`][execution] function:

  ```rust
  // $PROJECT_ROOT/server/graphman/src/entities/execution.rs
  
  // Other code ...
  
  fn parse_command_output(kind: CommandKind, value: serde_json::Value) -> Result<CommandOutput> {
      // Other code ...
  
      let parsed = match kind {
          // Other match patterns ...
  
          CommandKind::ChainVersion => {
              CommandOutput::ChainVersion(ChainVersion {
                  version: serde_json::from_value(value)?
              })
          }
      };
  
      Ok(parsed)
  }
  
  // Other code ...
  ```

One important note here is that the output of every command should implement `serde::Serialize`
and `serde::Deserialize`.

## 7. Test the new GraphQL query

To make sure that the graphman GraphQL API works as expected and is reliable, extensive testing is required for every
command.

Graphman GraphQL tests are located in the [`server/graphman/tests`][tests] directory.

Now, let's create a test for our command:

```rust
// $PROJECT_ROOT/server/graphman/tests/chain_query.rs

pub mod util;

use serde_json::json;

use self::util::client::send_graphql_request;
use self::util::run_test;
use self::util::server::VALID_TOKEN;

#[test]
fn graphql_returns_chain_version() {
    run_test(|| async {
        // We will skip the code that should make sure that the chain exists ...

        let resp = send_graphql_request(
            json!({
                "query": r#"{
                    chain {
                        version(chainId: "chain-id") {
                            version
                        }
                    }
                }"#
            }),
            VALID_TOKEN,
        ).await;

        let expected_resp = json!({
            "data": {
                "chain": {
                    "version": {
                        "version": "v0.0.1"
                    }
                }
            }
        });

        assert_eq!(resp, expected_resp);
    });
}
```

The tests require a real database connection, so please read the store [documentation][test-docs] to make sure
everything is ready for running tests.

## 8. Extend the graphman CLI with the command

Usually, the graphman GraphQL API will be extended with commands that already exist in the CLI, so the process will be
the following:

- Extract the command functionality from the CLI to the core crate
- Extend the GraphQL API
- Reintegrate the extracted command back to the CLI

The last step is very similar to what we did when we created a new GraphQL resolver. The command should be called in
the [`node/src/bin/manager`][manager] binary where appropriate, and that's it.

## 9. A note on long-running commands in the CLI

An essential feature of the graphman GraphQL API is the ability to execute commands in the background. Since the
extensions can be easily shared between the API and the CLI, it might seem reasonable to think that
the [`ExecuteInBackground`][execute-in-background] extension could be used in the CLI, but that's not the case.

The current implementation is pretty basic and relies on tokio tasks, and this works because the `graph-node` is a
long-running application. Contrary to this, if we try to spawn a tokio task in the CLI and the process finishes
execution, the task will never be completed.

So, at this time, the [`ExecuteInBackground`][execute-in-background] does not support the CLI.

## 10. A note on naming conventions

The naming conventions for the graphman-related crates are the following:

- All trait names are prefixed with `Graphman*`
- Type names are prefixed with `Graphman*` only when they are short or may be confusing without the prefix
- All commands have the `*Command` suffix

## 11. A note on similar types

The graphman-related crates introduce a bit of redundancy in some types, and this note attempts to explain the “why”.

At least in graphman-related crates, there are internal types that pass the useful information around, there are
database models, and there are presentation types. Every so often, a type might fall into all categories at once. In
graphman, in such cases, separate types are created for every category, even if the types look similar. This is done
because they serve different purposes, have different reasons for changes, and have different limitations, and usually,
it feels wrong to add, for example, GraphQL type limitations to an internal core type or to leak knowledge about
Postgres to a presentation type.

[graphman]: ../core/graphman

[commands]: ../core/graphman/src/commands

[deployment]: ../core/graphman/src/commands/deployment

[deployment-info]: ../core/graphman/src/commands/deployment/info.rs

[command-kind]: ../core/graphman/src/kind.rs


[graphman-extensions]: ../core/graphman_extensions

[identify-command]: ../core/graphman_extensions/src/identify_command.rs

[handle-broken-executions]: ../core/graphman_extensions/src/handle_broken_executions.rs

[prevent-duplicate-executions]: ../core/graphman_extensions/src/prevent_duplicate_executions.rs

[execute-in-background]: ../core/graphman_extensions/src/execute_in_background.rs

[track-execution]: ../core/graphman_extensions/src/track_execution.rs


[graphman-primitives]: ../core/graphman_primitives

[command]: ../core/graphman_primitives/src/command.rs

[extensible-command]: ../core/graphman_primitives/src/extensible_command.rs

[intuitive-layering]: ../core/graphman_primitives/src/intuitive_layering.rs


[graphman-server]: ../server/graphman

[entities]: ../server/graphman/src/entities

[graphql-command-kind]: ../server/graphman/src/entities/command_kind.rs

[execution]: ../server/graphman/src/entities/execution.rs

[resolvers]: ../server/graphman/src/resolvers

[query-root]: ../server/graphman/src/resolvers/query_root.rs

[tests]: ../server/graphman/tests


[manager]: ../node/src/bin/manager.rs


[test-docs]: ../store/test-store/README.md


[async-graphql]: https://crates.io/crates/async-graphql

[async-graphql-docs]: https://async-graphql.github.io/async-graphql/en/index.html
