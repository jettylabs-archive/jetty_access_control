# Jetty CLI

Jetty primary UI is via the `jetty` CLI. The CLI provides commands and tools designed to make managing access control easy.

:::tip
You can set the logging verbosity for the Jetty CLI using the the `-l` or `--log-level` option specifying a minimum level to be shown. The options are `debug`, `info`, `warn`, or `error`. The default value is `info` which is appropriate for most users.
:::

The available subcommands are:

-   **[new](./new)** - Launch a guided flow to create a Jetty project in a new directory
-   **[add](./add)** - Add connectors to an existing Jetty project via a similar flow to `jetty new`
-   **[bootstrap](./bootstrap)** - Fetch and build out initial configuration files for a project
-   **[fetch](./fetch)** - Fetch metadata for an existing Jetty project
-   **[dev](./dev)** - Watch config files for changes and update the YAML schema as needed to keep validation working properly. It's recommended that you run this while editing configuration files
-   **[rename](./rename)** -Rename a group or user across all the configuration files
-   **[remove](./remove)** - Remove references to a group or user across all the configuration files
-   **[diff](./diff)** - Diff the configuration and the current state of your environment
-   **[plan](./plan)** - Plan the changes needed to update the environment based on the diff
-   **[apply](./apply)** - Update the environment with the planned changes
-   **[explore](./explore)** - Launch the exploration web UI
-   **[subgraph](./subgraph)** - Get the [dot](https://graphviz.org/doc/info/lang.html) representation of a subgraph
